#include "media_reader.h"
#include <libavutil/avassert.h>

void test_packet_limits(Media *m) {
    Reader *r = m->reader;
    media_reader_pause(m, 1);
    media_clear_packets(m, AV_NOPTS_VALUE);
    pthread_mutex_lock(&r->mutex);
    av_assert0(av_new_packet(r->pending, MAX_PACKET_BYTES) == 0);
    r->pending->stream_index = r->stream[0];
    av_assert0(reader_queue(r) == 0);
    av_assert0(reader_buffered(r, 0) == -1);
    av_assert0(!reader_filled(r, READ_AHEAD_US));
    av_assert0(av_new_packet(r->pending, 1) == 0);
    r->pending->stream_index = r->stream[0];
    av_assert0(reader_queue(r) == AVERROR(EAGAIN));
    av_assert0(r->limited && r->bytes == MAX_PACKET_BYTES);
    av_assert0(reader_take(r, 1, m->packet) == AVERROR(ENOBUFS));
    av_assert0(reader_take(r, 0, m->packet) == 0);
    av_assert0(reader_queue(r) == 0 && !r->limited);
    pthread_mutex_unlock(&r->mutex);
    media_clear_packets(m, AV_NOPTS_VALUE);
    pthread_mutex_lock(&r->mutex);
    for (int i = 0; i <= MAX_PACKETS; i++) {
        av_assert0(av_new_packet(r->pending, 1) == 0);
        r->pending->stream_index = r->stream[0];
        av_assert0(reader_queue(r) == (i == MAX_PACKETS ? AVERROR(EAGAIN) : 0));
    }
    pthread_mutex_unlock(&r->mutex);
    media_clear_packets(m, AV_NOPTS_VALUE);
    printf("Missing timestamps and byte/packet caps passed\n");
}
