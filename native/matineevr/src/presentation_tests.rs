use super::*;

const RIGHT: Navigation = Navigation {
    horizontal: 1,
    vertical: 0,
};
const LEFT: Navigation = Navigation {
    horizontal: -1,
    vertical: 0,
};
const DOWN: Navigation = Navigation {
    horizontal: 0,
    vertical: 1,
};

#[test]
fn visible_fields_describe_the_existing_render_modes() {
    for (projection, format) in [
        (Projection::Flat, None),
        (Projection::Hemisphere, Some("180")),
        (Projection::Sphere, Some("360")),
    ] {
        let p = Presentation {
            projection,
            ..Presentation::default()
        };
        let rows: Vec<_> = p.fields().iter().map(|field| field.display(p)).collect();
        let mut expected = vec![(
            "Projection",
            if format.is_some() {
                "Equirectangular"
            } else {
                "Flat"
            },
        )];
        expected.extend(format.map(|value| ("Format", value)));
        expected.push(("Stereo", "Side by side"));
        if projection == Projection::Flat {
            expected.push(("SBS format", "Full"));
        }
        expected.extend([("Eye order", "Normal"), ("Reset", "Restore defaults")]);
        assert_eq!(
            rows.iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert_eq!(Presentation::default().projection, Projection::Hemisphere);
}

#[test]
fn navigation_changes_only_the_selected_field_and_wraps_choices() {
    let mut p = Presentation::default();
    let mut row = 0;
    navigate(&mut p, &mut row, RIGHT);
    assert_eq!(
        p,
        Presentation {
            projection: Projection::Flat,
            ..Presentation::default()
        }
    );
    assert_eq!(row, 0);
    navigate(&mut p, &mut row, LEFT);
    assert_eq!(p, Presentation::default());
    navigate(&mut p, &mut row, DOWN);
    navigate(&mut p, &mut row, RIGHT);
    assert_eq!(
        p,
        Presentation {
            projection: Projection::Sphere,
            ..Presentation::default()
        }
    );
    navigate(&mut p, &mut row, RIGHT);
    assert_eq!(p, Presentation::default());
    navigate(&mut p, &mut row, LEFT);
    assert_eq!(p.projection, Projection::Sphere);
    navigate(&mut p, &mut row, DOWN);
    for stereo in [Stereo::TopBottom, Stereo::Mono, Stereo::SideBySide] {
        navigate(&mut p, &mut row, RIGHT);
        assert_eq!(
            p,
            Presentation {
                projection: Projection::Sphere,
                stereo,
                swap_eyes: false,
                ..Default::default()
            }
        );
    }
    navigate(&mut p, &mut row, LEFT);
    assert_eq!(p.stereo, Stereo::Mono);
    navigate(&mut p, &mut row, DOWN);
    navigate(&mut p, &mut row, RIGHT);
    assert!(p.swap_eyes);
    assert_eq!(p.projection, Projection::Sphere);
    assert_eq!(p.stereo, Stereo::Mono);
    navigate(&mut p, &mut row, LEFT);
    assert!(!p.swap_eyes);
}

#[test]
fn reset_preserves_its_selection_and_navigation_clamps_stale_rows() {
    for projection in [
        Projection::Flat,
        Projection::Hemisphere,
        Projection::Sphere,
        Projection::Fisheye(190),
    ] {
        for direction in [LEFT, RIGHT] {
            let original = Presentation {
                projection,
                stereo: Stereo::Mono,
                sbs_format: StereoFormat::Half,
                tb_format: StereoFormat::Half,
                swap_eyes: true,
                alignment: crate::alignment::Alignment {
                    pitch: 12.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut p = original;
            let mut row = usize::MAX;
            navigate(&mut p, &mut row, DOWN);
            assert_eq!(p, original);
            assert_eq!(p.fields()[row], Field::Reset);
            navigate(&mut p, &mut row, direction);
            assert_eq!(p, Presentation::default());
            assert_eq!(p.fields()[row], Field::Reset);
            navigate(
                &mut p,
                &mut row,
                Navigation {
                    vertical: i8::MIN,
                    ..Navigation::default()
                },
            );
            assert_eq!(row, 0);
            navigate(&mut p, &mut row, Navigation::default());
            assert_eq!(p, Presentation::default());
        }
    }
}

#[test]
fn flat_stereo_changes_show_independent_formats_and_keep_the_selected_field() {
    let mut p = Presentation {
        projection: Projection::Flat,
        ..Default::default()
    };
    let mut row = 2;
    for (stereo, label) in [
        (Stereo::SideBySide, "SBS format"),
        (Stereo::TopBottom, "Top/bottom format"),
    ] {
        p.stereo = stereo;
        for direction in [RIGHT, LEFT] {
            navigate(&mut p, &mut row, direction);
            assert_eq!(p.fields()[row].display(p), (label, "Half".into()));
            assert_eq!(
                p.sbs_format == StereoFormat::Half,
                stereo == Stereo::SideBySide
            );
            assert_eq!(
                p.tb_format == StereoFormat::Half,
                stereo == Stereo::TopBottom
            );
            navigate(&mut p, &mut row, direction);
            assert_eq!(p.fields()[row].display(p), (label, "Full".into()));
        }
    }
    p.stereo = Stereo::SideBySide;
    row = 1;
    for stereo in [Stereo::TopBottom, Stereo::Mono, Stereo::SideBySide] {
        navigate(&mut p, &mut row, RIGHT);
        assert_eq!(p.stereo, stereo);
        assert_eq!(p.fields()[row], Field::Stereo);
        assert_eq!(
            p.fields().contains(&Field::StereoFormat),
            stereo != Stereo::Mono
        );
    }
}
