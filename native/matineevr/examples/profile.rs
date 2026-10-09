#[path = "profile/cpu.rs"]
mod cpu;
#[cfg(test)]
#[path = "profile/device_case.rs"]
mod device_case;
#[path = "profile/measure.rs"]
mod measure;
#[path = "profile/options.rs"]
mod options;
#[cfg(test)]
#[path = "profile/pipeline_case.rs"]
mod pipeline_case;
#[cfg(test)]
#[path = "profile/render_tests.rs"]
mod render_tests;
#[path = "profile/scenarios.rs"]
mod scenarios;
#[cfg(test)]
#[path = "profile/seek_cases.rs"]
mod seek_cases;
#[path = "profile/source.rs"]
mod source;
#[path = "profile/thumbnail_csv.rs"]
mod thumbnail_csv;
#[cfg(test)]
#[path = "profile/thumbnail_tests.rs"]
mod thumbnail_tests;
#[path = "profile/thumbnails.rs"]
mod thumbnails;

use anyhow::Result;
use options::Options;
use std::io::{self, Write};

fn main() -> Result<()> {
    let Some(options) = Options::parse(std::env::args().skip(1))? else {
        return Ok(());
    };
    eprintln!(
        "MatineeVR {}; file={}; seconds={}; repeat={}; cache=uncontrolled",
        env!("MATINEEVR_VERSION"),
        options.file.display(),
        options.seconds.as_secs_f64(),
        options.repeat
    );
    if options.thumbnails {
        return thumbnails::run(&options);
    }
    let mut output = io::BufWriter::new(io::stdout().lock());
    writeln!(
        output,
        "run,mode,operation,target_s,elapsed_ms,frames,first_pts_s,last_pts_s,skipped,eof,poll_ms,import_ms,draw_ms,gpu_ms,eye_width,eye_height,preview_pts_s,preview_ms,preview_ready_ms,cpu_percent"
    )?;
    for run in 1..=options.repeat {
        for row in scenarios::run(&options)? {
            let sample = row.sample;
            let number = |value: Option<f64>| value.map(|v| format!("{v:.6}")).unwrap_or_default();
            let gpu_ms = |duration: std::time::Duration| {
                number(options.render.map(|_| duration.as_secs_f64() * 1000.0))
            };
            let size = options.render.unwrap_or_default();
            writeln!(
                output,
                "{run},{},{},{},{:.6},{},{},{},{},{},{:.3},{},{},{},{},{},{},{},{},{}",
                if options.decoder {
                    "decoder"
                } else {
                    "playback"
                },
                row.operation,
                number(row.target),
                sample.elapsed.as_secs_f64() * 1000.0,
                sample.frames,
                number(sample.first),
                number(sample.last),
                sample.skipped,
                sample.eof,
                options.poll.as_secs_f64() * 1000.0,
                gpu_ms(sample.import),
                gpu_ms(sample.draw),
                gpu_ms(sample.gpu),
                size.0,
                size.1,
                number(sample.preview_pts),
                number(sample.preview.map(|time| time.as_secs_f64() * 1000.0)),
                number(sample.preview_ready.map(|time| time.as_secs_f64() * 1000.0)),
                number(sample.cpu_percent)
            )?;
        }
        output.flush()?;
    }
    Ok(())
}
