use super::App;
use crate::{
    browser_repeat::HoldRepeat,
    input::Controls,
    shortcuts::{PanelKind, Shortcuts},
};
use std::time::Instant;

impl App {
    pub fn reset_seek_repeat(&mut self) {
        self.seek_repeat = [HoldRepeat::SEEK; 2];
    }

    pub(super) fn navigate(&mut self, controls: &mut Controls, now: Instant) {
        self.adjust_alignment(controls, now);
        let targets = Shortcuts::targets(self.panels());
        let held = std::mem::take(&mut controls.held_vertical);
        let direction = if targets[0] == Some(PanelKind::Browser) {
            held
        } else {
            0
        };
        let movement = self
            .browser_repeat
            .update(direction, controls.dpad.vertical != 0, now);
        if movement != 0 {
            controls.dpad.vertical = movement;
        }
        let held = std::mem::take(&mut controls.held_horizontal);
        for (hand, (target, mut navigation)) in targets
            .into_iter()
            .zip([controls.dpad, controls.stick])
            .enumerate()
        {
            let seeking = matches!(
                target,
                None | Some(PanelKind::Information | PanelKind::Seek)
            ) && self.playback.is_some()
                && !controls.b;
            let movement = self.seek_repeat[hand].update(
                if seeking { held[hand] } else { 0 },
                navigation.horizontal != 0,
                now,
            );
            if movement != 0 {
                navigation.horizontal = movement;
                self.changed = true;
            }
            match target {
                Some(PanelKind::Browser) => self.browse(navigation),
                Some(PanelKind::Settings) => self.adjust_presentation(navigation),
                _ if seeking && navigation.horizontal != 0 => {
                    if let Some(player) = &mut self.playback {
                        player.skip(i32::from(navigation.horizontal) * 10);
                        self.seek_until = Some(now + crate::seek_view::VISIBLE_FOR);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{options::Options, playback::Playback};
    use std::time::Duration;

    fn app() -> App {
        let directory = crate::browser::tests::fixture();
        let options = Options::from_args([directory.to_str().unwrap().into()].into_iter())
            .unwrap()
            .unwrap();
        let mut app = App::new(&options);
        crate::app_tests::wait(&mut app, |app| !app.browser.loading());
        app.playback = Some(Playback::start(directory.join("missing.mp4")));
        app
    }

    fn check(app: &mut App, mut controls: Controls, now: Instant, expected: f64) {
        app.navigate(&mut controls, now);
        assert_eq!(app.playback.as_ref().unwrap().seek_target(), Some(expected));
    }

    #[test]
    fn each_input_taps_holds_reverses_and_releases() {
        for hand in 0..2 {
            let mut app = app();
            let start = Instant::now();
            let at = |ms| start + Duration::from_millis(ms);
            let mut held = Controls::default();
            held.held_horizontal[hand] = 1;
            let mut pressed = held;
            [&mut pressed.dpad, &mut pressed.stick][hand].horizontal = 1;
            check(&mut app, pressed, at(0), 10.0);
            check(&mut app, held, at(249), 10.0);
            check(&mut app, held, at(250), 20.0);
            assert_eq!(
                app.seek_until,
                Some(at(250) + crate::seek_view::VISIBLE_FOR)
            );
            check(&mut app, held, at(500), 30.0);
            held.held_horizontal[hand] = -1;
            pressed = held;
            [&mut pressed.dpad, &mut pressed.stick][hand].horizontal = -1;
            check(&mut app, pressed, at(501), 20.0);
            check(&mut app, held, at(750), 20.0);
            check(&mut app, held, at(751), 10.0);
            check(&mut app, Controls::default(), at(752), 10.0);
            check(&mut app, Controls::default(), at(10000), 10.0);
            app.stop();
            crate::app_tests::wait(&mut app, |app| !app.stopping());
        }
    }

    #[test]
    fn panels_route_holds_and_reset_seek_acceleration() {
        let mut app = app();
        let held = Controls {
            held_horizontal: [1; 2],
            ..Default::default()
        };
        let mut now = Instant::now();
        for panel in [PanelKind::Browser, PanelKind::Settings] {
            app.reset_seek_repeat();
            app.playback.as_mut().unwrap().seek(10.0).unwrap();
            check(&mut app, held, now, 10.0);
            now += Duration::from_millis(250);
            check(&mut app, held, now, 30.0);
            app.shortcuts.assignments = [panel; 2];
            app.shortcuts.held = [true; 2];
            let before = app.presentation;
            let directory = app.browser.directory.clone();
            for _ in 0..3 {
                now += Duration::from_secs(1);
                check(&mut app, held, now, 30.0);
            }
            assert_eq!(app.presentation, before);
            assert_eq!(app.browser.directory, directory);
            app.shortcuts.held = [false; 2];
            app.hud = true;
            check(&mut app, held, now, 30.0);
            now += Duration::from_millis(250);
            check(&mut app, held, now, 50.0);
        }
        app.shortcuts.assignments[0] = PanelKind::Browser;
        app.shortcuts.held = [true, false];
        now += Duration::from_secs(1);
        check(&mut app, held, now, 60.0);
        app.stop();
        crate::app_tests::wait(&mut app, |app| !app.stopping());
    }

    #[test]
    fn focus_reset_and_stop_restart_the_delay_without_redrawing_held_input() {
        let mut app = app();
        let held = Controls {
            held_horizontal: [1; 2],
            ..Default::default()
        };
        app.update(held);
        assert!(!app.changed);
        app.playback.as_mut().unwrap().seek(10.0).unwrap();
        let mut now = Instant::now();
        for stop in [false, true] {
            if stop {
                app.stop();
                crate::app_tests::wait(&mut app, |app| !app.stopping());
                app.playback = Some(Playback::start(app.path.join("missing.mp4")));
                app.playback.as_mut().unwrap().seek(10.0).unwrap();
            } else {
                app.reset_seek_repeat();
            }
            now += Duration::from_secs(10);
            check(&mut app, held, now, 10.0);
            check(&mut app, held, now + Duration::from_millis(249), 10.0);
            check(&mut app, held, now + Duration::from_millis(250), 30.0);
        }
        app.stop();
        crate::app_tests::wait(&mut app, |app| !app.stopping());
    }
}
