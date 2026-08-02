use ash::vk::SurfaceKHR;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::engine::context::Context;

pub struct Surface {
    pub loader: ash::khr::surface::Instance,
    pub raw: SurfaceKHR,
}

impl Surface {
    pub fn new(
        context: &Context,
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> Self {
        let loader = ash::khr::surface::Instance::load(&context.entry, &context.instance);

        let raw = unsafe {
            ash_window::SurfaceFactory::new(&context.entry, &context.instance, raw_display_handle)
                .expect("Error loading surface factory")
                .create_surface(raw_window_handle, None)
                .expect("Error creating vulkan surface")
        };

        Self { loader, raw }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.raw, None);
        }
    }
}
