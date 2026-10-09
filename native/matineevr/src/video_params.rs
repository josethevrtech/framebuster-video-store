use crate::presentation::{Presentation, Projection, Stereo, StereoFormat};
use openxr as xr;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct View {
    pub fov: [f32; 4],
    pub rotation: [f32; 4],
    pub framing: [f32; 4],
    pub stereo_offset: [f32; 4],
    pub aspect: f32,
    pub projection: i32,
    pub stereo: i32,
    pub eye: i32,
    pub lens: [f32; 4],
    pub distortion: [f32; 4],
}

impl View {
    pub fn new(
        size: (i32, i32),
        sample_aspect_ratio: f32,
        view: &xr::View,
        eye: i32,
        presentation: Presentation,
        yaw: f32,
    ) -> Self {
        let f = view.fov;
        let q = view.pose.orientation;
        let a = presentation.alignment;
        let offset = if presentation.stereo == Stereo::Mono {
            0.0
        } else {
            eye as f32 - 0.5
        };
        let eye = eye ^ i32::from(presentation.swap_eyes);
        let flat = presentation.projection == Projection::Flat;
        Self {
            fov: [
                f.angle_left.tan(),
                f.angle_right.tan(),
                f.angle_down.tan(),
                f.angle_up.tan(),
            ],
            rotation: a.rotation([q.x, q.y, q.z, q.w], yaw),
            framing: [a.position[0], a.position[1], (-a.zoom_log2).exp2(), 0.0],
            stereo_offset: [
                a.stereo_offset[0] * offset,
                a.stereo_offset[1] * offset,
                0.0,
                0.0,
            ],
            aspect: size.0 as f32 / size.1 as f32
                * if flat { sample_aspect_ratio } else { 1.0 }
                * match presentation.stereo {
                    Stereo::SideBySide
                        if !flat || presentation.sbs_format == StereoFormat::Full =>
                    {
                        0.5
                    }
                    Stereo::TopBottom if !flat || presentation.tb_format == StereoFormat::Full => {
                        2.0
                    }
                    _ => 1.0,
                },
            projection: match presentation.projection {
                Projection::Flat => 0,
                Projection::Hemisphere => 1,
                Projection::Sphere => 2,
                Projection::Fisheye(_) => 3,
            },
            stereo: presentation.stereo as i32,
            eye,
            lens: if let Projection::Fisheye(fov) = presentation.projection {
                presentation.lens.parameters(
                    fov,
                    if presentation.stereo == Stereo::Mono {
                        0
                    } else {
                        eye as usize
                    },
                )
            } else {
                [0.0; 4]
            },
            distortion: presentation.lens.distortion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_formats_correct_only_flat_aspect_and_preserve_eye_selection() {
        for projection in [
            Projection::Flat,
            Projection::Hemisphere,
            Projection::Sphere,
            Projection::Fisheye(190),
        ] {
            for (stereo, aspect) in [
                (Stereo::Mono, 16.0 / 9.0),
                (Stereo::SideBySide, 8.0 / 9.0),
                (Stereo::TopBottom, 32.0 / 9.0),
            ] {
                for (sbs_format, tb_format) in [
                    (StereoFormat::Full, StereoFormat::Full),
                    (StereoFormat::Half, StereoFormat::Full),
                    (StereoFormat::Full, StereoFormat::Half),
                    (StereoFormat::Half, StereoFormat::Half),
                ] {
                    let p = Presentation {
                        projection,
                        stereo,
                        sbs_format,
                        tb_format,
                        swap_eyes: true,
                        ..Default::default()
                    };
                    let corrects = projection == Projection::Flat && stereo != Stereo::Mono;
                    assert_eq!(
                        p.fields()
                            .contains(&crate::presentation::Field::StereoFormat),
                        corrects
                    );
                    for eye in 0..2 {
                        let params =
                            View::new((1920, 1080), 1.0, &xr::View::default(), eye, p, 0.0);
                        assert_eq!(
                            params.aspect,
                            if corrects
                                && (if stereo == Stereo::TopBottom {
                                    tb_format
                                } else {
                                    sbs_format
                                }) == StereoFormat::Half
                            {
                                16.0 / 9.0
                            } else {
                                aspect
                            }
                        );
                        assert_eq!(params.stereo, stereo as i32);
                        assert_eq!(params.eye, eye ^ 1);
                    }
                }
            }
        }
    }

    #[test]
    fn flat_video_uses_sample_aspect_and_full_stereo_dimensions() {
        for (width, height, sar, stereo, expected) in [
            (720, 576, 64.0 / 45.0, Stereo::Mono, 16.0 / 9.0),
            (720, 576, 16.0 / 15.0, Stereo::Mono, 4.0 / 3.0),
            (1440, 576, 64.0 / 45.0, Stereo::SideBySide, 16.0 / 9.0),
            (720, 1152, 64.0 / 45.0, Stereo::TopBottom, 16.0 / 9.0),
            (3840, 1080, 1.0, Stereo::SideBySide, 16.0 / 9.0),
            (1920, 2160, 1.0, Stereo::TopBottom, 16.0 / 9.0),
        ] {
            let p = Presentation {
                projection: Projection::Flat,
                stereo,
                ..Default::default()
            };
            assert!(
                (View::new((width, height), sar, &xr::View::default(), 0, p, 0.0).aspect
                    - expected)
                    .abs()
                    < 1e-6
            );
        }
    }
}
