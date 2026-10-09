#include "audio_internal.h"
#include <math.h>

void audio_close(Audio *a) {
    if (!a) return;
    output_close(a->output);
    swr_free(&a->resampler);
    av_frame_free(&a->frame);
    av_frame_free(&a->pcm);
    av_packet_free(&a->packet);
    avcodec_free_context(&a->decoder);
    free(a);
}

int audio_open(Media *m) {
    if (m->audio) return 1;
    if (m->reader) return AVERROR(EBUSY);
    const AVCodec *codec;
    int rc = av_find_best_stream(m->file, AVMEDIA_TYPE_AUDIO, -1, m->stream, &codec, 0);
    if (rc == AVERROR_STREAM_NOT_FOUND) return 0;
    if (rc < 0) return rc;
    m->audio_stream = rc;
    Audio *a = m->audio = calloc(1, sizeof(*a));
    if (!a) return AVERROR(ENOMEM);
    a->decoder = avcodec_alloc_context3(codec);
    a->frame = av_frame_alloc();
    a->pcm = av_frame_alloc();
    a->packet = av_packet_alloc();
    if (!a->decoder || !a->frame || !a->pcm || !a->packet) return AVERROR(ENOMEM);
    AVStream *stream = m->file->streams[m->audio_stream];
    a->time_base = av_q2d(stream->time_base);
    rc = avcodec_parameters_to_context(a->decoder, stream->codecpar);
    if (rc < 0) return rc;
    a->decoder->pkt_timebase = stream->time_base;
    rc = avcodec_open2(a->decoder, codec, NULL);
    if (rc < 0) return rc;
    a->output = output_open();
    if (!a->output) return AVERROR_EXTERNAL;
    a->next_pts = stream->start_time == AV_NOPTS_VALUE ? 0 :
                  stream->start_time * av_q2d(stream->time_base) - m->start;
    a->end = stream->duration == AV_NOPTS_VALUE ? INFINITY :
             a->next_pts + stream->duration * av_q2d(stream->time_base);
    return 1;
}

int audio_reset(Audio *a, double target) {
    if (!a) return 0;
    avcodec_flush_buffers(a->decoder);
    swr_free(&a->resampler);
    av_frame_unref(a->frame);
    av_frame_unref(a->pcm);
    av_packet_unref(a->packet);
    a->draining = a->ended = a->offset = 0;
    a->seeking = 1;
    a->written = 0;
    a->target = a->next_pts = target;
    output_reset(a->output, target);
    int rc;
    while ((rc = output_poll(a->output, 1)) == 0 && a->output->reset) av_usleep(1000);
    return rc < 0 ? rc : 0;
}

double audio_preroll(Audio *a) {
    if (!a || a->decoder->sample_rate <= 0) return 0;
    return fmax(a->decoder->seek_preroll, a->decoder->frame_size) / a->decoder->sample_rate;
}

static int write_pcm(Audio *a) {
    int64_t gap = a->frame_start + a->offset - a->written;
    if (gap < 0) {
        int skip = (int)fmin(-gap, a->pcm->nb_samples - a->offset);
        a->offset += skip;
    }
    int remaining = a->pcm->nb_samples - a->offset;
    if (!remaining) return 0;
    static const float silence[2048] = {0};
    const float *samples = (const float *)a->pcm->data[0] + a->offset * AUDIO_CHANNELS;
    int count = remaining;
    if (gap > 0) { samples = silence; count = (int)fmin(gap, 1024); }
    int rc = output_write(a->output, samples, count);
    if (rc > 0) {
        a->written += rc;
        if (gap <= 0) a->offset += rc;
    }
    return rc;
}

int audio_tick(Media *m, int paused, double *clock, double *limit, int *done) {
    Audio *a = m->audio;
    *clock = -1;
    *limit = 0;
    *done = !a;
    if (!a) return 0;
    int rc = output_poll(a->output, paused);
    if (rc < 0) return rc;
    int progress = 0;
    if (rc > 0) {
        if (a->offset < a->pcm->nb_samples) {
            rc = write_pcm(a);
            if (rc < 0) return rc;
            progress = rc > 0;
        } else if (!a->ended) {
            rc = audio_decode(m);
            if (rc < 0) return rc;
            progress = rc;
        } else {
            rc = output_finish(a->output);
            if (rc < 0) return rc;
            *done = rc;
        }
    }
    rc = output_time(a->output, clock, limit);
    return rc < 0 ? rc : progress;
}

int media_audio(void *handle, int paused, double *clock, double *limit, int *done, char *error, size_t capacity) {
    Media *m = handle;
    int64_t started = media_trace_start(m);
    int rc = audio_tick(m, paused, clock, limit, done);
    media_trace_end(started, &m->trace.audio_us);
    if (rc >= 0) return rc;
    if (rc == AVERROR_EXTERNAL && m->audio && m->audio->output) {
        snprintf(error, capacity, "system audio: %s", pa_strerror(pa_context_errno(m->audio->output->context)));
        return -1;
    }
    return media_error(error, capacity, "decode/output audio", rc);
}

int media_enable_audio(void *handle, char *error, size_t capacity) {
    int rc = audio_open(handle);
    return rc < 0 ? media_error(error, capacity, "open audio", rc) : rc;
}
