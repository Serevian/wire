use std::sync::Arc;

use ash::vk::{
    self, ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, ImageAspectFlags,
    ImageSubresourceRange, ImageUsageFlags, ImageViewType, PhysicalDeviceSurfaceInfo2KHR,
    PresentModeKHR, SharingMode, SurfaceCapabilities2KHR, SurfaceCapabilitiesKHR,
    SurfaceFormat2KHR, SurfaceFormatKHR, SwapchainKHR,
};

use crate::engine::{context::Context, device::Device, surface::Surface};

pub struct Swapchain {
    device: Arc<Device>,
    loader: ash::khr::swapchain::Device,
    raw: SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    pub format: SurfaceFormatKHR,
    pub extent: Extent2D,
}

impl Swapchain {
    pub fn new(
        context: &Context,
        surface: &Surface,
        device: Arc<Device>,
        width: u32,
        height: u32,
    ) -> Self {
        let swapchain_device =
            ash::khr::swapchain::Device::load(&context.instance, &device.logical);

        let capabilities_loader =
            ash::khr::get_surface_capabilities2::Instance::load(&context.entry, &context.instance);
        let capabilities2 = Self::get_capabilities(&capabilities_loader, surface, &device);
        let capabilities = &capabilities2.surface_capabilities;

        let extent = Self::choose_extent(capabilities, width, height);

        let min_image_count = Self::choose_min_image_count(capabilities);

        let format = Self::choose_format(&capabilities_loader, surface, &device);

        let present_mode = Self::choose_present_mode(surface, &device);

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw)
            .min_image_count(min_image_count)
            .image_format(format.surface_format.format)
            .image_color_space(format.surface_format.color_space)
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

        let image_views = Self::get_image_views(&device, format.surface_format, &images);

        Self {
            device,
            loader: swapchain_device,
            raw: swapchain,
            images,
            image_views,
            format: format.surface_format,
            extent,
        }
    }

    fn get_capabilities<'a>(
        capabilities_loader: &ash::khr::get_surface_capabilities2::Instance,
        surface: &Surface,
        device: &Device,
    ) -> SurfaceCapabilities2KHR<'a> {
        let surface_info = PhysicalDeviceSurfaceInfo2KHR::default().surface(surface.raw);

        let mut capabilities2 = vk::SurfaceCapabilities2KHR::default();

        unsafe {
            capabilities_loader
                .get_physical_device_surface_capabilities2(
                    device.physical,
                    &surface_info,
                    &mut capabilities2,
                )
                .expect("Error getting surface capabilities2");
        }

        capabilities2
    }

    fn choose_extent(capabilities: &SurfaceCapabilitiesKHR, width: u32, height: u32) -> Extent2D {
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

    fn choose_min_image_count(capabilities: &SurfaceCapabilitiesKHR) -> u32 {
        let mut min_image_count = capabilities.min_image_count.max(3);
        if capabilities.max_image_count > 0 && capabilities.max_image_count < min_image_count {
            min_image_count = capabilities.max_image_count;
        }

        min_image_count
    }

    fn choose_format<'a>(
        capabilities_loader: &ash::khr::get_surface_capabilities2::Instance,
        surface: &Surface,
        device: &Device,
    ) -> SurfaceFormat2KHR<'a> {
        let formats = unsafe {
            let surface_info = PhysicalDeviceSurfaceInfo2KHR::default().surface(surface.raw);

            let len = capabilities_loader
                .get_physical_device_surface_formats2_len(device.physical, &surface_info)
                .expect("Error getting surface formats len");

            let mut formats = vec![SurfaceFormat2KHR::default(); len];

            capabilities_loader
                .get_physical_device_surface_formats2(device.physical, &surface_info, &mut formats)
                .expect("Error getting surface formats");

            formats
        };

        *formats
            .iter()
            .find(|format| {
                format.surface_format.format == Format::B8G8R8A8_SRGB
                    && format.surface_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
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

    fn get_image_views(
        device: &Device,
        format: SurfaceFormatKHR,
        images: &[vk::Image],
    ) -> Vec<vk::ImageView> {
        let mut image_views = Vec::new();

        let mut image_view_info = vk::ImageViewCreateInfo::default()
            .view_type(ImageViewType::TYPE_2D)
            .format(format.format)
            .subresource_range(
                ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .layer_count(1)
                    .level_count(1),
            );

        for image in images {
            image_view_info = image_view_info.image(*image);
            let image = unsafe {
                device
                    .logical
                    .create_image_view(&image_view_info, None)
                    .expect("Error creating image view")
            };
            image_views.push(image);
        }

        image_views
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            for view in &self.image_views {
                self.device.logical.destroy_image_view(*view, None);
            }
            self.loader.destroy_swapchain(self.raw, None);
        }
    }
}
