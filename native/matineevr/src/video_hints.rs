use crate::presentation::{Presentation, Projection, Stereo, StereoFormat};
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Hint<T> {
    #[default]
    Unknown,
    Known(T),
    Conflict,
}

impl<T: Copy + Eq> Hint<T> {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unknown, hint) | (hint, Self::Unknown) => hint,
            (Self::Known(a), Self::Known(b)) if a == b => self,
            _ => Self::Conflict,
        }
    }

    fn apply(self, fallback: T) -> T {
        match self {
            Self::Known(value) => value,
            _ => fallback,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hints {
    pub projection: Hint<Projection>,
    pub stereo: Hint<Stereo>,
    pub sbs_format: Hint<StereoFormat>,
    pub tb_format: Hint<StereoFormat>,
    pub swap_eyes: Hint<bool>,
}

impl Hints {
    pub fn apply(self, fallback: Presentation) -> Presentation {
        Presentation {
            projection: self.projection.apply(fallback.projection),
            stereo: self.stereo.apply(fallback.stereo),
            sbs_format: self.sbs_format.apply(fallback.sbs_format),
            tb_format: self.tb_format.apply(fallback.tb_format),
            swap_eyes: self.swap_eyes.apply(fallback.swap_eyes),
            ..fallback
        }
    }

    pub fn combine(self, other: Self) -> Self {
        Self {
            projection: self.projection.combine(other.projection),
            stereo: self.stereo.combine(other.stereo),
            sbs_format: self.sbs_format.combine(other.sbs_format),
            tb_format: self.tb_format.combine(other.tb_format),
            swap_eyes: self.swap_eyes.combine(other.swap_eyes),
        }
    }

    pub fn filename(path: &Path) -> Self {
        use Hint::{Conflict, Known, Unknown};
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        let mut hints = Self::default();
        let mut previous = "";
        for token in name.split(|c: char| !c.is_alphanumeric()) {
            let projection = match token {
                "180" | "vr180" => Known(Projection::Hemisphere),
                "360" | "vr360" => Known(Projection::Sphere),
                "fisheye" => Known(Projection::Fisheye(180)),
                "fisheye190" => Known(Projection::Fisheye(190)),
                "f180" | "180f" | "mkx200" | "mkx22" | "rf52" | "vrca220" | "eac360" | "360eac" => {
                    Conflict
                }
                _ => Unknown,
            };
            let stereo = match token {
                "sbs" | "halfsbs" | "lr" => Some((Stereo::SideBySide, false)),
                "rl" => Some((Stereo::SideBySide, true)),
                "tb" | "halftb" => Some((Stereo::TopBottom, false)),
                "bt" => Some((Stereo::TopBottom, true)),
                "2d" => Some((Stereo::Mono, false)),
                _ => None,
            };
            hints = hints.combine(Self {
                projection,
                stereo: stereo.map_or(Unknown, |(layout, _)| Known(layout)),
                sbs_format: match (previous, token) {
                    (_, "halfsbs") | ("half", "sbs") => Known(StereoFormat::Half),
                    _ => Unknown,
                },
                tb_format: match (previous, token) {
                    (_, "halftb") | ("half", "tb") => Known(StereoFormat::Half),
                    _ => Unknown,
                },
                swap_eyes: stereo.map_or(Unknown, |(_, swap)| Known(swap)),
            });
            previous = token;
        }
        hints
    }

    pub(crate) fn native(fields: [i32; 3]) -> Self {
        fn field<T: Copy>(value: i32, values: &[T]) -> Hint<T> {
            if value == -1 {
                Hint::Unknown
            } else {
                values
                    .get(value as usize)
                    .map_or(Hint::Conflict, |&v| Hint::Known(v))
            }
        }
        Self {
            projection: field(
                fields[0],
                &[Projection::Flat, Projection::Hemisphere, Projection::Sphere],
            ),
            stereo: field(
                fields[1],
                &[Stereo::Mono, Stereo::SideBySide, Stereo::TopBottom],
            ),
            swap_eyes: field(fields[2], &[false, true]),
            ..Self::default()
        }
    }
}

#[cfg(test)]
#[path = "video_hints_tests.rs"]
mod tests;
