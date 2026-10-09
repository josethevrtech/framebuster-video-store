use crate::media::Decoder;
use std::{ffi::c_void, time::Instant};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NativeTrace {
    pub audio_reset_us: i64,
    pub seek_us: i64,
    pub close_us: i64,
    pub open_us: i64,
    pub read_us: i64,
    pub send_us: i64,
    pub receive_us: i64,
    pub audio_us: i64,
    pub seek_calls: u64,
    pub packets: u64,
    pub packet_bytes: u64,
    pub send_calls: u64,
    pub receive_calls: u64,
    pub again: u64,
    pub frames: u64,
    pub rejected: u64,
    pub audio_underflows: u64,
    pub audio_latency_samples: u64,
    pub audio_latency_us: i64,
}

#[derive(Clone, Copy, Debug)]
pub struct SeekTrace {
    pub requested: Instant,
    pub worker: Instant,
    pub released: Instant,
    pub published: Instant,
    pub native: NativeTrace,
}

unsafe extern "C" {
    fn media_profile(handle: *mut c_void, continuous: i32);
    fn media_trace(handle: *mut c_void) -> NativeTrace;
}

impl Decoder {
    pub(crate) fn profile(&mut self, continuous: bool) {
        unsafe { media_profile(self.handle, continuous.into()) };
    }

    pub(crate) fn trace(&self) -> NativeTrace {
        unsafe { media_trace(self.handle) }
    }
}

impl std::ops::Sub for NativeTrace {
    type Output = Self;

    fn sub(self, before: Self) -> Self {
        Self {
            audio_reset_us: self.audio_reset_us - before.audio_reset_us,
            seek_us: self.seek_us - before.seek_us,
            close_us: self.close_us - before.close_us,
            open_us: self.open_us - before.open_us,
            read_us: self.read_us - before.read_us,
            send_us: self.send_us - before.send_us,
            receive_us: self.receive_us - before.receive_us,
            audio_us: self.audio_us - before.audio_us,
            seek_calls: self.seek_calls - before.seek_calls,
            packets: self.packets - before.packets,
            packet_bytes: self.packet_bytes - before.packet_bytes,
            send_calls: self.send_calls - before.send_calls,
            receive_calls: self.receive_calls - before.receive_calls,
            again: self.again - before.again,
            frames: self.frames - before.frames,
            rejected: self.rejected - before.rejected,
            audio_underflows: self.audio_underflows - before.audio_underflows,
            audio_latency_samples: self.audio_latency_samples - before.audio_latency_samples,
            audio_latency_us: self.audio_latency_us - before.audio_latency_us,
        }
    }
}

impl NativeTrace {
    pub(crate) fn report(self) -> String {
        format!(
            "read_ms={:.3} send_ms={:.3} receive_ms={:.3} audio_ms={:.3} seek_ms={:.3} close_ms={:.3} open_ms={:.3} audio_reset_ms={:.3} seeks={} packets={} packet_bytes={} sends={} receives={} eagain={} decoded={} rejected={} audio_underflows={} audio_latency_samples={} audio_latency_mean_ms={:.3}",
            self.read_us as f64 / 1000.0,
            self.send_us as f64 / 1000.0,
            self.receive_us as f64 / 1000.0,
            self.audio_us as f64 / 1000.0,
            self.seek_us as f64 / 1000.0,
            self.close_us as f64 / 1000.0,
            self.open_us as f64 / 1000.0,
            self.audio_reset_us as f64 / 1000.0,
            self.seek_calls,
            self.packets,
            self.packet_bytes,
            self.send_calls,
            self.receive_calls,
            self.again,
            self.frames,
            self.rejected,
            self.audio_underflows,
            self.audio_latency_samples,
            self.audio_latency_us as f64 / self.audio_latency_samples.max(1) as f64 / 1000.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_deltas_keep_signed_audio_latency_and_seek_counts() {
        let before = NativeTrace {
            frames: 10,
            seek_calls: 2,
            audio_latency_samples: 3,
            audio_latency_us: 1000,
            ..Default::default()
        };
        let after = NativeTrace {
            frames: 15,
            seek_calls: 3,
            audio_latency_samples: 5,
            audio_latency_us: -3000,
            ..before
        };
        let delta = after - before;
        assert_eq!(delta.frames, 5);
        assert_eq!(delta.seek_calls, 1);
        assert!(delta.report().contains("audio_latency_mean_ms=-2.000"));
        assert_eq!((after - after).frames, 0);
    }
}
