use crate::navigation::Navigation;

#[path = "presentation_fields.rs"]
mod fields;
pub use fields::Field;
#[path = "fisheye.rs"]
mod fisheye;
pub use fisheye::Lens;
#[path = "presentation_parse.rs"]
mod parse;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Projection {
    Flat,
    #[default]
    Hemisphere,
    Sphere,
    Fisheye(u16),
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stereo {
    Mono,
    #[default]
    SideBySide,
    TopBottom,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum StereoFormat {
    #[default]
    Full,
    Half,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Presentation {
    pub projection: Projection,
    pub stereo: Stereo,
    pub sbs_format: StereoFormat,
    pub tb_format: StereoFormat,
    pub swap_eyes: bool,
    pub alignment: crate::alignment::Alignment,
    pub lens: Lens,
}

impl Presentation {
    pub fn fields(self) -> &'static [Field] {
        if self.projection == Projection::Flat && self.stereo != Stereo::Mono {
            &[
                Field::Projection,
                Field::Stereo,
                Field::StereoFormat,
                Field::EyeOrder,
                Field::Reset,
            ]
        } else if self.projection == Projection::Flat {
            &[
                Field::Projection,
                Field::Stereo,
                Field::EyeOrder,
                Field::Reset,
            ]
        } else if matches!(self.projection, Projection::Fisheye(_)) {
            &[
                Field::Projection,
                Field::Format,
                Field::Stereo,
                Field::EyeOrder,
                Field::Center(0, 0),
                Field::Center(0, 1),
                Field::Center(1, 0),
                Field::Center(1, 1),
                Field::Distortion(0),
                Field::Distortion(1),
                Field::Distortion(2),
                Field::Distortion(3),
                Field::Reset,
            ]
        } else {
            &[
                Field::Projection,
                Field::Format,
                Field::Stereo,
                Field::EyeOrder,
                Field::Reset,
            ]
        }
    }
}

pub fn navigate(presentation: &mut Presentation, row: &mut usize, navigation: Navigation) {
    let fields = presentation.fields();
    *row = (*row)
        .min(fields.len() - 1)
        .saturating_add_signed(navigation.vertical as isize)
        .min(fields.len() - 1);
    let selected = fields[*row];
    selected.adjust(presentation, navigation.horizontal);
    *row = presentation
        .fields()
        .iter()
        .position(|field| *field == selected)
        .unwrap();
}

#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;
