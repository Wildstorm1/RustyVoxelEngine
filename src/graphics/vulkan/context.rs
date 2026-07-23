/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::context::{
    Context, ContextError, BufferArgs, Buffer, CommandBufferArgs, BufferUsage, BufferMemoryProperties, UniformArgs, Uniform
};
use super::command_buffer::{
    VulkanCommandBuffer
};
use super::hardware::device::{ VulkanDevice };
use std::collections::{ HashMap };
use std::ffi::{ c_void };
use std::marker::{ PhantomData };
use std::sync::{ Arc, RwLock };
use ash::{ vk };

struct BufferData {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

struct UniformData {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapping: *mut c_void
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanContext<'a> {
    device: VulkanDevice,
    buffers: RwLock<HashMap<Buffer, BufferData>>,
    uniforms: RwLock<HashMap<Uniform, UniformData>>,
    _lifetime: PhantomData<&'a ()>
}

fn find_memory_type(
    device: &VulkanDevice,
    filter: u32,
    properties: vk::MemoryPropertyFlags
) -> Result<u32, ContextError> {
    let mem_properties = unsafe {
        device.instance.instance.get_physical_device_memory_properties(device.physical_device)
    };

    for i in 0..mem_properties.memory_types.len() {
        if filter & (1 << i) > 0 && (mem_properties.memory_types[i].property_flags & properties) == properties {
            return Ok(i as u32);
        }
    }

    Err(ContextError)
}

impl <'a> VulkanContext<'a> {
    pub fn new(device: VulkanDevice) -> Self {
        Self {
            device: device,
            buffers: RwLock::new(HashMap::new()),
            uniforms: RwLock::new(HashMap::new()),
            _lifetime: PhantomData
        }
    }

    pub fn get_vulkan_buffer_handle(&self, handle: Buffer) -> Result<Option<vk::Buffer>, ContextError> {
        let read_guard = match self.buffers.read() {
            Ok(read_guard) => read_guard,
            Err(err) => {
                println!("Buffer storage lock is poisoned {}", err);
                return Err(ContextError);
            },
        };

        match read_guard.get(&handle) {
            Some(buffer_info) => Ok(Some(buffer_info.buffer)),
            None => Ok(None)
        }
    }
}

fn create_buffer(
    device: &VulkanDevice,
    size: u64,
    usage_flags: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags
) -> Result<BufferData, ContextError> {
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
            return Err(ContextError);
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
            return Err(ContextError);
        },
    };

    match unsafe { device.device.bind_buffer_memory(buffer, device_memory, 0) } {
        Ok(_) => (),
        Err(err) => {
            println!("Error binding device memory {}", err);
            return Err(ContextError);
        }
    }

    Ok(BufferData {
        buffer: buffer,
        memory: device_memory
    })
}

impl <'a> Context<'a> for VulkanContext<'a> {
    type T = VulkanCommandBuffer<'a>;

    fn create_buffer(&'a self, args: BufferArgs) -> Result<Buffer, ContextError> {
        let mut usage_flags: vk::BufferUsageFlags = vk::BufferUsageFlags::empty();

        if args.usage.0 & BufferUsage::TRANSFER_DST > 0 {
            usage_flags = usage_flags | vk::BufferUsageFlags::TRANSFER_DST;
        }

        if args.usage.0 & BufferUsage::TRANSFER_SRC > 0 {
            usage_flags = usage_flags | vk::BufferUsageFlags::TRANSFER_SRC;
        }

        if args.usage.0 & BufferUsage::VERTEX_BUFFER > 0 {
            usage_flags = usage_flags | vk::BufferUsageFlags::VERTEX_BUFFER;
        }

        if args.usage.0 & BufferUsage::INDEX_BUFFER > 0 {
            usage_flags = usage_flags | vk::BufferUsageFlags::INDEX_BUFFER;
        }

        let mut properties: vk::MemoryPropertyFlags = vk::MemoryPropertyFlags::empty();

        if args.properties.0 & BufferMemoryProperties::DEVICE_LOCAL > 0 {
            properties = properties | vk::MemoryPropertyFlags::DEVICE_LOCAL;
        }

        if args.properties.0 & BufferMemoryProperties::HOST_VISIBLE > 0 {
            properties = properties | vk::MemoryPropertyFlags::HOST_VISIBLE;
        }

        if args.properties.0 & BufferMemoryProperties::HOST_COHERENT > 0 {
            properties = properties | vk::MemoryPropertyFlags::HOST_COHERENT;
        }

        let buffer_data = create_buffer(&self.device, args.size, usage_flags, properties)?;

        let mut write_guard = match self.buffers.write() {
            Ok(write_guard) => write_guard,
            Err(err) => {
                println!("Buffer storage lock is poisoned {}", err);
                return Err(ContextError);
            },
        };

        let handle = Buffer {
            handle: write_guard.len() as u64
        };

        write_guard.insert(handle, buffer_data);

        drop(write_guard);

        Ok(handle)
    }

    fn create_uniform(&'a self, args: UniformArgs) -> Result<Uniform, ContextError> {
        let mut usage_flags = vk::BufferUsageFlags::UNIFORM_BUFFER;
        let mut properties = vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let buffer_data = create_buffer(&self.device, args.size * args.count, usage_flags, properties)?;

        let mapped_memory = match unsafe { self.device.device.map_memory(buffer_data.memory, 0, args.size * args.count, vk::MemoryMapFlags::empty()) } {
            Ok(mapped_memory) => mapped_memory,
            Err(err) => {
                println!("Error mapping memory to cpu space {}", err);
                return Err(ContextError);
            },
        };

        let mut write_guard = match self.uniforms.write() {
            Ok(write_guard) => write_guard,
            Err(err) => {
                println!("Uniform storage lock is poisoned {}", err);
                return Err(ContextError);
            },
        };

        let handle = Uniform {
            handle: write_guard.len() as u64
        };

        write_guard.insert(handle, UniformData {
            buffer: buffer_data.buffer,
            memory: buffer_data.memory,
            mapping: mapped_memory
        });

        drop(write_guard);

        Ok(handle)
    }

    fn create_command_buffer(&'a self, args: CommandBufferArgs) -> Result<Self::T, ContextError> {
        match VulkanCommandBuffer::new(&self.device, self) {
            Ok(command_buffer) => Ok(command_buffer),
            Err(err) => {
                println!("Error creating command buffer {}", err);
                return Err(ContextError);
            }
        }
    }
}