use super::{vk_device::Device, vk_import::Imported, vk_memory::COLOR};
use crate::media::Pixels;
use anyhow::{Result, ensure};
use ash::vk;
use std::rc::Rc;

pub struct Sampling {
    pub sampler: vk::Sampler,
    conversion: vk::SamplerYcbcrConversion,
    device: Rc<Device>,
}

pub struct View {
    pub handle: vk::ImageView,
    pub imported: Imported,
    pub sampling: Rc<Sampling>,
}

impl Sampling {
    pub fn new(
        device: Rc<Device>,
        imported: &Imported,
        pixels: Option<&Pixels>,
        chroma: i32,
    ) -> Result<Self> {
        ensure!(
            pixels.is_none() || (0..=4).contains(&chroma),
            "Unsupported chroma location: {chroma}"
        );
        let model = pixels.map_or(vk::SamplerYcbcrModelConversion::RGB_IDENTITY, |p| {
            match p.colorspace {
                5 | 6 => vk::SamplerYcbcrModelConversion::YCBCR_601,
                9 => vk::SamplerYcbcrModelConversion::YCBCR_2020,
                _ => vk::SamplerYcbcrModelConversion::YCBCR_709,
            }
        });
        let range = if pixels.is_some_and(|p| p.full_range == 0) {
            vk::SamplerYcbcrRange::ITU_NARROW
        } else {
            vk::SamplerYcbcrRange::ITU_FULL
        };
        let filter = if pixels.is_some() {
            vk::Filter::LINEAR
        } else {
            vk::Filter::NEAREST
        };
        let x = if pixels.is_some() && [1, 3].contains(&chroma) {
            vk::ChromaLocation::COSITED_EVEN
        } else {
            vk::ChromaLocation::MIDPOINT
        };
        let y = if pixels.is_some() && [3, 4].contains(&chroma) {
            vk::ChromaLocation::COSITED_EVEN
        } else {
            vk::ChromaLocation::MIDPOINT
        };
        let mut required = vk::FormatFeatureFlags::SAMPLED_IMAGE;
        for location in [x, y] {
            required |= if location == vk::ChromaLocation::COSITED_EVEN {
                vk::FormatFeatureFlags::COSITED_CHROMA_SAMPLES
            } else {
                vk::FormatFeatureFlags::MIDPOINT_CHROMA_SAMPLES
            };
        }
        if pixels.is_some() {
            required |= vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
                | vk::FormatFeatureFlags::SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER;
        }
        ensure!(
            imported.features.contains(required),
            "Unsupported YCbCr filtering/chroma location"
        );
        let mut result = Self {
            sampler: vk::Sampler::null(),
            conversion: vk::SamplerYcbcrConversion::null(),
            device,
        };
        unsafe {
            let d = &result.device.api;
            result.conversion = d.create_sampler_ycbcr_conversion(
                &vk::SamplerYcbcrConversionCreateInfo::default()
                    .format(imported.format)
                    .ycbcr_model(model)
                    .ycbcr_range(range)
                    .x_chroma_offset(x)
                    .y_chroma_offset(y)
                    .chroma_filter(filter),
                None,
            )?;
            let mut conversion =
                vk::SamplerYcbcrConversionInfo::default().conversion(result.conversion);
            result.sampler = d.create_sampler(
                &vk::SamplerCreateInfo::default()
                    .mag_filter(filter)
                    .min_filter(filter)
                    .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .push_next(&mut conversion),
                None,
            )?;
        }
        Ok(result)
    }

    pub fn view(self: &Rc<Self>, imported: Imported) -> Result<View> {
        let mut conversion = vk::SamplerYcbcrConversionInfo::default().conversion(self.conversion);
        let handle = unsafe {
            self.device.api.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(imported.handle)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(imported.format)
                    .subresource_range(COLOR)
                    .push_next(&mut conversion),
                None,
            )?
        };
        Ok(View {
            handle,
            imported,
            sampling: self.clone(),
        })
    }
}

impl Drop for Sampling {
    fn drop(&mut self) {
        unsafe {
            self.device.api.destroy_sampler(self.sampler, None);
            self.device
                .api
                .destroy_sampler_ycbcr_conversion(self.conversion, None);
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            self.sampling
                .device
                .api
                .destroy_image_view(self.handle, None)
        };
    }
}
