use std::ffi::CStr;

use ash::vk::{self, PhysicalDevice, QueueFlags, TaggedStructure};

use crate::engine::{context::Context, surface::Surface};

const REQUIRED_DEVICE_EXTENSIONS: &[&CStr] = &[ash::khr::swapchain::NAME];

pub struct Queue {
    index: u32,
    family_index: u32,
    raw: vk::Queue,
}

impl Queue {
    pub const fn new(raw: vk::Queue, family_index: u32, index: u32) -> Self {
        Self {
            index,
            family_index,
            raw,
        }
    }
}

pub struct Device {
    queue: Queue,
    pub logical: ash::Device,
    pub physical: PhysicalDevice,
}

impl Device {
    pub fn new(context: &Context, surface: &Surface) -> Self {
        let physical = Self::query_physical_device(context);

        let (logical, queue) = Self::query_logical_device(context, physical, surface);

        Self {
            queue,
            logical,
            physical,
        }
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
                let mut properties = vk::PhysicalDeviceProperties2::default();

                context
                    .instance
                    .get_physical_device_properties2(**gpu, &mut properties);

                properties.properties.api_version >= vk::API_VERSION_1_3
            };

            let qfp = unsafe {
                let len = context
                    .instance
                    .get_physical_device_queue_family_properties2_len(**gpu);

                let mut qfp2 = vec![vk::QueueFamilyProperties2::default(); len];

                context
                    .instance
                    .get_physical_device_queue_family_properties2(**gpu, &mut qfp2);

                qfp2
            };
            let supports_graphics = qfp.iter().any(|queue_family| {
                queue_family
                    .queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            });

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

        physical.map_or_else(|| panic!("Couldn't find a suitable gpu"), |gpu| *gpu)
    }

    fn query_logical_device(
        context: &Context,
        physical: PhysicalDevice,
        surface: &Surface,
    ) -> (ash::Device, Queue) {
        let qfp = unsafe {
            let len = context
                .instance
                .get_physical_device_queue_family_properties2_len(physical);

            let mut qfp2 = vec![vk::QueueFamilyProperties2::default(); len];

            context
                .instance
                .get_physical_device_queue_family_properties2(physical, &mut qfp2);

            qfp2
        };

        let queue_family_index = u32::try_from(
            qfp.iter()
                .position(|family| {
                    family
                        .queue_family_properties
                        .queue_flags
                        .contains(QueueFlags::GRAPHICS)
                })
                .expect("Couldn't find a gpu with graphics capabilities"),
        )
        .unwrap();

        let supports_present = unsafe {
            surface
                .loader
                .get_physical_device_surface_support(physical, queue_family_index, surface.raw)
                .expect("Error querying surface support for the GPU")
        };

        assert!(
            supports_present,
            "GPU doesn't support present. What type of GPU are you using?"
        );

        let device_queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index as u32)
            .queue_priorities(&[0.5f32])];

        let mut vulkan11_features =
            vk::PhysicalDeviceVulkan11Features::default().shader_draw_parameters(true);
        let mut vulkan13_features =
            vk::PhysicalDeviceVulkan13Features::default().dynamic_rendering(true);
        let mut extended_dynamic_state_features =
            vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT::default()
                .extended_dynamic_state(true);

        let mut features2 = vk::PhysicalDeviceFeatures2::default()
            .push(&mut vulkan11_features)
            .push(&mut vulkan13_features)
            .push(&mut extended_dynamic_state_features);

        let extensions: Vec<*const i8> = REQUIRED_DEVICE_EXTENSIONS
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();
        let device_info = unsafe {
            vk::DeviceCreateInfo::default()
                .enabled_extension_names(&extensions)
                .queue_create_infos(&device_queue_info)
                .extend(&mut features2)
        };

        let device = unsafe {
            context
                .instance
                .create_device(physical, &device_info, None)
                .expect("Error creating logical device")
        };

        let raw_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        let queue = Queue::new(raw_queue, queue_family_index, 0);

        (device, queue)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.logical.destroy_device(None);
        }
    }
}
