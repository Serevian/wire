use std::sync::Arc;

use ash::vk;

use crate::engine::{device::Device, vertex::Vertex};

pub struct Buffer {
    device: Arc<Device>,
    pub raw: vk::Buffer,
    memory: vk::DeviceMemory,
    size: u64,
}

impl Buffer {
    pub fn new(device: Arc<Device>, vertex_count: usize) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size((size_of::<Vertex>() * vertex_count) as u64)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let raw = unsafe {
            device
                .logical
                .create_buffer(&buffer_info, None)
                .expect("Error creating buffer")
        };

        let memory_requirements = unsafe { device.logical.get_buffer_memory_requirements(raw) };

        let memory_type_index = Self::find_memory_type(
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &device,
        );

        let memory_allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = unsafe {
            device
                .logical
                .allocate_memory(&memory_allocate_info, None)
                .expect("Error allocating memory")
        };

        unsafe {
            device
                .logical
                .bind_buffer_memory(raw, buffer_memory, 0)
                .expect("Error binding buffer memory");
        }

        Self {
            device,
            raw,
            memory: buffer_memory,
            size: buffer_info.size,
        }
    }

    pub fn allocate(&self, vertices: &[Vertex]) {
        unsafe {
            let data = self
                .device
                .logical
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
                .expect("Error mapping memory to buffer");

            let bytes: &[u8] = bytemuck::cast_slice(vertices);

            std::ptr::copy_nonoverlapping(bytes.as_ptr(), data as *mut u8, bytes.len());

            self.device.logical.unmap_memory(self.memory);
        }
    }

    fn find_memory_type(
        filter: u32,
        properties: vk::MemoryPropertyFlags,
        device: &Arc<Device>,
    ) -> u32 {
        (0..device.memory_properties.memory_type_count)
            .find(|&i| {
                let type_supported = filter & (1 << i) != 0;
                let properties_supported = device.memory_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties);
                type_supported && properties_supported
            })
            .expect("Error finding right memory type")
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.free_memory(self.memory, None);
            self.device.logical.destroy_buffer(self.raw, None);
        }
    }
}
