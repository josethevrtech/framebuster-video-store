use crate::{
    offscreen::Offscreen,
    presentation::{Presentation, Projection, Stereo},
    render_fixture::upload_luma,
};

#[test]
#[ignore = "requires Frame Vulkan, hardware decoding and ffmpeg"]
fn fisheye_pixels_match_angular_mapping_centers_distortion_and_fov_cutoff() {
    let mut render = Offscreen::new((129, 129), 1).unwrap();
    render.renderer.stats = Some(Default::default());
    let luma: Vec<_> = (0..128 * 128)
        .map(|i| (40 + i % 128 + i / 128 / 2) as u8)
        .collect();
    upload_luma(&mut render.renderer, [128, 128], &luma);
    for degrees in [180, 190] {
        for centers in [[[0.5; 2]; 2], [[0.47, 0.52]; 2]] {
            for distortion in [[0.0; 4], [0.01, -0.002, 0.003, 0.0]] {
                let mut p = Presentation {
                    projection: Projection::Fisheye(degrees),
                    stereo: Stereo::Mono,
                    ..Default::default()
                };
                p.lens.centers = centers;
                p.lens.distortion = distortion;
                render.draw(p).unwrap();
                for [x, y] in [[64, 64], [100, 64], [64, 100], [25, 30]] {
                    let ray = [
                        2.0 * (x as f32 + 0.5) / 129.0 - 1.0,
                        2.0 * (y as f32 + 0.5) / 129.0 - 1.0,
                    ];
                    let radial = ray[0].hypot(ray[1]);
                    let project = |angle: f32| {
                        angle
                            + distortion
                                .iter()
                                .enumerate()
                                .map(|(i, k)| k * angle.powi(3 + 2 * i as i32))
                                .sum::<f32>()
                    };
                    let scale = 0.5 * project(radial.atan())
                        / project((degrees as f32).to_radians() * 0.5)
                        / radial.max(0.000001);
                    let u = centers[0][0] + ray[0] * scale;
                    let v = centers[0][1] - ray[1] * scale;
                    let expected = 40.0 + (u * 128.0 - 0.5) + (v * 128.0 - 0.5) * 0.5;
                    let pixel = render.pixel([x, y]);
                    assert!(
                        pixel[..3]
                            .iter()
                            .all(|&value| (value as f32 - expected).abs() <= 3.0),
                        "{p:?} at {x},{y}: {pixel:?} vs {expected}"
                    );
                }
            }
        }
    }
    let mut p = Presentation {
        projection: Projection::Fisheye(190),
        stereo: Stereo::Mono,
        ..Default::default()
    };
    for (yaw, black) in [(92.0, false), (96.0, true), (180.0, true)] {
        p.alignment.yaw = yaw;
        render.draw(p).unwrap();
        assert_eq!(render.pixel([64, 64]) == [0, 0, 0, 255], black);
    }
    let report = render.renderer.stats.as_mut().unwrap().report("test", 1.0);
    assert!(report.contains("gpu_draw_ms_n=11 "));
    assert!(report.contains("gpu_fence_wait_ms_n=11 "));
}
