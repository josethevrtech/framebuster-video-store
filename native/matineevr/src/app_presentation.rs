use super::App;
use crate::{alignment::Mode, navigation::Navigation, presentation, shortcuts::PanelKind};

impl App {
    pub fn panels(&self) -> [Option<PanelKind>; 3] {
        match (self.playback.is_some(), self.adjustment.mode()) {
            (true, Some(Mode::Orientation)) => [Some(PanelKind::Alignment), None, None],
            (true, Some(Mode::Position)) => [None, Some(PanelKind::Alignment), None],
            _ => self.shortcuts.panels(self.playback.is_some(), self.hud),
        }
    }

    pub(super) fn adjust_presentation(&mut self, navigation: Navigation) {
        presentation::navigate(&mut self.presentation, &mut self.settings_row, navigation);
        if navigation.horizontal != 0 {
            self.settings.edit(&self.path, self.presentation);
        }
    }

    pub fn video_uploaded(&mut self) {
        self.settings.uploaded(&self.path, self.presentation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        app_tests::wait,
        input::Controls,
        options::Options,
        presentation::{Presentation, Projection, Stereo},
        shortcut_tests::first_frame,
    };

    #[test]
    #[ignore = "requires Frame and FRAME_TEST_METADATA_VIDEO: H.264 Matroska with stereo_mode=bottom_top"]
    fn stream_metadata_manual_edits_and_reset_survive_reopening() {
        let options =
            Options::from_args([std::env::var("FRAME_TEST_METADATA_VIDEO").unwrap()].into_iter())
                .unwrap()
                .unwrap();
        let mut app = App::new(&options);
        first_frame(&mut app);
        assert_eq!(
            app.presentation,
            Presentation {
                projection: Projection::Hemisphere,
                stereo: Stereo::TopBottom,
                swap_eyes: true,
                ..Default::default()
            }
        );
        for reset in [false, true] {
            app.settings_row = if reset {
                app.presentation.fields().len() - 1
            } else {
                0
            };
            app.update(Controls {
                grips: [false, true],
                stick: Navigation {
                    horizontal: 1,
                    vertical: 0,
                },
                ..Default::default()
            });
            let expected = if reset {
                Presentation::default()
            } else {
                Presentation {
                    projection: Projection::Flat,
                    stereo: Stereo::TopBottom,
                    swap_eyes: true,
                    ..Default::default()
                }
            };
            assert_eq!(app.presentation, expected);
            app.stop();
            wait(&mut app, |app| !app.stopping() && !app.browser.loading());
            app.browser.selected = app
                .browser
                .entries
                .iter()
                .position(|entry| entry.path == options.file)
                .unwrap();
            app.update(Controls {
                a: true,
                ..Default::default()
            });
            first_frame(&mut app);
            assert_eq!(app.presentation, expected);
        }
        app.stop();
        wait(&mut app, |app| !app.stopping());
    }
}
