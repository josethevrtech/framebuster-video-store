#include "audio_internal.h"
#include <libavutil/time.h>

void media_profile(void *handle, int continuous) {
    Media *m = handle;
    m->profiling = 1;
    if (continuous) {
        m->continuous = m->tracing = 1;
        if (m->audio) m->audio->output->trace = &m->trace;
    }
    media_reader_trace(m, 0);
}

MediaTrace media_trace(void *handle) {
    Media *m = handle;
    m->tracing = m->continuous;
    MediaTrace trace = m->trace;
    MediaTrace reader = media_reader_trace(m, 0);
    trace.read_us = reader.read_us;
    trace.packets = reader.packets;
    trace.packet_bytes = reader.packet_bytes;
    return trace;
}

int64_t media_trace_start(Media *m) {
    return m->tracing ? av_gettime_relative() : 0;
}

void media_trace_end(int64_t started, int64_t *elapsed) {
    if (started) *elapsed += av_gettime_relative() - started;
}
