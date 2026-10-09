#[path = "media_output.rs"]
mod output;
pub use output::DmaBuf;

use anyhow::{Context, Result, bail};
use std::ffi::{CStr, CString, c_char, c_void};
use std::path::Path;

#[repr(C)]
#[derive(Default)]
pub struct Pixels {
    owner: *mut c_void,
    pub width: i32,
    pub height: i32,
    pub colorspace: i32,
    pub full_range: i32,
    pub sample_aspect_ratio: f32,
    pub pts: f64,
    pub dmabuf: DmaBuf,
}

unsafe extern "C" {
    fn media_open(path: *const c_char, error: *mut c_char, capacity: usize) -> *mut c_void;
    fn media_reconfigure(
        handle: *mut c_void,
        released: i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn media_duration(handle: *mut c_void) -> f64;
    fn media_metadata(handle: *mut c_void, fields: *mut i32);
    fn media_seek(handle: *mut c_void, seconds: f64, error: *mut c_char, capacity: usize) -> i32;
    fn media_next(
        handle: *mut c_void,
        frame: *mut Pixels,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
    fn media_release(frame: *mut Pixels);
    fn media_stop(handle: *mut c_void);
    fn media_close(handle: *mut c_void);
    fn media_supported(path: *const c_char, error: *mut c_char, capacity: usize) -> i32;
}

pub struct Decoder {
    pub(crate) handle: *mut c_void,
}

pub struct Frame {
    pub pixels: Pixels,
}

pub enum Decoded {
    Frame(Frame),
    Preview(Frame),
    Pending,
    End,
}

unsafe impl Send for Frame {}

pub fn supported(path: &Path) -> Result<bool> {
    let path = CString::new(path.as_os_str().as_encoded_bytes())?;
    let mut error = [0; 512];
    match unsafe { media_supported(path.as_ptr(), error.as_mut_ptr(), error.len()) } {
        0 => Ok(false),
        1 => Ok(true),
        _ => bail!(
            unsafe { CStr::from_ptr(error.as_ptr()) }
                .to_string_lossy()
                .into_owned()
        ),
    }
}

impl Decoder {
    pub fn open(path: &Path) -> Result<Self> {
        let path = path.canonicalize().context("Resolve local video path")?;
        let path = CString::new(path.as_os_str().as_encoded_bytes())?;
        let mut error = [0; 512];
        let handle = unsafe { media_open(path.as_ptr(), error.as_mut_ptr(), error.len()) };
        if handle.is_null() {
            bail!(
                unsafe { CStr::from_ptr(error.as_ptr()) }
                    .to_string_lossy()
                    .into_owned()
            );
        }
        Ok(Self { handle })
    }

    pub fn duration(&self) -> Option<f64> {
        let seconds = unsafe { media_duration(self.handle) };
        (seconds.is_finite() && seconds > 0.0).then_some(seconds)
    }

    pub fn metadata(&self) -> crate::video_settings::Hints {
        let mut fields = [-1; 3];
        unsafe { media_metadata(self.handle, fields.as_mut_ptr()) };
        crate::video_settings::Hints::native(fields)
    }

    pub fn next_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            self.reconfigure(true)?;
            match self.advance()? {
                Decoded::Frame(frame) => return Ok(Some(frame)),
                Decoded::End => return Ok(None),
                Decoded::Pending if self.buffer(false).waiting != 0 => {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                Decoded::Pending | Decoded::Preview(_) => {}
            }
        }
    }

    pub fn seek(&mut self, seconds: f64) -> Result<()> {
        let mut error = [0; 512];
        let result = unsafe { media_seek(self.handle, seconds, error.as_mut_ptr(), error.len()) };
        if result < 0 {
            bail!(
                unsafe { CStr::from_ptr(error.as_ptr()) }
                    .to_string_lossy()
                    .into_owned()
            );
        }
        Ok(())
    }

    pub fn advance(&mut self) -> Result<Decoded> {
        let mut pixels = Pixels::default();
        let mut error = [0; 512];
        match unsafe { media_next(self.handle, &mut pixels, error.as_mut_ptr(), error.len()) } {
            0 => Ok(Decoded::End),
            1 => Ok(Decoded::Frame(Frame { pixels })),
            2 => Ok(Decoded::Pending),
            3 => Ok(Decoded::Preview(Frame { pixels })),
            _ => bail!(
                unsafe { CStr::from_ptr(error.as_ptr()) }
                    .to_string_lossy()
                    .into_owned()
            ),
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            media_stop(self.handle);
            media_close(self.handle);
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe { media_release(&mut self.pixels) };
    }
}
