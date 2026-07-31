use std::ffi::{CStr, c_void};

use ash::{
    Entry,
    vk::{self, TaggedStructure},
};

#[cfg(debug_assertions)]
const REQUIRED_INSTANCE_EXTENSIONS: &[&CStr] = &[ash::ext::debug_utils::NAME];
#[cfg(not(debug_assertions))]
const REQUIRED_INSTANCE_EXTENSIONS: &[&CStr] = &[];

pub struct Context {
    entry: Entry,
    pub instance: ash::Instance,
    debug_utils_loader: ash::ext::debug_utils::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Context {
    pub fn new(
        required_extensions: &[*const i8],
        enable_validation: bool,
        validation_layers: &[&CStr],
    ) -> Self {
        let entry = unsafe { Entry::load().expect("Couldn't load vulkan entry") };

        let app_info = vk::ApplicationInfo::default()
            .application_name(c"Wire")
            .application_version(vk::make_api_version(1, 0, 0, 0))
            .engine_name(c"No Engine")
            .engine_version(vk::make_api_version(1, 0, 0, 0))
            .api_version(vk::API_VERSION_1_4);

        let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(required_extensions);

        let layer_fn_ptrs: Vec<*const i8> = validation_layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect();
        let extensions = Self::add_validation_extension(required_extensions);

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(Self::debug_callback));

        let has_layers =
            Self::check_validation_layers(&entry, enable_validation, validation_layers);
        if has_layers {
            create_info = create_info
                .enabled_layer_names(&layer_fn_ptrs)
                .enabled_extension_names(&extensions)
                .push(&mut debug_info);
        }

        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Error creating instance")
        };

        let debug_utils_loader = ash::ext::debug_utils::Instance::load(&entry, &instance);

        let debug_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .expect("failed to create debug messenger")
        };

        Self {
            entry,
            instance,
            debug_utils_loader,
            debug_messenger,
        }
    }

    fn check_validation_layers(
        entry: &Entry,
        enable_validation: bool,
        validation_layers: &[&CStr],
    ) -> bool {
        if !enable_validation {
            return false;
        }

        let supported_layers = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Error enumerating supported layers")
        };

        // Check if all required layers are supported
        validation_layers.iter().all(|val_layer| {
            supported_layers
                .iter()
                .any(|supported_layer| supported_layer.layer_name_as_c_str().unwrap() == val_layer)
        })
    }

    fn add_validation_extension(required_extensions: &[*const i8]) -> Vec<*const i8> {
        let mut array = required_extensions.to_vec();
        for ext in REQUIRED_INSTANCE_EXTENSIONS {
            array.push(ext.as_ptr());
        }

        array
    }

    unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut c_void,
    ) -> vk::Bool32 {
        unsafe {
            let callback_data = *p_callback_data;
            let message = if callback_data.p_message.is_null() {
                std::borrow::Cow::from("")
            } else {
                CStr::from_ptr(callback_data.p_message).to_string_lossy()
            };
            println!("[{:?}] [{:?}]: {}", message_severity, message_type, message);

            vk::FALSE
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.instance.destroy_instance(None);
        }
    }
}
