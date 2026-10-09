use super::*;
use crate::{browser::Entry, browser::tests::fixture, snapshot};
use std::{
    fs,
    time::{Duration, Instant},
};

#[test]
fn browser_sizes_scrolling_and_rendering() {
    let root = fixture();
    let mut browser = Browser::new(root);
    let deadline = Instant::now() + Duration::from_secs(5);
    while browser.loading() {
        assert!(Instant::now() < deadline);
        browser.poll();
        std::thread::yield_now();
    }
    browser.directory = "/Videos/Travel".into();
    browser.entries = (0..36).map(|i| Entry {
        path: browser.directory.join(format!("Video {i:02} - Walking through the forest and along the coast - VR180 side by side.mp4")),
        directory: false,
    }).collect();
    let output = std::env::var_os("FRAME_TEST_BROWSER_LAYOUT_DIR");
    if let Some(path) = &output {
        fs::create_dir(path).unwrap();
    }
    for playing in [false, true] {
        let size = panel_size(PanelKind::Browser, playing);
        assert_eq!(size, if playing { [792, 184] } else { [792, 736] });
        let mut canvas = Canvas::new(&browser.directory, size);
        assert_eq!(canvas.rows() - 4, if playing { 4 } else { 31 });
        assert_eq!(canvas.columns(), 64);
        let starts = if playing { [0, 24, 32] } else { [0, 1, 5] };
        for (selected, start) in [0, 26, 35].into_iter().zip(starts) {
            browser.selected = selected;
            canvas.browser(&browser, false, 0.0);
            let rows = canvas.rows() - 4;
            assert_eq!(canvas.browser_start, start);
            for row in 0..rows {
                let index = start + row;
                let color = if index == selected {
                    [100, 215, 255, 255]
                } else {
                    [235, 240, 245, 255]
                };
                assert_eq!(
                    row_has_color(&canvas, row + 2, color),
                    index < browser.entries.len()
                );
            }
            assert!(row_has_color(&canvas, rows + 2, [100, 215, 255, 255]));
            assert!(row_has_color(&canvas, rows + 3, [235, 240, 245, 255]));
            if let Some(path) = &output {
                let name = if playing { "playback" } else { "standalone" };
                snapshot::save_pixels(
                    &Path::new(path).join(format!("{name}-{selected}.ppm")),
                    size[0] as i32,
                    size[1] as i32,
                    &canvas.pixels,
                )
                .unwrap();
            }
        }
        browser.directory.push("other");
        browser.selected = if playing { 2 } else { 8 };
        canvas.browser(&browser, false, 0.0);
        assert_eq!(canvas.browser_start, 0);
        browser.directory.pop();
        for kind in [PanelKind::Settings, PanelKind::Information] {
            assert_eq!(panel_size(kind, playing), [WIDTH, HEIGHT]);
        }
    }
    assert_eq!(page("abcdef", 3.0, 4), "ef");
    assert_eq!(page("abcdef", 6.0, 4), "abcd");
    assert_eq!(page("", 0.0, 4), "");
}

#[test]
fn padding_scrolls_one_row_at_a_time_and_clamps_at_the_ends() {
    let mut start = 0;
    for (selected, expected) in [
        (25, 0),
        (26, 1),
        (27, 2),
        (26, 2),
        (7, 2),
        (6, 1),
        (0, 0),
        (99, 69),
    ] {
        start = scroll_start(start, selected, 100, 31);
        assert_eq!(start, expected);
    }
    for (selected, expected) in [(0, 0), (2, 0), (3, 1), (2, 1), (1, 0), (99, 96)] {
        start = scroll_start(start, selected, 100, 4);
        assert_eq!(start, expected);
    }
    assert_eq!(scroll_start(90, 1, 2, 31), 0);
    assert_eq!(scroll_start(90, 0, 0, 31), 0);
    assert_eq!(scroll_start(0, 3, 10, 1), 3);
}

fn row_has_color(canvas: &Canvas, row: usize, color: [u8; 4]) -> bool {
    let y = canvas.size[1] - 12 - row * 20;
    canvas.pixels[(y - 14) * canvas.size[0] * 4..y * canvas.size[0] * 4]
        .chunks_exact(4)
        .any(|pixel| pixel == color)
}
