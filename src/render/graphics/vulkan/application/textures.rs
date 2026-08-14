/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use image::{ GenericImageView, ImageReader };
use super::super::hardware::device::{ VulkanDevice };
use std::sync::{ Arc };
use ash::{ vk };

#[derive(Clone, Copy)]
struct BufferData {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    bytes: u64
}

#[derive(Clone, Copy)]
struct TextureLayer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    size: u64
}

#[derive(Clone, Copy)]
struct ImageData {
    image: vk::Image,
    memory: vk::DeviceMemory,
    size: u64
}

/*
 * Simple struct for texture errors
 */
#[derive(Debug)]
pub struct TextureError;

/*
 * Vulkan implementation of textures
 */
pub struct VulkanTextures {
    device: Arc<VulkanDevice>,
    image: vk::Image,
    image_memory: vk::DeviceMemory,
    image_view: vk::ImageView,
    image_sampler: vk::Sampler
}

fn find_memory_type(
    device: &VulkanDevice,
    filter: u32,
    properties: vk::MemoryPropertyFlags
) -> Result<u32, TextureError> {
    let mem_properties = unsafe {
        device.instance.instance.get_physical_device_memory_properties(device.physical_device)
    };

    for i in 0..mem_properties.memory_types.len() {
        if filter & (1 << i) > 0 && (mem_properties.memory_types[i].property_flags & properties) == properties {
            return Ok(i as u32);
        }
    }

    Err(TextureError)
}

fn create_buffer(
    device: &VulkanDevice,
    size: u64,
    usage_flags: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<BufferData, TextureError> {
    let create_info = vk::BufferCreateInfo {
        size: size,
        usage: usage_flags,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let buffer = unsafe { device.device.create_buffer(&create_info, None).map_err(|err| {
        println!("Error allocating vulkan buffer {}", err);
        TextureError
    }) }?;

    let memory_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(&device, memory_requirements.memory_type_bits, properties)?;

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index: memory_type_index,
        ..Default::default()
    };

    let device_memory = unsafe { device.device.allocate_memory(&alloc_info, None).map_err(|err| {
        println!("Error allocating device memory {}", err);
        TextureError
    }) }?;

    unsafe { device.device.bind_buffer_memory(buffer, device_memory, 0).map_err(|err| {
        println!("Error binding device memory {}", err);
        TextureError
    }) }?;

    Ok(BufferData {
        buffer: buffer,
        memory: device_memory,
        bytes: size
    })
}

fn create_texture_image(
    device: &VulkanDevice,
    width: u32,
    height: u32
) -> Result<ImageData, TextureError> {
    let image_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D {
            width: width,
            height: height,
            depth: 1,
        },
        mip_levels: 3,
        array_layers: 1,
        format: vk::Format::R8G8B8A8_SRGB,
        tiling: vk::ImageTiling::OPTIMAL,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        samples: vk::SampleCountFlags::TYPE_1,
        flags: vk::ImageCreateFlags::empty(),
        ..Default::default()
    };

    let texture_image = unsafe { device.device.create_image(&image_info, None).map_err(|err| {
        println!("Error creating texture image {}", err);
        TextureError
    }) }?;

    let memory_requirements = unsafe { device.device.get_image_memory_requirements(texture_image) };

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index: find_memory_type(
            &device,
            memory_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?,
        ..Default::default()
    };

    let texture_image_memory = unsafe { device.device.allocate_memory(&alloc_info, None).map_err(|err| {
        println!("Error allocating texture image memory {}", err);
        TextureError
    }) }?;

    unsafe { device.device.bind_image_memory(texture_image, texture_image_memory, 0).map_err(|err| {
        println!("Error binding texture image memory {}", err);
        TextureError
    }) }?;

    Ok(ImageData {
        image: texture_image,
        memory: texture_image_memory,
        size: (width * height * 4) as u64
    })
}

// TODO: Should add into being a part of the api?
fn transfer_texture_to_gpu(
    device: &VulkanDevice,
    width: u32,
    height: u32,
    texture_image: vk::Image,
    image_buffer: vk::Buffer,
    mip_level: u32
) -> Result<(), TextureError> {
    let alloc_info = vk::CommandBufferAllocateInfo {
        level: vk::CommandBufferLevel::PRIMARY,
        command_pool: device.command_pool,
        command_buffer_count: 1,
        ..Default::default()
    };

    let command_buffer = unsafe { device.device.allocate_command_buffers(&alloc_info).map_err(|err| {
        println!("Could not create texture atlas transfer buffer! {}", err);
        TextureError
    }) }?[0];

    let begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };

    unsafe { device.device.begin_command_buffer(command_buffer, &begin_info).map_err(|err| {
        println!("Could not begin transfer buffer! {}", err);
        TextureError
    }) }?;

    let barrier = vk::ImageMemoryBarrier {
        old_layout: vk::ImageLayout::UNDEFINED,
        new_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: texture_image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: mip_level,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::TRANSFER_WRITE,
        ..Default::default()
    };

    unsafe { device.device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::TRANSFER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier]
    ) };

    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: mip_level,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D {
            x: 0,
            y: 0,
            z: 0
        },
        image_extent: vk::Extent3D {
            width: width,
            height: height,
            depth: 1
        },
        ..Default::default()
    };

    unsafe { device.device.cmd_copy_buffer_to_image(
        command_buffer,
        image_buffer,
        texture_image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region]
    ) };

    let barrier = vk::ImageMemoryBarrier {
        old_layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: texture_image,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: mip_level,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
        dst_access_mask: vk::AccessFlags::SHADER_READ,
        ..Default::default()
    };

    unsafe { device.device.cmd_pipeline_barrier(
        command_buffer,
        vk::PipelineStageFlags::TRANSFER,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier]
    ) };

    unsafe { device.device.end_command_buffer(command_buffer).map_err(|err| {
        println!("Could not end transfer buffer! {}", err);
        TextureError
    }) }?;

    let submit_info = vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        ..Default::default()
    };

    unsafe { device.device.queue_submit(device.queues.graphics_queue, &[submit_info], vk::Fence::null()).map_err(|err| {
        println!("Could not submit transfer buffer! {}", err);
        TextureError
    }) }?;

    unsafe { device.device.queue_wait_idle(device.queues.graphics_queue).map_err(|err| {
        println!("Error waiting for queue to be idle! {}", err);
        TextureError
    }) }?;

    unsafe { device.device.free_command_buffers(device.command_pool, &[command_buffer]) };

    Ok(())
}

fn create_texture_image_view(
    device: &VulkanDevice,
    texture_image: vk::Image
) -> Result<vk::ImageView, TextureError> {
    let view_info = vk::ImageViewCreateInfo {
        image: texture_image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::R8G8B8A8_SRGB,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 3,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    let image_view = unsafe { device.device.create_image_view(&view_info, None) }.map_err(|err| {
        println!("Error creating texture atlas image view! {}", err);
        TextureError
    })?;

    Ok(image_view)
}

fn create_texture_sampler(
    device: &VulkanDevice,
) -> Result<vk::Sampler, TextureError> {
    let properties = unsafe { device.instance.instance.get_physical_device_properties(device.physical_device) };

    let sampler_info = vk::SamplerCreateInfo {
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        anisotropy_enable: vk::TRUE,
        max_anisotropy: properties.limits.max_sampler_anisotropy,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        mip_lod_bias: 0.0f32,
        min_lod: 0.0f32,
        max_lod: vk::LOD_CLAMP_NONE,
        ..Default::default()
    };

    let sampler = unsafe { device.device.create_sampler(&sampler_info, None).map_err(|err| {
        println!("Error creating texture sampler! {}", err);
        TextureError
    }) }?;

    Ok(sampler)
}

fn bind_texture_to_descriptor_set(
    device: &VulkanDevice,
    image_view: vk::ImageView,
    sampler: vk::Sampler
) -> Result<(), TextureError> {
    let image_info = vk::DescriptorImageInfo {
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        image_view: image_view,
        sampler: sampler,
        ..Default::default()
    };

    let descriptor_write = vk::WriteDescriptorSet {
        dst_set: device.descriptor_sets[0],
        dst_binding: 2,
        dst_array_element: 0,
        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        descriptor_count: 1,
        p_image_info: &image_info,
        ..Default::default()
    };

    unsafe { device.device.update_descriptor_sets(&[ descriptor_write ], &[]) };

    Ok(())
}

fn read_texture_to_buffer(
    device: &VulkanDevice,
    path: String
) -> Result<TextureLayer, TextureError> {
    let reader = ImageReader::open(path).map_err(|err| {
        println!("Could not open texture atlas l0! {}", err);
        TextureError
    })?;

    let image = reader.decode().map_err(|err| {
        println!("Could not read texture atlas l0! {}", err);
        TextureError
    })?;

    let (width, height) = image.dimensions();
    let image_size = width as u64 * height as u64 * 4;

    let buffer = create_buffer(
        &device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
    )?;

    let cpu_texture = unsafe { device.device.map_memory(buffer.memory, 0, image_size, vk::MemoryMapFlags::empty()).map_err(|err| {
        println!("Error mapping texture memory to cpu space {}", err);
        TextureError
    }) }? as *mut u8;

    unsafe { std::ptr::copy_nonoverlapping(image.as_bytes().as_ptr(), cpu_texture, image_size as usize) };

    unsafe { device.device.unmap_memory(buffer.memory) };

    Ok(TextureLayer {
        buffer: buffer.buffer,
        memory: buffer.memory,
        width: width,
        height: height,
        size: image_size
    })
}

impl VulkanTextures {
    pub fn new(device: Arc<VulkanDevice>) -> Result<Self, TextureError> {
        let layer_0_mip = read_texture_to_buffer(&device, "./assets/voxels_l0.png".to_owned())?;
        let texture = create_texture_image(&device, layer_0_mip.width, layer_0_mip.height)?;

        transfer_texture_to_gpu(
            &device,
            layer_0_mip.width,
            layer_0_mip.height,
            texture.image,
            layer_0_mip.buffer,
            0
        )?;

        unsafe {
            device.device.destroy_buffer(layer_0_mip.buffer, None);
            device.device.free_memory(layer_0_mip.memory, None);
        }

        let layer_1_mip = read_texture_to_buffer(&device, "./assets/voxels_l1.png".to_owned())?;

        transfer_texture_to_gpu(
            &device,
            layer_1_mip.width,
            layer_1_mip.height,
            texture.image,
            layer_1_mip.buffer,
            1
        )?;

        unsafe {
            device.device.destroy_buffer(layer_1_mip.buffer, None);
            device.device.free_memory(layer_1_mip.memory, None);
        }

        let layer_2_mip = read_texture_to_buffer(&device, "./assets/voxels_l2.png".to_owned())?;

        transfer_texture_to_gpu(
            &device,
            layer_2_mip.width,
            layer_2_mip.height,
            texture.image,
            layer_2_mip.buffer,
            2
        )?;

        unsafe {
            device.device.destroy_buffer(layer_2_mip.buffer, None);
            device.device.free_memory(layer_2_mip.memory, None);
        }

        let image_view = create_texture_image_view(&device, texture.image)?;
        let sampler = create_texture_sampler(&device)?;
        bind_texture_to_descriptor_set(&device, image_view, sampler)?;

        Ok(Self {
            device: device,
            image: texture.image,
            image_memory: texture.memory,
            image_view: image_view,
            image_sampler: sampler
        })
    }
}

impl Drop for VulkanTextures {
    fn drop(&mut self) {
        println!("Destroying texture wrapper");

        unsafe {
            println!("Destroying texture atlas image: {:?}", self.image);
            self.device.device.destroy_image(self.image, None);
            println!("Destroying texture atlas image memory: {:?}", self.image_memory);
            self.device.device.free_memory(self.image_memory, None);
            println!("Destroying texture atlas image view: {:?}", self.image_view);
            self.device.device.destroy_image_view(self.image_view, None);
            println!("Destroying texture atlas sampler: {:?}", self.image_sampler);
            self.device.device.destroy_sampler(self.image_sampler, None);
        }
    }
}