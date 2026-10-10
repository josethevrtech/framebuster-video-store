use crate::app::App;
use anyhow::{Result, ensure};
use std::{path::PathBuf, time::{Duration, Instant}};
pub use matineevr::stream_path::is_stream;

pub struct FramePlayback {
    path: Option<PathBuf>,
    next: Instant,
    pub return_on_stop: bool,
    position: f64,
    paused: bool,
}

impl FramePlayback {
    pub fn new(app: &mut App) -> Result<Self> {
        let start: f64 = std::env::var("HALCYON_FRAME_START_SECONDS").unwrap_or_else(|_| "0".into()).parse()?;
        ensure!(start.is_finite() && (0.0..=1e7).contains(&start), "Invalid resume position");
        if start > 0.0 && let Some(player) = &mut app.playback { player.seek(start)?; }
        Ok(Self { path: std::env::var_os("HALCYON_FRAME_PROGRESS_FILE").map(PathBuf::from),
            next: Instant::now(), return_on_stop: std::env::var("HALCYON_FRAME_RETURN").as_deref() == Ok("1"),
            position: start, paused: false })
    }

    pub fn observe(&mut self, app: &App) {
        if let Some(player) = &app.playback {
            self.position = player.seek_target().unwrap_or(player.position).max(0.0);
            self.paused = player.status() == "Paused";
        }
    }

    pub fn report(&mut self, app: &App, force: bool) -> Result<()> {
        let Some(path) = &self.path else { return Ok(()); };
        if !force && Instant::now() < self.next { return Ok(()); }
        let data = format!("{{\"position\":{},\"paused\":{},\"ended\":{},\"error\":{},\"running\":{}}}",
            self.position, self.paused, app.completed, app.failed, app.playback.is_some());
        std::fs::write(path, data)?;
        self.next = Instant::now() + Duration::from_secs(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use matineevr::stream_path::decoder_path;

    #[test]
    fn only_local_media_relay_urls_are_streams() {
        assert!(is_stream(Path::new("http://127.0.0.1:1420/__frame/media/id/stream")));
        let relay = Path::new("http://127.0.0.1:1420/__frame/media/id/media.m3u8");
        assert_eq!(decoder_path(relay).unwrap(), relay);
        for value in ["http://jellyfin.test:8096/Videos/id/stream", "file:///etc/passwd",
            "http://127.0.0.1:1420/__frame/native/request", "http://127.0.0.1:80/__frame/media/id/stream",
            "http://127.0.0.1:1420@evil.test/__frame/media/id/stream"] {
            assert!(!is_stream(Path::new(value)));
        }
    }
}
