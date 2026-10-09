use super::*;
use crate::presentation::{Projection, Stereo, StereoFormat};

fn play(settings: &mut Settings, path: &str, metadata: Hints) -> Presentation {
    let path = Path::new(path);
    let mut current = settings.open(path);
    settings.infer(path, metadata, &mut current);
    settings.uploaded(path, current);
    current
}

#[test]
fn stereo_formats_override_detection_and_are_remembered_until_exit() {
    let unknown = Hints::default();
    for path in ["film-halfsbs.mkv", "film-halftb.mkv"] {
        let mut settings = Settings::new(unknown);
        let detected = play(&mut settings, path, unknown);
        assert_eq!(
            detected.sbs_format == StereoFormat::Half,
            path.contains("sbs")
        );
        assert_eq!(
            detected.tb_format == StereoFormat::Half,
            path.contains("tb")
        );
        let p = Presentation {
            sbs_format: StereoFormat::Full,
            tb_format: StereoFormat::Full,
            ..detected
        };
        settings.edit(Path::new(path), p);
        play(&mut settings, "other-half-sbs-half-tb.mkv", unknown);
        assert_eq!(play(&mut settings, path, unknown), p);
        assert_eq!(play(&mut Settings::new(unknown), path, unknown), detected);
        settings = Settings::new(Hints {
            sbs_format: Hint::Known(StereoFormat::Full),
            tb_format: Hint::Known(StereoFormat::Full),
            ..Default::default()
        });
        assert_eq!(play(&mut settings, path, unknown), p);
        settings.edit(Path::new(path), detected);
        assert_eq!(play(&mut settings, path, unknown), detected);
    }
}

#[test]
fn every_played_video_keeps_its_complete_settings_for_the_session() {
    let mut settings = Settings::new(Hints::default());
    let a = Path::new("A.mp4");
    let initial = play(&mut settings, "A.mp4", Hints::default());
    let b = play(&mut settings, "B_360_TB.mp4", Hints::default());
    assert_eq!(b.projection, Projection::Sphere);
    assert_eq!(b.stereo, Stereo::TopBottom);
    assert_eq!(play(&mut settings, "A.mp4", Hints::default()), initial);
    let edited = Presentation {
        swap_eyes: true,
        ..initial
    };
    settings.edit(a, edited);
    assert_eq!(play(&mut settings, "C.mp4", Hints::default()), edited);
    assert_eq!(play(&mut settings, "B_360_TB.mp4", Hints::default()), b);
    assert_eq!(
        play(&mut settings, "A.mp4", Hints::native([2, 2, 0])),
        edited
    );
    assert_eq!(
        play(
            &mut Settings::new(Hints::default()),
            "A.mp4",
            Hints::default()
        ),
        initial
    );
}

#[test]
fn fisheye_calibration_is_remembered_per_file_and_manual_settings_win() {
    let mut settings = Settings::new(Hints::default());
    let path = "A_FISHEYE190.mp4";
    let mut p = play(&mut settings, path, Hints::default());
    assert_eq!(p.projection, Projection::Fisheye(190));
    p.projection = Projection::Fisheye(188);
    p.lens.centers = [[0.49, 0.51], [0.52, 0.48]];
    p.lens.distortion = [0.01, -0.002, 0.003, 0.0];
    settings.edit(Path::new(path), p);
    assert_eq!(
        play(&mut settings, "B_FISHEYE190.mp4", Hints::default()).lens,
        Default::default()
    );
    assert_eq!(play(&mut settings, path, Hints::default()), p);
}

#[test]
fn first_opens_use_explicit_options_then_hints_then_last_used_fields() {
    let mut settings = Settings::new(Hints {
        projection: Hint::Known(Projection::Flat),
        ..Default::default()
    });
    let a = play(&mut settings, "A_360_RL.mp4", Hints::native([2, 1, 1]));
    assert_eq!(
        a,
        Presentation {
            projection: Projection::Flat,
            stereo: Stereo::SideBySide,
            swap_eyes: true,
            ..Default::default()
        }
    );
    let b = play(&mut settings, "B_TB.mp4", Hints::default());
    assert_eq!(
        b,
        Presentation {
            stereo: Stereo::TopBottom,
            swap_eyes: false,
            ..a
        }
    );
    let manual = Presentation {
        projection: Projection::Hemisphere,
        ..b
    };
    settings.edit(Path::new("B_TB.mp4"), manual);
    assert_eq!(play(&mut settings, "B_TB.mp4", Hints::default()), manual);
    let next = play(&mut settings, "C.mp4", Hints::default());
    assert_eq!(
        next,
        Presentation {
            projection: Projection::Flat,
            ..manual
        }
    );
}

#[test]
fn failed_or_unuploaded_opens_do_not_change_history_or_fallback() {
    let mut settings = Settings::new(Hints::default());
    let a = play(&mut settings, "A_360_BT.mp4", Hints::default());
    let bad = Path::new("bad.mp4");
    let mut current = settings.open(bad);
    settings.infer(bad, Hints::native([0, 0, 0]), &mut current);
    settings.edit(bad, current);
    assert_eq!(settings.open(bad), a);
    assert_eq!(play(&mut settings, "C.mp4", Hints::default()), a);
}

#[test]
fn early_manual_change_and_reset_block_late_inference() {
    let mut settings = Settings::new(Hints::default());
    let path = Path::new("A_360_TB.mp4");
    let mut current = settings.open(path);
    let mut row = current.fields().len() - 1;
    crate::presentation::navigate(
        &mut current,
        &mut row,
        crate::navigation::Navigation {
            horizontal: 1,
            vertical: 0,
        },
    );
    settings.edit(path, current);
    settings.infer(path, Hints::native([2, 2, 1]), &mut current);
    assert_eq!(current, Presentation::default());
    settings.uploaded(path, current);
    assert_eq!(
        play(&mut settings, "A_360_TB.mp4", Hints::default()),
        current
    );
    settings.edit(
        path,
        Presentation {
            stereo: Stereo::Mono,
            ..current
        },
    );
    settings.uploaded(path, current);
    assert_eq!(settings.open(path).stereo, Stereo::Mono);
}

#[test]
fn alignment_survives_inference_and_reopening_but_never_leaks_to_another_video() {
    let mut settings = Settings::new(Hints::default());
    let path = Path::new("A_360_TB.mp4");
    let mut a = settings.open(path);
    a.alignment.pitch = 15.0;
    settings.remember(path, a);
    settings.infer(path, Hints::default(), &mut a);
    assert_eq!(a.projection, Projection::Sphere);
    assert_eq!(a.stereo, Stereo::TopBottom);
    assert_eq!(a.alignment.pitch, 15.0);
    settings.uploaded(path, a);
    assert_eq!(
        play(&mut settings, "B.mp4", Hints::default()).alignment,
        Default::default()
    );
    assert_eq!(play(&mut settings, "A_360_TB.mp4", Hints::default()), a);
    a.alignment = Default::default();
    settings.remember(path, a);
    assert_eq!(play(&mut settings, "A_360_TB.mp4", Hints::default()), a);
}
