/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::ffi::{CStr, CString};

use crate::graphics::vulkan::utils::{c_string_or_error};
use crate::graphics::vulkan::instance::{VulkanInstance};
use sdl3::sys::{vulkan};
use ash::{vk};
use sdl3::video::Window;

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct DeviceInitError;

/*
 * App specific device wrapper around vulkan
 */
pub struct VulkanDevice {
    instance: VulkanInstance,
}

impl VulkanDevice {
    pub unsafe fn new(window: &Window) -> Result<Self, DeviceInitError> {
        println!("Starting vulkan device setup");

        let instance = match unsafe { VulkanInstance::new() } {
            Ok(instance) => instance,
            Err(_) => return Err(DeviceInitError),
        };

        let surface = unsafe {
            match window.vulkan_create_surface(instance.instance.handle()) {
                Ok(surface) => surface,
                Err(_) => return Err(DeviceInitError),
            }
        };

        let physical_device_handles = unsafe {
            match instance.instance.enumerate_physical_devices() {
                Ok(devices) => devices,
                Err(_) => return Err(DeviceInitError),
            }
        };

        for physical_device_handle in physical_device_handles {
            let properties = unsafe {
                instance.instance.get_physical_device_properties(physical_device_handle)
            };

            let device_name_u8_bytes = properties.device_name.iter()
                .map(|&c| c as u8)
                .filter(|&x| x > 0)
                .collect();

            println!("\t Found Device: {:?}", String::from_utf8(device_name_u8_bytes).unwrap());
            println!("\t\t Device Type: {:?}", properties.device_type);
            println!("\t\t Device Index Limit: {:?}", properties.limits.max_draw_indexed_index_value);
        }

        Ok ( Self {
            instance: instance,
        } )
    }
}