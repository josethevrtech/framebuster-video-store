use crate::presentation::Presentation;
use crate::video_settings::{Hint, Hints};
use anyhow::{Context, Result, bail};
use std::path::PathBuf;

pub struct Options {
    pub file: PathBuf,
    pub presentation: Presentation,
    pub presentation_overrides: Hints,
    pub probe: bool,
    pub render_probe: bool,
    pub seconds: Option<f64>,
    pub snapshot: Option<PathBuf>,
    pub hud: bool,
    pub hud_snapshot: Option<PathBuf>,
    pub stats: bool,
}

impl Options {
    pub fn parse() -> Result<Option<Self>> {
        Self::from_args(std::env::args().skip(1))
    }

    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Option<Self>> {
        let mut result = Self {
            file: PathBuf::new(),
            presentation: Presentation::default(),
            presentation_overrides: Hints::default(),
            probe: false,
            render_probe: false,
            seconds: None,
            snapshot: None,
            hud: false,
            hud_snapshot: None,
            stats: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--version" | "-V" => {
                    println!("FrameBuster Video Store {}", env!("MATINEEVR_VERSION"));
                    return Ok(None);
                }
                "--help" | "-h" => {
                    println!(
                        "framebuster-video-store [FILE|DIRECTORY] [--projection flat|180|360|fisheye|fisheye190] [--stereo mono|sbs|tb]\n\
                        [--sbs-format full|half] [--tb-format full|half] [--probe | --render-probe] [--seconds N] [--snapshot FILE.ppm]\n\
                        [--hud] [--hud-snapshot FILE.ppm] [--stats] [--version]\n\
                        No file: browse ~/Videos. D-pad: navigate/open. A: open. B: parent folder.\n\
                        Playback: X: pause. Y: HUD. A: recenter. B: stop and browse.\n\
                        Hold left grip: browser. Hold right grip: video settings.\n\
                        Settings: stick up/down selects; left/right changes.\n\
                        FrameBuster Video Store: native Steam Frame OpenXR video and audio."
                    );
                    return Ok(None);
                }
                "--projection" => {
                    result.presentation_overrides.projection =
                        Hint::Known(args.next().context("Missing projection")?.parse()?);
                }
                "--stereo" => {
                    result.presentation_overrides.stereo =
                        Hint::Known(args.next().context("Missing stereo layout")?.parse()?);
                }
                "--sbs-format" | "--tb-format" => {
                    let format = Hint::Known(
                        args.next()
                            .with_context(|| format!("Missing value for {arg}"))?
                            .parse()?,
                    );
                    if arg == "--sbs-format" {
                        result.presentation_overrides.sbs_format = format;
                    } else {
                        result.presentation_overrides.tb_format = format;
                    }
                }
                "--probe" => result.probe = true,
                "--render-probe" => result.render_probe = true,
                "--hud" => result.hud = true,
                "--stats" => result.stats = true,
                "--seconds" => {
                    let value: f64 = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Missing seconds"))?
                        .parse()?;
                    if !value.is_finite() || value <= 0.0 {
                        bail!("Seconds must be positive");
                    }
                    result.seconds = Some(value);
                }
                "--snapshot" | "--hud-snapshot" => {
                    let path = args
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Missing snapshot path"))?;
                    if arg == "--snapshot" {
                        result.snapshot = Some(path.into());
                    } else {
                        result.hud_snapshot = Some(path.into());
                    }
                }
                _ if arg.starts_with('-') => bail!("Unknown option: {arg}"),
                _ if result.file.as_os_str().is_empty() => result.file = arg.into(),
                _ => bail!("Only one file or directory is accepted"),
            }
        }
        if result.file.as_os_str().is_empty() {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or(std::env::current_dir()?);
            result.file = if home.join("Videos").is_dir() {
                home.join("Videos")
            } else {
                home
            };
        }
        let stream = crate::frame_playback::is_stream(&result.file);
        if !stream && !result.file.is_file() && !result.file.is_dir() {
            bail!("Supply an existing local file or directory; see --help");
        }
        if (result.probe || result.render_probe) && !result.file.is_file() && !stream {
            bail!("Probe modes require a video file");
        }
        if result.probe && result.render_probe {
            bail!("Choose one probe mode");
        }
        if result.hud_snapshot.is_some()
            && (result.probe || result.render_probe || !result.hud && result.file.is_file())
        {
            bail!("--hud-snapshot requires the browser or an enabled playback HUD");
        }
        if !stream { result.file = result.file.canonicalize()?; }
        result.presentation = result.presentation_overrides.apply(result.presentation);
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::{Projection, Stereo, StereoFormat};

    #[test]
    fn explicit_defaults_remain_overrides_and_missing_options_remain_unknown() {
        let parse = |args: &[&str]| {
            Options::from_args(args.iter().map(|arg| arg.to_string()))
                .unwrap()
                .unwrap()
        };
        assert_eq!(parse(&["."]).presentation_overrides, Hints::default());
        let options = parse(&[
            ".",
            "--projection",
            "180",
            "--stereo",
            "sbs",
            "--sbs-format",
            "full",
            "--tb-format",
            "full",
        ]);
        assert_eq!(options.presentation, Presentation::default());
        assert_eq!(
            options.presentation_overrides,
            Hints {
                projection: Hint::Known(Projection::Hemisphere),
                stereo: Hint::Known(Stereo::SideBySide),
                sbs_format: Hint::Known(StereoFormat::Full),
                tb_format: Hint::Known(StereoFormat::Full),
                swap_eyes: Hint::Unknown
            }
        );
        assert_eq!(
            parse(&[".", "--stereo", "mono"]).presentation.stereo,
            Stereo::Mono
        );
        for (value, degrees) in [("fisheye", 180), ("fisheye190", 190)] {
            assert_eq!(
                parse(&[".", "--projection", value]).presentation.projection,
                Projection::Fisheye(degrees)
            );
        }
        assert_eq!(
            parse(&[".", "--sbs-format", "half"])
                .presentation
                .sbs_format,
            StereoFormat::Half
        );
        let p = parse(&[".", "--tb-format", "half"]).presentation;
        assert_eq!(p.tb_format, StereoFormat::Half);
        assert_eq!(p.sbs_format, StereoFormat::Full);
        for args in [
            vec![".", "--sbs-format"],
            vec![".", "--sbs-format", "invalid"],
            vec![".", "--tb-format"],
            vec![".", "--tb-format", "invalid"],
        ] {
            assert!(Options::from_args(args.into_iter().map(str::to_owned)).is_err());
        }
    }
}
