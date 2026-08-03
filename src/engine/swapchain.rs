use ash::vk::{
    self, ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageUsageFlags, PresentModeKHR,
    SharingMode, SurfaceCapabilities2KHR, SurfaceFormatKHR, SwapchainKHR,
};

use crate::engine::{context::Context, device::Device, surface::Surface};

pub struct Swapchain {
    device: ash::khr::swapchain::Device,
    raw: SwapchainKHR,
    images: Vec<vk::Image>,
    format: SurfaceFormatKHR,
    extent: Extent2D,
}

impl Swapchain {
    pub fn new(
        context: &Context,
        surface: &Surface,
        device: &Device,
        width: u32,
        height: u32,
    ) -> Self {
        let swapchain_device =
            ash::khr::swapchain::Device::load(&context.instance, &device.logical);

        let capabilities_loader =
            ash::khr::get_surface_capabilities2::Instance::load(&context.entry, &context.instance);

        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, surface.raw)
                .expect("Error getting surface capabilities")
        };

        let extent = Self::choose_extent(surface, device, width, height);

        let min_image_count = Self::choose_min_image_count(surface, device);

        let format = Self::choose_format(surface, device);

        let present_mode = Self::choose_present_mode(surface, device);

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw)
            .min_image_count(min_image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT) // Render directly to swapchain. If I want to make operations on the final image (i.e. post processing), you need to render a separate image then transfer it to the swapchain (TransferDst)
            .image_sharing_mode(SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let swapchain = unsafe {
            swapchain_device
                .create_swapchain(&swapchain_info, None)
                .expect("Error creating swapchain")
        };

        let images = unsafe {
            swapchain_device
                .get_swapchain_images(swapchain)
                .expect("Error getting swapchain images")
        };

        Self {
            device: swapchain_device,
            raw: swapchain,
            images,
            format,
            extent,
        }
    }

    fn choose_extent(surface: &Surface, device: &Device, width: u32, height: u32) -> Extent2D {
        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, surface.raw)
                .expect("Error getting surface capabilities")
        };

        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        let new_width = num::clamp(
            width,
            capabilities.min_image_extent.width,
            capabilities.max_image_extent.width,
        );
        let new_height = num::clamp(
            height,
            capabilities.min_image_extent.height,
            capabilities.max_image_extent.height,
        );
        vk::Extent2D::default().width(new_width).height(new_height)
    }

    fn choose_min_image_count(surface: &Surface, device: &Device) -> u32 {
        let capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, surface.raw)
                .expect("Error getting surface capabilities")
        };

        let mut min_image_count = capabilities.min_image_count.max(3);
        if capabilities.max_image_count > 0 && capabilities.max_image_count < min_image_count {
            min_image_count = capabilities.max_image_count;
        }

        min_image_count
    }

    fn choose_format(surface: &Surface, device: &Device) -> SurfaceFormatKHR {
        let formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device.physical, surface.raw)
                .expect("Error getting surface formats")
        };

        *formats
            .iter()
            .find(|format| {
                format.format == Format::B8G8R8A8_SRGB
                    && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            })
            .expect("Error getting a suitable surface format")
    }

    fn choose_present_mode(surface: &Surface, device: &Device) -> PresentModeKHR {
        let present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device.physical, surface.raw)
                .expect("Error getting surface present modes")
        };

        if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_swapchain(self.raw, None);
        }
    }
}
