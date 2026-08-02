use std::{ffi::CStr, sync::Arc};

use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::engine::{context::Context, device::Device, surface::Surface};

mod context;
mod device;
mod surface;
mod swapchain;

const VALIDATION_LAYERS: &[&CStr] = &[c"VK_LAYER_KHRONOS_validation"];

pub struct Engine {
    device: Arc<Device>,
    surface: Surface,
    context: Context,
}

impl Engine {
    pub fn new(
        required_extensions: &[*const i8],
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> Self {
        #[cfg(debug_assertions)]
        let enable_validation = true;
        #[cfg(not(debug_assertions))]
        let enable_validation = false;

        let context = Context::new(required_extensions, enable_validation, VALIDATION_LAYERS);

        let surface = Surface::new(&context, raw_display_handle, raw_window_handle);

        let device = Arc::new(Device::new(&context, &surface));

        Self {
            device,
            surface,
            context,
        }
    }
}
