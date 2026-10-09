use crate::{
    alignment::Mode,
    hud_text::Canvas,
    presentation::{Presentation, Stereo},
};

impl Canvas {
    pub fn alignment(&mut self, p: Presentation, mode: Mode) {
        self.clear();
        let cyan = [100, 215, 255, 255];
        let white = [235, 240, 245, 255];
        let a = p.alignment;
        let (title, left, right) = match mode {
            Mode::Orientation => (
                "ORIENTATION / ZOOM",
                format!("Yaw {:+.1} deg     Pitch {:+.1} deg", a.yaw, a.pitch),
                format!(
                    "Roll {:+.1} deg    Zoom {:.0}%",
                    a.roll,
                    a.zoom_log2.exp2() * 100.0
                ),
            ),
            Mode::Position => (
                "STEREO / POSITION",
                if p.stereo == Stereo::Mono {
                    "Stereo offsets disabled for Mono".into()
                } else {
                    format!(
                        "Stereo H {:+.2}%   V {:+.2}%",
                        a.stereo_offset[0] * 100.0,
                        a.stereo_offset[1] * 100.0
                    )
                },
                format!("Position H {:+.2}  V {:+.2}", a.position[0], a.position[1]),
            ),
        };
        self.line(0, title, cyan);
        self.line(2, "LEFT STICK: horizontal / vertical", cyan);
        self.line(3, &left, white);
        self.line(4, "RIGHT STICK: horizontal / vertical", cyan);
        self.line(5, &right, white);
        self.line(7, "A: reset alignment", white);
    }
}
