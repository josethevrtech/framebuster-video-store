use crate::audio::AudioStatus;
use crate::media::{Decoded, Decoder};
use crate::media_trace::{NativeTrace, SeekTrace};
use crate::statistics::Statistics;
use anyhow::{Result, bail};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[path = "decoder_run.rs"]
mod run;

#[derive(Default)]
struct State {
    frames: VecDeque<Result<Decoded>>,
    seek: Option<f64>,
    generation: u64,
    stopped: bool,
    release_required: bool,
    length: Option<f64>,
    metadata: Option<crate::video_settings::Hints>,
    paused: bool,
    running: bool,
    ready: bool,
    audio: AudioStatus,
    profiling: bool,
    requested: Option<Instant>,
    released: Option<Instant>,
    trace: Option<SeekTrace>,
    stats: Option<Statistics>,
    native: NativeTrace,
    native_observed: Option<Instant>,
    audio_observed: Option<Instant>,
}

type Shared = Arc<(Mutex<State>, Condvar)>;

pub struct Worker {
    shared: Shared,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn start(path: PathBuf) -> Self {
        let shared = Shared::default();
        let state = shared.clone();
        let thread = thread::spawn(move || {
            if let Err(error) = run::decode(path, &state) {
                state.0.lock().unwrap().frames.push_back(Err(error));
            }
        });
        Self {
            shared,
            thread: Some(thread),
        }
    }

    pub fn release_required(&self) -> bool {
        self.shared.0.lock().unwrap().release_required
    }

    pub fn released(&self) {
        let mut state = self.shared.0.lock().unwrap();
        state.release_required = false;
        state.released = state.profiling.then(Instant::now);
        self.shared.1.notify_one();
    }

    pub fn length(&self) -> Option<f64> {
        self.shared.0.lock().unwrap().length
    }

    pub fn take_metadata(&self) -> Option<crate::video_settings::Hints> {
        self.shared.0.lock().unwrap().metadata.take()
    }

    pub fn seek(&self, target: f64) {
        let mut state = self.shared.0.lock().unwrap();
        state.requested = state.profiling.then(Instant::now);
        state.released = None;
        state.trace = None;
        state.frames.clear();
        state.generation += 1;
        state.seek = Some(target);
        state.running = false;
        state.ready = false;
        state.release_required = true;
        state.audio.clock = state.audio.clock.map(|_| target);
        state.audio.limit = target;
        state.audio_observed = None;
        state.audio.done = false;
        self.shared.1.notify_one();
    }

    pub fn profile(&self) {
        self.shared.0.lock().unwrap().profiling = true;
    }

    pub fn take_trace(&self) -> Option<SeekTrace> {
        self.shared.0.lock().unwrap().trace.take()
    }

    pub fn pause(&self, paused: bool) {
        let mut state = self.shared.0.lock().unwrap();
        if state.paused != paused {
            state.paused = paused;
            self.shared.1.notify_one();
        }
    }

    pub fn ready(&self) -> Result<bool> {
        let mut state = self.shared.0.lock().unwrap();
        if let Some(index) = state.frames.iter().position(Result::is_err) {
            state.frames.remove(index).unwrap()?;
        }
        Ok(state.ready)
    }

    pub fn begin(&self) {
        self.shared.0.lock().unwrap().running = true;
        self.shared.1.notify_one();
    }

    pub fn clock(&self) -> Option<f64> {
        let mut state = self.shared.0.lock().unwrap();
        let elapsed = state.audio_observed.map(|observed| observed.elapsed());
        if let Some(elapsed) = elapsed
            && !state.audio.done
            && let Some(stats) = &mut state.stats
        {
            stats.sample("audio_observation_age_ms", elapsed.as_secs_f64() * 1000.0);
        }
        state.audio.position(elapsed.unwrap_or_default())
    }

    pub fn diagnostics(&self) {
        self.shared.0.lock().unwrap().stats.get_or_insert_default();
    }

    pub fn take_stats(&self) -> (Statistics, NativeTrace) {
        let mut state = self.shared.0.lock().unwrap();
        if let Some(observed) = state.native_observed {
            state.stats.as_mut().unwrap().sample(
                "native_snapshot_age_ms",
                observed.elapsed().as_secs_f64() * 1000.0,
            );
        }
        (std::mem::take(state.stats.as_mut().unwrap()), state.native)
    }

    pub fn next(&self) -> Result<Decoded> {
        let mut state = self.shared.0.lock().unwrap();
        let queued = state.frames.len();
        if let Some(stats) = &mut state.stats {
            stats.sample("queue_frames", queued as f64);
        }
        if state.frames.len() == 2 {
            self.shared.1.notify_one();
        }
        match state.frames.pop_front() {
            Some(frame) => frame,
            None if self.thread.as_ref().unwrap().is_finished() => {
                bail!("Decoder worker stopped unexpectedly")
            }
            None => Ok(Decoded::Pending),
        }
    }

    pub fn stop(mut self) -> JoinHandle<()> {
        self.thread.take().unwrap()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let mut state = self.shared.0.lock().unwrap();
        state.stopped = true;
        state.frames.clear();
        self.shared.1.notify_one();
    }
}
