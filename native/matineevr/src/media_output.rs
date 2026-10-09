use super::{Decoder, Frame, media_reconfigure};
use anyhow::{Result, bail};
use std::ffi::CStr;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DmaBuf {
    pub fd: i32,
    pub format: u32,
    pub size: u64,
    pub modifier: u64,
    pub offsets: [u64; 2],
    pub pitches: [u64; 2],
    pub pool: u64,
    pub width: i32,
    pub height: i32,
    pub crop: [i32; 4],
    pub chroma_location: i32,
    pub stream: i32,
    pub transfer: i32,
    pub primaries: i32,
}

impl Frame {
    pub fn dmabuf(&self) -> DmaBuf {
        self.pixels.dmabuf
    }
}

impl Decoder {
    pub fn reconfigure(&mut self, released: bool) -> Result<bool> {
        let mut error = [0; 512];
        let result = unsafe {
            media_reconfigure(
                self.handle,
                released as i32,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        if result < 0 {
            bail!(
                unsafe { CStr::from_ptr(error.as_ptr()) }
                    .to_string_lossy()
                    .into_owned()
            );
        }
        Ok(result != 0)
    }
}
