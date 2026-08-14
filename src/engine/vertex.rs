use std::mem::offset_of;

use ash::vk;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pos: ggmath::Vec2<f32>,
    color: ggmath::Vec3<f32>,
}

impl Vertex {
    pub fn new(pos: [f32; 2], color: [f32; 3]) -> Self {
        let pos = ggmath::Vec2::new(pos[0], pos[1]);
        let color = ggmath::Vec3::new(color[0], color[1], color[2]);
        Self { pos, color }
    }
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
                .offset(offset_of!(Vertex, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, color) as u32),
        ]
    }
}
