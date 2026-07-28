/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::super::hal::resources::{
    ResourcesError, Buffer, Uniform
};
use super::super::super::hal::stucts::{ UniformBufferObject };
use super::super::hardware::device::{ VulkanDevice };
use std::ffi::{ c_void };
use std::sync::{ Arc };
use ash::{ vk };

pub struct BufferData {
    pub buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    pub staging_buffer: vk::Buffer,
    staging_memory: vk::DeviceMemory,
    pub mapping: *mut c_void
}

pub struct UniformData {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    pub mapping: *mut c_void
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanResources {
    device: Arc<VulkanDevice>,
    vertex_handle: Buffer,
    pub vertex_data: BufferData,
    index_handle: Buffer,
    pub index_data: BufferData,
    frame1_uniform_handle: Uniform,
    pub frame1_uniform_data: UniformData,
    frame2_uniform_handle: Uniform,
    pub frame2_uniform_data: UniformData,
}

fn find_memory_type(
    device: &VulkanDevice,
    filter: u32,
    properties: vk::MemoryPropertyFlags
) -> Result<u32, ResourcesError> {
    let mem_properties = unsafe {
        device.instance.instance.get_physical_device_memory_properties(device.physical_device)
    };

    for i in 0..mem_properties.memory_types.len() {
        if filter & (1 << i) > 0 && (mem_properties.memory_types[i].property_flags & properties) == properties {
            return Ok(i as u32);
        }
    }

    Err(ResourcesError)
}

fn create_buffer(
    device: &VulkanDevice,
    size: u64,
    usage_flags: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<(vk::Buffer, vk::DeviceMemory), ResourcesError> {
    let create_info = vk::BufferCreateInfo {
        size: size,
        usage: usage_flags,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let buffer = match unsafe { device.device.create_buffer(&create_info, None) } {
        Ok(buffer) => buffer,
        Err(err) => {
            println!("Error allocating vulkan buffer {}", err);
            return Err(ResourcesError);
        },
    };

    let memory_requirements = unsafe { device.device.get_buffer_memory_requirements(buffer) };
    let memory_type_index = find_memory_type(&device, memory_requirements.memory_type_bits, properties)?;

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index: memory_type_index,
        ..Default::default()
    };

    let device_memory = match unsafe { device.device.allocate_memory(&alloc_info, None) } {
        Ok(device_memory) => device_memory,
        Err(err) => {
            println!("Error allocating device memory {}", err);
            return Err(ResourcesError);
        },
    };

    match unsafe { device.device.bind_buffer_memory(buffer, device_memory, 0) } {
        Ok(_) => (),
        Err(err) => {
            println!("Error binding device memory {}", err);
            return Err(ResourcesError);
        }
    }

    Ok((buffer, device_memory))
}

fn create_mapped_buffer(
    device: &VulkanDevice,
    size: u64,
    usage_flags: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<(vk::Buffer, vk::DeviceMemory, *mut c_void), ResourcesError> {
    let (buffer, device_memory) = create_buffer(&device, size, usage_flags, properties)?;

    let cpu_mapped_memory = match unsafe { device.device.map_memory(device_memory, 0, size, vk::MemoryMapFlags::empty()) } {
        Ok(mapped_memory) => mapped_memory,
        Err(err) => {
            println!("Error mapping memory to cpu space {}", err);
            return Err(ResourcesError);
        },
    };

    Ok((buffer, device_memory, cpu_mapped_memory))
}

fn create_staged_buffer(
    device: &VulkanDevice,
    usage_flag: vk::BufferUsageFlags,
    size: u64,
) -> Result<BufferData, ResourcesError> {
    let usage_flags = usage_flag | vk::BufferUsageFlags::TRANSFER_DST;
    let properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let (gpu_buffer, gpu_device_memory) = create_buffer(&device, size, usage_flags, properties)?;

    let usage_flags = vk::BufferUsageFlags::TRANSFER_SRC;
    let properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let (cpu_staging_buffer, cpu_staging_device_memory, cpu_mapped_memory) = create_mapped_buffer(&device, size, usage_flags, properties)?;

    return Ok(BufferData {
        buffer: gpu_buffer,
        memory: gpu_device_memory,
        staging_buffer: cpu_staging_buffer,
        staging_memory: cpu_staging_device_memory,
        mapping: cpu_mapped_memory
    });
}

fn bind_uniform_to_descriptor_set(
    device: &VulkanDevice,
    descriptor_set: vk::DescriptorSet,
    buffer: vk::Buffer
) {
    let buffer_info = vk::DescriptorBufferInfo {
        buffer: buffer,
        offset: 0,
        range: std::mem::size_of::<UniformBufferObject>() as u64,
        ..Default::default()
    };

    let descriptor_write = vk::WriteDescriptorSet {
        dst_set: descriptor_set,
        dst_binding: 0,
        dst_array_element: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        descriptor_count: 1,
        p_buffer_info: &buffer_info,
        ..Default::default()
    };

    unsafe { device.device.update_descriptor_sets(&[ descriptor_write ], &[]) };
}

impl VulkanResources {
    pub fn new(device: Arc<VulkanDevice>) -> Result<Self, ResourcesError> {
        let vertex_buffer = create_staged_buffer(&device, vk::BufferUsageFlags::VERTEX_BUFFER, 2500000000)?;
        let index_buffer = create_staged_buffer(&device, vk::BufferUsageFlags::INDEX_BUFFER, 2500000000)?;

        let usage_flags = vk::BufferUsageFlags::UNIFORM_BUFFER;
        let properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let (model_buffer_1, model_buffer_memory_1, model_mapped_memory_1) = create_mapped_buffer(&device, 2500000000, usage_flags, properties)?;
        bind_uniform_to_descriptor_set(&device, device.descriptor_sets[0], model_buffer_1);

        let usage_flags = vk::BufferUsageFlags::UNIFORM_BUFFER;
        let properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let (model_buffer_2, model_buffer_memory_2, model_mapped_memory_2) = create_mapped_buffer(&device, 2500000000, usage_flags, properties)?;
        bind_uniform_to_descriptor_set(&device, device.descriptor_sets[1], model_buffer_2);

        Ok(Self {
            device: device,
            vertex_handle: Buffer {
                handle: 0
            },
            vertex_data: vertex_buffer,
            index_handle: Buffer {
                handle: 1
            },
            index_data: index_buffer,
            frame1_uniform_handle: Uniform {
                handle: 0
            },
            frame1_uniform_data: UniformData {
                buffer: model_buffer_1,
                memory: model_buffer_memory_1,
                mapping: model_mapped_memory_1,
            },
            frame2_uniform_handle: Uniform {
                handle: 1
            },
            frame2_uniform_data: UniformData {
                buffer: model_buffer_2,
                memory: model_buffer_memory_2,
                mapping: model_mapped_memory_2,
            },
        })
    }

    pub fn get_vertex_buffer_handle(&self) -> Result<Buffer, ResourcesError> {
        Ok(self.vertex_handle)
    }

    pub fn get_index_buffer_handle(&self) -> Result<Buffer, ResourcesError> {
        Ok(self.index_handle)
    }

    pub fn get_model_buffer_frame1_handle(&self) -> Result<Uniform, ResourcesError> {
        Ok(self.frame1_uniform_handle)
    }

    pub fn get_model_buffer_frame2_handle(&self) -> Result<Uniform, ResourcesError> {
        Ok(self.frame2_uniform_handle)
    }
}

impl Drop for VulkanResources {
    fn drop(&mut self) {
        unsafe {
            self.device.device.destroy_buffer(self.vertex_data.buffer, None);
            self.device.device.free_memory(self.vertex_data.memory, None);
    
            self.device.device.destroy_buffer(self.vertex_data.staging_buffer, None);
            self.device.device.free_memory(self.vertex_data.staging_memory, None);
        
            self.device.device.destroy_buffer(self.index_data.buffer, None);
            self.device.device.free_memory(self.index_data.memory, None);

            self.device.device.destroy_buffer(self.index_data.staging_buffer, None);
            self.device.device.free_memory(self.index_data.staging_memory, None);

            self.device.device.destroy_buffer(self.frame1_uniform_data.buffer, None);
            self.device.device.free_memory(self.frame1_uniform_data.memory, None);

            self.device.device.destroy_buffer(self.frame2_uniform_data.buffer, None);
            self.device.device.free_memory(self.frame2_uniform_data.memory, None);
        }
    }
}