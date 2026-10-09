#include "audio_internal.h"
#include <libavutil/avassert.h>
#include <math.h>

extern AudioOutput *test_output;
void audio_assert_paused(void);

static void tick(Media *m, int paused, double *clock, int *done) {
    double limit;
    int rc = audio_tick(m, paused, clock, &limit, done);
    if (rc < 0) {
        char error[512];
        media_error(error, sizeof(error), "audio test", rc);
        fprintf(stderr, "%s\n", error);
    }
    av_assert0(rc >= 0);
    av_assert0(limit >= *clock && limit <= m->audio->output->end);
    if (m->audio->output->corked) av_assert0(limit == *clock);
    rc = media_packet(m, 0, m->packet);
    av_assert0(rc >= 0 || rc == AVERROR(EAGAIN) || rc == AVERROR_EOF);
}

static double play_to(Media *m, double target) {
    int done = 0;
    double clock = 0;
    int64_t deadline = av_gettime_relative() + 10000000;
    while (clock < target && !done) {
        av_assert0(av_gettime_relative() < deadline);
        tick(m, 0, &clock, &done);
        av_usleep(1000);
    }
    return clock;
}

static void seek_to(Media *m, double target) {
    media_reader_pause(m, 1);
    av_assert0(audio_reset(m->audio, target) == 0);
    audio_assert_paused();
    int64_t timestamp = llround((target + m->start) / av_q2d(m->file->streams[m->stream]->time_base));
    av_assert0(media_seek_file(m, timestamp) >= 0);
    media_reader_pause(m, 0);
}

static void compare_pcm(Media *m, const char *reference) {
    FILE *file = fopen(reference, "rb");
    av_assert0(file);
    int64_t last = 0;
    float peak_error = 0;
    while (!m->audio->ended) {
        Audio *a = m->audio;
        av_assert0(audio_decode(m) >= 0);
        if (!a->offset && a->pcm->nb_samples) {
            int first = a->frame_start < 0 ? -a->frame_start : 0;
            int64_t start = a->frame_start + first;
            if (first < a->pcm->nb_samples) {
                av_assert0(llabs(start - last) <= 1);
                int64_t reference_start = start + llround(a->target * AUDIO_RATE);
                av_assert0(fseek(file, reference_start * AUDIO_FRAME_BYTES, SEEK_SET) == 0);
                float *samples = (float *)a->pcm->data[0];
                for (int i = first * AUDIO_CHANNELS; i < a->pcm->nb_samples * AUDIO_CHANNELS; i++) {
                    float expected;
                    if (fread(&expected, sizeof(expected), 1, file) != 1) {
                        fprintf(stderr, "PCM exceeds reference: start=%lld samples=%d index=%d\n",
                                (long long)a->frame_start, a->pcm->nb_samples, i);
                        abort();
                    }
                    peak_error = fmaxf(peak_error, fabsf(samples[i] - expected));
                }
                last = a->frame_start + a->pcm->nb_samples;
            }
            a->offset = a->pcm->nb_samples;
        }
        int rc = media_packet(m, 0, m->packet);
        av_assert0(rc >= 0 || rc == AVERROR(EAGAIN) || rc == AVERROR_EOF);
    }
    av_assert0(last > 0);
    av_assert0(fgetc(file) == EOF);
    fclose(file);
    printf("PCM reference: target=%.3f samples=%lld peak_error=%.8f\n",
           m->audio->target, (long long)last, peak_error);
    av_assert0(peak_error < 0.00001f);
}

int main(int argc, char **argv) {
    setvbuf(stdout, NULL, _IONBF, 0);
    av_assert0(argc == 2 || argc == 3);
    Media m = {0};
    av_assert0(avformat_open_input(&m.file, argv[1], NULL, NULL) >= 0);
    av_assert0(avformat_find_stream_info(m.file, NULL) >= 0);
    m.stream = av_find_best_stream(m.file, AVMEDIA_TYPE_VIDEO, -1, -1, NULL, 0);
    av_assert0(m.stream >= 0);
    AVStream *video = m.file->streams[m.stream];
    m.start = video->start_time == AV_NOPTS_VALUE ? 0 : video->start_time * av_q2d(video->time_base);
    m.packet = av_packet_alloc();
    av_assert0(m.packet);
    if (argc == 2) {
        av_assert0(audio_open(&m) == 0);
        double clock, limit;
        int done;
        av_assert0(audio_tick(&m, 0, &clock, &limit, &done) == 0 && done && clock == -1);
        printf("Silent file needs no audio server\n");
        goto finished;
    }
    av_assert0(audio_open(&m) == 1);
    test_output = m.audio->output;
    compare_pcm(&m, argv[2]);
    if (m.audio->decoder->codec_id == AV_CODEC_ID_FLAC) {
        seek_to(&m, 0.25);
        compare_pcm(&m, argv[2]);
    }
    seek_to(&m, 0);
    double clock, limit;
    int done;
    for (int i = 0; i < 400; i++) { tick(&m, 1, &clock, &done); av_usleep(1000); }
    av_assert0(clock == 0 && !done);
    av_assert0(play_to(&m, 0.3) >= 0.3);
    av_usleep(600000);
    av_assert0(output_poll(m.audio->output, 0) >= 0);
    av_assert0(output_time(m.audio->output, &clock, &limit) == 0);
    av_assert0(clock == limit && limit == m.audio->output->end);
    printf("Starved audio clock stopped at %.6f\n", clock);
    for (int i = 0; i < 100; i++) { tick(&m, 1, &clock, &done); av_usleep(1000); }
    double paused = clock;
    for (int i = 0; i < 200; i++) { tick(&m, 1, &clock, &done); av_usleep(1000); }
    av_assert0(fabs(clock - paused) < 0.001);
    printf("Pause held audio clock at %.6f\n", clock);
    for (int i = 0; i < 4; i++) {
        double target = i % 2 ? 0.1 : 1.0;
        seek_to(&m, target);
        for (int j = 0; j < 200; j++) { tick(&m, 1, &clock, &done); av_usleep(1000); }
        av_assert0(clock == target && !done);
        av_assert0(play_to(&m, target + 0.1) >= target + 0.1);
        printf("Seek/resume: target=%.3f clock=%.6f\n", target, m.audio->output->clock);
    }
    double end = play_to(&m, 20);
    av_assert0(m.audio->output->drained == 1 && end > 1);
    seek_to(&m, 0.25);
    av_assert0(play_to(&m, 0.35) >= 0.35);
    printf("EOF drained at %.6f; seek after EOF succeeded\n", end);
    seek_to(&m, 3);
    int64_t deadline = av_gettime_relative() + 5000000;
    do {
        av_assert0(av_gettime_relative() < deadline);
        tick(&m, 1, &clock, &done);
        av_usleep(1000);
    } while (!done);
    av_assert0(clock == 3);
    printf("Paused seek beyond EOF completed without starting output\n");
    pa_context_disconnect(m.audio->output->context);
    av_assert0(audio_tick(&m, 0, &clock, &limit, &done) < 0);
    printf("Disconnected audio server returns an error\n");
finished:
    media_stop(&m);
    av_assert0(!m.audio);
    av_packet_free(&m.packet);
    avformat_close_input(&m.file);
    return 0;
}
