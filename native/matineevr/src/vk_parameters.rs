use crate::video_params::View;

#[repr(C)]
pub struct Parameters {
    pub view: View,
    pub crop: [f32; 4],
}

impl Parameters {
    pub fn bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, size_of::<Self>()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_constants_match_glsl_offsets() {
        assert_eq!(size_of::<View>(), 112);
        assert_eq!(std::mem::offset_of!(View, framing), 32);
        assert_eq!(std::mem::offset_of!(View, stereo_offset), 48);
        assert_eq!(std::mem::offset_of!(View, lens), 80);
        assert_eq!(std::mem::offset_of!(View, distortion), 96);
        assert_eq!(std::mem::offset_of!(Parameters, crop), 112);
        assert_eq!(size_of::<Parameters>(), 128);
    }
}
