use crate::media::Decoder;
use anyhow::{Result, bail};
use std::ffi::{CStr, c_char, c_void};
use std::time::Duration;

unsafe extern "C" {
    fn media_enable_audio(handle: *mut c_void, error: *mut c_char, capacity: usize) -> i32;
    fn media_audio(
        handle: *mut c_void,
        paused: i32,
        clock: *mut f64,
        limit: *mut f64,
        done: *mut i32,
        error: *mut c_char,
        capacity: usize,
    ) -> i32;
}

#[derive(Clone, Copy, Default)]
pub(crate) struct AudioStatus {
    pub clock: Option<f64>,
    pub limit: f64,
    pub done: bool,
    pub progress: bool,
}

impl AudioStatus {
    pub fn position(&self, elapsed: Duration) -> Option<f64> {
        self.clock
            .filter(|_| !self.done)
            .map(|clock| (clock + elapsed.as_secs_f64()).min(self.limit))
    }
}

fn check(result: i32, error: &[c_char]) -> Result<i32> {
    if result < 0 {
        bail!(
            unsafe { CStr::from_ptr(error.as_ptr()) }
                .to_string_lossy()
                .into_owned()
        );
    }
    Ok(result)
}

impl Decoder {
    pub(crate) fn enable_audio(&mut self) -> Result<()> {
        let mut error = [0; 512];
        check(
            unsafe { media_enable_audio(self.handle, error.as_mut_ptr(), error.len()) },
            &error,
        )?;
        Ok(())
    }

    pub(crate) fn audio(&mut self, paused: bool) -> Result<AudioStatus> {
        let mut error = [0; 512];
        let mut clock = -1.0;
        let mut limit = 0.0;
        let mut done = 0;
        let result = unsafe {
            media_audio(
                self.handle,
                paused.into(),
                &mut clock,
                &mut limit,
                &mut done,
                error.as_mut_ptr(),
                error.len(),
            )
        };
        Ok(AudioStatus {
            clock: (clock >= 0.0).then_some(clock),
            limit,
            done: done != 0,
            progress: check(result, &error)? > 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_accounts_for_age_without_passing_queued_audio() {
        let mut audio = AudioStatus {
            clock: Some(1.0),
            limit: 1.25,
            ..AudioStatus::default()
        };
        assert_eq!(audio.position(Duration::from_millis(20)), Some(1.02));
        audio.clock = Some(1.02);
        assert_eq!(audio.position(Duration::ZERO), Some(1.02));
        assert_eq!(audio.position(Duration::from_secs(1)), Some(1.25));
        audio.clock = Some(3.0);
        audio.limit = 3.0;
        assert_eq!(audio.position(Duration::from_secs(1)), Some(3.0));
        audio.done = true;
        assert_eq!(audio.position(Duration::ZERO), None);
        assert_eq!(AudioStatus::default().position(Duration::ZERO), None);
    }
}
