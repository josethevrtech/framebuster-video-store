#include "media_reader.h"

static int64_t timestamp(const AVPacket *p) {
    return p->dts == AV_NOPTS_VALUE ? p->pts : p->dts;
}

int64_t reader_buffered(Reader *r, int stream) {
    size_t count = av_fifo_can_read(r->packets[stream]);
    if (!count) return 0;
    AVPacket first, last;
    av_fifo_peek(r->packets[stream], &first, 1, 0);
    av_fifo_peek(r->packets[stream], &last, 1, count - 1);
    int64_t start = timestamp(&first), end = timestamp(&last);
    if (start == AV_NOPTS_VALUE || end == AV_NOPTS_VALUE || end < start) return -1;
    int64_t span = av_sat_add64(av_sat_sub64(end, start), FFMAX(last.duration, 0));
    return av_rescale_q(span, r->time_base[stream], AV_TIME_BASE_Q);
}

int reader_filled(Reader *r, int64_t target) {
    for (int i = 0; i < 2; i++)
        if (r->stream[i] >= 0 && !(i && r->ended[i]) && reader_buffered(r, i) < target) return 0;
    return 1;
}

void media_clear_packets(Media *m, int64_t target) {
    Reader *r = m->reader;
    if (!r) return;
    pthread_mutex_lock(&r->mutex);
    for (int i = 0; i < 2; i++) {
        AVPacket packet;
        while (av_fifo_can_read(r->packets[i])) {
            av_fifo_read(r->packets[i], &packet, 1);
            av_packet_unref(&packet);
        }
        r->ended[i] = i && r->stream[i] >= 0 && target != AV_NOPTS_VALUE && r->end[i] != AV_NOPTS_VALUE &&
                      av_compare_ts(target, r->time_base[0], r->end[i], r->time_base[i]) >= 0;
    }
    av_packet_unref(r->pending);
    r->bytes = 0;
    r->result = r->limited = r->waiting = 0;
    r->filling = 1;
    pthread_mutex_unlock(&r->mutex);
}

static int has_space(Reader *r, int stream) {
    return av_fifo_can_write(r->packets[stream]) &&
           (size_t)r->pending->size <= MAX_PACKET_BYTES - r->bytes;
}

int reader_take(Reader *r, int stream, AVPacket *packet) {
    av_packet_unref(packet);
    r->waiting &= ~(1 << stream);
    if (av_fifo_can_read(r->packets[stream])) {
        av_fifo_read(r->packets[stream], packet, 1);
        r->bytes -= packet->size;
        pthread_cond_broadcast(&r->changed);
        return 0;
    }
    if (r->result) return r->result;
    if (stream && r->ended[stream]) return AVERROR_EOF;
    r->waiting |= 1 << stream;
    int destination = r->pending->stream_index == r->stream[0] ? 0 : 1;
    return r->limited && !has_space(r, destination) ? AVERROR(ENOBUFS) : AVERROR(EAGAIN);
}

int reader_queue(Reader *r) {
    AVPacket *p = r->pending;
    int stream = p->stream_index == r->stream[0] ? 0 :
                 p->stream_index == r->stream[1] ? 1 : -1;
    if (stream < 0) { av_packet_unref(p); return 0; }
    if (p->size < 0 || p->size > MAX_PACKET_BYTES) return AVERROR(ENOBUFS);
    r->limited = !has_space(r, stream);
    if (r->limited) return AVERROR(EAGAIN);
    r->waiting &= ~(1 << stream);
    r->bytes += p->size;
    if (p->pts != AV_NOPTS_VALUE && r->end[stream] != AV_NOPTS_VALUE &&
        p->duration > 0 && p->pts >= av_sat_sub64(r->end[stream], p->duration))
        r->ended[stream] = 1;
    AVPacket queued;
    av_packet_move_ref(&queued, p);
    av_fifo_write(r->packets[stream], &queued, 1);
    return 0;
}

int media_packet(Media *m, int audio, AVPacket *packet) {
    int rc = media_reader_start(m);
    if (rc < 0) return rc;
    Reader *r = m->reader;
    pthread_mutex_lock(&r->mutex);
    rc = reader_take(r, audio, packet);
    while (rc == AVERROR(EAGAIN) && r->paused) {
        if (reader_read(m) == AVERROR(EAGAIN)) break;
        if (!r->result) {
            int queued = reader_queue(r);
            if (queued < 0) r->result = queued == AVERROR(EAGAIN) ? AVERROR(ENOBUFS) : queued;
        }
        rc = reader_take(r, audio, packet);
    }
    pthread_mutex_unlock(&r->mutex);
    return rc;
}

int media_read_packet(Media *m) {
    int rc = media_packet(m, 0, m->packet);
    if (rc >= 0 && m->seek_target >= 0 && m->packet->pts == AV_NOPTS_VALUE)
        return AVERROR_INVALIDDATA;
    return rc;
}
