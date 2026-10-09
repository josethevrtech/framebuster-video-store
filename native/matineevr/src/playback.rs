use crate::{
    decoder_worker::Worker,
    media::{Decoded, Frame},
};
use anyhow::{Result, ensure};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "audio_playback_tests.rs"]
mod audio_tests;

#[path = "playback_info.rs"]
mod info;
#[path = "playback_stats.rs"]
mod stats;

pub struct Playback {
    worker: Worker,
    pending: Option<Frame>,
    preview: Option<Frame>,
    started: Option<Instant>,
    first_pts: f64,
    pause_started: Option<Instant>,
    manual_pause: bool,
    suspended: bool,
    paused_time: Duration,
    ended: bool,
    target: Option<f64>,
    stats: Option<stats::PlaybackStats>,
    pub length: Option<f64>,
    pub position: f64,
    pub skipped: u64,
}

impl Playback {
    pub fn start(path: PathBuf) -> Self {
        Self {
            worker: Worker::start(path),
            pending: None,
            preview: None,
            started: None,
            first_pts: 0.0,
            pause_started: None,
            manual_pause: false,
            suspended: false,
            paused_time: Duration::ZERO,
            ended: false,
            target: None,
            stats: None,
            length: None,
            position: 0.0,
            skipped: 0,
        }
    }

    pub fn skip(&mut self, seconds: i32) {
        self.length = self.length.or_else(|| self.worker.length());
        let end = self.length.unwrap_or(f64::INFINITY);
        let position = self.target.unwrap_or(self.position);
        let target = (position.min(end) + f64::from(seconds)).max(0.0).min(end);
        if target != position {
            self.request_seek(target);
        }
    }

    pub fn seek(&mut self, seconds: f64) -> Result<()> {
        ensure!(seconds.is_finite() && seconds >= 0.0, "Invalid seek target");
        self.length = self.length.or_else(|| self.worker.length());
        self.request_seek(seconds.min(self.length.unwrap_or(seconds)));
        Ok(())
    }

    pub fn seek_target(&self) -> Option<f64> {
        self.target
    }

    pub fn take_preview(&mut self) -> Option<Frame> {
        self.preview.take()
    }

    fn request_seek(&mut self, target: f64) {
        self.pending = None;
        self.preview = None;
        self.target = Some(target);
        self.started = None;
        self.ended = false;
        self.worker.seek(target);
        eprintln!("Seek: {target:.3}s");
    }

    pub fn toggle_pause(&mut self) {
        self.manual_pause = !self.manual_pause;
        self.update_pause();
        eprintln!(
            "{}",
            if self.manual_pause {
                "Paused"
            } else {
                "Playing"
            }
        );
    }

    pub fn set_suspended(&mut self, suspended: bool) {
        self.suspended = suspended;
        self.update_pause();
    }

    fn update_pause(&mut self) {
        if (self.manual_pause || self.suspended) && self.pause_started.is_none() {
            self.pause_started = Some(Instant::now());
        } else if !self.manual_pause
            && !self.suspended
            && let Some(start) = self.pause_started.take()
        {
            self.paused_time += start.elapsed();
        }
        self.worker.pause(self.pause_started.is_some());
    }

    fn poll_frame(&mut self) -> Result<Option<Frame>> {
        self.length = self.length.or_else(|| self.worker.length());
        if self.pause_started.is_some() && self.target.is_none() {
            return Ok(None);
        }
        let mut latest = None;
        loop {
            if self.pending.is_none() && !self.ended {
                match self.worker.next()? {
                    Decoded::Frame(frame) => self.pending = Some(frame),
                    Decoded::Preview(frame) => {
                        self.preview = Some(frame);
                        break;
                    }
                    Decoded::End => {
                        self.ended = true;
                        if let Some(target) = self.target.take() {
                            self.position = target.min(self.length.unwrap_or(target));
                        }
                        eprintln!("End of video");
                    }
                    Decoded::Pending => break,
                }
            }
            let Some(frame) = &self.pending else { break };
            let first = self.started.is_none();
            if first && !self.worker.ready()? {
                break;
            }
            if first {
                self.started = Some(Instant::now());
                self.first_pts = frame.pixels.pts;
                self.paused_time = Duration::ZERO;
                self.target = None;
                if self.pause_started.is_some() {
                    self.pause_started = self.started;
                }
                self.worker.begin();
            }
            if let Some(clock) = self.worker.clock() {
                self.first_pts = clock;
                self.started = Some(Instant::now());
                self.paused_time = Duration::ZERO;
            }
            let time = self
                .started
                .unwrap()
                .elapsed()
                .saturating_sub(self.paused_time)
                .as_secs_f64();
            let early = !first && frame.pixels.pts - self.first_pts > time;
            if let Some(stats) = &mut self.stats {
                let lead = (frame.pixels.pts - self.first_pts - time) * 1000.0;
                if early {
                    stats.summary.sample("pending_lead_ms", lead);
                } else {
                    stats.selected_lead_ms = lead;
                }
            }
            if early {
                break;
            }
            if latest.is_some() {
                self.skipped += 1;
            }
            latest = self.pending.take();
            if self.pause_started.is_some() {
                break;
            }
        }
        if let Some(frame) = &latest {
            self.position = frame.pixels.pts.max(0.0);
        }
        Ok(latest)
    }
}
