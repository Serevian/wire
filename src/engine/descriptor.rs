use std::sync::Arc;

use ash::vk;

use crate::engine::device::Device;

pub struct DescriptorLayout {
    device: Arc<Device>,
    pub raw: vk::DescriptorSetLayout,
}

impl DescriptorLayout {
    pub fn new(device: Arc<Device>) -> Self {
        let ubo_layout_binding = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];

        let layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&ubo_layout_binding);
        let raw = unsafe {
            device
                .logical
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Error creating descriptor set layout")
        };

        Self { device, raw }
    }
}
