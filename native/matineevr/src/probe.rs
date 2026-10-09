use crate::{media::Decoder, options::Options, snapshot};
use anyhow::Result;
use matineevr::offscreen::Offscreen;
use std::time::Instant;

pub fn run(options: &Options) -> Result<()> {
    let mut decoder = Decoder::open(&options.file)?;
    let mut render = options
        .render_probe
        .then(|| Offscreen::new((1024, 1024), 1))
        .transpose()?;
    let started = Instant::now();
    let mut frames = 0;
    loop {
        if decoder.reconfigure(false)? {
            if let Some(render) = &mut render {
                render.synchronize(None)?;
            }
            decoder.reconfigure(true)?;
        }
        let frame = match decoder.advance()? {
            crate::media::Decoded::Frame(frame) => frame,
            crate::media::Decoded::End => break,
            _ => continue,
        };
        if let Some(render) = &mut render {
            render.renderer.upload(std::rc::Rc::new(frame))?;
            render.draw(options.presentation)?;
            render.finish()?;
        }
        frames += 1;
        if options
            .seconds
            .is_some_and(|limit| started.elapsed().as_secs_f64() >= limit)
        {
            break;
        }
    }
    if let Some(render) = &mut render
        && frames > 0
        && let Some(path) = &options.snapshot
    {
        let pixels = render.pixels()?;
        let pixels: Vec<_> = pixels
            .chunks_exact(1024 * 4)
            .rev()
            .flatten()
            .copied()
            .collect();
        snapshot::save_pixels(path, 1024, 1024, &pixels)?;
    }
    let elapsed = started.elapsed().as_secs_f64();
    eprintln!(
        "Probe: {frames} frames in {elapsed:.3}s ({:.1} fps), render={}",
        frames as f64 / elapsed,
        options.render_probe
    );
    anyhow::ensure!(frames > 0, "No frames decoded");
    Ok(())
}
