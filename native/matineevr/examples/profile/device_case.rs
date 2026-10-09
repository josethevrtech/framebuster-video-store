use crate::{
    measure::{Sample, measure},
    options::Options,
    source::Source,
    thumbnail_csv::{HEADER, Request},
};
use anyhow::{Result, ensure};
use matineevr::{media::Decoded, offscreen::Offscreen, playback::Playback};
use std::{
    collections::HashSet,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

pub fn options() -> Result<Options> {
    let file = std::env::var("FRAME_TEST_VIDEO")?;
    let args = std::env::var("FRAME_TEST_ARGS")?;
    let options =
        Options::parse(std::iter::once(file).chain(args.split_whitespace().map(str::to_owned)))?
            .unwrap();
    ensure!(
        options.thumbnails && options.targets.len() >= 2 && !options.seconds.is_zero(),
        "Cases require --thumbnails, two targets and a positive playback interval"
    );
    Ok(options)
}

pub struct Case<'a> {
    pub options: Options,
    pub render: &'a mut Option<Offscreen>,
    pub source: Source,
    pub request: Request,
    visited: HashSet<u64>,
}

impl<'a> Case<'a> {
    pub fn run(
        options: Options,
        render: &'a mut Option<Offscreen>,
        test: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<()> {
        let source = Source::open(&options)?;
        let mut case = Self {
            options,
            render,
            source,
            visited: HashSet::new(),
            request: Request {
                scenario: "start",
                run: 1,
                index: 0,
                operation: "start",
                target: None,
                repeated: false,
                started: Instant::now(),
            },
        };
        case.player().profile();
        let render = case.render.as_mut().unwrap();
        for renderer in [&mut render.renderer, &mut render.preview_renderer] {
            renderer.stats.get_or_insert_default();
        }
        println!("{HEADER}");
        let result = test(&mut case);
        let released = case.render.as_mut().unwrap().synchronize(None);
        let stopped = case.source.stop();
        result?;
        released?;
        stopped
    }

    pub fn player(&mut self) -> &mut Playback {
        let Source::Playback(player) = &mut self.source else {
            unreachable!()
        };
        player
    }

    pub fn measure(
        &mut self,
        operation: &'static str,
        window: Option<Duration>,
        seeking: bool,
    ) -> Result<Sample> {
        self.request.operation = operation;
        let sample =
            self.request
                .bounded(Duration::from_secs(30) + window.unwrap_or_default(), || {
                    measure(
                        &mut self.source,
                        self.render,
                        self.request.started,
                        window,
                        &self.options,
                        seeking,
                    )
                })?;
        self.request
            .write(if sample.eof { "eof" } else { "ok" }, Some(&sample))?;
        ensure!(!sample.eof, "Unexpected EOF during {operation}");
        Ok(sample)
    }

    pub fn seek(&mut self, targets: &[f64]) -> Result<Sample> {
        let target = *targets.last().unwrap();
        self.request.index += 1;
        self.request.target = Some(target);
        self.request.repeated = !self.visited.insert(target.to_bits());
        self.request.started = Instant::now();
        for &target in targets {
            self.source.seek(target)?;
        }
        let preview = self.measure("seek", None, true)?;
        ensure!(preview.preview_ready.is_some(), "No completed preview");
        let target_frame = if preview.frames > 0 {
            self.request.operation = "seek_target";
            self.request.write("ok", Some(&preview))?;
            preview
        } else {
            self.measure("seek_target", None, false)?
        };
        ensure!(
            target_frame
                .first
                .is_some_and(|pts| pts >= target && pts < target + 0.1),
            "Wrong target frame: {:?} for {target}",
            target_frame.first
        );
        self.report();
        Ok(target_frame)
    }

    pub fn play(&mut self, asynchronous: bool) -> Result<Sample> {
        self.request.started = Instant::now();
        if !asynchronous {
            return self.measure("play", Some(self.options.seconds), false);
        }
        self.request.operation = "play";
        let sample =
            self.request
                .bounded(Duration::from_secs(30) + self.options.seconds, || {
                    let mut sample = Sample::default();
                    let skipped = self.source.skipped();
                    while self.request.started.elapsed() < self.options.seconds {
                        match self.source.next(self.render)? {
                            Decoded::Frame(frame) => {
                                sample.first.get_or_insert(frame.pixels.pts);
                                sample.last = Some(frame.pixels.pts);
                                sample.frames += 1;
                                let render = self.render.as_mut().unwrap();
                                render.renderer.upload(Rc::new(frame))?;
                                render.draw(self.options.presentation)?;
                            }
                            Decoded::Preview(_) | Decoded::End => {
                                anyhow::bail!("Unexpected preview or EOF during playback")
                            }
                            Decoded::Pending => thread::sleep(self.options.poll),
                        }
                    }
                    sample.elapsed = self.request.started.elapsed();
                    sample.skipped = self.source.skipped() - skipped;
                    sample.audio_clock = self.player().audio_clock();
                    ensure!(
                        sample.frames >= 3,
                        "Insufficient frames to exercise asynchronous rendering"
                    );
                    Ok(sample)
                })?;
        self.request.write("ok", Some(&sample))?;
        Ok(sample)
    }

    pub fn report(&mut self) {
        let render = self.render.as_mut().unwrap();
        for (name, renderer) in [
            ("video", &mut render.renderer),
            ("preview", &mut render.preview_renderer),
        ] {
            eprintln!(
                "{} request={} scenario={}",
                renderer
                    .stats
                    .as_mut()
                    .unwrap()
                    .report(name, self.request.started.elapsed().as_secs_f64()),
                self.request.index,
                self.request.scenario
            );
        }
    }
}
