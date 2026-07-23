/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::command_buffer::{
    CommandBuffer, CommandBufferError, DrawIndexedPacket, CopyBufferPacket
};
use super::context::{ VulkanContext };
use super::hardware::device::{ VulkanDevice };
use ash::{ vk };

/*
 * Internal type used to store multiple different packet types all in
 * the same vec
 */
enum InstructionPacket {
    DrawIndexed(DrawIndexedPacket),
    CopyBuffer(CopyBufferPacket),
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanCommandBuffer<'a> {
    device: &'a VulkanDevice,
    context: &'a VulkanContext<'a>,
    command_buffer: vk::CommandBuffer,
    buffer: Vec<InstructionPacket>,
    invalidated: bool,
}

impl <'a> VulkanCommandBuffer<'a> {
    pub fn new(
        device: &'a VulkanDevice,
        context: &'a VulkanContext<'a>
    ) -> Result<Self, CommandBufferError> {
        let allocate_info = vk::CommandBufferAllocateInfo {
            command_pool: device.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
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
            context: context,
            command_buffer: command_buffer[0],
            buffer: Vec::new(),
            invalidated: true,
        } )
    }
}

fn push_draw_indexed(
    device: &VulkanDevice,
    context: &VulkanContext,
    command_buffer: vk::CommandBuffer,
    packet: DrawIndexedPacket
) -> Result<(), CommandBufferError> {
    let vertex_buffer = match context.get_vulkan_buffer_handle(packet.vertex) {
        Ok(vertex_buffer) => match vertex_buffer {
            Some(buffer) => buffer,
            None => {
                println!("No vertex buffer with handle {} found!", packet.vertex);
                return Err(CommandBufferError);
            }
        },
        Err(err) => {
            println!("Error looking up vertex buffer with handle {}! {:?}", packet.vertex, err);
            return Err(CommandBufferError);
        }
    };

    let index_buffer = match context.get_vulkan_buffer_handle(packet.index) {
        Ok(index_buffer) => match index_buffer {
            Some(buffer) => buffer,
            None => {
                println!("No index buffer with handle {} found!", packet.index);
                return Err(CommandBufferError);
            }
        },
        Err(err) => {
            println!("Error looking up index buffer with handle {}! {:?}", packet.index, err);
            return Err(CommandBufferError);
        }
    };

    unsafe { device.device.cmd_bind_vertex_buffers(
        command_buffer, 0, &[vertex_buffer], &[0]
    ) };

    unsafe { device.device.cmd_bind_index_buffer(
        command_buffer, index_buffer, 0, vk::IndexType::UINT32
    ) };

    //self.device.device.cmd_bind_descriptor_sets(command_buffer, pipeline_bind_point, layout, first_set, descriptor_sets, dynamic_offsets);
    //self.device.device.cmd_draw_indexed(command_buffer, index_count, instance_count, first_index, vertex_offset, first_instance);

    Ok(())
}

fn push_copy_buffer(
    device: &VulkanDevice,
    context: &VulkanContext,
    command_buffer: vk::CommandBuffer,
    packet: CopyBufferPacket
) -> Result<(), CommandBufferError> {
    Ok(())
}

impl <'a> CommandBuffer for VulkanCommandBuffer<'a> {
    fn draw_indexed(&mut self, packet: DrawIndexedPacket) -> Result<(), CommandBufferError> {
        if self.invalidated {
            return Err(CommandBufferError);
        }

        self.buffer.push(InstructionPacket::DrawIndexed(packet));

        Ok(())
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
        let elements: Vec<InstructionPacket> = self.buffer.drain(..).collect();

        for packet in elements {
            match packet {
                InstructionPacket::DrawIndexed(packet) => push_draw_indexed(self.device, self.context, self.command_buffer, packet)?,
                InstructionPacket::CopyBuffer(packet) => push_copy_buffer(self.device, self.context, self.command_buffer, packet)?,
            }
        }

        Ok(())
    }
}