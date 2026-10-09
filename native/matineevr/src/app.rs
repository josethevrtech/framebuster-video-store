use crate::{
    adjustment::Adjustment, browser::Browser, browser_repeat::HoldRepeat, input::Controls,
    media::Frame, navigation::Navigation, options::Options, playback::Playback,
    presentation::Presentation, shortcuts::Shortcuts, video_settings::Settings,
};
use std::{path::PathBuf, thread::JoinHandle, time::Instant};

#[path = "app_alignment.rs"]
mod alignment;
#[path = "app_navigation.rs"]
mod navigation;
#[path = "app_presentation.rs"]
mod presentation;

pub struct App {
    pub browser: Browser,
    pub playback: Option<Playback>,
    pub path: PathBuf,
    pub hud: bool,
    pub changed: bool,
    pub completed: bool,
    pub failed: bool,
    pub shortcuts: Shortcuts,
    pub presentation: Presentation,
    pub adjustment: Adjustment,
    pub settings_row: usize,
    pub seek_until: Option<Instant>,
    browser_repeat: HoldRepeat,
    seek_repeat: [HoldRepeat; 2],
    settings: Settings,
    pending: Option<PathBuf>,
    stopping: Option<JoinHandle<()>>,
    retiring: Option<Playback>,
}

impl App {
    pub fn new(options: &Options) -> Self {
        let stream = crate::frame_playback::is_stream(&options.file);
        let file = options.file.is_file() || stream;
        let directory = if stream {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_else(|| ".".into()))
        } else if file {
            options.file.parent().unwrap().to_owned()
        } else {
            options.file.clone()
        };
        let mut settings = Settings::new(options.presentation_overrides);
        let presentation = settings.open(&options.file);
        Self {
            browser: Browser::new(directory),
            playback: file.then(|| Playback::start(options.file.clone())),
            path: options.file.clone(),
            hud: options.hud,
            changed: true,
            completed: false,
            failed: false,
            shortcuts: Shortcuts::default(),
            presentation,
            adjustment: Adjustment::default(),
            settings_row: 0,
            seek_until: None,
            browser_repeat: HoldRepeat::BROWSER,
            seek_repeat: [HoldRepeat::SEEK; 2],
            settings,
            pending: None,
            stopping: None,
            retiring: None,
        }
    }

    pub fn stopping(&self) -> bool {
        self.stopping.is_some() || self.retiring.is_some()
    }

    pub fn synchronize(
        &mut self,
        renderers: &mut [&mut crate::renderer::Renderer],
    ) -> anyhow::Result<()> {
        if let Some(player) = self.retiring.take() {
            matineevr::playback_render::synchronize(None, renderers)?;
            self.stopping = Some(player.stop());
        }
        matineevr::playback_render::synchronize(self.playback.as_mut(), renderers)
    }

    pub fn stop(&mut self) {
        self.reset_seek_repeat();
        self.pending = None;
        self.seek_until = None;
        if let Some(player) = self.playback.take() {
            self.retiring = Some(player);
            self.changed = true;
            eprintln!("Browser: stopped {}", self.path.display());
        }
    }

    pub fn update(&mut self, mut controls: Controls) {
        self.changed = self.browser.poll() || self.shortcuts.held != controls.grips;
        self.shortcuts.held = controls.grips;
        if self.stopping.as_ref().is_some_and(JoinHandle::is_finished) {
            if self.stopping.take().unwrap().join().is_err() {
                self.browser.message = "Decoder thread panicked".into();
                self.pending = None;
            }
            self.changed = true;
        }
        if !self.stopping()
            && let Some(path) = self.pending.take()
        {
            self.path = path;
            self.completed = false;
            self.failed = false;
            self.presentation = self.settings.open(&self.path);
            self.playback = Some(Playback::start(self.path.clone()));
            self.changed = true;
            eprintln!("Browser: open {}", self.path.display());
        }
        self.navigate(&mut controls, Instant::now());
        self.changed |= controls
            != Controls {
                grips: controls.grips,
                ..Default::default()
            };
        if self.playback.is_some() {
            if controls.b {
                self.stop();
            } else if let Some(player) = &mut self.playback {
                if controls.x {
                    player.toggle_pause();
                }
                if controls.y {
                    self.hud = !self.hud;
                }
            }
        } else {
            self.browse(Navigation {
                horizontal: i8::from(controls.a) - i8::from(controls.b),
                vertical: i8::from(controls.x) - i8::from(controls.y),
            });
        }
    }

    fn browse(&mut self, navigation: Navigation) {
        self.browser.move_selection(navigation.vertical as isize);
        if navigation.horizontal < 0 {
            self.browser.parent();
        } else if navigation.horizontal > 0
            && !self.stopping()
            && let Some(path) = self.browser.open()
        {
            self.stop();
            self.browser.message.clear();
            self.pending = Some(path);
        }
    }

    pub fn frame(&mut self) -> Option<Frame> {
        let player = self.playback.as_mut()?;
        let result = player.due_frame();
        if self.settings.needs_inference()
            && let Some(metadata) = player.take_metadata()
        {
            self.settings
                .infer(&self.path, metadata, &mut self.presentation);
            self.changed = true;
        }
        match result {
            Ok(None) if player.finished() => {
                self.completed = true;
                self.stop();
                None
            }
            Ok(frame) => frame,
            Err(error) => {
                self.fail(error);
                None
            }
        }
    }

    pub fn fail(&mut self, error: anyhow::Error) {
        self.failed = true;
        self.browser.message = format!("{error:#}");
        eprintln!("Playback failed: {}", self.browser.message);
        self.stop();
    }
}
