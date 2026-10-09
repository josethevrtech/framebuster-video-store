#include "../native/media_internal.h"
#include <libavutil/spherical.h>
#include <libavutil/stereo3d.h>
#undef NDEBUG
#include <assert.h>
#include <math.h>

static void expect(Media *m, int projection, int stereo, int swap) {
    int fields[3];
    media_read_metadata(m);
    media_metadata(m, fields);
    if (fields[0] != projection || fields[1] != stereo || fields[2] != swap)
        fprintf(stderr, "Expected [%d, %d, %d], got [%d, %d, %d]\n",
                projection, stereo, swap, fields[0], fields[1], fields[2]);
    assert(fields[0] == projection && fields[1] == stereo && fields[2] == swap);
}

int main(void) {
    Media m = {.file = avformat_alloc_context(), .stream = 1};
    assert(m.file);
    assert(avformat_new_stream(m.file, NULL));
    AVStream *stream = avformat_new_stream(m.file, NULL);
    assert(stream);
    AVFrame frame = {.sample_aspect_ratio = {64, 45}};
    assert(fabsf(media_sample_aspect_ratio(&m, &frame) - 64.0f / 45) < 1e-6f);
    stream->sample_aspect_ratio = (AVRational){16, 15};
    assert(fabsf(media_sample_aspect_ratio(&m, &frame) - 16.0f / 15) < 1e-6f);
    stream->sample_aspect_ratio = (AVRational){0, 1};
    for (int numerator = -1; numerator <= 0; numerator++) {
        frame.sample_aspect_ratio = (AVRational){numerator, 1};
        assert(media_sample_aspect_ratio(&m, &frame) == 1.0f);
    }
    frame.sample_aspect_ratio = (AVRational){1, 0};
    assert(media_sample_aspect_ratio(&m, &frame) == 1.0f);
    AVCodecParameters *p = stream->codecpar;
    expect(&m, -1, -1, -1);
    size_t size;
    AVStereo3D *stereo = av_stereo3d_alloc();
    assert(stereo);
    assert(av_packet_side_data_add(&p->coded_side_data, &p->nb_coded_side_data,
                                  AV_PKT_DATA_STEREO3D, stereo, sizeof(*stereo), 0));
    for (int type = 0; type < 3; ++type) {
        stereo->type = type == 0 ? AV_STEREO3D_2D :
                       type == 1 ? AV_STEREO3D_SIDEBYSIDE : AV_STEREO3D_TOPBOTTOM;
        stereo->flags = AV_STEREO3D_FLAG_INVERT;
        expect(&m, -1, type, type != 0);
    }
    stereo->flags = 0;
    expect(&m, -1, 2, 0);
    stereo->type = AV_STEREO3D_CHECKERBOARD;
    expect(&m, -1, -2, -2);
    stereo->type = AV_STEREO3D_SIDEBYSIDE;
    stereo->view = AV_STEREO3D_VIEW_LEFT;
    expect(&m, -1, -2, -2);
    stereo->view = AV_STEREO3D_VIEW_PACKED;
    AVSphericalMapping *sphere = av_spherical_alloc(&size);
    assert(sphere);
    sphere->projection = AV_SPHERICAL_EQUIRECTANGULAR;
    sphere->yaw = sphere->pitch = sphere->roll = 0;
    assert(av_packet_side_data_add(&p->coded_side_data, &p->nb_coded_side_data,
                                  AV_PKT_DATA_SPHERICAL, sphere, size, 0));
    expect(&m, 2, 1, 0);
    sphere->projection = AV_SPHERICAL_EQUIRECTANGULAR_TILE;
    sphere->bound_left = sphere->bound_right = 1U << 30;
    expect(&m, 1, 1, 0);
    sphere->bound_top = 1;
    expect(&m, -2, 1, 0);
    sphere->projection = AV_SPHERICAL_CUBEMAP;
    expect(&m, -2, 1, 0);
    sphere->projection = AV_SPHERICAL_EQUIRECTANGULAR;
    sphere->pitch = 65536;
    expect(&m, -2, 1, 0);
    m.stream = 0;
    expect(&m, -1, -1, -1);
    avformat_free_context(m.file);
    puts("Presentation metadata checks passed");
}
