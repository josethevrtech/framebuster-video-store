use crate::{
    offscreen::Offscreen,
    presentation::{Presentation, Projection, Stereo},
    preview::SIZE,
    render_fixture::upload_luma,
};

#[test]
#[ignore = "requires Frame Vulkan, hardware decoding and ffmpeg"]
fn preview_projects_one_eye_and_preserves_normal_rendering() {
    verify(Offscreen::new((64, 64), 2).unwrap());
}

fn verify(mut render: Offscreen) {
    render.preview_renderer.stats = Some(Default::default());
    let mut luma = [224; 16 * 16];
    upload_luma(&mut render.renderer, [16, 16], &luma);
    for projection in [
        Projection::Flat,
        Projection::Hemisphere,
        Projection::Sphere,
        Projection::Fisheye(190),
    ] {
        for stereo in [Stereo::Mono, Stereo::SideBySide, Stereo::TopBottom] {
            for (index, value) in luma.iter_mut().enumerate() {
                let second = match stereo {
                    Stereo::Mono => false,
                    Stereo::SideBySide => index % 16 >= 8,
                    Stereo::TopBottom => index / 16 >= 8,
                };
                *value = if second { 192 } else { 64 };
            }
            for swap_eyes in [false, true] {
                upload_luma(&mut render.preview_renderer, [16, 16], &luma);
                let presentation = Presentation {
                    projection,
                    stereo,
                    swap_eyes,
                    ..Default::default()
                };
                render.draw_preview(presentation).unwrap();
                render.finish().unwrap();
                let pixel = render.pixel(SIZE.map(|v| v as i32 / 2));
                let expected = if swap_eyes && stereo != Stereo::Mono {
                    192
                } else {
                    64
                };
                assert!(
                    pixel[..3]
                        .iter()
                        .all(|&value| value.abs_diff(expected) <= 1),
                    "{presentation:?}: {pixel:?}"
                );
                assert_eq!(pixel[3], 255);
                render.draw(presentation).unwrap();
                render.finish().unwrap();
                assert_eq!(render.pixel([32, 32]), [224, 224, 224, 255]);
            }
        }
    }
    render.draw_preview(Presentation::default()).unwrap();
    assert_eq!(render.pixel([128, 72]), [0, 0, 0, 255]);
    let report = render
        .preview_renderer
        .stats
        .as_mut()
        .unwrap()
        .report("test", 1.0);
    assert!(report.contains("pipeline_create_ms_n=1 "));
}
