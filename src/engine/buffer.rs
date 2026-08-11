use std::sync::Arc;

use ash::vk;

use crate::engine::{device::Device, vertex::Vertex};

pub struct Buffer {
    device: Arc<Device>,
    raw: vk::Buffer,
    info: Vertex,
}

impl Buffer {
    pub fn new(device: Arc<Device>) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size_of::<Vertex>() as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let raw = unsafe {
            device
                .logical
                .create_buffer(&buffer_info, None)
                .expect("Error creating buffer")
        };

        let memory_requirements = unsafe { device.logical.get_buffer_memory_requirements(raw) };
        

        todo!()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.destroy_buffer(self.raw, None);
        }
    }
}
