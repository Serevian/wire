use std::{ffi::CStr, sync::Arc};

use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::engine::{
    buffer::{Buffer, Index, Staging, Vertex},
    context::Context,
    device::Device,
    frame_data::FramesInFlight,
    pipeline::Pipeline,
    surface::Surface,
    swapchain::Swapchain,
};

mod buffer;
mod context;
mod device;
mod frame_data;
mod pipeline;
mod surface;
mod swapchain;
mod vertex;

const VALIDATION_LAYERS: &[&CStr] = &[c"VK_LAYER_KHRONOS_validation"];
const FRAMES_IN_FLIGHT: usize = 2;

pub struct Engine {
    indices: Vec<u16>,
    index_buffer: Buffer<Index>,
    vertex_buffer: Buffer<Vertex>,
    index: usize,
    frames: FramesInFlight,
    pipeline: Pipeline,
    swapchain: Swapchain,
    allocator: Arc<vk_mem::Allocator>,
    device: Arc<Device>,
    surface: Surface,
    context: Context,
}

impl Engine {
    pub fn new(
        required_extensions: &[*const i8],
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
        width: u32,
        height: u32,
    ) -> Self {
        #[cfg(debug_assertions)]
        let enable_validation = true;
        #[cfg(not(debug_assertions))]
        let enable_validation = false;

        let context = Context::new(required_extensions, enable_validation, VALIDATION_LAYERS);

        let surface = Surface::new(&context, raw_display_handle, raw_window_handle);

        let device = Arc::new(Device::new(&context, &surface));

        let mut allocator_info =
            vk_mem::AllocatorCreateInfo::new(&context.instance, &device.logical, device.physical);
        allocator_info.flags = vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

        let allocator = unsafe {
            Arc::new(vk_mem::Allocator::new(allocator_info).expect("Error creating allocator"))
        };

        let swapchain = Swapchain::new(&context, &surface, device.clone(), width, height, None);

        let pipeline = Pipeline::new(device.clone(), &swapchain);

        let frames = FramesInFlight::new(device.clone(), FRAMES_IN_FLIGHT);

        let vertices = vec![
            vertex::Vertex::new([-0.5, -0.5], [1.0, 1.0, 1.0]),
            vertex::Vertex::new([0.5, -0.5], [0.0, 1.0, 0.0]),
            vertex::Vertex::new([0.5, 0.5], [0.0, 0.0, 1.0]),
            vertex::Vertex::new([-0.5, 0.5], [1.0, 1.0, 1.0]),
        ];
        let vertices_size = std::mem::size_of_val(vertices.as_slice()) as u64;

        let vertex_staging_buffer =
            Buffer::<Staging>::new(device.clone(), allocator.clone(), vertices_size);
        let vertex_buffer = Buffer::<Vertex>::new(device.clone(), allocator.clone(), vertices_size);
        vertex_staging_buffer.write_slice(&vertices);

        let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
        let indices_size = std::mem::size_of_val(indices.as_slice()) as u64;

        let index_staging_buffer =
            Buffer::<Staging>::new(device.clone(), allocator.clone(), indices_size);
        let index_buffer = Buffer::<Index>::new(device.clone(), allocator.clone(), indices_size);
        index_staging_buffer.write_slice(&indices);

        let staging_command_buffer = frames.data[0].buffer;
        let command_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .logical
                .begin_command_buffer(staging_command_buffer, &command_info)
                .expect("Error beginning staging commands");
        }

        vertex_staging_buffer.copy_to(staging_command_buffer, &vertex_buffer);
        index_staging_buffer.copy_to(staging_command_buffer, &index_buffer);

        unsafe {
            device
                .logical
                .end_command_buffer(staging_command_buffer)
                .expect("Error ending staging commands");
        }

        unsafe {
            device
                .logical
                .queue_submit(
                    device.queue.raw,
                    &[vk::SubmitInfo::default().command_buffers(&[staging_command_buffer])],
                    vk::Fence::null(),
                )
                .expect("Error submiting staging commands");

            device
                .logical
                .queue_wait_idle(device.queue.raw)
                .expect("Error waiting idle");
        }

        Self {
            indices,
            index_buffer,
            vertex_buffer,
            index: 0,
            frames,
            pipeline,
            swapchain,
            allocator,
            device,
            surface,
            context,
        }
    }

    pub fn draw(&mut self) {
        let fences = [self.frames.data[self.index].draw_fence];
        let fence_result = unsafe { self.device.logical.wait_for_fences(&fences, true, u64::MAX) };

        match fence_result {
            Err(_) => panic!("Failed to wait for fence!!!!"),
            Ok(()) => unsafe {
                self.device
                    .logical
                    .reset_fences(&fences)
                    .expect("Error reseting fences");
            },
        }

        let acquire_result = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.raw,
                u64::MAX,
                self.frames.data[self.index].present_complete,
                vk::Fence::null(),
            )
        };

        let (index, _suboptimal) = match acquire_result {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return;
            }
            Err(e) => panic!("Error acquiring next image: {e:?}"),
        };

        unsafe {
            self.device.logical.reset_fences(&fences);
        }

        let image = self.swapchain.images[index as usize];
        let image_view = self.swapchain.image_views[index as usize];
        self.frames.data[self.index].record_command(
            image,
            image_view,
            self.swapchain.extent,
            &self.pipeline,
            &self.vertex_buffer,
            &self.index_buffer,
            &self.indices,
        );

        let wait_destination_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let present_semaphores = [self.frames.data[self.index].present_complete];
        let command_buffers = [self.frames.data[self.index].buffer];
        let render_semaphores = [self.frames.data[self.index].render_finished];
        let submit_info = [vk::SubmitInfo::default()
            .wait_semaphores(&present_semaphores)
            .wait_dst_stage_mask(&wait_destination_stage_mask)
            .command_buffers(&command_buffers)
            .signal_semaphores(&render_semaphores)];

        unsafe {
            self.device
                .logical
                .queue_submit(
                    self.device.queue.raw,
                    &submit_info,
                    self.frames.data[self.index].draw_fence,
                )
                .expect("Error submiting work to the queue");
        };

        let index_binding = [index];
        let swapchains = [self.swapchain.raw];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&render_semaphores)
            .swapchains(&swapchains)
            .image_indices(&index_binding);

        // TODO: If ErrorOutOfDate or Suboptimal, then recreate swapchain and try again in next draw
        let result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.device.queue.raw, &present_info)
        };

        match result {
            Ok(false) => {}
            Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                // TODO: Need to resize swapchain
            }
            Err(e) => panic!("Error presenting: {e:?}"),
        }

        self.index = (self.index + 1) % FRAMES_IN_FLIGHT;
    }

    pub fn resize_swapchain(&mut self, width: u32, height: u32) {
        unsafe {
            self.device
                .logical
                .queue_wait_idle(self.device.queue.raw)
                .expect("Error waiting idle");
        }

        self.swapchain = Swapchain::new(
            &self.context,
            &self.surface,
            self.device.clone(),
            width,
            height,
            Some(self.swapchain.raw),
        );
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.device
                .logical
                .queue_wait_idle(self.device.queue.raw)
                .expect("Error waiting idle");
        };
    }
}
