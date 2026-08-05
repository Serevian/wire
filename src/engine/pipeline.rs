use std::{io::Cursor, sync::Arc};

use ash::vk::{
    self, ColorComponentFlags, CullModeFlags, FrontFace, LogicOp, PipelineCache, PolygonMode,
    PrimitiveTopology, SampleCountFlags, ShaderStageFlags, TaggedStructure,
};

use crate::engine::{device::Device, swapchain::Swapchain};

pub struct Pipeline {
    device: Arc<Device>,
    layout: vk::PipelineLayout,
    raw: vk::Pipeline,
}

impl Pipeline {
    pub fn new(device: Arc<Device>, swapchain: &Swapchain) -> Self {
        let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/slang.spv"));

        let shader_module = Self::create_shader_module(&device, bytes);

        let vertex_shader_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::VERTEX)
            .module(shader_module)
            .name(c"vertMain");

        let fragment_shader_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(shader_module)
            .name(c"fragMain");

        let shader_stages = [vertex_shader_info, fragment_shader_info];

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let pipeline_dynamic_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(PolygonMode::FILL)
            .cull_mode(CullModeFlags::BACK)
            .front_face(FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .line_width(1.0);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        let color_blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(ColorComponentFlags::RGBA)];

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(LogicOp::COPY)
            .attachments(&color_blend_attachment);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();

        let layout = unsafe {
            device
                .logical
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Error creating pipeline layour")
        };

        let format = [swapchain.format.format];
        let mut pipeline_rendering_info =
            vk::PipelineRenderingCreateInfo::default().color_attachment_formats(&format);

        let pipeline_graphics_info = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .dynamic_state(&pipeline_dynamic_state_info)
            .layout(layout)
            .push(&mut pipeline_rendering_info)];

        let pipelines = unsafe {
            device
                .logical
                .create_graphics_pipelines(PipelineCache::null(), &pipeline_graphics_info, None)
                .expect("Error creating pipeline")
        };

        unsafe {
            device.logical.destroy_shader_module(shader_module, None);
        }

        Self {
            device,
            layout,
            raw: pipelines[0],
        }
    }

    fn create_shader_module(device: &Arc<Device>, shader_bytes: &[u8]) -> vk::ShaderModule {
        let mut cursor = Cursor::new(shader_bytes);
        let code = ash::util::read_spv(&mut cursor).expect("Failed to read SPIR-V code");

        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(&code);

        unsafe {
            device
                .logical
                .create_shader_module(&shader_module_info, None)
                .expect("Error creating shader module")
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.destroy_pipeline(self.raw, None);

            self.device
                .logical
                .destroy_pipeline_layout(self.layout, None);
        }
    }
}
