#include "media_internal.h"
#include "audio_output.h"
#include <libswresample/swresample.h>

struct Audio {
    AVCodecContext *decoder;
    SwrContext *resampler;
    AVFrame *frame, *pcm;
    AVPacket *packet;
    AudioOutput *output;
    int draining, ended, offset, seeking, input_rate;
    int64_t written, frame_start;
    double target, next_pts, end, time_base;
};

int audio_decode(Media *m);
