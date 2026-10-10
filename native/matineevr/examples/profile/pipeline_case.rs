use crate::device_case::{Case, options};
use anyhow::{Result, ensure};
use matineevr::{media::Decoded, offscreen::Offscreen};
use std::{
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

#[test]
#[ignore = "requires Frame and FRAME_TEST_VIDEO/FRAME_TEST_ARGS/FRAME_TEST_VARIANT"]
fn pipeline_lifetime() -> Result<()> {
    let mut render = Some(Offscreen::new((64, 64), 2)?);
    let options = options()?;
    let original = options.file;
    let variant = std::env::var("FRAME_TEST_VARIANT")?;
    let mut previous = None;
    for (scenario, file) in [
        ("pipeline_original", original.clone()),
        ("pipeline_variant", variant.into()),
        ("pipeline_return", original),
    ] {
        let mut options = super::device_case::options()?;
        options.file = file;
        Case::run(options, &mut render, |case| {
            case.request.scenario = scenario;
            case.measure("start", None, false)?;
            let conversion = compare_pixels(case)?;
            ensure!(
                previous != Some(conversion),
                "Variant must change format or colour conversion"
            );
            previous = Some(conversion);
            for run in 1..=case.options.repeat {
                case.request.run = run;
                for target in case.options.targets.clone() {
                    case.play(false)?;
                    case.seek(&[target])?;
                }
            }
            case.seek(&[0.0])?;
            ensure!(
                compare_pixels(case)? == conversion,
                "Unexpected conversion change within file"
            );
            case.report();
            Ok(())
        })?;
    }
    Ok(())
}

fn compare_pixels(case: &mut Case<'_>) -> Result<[i64; 4]> {
    let request = crate::thumbnail_csv::Request {
        started: Instant::now(),
        operation: "verify_pixels",
        ..case.request
    };
    request.bounded(Duration::from_secs(30), || {
        loop {
            match case.source.next(case.render)? {
                Decoded::Frame(frame) => {
                    let frame = Rc::new(frame);
                    let conversion = [
                        i64::from(frame.dmabuf().format),
                        i64::from(frame.pixels.colorspace),
                        i64::from(frame.pixels.full_range),
                        i64::from(frame.dmabuf().chroma_location),
                    ];
                    let render = case.render.as_mut().unwrap();
                    render.renderer.upload(frame.clone())?;
                    render.draw(case.options.presentation)?;
                    let pixels = render.pixels()?;
                    let mut fresh = Offscreen::new((64, 64), 2)?;
                    fresh.renderer.upload(frame.clone())?;
                    fresh.draw(case.options.presentation)?;
                    ensure!(
                        pixels == fresh.pixels()?,
                        "Persistent renderer differs from fresh renderer"
                    );
                    render.preview_renderer.upload(frame.clone())?;
                    render.draw_preview(case.options.presentation)?;
                    let pixels = render.pixels()?;
                    fresh.preview_renderer.upload(frame)?;
                    fresh.draw_preview(case.options.presentation)?;
                    ensure!(
                        pixels == fresh.pixels()?,
                        "Persistent thumbnail differs from fresh renderer"
                    );
                    return Ok(conversion);
                }
                Decoded::End => anyhow::bail!("EOF during pixel comparison"),
                _ => thread::sleep(case.options.poll),
            }
        }
    })
}
