use super::*;

pub(super) fn decode(path: PathBuf, shared: &Shared) -> Result<()> {
    let mut decoder = Decoder::open(&path)?;
    decoder.enable_audio()?;
    let mut state = shared.0.lock().unwrap();
    state.length = decoder.duration();
    state.metadata = Some(decoder.metadata());
    let mut video_ended = false;
    let mut reported_end = false;
    let mut trace_request = None;
    let mut trace_start = NativeTrace::default();
    loop {
        if state.stopped {
            return Ok(());
        }
        if state.release_required {
            state = shared.1.wait(state).unwrap();
            continue;
        }
        let generation = state.generation;
        let seek = state.seek.take();
        let requested = state.requested;
        let released = state.released;
        let prefill = !state.running;
        let paused = state.paused || prefill;
        let room = state.frames.len() < 2;
        let diagnostic = state.stats.is_some();
        drop(state);
        if diagnostic {
            decoder.profile(true);
        }
        let mut audio_observed = None;
        let result: Result<_> = (|| {
            if let Some(target) = seek {
                trace_request = requested.zip(released).map(|times| (times, Instant::now()));
                if trace_request.is_some() {
                    decoder.profile(false);
                }
                trace_start = if diagnostic {
                    decoder.trace()
                } else {
                    NativeTrace::default()
                };
                decoder.seek(target)?;
                video_ended = false;
                reported_end = false;
            }
            decoder.reconfigure(true)?;
            let audio = decoder.audio(paused)?;
            audio_observed = audio.clock.map(|_| Instant::now());
            let frame = if !video_ended && room {
                decoder.advance()?
            } else {
                Decoded::Pending
            };
            Ok((audio, frame, decoder.reconfigure(false)?))
        })();
        let buffer = decoder.buffer(prefill);
        state = shared.0.lock().unwrap();
        if diagnostic {
            state.native = decoder.trace();
            state.native_observed = Some(Instant::now());
        }
        if state.stopped || state.generation != generation {
            continue;
        }
        if !matches!(&result, Ok((_, Decoded::Pending, _)) | Ok((_, _, true)))
            && let Some(((requested, released), worker)) = trace_request.take()
        {
            state.trace = Some(SeekTrace {
                requested,
                worker,
                released,
                published: Instant::now(),
                native: decoder.trace() - trace_start,
            });
        }
        let (audio, result, changed) = result?;
        if changed {
            drop(result);
            state.frames.clear();
            state.release_required = true;
            while state.release_required && !state.stopped {
                state = shared.1.wait(state).unwrap();
            }
            continue;
        }
        state.ready = buffer.ready != 0;
        if let Some(stats) = &mut state.stats {
            buffer.sample(stats);
        }
        let waiting_packet = matches!(result, Decoded::Pending) && buffer.waiting != 0;
        state.audio = audio;
        state.audio_observed = audio_observed;
        match result {
            Decoded::Pending => {}
            Decoded::End => {
                video_ended = true;
                state.running = true;
            }
            result => state.frames.push_back(Ok(result)),
        }
        if video_ended && audio.done && !reported_end {
            state.frames.push_back(Ok(Decoded::End));
            reported_end = true;
        }
        if !audio.progress && (waiting_packet || video_ended || state.frames.len() >= 2) {
            let waiting = diagnostic.then(Instant::now);
            state = if waiting_packet || !state.ready || (audio.clock.is_some() && !audio.done) {
                shared
                    .1
                    .wait_timeout(state, Duration::from_millis(5))
                    .unwrap()
                    .0
            } else {
                shared.1.wait(state).unwrap()
            };
            if let Some(waiting) = waiting {
                state.stats.as_mut().unwrap().sample(
                    if waiting_packet {
                        "packet_wait_ms"
                    } else {
                        "queue_full_wait_ms"
                    },
                    waiting.elapsed().as_secs_f64() * 1000.0,
                );
            }
        }
    }
}
