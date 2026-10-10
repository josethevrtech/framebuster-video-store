use crate::{
    browser::Browser,
    hud_text::{Canvas, HEIGHT, WIDTH, escape},
    shortcuts::PanelKind,
};
use std::path::Path;

const SCROLL_PADDING: usize = 5;

pub fn panel_size(kind: PanelKind, playing: bool) -> [usize; 2] {
    if kind == PanelKind::Browser && !playing {
        [WIDTH, HEIGHT * 4]
    } else {
        [WIDTH, HEIGHT]
    }
}

impl Canvas {
    pub fn browser(&mut self, browser: &Browser, stopping: bool, elapsed: f64) {
        self.clear();
        let white = [235, 240, 245, 255];
        let cyan = [100, 215, 255, 255];
        let columns = self.columns();
        let rows = self.rows() - 4;
        self.line(
            0,
            page(&escape(&browser.directory), elapsed, columns),
            white,
        );
        if self.browser_directory != browser.directory {
            self.browser_directory.clone_from(&browser.directory);
            self.browser_start = 0;
        }
        let start = scroll_start(
            self.browser_start,
            browser.selected,
            browser.entries.len(),
            rows,
        );
        self.browser_start = start;
        for (index, entry) in browser.entries.iter().enumerate().skip(start).take(rows) {
            let name = if Some(entry.path.as_path()) == browser.directory.parent() {
                "..".into()
            } else {
                escape(Path::new(entry.path.file_name().unwrap_or_default()))
            };
            let selected = index == browser.selected;
            let label = format!(
                "{} {}{}",
                if selected { ">" } else { " " },
                if entry.directory { "[DIR] " } else { "" },
                name
            );
            self.line(
                index - start + 2,
                &label,
                if selected { cyan } else { white },
            );
        }
        if let Some(entry) = browser.entries.get(browser.selected) {
            self.line(rows + 2, page(&escape(&entry.path), elapsed, columns), cyan);
        }
        let status = if stopping {
            "Stopping previous video...".into()
        } else if browser.loading() {
            "Scanning video metadata...".into()
        } else if !browser.message.is_empty() {
            browser.message.clone()
        } else if browser.entries.iter().all(|entry| entry.directory) {
            "No supported videos here. Select a folder.".into()
        } else {
            format!("{} / {}", browser.selected + 1, browser.entries.len())
        };
        self.line(
            rows + 3,
            page(&escape(Path::new(&status)), elapsed, columns),
            if browser.message.is_empty() {
                white
            } else {
                [255, 160, 130, 255]
            },
        );
    }
}

pub(crate) fn scroll_start(start: usize, selected: usize, entries: usize, rows: usize) -> usize {
    let padding = SCROLL_PADDING.min((rows - 1) / 2);
    start
        .min(selected.saturating_sub(padding))
        .max((selected + padding + 1).saturating_sub(rows))
        .min(entries.saturating_sub(rows))
}

fn page(text: &str, elapsed: f64, columns: usize) -> &str {
    let start = elapsed as usize / 3 % text.len().div_ceil(columns).max(1) * columns;
    &text[start..(start + columns).min(text.len())]
}

#[cfg(test)]
#[path = "browser_view_tests.rs"]
mod tests;
