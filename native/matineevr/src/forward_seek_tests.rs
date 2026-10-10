use crate::{media::Decoder, playback_tests::pixels};
use std::path::PathBuf;

#[test]
#[ignore = "requires Frame, an eight-second FRAME_TEST_VIDEO and FRAME_TEST_MEDIA_DIR"]
fn forward_seeks_preserve_pixels_across_keyframes_and_eof() {
    let video = PathBuf::from(std::env::var_os("FRAME_TEST_VIDEO").unwrap());
    let directory = PathBuf::from(std::env::var_os("FRAME_TEST_MEDIA_DIR").unwrap());
    for file in [
        video,
        directory.join("h264.mp4"),
        directory.join("hevc.mp4"),
    ] {
        let mut decoder = Decoder::open(&file).unwrap();
        let duration = decoder.duration().unwrap();
        let targets = [0.0, 0.25, 0.26, 1.25, duration - 0.5];
        let mut expected = Vec::new();
        let mut frame = decoder.next_frame().unwrap().unwrap();
        for target in targets {
            while frame.pixels.pts < target {
                let pts = frame.pixels.pts;
                drop(frame);
                frame = decoder
                    .next_frame()
                    .unwrap_or_else(|error| {
                        panic!("{} after {pts}s, target {target}s: {error}", file.display())
                    })
                    .unwrap();
            }
            expected.push((frame.pixels.pts, pixels(&frame)));
        }
        drop(frame);
        for index in [0, 1, 2, 3, 4, 4, 0, 3, 1, 2] {
            decoder.seek(targets[index]).unwrap();
            let frame = decoder.next_frame().unwrap().unwrap();
            assert_eq!(frame.pixels.pts, expected[index].0, "{}", file.display());
            assert_eq!(pixels(&frame), expected[index].1, "{}", file.display());
        }
        decoder.seek(duration + 1.0).unwrap();
        assert!(decoder.next_frame().unwrap().is_none());
        decoder.seek(0.26).unwrap();
        let frame = decoder.next_frame().unwrap().unwrap();
        assert_eq!(frame.pixels.pts, expected[2].0);
        assert_eq!(pixels(&frame), expected[2].1);
    }
}
