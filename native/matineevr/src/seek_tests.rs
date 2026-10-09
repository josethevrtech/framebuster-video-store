use crate::{
    media::{Decoder, Frame},
    playback::Playback,
    playback_tests::pixels,
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
#[ignore = "requires Frame and FRAME_TEST_UNTIMED_VIDEO without timestamps"]
fn untimed_video_rejects_seeking() {
    let path = PathBuf::from(std::env::var_os("FRAME_TEST_UNTIMED_VIDEO").unwrap());
    let mut decoder = Decoder::open(&path).unwrap();
    drop(decoder.next_frame().unwrap().unwrap());
    assert!(decoder.seek(0.5).is_err());
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_MEDIA_DIR"]
fn native_seeks_match_pixels_and_restart_after_eof() {
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    for name in ["h264.mp4", "hevc.mp4", "vp9.webm", "mixed.mkv"] {
        let mut decoder = Decoder::open(&directory.join(name)).unwrap();
        let mut expected = Vec::new();
        for target in [0.25, 1.25] {
            let frame = next_frame(|| {
                decoder
                    .next_frame()
                    .unwrap()
                    .filter(|f| f.pixels.pts >= target)
            });
            expected.push((frame.pixels.pts, pixels(&frame)));
        }
        while decoder.next_frame().unwrap().is_some() {}
        for index in [1, 0, 1] {
            decoder.seek(expected[index].0).unwrap();
            let frame = decoder.next_frame().unwrap().unwrap();
            assert_eq!(frame.pixels.pts, expected[index].0);
            assert_eq!(pixels(&frame), expected[index].1);
            assert!(
                decoder
                    .seek(0.0)
                    .unwrap_err()
                    .to_string()
                    .contains("release decoded frames")
            );
        }
        for invalid in [f64::NAN, f64::INFINITY, -1.0, f64::MAX] {
            assert!(decoder.seek(invalid).is_err());
        }
        decoder.seek(decoder.duration().unwrap() + 10.0).unwrap();
        assert!(decoder.next_frame().unwrap().is_none());
    }
}

#[test]
#[ignore = "requires Frame and an eight-second FRAME_TEST_VIDEO"]
fn seeks_coalesce_while_paused_preserve_clock_and_cancel() {
    let path = PathBuf::from(std::env::var_os("FRAME_TEST_VIDEO").unwrap());
    let mut player = Playback::start(path);
    drop(next_frame(|| {
        crate::playback_tests::due_frame(&mut player).unwrap()
    }));
    player.toggle_pause();
    thread::sleep(Duration::from_millis(250));
    player.skip(3);
    player.skip(2);
    player.skip(-1);
    let frame = next_frame(|| crate::playback_tests::due_frame(&mut player).unwrap());
    assert!((4.0..4.1).contains(&frame.pixels.pts));
    drop(frame);
    assert_eq!(player.status(), "Paused");
    let position = player.position;
    thread::sleep(Duration::from_millis(200));
    assert!(
        crate::playback_tests::due_frame(&mut player)
            .unwrap()
            .is_none()
    );
    assert_eq!(player.position, position);
    player.toggle_pause();
    thread::sleep(Duration::from_millis(100));
    drop(next_frame(|| {
        crate::playback_tests::due_frame(&mut player).unwrap()
    }));
    assert!((position..position + 0.25).contains(&player.position));
    player.set_suspended(true);
    player.skip(-10);
    assert_eq!(
        next_frame(|| crate::playback_tests::due_frame(&mut player).unwrap())
            .pixels
            .pts,
        0.0
    );
    assert_eq!(player.status(), "Paused");
    for invalid in [f64::NAN, f64::INFINITY, -1.0] {
        assert!(player.seek(invalid).is_err());
    }
    player.seek(1.25).unwrap();
    drop(next_frame(|| {
        crate::playback_tests::due_frame(&mut player).unwrap()
    }));
    assert!((1.25..1.35).contains(&player.position));
    assert_eq!(player.status(), "Paused");
    player.set_suspended(false);
    player.skip(20);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !player.finished() {
        assert!(Instant::now() < deadline);
        crate::playback_tests::due_frame(&mut player).unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    player.skip(-2);
    drop(next_frame(|| {
        crate::playback_tests::due_frame(&mut player).unwrap()
    }));
    assert!((6.0..6.1).contains(&player.position));
    player.skip(-5);
    let worker = player.stop();
    while !worker.is_finished() {
        assert!(Instant::now() < deadline, "Stopping a seek stalled");
        thread::sleep(Duration::from_millis(2));
    }
    worker.join().unwrap();
}
