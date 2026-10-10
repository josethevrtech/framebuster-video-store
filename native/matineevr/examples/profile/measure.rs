use crate::{options::Options, source::Source};
use anyhow::{Result, ensure};
use matineevr::{
    media::{Decoded, Frame},
    media_trace::SeekTrace,
    offscreen::Offscreen,
    presentation::Presentation,
};
use std::{
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Sample {
    pub elapsed: Duration,
    pub cpu_percent: Option<f64>,
    pub frames: u64,
    pub first: Option<f64>,
    pub last: Option<f64>,
    pub skipped: u64,
    pub eof: bool,
    pub import: Duration,
    pub gpu: Duration,
    pub draw: Duration,
    pub preview_pts: Option<f64>,
    pub preview: Option<Duration>,
    pub preview_ready: Option<Duration>,
    pub preview_import: f64,
    pub preview_draw: Duration,
    pub preview_gpu: f64,
    pub preview_submit: f64,
    pub preview_wait: f64,
    pub trace: Option<SeekTrace>,
    pub consumed: Option<Instant>,
    pub audio_clock: Option<f64>,
}

impl Sample {
    fn record_preview(
        &mut self,
        frame: Rc<Frame>,
        render: &mut Option<Offscreen>,
        started: Instant,
        presentation: Presentation,
    ) -> Result<()> {
        self.preview_pts = Some(frame.pixels.pts);
        self.preview = Some(self.elapsed);
        if let Some(render) = render {
            let draw = Instant::now();
            render.preview_renderer.upload(frame)?;
            let timing = render.draw_preview(presentation)?;
            self.preview_import = render.preview_renderer.import_ms;
            self.preview_draw = draw.elapsed();
            self.preview_gpu = timing.gpu_ms;
            self.preview_submit = timing.submit_ms;
            self.preview_wait = timing.wait_ms;
            self.preview_ready = Some(timing.ready.duration_since(started));
        }
        self.elapsed = started.elapsed();
        Ok(())
    }
}

pub fn measure(
    source: &mut Source,
    render: &mut Option<Offscreen>,
    started: Instant,
    window: Option<Duration>,
    options: &Options,
    seeking: bool,
) -> Result<Sample> {
    let cpu = window.map(|_| crate::cpu::cpu_seconds()).transpose()?;
    let skipped = source.skipped();
    let mut sample = Sample::default();
    loop {
        let next = source.next(render)?;
        if options.thumbnails && sample.trace.is_none() && !matches!(&next, Decoded::Pending) {
            sample.consumed = Some(Instant::now());
            sample.trace = source.take_trace();
        }
        sample.elapsed = started.elapsed();
        match next {
            Decoded::Preview(frame) => {
                sample.record_preview(Rc::new(frame), render, started, options.presentation)?;
            }
            Decoded::Frame(frame) => {
                let frame = Rc::new(frame);
                if seeking && sample.preview.is_none() {
                    sample.record_preview(frame.clone(), render, started, options.presentation)?;
                }
                let pts = frame.pixels.pts;
                ensure!(pts.is_finite(), "Nonfinite frame timestamp");
                sample.first.get_or_insert(pts);
                sample.last = Some(pts);
                sample.frames += 1;
                if let Some(render) = render.as_mut().filter(|_| !options.thumbnails) {
                    let draw = Instant::now();
                    render.renderer.upload(frame)?;
                    render.draw(options.presentation)?;
                    let completed = render.finish()?;
                    sample.draw += draw.elapsed();
                    sample.import += Duration::from_secs_f64(render.renderer.import_ms / 1000.0);
                    sample.gpu += Duration::from_secs_f64(
                        completed.iter().map(|(_, ms)| ms[1]).sum::<f64>() / 1000.0,
                    );
                    sample.elapsed = started.elapsed();
                }
            }
            Decoded::End => sample.eof = true,
            Decoded::Pending => {}
        }
        if sample.eof
            || window.map_or(
                sample.frames > 0 || (options.thumbnails && sample.preview.is_some()),
                |limit| sample.elapsed >= limit,
            )
        {
            if let Source::Playback(player) = source {
                sample.audio_clock = player.audio_clock();
            }
            sample.skipped = source.skipped() - skipped;
            sample.cpu_percent = cpu
                .map(|cpu| {
                    Ok::<_, anyhow::Error>(
                        100.0 * (crate::cpu::cpu_seconds()? - cpu) / sample.elapsed.as_secs_f64(),
                    )
                })
                .transpose()?;
            return Ok(sample);
        }
        if window.is_none() && sample.elapsed >= Duration::from_secs(30) {
            return Err(
                std::io::Error::new(std::io::ErrorKind::TimedOut, "First frame timed out").into(),
            );
        }
        if !options.poll.is_zero() {
            thread::sleep(options.poll);
        }
    }
}
