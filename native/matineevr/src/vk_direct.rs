use super::{
    vk_device::Device,
    vk_import::Imported,
    vk_pipeline::Pipeline,
    vk_sample::{Sampling, View},
};
use crate::media::{DmaBuf, Frame};
use anyhow::{Result, ensure};
use ash::vk;
use std::{collections::HashMap, rc::Rc};

pub struct Direct {
    images: HashMap<i32, View>,
    pub pipeline: Pipeline,
    source: DmaBuf,
    color: (i32, i32),
    device: Rc<Device>,
}

impl Direct {
    pub fn new(
        device: Rc<Device>,
        source: DmaBuf,
        frame: &Frame,
        stats: Option<&mut crate::statistics::Statistics>,
    ) -> Result<Self> {
        ensure!(
            ![16, 18].contains(&source.transfer),
            "HDR tone mapping is not implemented"
        );
        let imported = Imported::new(device.clone(), &source)?;
        let sampling = Rc::new(Sampling::new(
            device.clone(),
            &imported,
            Some(&frame.pixels),
            source.chroma_location,
        )?);
        let pipeline = crate::statistics::timed(stats, "pipeline_create_ms", || {
            Pipeline::imported(device.clone(), sampling.clone(), imported.descriptor_count)
        })?;
        Ok(Self {
            images: HashMap::from([(source.fd, sampling.view(imported)?)]),
            pipeline,
            source,
            color: (frame.pixels.colorspace, frame.pixels.full_range),
            device,
        })
    }

    pub fn compatible(&self, frame: &Frame) -> bool {
        let source = frame.dmabuf();
        source.format == self.source.format
            && source.modifier == self.source.modifier
            && source.chroma_location == self.source.chroma_location
            && source.transfer == self.source.transfer
            && source.primaries == self.source.primaries
            && self.color == (frame.pixels.colorspace, frame.pixels.full_range)
    }

    pub fn reset(&mut self) {
        self.images.clear();
    }

    pub fn prepare(&mut self, frame: &Frame) -> Result<()> {
        let source = frame.dmabuf();
        if self.images.is_empty() {
            ensure!(self.compatible(frame), "Incompatible sampling conversion");
            self.source = source;
        }
        ensure!(
            source.pool == self.source.pool
                && source.width == self.source.width
                && source.height == self.source.height
                && source.crop == self.source.crop
                && self.compatible(frame),
            "Decoder layout changed without releasing the old pool"
        );
        if let std::collections::hash_map::Entry::Vacant(entry) = self.images.entry(source.fd) {
            entry.insert(
                self.pipeline
                    .sampling
                    .view(Imported::new(self.device.clone(), &source)?)?,
            );
        }
        Ok(())
    }

    pub fn bind(&self, frame: &Frame, slot: usize) -> vk::Image {
        let image = &self.images[&frame.dmabuf().fd];
        let info = [vk::DescriptorImageInfo::default()
            .image_view(image.handle)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        unsafe {
            self.device.api.update_descriptor_sets(
                &[vk::WriteDescriptorSet::default()
                    .dst_set(self.pipeline.descriptors[slot])
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&info)],
                &[],
            );
        }
        image.imported.handle
    }

    pub fn crop(&self) -> [f32; 4] {
        let s = &self.source;
        let width = (s.width - s.crop[2]) as f32;
        let height = (s.height - s.crop[3]) as f32;
        [
            s.crop[0] as f32 / width,
            s.crop[1] as f32 / height,
            (s.width - s.crop[0] - s.crop[2]) as f32 / width,
            (s.height - s.crop[1] - s.crop[3]) as f32 / height,
        ]
    }
}
