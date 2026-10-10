#include <stddef.h>
#include <stdint.h>
#include "media_trace.h"

typedef struct {
    int fd;
    uint32_t format;
    uint64_t size, modifier, offsets[2], pitches[2], pool;
    int width, height, crop[4], chroma_location, stream, transfer, primaries;
} DmaBuf;

typedef struct {
    void *owner;
    int width, height, colorspace, full_range;
    float sample_aspect_ratio;
    double pts;
    DmaBuf dmabuf;
} Frame;

int media_reconfigure(void *handle, int released, char *error, size_t capacity);
void *media_open(const char *path, char *error, size_t capacity);
int media_supported(const char *path, char *error, size_t capacity);
double media_duration(void *handle);
void media_metadata(void *handle, int fields[3]);
int media_seek(void *handle, double seconds, char *error, size_t capacity);
int media_next(void *handle, Frame *frame, char *error, size_t capacity);
void media_release(Frame *frame);
void media_close(void *handle);
int media_enable_audio(void *handle, char *error, size_t capacity);
int media_audio(void *handle, int paused, double *clock, double *limit, int *done, char *error, size_t capacity);
void media_stop(void *handle);

typedef struct {
    int64_t video_us, audio_us;
    uint64_t bytes;
    int ready, limited, waiting;
} MediaBuffer;

MediaBuffer media_buffer(void *handle, int prefill);
