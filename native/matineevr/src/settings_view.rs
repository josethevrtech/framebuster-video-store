use crate::{hud_text::Canvas, presentation::Presentation};

impl Canvas {
    pub fn settings(&mut self, presentation: Presentation, selected: usize) {
        self.clear();
        let white = [235, 240, 245, 255];
        let cyan = [100, 215, 255, 255];
        self.line(
            0,
            &format!("VIDEO SETTINGS | {}", env!("MATINEEVR_VERSION")),
            cyan,
        );
        let fields = presentation.fields();
        let selected = selected.min(fields.len() - 1);
        let rows = self.rows() - 2;
        let start = crate::browser_view::scroll_start(0, selected, fields.len(), rows);
        for (row, field) in fields.iter().enumerate().skip(start).take(rows) {
            let (label, value) = field.display(presentation);
            self.line(
                row - start + 2,
                &format!(
                    "{} {label}: < {value} >",
                    if row == selected { ">" } else { " " }
                ),
                if row == selected { cyan } else { white },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hud_text::{HEIGHT, WIDTH},
        presentation::Projection,
        snapshot,
    };
    use std::path::Path;

    #[test]
    fn layouts_fit_and_stale_selection_highlights_the_last_field() {
        for (projection, name) in [
            (Projection::Flat, "flat"),
            (Projection::Hemisphere, "180"),
            (Projection::Sphere, "360"),
            (Projection::Fisheye(190), "fisheye190"),
        ] {
            let p = Presentation {
                projection,
                ..Presentation::default()
            };
            let mut canvas = Canvas::new(Path::new("video.mp4"), [WIDTH, HEIGHT]);
            canvas.settings(p, usize::MAX);
            let stale = canvas.pixels.clone();
            canvas.settings(p, p.fields().len() - 1);
            assert_eq!(stale, canvas.pixels);
            canvas.settings(p, 0);
            assert_ne!(stale, canvas.pixels);
            for selected in 0..p.fields().len() {
                canvas.settings(p, selected);
            }
            if let Some(directory) = std::env::var_os("FRAME_TEST_SETTINGS_LAYOUT_DIR") {
                snapshot::save_pixels(
                    &Path::new(&directory).join(format!("{name}.ppm")),
                    WIDTH as i32,
                    HEIGHT as i32,
                    &canvas.pixels,
                )
                .unwrap();
            }
        }
    }
}
