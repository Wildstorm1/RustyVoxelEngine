/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::instance::{ VulkanInstance };
use super::device::{ VulkanDevice };
use ash::{ vk };
use sdl3::video::Window;
use std::sync::{ Arc };
use std::ptr;

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct SwapchainInitError;

/*
 * App specific swapchain wrapper around vulkan
 */
pub struct VulkanSwapchain {
    device: Arc<VulkanDevice>,
    pub swap_chain: vk::SwapchainKHR,
    pub swap_chain_images: Vec<vk::Image>,
    pub swap_chain_image_format: vk::Format,
    pub swap_chain_extent: vk::Extent2D,
    pub swap_chain_image_views: Vec<vk::ImageView>,
}

fn choose_swap_surface_format(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> Result<vk::SurfaceFormatKHR, SwapchainInitError> {
    let formats = unsafe { match instance.surface.get_physical_device_surface_formats(device_handle, surface) {
        Ok(formats) => formats,
        Err(err) => {
            println!("Error getting surface formats {}", err);
            return Err(SwapchainInitError);
        },
    } };

    let valid_formats: Vec<vk::SurfaceFormatKHR> = formats.into_iter().filter(|available_format| {
        return available_format.format == vk::Format::B8G8R8A8_SRGB && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR;
    }).collect();

    if valid_formats.len() == 0 {
        println!("No valid surface formats found!");
        return Err(SwapchainInitError);
    }

    Ok(valid_formats[0])
}

fn choose_swap_surface_present_mode(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> Result<vk::PresentModeKHR, SwapchainInitError> {
    let present_modes = unsafe { match instance.surface.get_physical_device_surface_present_modes(device_handle, surface) {
        Ok(present_modes) => present_modes,
        Err(err) => {
            println!("Error getting surface present modes {}", err);
            return Err(SwapchainInitError);
        },
    } };

    let valid_modes: Vec<vk::PresentModeKHR> = present_modes.into_iter().filter(|available_mode| {
        return *available_mode == vk::PresentModeKHR::MAILBOX;
    }).collect();

    if valid_modes.len() == 0 {
        println!("No valid surface modes found!");
        return Err(SwapchainInitError);
    }

    Ok(valid_modes[0])
}

fn choose_swap_extent(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    window: &Window
) -> Result<vk::Extent2D, SwapchainInitError> {
    let capabilities = unsafe { match instance.surface.get_physical_device_surface_capabilities(device_handle, surface) {
        Ok(capabilities) => capabilities,
        Err(err) => {
            println!("Error getting surface capabilities {}", err);
            return Err(SwapchainInitError);
        },
    } };

    if capabilities.current_extent.width != u32::MAX {
        return Ok(capabilities.current_extent);
    }

    let window_size = window.size_in_pixels();
    let mut extent = vk::Extent2D {
        width: window_size.0,
        height: window_size.1,
    };

    extent.width = extent.width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width);
    extent.height = extent.height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height);

    Ok(extent)
}

fn choose_image_count(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> Result<u32, SwapchainInitError> {
    let capabilities = unsafe { match instance.surface.get_physical_device_surface_capabilities(device_handle, surface) {
        Ok(capabilities) => capabilities,
        Err(err) => {
            println!("Error getting surface capabilities {}", err);
            return Err(SwapchainInitError);
        },
    } };

    let mut image_count = capabilities.min_image_count + 1;

    if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
        image_count = capabilities.max_image_count;
    }

    Ok(image_count)
}

fn choose_transform(
    instance: &VulkanInstance,
    device_handle: vk::PhysicalDevice,
    surface: vk::SurfaceKHR
) -> Result<vk::SurfaceTransformFlagsKHR, SwapchainInitError> {
    let capabilities = unsafe { match instance.surface.get_physical_device_surface_capabilities(device_handle, surface) {
        Ok(capabilities) => capabilities,
        Err(err) => {
            println!("Error getting surface capabilities {}", err);
            return Err(SwapchainInitError);
        },
    } };

    Ok(capabilities.current_transform)
}

fn create_image_views(
    device: Arc<VulkanDevice>,
    swap_chain_images: &Vec<vk::Image>,
    swap_chain_image_format: vk::Format
) -> Result<Vec<vk::ImageView>, SwapchainInitError> {
    let mut image_views = Vec::new();

    for image in swap_chain_images {
        let image_view_create_info = vk::ImageViewCreateInfo {
            image: *image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: swap_chain_image_format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        let image_view = match unsafe { device.device.create_image_view(&image_view_create_info, None) } {
            Ok(image_view) => image_view,
            Err(err) => {
                println!("Error creating image view for swapchain image {}", err);
                return Err(SwapchainInitError)
            },
        };

        image_views.push(image_view);
    }

    Ok(image_views)
}

impl VulkanSwapchain {
    pub unsafe fn new(
        device: Arc<VulkanDevice>,
        window: &Window
    ) -> Result<Self, SwapchainInitError> {
        let surface_format = choose_swap_surface_format(&device.instance, device.physical_device, device.surface)?;
        let present_mode = choose_swap_surface_present_mode(&device.instance, device.physical_device, device.surface)?;
        let extent = choose_swap_extent(&device.instance, device.physical_device, device.surface, window)?;
        let image_count = choose_image_count(&device.instance, device.physical_device, device.surface)?;
        let transform = choose_transform(&device.instance, device.physical_device, device.surface)?;

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR {
            surface: device.surface,
            min_image_count: image_count,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            pre_transform: transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: present_mode,
            clipped: vk::TRUE,
            ..Default::default()
        };

        let queue_family_indices = vec![ device.queues.graphics_index, device.queues.presentation_index ];

        if queue_family_indices[0] != queue_family_indices[1] {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            swapchain_create_info.queue_family_index_count = 2;
            swapchain_create_info.p_queue_family_indices = queue_family_indices.as_ptr();
        } else {
            swapchain_create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
            swapchain_create_info.queue_family_index_count = 2;
            swapchain_create_info.p_queue_family_indices = ptr::null();
        }

        let swapchain = match unsafe { device.swapchain_extension.create_swapchain(&swapchain_create_info, None) } {
            Ok(swapchain) => swapchain,
            Err(err) => {
                println!("Error creating swapchain {}", err);
                return Err(SwapchainInitError);
            }
        };

        let swapchain_images = match unsafe { device.swapchain_extension.get_swapchain_images(swapchain) } {
            Ok(swapchain_images) => swapchain_images,
            Err(err) => {
                println!("Error creating swapchain images {}", err);
                return Err(SwapchainInitError);
            }
        };

        let image_views = create_image_views(device.clone(), &swapchain_images, surface_format.format)?;

        Ok( Self {
            device: device,
            swap_chain: swapchain,
            swap_chain_images: swapchain_images,
            swap_chain_image_format: surface_format.format,
            swap_chain_extent: extent,
            swap_chain_image_views: image_views,
        } )
    }
}


impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            self.swap_chain_image_views.iter().for_each(|view| {
                self.device.device.destroy_image_view(*view, None);
            });

            self.device.swapchain_extension.destroy_swapchain(self.swap_chain, None);
        }
    }
}