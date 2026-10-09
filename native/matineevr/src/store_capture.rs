use anyhow::{Result, ensure};
use matineevr::{graphics::Graphics, renderer::Target};
use std::{fs, path::PathBuf, rc::Rc};

pub fn capture(device: &Rc<Graphics>, target: Target, previous: &mut String) -> Result<()> {
    let Some(directory) = std::env::var_os("HALCYON_FRAME_STORE_IPC") else { return Ok(()); };
    let directory = PathBuf::from(directory);
    let Ok(name) = fs::read_to_string(directory.join("capture-request")) else { return Ok(()); };
    if name == *previous { return Ok(()); }
    ensure!(name.len() <= 100 && name.ends_with(".ppm")
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')),
        "Invalid scene capture filename");
    let path = directory.join(&name);
    unsafe { device.api.device_wait_idle()?; }
    crate::snapshot::save(device, &path, target)?;
    *previous = name;
    Ok(())
}
