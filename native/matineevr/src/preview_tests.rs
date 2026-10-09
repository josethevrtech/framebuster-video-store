use crate::{
    media::{Decoded, Decoder, Frame},
    playback::Playback,
    playback_tests::pixels,
};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn hash(frame: &Frame) -> u64 {
    let mut hash = DefaultHasher::new();
    pixels(frame).hash(&mut hash);
    hash.finish()
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_MEDIA_DIR"]
fn previews_match_decoded_pixels_without_changing_target_frames() {
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    for name in ["h264.mp4", "hevc.mp4", "vp9.webm", "mixed.mkv"] {
        let mut decoder = Decoder::open(&directory.join(name)).unwrap();
        let mut expected = HashMap::new();
        while let Some(frame) = decoder.next_frame().unwrap() {
            expected.insert(frame.pixels.pts.to_bits(), hash(&frame));
        }
        for target in [1.25, 0.25, 1.75, 0.0] {
            decoder.seek(target).unwrap();
            let mut previews = 0;
            let deadline = Instant::now() + Duration::from_secs(20);
            loop {
                assert!(Instant::now() < deadline);
                match decoder.advance().unwrap() {
                    Decoded::Preview(frame) => {
                        previews += 1;
                        assert_eq!(previews, 1);
                        assert!(frame.pixels.pts <= target);
                        assert_eq!(
                            hash(&frame),
                            expected[&frame.pixels.pts.to_bits()],
                            "{name}"
                        );
                        assert!(
                            decoder
                                .seek(0.0)
                                .unwrap_err()
                                .to_string()
                                .contains("release decoded frames")
                        );
                    }
                    Decoded::Frame(frame) => {
                        assert!((target..target + 0.05).contains(&frame.pixels.pts));
                        assert_eq!(
                            hash(&frame),
                            expected[&frame.pixels.pts.to_bits()],
                            "{name}"
                        );
                        break;
                    }
                    Decoded::Pending => {}
                    Decoded::End => panic!("Ended before target"),
                }
            }
            assert_eq!(previews, 1);
        }
    }
}

fn poll(player: &mut Playback) -> Decoded {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        assert!(Instant::now() < deadline);
        if let Some(frame) = crate::playback_tests::due_frame(player).unwrap() {
            return Decoded::Frame(frame);
        }
        if let Some(frame) = player.take_preview() {
            return Decoded::Preview(frame);
        }
        thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "requires Frame and an eight-second FRAME_TEST_VIDEO"]
fn preview_preserves_pause_and_position_and_repeated_seeks_release_frames() {
    let mut player = Playback::start(PathBuf::from(std::env::var_os("FRAME_TEST_VIDEO").unwrap()));
    assert!(matches!(poll(&mut player), Decoded::Frame(_)));
    player.toggle_pause();
    let position = player.position;
    player.seek(1.25).unwrap();
    assert!(matches!(poll(&mut player), Decoded::Preview(_)));
    assert_eq!(player.position, position);
    assert_eq!(player.seek_target(), Some(1.25));
    for target in [6.6, 0.25, 1.25] {
        player.seek(target).unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            assert!(Instant::now() < deadline);
            if let Some(frame) = crate::playback_tests::due_frame(&mut player).unwrap() {
                assert!((target..target + 0.05).contains(&frame.pixels.pts));
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(player.status(), "Paused");
        assert!(
            crate::playback_tests::due_frame(&mut player)
                .unwrap()
                .is_none()
        );
    }
    player.seek(6.6).unwrap();
    player.seek(0.0).unwrap();
    let Decoded::Preview(frame) = poll(&mut player) else {
        panic!("Missing preview after seek to zero")
    };
    assert_eq!(frame.pixels.pts, 0.0);
    drop(frame);
    assert!(matches!(poll(&mut player), Decoded::Frame(_)));
    player.stop().join().unwrap();
}
