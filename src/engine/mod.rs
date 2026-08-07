use std::{ffi::CStr, sync::Arc};

use ash::vk;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::engine::{
    command::FrameData, context::Context, device::Device, pipeline::Pipeline, surface::Surface,
    swapchain::Swapchain,
};

mod command;
mod context;
mod device;
mod pipeline;
mod surface;
mod swapchain;

const VALIDATION_LAYERS: &[&CStr] = &[c"VK_LAYER_KHRONOS_validation"];

pub struct Engine {
    frame_data: FrameData,
    pipeline: Pipeline,
    swapchain: Swapchain,
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

        let swapchain = Swapchain::new(&context, &surface, device.clone(), width, height);

        let pipeline = Pipeline::new(device.clone(), &swapchain);

        let frame_data = FrameData::new(device.clone());

        Self {
            frame_data,
            pipeline,
            swapchain,
            device,
            surface,
            context,
        }
    }

    pub fn draw(&mut self) {
        let fences = [self.frame_data.draw_fence];
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

        let (index, suboptimal) = unsafe {
            self.swapchain
                .loader
                .acquire_next_image(
                    self.swapchain.raw,
                    u64::MAX,
                    self.frame_data.present_complete,
                    vk::Fence::null(),
                )
                .expect("Error acquiring next image")
        };

        let image = self.swapchain.images[index as usize];
        let image_view = self.swapchain.image_views[index as usize];
        self.frame_data
            .record_command(image, image_view, self.swapchain.extent, &self.pipeline);

        let wait_destination_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let present_semaphores = [self.frame_data.present_complete];
        let command_buffers = [self.frame_data.buffer];
        let render_semaphores = [self.frame_data.render_finished];
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
                    self.frame_data.draw_fence,
                )
                .expect("Error submiting work to the queue");
        };

        let index_binding = [index];
        let swapchains = [self.swapchain.raw];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&render_semaphores)
            .swapchains(&swapchains)
            .image_indices(&index_binding);

        let result = unsafe {
            self.swapchain
                .loader
                .queue_present(self.device.queue.raw, &present_info)
        };
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
