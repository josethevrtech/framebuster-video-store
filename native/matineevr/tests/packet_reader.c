#include "media_reader.h"
#include <libavutil/avassert.h>

static atomic_int gated, reading, calls, failure, paused;
int __real_av_read_frame(AVFormatContext *file, AVPacket *packet);

int __wrap_av_read_frame(AVFormatContext *file, AVPacket *packet) {
    atomic_store(&reading, 1);
    atomic_fetch_add(&calls, 1);
    while (atomic_load(&gated)) av_usleep(1000);
    int error = atomic_exchange(&failure, 0);
    int rc = error ? error : __real_av_read_frame(file, packet);
    atomic_store(&reading, 0);
    return rc;
}

void test_packet_limits(Media *m);

static void wait_for(atomic_int *value) {
    int64_t deadline = av_gettime_relative() + 5000000;
    while (!atomic_load(value)) {
        av_assert0(av_gettime_relative() < deadline);
        av_usleep(1000);
    }
}

static uint64_t digest(const AVPacket *p) {
    uint64_t value = p->pts;
    for (int i = 0; i < p->size; i++) value = value * 31 + p->data[i];
    return value;
}

static Media open_file(const char *path) {
    Media m = {0};
    av_assert0(avformat_open_input(&m.file, path, NULL, NULL) == 0);
    av_assert0(avformat_find_stream_info(m.file, NULL) >= 0);
    m.stream = av_find_best_stream(m.file, AVMEDIA_TYPE_VIDEO, -1, -1, NULL, 0);
    m.audio_stream = av_find_best_stream(m.file, AVMEDIA_TYPE_AUDIO, -1, -1, NULL, 0);
    m.packet = av_packet_alloc();
    av_assert0(m.stream >= 0 && m.packet);
    return m;
}

static void rewind_file(Media *m) {
    media_reader_pause(m, 1);
    media_clear_packets(m, AV_NOPTS_VALUE);
    av_assert0(av_seek_frame(m->file, m->stream, 0, AVSEEK_FLAG_BACKWARD) >= 0);
    media_reader_pause(m, 0);
}

static void *pause_reader(void *handle) {
    media_reader_pause(handle, 1);
    atomic_store(&paused, 1);
    return NULL;
}

static void *stop_reader(void *handle) {
    media_reader_close(handle);
    atomic_store(&paused, 1);
    return NULL;
}

int main(int argc, char **argv) {
    av_assert0(argc == 2);
    Media m = open_file(argv[1]);
    uint64_t expected[2] = {0}, actual[2] = {0};
    while (__real_av_read_frame(m.file, m.packet) >= 0) {
        int stream = m.packet->stream_index == m.stream ? 0 : 1;
        expected[stream] = expected[stream] * 31 + digest(m.packet);
        av_packet_unref(m.packet);
    }
    av_assert0(av_seek_frame(m.file, m.stream, 0, AVSEEK_FLAG_BACKWARD) >= 0);
    m.tracing = 1;
    atomic_store(&gated, 1);
    av_assert0(media_reader_start(&m) == 0);
    wait_for(&reading);
    int64_t started = av_gettime_relative();
    av_assert0(media_packet(&m, 0, m.packet) == AVERROR(EAGAIN));
    av_assert0(av_gettime_relative() - started < 100000);
    pthread_t thread;
    av_assert0(pthread_create(&thread, NULL, pause_reader, &m) == 0);
    av_usleep(20000);
    av_assert0(!atomic_load(&paused));
    atomic_store(&gated, 0);
    pthread_join(thread, NULL);
    av_assert0(atomic_load(&paused));
    rewind_file(&m);
    int64_t deadline = av_gettime_relative() + 5000000;
    while (!media_buffer(&m, 0).ready) {
        av_assert0(av_gettime_relative() < deadline);
        av_usleep(1000);
    }
    MediaBuffer buffer = media_buffer(&m, 0);
    av_assert0(buffer.video_us >= READ_AHEAD_US && buffer.audio_us >= READ_AHEAD_US);
    int count = atomic_load(&calls);
    av_usleep(30000);
    av_assert0(atomic_load(&calls) == count);
    printf("One-second prefill, idle reader and nonblocking dequeue passed\n");
    for (int i = 0; i < 5; i++) {
        av_assert0(media_packet(&m, 0, m.packet) == 0);
        actual[0] = actual[0] * 31 + digest(m.packet);
    }
    av_assert0(!media_buffer(&m, 1).ready);
    while (!media_buffer(&m, 1).ready) {
        av_assert0(av_gettime_relative() < deadline);
        av_usleep(1000);
    }
    printf("Startup refill bypasses playback hysteresis\n");
    atomic_store(&gated, 1);
    for (int i = 0; i < 14; i++) {
        av_assert0(media_packet(&m, 0, m.packet) == 0);
        actual[0] = actual[0] * 31 + digest(m.packet);
    }
    wait_for(&reading);
    av_assert0(media_buffer(&m, 0).video_us >= 250000);
    started = av_gettime_relative();
    for (int i = 0; i < 4; i++) {
        av_assert0(media_packet(&m, 0, m.packet) == 0);
        actual[0] = actual[0] * 31 + digest(m.packet);
    }
    av_assert0(av_gettime_relative() - started < 100000);
    atomic_store(&gated, 0);
    printf("Queued packets remain available during a blocked refill\n");
    int ended[2] = {0};
    while (!ended[0] || !ended[1]) {
        av_assert0(av_gettime_relative() < deadline);
        for (int i = 0; i < 2; i++) {
            if (ended[i]) continue;
            int rc = media_packet(&m, i, m.packet);
            av_assert0(rc >= 0 || rc == AVERROR(EAGAIN) || rc == AVERROR_EOF);
            if (rc >= 0) actual[i] = actual[i] * 31 + digest(m.packet);
            ended[i] = rc == AVERROR_EOF;
        }
        av_usleep(100);
    }
    av_assert0(actual[0] == expected[0] && actual[1] == expected[1]);
    av_assert0(media_reader_trace(&m, 0).packets > 0);
    printf("Buffered packet contents/order, short audio and EOF passed\n");
    for (int i = 0; i < 20; i++) {
        media_reader_pause(&m, 1);
        media_clear_packets(&m, AV_NOPTS_VALUE);
        int64_t target = av_rescale_q(i % 2, (AVRational){1, 1}, m.file->streams[m.stream]->time_base);
        av_assert0(av_seek_frame(m.file, m.stream, target, AVSEEK_FLAG_BACKWARD) >= 0);
        av_assert0(media_read_packet(&m) == 0);
        av_assert0(m.packet->pts <= target);
        media_reader_pause(&m, 0);
    }
    test_packet_limits(&m);
    atomic_store(&failure, AVERROR(EAGAIN));
    media_reader_pause(&m, 0);
    av_usleep(30000);
    media_reader_pause(&m, 1);
    media_clear_packets(&m, AV_NOPTS_VALUE);
    atomic_store(&failure, AVERROR(EIO));
    media_reader_pause(&m, 0);
    int rc;
    do { rc = media_packet(&m, 0, m.packet); av_usleep(1000); } while (rc == AVERROR(EAGAIN));
    av_assert0(rc == AVERROR(EIO));
    atomic_store(&gated, 1);
    atomic_store(&paused, 0);
    rewind_file(&m);
    wait_for(&reading);
    av_assert0(pthread_create(&thread, NULL, stop_reader, &m) == 0);
    av_usleep(20000);
    av_assert0(!atomic_load(&paused));
    atomic_store(&gated, 0);
    pthread_join(thread, NULL);
    av_assert0(atomic_load(&paused) && !m.reader);
    av_packet_free(&m.packet);
    avformat_close_input(&m.file);
    printf("Repeated seeks, synchronous preroll, read errors and stop passed\n");
    return 0;
}
