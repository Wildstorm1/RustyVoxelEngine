/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::utils::{c_string_or_error};
use std::ffi::{ CStr};
use std::collections::HashSet;
use ash::{vk, khr, Entry, Instance};
use sdl3::sys::{vulkan};

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct InstanceInitError;

/*
 * App specific instance of vulkan
 */
pub struct VulkanInstance {
    pub instance: Instance,
    pub surface: khr::surface::Instance
}

impl VulkanInstance {
    pub unsafe fn new() -> Result<Self, InstanceInitError> {
        println!("Starting vulkan instance setup");

        let entry = Entry::linked();

        let engine_name = c_string_or_error("NoEngine".to_string(), InstanceInitError)?;
        let application_name = c_string_or_error("Settle And Survive".to_string(), InstanceInitError)?;
        let app_info = vk::ApplicationInfo {
            application_version: vk::make_api_version(0, 0, 1, 0),
            engine_version: vk::make_api_version(0, 0, 1, 0),
            api_version: vk::API_VERSION_1_3,
            p_engine_name: engine_name.as_ptr(),
            p_application_name: application_name.as_ptr(),
            ..Default::default()
        };

        let mut count: u32 = 0;
        let extensions = unsafe {
            vulkan::SDL_Vulkan_GetInstanceExtensions(&mut count)
        };

        let available_layer_names = match unsafe { entry.enumerate_instance_layer_properties() } {
            Ok(available_layers) => available_layers.iter().map(|layer| {
                // Unsafe because I can't promise this is a valid pointer to a valid string in memory
                unsafe {
                    let layer_name = CStr::from_ptr(layer.layer_name.as_ptr());
                    layer_name.to_owned()
                }
            }).collect::<HashSet<_>>(),
            Err(e) => {
                println!("Failed to enumerate instance layer properties: {}", e);
                return Err(InstanceInitError);
            }
        };

        let validation_layer_name = c_string_or_error("VK_LAYER_KHRONOS_validation".to_string(), InstanceInitError)?;

        if !available_layer_names.contains(&validation_layer_name) {
            println!("Validation layer not available on machine");
            return Err(InstanceInitError);
        }

        let enabled_layers = vec![validation_layer_name.as_ptr()];

        let create_info;

        if cfg!(debug_assertions) {
            create_info = vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_extension_count: count,
                pp_enabled_extension_names: extensions,
                enabled_layer_count: enabled_layers.len() as u32,
                pp_enabled_layer_names: enabled_layers.as_ptr(),
                ..Default::default()
            };
        } else {
            create_info = vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_extension_count: count,
                pp_enabled_extension_names: extensions,
                ..Default::default()
            };
        }

        let instance = unsafe {
            entry.create_instance(&create_info, None).unwrap()
        };

        let surface_instance = khr::surface::Instance::new(&entry, &instance);

        Ok( Self {
            instance: instance,
            surface: surface_instance
        } )
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        println!("Destroying instance wrapper");

        unsafe {
            println!("Destroying vulkan instance");
            self.instance.destroy_instance(None);
        }
    }
}