use std::sync::Arc;

use ash::vk::{self, ImageAspectFlags, ImageSubresourceRange};

use crate::engine::{buffer::Buffer, device::Device, pipeline::Pipeline, vertex::Vertex};

pub struct FramesInFlight {
    device: Arc<Device>,
    pool: vk::CommandPool,
    pub data: Vec<FrameData>,
    frames: usize,
}

impl FramesInFlight {
    pub fn new(device: Arc<Device>, frames_in_flight: usize) -> Self {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.queue.family_index);

        let pool = unsafe {
            device
                .logical
                .create_command_pool(&pool_info, None)
                .expect("Error creating command pool")
        };

        let mut data = vec![];

        for _ in 0..frames_in_flight {
            let frame = FrameData::new(device.clone(), &pool);
            data.push(frame);
        }

        Self {
            device,
            pool,
            data,
            frames: frames_in_flight,
        }
    }
}

impl Drop for FramesInFlight {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.destroy_command_pool(self.pool, None);
        }
    }
}

pub struct FrameData {
    device: Arc<Device>,
    pub buffer: vk::CommandBuffer,
    pub present_complete: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub draw_fence: vk::Fence,
}

impl FrameData {
    pub fn new(device: Arc<Device>, pool: &vk::CommandPool) -> Self {
        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(*pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe {
            device
                .logical
                .allocate_command_buffers(&command_buffer_info)
                .expect("Error allocating command buffers")[0]
        };

        let present_complete = unsafe {
            device
                .logical
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("Error creating present complete semaphore")
        };

        let render_finished = unsafe {
            device
                .logical
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .expect("Error creating render finished semaphore")
        };

        let draw_fence = unsafe {
            device
                .logical
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .expect("Error creating draw fence")
        };

        Self {
            device,
            buffer: command_buffer,
            present_complete,
            render_finished,
            draw_fence,
        }
    }

    pub fn record_command(
        &mut self,
        image: vk::Image,
        image_view: vk::ImageView,
        extent: vk::Extent2D,
        pipeline: &Pipeline,
        buffer: &Buffer,
        vertices: &[Vertex],
    ) {
        let begin_command_info = vk::CommandBufferBeginInfo::default();

        unsafe {
            self.device
                .logical
                .begin_command_buffer(self.buffer, &begin_command_info)
                .expect("Error beginning command buffer");
        };

        self.transition_image_layout(
            image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        );

        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let attachment_info = [vk::RenderingAttachmentInfo::default()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(clear_color)];

        let rendering_info = vk::RenderingInfo::default()
            .render_area(
                vk::Rect2D::default()
                    .offset(vk::Offset2D::default().x(0).y(0))
                    .extent(extent),
            )
            .layer_count(1)
            .color_attachments(&attachment_info);

        unsafe {
            self.device
                .logical
                .cmd_begin_rendering(self.buffer, &rendering_info);
        };

        unsafe {
            self.device.logical.cmd_bind_pipeline(
                self.buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.raw,
            );
        }

        let buffers = &[buffer.raw];
        unsafe {
            self.device
                .logical
                .cmd_bind_vertex_buffers(self.buffer, 0, buffers, &[0]);
        }

        let viewports = [vk::Viewport::default()
            .width(extent.width as f32)
            .height(extent.height as f32)
            .max_depth(1.0)];
        unsafe {
            self.device
                .logical
                .cmd_set_viewport(self.buffer, 0, &viewports);
        }

        let scissors = [vk::Rect2D::default()
            .offset(vk::Offset2D::default())
            .extent(extent)];
        unsafe {
            self.device
                .logical
                .cmd_set_scissor(self.buffer, 0, &scissors);
        }

        unsafe {
            self.device
                .logical
                .cmd_draw(self.buffer, vertices.len() as u32, 1, 0, 0);
        }

        unsafe {
            self.device.logical.cmd_end_rendering(self.buffer);
        }

        self.transition_image_layout(
            image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::AccessFlags2::NONE,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
        );

        unsafe {
            self.device
                .logical
                .end_command_buffer(self.buffer)
                .expect("Error ending command buffer");
        };
    }

    fn transition_image_layout(
        &mut self,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags2,
        dst_access_mask: vk::AccessFlags2,
        src_stage_mask: vk::PipelineStageFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
    ) {
        let barrier = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(src_stage_mask)
            .src_access_mask(src_access_mask)
            .dst_stage_mask(dst_stage_mask)
            .dst_access_mask(dst_access_mask)
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            )];

        let dependency_info = vk::DependencyInfo::default().image_memory_barriers(&barrier);

        unsafe {
            self.device
                .logical
                .cmd_pipeline_barrier2(self.buffer, &dependency_info);
        };
    }
}

impl Drop for FrameData {
    fn drop(&mut self) {
        unsafe {
            self.device.logical.destroy_fence(self.draw_fence, None);
            self.device
                .logical
                .destroy_semaphore(self.render_finished, None);
            self.device
                .logical
                .destroy_semaphore(self.present_complete, None);
        }
    }
}
