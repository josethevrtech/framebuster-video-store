use crate::{
    app::App,
    app_tests::wait,
    hud_text::{Canvas, HEIGHT, WIDTH},
    input::Controls,
    navigation::Navigation,
    options::Options,
    presentation::{Projection, Stereo},
    shortcuts::PanelKind::{Browser, Information, Settings},
    snapshot,
};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

pub(crate) fn first_frame(app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(Instant::now() < deadline, "First frame timed out");
        app.update(Controls::default());
        if app.frame().is_some() {
            app.video_uploaded();
            return;
        }
        assert!(app.browser.message.is_empty(), "{}", app.browser.message);
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[ignore = "requires Frame hardware decoder and FRAME_TEST_MEDIA_DIR fixtures"]
fn browse_during_playback_adjust_settings_and_switch_decoders() {
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    let options =
        Options::from_args([directory.join("h264.mp4").to_str().unwrap().into()].into_iter())
            .unwrap()
            .unwrap();
    let mut app = App::new(&options);
    first_frame(&mut app);
    wait(&mut app, |a| !a.browser.loading());
    assert_eq!(app.panels(), [None; 3]);
    for expected in [Some(Information), None] {
        app.update(Controls {
            y: true,
            ..Default::default()
        });
        assert_eq!(app.panels(), [None, None, expected]);
        app.update(Controls::default());
    }
    let position = app.playback.as_ref().unwrap().position;
    let right = Controls {
        grips: [false, true],
        ..Default::default()
    };
    app.update(right);
    assert_eq!(app.panels(), [None, Some(Settings), None]);
    app.update(right);
    assert!(!app.changed);
    app.update(Controls {
        dpad: Navigation {
            horizontal: 1,
            vertical: 1,
        },
        ..right
    });
    assert_eq!(app.presentation.projection, Projection::Sphere);
    app.update(Controls {
        stick: Navigation {
            horizontal: 0,
            vertical: 1,
        },
        ..right
    });
    app.update(Controls {
        stick: Navigation {
            horizontal: 1,
            vertical: 0,
        },
        ..right
    });
    assert_eq!(app.presentation.stereo, Stereo::TopBottom);
    assert_eq!(app.playback.as_ref().unwrap().status(), "Playing");
    assert!(app.playback.as_ref().unwrap().position >= position);
    app.update(Controls::default());
    assert_eq!(app.panels(), [None; 3]);
    assert_eq!(app.presentation.projection, Projection::Sphere);
    if let Some(path) = std::env::var_os("FRAME_TEST_SETTINGS_SNAPSHOT") {
        let mut canvas = Canvas::new(&app.path, [WIDTH, HEIGHT]);
        canvas.settings(app.presentation, app.settings_row);
        snapshot::save_pixels(
            &PathBuf::from(path),
            crate::hud_text::WIDTH as i32,
            crate::hud_text::HEIGHT as i32,
            &canvas.pixels,
        )
        .unwrap();
    }
    app.browser.selected = app
        .browser
        .entries
        .iter()
        .position(|entry| entry.path.ends_with("hevc.mp4"))
        .unwrap();
    let both = Controls {
        grips: [true, true],
        ..Default::default()
    };
    app.update(both);
    assert_eq!(app.panels(), [Some(Browser), Some(Settings), None]);
    app.update(Controls {
        dpad: Navigation {
            horizontal: 1,
            vertical: 0,
        },
        ..both
    });
    assert!(app.stopping());
    assert!(app.playback.is_none());
    first_frame(&mut app);
    assert_eq!(app.path, directory.join("hevc.mp4"));
    assert_eq!(app.presentation.projection, Projection::Sphere);
    assert_eq!(app.presentation.stereo, Stereo::TopBottom);
    app.stop();
    wait(&mut app, |a| !a.stopping());
}
