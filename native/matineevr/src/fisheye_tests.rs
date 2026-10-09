use crate::{
    navigation::Navigation,
    presentation::{Field, Lens, Presentation, Projection, Stereo, navigate},
    video_params::View,
};
use openxr as xr;

#[test]
fn controls_select_fisheye_adjust_independent_parameters_and_clamp() {
    let mut p = Presentation::default();
    let mut row = 0;
    let left = Navigation {
        horizontal: -1,
        vertical: 0,
    };
    navigate(&mut p, &mut row, left);
    assert_eq!(p.projection, Projection::Fisheye(190));
    for (field, expected) in [
        (Field::Format, "189 degrees"),
        (Field::Center(0, 0), "0.499"),
        (Field::Center(0, 1), "0.499"),
        (Field::Center(1, 0), "0.499"),
        (Field::Center(1, 1), "0.499"),
        (Field::Distortion(0), "-0.0001"),
        (Field::Distortion(1), "-0.0001"),
        (Field::Distortion(2), "-0.0001"),
        (Field::Distortion(3), "-0.0001"),
    ] {
        row = p.fields().iter().position(|&f| f == field).unwrap();
        navigate(&mut p, &mut row, left);
        assert_eq!(field.display(p).1, expected);
    }
    assert_eq!(p.lens.centers, [[0.499; 2]; 2]);
    for horizontal in [-127, 127] {
        for field in [Field::Format, Field::Center(0, 0), Field::Distortion(0)] {
            row = p.fields().iter().position(|&f| f == field).unwrap();
            for _ in 0..200 {
                navigate(
                    &mut p,
                    &mut row,
                    Navigation {
                        horizontal,
                        vertical: 0,
                    },
                );
            }
        }
        assert_eq!(
            p.projection,
            Projection::Fisheye(if horizontal < 0 { 1 } else { 359 })
        );
        assert_eq!(p.lens.centers[0][0], if horizontal < 0 { 0.0 } else { 1.0 });
        assert_eq!(p.lens.distortion[0], horizontal.signum() as f32);
    }
}

#[test]
fn source_eye_centers_follow_swapping_and_mono_uses_the_first_image() {
    let view = xr::View {
        pose: xr::Posef::IDENTITY,
        fov: xr::Fovf {
            angle_left: -0.5,
            angle_right: 0.5,
            angle_up: 0.5,
            angle_down: -0.5,
        },
    };
    let lens = Lens {
        centers: [[0.4, 0.5], [0.6, 0.7]],
        distortion: [0.01, -0.002, 0.003, 0.0],
    };
    for stereo in [Stereo::Mono, Stereo::SideBySide, Stereo::TopBottom] {
        for swap_eyes in [false, true] {
            for eye in 0..2 {
                let p = Presentation {
                    projection: Projection::Fisheye(190),
                    lens,
                    stereo,
                    swap_eyes,
                    ..Default::default()
                };
                let params = View::new((8192, 4096), 2.0, &view, eye, p, 0.0);
                let source = if stereo == Stereo::Mono {
                    0
                } else {
                    eye as usize ^ usize::from(swap_eyes)
                };
                assert_eq!(&params.lens[..2], &lens.centers[source]);
                assert_eq!(params.distortion, lens.distortion);
                assert_eq!(params.projection, 3);
                assert_eq!(
                    params.aspect,
                    match stereo {
                        Stereo::Mono => 2.0,
                        Stereo::SideBySide => 1.0,
                        Stereo::TopBottom => 4.0,
                    }
                );
            }
        }
    }
    let radians = 95_f32.to_radians();
    let ideal = Lens::default().parameters(190, 0);
    assert_eq!(ideal, [0.5, 0.5, radians, radians]);
    let radius = radians
        + lens
            .distortion
            .iter()
            .enumerate()
            .map(|(i, k)| k * radians.powi(3 + 2 * i as i32))
            .sum::<f32>();
    assert!((lens.parameters(190, 0)[3] - radius).abs() < 0.000001);
}
