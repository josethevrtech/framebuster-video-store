use crate::presentation::Presentation;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[path = "video_hints.rs"]
mod hints;
pub use hints::{Hint, Hints};

pub struct Settings {
    remembered: HashMap<PathBuf, Presentation>,
    last: Presentation,
    overrides: Hints,
    resolved: bool,
    played: bool,
}

impl Settings {
    pub fn new(overrides: Hints) -> Self {
        Self {
            remembered: HashMap::new(),
            last: overrides.apply(Presentation::default()),
            overrides,
            resolved: false,
            played: false,
        }
    }

    pub fn open(&mut self, path: &Path) -> Presentation {
        self.played = false;
        self.resolved = self.remembered.contains_key(path);
        self.remembered
            .get(path)
            .copied()
            .unwrap_or(self.overrides.apply(self.last))
    }

    pub fn infer(&mut self, path: &Path, metadata: Hints, current: &mut Presentation) {
        if !self.resolved {
            *current = self
                .overrides
                .apply(metadata.combine(Hints::filename(path)).apply(Presentation {
                    alignment: current.alignment,
                    ..self.last
                }));
            self.resolved = true;
        }
    }

    pub fn needs_inference(&self) -> bool {
        !self.resolved
    }

    pub fn uploaded(&mut self, path: &Path, current: Presentation) {
        if !self.played {
            self.played = true;
            self.edit(path, current);
        }
    }

    pub fn edit(&mut self, path: &Path, current: Presentation) {
        self.resolved = true;
        self.remember(path, current);
    }

    pub fn remember(&mut self, path: &Path, current: Presentation) {
        if self.played {
            self.remembered.insert(path.to_owned(), current);
            self.last = Presentation {
                alignment: Default::default(),
                lens: Default::default(),
                ..current
            };
        }
    }
}

#[cfg(test)]
#[path = "video_settings_tests.rs"]
mod tests;
