/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::instance::{VulkanInstance};
use std::collections::{HashMap, HashSet};
use std::ffi::{CStr, CString, c_void};
use ash::{vk, khr, ext, Device};
use sdl3::video::Window;

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct DeviceInitError;

/*
 * List of indices for relevant device queues
 */
struct DeviceQueueIndexes {
    graphics_index: Option<u32>,
    presentation_index: Option<u32>,
}

/*
 * List of indices for relevant device queues
 */
pub struct DeviceQueueMetadata {
    pub graphics_index: u32,
    pub graphics_queue: vk::Queue,
    pub presentation_index: u32,
    pub presentation_queue: vk::Queue,
}

/*
 * App specific device wrapper around vulkan
 */
pub struct VulkanDevice {
    pub instance: VulkanInstance,
    pub physical_device: vk::PhysicalDevice,
    pub surface: vk::SurfaceKHR,
    pub device: Device,
    pub queues: DeviceQueueMetadata,
    pub swapchain_extension: khr::swapchain::Device,
    pub mesh_shader_extension: ext::mesh_shader::Device,
    pub command_pool: vk::CommandPool,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

fn device_supports_extensions(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    extensions: HashSet<String>
) -> bool {
    let properties = unsafe { match instance.instance.enumerate_device_extension_properties(device_handle) {
        Ok(extensions) => extensions,
        Err(err) => {
            println!("Error getting device extension properties {}", err);
            return false;
        }
    } };

    let mut supported_extensions = HashSet::new();

    properties.iter().map(|extension| {
        let c_str = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };

        match c_str.to_str() {
            Ok(string) => string.to_string(),
            Err(err) => {
                println!("Error converting c string array to rust string {}", err);
                return "_invalid_extension_name_".to_string();
            }
        }
    }).filter(|extension| {
        extensions.contains(extension.as_str())
    }).for_each(|extension| {
        println!("\t Required Extension Supported: {}", extension);
        supported_extensions.insert(extension);
    });

    supported_extensions.len() == extensions.len()
}

fn device_supports_surface(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> bool {
    let formats = unsafe { match instance.surface.get_physical_device_surface_formats(device_handle, surface) {
        Ok(formats) => formats,
        Err(err) => {
            println!("Error getting surface formats {}", err);
            return false;
        },
    } };

    let present_modes = unsafe { match instance.surface.get_physical_device_surface_present_modes(device_handle, surface) {
        Ok(present_modes) => present_modes,
        Err(err) => {
            println!("Error getting surface present modes {}", err);
            return false;
        },
    } };

    formats.len() > 0 && present_modes.len() > 0
}

fn get_device_queues(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> DeviceQueueIndexes {
    let queue_family_properties = unsafe { instance.instance.get_physical_device_queue_family_properties(device_handle) };
    let mut graphics_queue = Option::None;
    let mut presentation_queue = Option::None;

    queue_family_properties.iter().enumerate().for_each(|(i, family_properties)| {
        let presentation_supported = unsafe { match instance.surface.get_physical_device_surface_support(device_handle, i as u32, surface) {
            Ok(supported) => supported,
            Err(err) => {
                println!("Error present queue support {}", err);
                false
            }
        } };

        if presentation_supported {
            presentation_queue = Some(i as u32);
        }

        if family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            graphics_queue = Some(i as u32);
        }
    });

    DeviceQueueIndexes {
        graphics_index: graphics_queue,
        presentation_index: presentation_queue
    }
}

fn select_physical_device(
    instance: &VulkanInstance,
    surface_handle: vk::SurfaceKHR,
    extensions: Vec<String>
) -> Result<vk::PhysicalDevice, DeviceInitError> {
    let physical_device_handles = unsafe {
        match instance.instance.enumerate_physical_devices() {
            Ok(devices) => devices,
            Err(_) => return Err(DeviceInitError),
        }
    };

    let mut device_scores = HashMap::new();

    for physical_device_handle in &physical_device_handles {
        let properties = unsafe {
            instance.instance.get_physical_device_properties(*physical_device_handle)
        };

        let _features = unsafe {
            instance.instance.get_physical_device_features(*physical_device_handle)
        };

        let device_name_u8_bytes = properties.device_name.iter()
            .map(|&c| c as u8)
            .filter(|&x| x > 0)
            .collect();

        println!("\t Found Device: {:?}", String::from_utf8(device_name_u8_bytes).unwrap());
        println!("\t\t Device Type: {:?}", properties.device_type);
        println!("\t\t Device Index Limit: {:?}", properties.limits.max_draw_indexed_index_value);

        device_scores.insert(*physical_device_handle, 0);

        // Safe to unwrap since we just inserted a handle
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            device_scores.insert(*physical_device_handle, device_scores.get(physical_device_handle).unwrap() + 1000);
        }

        if device_supports_extensions(&instance, *physical_device_handle, extensions.clone().into_iter().collect()) {
            device_scores.insert(*physical_device_handle, device_scores.get(physical_device_handle).unwrap() + 750);
        }

        if device_supports_surface(&instance, *physical_device_handle, surface_handle) {
            device_scores.insert(*physical_device_handle, device_scores.get(physical_device_handle).unwrap() + 100);
        }

        let device_queues = get_device_queues(&instance, *physical_device_handle, surface_handle);

        if device_queues.graphics_index.is_some() && device_queues.presentation_index.is_some() {
            device_scores.insert(*physical_device_handle, device_scores.get(physical_device_handle).unwrap() + 750);
        }

        println!("\t\t Device Score {:?}", device_scores.get(&physical_device_handle));
    }

    let mut best_score = 0;
    let mut best_device_handle = physical_device_handles[0];

    device_scores.iter().for_each(|(device, score)| {
        if *score > best_score {
            best_device_handle = *device;
            best_score = *score;
        }
    });

    Ok(best_device_handle)
}

fn create_descriptor_set_resources(
    device: &Device,
) -> Result<(
    vk::DescriptorPool,
    vk::DescriptorSetLayout,
    Vec<vk::DescriptorSet>
), DeviceInitError> {
    let model_descriptor_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        ..Default::default()
    };

    let voxels_descriptor_pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        ..Default::default()
    };

    let pool_sizes = [ model_descriptor_pool_size, voxels_descriptor_pool_size ];

    let descriptor_pool_create_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: 2,
        p_pool_sizes: pool_sizes.as_ptr(),
        flags: vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND,
        max_sets: 1,
        ..Default::default()
    };

    let descriptor_pool = match unsafe { device.create_descriptor_pool(&descriptor_pool_create_info, None) } {
        Ok(descriptor_pool) => descriptor_pool,
        Err(err) => {
            println!("Error creating descriptor pool {}", err);
            return Err(DeviceInitError);
        }
    };

    let flags = [
        vk::DescriptorBindingFlags::PARTIALLY_BOUND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND
    ];

    let binding_flags = vk::DescriptorSetLayoutBindingFlagsCreateInfo {
        p_binding_flags: flags.as_ptr(),
        binding_count: 2,
        ..Default::default()
    };

    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::MESH_EXT,
        ..Default::default()
    };

    let voxel_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 1,
        descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::MESH_EXT,
        ..Default::default()
    };

    let bindings = [ ubo_layout_binding, voxel_layout_binding ];

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 2,
        flags: vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL,
        p_bindings: bindings.as_ptr(),
        p_next: (&raw const binding_flags) as *const c_void,
        ..Default::default()
    };

    let descriptor_set_layout = match unsafe { device.create_descriptor_set_layout(&layout_info, None) } {
        Ok(descriptor_set_layout) => descriptor_set_layout,
        Err(err) => {
            println!("Error creating descriptor pool layout {}", err);
            return Err(DeviceInitError);
        }
    };

    let layouts = [ descriptor_set_layout ];

    let dset_alloc_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool: descriptor_pool,
        descriptor_set_count: 1,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };

    let descriptor_sets = match unsafe { device.allocate_descriptor_sets(&dset_alloc_info) } {
        Ok(descriptor_sets) => descriptor_sets,
        Err(err) => {
            println!("Error creating descriptor sets {}", err);
            return Err(DeviceInitError);
        }
    };

    Ok((descriptor_pool, descriptor_set_layout, descriptor_sets))
}

impl VulkanDevice {
    pub unsafe fn new(window: &Window) -> Result<Self, DeviceInitError> {
        println!("Starting vulkan device setup");

        let extensions = vec![
            "VK_KHR_swapchain",
            "VK_EXT_mesh_shader",
            "VK_EXT_descriptor_indexing",
            "VK_KHR_maintenance4"
        ];

        let extensions_strings: Vec<String> = extensions.iter()
            .map(|string| string.to_string())
            .collect();

        let extensions_c_strings: Vec<CString> = extensions.iter()
            .map(|string|
                CString::new(*string).expect("Rust string shouldn't contain nul byte")
            )
            .collect();

        let extensions_raw_c_strings: Vec<*const i8> = extensions_c_strings.iter()
            .map(|string|
                string.as_ptr() as *const i8
            )
            .collect();

        let instance = match unsafe { VulkanInstance::new() } {
            Ok(instance) => instance,
            Err(_) => return Err(DeviceInitError),
        };

        let surface_handle = unsafe {
            match window.vulkan_create_surface(instance.instance.handle()) {
                Ok(surface) => surface,
                Err(_) => return Err(DeviceInitError),
            }
        };

        let best_device_handle = select_physical_device(&instance, surface_handle, extensions_strings)?;
        let device_queues = get_device_queues(&instance, best_device_handle, surface_handle);

        let priority = 1.0f32;

        let queue_infos = [
            vk::DeviceQueueCreateInfo {
                queue_family_index: device_queues.graphics_index.unwrap(),
                queue_count: 1,
                p_queue_priorities: &priority,
                ..Default::default()
            },
            vk::DeviceQueueCreateInfo {
                queue_family_index: device_queues.presentation_index.unwrap(),
                queue_count: 1,
                p_queue_priorities: &priority,
                ..Default::default()
            }
        ];

        let mesh_shader_features = vk::PhysicalDeviceMeshShaderFeaturesEXT {
            mesh_shader: vk::TRUE,
            task_shader: vk::TRUE,
            ..Default::default()
        };

        let vulkan_1_3_features= vk::PhysicalDeviceVulkan13Features {
            p_next: (&raw const mesh_shader_features) as *mut c_void,
            maintenance4: vk::TRUE,
            ..Default::default()
        };

        let descriptor_indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures {
            p_next: (&raw const vulkan_1_3_features) as *mut c_void,
            descriptor_binding_uniform_buffer_update_after_bind: vk::TRUE,
            descriptor_binding_partially_bound: vk::TRUE,
            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures {
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            p_queue_create_infos: queue_infos.as_ptr(),
            queue_create_info_count: queue_infos.len() as u32,
            p_enabled_features: &device_features,
            enabled_extension_count: extensions_raw_c_strings.len() as u32,
            pp_enabled_extension_names: extensions_raw_c_strings.as_ptr(),
            p_next: (&raw const descriptor_indexing_features) as *const c_void,
            ..Default::default()
        };

        let logical_device = match unsafe { instance.instance.create_device(best_device_handle, &create_info, None) } {
            Ok(device) => device,
            Err(err) => {
                println!("Error creating logical device {}", err);
                return Err(DeviceInitError);
            }
        };

        let graphics_queue = unsafe {
            logical_device.get_device_queue(device_queues.graphics_index.unwrap(), 0)
        };

        let presentation_queue = unsafe {
            logical_device.get_device_queue(device_queues.presentation_index.unwrap(), 0)
        };

        let swapchain_device = khr::swapchain::Device::new(&instance.instance, &logical_device);

        let command_pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: device_queues.graphics_index.unwrap(),
            ..Default::default()
        };

        let command_pool = match unsafe { logical_device.create_command_pool(&command_pool_info, None) } {
            Ok(command_pool) => command_pool,
            Err(err) => {
                println!("Error creating command pool {}", err);
                return Err(DeviceInitError);
            },
        };

        let (descriptor_pool, descriptor_set_layout, descriptor_sets) = create_descriptor_set_resources(&logical_device)?;

        let mesh_shader_device = ext::mesh_shader::Device::new(&instance.instance, &logical_device);

        Ok ( Self {
            instance: instance,
            physical_device: best_device_handle,
            surface: surface_handle,
            device: logical_device,
            swapchain_extension: swapchain_device,
            mesh_shader_extension: mesh_shader_device,
            queues: DeviceQueueMetadata {
                graphics_index: device_queues.graphics_index.unwrap(),
                graphics_queue: graphics_queue,
                presentation_index: device_queues.presentation_index.unwrap(),
                presentation_queue: presentation_queue,
            },
            command_pool: command_pool,
            descriptor_pool: descriptor_pool,
            descriptor_set_layout: descriptor_set_layout,
            descriptor_sets: descriptor_sets,
        } )
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        println!("Destroying device wrapper");

        unsafe {
            println!("Destroying vulkan descriptor pool: {:?}", self.descriptor_pool);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            println!("Destroying vulkan command pool: {:?}", self.command_pool);
            self.device.destroy_command_pool(self.command_pool, None);
            println!("Destroying vulkan descriptor set layout: {:?}", self.descriptor_set_layout);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            println!("Destroying vulkan device");
            self.device.destroy_device(None);
            println!("Destroying vulkan surface");
            self.instance.surface.destroy_surface(self.surface, None);
        }
    }
}