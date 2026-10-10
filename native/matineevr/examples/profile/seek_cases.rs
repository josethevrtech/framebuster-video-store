use crate::device_case::{Case, options};
use anyhow::{Result, ensure};
use matineevr::offscreen::Offscreen;
use std::{
    thread,
    time::{Duration, Instant},
};

#[test]
#[ignore = "requires Frame and FRAME_TEST_VIDEO/FRAME_TEST_ARGS"]
fn audio_reset() -> Result<()> {
    let mut render = Some(Offscreen::new((64, 64), 2)?);
    Case::run(options()?, &mut render, |case| {
        case.measure("start", None, false)?;
        for run in 1..=case.options.repeat {
            case.request.run = run;
            for paused in [false, true] {
                case.request.scenario = if paused {
                    "audio_paused"
                } else {
                    "audio_playing"
                };
                if paused {
                    case.player().toggle_pause();
                }
                for target in case.options.targets.clone() {
                    if !paused {
                        case.play(false)?;
                    }
                    let sample = case.seek(&[target])?;
                    ensure!(
                        sample.audio_clock.is_some(),
                        "Audio case requires an audio track"
                    );
                    if paused {
                        ensure!(
                            case.player().status() == "Paused",
                            "Seek resumed paused playback"
                        );
                        thread::sleep(Duration::from_millis(50));
                        ensure!(
                            case.player().audio_clock() == Some(target),
                            "Paused clock moved"
                        );
                        ensure!(
                            case.player().due_frame()?.is_none(),
                            "Paused playback delivered another frame"
                        );
                    }
                }
                let targets = case.options.targets.clone();
                case.request.scenario = if paused {
                    "audio_paused_burst"
                } else {
                    "audio_playing_burst"
                };
                case.seek(&targets)?;
                let target = *targets.last().unwrap();
                case.request.started = Instant::now();
                case.request.operation = "resume";
                if paused {
                    case.player().toggle_pause();
                }
                let resumed = case.request.bounded(Duration::from_secs(30), || {
                    loop {
                        case.source.next(case.render)?;
                        let clock = case.player().audio_clock();
                        if clock.is_some_and(|clock| clock > target) {
                            return Ok(crate::measure::Sample {
                                elapsed: case.request.started.elapsed(),
                                audio_clock: clock,
                                ..Default::default()
                            });
                        }
                        thread::sleep(case.options.poll);
                    }
                })?;
                case.request.write("ok", Some(&resumed))?;
            }
        }
        Ok(())
    })
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_VIDEO/FRAME_TEST_ARGS"]
fn capture_pressure() -> Result<()> {
    let mut render = Some(Offscreen::new((1440, 1440), 2)?);
    Case::run(options()?, &mut render, |case| {
        case.request.scenario = "capture_pressure";
        case.measure("start", None, false)?;
        for run in 1..=case.options.repeat {
            case.request.run = run;
            for target in case.options.targets.clone() {
                case.play(true)?;
                let stats = case
                    .render
                    .as_mut()
                    .unwrap()
                    .renderer
                    .stats
                    .as_mut()
                    .unwrap()
                    .report("pressure", case.options.seconds.as_secs_f64());
                ensure!(
                    stats.contains("held_submissions_max=3.000"),
                    "Submission slots were not filled: {stats}"
                );
                eprintln!("{stats}");
                case.seek(&[target])?;
            }
            case.play(true)?;
            case.seek(&case.options.targets.clone())?;
        }
        Ok(())
    })
}
