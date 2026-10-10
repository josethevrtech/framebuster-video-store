use anyhow::Result;
use ash::vk;
use matineevr::{vk_device::Device, vk_memory::Buffer, vk_sample::View};
use std::{io::Cursor, rc::Rc};

pub struct Pipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    set_layout: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    shader: vk::ShaderModule,
    device: Rc<Device>,
}

impl Pipeline {
    pub fn new(
        device: Rc<Device>,
        view: &View,
        buffer: &Buffer,
    ) -> Result<(Self, vk::DescriptorSet)> {
        let mut pipeline = Self {
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            set_layout: vk::DescriptorSetLayout::null(),
            pool: vk::DescriptorPool::null(),
            shader: vk::ShaderModule::null(),
            device: device.clone(),
        };
        unsafe {
            let d = &device.api;
            let samplers = [view.sampling.sampler];
            let bindings = [
                vk::DescriptorSetLayoutBinding::default()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .immutable_samplers(&samplers),
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
            ];
            pipeline.set_layout = d.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings),
                None,
            )?;
            let sizes = [
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: view.imported.descriptor_count,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::STORAGE_BUFFER,
                    descriptor_count: 1,
                },
            ];
            pipeline.pool = d.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&sizes),
                None,
            )?;
            let set = d.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(pipeline.pool)
                    .set_layouts(&[pipeline.set_layout]),
            )?[0];
            let image = [vk::DescriptorImageInfo::default()
                .image_view(view.handle)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
            let storage = [vk::DescriptorBufferInfo::default()
                .buffer(buffer.handle)
                .range(buffer.size as u64)];
            d.update_descriptor_sets(
                &[
                    vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(&image),
                    vk::WriteDescriptorSet::default()
                        .dst_set(set)
                        .dst_binding(1)
                        .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                        .buffer_info(&storage),
                ],
                &[],
            );
            pipeline.layout = d.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[pipeline.set_layout])
                    .push_constant_ranges(&[vk::PushConstantRange::default()
                        .stage_flags(vk::ShaderStageFlags::COMPUTE)
                        .size(16)]),
                None,
            )?;
            let code = ash::util::read_spv(&mut Cursor::new(include_bytes!(concat!(
                env!("OUT_DIR"),
                "/dmabuf_check.comp.spv"
            ))))?;
            pipeline.shader =
                d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None)?;
            pipeline.handle = match d.create_compute_pipelines(
                vk::PipelineCache::null(),
                &[vk::ComputePipelineCreateInfo::default()
                    .layout(pipeline.layout)
                    .stage(
                        vk::PipelineShaderStageCreateInfo::default()
                            .stage(vk::ShaderStageFlags::COMPUTE)
                            .module(pipeline.shader)
                            .name(c"main"),
                    )],
                None,
            ) {
                Ok(pipelines) => pipelines[0],
                Err((pipelines, error)) => {
                    for pipeline in pipelines {
                        d.destroy_pipeline(pipeline, None);
                    }
                    return Err(error.into());
                }
            };
            Ok((pipeline, set))
        }
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
            d.destroy_shader_module(self.shader, None);
        }
    }
}
