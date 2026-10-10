#include "media_internal.h"
#include <libavutil/opt.h>
#include <math.h>

int media_error(char *error, size_t capacity, const char *operation, int code) {
    char detail[AV_ERROR_MAX_STRING_SIZE];
    av_strerror(code, detail, sizeof(detail));
    snprintf(error, capacity, "%s: %s", operation, detail);
    return -1;
}

void media_close(void *handle) {
    Media *m = handle;
    if (!m || atomic_fetch_sub(&m->references, 1) != 1) return;
    media_reader_close(m);
    audio_close(m->audio);
    av_frame_free(&m->frame);
    av_packet_free(&m->packet);
    avcodec_free_context(&m->decoder);
    avformat_close_input(&m->file);
    free(m);
}

static const char *decoder_name(const AVCodecParameters *p) {
    const AVPixFmtDescriptor *format = av_pix_fmt_desc_get(p->format);
    if (!format || (format->comp[0].depth != 8 &&
        !(p->codec_id == AV_CODEC_ID_HEVC && format->comp[0].depth == 10))) return NULL;
    return p->codec_id == AV_CODEC_ID_H264 ? "h264_v4l2m2m" :
           p->codec_id == AV_CODEC_ID_HEVC ? "hevc_v4l2m2m" :
           p->codec_id == AV_CODEC_ID_VP9 ? "vp9_v4l2m2m" : NULL;
}

static int video_stream(AVFormatContext **file, const char *path) {
    int rc = avformat_open_input(file, path, NULL, NULL);
    if (rc >= 0) rc = avformat_find_stream_info(*file, NULL);
    if (rc < 0) return rc;
    int preferred = av_find_best_stream(*file, AVMEDIA_TYPE_VIDEO, -1, -1, NULL, 0);
    if (preferred < 0 || decoder_name((*file)->streams[preferred]->codecpar)) return preferred;
    for (unsigned i = 0; i < (*file)->nb_streams; ++i) {
        AVCodecParameters *p = (*file)->streams[i]->codecpar;
        if (p->codec_type == AVMEDIA_TYPE_VIDEO && decoder_name(p)) return i;
    }
    return preferred;
}

int media_supported(const char *path, char *error, size_t capacity) {
    AVFormatContext *file = NULL;
    int stream = video_stream(&file, path);
    int supported = stream >= 0 && decoder_name(file->streams[stream]->codecpar);
    avformat_close_input(&file);
    if (stream < 0 && stream != AVERROR_STREAM_NOT_FOUND)
        return media_error(error, capacity, "inspect video", stream);
    return supported;
}

int media_open_decoder(Media *m) {
    m->pool++;
    AVStream *stream = m->file->streams[m->stream];
    const AVCodec *codec = avcodec_find_decoder_by_name(decoder_name(stream->codecpar));
    if (!codec) return AVERROR_DECODER_NOT_FOUND;
    m->decoder = avcodec_alloc_context3(codec);
    if (!m->decoder) return AVERROR(ENOMEM);
    int rc = avcodec_parameters_to_context(m->decoder, stream->codecpar);
    if (rc >= 0) rc = av_opt_set_int(m->decoder->priv_data, "num_capture_buffers", 4, 0);
    if (rc >= 0)
        rc = av_opt_set_int(m->decoder->priv_data, "drm_prime", 1, 0);
    if (rc < 0) return rc;
    m->decoder->apply_cropping = 0;
    m->decoder->pkt_timebase = stream->time_base;
    m->decoder->framerate = av_guess_frame_rate(m->file, stream, NULL);
#ifdef FRAME_PROBE_DEFER_DIMENSIONS
    m->decoder->width = m->decoder->height = 128;
    m->decoder->coded_width = m->decoder->coded_height = 128;
#endif
    return avcodec_open2(m->decoder, codec, NULL);
}

static int interrupted(void *handle) {
    return atomic_load(&((Media *)handle)->interrupted);
}

void *media_open(const char *path, char *error, size_t capacity) {
    Media *m = calloc(1, sizeof(*m));
    if (!m) { media_error(error, capacity, "allocate decoder", AVERROR(ENOMEM)); return NULL; }
    atomic_init(&m->references, 1);
    atomic_init(&m->interrupted, 0);
    m->audio_stream = -1;
    m->file = avformat_alloc_context();
    int rc = AVERROR(ENOMEM);
    if (!m->file) goto failed;
    m->file->interrupt_callback = (AVIOInterruptCB){interrupted, m};
    rc = video_stream(&m->file, path);
    if (rc < 0) goto failed;
    m->stream = rc;
    AVStream *stream = m->file->streams[m->stream];
    AVCodecParameters *p = stream->codecpar;
    const char *name = decoder_name(p);
    if (!name) {
        snprintf(error, capacity, "Unsupported video codec or bit depth");
        media_close(m);
        return NULL;
    }
    m->packet = av_packet_alloc();
    m->frame = av_frame_alloc();
    if (!m->packet || !m->frame) { rc = AVERROR(ENOMEM); goto failed; }
    m->time_base = av_q2d(stream->time_base);
    m->step = av_q2d(av_inv_q(av_guess_frame_rate(m->file, stream, NULL)));
    if (m->step <= 0) m->step = 1.0 / 30.0;
    m->start = stream->start_time == AV_NOPTS_VALUE ? 0 : stream->start_time * m->time_base;
    m->seek_target = -1;
    rc = media_open_decoder(m);
    if (rc < 0) goto failed;
    fprintf(stderr, "Decoder: %s, %dx%d, %.3f fps; hardware required\n",
            name, p->width, p->height, 1.0 / m->step);
    if (stream->duration != AV_NOPTS_VALUE && stream->duration > 0)
        m->duration = stream->duration * m->time_base;
    else if (m->file->duration != AV_NOPTS_VALUE && m->file->duration > 0)
        m->duration = m->file->duration / (double)AV_TIME_BASE;
    media_read_metadata(m);
    return m;
failed:
    media_error(error, capacity, "open native decoder", rc);
    media_close(m);
    return NULL;
}

double media_duration(void *handle) {
    return ((Media *)handle)->duration;
}

void media_metadata(void *handle, int fields[3]) {
    Media *m = handle;
    for (int i = 0; i < 3; i++) fields[i] = m->metadata[i];
}

void media_stop(void *handle) {
    Media *m = handle;
    media_reader_close(m);
    audio_close(m->audio);
    m->audio = NULL;
}
