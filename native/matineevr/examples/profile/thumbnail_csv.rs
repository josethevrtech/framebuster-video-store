use crate::measure::Sample;
use anyhow::Result;
use std::{
    io::{self, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub const HEADER: &str = "run,request,operation,target_s,visit,cache,status,elapsed_ms,preview_pts_s,preview_ms,preview_ready_ms,release_handshake_ms,worker_wait_ms,handoff_ms,audio_reset_ms,container_seek_ms,decoder_close_ms,decoder_open_ms,read_ms,send_ms,receive_ms,audio_ms,seek_calls,packets,packet_bytes,send_calls,receive_calls,eagain,decoded_frames,rejected_frames,import_ms,draw_ms,gpu_ms,submit_ms,fence_wait_ms,frames,skipped,first_pts_s,audio_clock_s,scenario";

#[derive(Clone, Copy)]
pub struct Request {
    pub scenario: &'static str,
    pub run: u32,
    pub index: usize,
    pub operation: &'static str,
    pub target: Option<f64>,
    pub repeated: bool,
    pub started: Instant,
}

impl Request {
    pub fn fields(self, status: &str, sample: Option<&Sample>) -> Vec<String> {
        let number = |value: Option<f64>| value.map(|v| format!("{v:.6}")).unwrap_or_default();
        let ms = |duration: Duration| duration.as_secs_f64() * 1000.0;
        let mut fields = vec![
            self.run.to_string(),
            self.index.to_string(),
            self.operation.into(),
            number(self.target),
            if self.repeated { "repeat" } else { "first" }.into(),
            "uncontrolled".into(),
            status.into(),
            number(Some(ms(
                sample.map_or_else(|| self.started.elapsed(), |s| s.elapsed)
            ))),
        ];
        if let Some(s) = sample {
            let t = s.trace;
            fields.extend([
                number(s.preview_pts),
                number(s.preview.map(ms)),
                number(s.preview_ready.map(ms)),
                number(t.map(|t| ms(t.released.duration_since(t.requested)))),
                number(t.map(|t| ms(t.worker.duration_since(t.requested)))),
                number(
                    t.zip(s.consumed)
                        .map(|(t, time)| ms(time.duration_since(t.published))),
                ),
            ]);
            let native = t.map(|t| t.native);
            for value in [
                native.map(|t| t.audio_reset_us),
                native.map(|t| t.seek_us),
                native.map(|t| t.close_us),
                native.map(|t| t.open_us),
                native.map(|t| t.read_us),
                native.map(|t| t.send_us),
                native.map(|t| t.receive_us),
                native.map(|t| t.audio_us),
            ] {
                fields.push(number(value.map(|v| v as f64 / 1000.0)));
            }
            for value in [
                native.map(|t| t.seek_calls),
                native.map(|t| t.packets),
                native.map(|t| t.packet_bytes),
                native.map(|t| t.send_calls),
                native.map(|t| t.receive_calls),
                native.map(|t| t.again),
                native.map(|t| t.frames),
                native.map(|t| t.rejected),
            ] {
                fields.push(value.map(|v| v.to_string()).unwrap_or_default());
            }
            for value in [
                s.preview_import,
                ms(s.preview_draw),
                s.preview_gpu,
                s.preview_submit,
                s.preview_wait,
            ] {
                fields.push(number(s.preview_ready.map(|_| value)));
            }
            fields.extend([
                s.frames.to_string(),
                s.skipped.to_string(),
                number(s.first),
                number(s.audio_clock),
            ]);
        }
        if sample.is_none() {
            fields.resize(HEADER.split(',').count() - 1, String::new());
        }
        fields.push(self.scenario.into());
        fields
    }

    pub fn write(self, status: &str, sample: Option<&Sample>) -> Result<()> {
        let mut output = io::stdout().lock();
        writeln!(output, "{}", self.fields(status, sample).join(","))?;
        output.flush()?;
        Ok(())
    }

    pub fn bounded<T>(self, allowance: Duration, work: impl FnOnce() -> Result<T>) -> Result<T> {
        let (done, wait) = mpsc::channel();
        let watchdog = thread::spawn(move || {
            if wait.recv_timeout(allowance.saturating_sub(self.started.elapsed()))
                == Err(mpsc::RecvTimeoutError::Timeout)
            {
                eprintln!(
                    "{} timed out after {}s",
                    self.operation,
                    allowance.as_secs_f64()
                );
                if let Err(error) = self.write("timeout", None) {
                    eprintln!("Write timeout result: {error}");
                }
                std::process::exit(124);
            }
        });
        let result = work();
        let _ = done.send(());
        watchdog.join().expect("Profiling watchdog panicked");
        if let Err(error) = &result {
            let timeout = error
                .downcast_ref::<io::Error>()
                .is_some_and(|e| e.kind() == io::ErrorKind::TimedOut);
            self.write(if timeout { "timeout" } else { "error" }, None)?;
        }
        result
    }
}
