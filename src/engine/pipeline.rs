use std::sync::Arc;

use ash::vk::{self, ShaderStageFlags};

use crate::engine::device::Device;

pub struct Pipeline {
    device: Arc<Device>,
}

impl Pipeline {
    pub fn new(device: Arc<Device>) -> Self {
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

        todo!()
    }

    fn create_shader_module(device: &Arc<Device>, shader_bytes: &[u8]) -> vk::ShaderModule {
        let aligned_bytes: &[u32] = bytemuck::cast_slice(shader_bytes);

        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(aligned_bytes);

        unsafe {
            device
                .logical
                .create_shader_module(&shader_module_info, None)
                .expect("Error creating shader module")
        }
    }
}
