use crate::{
    alignment::Mode,
    offscreen::Offscreen,
    presentation::{Presentation, Projection, Stereo},
    render_fixture::upload_luma,
};

#[test]
#[ignore = "requires Frame Vulkan, hardware decoding and ffmpeg"]
fn every_adjustment_moves_pixels_in_all_supported_projections() {
    for projection in [
        Projection::Flat,
        Projection::Hemisphere,
        Projection::Sphere,
        Projection::Fisheye(190),
    ] {
        for stereo in [Stereo::Mono, Stereo::SideBySide, Stereo::TopBottom] {
            for eyes in [1, 2] {
                let mut render = Offscreen::new((128, 128), eyes).unwrap();
                let luma: Vec<_> = (0..128 * 128)
                    .map(|i| (40 + i % 128 + i / 128 / 2) as u8)
                    .collect();
                upload_luma(&mut render.renderer, [128, 128], &luma);
                let direction = match stereo {
                    Stereo::Mono => 0,
                    _ if eyes == 1 => -1,
                    _ => 1,
                };
                for swap_eyes in [false, true] {
                    let p = Presentation {
                        projection,
                        stereo,
                        swap_eyes,
                        ..Default::default()
                    };
                    for (mode, directions) in [
                        (Mode::Orientation, [1, -1, 1, -1]),
                        (Mode::Position, [direction, direction, 1, -1]),
                    ] {
                        for (axis, direction) in directions.into_iter().enumerate() {
                            let point = if mode == Mode::Orientation && axis >= 2 {
                                [96, 72]
                            } else {
                                [64, 64]
                            };
                            let mut adjusted = p;
                            let mut sticks = [[0.0; 2]; 2];
                            sticks[axis / 2][axis % 2] = 1.0;
                            let seconds = match (mode, axis) {
                                (Mode::Orientation, 0..=2) | (Mode::Position, 2..=3) => 0.4,
                                _ => 4.0,
                            };
                            adjusted.alignment.adjust(mode, sticks, seconds, stereo);
                            let baseline = pixel(&mut render, p, point);
                            let value = pixel(&mut render, adjusted, point);
                            assert_eq!(
                                value.cmp(&baseline),
                                direction.cmp(&0),
                                "{p:?} eye={eyes} {mode:?} axis={axis}: {value} vs {baseline}"
                            );
                        }
                    }
                }
            }
        }
    }
}

fn pixel(render: &mut Offscreen, presentation: Presentation, [x, y]: [i32; 2]) -> u8 {
    render.draw(presentation).unwrap();
    let pixel = render.pixel([x, y]);
    render.finish().unwrap();
    assert_eq!(pixel[3], 255);
    pixel[0]
}
