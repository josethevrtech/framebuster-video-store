use anyhow::{Context, Result, bail, ensure};
use matineevr::presentation::Presentation;
use std::{path::PathBuf, time::Duration};

pub struct Options {
    pub file: PathBuf,
    pub targets: Vec<f64>,
    pub seconds: Duration,
    pub repeat: u32,
    pub poll: Duration,
    pub decoder: bool,
    pub render: Option<(i32, i32)>,
    pub thumbnails: bool,
    pub stats: bool,
    pub presentation: Presentation,
}

impl Options {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut file = None;
        let mut options = Self {
            file: PathBuf::new(),
            targets: Vec::new(),
            seconds: Duration::from_secs(1),
            repeat: 1,
            poll: Duration::from_millis(1),
            decoder: false,
            render: None,
            thumbnails: false,
            stats: false,
            presentation: Presentation::default(),
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => {
                    println!(
                        "Usage: profile FILE [SEEK_SECONDS ...] [--seconds N] [--repeat N] [--poll-ms N] [--decoder] [--render WIDTH HEIGHT]\n\
                        Defaults: playback, 1s after each first frame, 1 run, 1ms polling.\n\
                        --decoder measures synchronous decoding without pacing or polling.\n\
                        --render adds headless DMA-BUF import and two-eye draw timings through GPU completion.\n\
                        --thumbnails measures seek stages and 256x144 previews; requires seek positions.\n\
                        --stats reports shared playback pacing counters to stderr every five seconds.\n\
                        --projection flat|180|360|fisheye|fisheye190, --stereo mono|sbs|tb, --sbs-format full|half and --tb-format full|half select rendering.\n\
                        CSV on stdout; diagnostics on stderr. Use timeout to bound native calls."
                    );
                    return Ok(None);
                }
                "--decoder" => options.decoder = true,
                "--thumbnails" => options.thumbnails = true,
                "--stats" => options.stats = true,
                "--projection" => {
                    options.presentation.projection =
                        args.next().context("Missing projection")?.parse()?;
                }
                "--stereo" => {
                    options.presentation.stereo =
                        args.next().context("Missing stereo layout")?.parse()?;
                }
                "--sbs-format" | "--tb-format" => {
                    let format = args
                        .next()
                        .with_context(|| format!("Missing value for {arg}"))?
                        .parse()?;
                    if arg == "--sbs-format" {
                        options.presentation.sbs_format = format;
                    } else {
                        options.presentation.tb_format = format;
                    }
                }
                "--render" => {
                    let mut size = [0; 2];
                    for value in &mut size {
                        *value = args
                            .next()
                            .context("--render requires width and height")?
                            .parse()?;
                        ensure!(*value > 0, "Render dimensions must be positive");
                    }
                    options.render = Some((size[0], size[1]));
                }
                "--seconds" | "--repeat" | "--poll-ms" => {
                    let value = args
                        .next()
                        .with_context(|| format!("Missing value for {arg}"))?;
                    match arg.as_str() {
                        "--repeat" => {
                            options.repeat = value.parse().context("Invalid repeat count")?;
                            ensure!(options.repeat > 0, "Repeat count must be positive");
                        }
                        "--seconds" => options.seconds = duration(&value, 1.0)?,
                        _ => {
                            options.poll = duration(&value, 0.001)?;
                            ensure!(!options.poll.is_zero(), "Polling interval must be positive");
                        }
                    }
                }
                _ if arg.starts_with('-') => bail!("Unknown option: {arg}"),
                _ if file.is_none() => file = Some(PathBuf::from(arg)),
                _ => {
                    let target: f64 = arg.parse().context("Invalid seek position")?;
                    ensure!(target.is_finite() && target >= 0.0, "Invalid seek position");
                    options.targets.push(target);
                }
            }
        }
        options.file = file.context("Usage: profile FILE [SEEK_SECONDS ...] [--help]")?;
        if options.thumbnails {
            ensure!(
                !options.decoder && options.render.is_none(),
                "--thumbnails uses playback and its own preview target"
            );
            ensure!(
                !options.targets.is_empty(),
                "--thumbnails requires seek positions"
            );
        }
        if options.decoder {
            ensure!(
                !options.stats,
                "--stats measures paced playback; omit --decoder"
            );
            options.poll = Duration::ZERO;
        }
        Ok(Some(options))
    }
}

fn duration(value: &str, scale: f64) -> Result<Duration> {
    let seconds = value.parse::<f64>().context("Invalid duration")? * scale;
    Duration::try_from_secs_f64(seconds).context("Duration must be finite and nonnegative")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &str) -> Result<Option<Options>> {
        Options::parse(args.split_whitespace().map(str::to_owned))
    }

    #[test]
    fn preserves_seek_order_and_validates_before_opening() {
        let options = parse("video.mp4 30.01 0 30.01 --seconds 0 --repeat 2 --poll-ms 0.5")
            .unwrap()
            .unwrap();
        assert_eq!(options.targets, [30.01, 0.0, 30.01]);
        assert_eq!(options.repeat, 2);
        assert!(options.seconds.is_zero());
        assert_eq!(options.poll, Duration::from_micros(500));
        assert!(parse("video --stats").unwrap().unwrap().stats);
        assert_eq!(
            parse("video --render 1440 1200").unwrap().unwrap().render,
            Some((1440, 1200))
        );
        assert!(
            parse("video.mp4 --decoder")
                .unwrap()
                .unwrap()
                .poll
                .is_zero()
        );
        for args in [
            "",
            "video NaN",
            "video inf",
            "video -1",
            "video --repeat 0",
            "video --seconds -1",
            "video --seconds NaN",
            "video --poll-ms 0",
            "video --repeat",
            "video --render 0 1440",
            "video --render 1440",
            "video --decoder --stats",
        ] {
            assert!(parse(args).is_err(), "{args}");
        }
    }
}
