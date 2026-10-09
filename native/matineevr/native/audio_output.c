#include "audio_output.h"
#include "media_trace.h"
#include <math.h>
#include <stdlib.h>

AudioOutput *output_open(void) {
    AudioOutput *p = calloc(1, sizeof(*p));
    if (!p) return NULL;
    p->loop = pa_mainloop_new();
    if (!p->loop) goto failed;
    p->context = pa_context_new(pa_mainloop_get_api(p->loop), "MatineeVR");
    if (!p->context || pa_context_connect(p->context, NULL, PA_CONTEXT_NOAUTOSPAWN, NULL) < 0)
        goto failed;
    p->corked = 1;
    p->deadline = av_gettime_relative() + 5000000;
    return p;
failed:
    output_close(p);
    return NULL;
}

void output_close(AudioOutput *p) {
    if (!p) return;
    if (p->operation) { pa_operation_cancel(p->operation); pa_operation_unref(p->operation); }
    if (p->drain) { pa_operation_cancel(p->drain); pa_operation_unref(p->drain); }
    if (p->stream) { pa_stream_disconnect(p->stream); pa_stream_unref(p->stream); }
    if (p->context) { pa_context_disconnect(p->context); pa_context_unref(p->context); }
    if (p->loop) pa_mainloop_free(p->loop);
    free(p);
}

static void underflow(pa_stream *stream, void *userdata) {
    (void)stream;
    AudioOutput *p = userdata;
    if (p->trace) p->trace->audio_underflows++;
}

static int connect_stream(AudioOutput *p) {
    pa_sample_spec spec = { PA_SAMPLE_FLOAT32NE, AUDIO_RATE, AUDIO_CHANNELS };
    p->stream = pa_stream_new(p->context, "Video audio", &spec, NULL);
    if (!p->stream) return AVERROR_EXTERNAL;
    pa_stream_set_underflow_callback(p->stream, underflow, p);
    uint32_t buffer = AUDIO_RATE * AUDIO_FRAME_BYTES / 4;
    pa_buffer_attr attr = { buffer, buffer, (uint32_t)-1, (uint32_t)-1, (uint32_t)-1 };
    pa_stream_flags_t flags = PA_STREAM_START_CORKED | PA_STREAM_AUTO_TIMING_UPDATE |
                              PA_STREAM_INTERPOLATE_TIMING;
    if (pa_stream_connect_playback(p->stream, NULL, &attr, flags, NULL, NULL) < 0)
        return AVERROR_EXTERNAL;
    return 0;
}

int output_poll(AudioOutput *p, int paused) {
    if (pa_mainloop_iterate(p->loop, 0, NULL) < 0) return AVERROR_EXTERNAL;
    pa_context_state_t context = pa_context_get_state(p->context);
    if (!PA_CONTEXT_IS_GOOD(context)) return AVERROR_EXTERNAL;
    if (context == PA_CONTEXT_READY && !p->stream) {
        int rc = connect_stream(p);
        if (rc < 0) return rc;
    }
    if (!p->stream || pa_stream_get_state(p->stream) == PA_STREAM_CREATING) {
        return av_gettime_relative() > p->deadline ? AVERROR(ETIMEDOUT) : 0;
    }
    if (pa_stream_get_state(p->stream) != PA_STREAM_READY) return AVERROR_EXTERNAL;
    return output_control(p, paused);
}

int output_write(AudioOutput *p, const float *samples, int frames) {
    size_t writable = pa_stream_writable_size(p->stream);
    if (writable == (size_t)-1) return AVERROR_EXTERNAL;
    if (writable / AUDIO_FRAME_BYTES < (size_t)frames) frames = writable / AUDIO_FRAME_BYTES;
    if (!frames) return 0;
    if (pa_stream_write(p->stream, samples, frames * AUDIO_FRAME_BYTES, NULL, 0, PA_SEEK_RELATIVE) < 0)
        return AVERROR_EXTERNAL;
    p->end += frames / (double)AUDIO_RATE;
    return frames;
}

int output_time(AudioOutput *p, double *clock, double *limit) {
    pa_usec_t latency;
    int negative;
    *limit = p->clock;
    if (p->stream && !p->corked && !p->reset && !p->timing && !p->operation &&
        pa_stream_get_state(p->stream) == PA_STREAM_READY) {
        if (pa_stream_get_latency(p->stream, &latency, &negative) == 0) {
            if (p->trace) {
                p->trace->audio_latency_samples++;
                p->trace->audio_latency_us += negative ? -(int64_t)latency : (int64_t)latency;
            }
            double position = p->end - (negative ? 0 : latency / 1000000.0);
            p->clock = fmax(p->clock, fmin(position, p->end));
            *limit = p->end;
        } else if (pa_context_errno(p->context) != PA_ERR_NODATA) {
            return AVERROR_EXTERNAL;
        }
    }
    *clock = p->clock;
    return 0;
}
