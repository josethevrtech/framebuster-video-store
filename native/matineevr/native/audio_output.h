#include <pulse/pulseaudio.h>
#include <libavutil/error.h>
#include <libavutil/time.h>

enum { AUDIO_RATE = 48000, AUDIO_CHANNELS = 2, AUDIO_FRAME_BYTES = 8 };

typedef struct {
    pa_mainloop *loop;
    pa_context *context;
    pa_stream *stream;
    pa_operation *operation, *drain;
    int success, drained, corked, reset, timing;
    int64_t deadline;
    double end, clock;
    struct MediaTrace *trace;
} AudioOutput;

AudioOutput *output_open(void);
void output_close(AudioOutput *p);
void output_reset(AudioOutput *p, double target);
int output_poll(AudioOutput *p, int paused);
int output_write(AudioOutput *p, const float *samples, int frames);
int output_finish(AudioOutput *p);
int output_time(AudioOutput *p, double *clock, double *limit);
int output_control(AudioOutput *p, int paused);
