use super::{Projection, Stereo, StereoFormat};
use anyhow::{Error, bail};
use std::str::FromStr;

impl FromStr for Projection {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "flat" => Self::Flat,
            "180" => Self::Hemisphere,
            "360" => Self::Sphere,
            "fisheye" => Self::Fisheye(180),
            "fisheye190" => Self::Fisheye(190),
            _ => bail!("--projection expects flat, 180, 360, fisheye or fisheye190"),
        })
    }
}

impl FromStr for Stereo {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "mono" => Self::Mono,
            "sbs" => Self::SideBySide,
            "tb" => Self::TopBottom,
            _ => bail!("--stereo expects mono, sbs or tb"),
        })
    }
}

impl FromStr for StereoFormat {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "full" => Self::Full,
            "half" => Self::Half,
            _ => bail!("Stereo format expects full or half"),
        })
    }
}
