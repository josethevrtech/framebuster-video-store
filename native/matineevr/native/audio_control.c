#include "audio_output.h"

static void completed(pa_stream *stream, int success, void *userdata) {
    (void)stream;
    *(int *)userdata = success ? 1 : -1;
}

void output_reset(AudioOutput *p, double target) {
    p->reset = 1;
    p->end = p->clock = target;
    p->drained = 0;
    if (p->drain) {
        pa_operation_cancel(p->drain);
        pa_operation_unref(p->drain);
        p->drain = NULL;
    }
}

int output_control(AudioOutput *p, int paused) {
    if (p->drain && paused) {
        pa_operation_cancel(p->drain);
        pa_operation_unref(p->drain);
        p->drain = NULL;
        p->drained = 0;
    }
    if (p->operation) {
        if (pa_operation_get_state(p->operation) == PA_OPERATION_RUNNING)
            return av_gettime_relative() > p->deadline ? AVERROR(ETIMEDOUT) : 0;
        pa_operation_unref(p->operation);
        p->operation = NULL;
        if (p->success != 1) return AVERROR_EXTERNAL;
    }
    int corked = paused || p->reset || p->timing;
    if (corked != p->corked) {
        p->operation = pa_stream_cork(p->stream, corked, completed, &p->success);
        p->corked = corked;
    } else if (p->reset) {
        p->operation = pa_stream_flush(p->stream, completed, &p->success);
        p->reset = 0;
        p->timing = 1;
    } else if (p->timing) {
        p->operation = pa_stream_update_timing_info(p->stream, completed, &p->success);
        p->timing = 0;
    } else {
        return 1;
    }
    if (!p->operation) return AVERROR_EXTERNAL;
    p->success = 0;
    p->deadline = av_gettime_relative() + 5000000;
    return 0;
}

int output_finish(AudioOutput *p) {
    if (p->end == p->clock) p->drained = 1;
    if (p->drained < 0) return AVERROR_EXTERNAL;
    if (p->drained) {
        p->clock = p->end;
        return 1;
    }
    if (!p->corked && !p->drain) {
        p->drain = pa_stream_drain(p->stream, completed, &p->drained);
        if (!p->drain) return AVERROR_EXTERNAL;
    }
    return 0;
}
