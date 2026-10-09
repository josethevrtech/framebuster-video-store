#include "media_reader.h"

int reader_read(Media *m) {
    Reader *r = m->reader;
    int64_t started = r->tracing ? av_gettime_relative() : 0;
    r->reading = 1;
    pthread_mutex_unlock(&r->mutex);
    int rc = av_read_frame(m->file, r->pending);
    int64_t elapsed = started ? av_gettime_relative() - started : 0;
    if (rc == AVERROR(EAGAIN)) av_usleep(1000);
    pthread_mutex_lock(&r->mutex);
    if (started) {
        r->trace.read_us += elapsed;
        r->trace.packets += rc >= 0;
        if (rc >= 0) r->trace.packet_bytes += r->pending->size;
    }
    r->reading = 0;
    if (rc < 0 && rc != AVERROR(EAGAIN)) r->result = rc;
    pthread_cond_broadcast(&r->changed);
    return rc;
}

static void *read_packets(void *handle) {
    Media *m = handle;
    Reader *r = m->reader;
    pthread_mutex_lock(&r->mutex);
    while (!r->stopped) {
        if (reader_filled(r, READ_AHEAD_US)) r->filling = 0;
        if (!reader_filled(r, READ_AHEAD_US / 2)) r->filling = 1;
        if (r->paused || r->result || !r->filling) {
            pthread_cond_wait(&r->changed, &r->mutex);
            continue;
        }
        if (!r->limited && reader_read(m) < 0) continue;
        if (r->result) continue;
        int rc = reader_queue(r);
        if (rc == AVERROR(EAGAIN)) pthread_cond_wait(&r->changed, &r->mutex);
        else if (rc < 0) r->result = rc;
    }
    pthread_mutex_unlock(&r->mutex);
    return NULL;
}

int media_reader_start(Media *m) {
    if (m->reader) return 0;
    Reader *r = calloc(1, sizeof(*r));
    if (!r) return AVERROR(ENOMEM);
    int rc = pthread_mutex_init(&r->mutex, NULL);
    if (rc) { free(r); return AVERROR(rc); }
    rc = pthread_cond_init(&r->changed, NULL);
    if (rc) { pthread_mutex_destroy(&r->mutex); free(r); return AVERROR(rc); }
    r->stream[0] = m->stream;
    r->stream[1] = m->audio_stream;
    for (int i = 0; i < 2; i++) {
        r->packets[i] = av_fifo_alloc2(MAX_PACKETS, sizeof(AVPacket), 0);
        if (r->stream[i] < 0) continue;
        AVStream *s = m->file->streams[r->stream[i]];
        r->time_base[i] = s->time_base;
        r->end[i] = s->duration == AV_NOPTS_VALUE || s->start_time == AV_NOPTS_VALUE ?
                    AV_NOPTS_VALUE : av_sat_add64(s->start_time, s->duration);
    }
    r->pending = av_packet_alloc();
    r->filling = 1;
    r->tracing = m->tracing;
    m->reader = r;
    rc = !r->pending || !r->packets[0] || !r->packets[1] ? ENOMEM :
         pthread_create(&r->thread, NULL, read_packets, m);
    if (!rc) return 0;
    for (int i = 0; i < 2; i++) av_fifo_freep2(&r->packets[i]);
    av_packet_free(&r->pending);
    pthread_cond_destroy(&r->changed);
    pthread_mutex_destroy(&r->mutex);
    free(r);
    m->reader = NULL;
    return AVERROR(rc);
}

void media_reader_pause(Media *m, int paused) {
    Reader *r = m->reader;
    if (!r) return;
    pthread_mutex_lock(&r->mutex);
    r->paused = paused;
    pthread_cond_broadcast(&r->changed);
    while (paused && r->reading) pthread_cond_wait(&r->changed, &r->mutex);
    pthread_mutex_unlock(&r->mutex);
}

void media_reader_close(Media *m) {
    Reader *r = m->reader;
    if (!r) return;
    pthread_mutex_lock(&r->mutex);
    r->stopped = 1;
    atomic_store(&m->interrupted, 1);
    pthread_cond_signal(&r->changed);
    pthread_mutex_unlock(&r->mutex);
    pthread_join(r->thread, NULL);
    media_clear_packets(m, AV_NOPTS_VALUE);
    for (int i = 0; i < 2; i++) av_fifo_freep2(&r->packets[i]);
    av_packet_free(&r->pending);
    pthread_cond_destroy(&r->changed);
    pthread_mutex_destroy(&r->mutex);
    free(r);
    m->reader = NULL;
}

MediaTrace media_reader_trace(Media *m, int reset) {
    Reader *r = m->reader;
    MediaTrace trace = {0};
    if (!r) return trace;
    pthread_mutex_lock(&r->mutex);
    if (reset) r->trace = trace;
    r->tracing = m->tracing;
    trace = r->trace;
    pthread_mutex_unlock(&r->mutex);
    return trace;
}

MediaBuffer media_buffer(void *handle, int prefill) {
    Reader *r = ((Media *)handle)->reader;
    MediaBuffer buffer = {0};
    if (!r) return buffer;
    pthread_mutex_lock(&r->mutex);
    buffer = (MediaBuffer){reader_buffered(r, 0), reader_buffered(r, 1), r->bytes,
                           r->result || r->limited || reader_filled(r, READ_AHEAD_US),
                           r->limited, r->waiting};
    if (prefill && !buffer.ready) {
        r->filling = 1;
        pthread_cond_signal(&r->changed);
    }
    pthread_mutex_unlock(&r->mutex);
    return buffer;
}
