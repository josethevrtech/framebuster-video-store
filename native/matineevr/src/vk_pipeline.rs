pub use super::vk_parameters::Parameters;
use super::{renderer::IN_FLIGHT, vk_device::Device, vk_sample::Sampling};
use anyhow::{Result, anyhow};
use ash::vk;
use std::{io::Cursor, rc::Rc};

pub const FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;

pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptors: [vk::DescriptorSet; IN_FLIGHT],
    pool: vk::DescriptorPool,
    set_layout: vk::DescriptorSetLayout,
    shaders: Vec<vk::ShaderModule>,
    device: Rc<Device>,
    pub sampling: Rc<Sampling>,
}

impl Pipeline {
    pub fn imported(device: Rc<Device>, sampling: Rc<Sampling>, count: u32) -> Result<Self> {
        let mut p = Self {
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            descriptors: [vk::DescriptorSet::null(); IN_FLIGHT],
            pool: vk::DescriptorPool::null(),
            set_layout: vk::DescriptorSetLayout::null(),
            shaders: Vec::new(),
            device,
            sampling,
        };
        unsafe {
            let d = &p.device.api;
            let samplers = [p.sampling.sampler];
            let bindings = [vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .immutable_samplers(&samplers)];
            p.set_layout = d.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )?;
            let pool_sizes = [vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: count * IN_FLIGHT as u32,
            }];
            p.pool = d.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(IN_FLIGHT as u32)
                    .pool_sizes(&pool_sizes),
                None,
            )?;
            p.descriptors = d
                .allocate_descriptor_sets(
                    &vk::DescriptorSetAllocateInfo::default()
                        .descriptor_pool(p.pool)
                        .set_layouts(&[p.set_layout; IN_FLIGHT]),
                )?
                .try_into()
                .unwrap();
            let constants = [vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .size(size_of::<Parameters>() as u32)];
            p.layout = d.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[p.set_layout])
                    .push_constant_ranges(&constants),
                None,
            )?;
            for code in [
                include_bytes!(concat!(env!("OUT_DIR"), "/video.vert.spv")).as_slice(),
                include_bytes!(concat!(env!("OUT_DIR"), "/video.frag.spv")).as_slice(),
            ] {
                let words = ash::util::read_spv(&mut Cursor::new(code))?;
                p.shaders.push(d.create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&words),
                    None,
                )?);
            }
            let stages =
                [vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT].map(|stage| {
                    vk::PipelineShaderStageCreateInfo::default()
                        .stage(stage)
                        .name(c"main")
                        .module(p.shaders[usize::from(stage == vk::ShaderStageFlags::FRAGMENT)])
                });
            let vertex = vk::PipelineVertexInputStateCreateInfo::default();
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            let viewport = vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1);
            let raster = vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0);
            let samples = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let attachment = [vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachment);
            let dynamic = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
            let formats = [FORMAT];
            let mut rendering =
                vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&formats);
            let info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .layout(p.layout)
                .vertex_input_state(&vertex)
                .input_assembly_state(&assembly)
                .viewport_state(&viewport)
                .rasterization_state(&raster)
                .multisample_state(&samples)
                .color_blend_state(&blend)
                .dynamic_state(&dynamic)
                .push_next(&mut rendering);
            p.handle = match d.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) {
                Ok(pipelines) => pipelines[0],
                Err((pipelines, error)) => {
                    for pipeline in pipelines {
                        d.destroy_pipeline(pipeline, None);
                    }
                    return Err(anyhow!("Create video pipeline: {error}"));
                }
            };
        }
        Ok(p)
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            d.destroy_pipeline(self.handle, None);
            d.destroy_pipeline_layout(self.layout, None);
            d.destroy_descriptor_pool(self.pool, None);
            d.destroy_descriptor_set_layout(self.set_layout, None);
            for shader in &self.shaders {
                d.destroy_shader_module(*shader, None);
            }
        }
    }
}
