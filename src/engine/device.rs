use std::ffi::CStr;

use ash::vk::{self, PhysicalDevice, QueueFlags, TaggedStructure};

use crate::engine::context::Context;

const REQUIRED_DEVICE_EXTENSIONS: &[&CStr] = &[ash::khr::swapchain::NAME];

pub struct Device {
    physical: PhysicalDevice,
}

impl Device {
    pub fn new(context: &Context) -> Self {
        let physical = Self::query_physical_device(context);

        Self { physical }
    }

    fn query_physical_device(context: &Context) -> PhysicalDevice {
        let gpus = unsafe {
            context
                .instance
                .enumerate_physical_devices()
                .expect("Error enumerating GPUs")
        };

        let physical = gpus.iter().find(|gpu| {
            let vulkan13 = unsafe {
                context
                    .instance
                    .get_physical_device_properties(**gpu)
                    .api_version
                    >= vk::API_VERSION_1_3
            };

            let qfp = unsafe {
                context
                    .instance
                    .get_physical_device_queue_family_properties(**gpu)
            };
            let supports_graphics = qfp
                .iter()
                .any(|queue_family| queue_family.queue_flags.contains(QueueFlags::GRAPHICS));

            let supported_extensions = unsafe {
                context
                    .instance
                    .enumerate_device_extension_properties(**gpu)
                    .expect("Couldn't enumerate device extensions")
            };
            let supports_extensions = REQUIRED_DEVICE_EXTENSIONS.iter().all(|device_ext| {
                supported_extensions.iter().any(|supported_ext| {
                    supported_ext.extension_name_as_c_str().unwrap() == device_ext
                })
            });

            let mut vulkan11_features = vk::PhysicalDeviceVulkan11Features::default();
            let mut vulkan13_features = vk::PhysicalDeviceVulkan13Features::default();
            let mut extended_dynamic_state_features =
                vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default();

            let mut features2 = vk::PhysicalDeviceFeatures2::default()
                .push(&mut vulkan11_features)
                .push(&mut vulkan13_features)
                .push(&mut extended_dynamic_state_features);

            unsafe {
                context
                    .instance
                    .get_physical_device_features2(**gpu, &mut features2);
            }

            let supports_required_features = vulkan11_features.shader_draw_parameters == vk::TRUE
                && vulkan13_features.dynamic_rendering == vk::TRUE
                && extended_dynamic_state_features.extended_dynamic_state == vk::TRUE;

            vulkan13 && supports_graphics && supports_extensions && supports_required_features
        });

        if let Some(gpu) = physical {
            *gpu
        } else {
            panic!("Couldn't find a suitable gpu")
        }
    }
}
