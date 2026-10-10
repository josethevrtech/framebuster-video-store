#include "media_internal.h"
#include <math.h>

int media_seek_file(Media *m, int64_t target) {
    AVStream *stream = m->file->streams[m->stream];
    int64_t start = stream->start_time == AV_NOPTS_VALUE ? 0 : stream->start_time;
    int64_t preroll = av_rescale_q((int64_t)ceil(audio_preroll(m->audio) * AV_TIME_BASE),
                                  AV_TIME_BASE_Q, stream->time_base);
    int64_t before = target - preroll < start ? start : target - preroll;
    for (;;) {
        media_clear_packets(m, target);
        int64_t started = media_trace_start(m);
        int rc = av_seek_frame(m->file, m->stream, before, AVSEEK_FLAG_BACKWARD);
        media_trace_end(started, &m->trace.seek_us);
        if (m->tracing) m->trace.seek_calls++;
        if (rc < 0) return rc;
        do { rc = media_read_packet(m); } while (rc == AVERROR(EAGAIN));
        if (rc < 0) return rc == AVERROR_EOF ? 0 : rc;
        if (m->packet->pts == AV_NOPTS_VALUE) return AVERROR_INVALIDDATA;
        if (m->packet->pts <= target) return 0;
        if (m->packet->dts == AV_NOPTS_VALUE || m->packet->dts > before)
            return AVERROR_INVALIDDATA;
        before = m->packet->dts - 1;
    }
}

static int can_continue_to(Media *m, int64_t target) {
    int64_t current = m->frame->best_effort_timestamp;
    if (m->draining || current == AV_NOPTS_VALUE || current >= target ||
        m->frame->decode_error_flags || m->frame->flags & AV_FRAME_FLAG_CORRUPT)
        return 0;
    AVStream *stream = m->file->streams[m->stream];
    int index = av_index_search_timestamp(stream, current, AVSEEK_FLAG_BACKWARD);
    return index >= 0 && index == av_index_search_timestamp(stream, target, AVSEEK_FLAG_BACKWARD);
}

static int seek(Media *m, double seconds, char *error, size_t capacity) {
    if (!m->continuous) m->trace = (MediaTrace){0};
    m->tracing = m->profiling;
    media_reader_trace(m, !m->continuous);
    if (!isfinite(seconds) || seconds < 0 || seconds >= (double)(INT64_MAX / AV_TIME_BASE))
        return media_error(error, capacity, "invalid seek target", AVERROR(EINVAL));
    if (atomic_load(&m->references) != 1)
        return media_error(error, capacity, "release decoded frames before seeking", AVERROR(EBUSY));
    AVStream *stream = m->file->streams[m->stream];
    double duration = media_duration(m);
    if (duration > 0 && seconds > duration) seconds = duration;
    int64_t offset = stream->start_time == AV_NOPTS_VALUE ? 0 : stream->start_time;
    int64_t timestamp = av_rescale_q((int64_t)(seconds * AV_TIME_BASE), AV_TIME_BASE_Q, stream->time_base);
    if (timestamp < 0 || (offset > 0 && timestamp > INT64_MAX - offset))
        return media_error(error, capacity, "seek timestamp overflow", AVERROR(EINVAL));
    m->preview_pending = 1;
    m->preview_repeat = 0;
    if (!m->audio && can_continue_to(m, timestamp + offset)) {
        m->seek_target = seconds;
        return 0;
    }
    av_frame_unref(m->frame);
    av_packet_unref(m->packet);
    int64_t started = media_trace_start(m);
    int rc = audio_reset(m->audio, seconds);
    media_trace_end(started, &m->trace.audio_reset_us);
    if (rc < 0) return media_error(error, capacity, "reset audio", rc);
    rc = media_seek_file(m, timestamp + offset);
    if (rc < 0) return media_error(error, capacity, "seek video", rc);
    started = media_trace_start(m);
    avcodec_free_context(&m->decoder);
    media_trace_end(started, &m->trace.close_us);
    started = media_trace_start(m);
    rc = media_open_decoder(m);
    media_trace_end(started, &m->trace.open_us);
    if (rc < 0) return media_error(error, capacity, "reopen Iris decoder", rc);
    m->draining = 0;
    m->fallback_pts = m->seek_target = seconds;
    return 0;
}

int media_seek(void *handle, double seconds, char *error, size_t capacity) {
    Media *m = handle;
    int rc = media_reader_start(m);
    if (rc < 0) return media_error(error, capacity, "start packet reader", rc);
    media_reader_pause(m, 1);
    rc = seek(m, seconds, error, capacity);
    media_reader_pause(m, 0);
    return rc;
}
