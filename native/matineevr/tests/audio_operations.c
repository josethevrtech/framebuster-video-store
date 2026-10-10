#include "audio_internal.h"
#include <libavutil/avassert.h>

AudioOutput *test_output;
static pa_stream_success_cb_t cork_callback;
static void *cork_userdata;
static int requested_cork, confirmed_cork;

static void cork_completed(pa_stream *stream, int success, void *userdata) {
    (void)userdata;
    confirmed_cork = success ? requested_cork : -1;
    cork_callback(stream, success, cork_userdata);
}

pa_operation *__real_pa_stream_cork(pa_stream *, int, pa_stream_success_cb_t, void *);

pa_operation *__wrap_pa_stream_cork(pa_stream *stream, int cork, pa_stream_success_cb_t callback, void *userdata) {
    requested_cork = cork;
    cork_callback = callback;
    cork_userdata = userdata;
    return __real_pa_stream_cork(stream, cork, cork_completed, NULL);
}

int __real_pa_stream_write(pa_stream *, const void *, size_t, pa_free_cb_t, int64_t, pa_seek_mode_t);

int __wrap_pa_stream_write(pa_stream *stream, const void *data, size_t bytes, pa_free_cb_t release, int64_t offset, pa_seek_mode_t seek) {
    av_assert0(test_output && !test_output->reset && !test_output->timing && !test_output->operation);
    return __real_pa_stream_write(stream, data, bytes, release, offset, seek);
}

void audio_assert_paused(void) {
    av_assert0(!cork_callback || confirmed_cork == 1);
    av_assert0(pa_stream_is_corked(test_output->stream) == 1);
}
