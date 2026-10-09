use anyhow::{Context, Result};
use std::{env, ffi::OsString, os::unix::process::CommandExt, path::Path, process::Command};

const LIBRARIES: &str = "/opt/steamvr/bin/linuxarm64";
const RUNTIME: &str = "/opt/steamvr/steamxr_linuxarm64.json";
const RLIMIT_CORE: i32 = 4;

unsafe extern "C" {
    fn setrlimit(resource: i32, limits: *const [u64; 2]) -> i32;
}

pub fn initialize() -> Result<()> {
    let libraries = env::var_os("LD_LIBRARY_PATH").unwrap_or_default();
    if !env::split_paths(&libraries).any(|path| path == Path::new(LIBRARIES))
        || env::var_os("XR_RUNTIME_JSON").as_deref() != Some(RUNTIME.as_ref())
    {
        let mut search_path = OsString::from(LIBRARIES);
        if !libraries.is_empty() {
            search_path.push(":");
            search_path.push(libraries);
        }
        return Err(Command::new(env::current_exe()?)
            .args(env::args_os().skip(1))
            .env("LD_LIBRARY_PATH", search_path)
            .env("XR_RUNTIME_JSON", RUNTIME)
            .exec())
        .context("Initialize SteamVR environment");
    }
    if unsafe { setrlimit(RLIMIT_CORE, &[0, 0]) } != 0 {
        return Err(std::io::Error::last_os_error()).context("Disable core dumps");
    }
    Ok(())
}
