/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hardware::device::{ VulkanDevice };
use super::super::hardware::swapchain::{ VulkanSwapchain };
use std::sync::{ Arc };
use ash::{ vk };

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct RenderPassError;

/*
 * Vulkan implementation of a render pass
 */
pub struct VulkanRenderPass {
    device: Arc<VulkanDevice>,
    depth_image: vk::Image,
    depth_memory: vk::DeviceMemory,
    depth_view: vk::ImageView,
    pub render_pass: vk::RenderPass,
    pub frame_buffers: Vec<vk::Framebuffer>
}

fn find_supported_format(
    device: Arc<VulkanDevice>,
    candidates: Vec<vk::Format>,
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags
) -> Option<vk::Format> {
    for format in candidates {
        let props = unsafe { device.instance.instance.get_physical_device_format_properties(device.physical_device, format) };

        if tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features & features == features {
            return Some(format);
        }

        if tiling == vk::ImageTiling::OPTIMAL && props.optimal_tiling_features & features == features {
            return Some(format);
        }
    }

    None
}

fn find_depth_format(
    device: Arc<VulkanDevice>
) -> Option<vk::Format> {
    find_supported_format(
        device,
        vec![vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}

fn find_memory_type(
    device: Arc<VulkanDevice>,
    filter: u32,
    properties: vk::MemoryPropertyFlags
) -> Result<u32, RenderPassError> {
    let mem_properties = unsafe {
        device.instance.instance.get_physical_device_memory_properties(device.physical_device)
    };

    for i in 0..mem_properties.memory_types.len() {
        if filter & (1 << i) > 0 && (mem_properties.memory_types[i].property_flags & properties) == properties {
            return Ok(i as u32);
        }
    }

    Err(RenderPassError)
}

fn create_depth_image(
    device: Arc<VulkanDevice>,
    swap_chain_extent: vk::Extent2D,
    depth_format: vk::Format
) -> Result<(vk::Image, vk::DeviceMemory), RenderPassError> {
    let image_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D {
            width: swap_chain_extent.width,
            height: swap_chain_extent.height,
            depth: 1
        },
        mip_levels: 1,
        array_layers: 1,
        format: depth_format,
        tiling: vk::ImageTiling::OPTIMAL,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        samples: vk::SampleCountFlags::TYPE_1,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let depth_image= match unsafe { device.device.create_image(&image_info, None) } {
        Ok(depth_image) => depth_image,
        Err(err) => {
            println!("Could not create depth image! {}", err);
            return Err(RenderPassError);
        }
    };

    let memory_requirements = unsafe { device.device.get_image_memory_requirements(depth_image) };

    let memory_alloc_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index: find_memory_type(
            device.clone(),
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?,
        ..Default::default()
    };

    let depth_image_memory = match unsafe { device.device.allocate_memory(&memory_alloc_info, None) } {
        Ok(depth_image_memory) => depth_image_memory,
        Err(err) => {
            println!("Could not create depth image memory! {}", err);
            return Err(RenderPassError);
        }
    };

    match unsafe { device.device.bind_image_memory(depth_image, depth_image_memory, 0) } {
        Ok(_) => Ok((depth_image, depth_image_memory)),
        Err(err) => {
            println!("Could not bind depth image memory! {}", err);
            return Err(RenderPassError);
        }
    }
}

fn create_depth_image_view(
    device: Arc<VulkanDevice>,
    depth_image: vk::Image,
    depth_format: vk::Format
) -> Result<vk::ImageView, RenderPassError> {
    let create_info = vk::ImageViewCreateInfo {
        image: depth_image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: depth_format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    let image_view = match unsafe { device.device.create_image_view(&create_info, None) } {
        Ok(image_view) => image_view,
        Err(err) => {
            println!("Could not create depth image view! {}", err);
            return Err(RenderPassError);
        },
    };

    Ok(image_view)
}

fn create_render_pass(
    device: Arc<VulkanDevice>,
    depth_format: vk::Format,
    swapchain_format: vk::Format
) -> Result<vk::RenderPass, RenderPassError> {
    let depth_attachment = vk::AttachmentDescription {
        format: depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let color_attachment = vk::AttachmentDescription {
        format: swapchain_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: &depth_attachment_ref,
        ..Default::default()
    };

    let attachments = vec![ color_attachment, depth_attachment ];

    let dependency = vk::SubpassDependency {
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
        src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::BY_REGION,
        ..Default::default()
    };

    let render_pass_info = vk::RenderPassCreateInfo {
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &dependency,
        ..Default::default()
    };

    match unsafe { device.device.create_render_pass(&render_pass_info, None) } {
        Ok(render_pass) => Ok(render_pass),
        Err(err) => {
            println!("Could not create render pass! {}", err);
            return Err(RenderPassError);
        }
    }
}

fn create_frame_buffers(
    device: Arc<VulkanDevice>,
    render_pass: vk::RenderPass,
    swapchain_extent: vk::Extent2D,
    depth_image_view: vk::ImageView,
    swapchain_image_views: &Vec<vk::ImageView>,
) -> Result<Vec<vk::Framebuffer>, RenderPassError> {
    let mut frame_buffers = Vec::new();

    for i in 0..swapchain_image_views.len() {
        let attachments = vec![ swapchain_image_views[i], depth_image_view ];

        let frame_buffer_info = vk::FramebufferCreateInfo {
            render_pass: render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: swapchain_extent.width,
            height: swapchain_extent.height,
            layers: 1,
            ..Default::default()
        };

        match unsafe { device.device.create_framebuffer(&frame_buffer_info, None) } {
            Ok(frame_buffer) => frame_buffers.push(frame_buffer),
            Err(err) => {
                println!("Could not create {}th framebuffer! {}", i, err);
                return Err(RenderPassError);
            }
        };
    }

    Ok(frame_buffers)
}

impl VulkanRenderPass {
    pub fn new(
        device: Arc<VulkanDevice>,
        swapchain: &VulkanSwapchain
    ) -> Result<Self, RenderPassError> {
        let swap_chain_extent = swapchain.swap_chain_extent;
        let swap_chain_image_format = swapchain.swap_chain_image_format;
        let swap_chain_image_views = &swapchain.swap_chain_image_views;

        let depth_format = match find_depth_format(device.clone()) {
            Some(depth_format) => depth_format,
            None => {
                println!("Found no suitable format for depth image!");
                return Err(RenderPassError);
            }
        };

        let (depth_image, depth_image_memory) = create_depth_image(device.clone(), swap_chain_extent, depth_format)?;
        let depth_image_view= create_depth_image_view(device.clone(), depth_image, depth_format)?;
        let render_pass = create_render_pass(device.clone(), depth_format, swap_chain_image_format)?;
        let frame_buffers = create_frame_buffers(
            device.clone(),
            render_pass,
            swap_chain_extent,
            depth_image_view,
            swap_chain_image_views
        )?;

        Ok( Self {
            device: device,
            depth_image: depth_image,
            depth_memory: depth_image_memory,
            depth_view: depth_image_view,
            render_pass: render_pass,
            frame_buffers: frame_buffers
        } )
    }
}

impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_image(self.depth_image, None);
            self.device.device.free_memory(self.depth_memory, None);
            self.device.device.destroy_image_view(self.depth_view, None);

            self.frame_buffers.iter().for_each(|framebuffer| {
                self.device.device.destroy_framebuffer(*framebuffer, None);
            });

            self.device.device.destroy_render_pass(self.render_pass, None);
        }
    }
}