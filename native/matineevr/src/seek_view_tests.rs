use super::*;
use std::path::Path;

#[test]
fn visibility_expires_unless_seeking_and_restarts_on_another_skip() {
    let now = Instant::now();
    let until = Some(now + VISIBLE_FOR);
    assert!(!visible(None, false, now));
    assert!(!visible(None, true, now));
    assert!(visible(until, false, now));
    assert!(!visible(until, false, now + VISIBLE_FOR));
    assert!(visible(until, true, now + VISIBLE_FOR));
    assert!(visible(
        Some(now + VISIBLE_FOR * 2),
        false,
        now + VISIBLE_FOR
    ));
}

fn pixel(canvas: &Canvas, x: usize, y: usize) -> &[u8] {
    let offset = ((canvas.size[1] - 1 - y) * canvas.size[0] + x) * 4;
    &canvas.pixels[offset..offset + 4]
}

#[test]
fn transparent_outline_marker_clamping_and_unknown_duration() {
    let mut canvas = Canvas::new(Path::new("video.mp4"), SIZE);
    let right = SIZE[0] - MARGIN;
    let bottom = SIZE[1] - MARGIN;
    for (position, length, marker) in [
        (50.0, Some(100.0), Some((SIZE[0] - STROKE) / 2)),
        (0.0, Some(100.0), Some(MARGIN)),
        (150.0, Some(100.0), Some(right - STROKE)),
        (-1.0, Some(100.0), Some(MARGIN)),
        (50.0, None, None),
    ] {
        canvas.seek(position, length);
        let mut expected = vec![12, 13, 778, 779];
        expected.extend(marker.into_iter().flat_map(|x| [x, x + 1]));
        expected.sort_unstable();
        expected.dedup();
        let columns: Vec<_> = (0..SIZE[0])
            .filter(|&x| pixel(&canvas, x, 44) == [255; 4])
            .collect();
        assert_eq!(columns, expected);
        for x in MARGIN..right {
            assert_eq!(pixel(&canvas, x, BAR_TOP), [255; 4]);
            assert_eq!(pixel(&canvas, x, bottom - 1), [255; 4]);
        }
        assert_eq!(pixel(&canvas, 0, 0), [0; 4]);
        assert_eq!(pixel(&canvas, MARGIN - 1, BAR_TOP), [0; 4]);
        assert_eq!(pixel(&canvas, right, bottom), [0; 4]);
    }
    let mut expected = Canvas::new(Path::new("video.mp4"), SIZE);
    expected.line(0, "00:50 / --:--", [255; 4]);
    for y in 0..BAR_TOP {
        for x in 0..SIZE[0] {
            assert_eq!(pixel(&canvas, x, y), pixel(&expected, x, y));
        }
    }
    if let Some(path) = std::env::var_os("FRAME_TEST_SEEK_SNAPSHOT") {
        canvas.seek(754.0, Some(2900.0));
        crate::snapshot::save_pixels(
            Path::new(&path),
            SIZE[0] as i32,
            SIZE[1] as i32,
            &canvas.pixels,
        )
        .unwrap();
    }
}
