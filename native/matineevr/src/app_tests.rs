use crate::{app::App, browser, hud_text::Canvas, input::Controls, options::Options, snapshot};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

pub fn wait(app: &mut App, ready: impl Fn(&App) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !ready(app) {
        assert!(
            Instant::now() < deadline,
            "Timed out: {}",
            app.browser.message
        );
        app.update(Controls::default());
        app.synchronize(&mut []).unwrap();
        thread::sleep(Duration::from_millis(5));
    }
}

#[test]
#[ignore = "requires FRAME_TEST_MEDIA_DIR fixtures"]
fn native_metadata_filtering_and_browser_error_recovery() {
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    let entries = browser::scan(&directory).unwrap();
    let names: Vec<_> = entries
        .iter()
        .map(|e| e.path.file_name().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "folder",
            "h264.mp4",
            "hevc.mp4",
            "main10.mp4",
            "mixed.mkv",
            "no-extension",
            "vp9.webm"
        ]
    );
    for name in ["broken.mp4", "missing.mp4"] {
        let error = crate::media::supported(&directory.join(name)).unwrap_err();
        assert!(error.to_string().contains("inspect video"));
    }
    assert!(crate::media::supported(&directory.join("main10.mp4")).unwrap());
    assert!(!crate::media::supported(&directory.join("audio.m4a")).unwrap());
    let options =
        Options::from_args([directory.join("broken.mp4").to_str().unwrap().into()].into_iter())
            .unwrap()
            .unwrap();
    let mut app = App::new(&options);
    let deadline = Instant::now() + Duration::from_secs(20);
    while app.playback.is_some() {
        assert!(Instant::now() < deadline);
        app.update(Controls::default());
        app.synchronize(&mut []).unwrap();
        app.frame();
        thread::sleep(Duration::from_millis(5));
    }
    wait(&mut app, |a| !a.stopping() && !a.browser.loading());
    assert!(app.browser.message.contains("open native decoder"));
    assert!(!app.hud);
    if let Some(path) = std::env::var_os("FRAME_TEST_BROWSER_SNAPSHOT") {
        let size = crate::browser_view::panel_size(crate::shortcuts::PanelKind::Browser, false);
        let mut canvas = Canvas::new(&directory, size);
        canvas.browser(&app.browser, false, 0.0);
        snapshot::save_pixels(
            &PathBuf::from(path),
            size[0] as i32,
            size[1] as i32,
            &canvas.pixels,
        )
        .unwrap();
    }
    app.update(Controls {
        b: true,
        ..Default::default()
    });
    assert_eq!(app.browser.directory, directory.parent().unwrap());
}

#[test]
#[ignore = "requires Frame hardware decoder and FRAME_TEST_MEDIA_DIR fixtures"]
fn select_pause_stop_reopen_and_end_of_file() {
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    let options = Options::from_args([directory.to_str().unwrap().into()].into_iter())
        .unwrap()
        .unwrap();
    let mut app = App::new(&options);
    wait(&mut app, |a| !a.browser.loading());
    assert!(app.playback.is_none());
    for (index, name) in ["h264.mp4", "hevc.mp4", "h264.mp4"].into_iter().enumerate() {
        let selected = app
            .browser
            .entries
            .iter()
            .position(|e| e.path.ends_with(name))
            .unwrap();
        while app.browser.selected != selected {
            let down = app.browser.selected < selected;
            app.update(Controls {
                x: down,
                y: !down,
                ..Default::default()
            });
        }
        app.update(Controls {
            a: true,
            ..Default::default()
        });
        wait(&mut app, |a| a.playback.is_some());
        assert_eq!(app.path, directory.join(name));
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            assert!(Instant::now() < deadline, "First frame timed out");
            app.update(Controls::default());
            app.synchronize(&mut []).unwrap();
            if let Some(frame) = app.frame() {
                assert_eq!((frame.pixels.width, frame.pixels.height), (1920, 1080));
                app.video_uploaded();
                break;
            }
            assert!(app.playback.is_some(), "{}", app.browser.message);
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            app.presentation.projection,
            if index == 0 {
                crate::presentation::Projection::Hemisphere
            } else {
                crate::presentation::Projection::Sphere
            }
        );
        if index < 2 {
            app.settings_row = 0;
            app.update(Controls {
                grips: [false, true],
                stick: crate::navigation::Navigation {
                    vertical: 1,
                    horizontal: 1,
                },
                ..Default::default()
            });
        }
        if index == 2 {
            while app.playback.is_some() {
                assert!(Instant::now() < deadline);
                app.frame();
                thread::sleep(Duration::from_millis(5));
            }
        } else {
            app.update(Controls {
                x: true,
                ..Default::default()
            });
            assert_eq!(app.playback.as_ref().unwrap().status(), "Paused");
            thread::sleep(Duration::from_millis(100));
            app.update(Controls {
                b: true,
                ..Default::default()
            });
        }
        assert!(app.playback.is_none());
        assert!(
            app.panels()
                .contains(&Some(crate::shortcuts::PanelKind::Browser))
        );
        assert_eq!(app.browser.selected, selected);
        assert!(app.stopping());
        wait(&mut app, |a| !a.stopping());
        assert!(app.browser.message.is_empty(), "{}", app.browser.message);
    }
    app.update(Controls {
        b: true,
        ..Default::default()
    });
    assert_eq!(app.browser.directory, directory.parent().unwrap());
}
