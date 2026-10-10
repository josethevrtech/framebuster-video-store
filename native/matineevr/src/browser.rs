use crate::media;
use anyhow::Result;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
    },
};

#[path = "browser_scan.rs"]
mod scanning;
use scanning::Scanner;

#[derive(Debug)]
pub struct Entry {
    pub path: PathBuf,
    pub directory: bool,
}

pub struct Browser {
    pub directory: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    pub message: String,
    selections: HashMap<PathBuf, PathBuf>,
    pending: Option<Receiver<Result<Vec<Entry>>>>,
    scanner: Arc<Scanner>,
    cancelled: Arc<AtomicBool>,
}

impl Browser {
    pub fn new(directory: PathBuf) -> Self {
        let mut browser = Self {
            directory: PathBuf::new(),
            entries: Vec::new(),
            selected: 0,
            message: String::new(),
            selections: HashMap::new(),
            pending: None,
            scanner: Arc::default(),
            cancelled: Arc::default(),
        };
        browser.enter(directory);
        browser
    }

    fn enter(&mut self, directory: PathBuf) {
        self.cancelled.store(true, Ordering::Relaxed);
        self.cancelled = Arc::default();
        let cancelled = self.cancelled.clone();
        let scanner = self.scanner.clone();
        let (sender, receiver) = mpsc::channel();
        let path = directory.clone();
        std::thread::spawn(move || {
            let _ = sender.send(scanner.scan(&path, &cancelled, media::supported));
        });
        self.entries = directory
            .parent()
            .map(|parent| Entry {
                path: parent.into(),
                directory: true,
            })
            .into_iter()
            .collect();
        self.directory = directory;
        self.selected = 0;
        self.message.clear();
        self.pending = Some(receiver);
    }

    pub fn parent(&mut self) {
        if let Some(parent) = self.directory.parent() {
            self.enter(parent.to_owned());
        }
    }

    pub fn loading(&self) -> bool {
        self.pending.is_some()
    }

    pub fn poll(&mut self) -> bool {
        let Some(receiver) = &self.pending else {
            return false;
        };
        match receiver.try_recv() {
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => self.message = "Directory scan stopped".into(),
            Ok(Err(error)) => self.message = format!("{error:#}"),
            Ok(Ok(entries)) => {
                let first = self.entries.len().min(entries.len());
                self.entries.extend(entries);
                self.selected = self
                    .selections
                    .get(&self.directory)
                    .and_then(|path| self.entries.iter().position(|entry| &entry.path == path))
                    .unwrap_or(first);
                self.remember_selection();
            }
        }
        self.pending = None;
        true
    }

    pub fn move_selection(&mut self, direction: isize) {
        let selected = self
            .selected
            .saturating_add_signed(direction)
            .min(self.entries.len().saturating_sub(1));
        if selected != self.selected {
            self.selected = selected;
            self.remember_selection();
        }
    }

    fn remember_selection(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            self.selections
                .insert(self.directory.clone(), entry.path.clone());
        }
    }

    pub fn open(&mut self) -> Option<PathBuf> {
        if self.loading() {
            return None;
        }
        let entry = self.entries.get(self.selected)?;
        let path = entry.path.clone();
        if entry.directory {
            self.enter(path);
            None
        } else {
            Some(path)
        }
    }
}

impl Drop for Browser {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
pub fn scan(directory: &std::path::Path) -> Result<Vec<Entry>> {
    Scanner::default().scan(directory, &AtomicBool::new(false), media::supported)
}

#[cfg(test)]
#[path = "browser_tests.rs"]
pub(crate) mod tests;
