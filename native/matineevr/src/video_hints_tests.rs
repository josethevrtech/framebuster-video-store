use super::*;
use Hint::{Conflict, Known, Unknown};

#[test]
fn filename_tokens_are_bounded_case_insensitive_and_independent() {
    for (name, projection, stereo, swap_eyes) in [
        (
            "walk_SbS_180.mp4",
            Known(Projection::Hemisphere),
            Known(Stereo::SideBySide),
            Known(false),
        ),
        (
            "walk-VR180-RL.mkv",
            Known(Projection::Hemisphere),
            Known(Stereo::SideBySide),
            Known(true),
        ),
        (
            "walk 360 BT.mp4",
            Known(Projection::Sphere),
            Known(Stereo::TopBottom),
            Known(true),
        ),
        (
            "walk_TB.mp4",
            Unknown,
            Known(Stereo::TopBottom),
            Known(false),
        ),
        ("walk_2D.mp4", Unknown, Known(Stereo::Mono), Known(false)),
        (
            "walk_180.mp4",
            Known(Projection::Hemisphere),
            Unknown,
            Unknown,
        ),
        ("walk180_sbsish_1800.mp4", Unknown, Unknown, Unknown),
        ("фильм180.mp4", Unknown, Unknown, Unknown),
        ("/360/SBS/walk.mp4", Unknown, Unknown, Unknown),
        (
            "walk_FISHEYE190.mp4",
            Known(Projection::Fisheye(190)),
            Unknown,
            Unknown,
        ),
        (
            "walk_fisheye.mp4",
            Known(Projection::Fisheye(180)),
            Unknown,
            Unknown,
        ),
    ] {
        assert_eq!(
            Hints::filename(Path::new(name)),
            Hints {
                projection,
                stereo,
                swap_eyes,
                ..Default::default()
            }
        );
    }
}

#[test]
fn half_stereo_hints_require_complete_tokens() {
    for (name, stereo) in [
        ("Pokemon-halfsbs-sample.mkv", Stereo::SideBySide),
        ("film.HaLf-SbS.mkv", Stereo::SideBySide),
        ("film-halftb.mkv", Stereo::TopBottom),
        ("film.HaLf-Tb.mkv", Stereo::TopBottom),
    ] {
        let hints = Hints::filename(Path::new(name));
        assert_eq!(hints.stereo, Known(stereo));
        let formats = if stereo == Stereo::SideBySide {
            [hints.sbs_format, hints.tb_format]
        } else {
            [hints.tb_format, hints.sbs_format]
        };
        assert_eq!(formats, [Known(StereoFormat::Half), Unknown]);
        assert_eq!(hints.projection, Unknown);
    }
    for name in [
        "halfsbsish.mkv",
        "nothalfsbs.mkv",
        "behalf-sbs.mkv",
        "half-sbsish.mkv",
        "/half-sbs/film.mkv",
        "half-film-sbs.mkv",
        "Immortals 3D-SBS 1080p-Sample.mkv",
        "halftbish.mkv",
        "nothalftb.mkv",
        "behalf-tb.mkv",
        "half-tbish.mkv",
        "/half-tb/film.mkv",
        "half-film-tb.mkv",
    ] {
        let hints = Hints::filename(Path::new(name));
        assert_eq!([hints.sbs_format, hints.tb_format], [Unknown; 2]);
    }
}

#[test]
fn conflicting_or_unsupported_hints_cannot_become_a_known_projection() {
    for name in [
        "film_180_360_180.mp4",
        "film_fisheye_180.mp4",
        "film_eac360_360.mp4",
        "film_fisheye190_fisheye.mp4",
        "film_rf52.mp4",
    ] {
        assert_eq!(Hints::filename(Path::new(name)).projection, Conflict);
    }
    let filename = Hints::filename(Path::new("film_180_LR_RL.mp4"));
    let metadata = Hints::native([2, 1, 0]);
    let hints = filename.combine(metadata);
    assert_eq!(hints.projection, Conflict);
    assert_eq!(hints.stereo, Known(Stereo::SideBySide));
    assert_eq!(hints.swap_eyes, Conflict);
    assert_eq!(filename.combine(metadata), metadata.combine(filename));
    assert_eq!(hints.combine(filename).projection, Conflict);
    assert_eq!(
        Hints::native([-2, -2, -2]).combine(filename).projection,
        Conflict
    );
}

#[test]
fn missing_fields_fall_back_without_turning_absence_into_mono() {
    let fallback = Presentation {
        projection: Projection::Flat,
        stereo: Stereo::TopBottom,
        swap_eyes: true,
        ..Default::default()
    };
    assert_eq!(Hints::native([-1; 3]).apply(fallback), fallback);
    assert_eq!(
        Hints::filename(Path::new("film_180.mp4")).apply(fallback),
        Presentation {
            projection: Projection::Hemisphere,
            ..fallback
        }
    );
    assert_eq!(
        Hints::native([2, 1, 0]).apply(fallback),
        Presentation {
            projection: Projection::Sphere,
            stereo: Stereo::SideBySide,
            swap_eyes: false,
            ..Default::default()
        }
    );
}
