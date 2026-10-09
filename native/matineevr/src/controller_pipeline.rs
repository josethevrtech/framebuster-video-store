use anyhow::{Result, anyhow};
use ash::vk;
use crate::graphics::Graphics;
use std::{io::Cursor, rc::Rc};

pub struct ControllerPipeline {
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    shaders: Vec<vk::ShaderModule>,
    device: Rc<Graphics>,
}

impl ControllerPipeline {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let mut p = Self { handle: vk::Pipeline::null(), layout: vk::PipelineLayout::null(),
            shaders: Vec::new(), device };
        unsafe {
            let d = &p.device.api;
            let ranges = [vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX).size(80)];
            p.layout = d.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().push_constant_ranges(&ranges), None)?;
            for code in [
                include_bytes!(concat!(env!("OUT_DIR"), "/controller.vert.spv")).as_slice(),
                include_bytes!(concat!(env!("OUT_DIR"), "/controller.frag.spv")).as_slice(),
            ] {
                let words = ash::util::read_spv(&mut Cursor::new(code))?;
                p.shaders.push(d.create_shader_module(
                    &vk::ShaderModuleCreateInfo::default().code(&words), None)?);
            }
            let stages = [vk::ShaderStageFlags::VERTEX, vk::ShaderStageFlags::FRAGMENT]
                .map(|stage| vk::PipelineShaderStageCreateInfo::default().stage(stage)
                    .name(c"main").module(p.shaders[usize::from(stage == vk::ShaderStageFlags::FRAGMENT)]));
            let bindings = [vk::VertexInputBindingDescription {
                binding: 0, stride: 48, input_rate: vk::VertexInputRate::VERTEX }];
            let attributes: [_; 3] = std::array::from_fn(|i| vk::VertexInputAttributeDescription {
                location: i as u32, binding: 0, format: vk::Format::R32G32B32A32_SFLOAT,
                offset: i as u32 * 16 });
            let vertex = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(&bindings).vertex_attribute_descriptions(&attributes);
            let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            let viewport = vk::PipelineViewportStateCreateInfo::default().viewport_count(1).scissor_count(1);
            let raster = vk::PipelineRasterizationStateCreateInfo::default().line_width(1.0);
            let samples = vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let attachments = [vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachments);
            let depth = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true).depth_write_enable(true).depth_compare_op(vk::CompareOp::LESS);
            let dynamic = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
            let formats = [matineevr::vk_pipeline::FORMAT];
            let mut rendering = vk::PipelineRenderingCreateInfo::default()
                .color_attachment_formats(&formats).depth_attachment_format(vk::Format::D32_SFLOAT);
            let info = vk::GraphicsPipelineCreateInfo::default().stages(&stages).layout(p.layout)
                .vertex_input_state(&vertex).input_assembly_state(&assembly).viewport_state(&viewport)
                .rasterization_state(&raster).multisample_state(&samples).color_blend_state(&blend)
                .depth_stencil_state(&depth).dynamic_state(&dynamic).push_next(&mut rendering);
            p.handle = match d.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) {
                Ok(pipelines) => pipelines[0],
                Err((pipelines, error)) => {
                    for pipeline in pipelines { d.destroy_pipeline(pipeline, None); }
                    return Err(anyhow!("Create controller pipeline: {error}"));
                }
            };
        }
        Ok(p)
    }
}

impl Drop for ControllerPipeline {
    fn drop(&mut self) {
        unsafe {
            let d = &self.device.api;
            if let Err(error) = d.device_wait_idle() { eprintln!("Controller shutdown: {error}"); }
            d.destroy_pipeline(self.handle, None);
            d.destroy_pipeline_layout(self.layout, None);
            for shader in &self.shaders { d.destroy_shader_module(*shader, None); }
        }
    }
}
