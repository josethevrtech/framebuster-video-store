#include "audio_internal.h"
#include <math.h>

static int convert(Audio *a, AVFrame *input, double pts) {
    av_frame_unref(a->pcm);
    a->pcm->format = AV_SAMPLE_FMT_FLT;
    a->pcm->sample_rate = AUDIO_RATE;
    a->pcm->ch_layout = (AVChannelLayout)AV_CHANNEL_LAYOUT_STEREO;
    if (!a->resampler) {
        if (!input) { a->ended = 1; return 1; }
        a->resampler = swr_alloc();
        if (!a->resampler) return AVERROR(ENOMEM);
        int rc = swr_config_frame(a->resampler, a->pcm, input);
        if (rc < 0) return rc;
        rc = swr_init(a->resampler);
        if (rc < 0) return rc;
        a->input_rate = input->sample_rate;
    }
    int64_t base = (int64_t)a->input_rate * AUDIO_RATE;
    double delay = swr_get_delay(a->resampler, base) / (double)base;
    int count = input ? input->nb_samples : 0;
    int capacity = swr_get_out_samples(a->resampler, count);
    if (capacity < 0) return capacity;
    if (capacity > 2 * AUDIO_RATE) return AVERROR(ENOBUFS);
    a->pcm->nb_samples = capacity;
    int rc = swr_convert_frame(a->resampler, a->pcm, input);
    if (rc < 0) return rc;
    double start = (pts - delay - a->target) * AUDIO_RATE;
    if (!isfinite(start) || start <= INT64_MIN || start >= (double)INT64_MAX)
        return AVERROR_INVALIDDATA;
    a->frame_start = llround(start);
    double remaining = (a->end - a->target) * AUDIO_RATE - a->frame_start;
    if (remaining < a->pcm->nb_samples) a->pcm->nb_samples = (int)fmax(0, round(remaining));
    a->offset = 0;
    if (!input && !a->pcm->nb_samples) a->ended = 1;
    return 1;
}

int audio_decode(Media *m) {
    Audio *a = m->audio;
    av_frame_unref(a->frame);
    int rc = avcodec_receive_frame(a->decoder, a->frame);
    if (rc == AVERROR_EOF) return convert(a, NULL, a->next_pts);
    if (rc == AVERROR(EAGAIN)) {
        rc = media_packet(m, 1, a->packet);
        if (rc == AVERROR(EAGAIN)) return 0;
        if (rc < 0 && rc != AVERROR_EOF) return rc;
        if (a->draining) return AVERROR_BUG;
        a->draining = rc == AVERROR_EOF;
        rc = avcodec_send_packet(a->decoder, a->draining ? NULL : a->packet);
        av_packet_unref(a->packet);
        return rc < 0 ? rc : 1;
    }
    if (rc < 0) return rc;
    AVFrame *frame = a->frame;
    if (frame->decode_error_flags || frame->flags & AV_FRAME_FLAG_CORRUPT || frame->sample_rate <= 0)
        return AVERROR_INVALIDDATA;
    if (a->seeking && frame->best_effort_timestamp == AV_NOPTS_VALUE) return AVERROR_INVALIDDATA;
    double pts = frame->best_effort_timestamp == AV_NOPTS_VALUE ? a->next_pts :
                 frame->best_effort_timestamp * a->time_base - m->start;
    double tolerance = 2 * a->time_base + 1.0 / frame->sample_rate;
    if (a->resampler && fabs(pts - a->next_pts) <= tolerance)
        pts = a->next_pts;
    a->next_pts = pts + frame->nb_samples / (double)frame->sample_rate;
    if (pts >= a->target) a->seeking = 0;
    return convert(a, frame, pts);
}
