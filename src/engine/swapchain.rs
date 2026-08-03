use std::sync::Arc;

use ash::vk::{ColorSpaceKHR, Format, SurfaceFormatKHR};

use crate::engine::{device::Device, surface::Surface};

pub struct Swapchain {}

impl Swapchain {
    pub fn new(surface: &Surface, device: Arc<Device>) -> Self {
        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, surface.raw)
                .expect("Error getting surface capabilities")
        };

        let formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device.physical, surface.raw)
                .expect("Error getting surface formats")
        };

        let format = formats
            .iter()
            .find(|format| {
                format.format == Format::B8G8R8A8_SRGB
                    && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .expect("Error getting a suitable surface format");

        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device.physical, surface.raw)
                .expect("Error getting surface present modes")
        };

        todo!()
    }
}
