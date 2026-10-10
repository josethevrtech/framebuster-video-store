use crate::{
    measure::measure,
    options::Options,
    source::Source,
    thumbnail_csv::{HEADER, Request},
};
use anyhow::Result;
use matineevr::{offscreen::Offscreen, preview};
use std::{
    collections::HashSet,
    io::{self, Write},
    time::{Duration, Instant},
};

pub fn run(options: &Options) -> Result<()> {
    println!("{HEADER}");
    io::stdout().flush()?;
    let limit = Duration::from_secs(30);
    let mut request = Request {
        scenario: "thumbnail",
        run: 0,
        index: 0,
        operation: "graphics_init",
        target: None,
        repeated: false,
        started: Instant::now(),
    };
    let size = (preview::SIZE[0] as i32, preview::SIZE[1] as i32);
    let mut render = Some(request.bounded(limit, || Offscreen::new(size, 1))?);
    let mut visited = HashSet::new();
    for run in 1..=options.repeat {
        request = Request {
            scenario: "thumbnail",
            run,
            index: 0,
            target: None,
            repeated: false,
            operation: "start",
            started: Instant::now(),
        };
        let mut source = Source::open(options)?;
        if let Source::Playback(player) = &source {
            player.profile();
        }
        request.bounded(limit, || {
            measure(
                &mut source,
                &mut render,
                request.started,
                None,
                options,
                false,
            )
        })?;
        for index in 0..=options.targets.len() {
            if !options.seconds.is_zero() {
                request.operation = "play";
                request.started = Instant::now();
                request.bounded(limit + options.seconds, || {
                    measure(
                        &mut source,
                        &mut render,
                        request.started,
                        Some(options.seconds),
                        options,
                        false,
                    )
                })?;
            }
            let Some(&target) = options.targets.get(index) else {
                break;
            };
            request = Request {
                scenario: "thumbnail",
                run,
                index: index + 1,
                operation: "seek",
                target: Some(target),
                repeated: !visited.insert(target.to_bits()),
                started: Instant::now(),
            };
            let sample = request.bounded(limit, || {
                source.seek(target)?;
                measure(
                    &mut source,
                    &mut render,
                    request.started,
                    None,
                    options,
                    true,
                )
            })?;
            request.write(if sample.eof { "eof" } else { "ok" }, Some(&sample))?;
            if !options.seconds.is_zero() && !sample.eof {
                request.operation = "seek_target";
                let target = if sample.frames > 0 {
                    sample
                } else {
                    request.bounded(limit, || {
                        measure(
                            &mut source,
                            &mut render,
                            request.started,
                            None,
                            options,
                            false,
                        )
                    })?
                };
                request.write(if target.eof { "eof" } else { "ok" }, Some(&target))?;
            }
        }
        request.operation = "stop";
        request.started = Instant::now();
        request.bounded(limit, || {
            render.as_mut().unwrap().synchronize(None)?;
            source.stop()
        })?;
    }
    Ok(())
}
