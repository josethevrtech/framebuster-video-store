use super::App;
use crate::input::Controls;
use std::time::Instant;

impl App {
    pub(super) fn adjust_alignment(&mut self, controls: &mut Controls, now: Instant) {
        let before = self.presentation.alignment;
        let mode = self.adjustment.mode();
        let sticks = std::mem::take(&mut controls.sticks);
        let seconds = self.adjustment.update(
            std::mem::take(&mut controls.triggers),
            sticks,
            self.playback.is_some() && !controls.b,
            now,
        );
        if let Some(mode) = self.adjustment.mode() {
            if controls.a {
                self.presentation.alignment = Default::default();
            } else {
                self.presentation
                    .alignment
                    .adjust(mode, sticks, seconds, self.presentation.stereo);
            }
            controls.dpad = Default::default();
            controls.held_vertical = 0;
            controls.held_horizontal[0] = 0;
        }
        if self.adjustment.blocked() {
            controls.stick = Default::default();
            controls.held_horizontal[1] = 0;
        }
        if before != self.presentation.alignment {
            self.settings.remember(&self.path, self.presentation);
            self.changed = true;
        }
        self.changed |= mode != self.adjustment.mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        navigation::Navigation, options::Options, playback::Playback, shortcuts::PanelKind,
    };
    use std::time::Duration;

    fn app() -> App {
        let directory = crate::browser::tests::fixture();
        let options = Options::from_args([directory.to_str().unwrap().into()].into_iter())
            .unwrap()
            .unwrap();
        let mut app = App::new(&options);
        app.path = options.file.join("alignment-missing-test-video.mp4");
        app.playback = Some(Playback::start(app.path.clone()));
        app.video_uploaded();
        app
    }

    #[test]
    fn adjustment_overrides_panels_blocks_seeking_resets_and_remembers() {
        let mut app = app();
        let mut controls = Controls {
            triggers: [1.0, 0.0],
            ..Default::default()
        };
        for grips in [[true; 2], [false; 2]] {
            controls.grips = grips;
            app.update(controls);
            assert_eq!(app.panels(), [Some(PanelKind::Alignment), None, None]);
        }
        controls.sticks = [[0.8; 2]; 2];
        controls.stick = Navigation::stick(1.0, 0.0);
        controls.dpad = controls.stick;
        std::thread::sleep(std::time::Duration::from_millis(10));
        app.update(controls);
        assert!(app.presentation.alignment.yaw < 0.0);
        assert_eq!(app.playback.as_ref().unwrap().seek_target(), None);
        assert_eq!(app.settings.open(&app.path), app.presentation);
        app.video_uploaded();
        controls.a = true;
        app.update(controls);
        assert_eq!(app.presentation.alignment, Default::default());
        assert_eq!(app.settings.open(&app.path), app.presentation);
        app.update(Controls::default());
        controls.triggers = [0.0, 1.0];
        app.update(controls);
        assert_eq!(app.panels(), [None, Some(PanelKind::Alignment), None]);
        app.stop();
        crate::app_tests::wait(&mut app, |app| !app.stopping());
    }

    #[test]
    fn adjustment_blocks_held_seeks_until_centered_and_restarts_repeat_delay() {
        for hand in 0..2 {
            let mut app = app();
            app.playback.as_mut().unwrap().seek(10.0).unwrap();
            let start = Instant::now();
            let mut step = |mut controls: Controls, ms, expected| {
                app.navigate(&mut controls, start + Duration::from_millis(ms));
                assert_eq!(app.playback.as_ref().unwrap().seek_target(), Some(expected));
            };
            let held = Controls {
                sticks: [[1.0; 2]; 2],
                held_horizontal: [1; 2],
                ..Default::default()
            };
            step(held, 0, 10.0);
            step(held, 250, 30.0);
            let mut adjusting = Controls::default();
            adjusting.triggers[hand] = 1.0;
            step(adjusting, 300, 30.0);
            adjusting = Controls {
                triggers: adjusting.triggers,
                dpad: Navigation::stick(1.0, 0.0),
                stick: Navigation::stick(1.0, 0.0),
                ..held
            };
            for ms in [310, 610, 1210] {
                step(adjusting, ms, 30.0);
            }
            let released = Controls {
                held_horizontal: [0, 1],
                ..held
            };
            for ms in [1220, 1520, 2120] {
                step(released, ms, 30.0);
            }
            step(Controls::default(), 2130, 30.0);
            step(held, 2140, 30.0);
            step(held, 2389, 30.0);
            step(held, 2390, 50.0);
            step(held, 2640, 70.0);
            app.stop();
            crate::app_tests::wait(&mut app, |app| !app.stopping());
        }
    }
}
