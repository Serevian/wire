use std::{ffi::CStr, sync::Arc};

use crate::engine::{context::Context, device::Device};

mod context;
mod device;

const VALIDATION_LAYERS: &[&CStr] = &[c"VK_LAYER_KHRONOS_validation"];

pub struct Engine {
    device: Arc<Device>,
    context: Context,
}

impl Engine {
    pub fn new(required_extensions: &[*const i8]) -> Self {
        #[cfg(debug_assertions)]
        let enable_validation = true;
        #[cfg(not(debug_assertions))]
        let enable_validation = false;

        let context = Context::new(required_extensions, enable_validation, VALIDATION_LAYERS);

        let device = Arc::new(Device::new(&context));

        Self { device, context }
    }
}
