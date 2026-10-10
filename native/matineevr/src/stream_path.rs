use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn is_stream(path: &Path) -> bool {
    let Some(value) = path.to_str().and_then(|p| p.strip_prefix("http://127.0.0.1:")) else { return false; };
    let Some((port, route)) = value.split_once('/') else { return false; };
    port.parse::<u16>().is_ok_and(|port| port >= 1024)
        && route.starts_with("__frame/media/") && !value.chars().any(char::is_whitespace)
}

pub fn decoder_path(path: &Path) -> Result<PathBuf> {
    if is_stream(path) { Ok(path.to_path_buf()) }
    else { path.canonicalize().context("Resolve local video path") }
}
