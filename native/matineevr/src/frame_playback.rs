use crate::app::App;
use anyhow::{Result, ensure};
use std::{path::{Path, PathBuf}, time::{Duration, Instant}};

pub fn is_stream(path: &Path) -> bool {
    let Some(value) = path.to_str().and_then(|p| p.strip_prefix("http://127.0.0.1:")) else { return false; };
    let Some((port, route)) = value.split_once('/') else { return false; };
    port.parse::<u16>().is_ok_and(|port| port >= 1024)
        && route.starts_with("__frame/media/") && !value.chars().any(char::is_whitespace)
}

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
        let data = format!("{{\"position\":{},\"paused\":{},\"ended\":{},\"error\":{}}}",
            self.position, self.paused, app.completed, app.failed);
        std::fs::write(path, data)?;
        self.next = Instant::now() + Duration::from_secs(1);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_local_media_relay_urls_are_streams() {
        assert!(is_stream(Path::new("http://127.0.0.1:1420/__frame/media/id/stream")));
        for value in ["http://jellyfin.test:8096/Videos/id/stream", "file:///etc/passwd",
            "http://127.0.0.1:1420/__frame/native/request", "http://127.0.0.1:80/__frame/media/id/stream",
            "http://127.0.0.1:1420@evil.test/__frame/media/id/stream"] {
            assert!(!is_stream(Path::new(value)));
        }
    }
}
