use crate::{
    app::App,
    input::Controls,
    media::{Decoder, Frame},
    options::Options,
    playback_tests::pixels,
    presentation::Projection,
};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn next_frame(mut next: impl FnMut() -> Option<Frame>) -> Frame {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(Instant::now() < deadline, "Seek did not produce a frame");
        if let Some(frame) = next() {
            return frame;
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_SEEK_VIDEO longer than 30 seconds"]
fn stick_and_dpad_seek_ten_seconds_and_preserve_panel_controls() {
    let path = PathBuf::from(std::env::var_os("FRAME_TEST_SEEK_VIDEO").unwrap());
    let mut decoder = Decoder::open(&path).unwrap();
    let frame = next_frame(|| {
        decoder
            .next_frame()
            .unwrap()
            .filter(|f| f.pixels.pts >= 1.98)
    });
    let expected = (frame.pixels.pts, pixels(&frame));
    drop(frame);
    decoder.seek(1.98).unwrap();
    let frame = decoder.next_frame().unwrap().unwrap();
    assert_eq!(frame.pixels.pts, expected.0);
    assert_eq!(pixels(&frame), expected.1);
    drop(frame);
    drop(decoder);
    let options = Options::from_args([path.to_str().unwrap().to_owned()].into_iter())
        .unwrap()
        .unwrap();
    for dpad in [false, true] {
        let mut app = App::new(&options);
        drop(next_frame(|| {
            app.synchronize(&mut []).unwrap();
            app.frame()
        }));
        app.update(Controls {
            x: true,
            ..Default::default()
        });
        let mut right = Controls::default();
        if dpad {
            right.dpad.horizontal = 1;
        } else {
            right.stick.horizontal = 1;
        }
        let target = app.playback.as_ref().unwrap().position + 10.0;
        app.update(right);
        assert_eq!(app.playback.as_ref().unwrap().seek_target(), Some(target));
        let until = app.seek_until.unwrap();
        drop(next_frame(|| {
            app.synchronize(&mut []).unwrap();
            app.frame()
        }));
        let position = app.playback.as_ref().unwrap().position;
        assert!((10.0..10.1).contains(&position));
        assert_eq!(app.playback.as_ref().unwrap().status(), "Paused");
        app.update(Controls {
            grips: [false, true],
            ..right
        });
        assert_eq!(app.presentation.projection, Projection::Flat);
        assert_eq!(app.seek_until, Some(until));
        assert_eq!(app.playback.as_ref().unwrap().position, position);
        assert_eq!(app.playback.as_ref().unwrap().status(), "Paused");
        app.update(right);
        app.update(right);
        assert!(app.seek_until.unwrap() > until);
        drop(next_frame(|| {
            app.synchronize(&mut []).unwrap();
            app.frame()
        }));
        assert!((30.0..30.1).contains(&app.playback.as_ref().unwrap().position));
        let mut left = right;
        left.dpad.horizontal = -right.dpad.horizontal;
        left.stick.horizontal = -right.stick.horizontal;
        app.update(left);
        drop(next_frame(|| {
            app.synchronize(&mut []).unwrap();
            app.frame()
        }));
        assert!((20.0..20.1).contains(&app.playback.as_ref().unwrap().position));
        app.hud = true;
        app.update(left);
        drop(next_frame(|| {
            app.synchronize(&mut []).unwrap();
            app.frame()
        }));
        assert!((10.0..10.1).contains(&app.playback.as_ref().unwrap().position));
        app.stop();
        assert!(app.seek_until.is_none());
        crate::app_tests::wait(&mut app, |app| !app.stopping());
    }
}
