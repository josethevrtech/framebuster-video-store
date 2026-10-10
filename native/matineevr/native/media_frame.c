#include "media_internal.h"
#include <libavutil/hwcontext_drm.h>
#include <libavutil/opt.h>

static int validate_frame(AVFrame *f, char *error, size_t capacity) {
    if (f->decode_error_flags || f->flags & AV_FRAME_FLAG_CORRUPT)
        return media_error(error, capacity, "Iris returned decoding errors", AVERROR_INVALIDDATA);
    if (f->format != AV_PIX_FMT_DRM_PRIME)
        return media_error(error, capacity, "Iris did not return DMA-BUF", AVERROR_INVALIDDATA);
    AVDRMFrameDescriptor *d = (void *)f->data[0];
    if (!f->buf[0] || f->buf[0]->data != f->data[0] ||
        f->buf[0]->size < sizeof(*d) || f->width <= 0 || f->height <= 0 ||
        f->crop_left + f->crop_right >= (size_t)f->width ||
        f->crop_top + f->crop_bottom >= (size_t)f->height ||
        d->nb_objects != 1 || d->nb_layers != 1 || d->layers[0].nb_planes != 2 ||
        d->objects[0].fd < 0 || !d->objects[0].size)
        return media_error(error, capacity, "invalid DMA-BUF frame", AVERROR_INVALIDDATA);
    for (int i = 0; i < 2; i++) {
        AVDRMPlaneDescriptor *p = &d->layers[0].planes[i];
        if (p->object_index != 0 || p->offset < 0 ||
            (size_t)p->offset >= d->objects[0].size || p->pitch <= 0)
            return media_error(error, capacity, "invalid DMA-BUF plane", AVERROR_INVALIDDATA);
    }
    return 0;
}

static void frame_descriptor(Frame *out, AVFrame *f, Media *m) {
    const AVDRMFrameDescriptor *d = (const void *)f->data[0];
    const AVDRMLayerDescriptor *layer = &d->layers[0];
    out->dmabuf = (DmaBuf){
        .fd = d->objects[0].fd, .format = layer->format,
        .size = d->objects[0].size, .modifier = d->objects[0].format_modifier,
        .pool = m->pool,
        .offsets = {layer->planes[0].offset, layer->planes[1].offset},
        .pitches = {layer->planes[0].pitch, layer->planes[1].pitch},
        .width = f->width, .height = f->height,
        .crop = {f->crop_left, f->crop_top, f->crop_right, f->crop_bottom},
        .chroma_location = f->chroma_location,
        .transfer = f->color_trc, .primaries = f->color_primaries,
        .stream = m->stream,
    };
}

static int receive_frame(Media *m, char *error, size_t capacity) {
    av_frame_unref(m->frame);
    for (;;) {
        int64_t started = media_trace_start(m);
        int rc = avcodec_receive_frame(m->decoder, m->frame);
        media_trace_end(started, &m->trace.receive_us);
        if (m->tracing) {
            m->trace.receive_calls++;
            m->trace.again += rc == AVERROR(EAGAIN);
            m->trace.frames += rc == 0;
        }
        if (rc == AVERROR_EOF) return 0;
        if (rc == 0) break;
        if (rc != AVERROR(EAGAIN)) return media_error(error, capacity, "Iris receive frame", rc);
        int changed = media_reconfigure(m, 0, error, capacity);
        if (changed < 0) return changed;
        if (m->draining || changed) return 2;
        rc = m->packet->size ? 0 : media_read_packet(m);
        if (rc == AVERROR(EAGAIN)) return 2;
        if (rc < 0 && rc != AVERROR_EOF) return media_error(error, capacity, "read file", rc);
        m->draining = rc == AVERROR_EOF;
        started = media_trace_start(m);
        rc = avcodec_send_packet(m->decoder, m->draining ? NULL : m->packet);
        media_trace_end(started, &m->trace.send_us);
        if (m->tracing) {
            m->trace.send_calls++;
            m->trace.again += rc == AVERROR(EAGAIN);
        }
        if (rc == AVERROR(EAGAIN)) return 2;
        av_packet_unref(m->packet);
        if (rc < 0) return media_error(error, capacity, "Iris send packet", rc);
    }
    return 1;
}

int media_next(void *handle, Frame *out, char *error, size_t capacity) {
    Media *m = handle;
    if (!m->preview_repeat) {
        int rc = receive_frame(m, error, capacity);
        if (rc != 1) return rc;
    }
    m->preview_repeat = 0;
    AVFrame *f = m->frame;
    if (m->seek_target >= 0 && f->best_effort_timestamp == AV_NOPTS_VALUE)
        return media_error(error, capacity, "seek requires frame timestamps", AVERROR_INVALIDDATA);
    double pts = f->best_effort_timestamp == AV_NOPTS_VALUE ? m->fallback_pts :
                 f->best_effort_timestamp * m->time_base - m->start;
    m->fallback_pts = pts + m->step;
    int before = m->seek_target >= 0 && pts < m->seek_target;
    if (before && !m->preview_pending) return 2;
    int rc = validate_frame(f, error, capacity);
    if (rc < 0) {
        if (m->tracing) m->trace.rejected++;
        return before ? 2 : rc;
    }
    int preview = m->preview_pending;
    m->preview_pending = 0;
    m->preview_repeat = preview && !before;
    if (!preview && !before) m->seek_target = -1;
    AVFrame *owned = av_frame_clone(f);
    if (!owned) return media_error(error, capacity, "retain frame", AVERROR(ENOMEM));
    FrameOwner *owner = malloc(sizeof(*owner));
    if (!owner) {
        av_frame_free(&owned);
        return media_error(error, capacity, "allocate frame owner", AVERROR(ENOMEM));
    }
    *owner = (FrameOwner){owned, m};
    atomic_fetch_add(&m->references, 1);
    *out = (Frame){
        .owner = owner,
        .width = owned->width - owned->crop_left - owned->crop_right,
        .height = owned->height - owned->crop_top - owned->crop_bottom,
        .colorspace = owned->colorspace, .full_range = owned->color_range == AVCOL_RANGE_JPEG,
        .sample_aspect_ratio = media_sample_aspect_ratio(m, owned),
        .pts = pts,
    };
    frame_descriptor(out, owned, m);
    return preview ? 3 : 1;
}

void media_release(Frame *frame) {
    FrameOwner *owner = frame->owner;
    av_frame_free(&owner->frame);
    media_close(owner->media);
    free(owner);
    frame->owner = NULL;
}

int media_reconfigure(void *handle, int released, char *error, size_t capacity) {
    Media *m = handle;
    int64_t pending = 0;
    int rc = av_opt_get_int(m->decoder->priv_data, "capture_changed", 0, &pending);
    if (rc < 0) return media_error(error, capacity, "query capture change", rc);
    if (!released || pending != 1) return pending == 1;
    if (atomic_load(&m->references) != 1)
        return media_error(error, capacity, "release frames before reconfiguration", AVERROR(EBUSY));
    av_frame_unref(m->frame);
    m->preview_pending |= m->preview_repeat;
    m->preview_repeat = 0;
    rc = av_opt_set_int(m->decoder->priv_data, "capture_changed", 2, 0);
    if (rc < 0) return media_error(error, capacity, "acknowledge capture change", rc);
    m->pool++;
    return 0;
}
