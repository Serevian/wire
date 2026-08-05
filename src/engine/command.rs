use std::sync::Arc;

use ash::vk::{self, CommandBufferLevel};

use crate::engine::device::Device;

pub struct Command {
    device: Arc<Device>,
    pool: vk::CommandPool,
    buffer: vk::CommandBuffer,
}

impl Command {
    pub fn new(device: Arc<Device>) -> Self {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue.family_index);

        let pool = unsafe {
            device
                .logical
                .create_command_pool(&pool_info, None)
                .expect("Error creating command pool")
        };

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        todo!()
    }
}

impl Drop for Command {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.destroy_command_pool(self.pool, None);
        }
    }
}
