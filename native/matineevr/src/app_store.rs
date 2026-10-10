use super::App;
use std::path::PathBuf;

impl App {
    pub fn queue_store_movie(&mut self, path: PathBuf) {
        self.stop();
        self.pending = Some(path);
    }
}
