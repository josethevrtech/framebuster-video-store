#include "media_internal.h"
#include <pthread.h>
#include <libavutil/time.h>

enum { READ_AHEAD_US = AV_TIME_BASE, MAX_PACKET_BYTES = 64 * 1024 * 1024, MAX_PACKETS = 4096 };

struct Reader {
    pthread_t thread;
    pthread_mutex_t mutex;
    pthread_cond_t changed;
    AVFifo *packets[2];
    AVPacket *pending;
    AVRational time_base[2];
    int stream[2], ended[2];
    int64_t end[2];
    size_t bytes;
    int result, paused, reading, stopped, filling, limited, waiting, tracing;
    MediaTrace trace;
};

int64_t reader_buffered(Reader *r, int stream);
int reader_filled(Reader *r, int64_t target);
int reader_take(Reader *r, int stream, AVPacket *packet);
int reader_queue(Reader *r);
int reader_read(Media *m);
