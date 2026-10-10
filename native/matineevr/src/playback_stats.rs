use super::*;
use crate::{media_trace::NativeTrace, statistics::Statistics};

pub(super) struct PlaybackStats {
    pub summary: Statistics,
    pub selected_lead_ms: f64,
    report: Instant,
    native: NativeTrace,
}

impl Playback {
    pub fn stop(mut self) -> std::thread::JoinHandle<()> {
        self.report_stats(true);
        self.worker.stop()
    }

    pub fn diagnostics(&mut self) {
        if self.stats.is_none() {
            self.worker.diagnostics();
            self.stats = Some(PlaybackStats {
                summary: Statistics::default(),
                selected_lead_ms: 0.0,
                report: Instant::now(),
                native: NativeTrace::default(),
            });
        }
    }

    pub fn due_frame(&mut self) -> Result<Option<Frame>> {
        let skipped = self.skipped;
        let state = self.status();
        let frame = self.poll_frame()?;
        if let Some(stats) = &mut self.stats {
            stats.summary.count(state, 1);
            stats.summary.count("fresh", u64::from(frame.is_some()));
            stats.summary.count("repeat", u64::from(frame.is_none()));
            stats.summary.count("skipped", self.skipped - skipped);
            if frame.is_some() {
                stats
                    .summary
                    .sample("selected_lead_ms", stats.selected_lead_ms);
            }
            if frame.is_none() && state == "Playing" {
                stats.summary.count(
                    if self.pending.is_some() {
                        "early"
                    } else {
                        "empty"
                    },
                    1,
                );
            }
        }
        self.report_stats(false);
        Ok(frame)
    }

    pub(super) fn report_stats(&mut self, final_report: bool) {
        let Some(stats) = &mut self.stats else { return };
        let seconds = stats.report.elapsed().as_secs_f64();
        if !final_report && seconds < 5.0 {
            return;
        }
        let (worker, native) = self.worker.take_stats();
        stats.summary.merge(worker);
        eprintln!(
            "{} {}",
            stats.summary.report("playback", seconds),
            (native - stats.native).report()
        );
        stats.native = native;
        stats.report = Instant::now();
    }
}
