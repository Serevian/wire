use std::{marker::PhantomData, sync::Arc};

use ash::vk;
use vk_mem::Alloc;

use crate::engine::device::Device;

pub struct Index;
pub struct Staging;
pub struct Uniform;
pub struct Vertex;

pub struct Buffer<Usage> {
    device: Arc<Device>,
    allocator: Arc<vk_mem::Allocator>,
    pub raw: vk::Buffer,
    allocation: vk_mem::Allocation,
    size: u64,
    _marker: PhantomData<Usage>,
}

impl<U> Drop for Buffer<U> {
    fn drop(&mut self) {
        unsafe {
            self.allocator
                .destroy_buffer(self.raw, &mut self.allocation);
        }
    }
}

impl Buffer<Staging> {
    pub fn new(device: Arc<Device>, allocator: Arc<vk_mem::Allocator>, size: u64) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferHost,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
                | vk_mem::AllocationCreateFlags::MAPPED,
            ..Default::default()
        };

        let (raw, allocation) =
            unsafe { allocator.create_buffer(&buffer_info, &alloc_info).unwrap() };

        Self {
            device,
            allocator,
            raw,
            allocation,
            size,
            _marker: PhantomData,
        }
    }

    pub fn write_slice<T: bytemuck::NoUninit>(&self, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        assert!(bytes.len() as u64 <= self.size, "Data exceeds buffer size");

        let alloc_info = self.allocator.get_allocation_info(&self.allocation);
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                alloc_info.mapped_data as *mut u8,
                bytes.len(),
            );
        }
    }

    pub fn copy_to<U>(&self, cmd: vk::CommandBuffer, dst: &Buffer<U>) {
        let copy_region = vk::BufferCopy::default().size(self.size.min(dst.size));
        unsafe {
            self.device
                .logical
                .cmd_copy_buffer(cmd, self.raw, dst.raw, &[copy_region]);
        }
    }
}

impl Buffer<Vertex> {
    /// Vertex buffers are gpu mapped, you need a staging buffer to allocate data
    pub fn new(device: Arc<Device>, allocator: Arc<vk_mem::Allocator>, size: u64) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (raw, allocation) = unsafe {
            allocator
                .create_buffer(&buffer_info, &alloc_info)
                .expect("Error creating vertex buffer")
        };

        Self {
            device,
            allocator,
            raw,
            allocation,
            size,
            _marker: PhantomData,
        }
    }

    pub fn bind(&self, cmd: vk::CommandBuffer, binding: u32) {
        unsafe {
            self.device
                .logical
                .cmd_bind_vertex_buffers(cmd, binding, &[self.raw], &[0]);
        }
    }
}

impl Buffer<Index> {
    pub fn new(device: Arc<Device>, allocator: Arc<vk_mem::Allocator>, size: u64) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferDevice,
            ..Default::default()
        };

        let (raw, allocation) =
            unsafe { allocator.create_buffer(&buffer_info, &alloc_info).unwrap() };

        Self {
            device,
            allocator,
            raw,
            allocation,
            size,
            _marker: PhantomData,
        }
    }

    pub fn bind(&self, cmd: vk::CommandBuffer, index_type: vk::IndexType) {
        unsafe {
            self.device
                .logical
                .cmd_bind_index_buffer(cmd, self.raw, 0, index_type);
        }
    }
}

impl Buffer<Uniform> {
    pub fn new(device: Arc<Device>, allocator: Arc<vk_mem::Allocator>, size: u64) -> Self {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let alloc_info = vk_mem::AllocationCreateInfo {
            usage: vk_mem::MemoryUsage::AutoPreferHost,
            flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE
                | vk_mem::AllocationCreateFlags::MAPPED,
            ..Default::default()
        };

        let (raw, allocation) =
            unsafe { allocator.create_buffer(&buffer_info, &alloc_info).unwrap() };

        Self {
            device,
            allocator,
            raw,
            allocation,
            size,
            _marker: PhantomData,
        }
    }
}
