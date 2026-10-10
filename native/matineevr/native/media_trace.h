#include <stdint.h>

typedef struct MediaTrace {
    int64_t audio_reset_us, seek_us, close_us, open_us, read_us, send_us, receive_us, audio_us;
    uint64_t seek_calls, packets, packet_bytes, send_calls, receive_calls, again, frames, rejected;
    uint64_t audio_underflows, audio_latency_samples;
    int64_t audio_latency_us;
} MediaTrace;

void media_profile(void *handle, int continuous);
MediaTrace media_trace(void *handle);
