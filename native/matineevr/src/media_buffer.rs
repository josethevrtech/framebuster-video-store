use crate::{media::Decoder, statistics::Statistics};
use std::ffi::c_void;

#[repr(C)]
#[derive(Default)]
pub(crate) struct Buffer {
    video_us: i64,
    audio_us: i64,
    bytes: u64,
    pub ready: i32,
    limited: i32,
    pub waiting: i32,
}

unsafe extern "C" {
    fn media_buffer(handle: *mut c_void, prefill: i32) -> Buffer;
}

impl Decoder {
    pub(crate) fn buffer(&self, prefill: bool) -> Buffer {
        unsafe { media_buffer(self.handle, prefill.into()) }
    }
}

impl Buffer {
    pub fn sample(&self, stats: &mut Statistics) {
        stats.sample("packet_buffer_bytes", self.bytes as f64);
        stats.sample("video_buffer_ms", self.video_us as f64 / 1000.0);
        stats.sample("audio_buffer_ms", self.audio_us as f64 / 1000.0);
        stats.count("packet_buffer_limited", u64::from(self.limited != 0));
    }
}
