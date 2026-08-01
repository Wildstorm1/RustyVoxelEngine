/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::super::hal::resources::{
    ResourcesError, Buffer, Uniform, BufferHandles, UniformHandles
};
use super::super::super::hal::stucts::{ UniformBufferObject };
use super::super::hardware::device::{ VulkanDevice };
use std::collections::HashMap;
use std::ffi::{ c_void };
use std::sync::{ Arc };
use ash::{ vk };

#[derive(Clone, Copy)]
pub struct BufferData {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub bytes: u64
}

#[derive(Clone, Copy)]
pub struct MappedBufferData {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub mapping: *mut c_void,
    pub bytes: u64
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanResources {
    device: Arc<VulkanDevice>,
    buffer_handles: HashMap<Buffer, BufferData>,
    staging_handles: HashMap<Buffer, MappedBufferData>,
    uniform_handles: HashMap<Uniform, MappedBufferData>,
    buffer_mappings: HashMap<BufferHandles, Buffer>,
    uniform_mappings: HashMap<UniformHandles, Uniform>,
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
) -> Result<BufferData, ResourcesError> {
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

    Ok(BufferData {
        buffer: buffer,
        memory: device_memory,
        bytes: size
    })
}

fn create_mapped_buffer(
    device: &VulkanDevice,
    size: u64,
    usage_flags: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<MappedBufferData, ResourcesError> {
    let buffer_data = create_buffer(&device, size, usage_flags, properties)?;

    let cpu_mapped_memory = match unsafe { device.device.map_memory(buffer_data.memory, 0, size, vk::MemoryMapFlags::empty()) } {
        Ok(mapped_memory) => mapped_memory,
        Err(err) => {
            println!("Error mapping memory to cpu space {}", err);
            return Err(ResourcesError);
        },
    };

    Ok(MappedBufferData {
        buffer: buffer_data.buffer,
        memory: buffer_data.memory,
        mapping: cpu_mapped_memory,
        bytes: size,
    })
}

fn create_staged_buffer(
    device: &VulkanDevice,
    usage_flag: vk::BufferUsageFlags,
    size: u64,
) -> Result<(BufferData, MappedBufferData), ResourcesError> {
    let usage_flags = usage_flag | vk::BufferUsageFlags::TRANSFER_DST;
    let properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;
    let gpu_buffer = create_buffer(&device, size, usage_flags, properties)?;

    let usage_flags = vk::BufferUsageFlags::TRANSFER_SRC;
    let properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
    let cpu_staging_buffer = create_mapped_buffer(&device, size, usage_flags, properties)?;

    return Ok((gpu_buffer, cpu_staging_buffer));
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
        let model_buffer_1 = create_mapped_buffer(&device, 2500000000, usage_flags, properties)?;
        bind_uniform_to_descriptor_set(&device, device.descriptor_sets[0], model_buffer_1.buffer);

        let usage_flags = vk::BufferUsageFlags::UNIFORM_BUFFER;
        let properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let model_buffer_2 = create_mapped_buffer(&device, 2500000000, usage_flags, properties)?;
        bind_uniform_to_descriptor_set(&device, device.descriptor_sets[1], model_buffer_2.buffer);

        let vertex_handle = Buffer { handle: 0 }; // Vertex Buffer
        let index_handle = Buffer { handle: 1 }; // Index Buffer
        let vertex_staging_handle = Buffer { handle: 2 }; // Vertex Staging Buffer
        let index_staging_handle = Buffer { handle: 3 }; // Index Staging Buffer
        let frame_1_uniform_handle = Uniform { handle: 0 }; // Frame 1 Models
        let frame_2_uniform_handle = Uniform { handle: 1 }; // Frame 2 Models

        let mut buffer_handles = HashMap::new();
        let mut staging_handles = HashMap::new();
        let mut uniform_handles = HashMap::new();

        buffer_handles.insert(vertex_handle.clone(), vertex_buffer.0);
        buffer_handles.insert(index_handle.clone(), index_buffer.0);
        staging_handles.insert(vertex_staging_handle.clone(), vertex_buffer.1);
        staging_handles.insert(index_staging_handle.clone(), index_buffer.1);
        uniform_handles.insert(frame_1_uniform_handle.clone(), model_buffer_1);
        uniform_handles.insert(frame_2_uniform_handle.clone(), model_buffer_2);

        let mut buffer_mappings = HashMap::new();
        let mut uniform_mappings = HashMap::new();

        buffer_mappings.insert(BufferHandles::VertexCPU, vertex_staging_handle);
        buffer_mappings.insert(BufferHandles::VertexGPU, vertex_handle);
        buffer_mappings.insert(BufferHandles::IndexCPU, index_staging_handle);
        buffer_mappings.insert(BufferHandles::IndexGPU, index_handle);
        uniform_mappings.insert(UniformHandles::ModelsFrame1CPU, frame_1_uniform_handle);
        uniform_mappings.insert(UniformHandles::ModelsFrame2CPU, frame_2_uniform_handle);

        Ok(Self {
            device: device,
            buffer_handles: buffer_handles,
            staging_handles: staging_handles,
            uniform_handles: uniform_handles,
            buffer_mappings: buffer_mappings,
            uniform_mappings: uniform_mappings,
        })
    }

    pub fn get_buffer_handle(&self, label: BufferHandles) -> Result<Buffer, ResourcesError> {
        let handle = self.buffer_mappings.get(&label);

        match handle {
            Some(handle) => Ok(*handle),
            None => Err(ResourcesError)
        }
    }

    pub fn get_uniform_handle(&self, label: UniformHandles) -> Result<Uniform, ResourcesError> {
        let handle = self.uniform_mappings.get(&label);

        match handle {
            Some(handle) => Ok(*handle),
            None => Err(ResourcesError)
        }
    }

    pub fn get_vulkan_buffer_handle(&self, label: BufferHandles) -> Result<vk::Buffer, ResourcesError> {
        let handle = self.get_buffer_handle(label)?;
        self.get_vulkan_buffer_handle_from_handle(handle)
    }

    pub fn get_vulkan_uniform_handle(&self, label: UniformHandles) -> Result<vk::Buffer, ResourcesError> {
        let handle = self.get_uniform_handle(label)?;
        self.get_vulkan_uniform_handle_from_handle(handle)
    }

    pub fn get_vulkan_buffer_handle_from_handle(&self, handle: Buffer) -> Result<vk::Buffer, ResourcesError> {
        let buffer_data = self.buffer_handles.get(&handle);
        let staging_data = self.staging_handles.get(&handle);

        match buffer_data {
            Some(buffer_data) => return Ok(buffer_data.buffer),
            _ => {}
        }

        match staging_data {
            Some(staging_data) => Ok(staging_data.buffer),
            None => Err(ResourcesError)
        }
    }

    pub fn get_vulkan_uniform_handle_from_handle(&self, handle: Uniform) -> Result<vk::Buffer, ResourcesError> {
        let buffer_data = self.uniform_handles.get(&handle);

        match buffer_data {
            Some(buffer_data) => Ok(buffer_data.buffer),
            None => Err(ResourcesError)
        }
    }

    pub fn get_vulkan_buffer_pointer(&self, label: BufferHandles) -> Result<*mut c_void, ResourcesError> {
        let handle = self.get_buffer_handle(label)?;
        self.get_vulkan_buffer_pointer_from_handle(handle)
    }

    pub fn get_vulkan_uniform_pointer(&self, label: UniformHandles) -> Result<*mut c_void, ResourcesError> {
        let handle = self.get_uniform_handle(label)?;
        self.get_vulkan_uniform_pointer_from_handle(handle)
    }

    pub fn get_vulkan_buffer_pointer_from_handle(&self, handle: Buffer) -> Result<*mut c_void, ResourcesError> {
        let staging_data = self.staging_handles.get(&handle);

        match staging_data {
            Some(staging_data) => Ok(staging_data.mapping),
            None => Err(ResourcesError)
        }
    }

    pub fn get_vulkan_uniform_pointer_from_handle(&self, handle: Uniform) -> Result<*mut c_void, ResourcesError> {
        let buffer_data = self.uniform_handles.get(&handle);

        match buffer_data {
            Some(buffer_data) => Ok(buffer_data.mapping),
            None => Err(ResourcesError)
        }
    }
}

impl Drop for VulkanResources {
    fn drop(&mut self) {
        println!("Destroying resources wrapper");

        unsafe {
            self.buffer_handles.values().into_iter().for_each(|handle_data| {
                println!("Destroying resource buffer: {:?}", handle_data.buffer);
                self.device.device.destroy_buffer(handle_data.buffer, None);
                println!("Destroying resource buffer memory: {:?}", handle_data.memory);
                self.device.device.free_memory(handle_data.memory, None);
            });

            self.staging_handles.values().into_iter().for_each(|handle_data| {
                println!("Destroying staging buffer: {:?}", handle_data.buffer);
                self.device.device.destroy_buffer(handle_data.buffer, None);
                println!("Destroying staging buffer memory: {:?}", handle_data.memory);
                self.device.device.free_memory(handle_data.memory, None);
            });

            self.uniform_handles.values().into_iter().for_each(|handle_data| {
                println!("Destroying uniform buffer: {:?}", handle_data.buffer);
                self.device.device.destroy_buffer(handle_data.buffer, None);
                println!("Destroying uniform buffer memory: {:?}", handle_data.memory);
                self.device.device.free_memory(handle_data.memory, None);
            });
        }
    }
}