use crate::{
    measure::{Sample, measure},
    options::Options,
    source::Source,
};
use anyhow::{Context, Result, ensure};
use matineevr::offscreen::Offscreen;
use std::time::{Duration, Instant};

pub struct Row {
    pub operation: &'static str,
    pub target: Option<f64>,
    pub sample: Sample,
}

pub fn run(options: &Options) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    let setup = Instant::now();
    let mut render = options
        .render
        .map(|size| Offscreen::new(size, 2))
        .transpose()?;
    if render.is_some() {
        rows.push(Row {
            operation: "graphics_init",
            target: None,
            sample: Sample {
                elapsed: setup.elapsed(),
                ..Default::default()
            },
        });
    }
    let started = Instant::now();
    let mut source = Source::open(options)?;
    let opened = started.elapsed();
    let result = scenarios(
        &mut source,
        options,
        started,
        opened,
        &mut render,
        &mut rows,
    );
    if let Some(render) = &mut render {
        render.synchronize(None)?;
    }
    let stopped = source.stop();
    result?;
    stopped?;
    Ok(rows)
}

fn scenarios(
    source: &mut Source,
    options: &Options,
    started: Instant,
    opened: Duration,
    render: &mut Option<Offscreen>,
    rows: &mut Vec<Row>,
) -> Result<()> {
    let first = measure(source, render, started, None, options, false).context("Start playback")?;
    ensure!(first.frames > 0, "Video ended without producing a frame");
    if options.decoder {
        rows.push(Row {
            operation: "open",
            target: None,
            sample: Sample {
                elapsed: opened,
                ..Default::default()
            },
        });
    }
    rows.push(Row {
        operation: "start",
        target: None,
        sample: first,
    });
    play(source, options, render, rows).context("Play after opening")?;
    for &target in &options.targets {
        let started = Instant::now();
        if let Some(render) = render {
            render.synchronize(None)?;
        }
        source
            .seek(target)
            .with_context(|| format!("Seek to {target}s"))?;
        let called = started.elapsed();
        let sample = measure(source, render, started, None, options, true)
            .with_context(|| format!("First frame after seek to {target}s"))?;
        let eof = sample.eof;
        if options.decoder {
            rows.push(Row {
                operation: "seek_call",
                target: Some(target),
                sample: Sample {
                    elapsed: called,
                    ..Default::default()
                },
            });
        }
        rows.push(Row {
            operation: "seek",
            target: Some(target),
            sample,
        });
        if !eof {
            play(source, options, render, rows)
                .with_context(|| format!("Play after seek to {target}s"))?;
        }
    }
    Ok(())
}

fn play(
    source: &mut Source,
    options: &Options,
    render: &mut Option<Offscreen>,
    rows: &mut Vec<Row>,
) -> Result<()> {
    if !options.seconds.is_zero() {
        let sample = measure(
            source,
            render,
            Instant::now(),
            Some(options.seconds),
            options,
            false,
        )?;
        rows.push(Row {
            operation: "play",
            target: None,
            sample,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires Frame and an eight-second FRAME_TEST_VIDEO"]
    fn measures_shared_playback_and_decoder_through_eof_and_back() {
        let file = std::env::var("FRAME_TEST_VIDEO").unwrap();
        for decoder in [false, true] {
            let mut options =
                Options::parse([file.clone(), "1.25".into(), "20".into(), "0".into()].into_iter())
                    .unwrap()
                    .unwrap();
            options.decoder = decoder;
            options.seconds = Duration::from_millis(100);
            if decoder {
                options.poll = Duration::ZERO;
            }
            for _ in 0..2 {
                let rows = run(&options).unwrap();
                let start = rows.iter().find(|r| r.operation == "start").unwrap();
                assert_eq!(start.sample.frames, 1);
                assert_eq!(start.sample.first, Some(0.0));
                if decoder {
                    let open = rows.iter().find(|r| r.operation == "open").unwrap();
                    assert!(start.sample.elapsed >= open.sample.elapsed);
                }
                for row in &rows {
                    assert!(row.sample.import.is_zero() && row.sample.draw.is_zero());
                }
                let seeks: Vec<_> = rows.iter().filter(|r| r.operation == "seek").collect();
                assert_eq!(seeks.len(), 3);
                assert!((1.25..1.35).contains(&seeks[0].sample.first.unwrap()));
                assert!(seeks[1].sample.eof);
                assert_eq!(seeks[1].sample.frames, 0);
                assert!(seeks[1].sample.first.is_none());
                assert_eq!(seeks[2].sample.first, Some(0.0));
                assert!(rows.iter().all(|r| !r.sample.elapsed.is_zero()));
                assert!(
                    rows.iter()
                        .filter(|r| r.operation == "play")
                        .all(|r| r.sample.frames > 0)
                );
            }
        }
    }
}
