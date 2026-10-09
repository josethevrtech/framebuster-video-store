#include "media.h"
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
#include <libavutil/pixdesc.h>
#include <libavutil/fifo.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct Audio Audio;
typedef struct Reader Reader;

typedef struct {
    atomic_uint references;
    atomic_int interrupted;
    uint64_t pool;
    AVFormatContext *file;
    AVCodecContext *decoder;
    AVPacket *packet;
    AVFrame *frame;
    int stream, draining, preview_pending, preview_repeat;
    double time_base, start, fallback_pts, step, seek_target;
    Audio *audio;
    Reader *reader;
    int audio_stream, metadata[3];
    double duration;
    int profiling, tracing, continuous;
    MediaTrace trace;
} Media;

typedef struct {
    AVFrame *frame;
    Media *media;
} FrameOwner;

int media_error(char *error, size_t capacity, const char *operation, int code);
int media_read_packet(Media *m);
int media_packet(Media *m, int audio, AVPacket *packet);
void media_clear_packets(Media *m, int64_t target);
int media_reader_start(Media *m);
void media_reader_pause(Media *m, int paused);
void media_reader_close(Media *m);
MediaTrace media_reader_trace(Media *m, int reset);
void media_read_metadata(Media *m);
float media_sample_aspect_ratio(Media *m, AVFrame *frame);
int media_open_decoder(Media *m);
int media_seek_file(Media *m, int64_t target);
int audio_open(Media *m);
void audio_close(Audio *a);
int audio_reset(Audio *a, double target);
int audio_tick(Media *m, int paused, double *clock, double *limit, int *done);
double audio_preroll(Audio *a);
int64_t media_trace_start(Media *m);
void media_trace_end(int64_t started, int64_t *elapsed);
