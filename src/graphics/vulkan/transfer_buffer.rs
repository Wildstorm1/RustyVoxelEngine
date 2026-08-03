/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::command_buffer::{
    CommandBuffer, CommandBufferError, DrawPacket, CopyBufferPacket
};
use super::frame_resources::{ VulkanFrameResources };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use super::application::pipeline::{ VulkanPipeline };
use std::sync::Arc;
use ash::{ vk };

/*
 * Internal type used to store multiple different packet types all in
 * the same vec
 */
enum InstructionPacket {
    CopyBuffer(CopyBufferPacket),
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanTransferCommandBuffer {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    pub command_buffer: vk::CommandBuffer,
    buffer: Vec<InstructionPacket>,
    invalidated: bool,
}

impl VulkanTransferCommandBuffer {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
    ) -> Result<Self, CommandBufferError> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: device.command_pool,
            level: vk::CommandBufferLevel::SECONDARY,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_buffer = match unsafe { device.device.allocate_command_buffers(&allocate_info) } {
            Ok(command_buffer) => command_buffer,
            Err(err) => {
                println!("Error creating vulkan command buffer {}!", err);
                return Err(CommandBufferError);
            }
        };

        Ok( Self {
            device: device,
            resources: resources,
            command_buffer: command_buffer[0],
            buffer: Vec::new(),
            invalidated: false,
        } )
    }
}

fn push_copy_buffer(
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    command_buffer: vk::CommandBuffer,
    packet: CopyBufferPacket
) -> Result<(), CommandBufferError> {
    let copy_region = vk::BufferCopy {
        src_offset: packet.src_offset,
        dst_offset: packet.dst_offset,
        size: packet.bytes,
        ..Default::default()
    };

    let src_buffer = resources.get_vulkan_buffer_handle(packet.src).map_err(|err| {
        println!("Could not get vulkan buffer for src handle {} {:?}", packet.src, err);
        CommandBufferError
    })?;

    let dst_buffer = resources.get_vulkan_buffer_handle(packet.dst).map_err(|err| {
        println!("Could not get vulkan buffer for dst handle {} {:?}", packet.dst, err);
        CommandBufferError
    })?;

    unsafe {
        device.device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region])
    };

    Ok(())
}

impl CommandBuffer for VulkanTransferCommandBuffer {
    fn draw(&mut self, packet: DrawPacket) -> Result<(), CommandBufferError> {
        Err(CommandBufferError)
    }

    fn copy_buffer(&mut self, packet: CopyBufferPacket) -> Result<(), CommandBufferError> {
        if self.invalidated {
            return Err(CommandBufferError);
        }

        self.buffer.push(InstructionPacket::CopyBuffer(packet));

        Ok(())
    }

    fn stage(&mut self) -> Result<(), CommandBufferError> {
        if self.invalidated {
            return Err(CommandBufferError);
        }

        self.invalidated = true;

        let inheritance_info = vk::CommandBufferInheritanceInfo {
            ..Default::default()
        };

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: &inheritance_info,
            ..Default::default()
        };

        unsafe { self.device.device.begin_command_buffer(self.command_buffer, &begin_info).map_err(|err| {
            println!("Error beginning command buffer {}", err);
            CommandBufferError
        }) }?;

        let elements: Vec<InstructionPacket> = self.buffer.drain(..).collect();

        for packet in elements {
            match packet {
                InstructionPacket::CopyBuffer(packet) => push_copy_buffer(self.device.clone(), self.resources.clone(), self.command_buffer, packet)?,
            }
        }

        unsafe { self.device.device.end_command_buffer(self.command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            CommandBufferError
        }) }?;

        Ok(())
    }
}
