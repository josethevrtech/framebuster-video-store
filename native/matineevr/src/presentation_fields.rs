use super::{Presentation, Projection, Stereo, StereoFormat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Projection,
    Format,
    Stereo,
    StereoFormat,
    EyeOrder,
    Center(usize, usize),
    Distortion(usize),
    Reset,
}

impl Field {
    pub fn display(self, p: Presentation) -> (&'static str, String) {
        let (label, value) = match self {
            Self::Projection => (
                "Projection",
                match p.projection {
                    Projection::Flat => "Flat",
                    Projection::Fisheye(_) => "Fisheye",
                    _ => "Equirectangular",
                },
            ),
            Self::Format => (
                "Format",
                if p.projection == Projection::Hemisphere {
                    "180"
                } else if let Projection::Fisheye(fov) = p.projection {
                    return ("Fisheye FOV", format!("{fov} degrees"));
                } else {
                    "360"
                },
            ),
            Self::Stereo => (
                "Stereo",
                match p.stereo {
                    Stereo::Mono => "Mono",
                    Stereo::SideBySide => "Side by side",
                    Stereo::TopBottom => "Top / bottom",
                },
            ),
            Self::StereoFormat => (
                if p.stereo == Stereo::TopBottom {
                    "Top/bottom format"
                } else {
                    "SBS format"
                },
                match if p.stereo == Stereo::TopBottom {
                    p.tb_format
                } else {
                    p.sbs_format
                } {
                    StereoFormat::Full => "Full",
                    StereoFormat::Half => "Half",
                },
            ),
            Self::EyeOrder => ("Eye order", if p.swap_eyes { "Reversed" } else { "Normal" }),
            Self::Center(eye, axis) => {
                let labels = [
                    ["Image 1 center X", "Image 1 center Y"],
                    ["Image 2 center X", "Image 2 center Y"],
                ];
                return (
                    labels[eye][axis],
                    format!("{:.3}", p.lens.centers[eye][axis]),
                );
            }
            Self::Distortion(index) => {
                return (
                    ["Lens k1", "Lens k2", "Lens k3", "Lens k4"][index],
                    format!("{:.4}", p.lens.distortion[index]),
                );
            }
            Self::Reset => ("Reset", "Restore defaults"),
        };
        (label, value.to_owned())
    }

    pub(super) fn adjust(self, p: &mut Presentation, direction: i8) {
        if direction == 0 {
            return;
        }
        match self {
            Self::Projection => {
                let values = [
                    Projection::Hemisphere,
                    Projection::Flat,
                    Projection::Fisheye(190),
                ];
                let index = match p.projection {
                    Projection::Flat => 1,
                    Projection::Fisheye(_) => 2,
                    _ => 0,
                };
                p.projection =
                    values[(index + direction as isize).rem_euclid(values.len() as isize) as usize];
            }
            Self::Format => {
                p.projection = if let Projection::Fisheye(fov) = p.projection {
                    Projection::Fisheye((fov as i32 + direction as i32).clamp(1, 359) as u16)
                } else if p.projection == Projection::Hemisphere {
                    Projection::Sphere
                } else {
                    Projection::Hemisphere
                }
            }
            Self::Stereo => {
                let values = [Stereo::Mono, Stereo::SideBySide, Stereo::TopBottom];
                let index = values.iter().position(|value| *value == p.stereo).unwrap() as isize;
                p.stereo =
                    values[(index + direction as isize).rem_euclid(values.len() as isize) as usize];
            }
            Self::StereoFormat => {
                let format = if p.stereo == Stereo::TopBottom {
                    &mut p.tb_format
                } else {
                    &mut p.sbs_format
                };
                *format = match *format {
                    StereoFormat::Full => StereoFormat::Half,
                    StereoFormat::Half => StereoFormat::Full,
                };
            }
            Self::EyeOrder => p.swap_eyes = !p.swap_eyes,
            Self::Center(eye, axis) => {
                let value = &mut p.lens.centers[eye][axis];
                *value = (*value + direction as f32 * 0.001).clamp(0.0, 1.0);
            }
            Self::Distortion(index) => {
                let value = &mut p.lens.distortion[index];
                *value = (*value + direction as f32 * 0.0001).clamp(-1.0, 1.0);
            }
            Self::Reset => *p = Presentation::default(),
        }
    }
}
