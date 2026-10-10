#include "media_internal.h"
#include <libavutil/spherical.h>
#include <libavutil/stereo3d.h>

float media_sample_aspect_ratio(Media *m, AVFrame *frame) {
    AVRational sar = av_guess_sample_aspect_ratio(m->file, m->file->streams[m->stream], frame);
    return sar.num > 0 && sar.den > 0 ? av_q2d(sar) : 1.0;
}

void media_read_metadata(Media *m) {
    int *fields = m->metadata;
    const AVCodecParameters *p = m->file->streams[m->stream]->codecpar;
    fields[0] = fields[1] = fields[2] = -1;
    const AVPacketSideData *data = av_packet_side_data_get(
        p->coded_side_data, p->nb_coded_side_data, AV_PKT_DATA_STEREO3D);
    if (data && data->size >= sizeof(AVStereo3D)) {
        const AVStereo3D *s = (const AVStereo3D *)data->data;
        fields[1] = s->type == AV_STEREO3D_2D ? 0 :
                    s->type == AV_STEREO3D_SIDEBYSIDE ? 1 :
                    s->type == AV_STEREO3D_TOPBOTTOM ? 2 : -2;
        if (s->view != AV_STEREO3D_VIEW_PACKED) fields[1] = -2;
        fields[2] = fields[1] < 0 ? -2 :
                    fields[1] == 0 ? 0 : !!(s->flags & AV_STEREO3D_FLAG_INVERT);
    }
    data = av_packet_side_data_get(
        p->coded_side_data, p->nb_coded_side_data, AV_PKT_DATA_SPHERICAL);
    if (data && data->size >= sizeof(AVSphericalMapping)) {
        const AVSphericalMapping *s = (const AVSphericalMapping *)data->data;
        fields[0] = -2;
        if (s->yaw || s->pitch || s->roll) return;
        if (s->projection == AV_SPHERICAL_EQUIRECTANGULAR) fields[0] = 2;
        if (s->projection == AV_SPHERICAL_EQUIRECTANGULAR_TILE && !s->bound_top && !s->bound_bottom) {
            if (!s->bound_left && !s->bound_right) fields[0] = 2;
            if (s->bound_left == (1U << 30) && s->bound_right == (1U << 30)) fields[0] = 1;
        }
    }
}
