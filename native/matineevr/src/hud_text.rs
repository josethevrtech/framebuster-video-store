use crate::{font, metrics::Sample, performance::Performance, playback::Playback};
use std::path::{Path, PathBuf};

pub(crate) const MARGIN: usize = 12;
const CHARACTER_WIDTH: usize = 12;
const LINE_HEIGHT: usize = 20;
pub const COLUMNS: usize = 64;
pub const WIDTH: usize = COLUMNS * CHARACTER_WIDTH + MARGIN * 2;
pub const HEIGHT: usize = 8 * LINE_HEIGHT + MARGIN * 2;

pub struct Canvas {
    pub pixels: Vec<u8>,
    pub size: [usize; 2],
    pub(crate) browser_directory: PathBuf,
    pub(crate) browser_start: usize,
    path: Vec<u8>,
}

impl Canvas {
    pub fn new(path: &Path, size: [usize; 2]) -> Self {
        Self {
            pixels: vec![0; size[0] * size[1] * 4],
            size,
            browser_directory: PathBuf::new(),
            browser_start: 0,
            path: escape(path).into_bytes(),
        }
    }

    pub fn columns(&self) -> usize {
        (self.size[0] - MARGIN * 2) / CHARACTER_WIDTH
    }

    pub fn rows(&self) -> usize {
        (self.size[1] - MARGIN * 2) / LINE_HEIGHT
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = escape(path).into_bytes();
    }

    pub fn clear(&mut self) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[6, 10, 16, 220]);
        }
    }

    pub fn update(&mut self, player: &Playback, stats: &Performance, sample: Sample, elapsed: f64) {
        self.clear();
        let pages = self.path.len().div_ceil(COLUMNS * 2).max(1);
        let page = elapsed as usize / 4 % pages;
        self.line(
            0,
            &format!("Path {}/{pages}", page + 1),
            [100, 215, 255, 255],
        );
        for row in 0..2 {
            let start = ((page * 2 + row) * COLUMNS).min(self.path.len());
            let end = (start + COLUMNS).min(self.path.len());
            let text = String::from_utf8_lossy(&self.path[start..end]).into_owned();
            self.line(row + 1, &text, [215, 225, 235, 255]);
        }
        self.line(
            3,
            &format!(
                "{}  {} / {}",
                player.status(),
                timestamp(Some(player.position)),
                timestamp(player.length)
            ),
            [255, 255, 255, 255],
        );
        self.line(
            4,
            &format!(
                "Device CPU {} | GPU {}",
                percent(sample.cpu),
                percent(sample.gpu)
            ),
            [255, 255, 255, 255],
        );
        let ram = sample.memory.map_or_else(
            || "N/A".into(),
            |(used, total)| {
                format!(
                    "{:.2} / {:.2} GiB",
                    used as f64 / 1073741824.0,
                    total as f64 / 1073741824.0
                )
            },
        );
        self.line(5, &format!("Device RAM {ram}"), [255, 255, 255, 255]);
        self.line(
            6,
            &format!(
                "XR {:.1} fps (target {:.0}) | Video {:.1} fps",
                stats.xr_fps, stats.target_fps, stats.video_fps
            ),
            [255, 255, 255, 255],
        );
        self.line(
            7,
            &format!(
                "Iris HW {}x{} | Skipped video frames {}",
                stats.size.0, stats.size.1, player.skipped
            ),
            [160, 205, 220, 255],
        );
    }

    pub fn line(&mut self, row: usize, text: &str, color: [u8; 4]) {
        assert!(row < self.rows());
        for (column, c) in text.chars().take(self.columns()).enumerate() {
            for (x, bits) in font::glyph(c).iter().enumerate() {
                for y in 0..7 {
                    if bits & (1 << y) == 0 {
                        continue;
                    }
                    for dy in 0..2 {
                        for dx in 0..2 {
                            let px = MARGIN + column * CHARACTER_WIDTH + x * 2 + dx;
                            let py = MARGIN + row * LINE_HEIGHT + y * 2 + dy;
                            let offset = ((self.size[1] - 1 - py) * self.size[0] + px) * 4;
                            self.pixels[offset..offset + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }
}

pub fn escape(path: &Path) -> String {
    path.as_os_str()
        .as_encoded_bytes()
        .iter()
        .map(|&c| {
            if c.is_ascii_graphic() && c != b'\\' || c == b' ' {
                char::from(c).to_string()
            } else {
                format!("\\x{c:02x}")
            }
        })
        .collect()
}

fn percent(value: Option<f64>) -> String {
    value.map_or_else(|| "N/A".into(), |v| format!("{v:.0}%"))
}

pub(crate) fn timestamp(seconds: Option<f64>) -> String {
    let Some(seconds) = seconds.filter(|s| s.is_finite() && *s >= 0.0) else {
        return "--:--".into();
    };
    let total = seconds as u64;
    if total >= 3600 {
        format!("{}:{:02}:{:02}", total / 3600, total / 60 % 60, total % 60)
    } else {
        format!("{:02}:{:02}", total / 60, total % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_handle_unknown_duration_and_hours() {
        assert_eq!(timestamp(None), "--:--");
        assert_eq!(timestamp(Some(f64::NAN)), "--:--");
        assert_eq!(timestamp(Some(661.227)), "11:01");
        assert_eq!(timestamp(Some(3661.0)), "1:01:01");
    }

    #[test]
    fn path_escaping_preserves_case_and_non_ascii_bytes() {
        let canvas = Canvas::new(Path::new("/Videos/Caf\u{e9}\\a\n.mp4"), [WIDTH, HEIGHT]);
        assert_eq!(
            String::from_utf8(canvas.path).unwrap(),
            "/Videos/Caf\\xc3\\xa9\\x5ca\\x0a.mp4"
        );
    }
}
