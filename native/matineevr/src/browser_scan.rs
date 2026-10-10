use super::Entry;
use anyhow::{Context, Result, anyhow};
use std::{
    collections::HashMap,
    fs::{self, Metadata},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

const WORKERS: usize = 8;

#[derive(Default)]
pub struct Scanner {
    cache: Mutex<HashMap<PathBuf, (Metadata, bool)>>,
}

impl Scanner {
    pub fn scan(
        &self,
        directory: &Path,
        cancelled: &AtomicBool,
        supported: impl Fn(&Path) -> Result<bool> + Sync,
    ) -> Result<Vec<Entry>> {
        let files = Mutex::new(
            fs::read_dir(directory).with_context(|| format!("Read {}", directory.display()))?,
        );
        thread::scope(|scope| {
            let workers: Vec<_> = (0..WORKERS)
                .map(|_| {
                    scope.spawn(|| {
                        let mut entries = Vec::new();
                        while !cancelled.load(Ordering::Relaxed) {
                            let Some(file) = files.lock().unwrap().next() else {
                                break;
                            };
                            let path = file?.path();
                            let metadata = match fs::metadata(&path) {
                                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                                    continue;
                                }
                                result => {
                                    result.with_context(|| format!("Stat {}", path.display()))?
                                }
                            };
                            let directory = metadata.is_dir();
                            if directory
                                || metadata.is_file()
                                    && self.supported(&path, metadata, &supported).unwrap_or_else(
                                        |error| {
                                            eprintln!("{error:#}");
                                            false
                                        },
                                    )
                            {
                                entries.push(Entry { path, directory });
                            }
                        }
                        Ok::<_, anyhow::Error>(entries)
                    })
                })
                .collect();
            let results: Vec<_> = workers
                .into_iter()
                .map(|worker| {
                    worker
                        .join()
                        .map_err(|_| anyhow!("Metadata worker panicked"))?
                })
                .collect();
            let mut entries: Vec<_> = results
                .into_iter()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect();
            entries.sort_by(|a, b| b.directory.cmp(&a.directory).then(a.path.cmp(&b.path)));
            Ok(entries)
        })
    }

    fn supported(
        &self,
        path: &Path,
        metadata: Metadata,
        probe: &impl Fn(&Path) -> Result<bool>,
    ) -> Result<bool> {
        if let Some((saved, supported)) = self.cache.lock().unwrap().get(path)
            && unchanged(saved, &metadata)
        {
            return Ok(*supported);
        }
        let supported = probe(path).with_context(|| format!("Probe {}", path.display()))?;
        self.cache
            .lock()
            .unwrap()
            .insert(path.into(), (metadata, supported));
        Ok(supported)
    }
}

fn unchanged(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev()
        && a.ino() == b.ino()
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}

#[cfg(test)]
#[path = "browser_scan_tests.rs"]
mod tests;
