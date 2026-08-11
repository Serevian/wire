use std::mem::offset_of;

use ash::vk;

#[repr(C)]
pub struct Vertex {
    pos: ggmath::Vec2<f32>,
    color: ggmath::Vec2<f32>,
}

impl Vertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_description() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription::default()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}
