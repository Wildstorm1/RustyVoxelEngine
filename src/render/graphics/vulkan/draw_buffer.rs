/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::command_buffer::{
    CommandBuffer, CommandBufferError, DrawPacket, CopyBufferPacket
};
use super::super::hal::stucts::{ Camera, UniformBufferObject, Voxel };
use super::frame_resources::{ VulkanFrameResources };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use super::application::pipeline::{ VulkanPipeline };
use ash::{ vk };
use std::mem::{ size_of };
use std::slice;
use std::sync::Arc;

/*
 * Internal type used to store multiple different packet types all in
 * the same vec
 */
enum InstructionPacket {
    Draw(DrawPacket),
}

/*
 * Vulkan implementation of command buffer
 */
pub struct VulkanDrawCommandBuffer {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    swap_chain: Arc<VulkanSwapchain>,
    pipeline: Arc<VulkanPipeline>,
    pub command_buffer: vk::CommandBuffer,
    buffer: Vec<InstructionPacket>,
    invalidated: bool,
}

impl VulkanDrawCommandBuffer {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        swap_chain: Arc<VulkanSwapchain>,
        pipeline: Arc<VulkanPipeline>,
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
            swap_chain: swap_chain,
            pipeline: pipeline,
            command_buffer: command_buffer[0],
            buffer: Vec::new(),
            invalidated: false,
        } )
    }
}

fn push_draw(
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    command_buffer: vk::CommandBuffer,
    packet: DrawPacket
) -> Result<(), CommandBufferError> {
    let descriptor_set = resources.get_descriptor_set().map_err(|err| {
        println!("Error getting model buffer pointer {:?}", err);
        CommandBufferError
    })?;

    let pipeline_layout = resources.get_pipeline_layout();

    let camera_bytes = unsafe {
        slice::from_raw_parts(
            &packet.camera as *const Camera as *const u8,
            size_of::<Camera>()
        )
    };

    unsafe {
        device.device.cmd_push_constants(
            command_buffer,
            pipeline_layout,
            vk::ShaderStageFlags::MESH_EXT,
            0,
            camera_bytes
        );
    }

    unsafe {
        // TODO: Hardcoded chunk size
        device.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[ descriptor_set ],
            &[
                packet.model_uniform_index as u32 * device.derived_properties.ubo_alignment_size as u32,
                packet.chunk_index as u32 * 16 * 16 * 16 * size_of::<Voxel>() as u32
            ]
        );
    }

    unsafe { 
        device.mesh_shader_extension.cmd_draw_mesh_tasks(command_buffer, 1, 1, 1);
    };

    Ok(())
}

impl CommandBuffer for VulkanDrawCommandBuffer {
    fn draw(&mut self, packet: DrawPacket) -> Result<(), CommandBufferError> {
        if self.invalidated {
            return Err(CommandBufferError);
        }

        self.buffer.push(InstructionPacket::Draw(packet));

        Ok(())
    }

    fn copy_buffer(&mut self, packet: CopyBufferPacket) -> Result<(), CommandBufferError> {
        Err(CommandBufferError)
    }

    fn stage(&mut self) -> Result<(), CommandBufferError> {
        if self.invalidated {
            return Err(CommandBufferError);
        }

        self.invalidated = true;

        let inheritance_info = vk::CommandBufferInheritanceInfo {
            render_pass: self.resources.get_render_pass(),
            subpass: 0,
            framebuffer: self.resources.get_frame_buffer(),
            ..Default::default()
        };

        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE,
            p_inheritance_info: &inheritance_info,
            ..Default::default()
        };

        unsafe { self.device.device.begin_command_buffer(self.command_buffer, &begin_info).map_err(|err| {
            println!("Error beginning command buffer {}", err);
            CommandBufferError
        }) }?;

        unsafe { self.device.device.cmd_bind_pipeline(self.command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline.pipeline) };

        let extent = self.swap_chain.swap_chain_extent;

        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32,
            ..Default::default()
        };

        unsafe { self.device.device.cmd_set_viewport(self.command_buffer, 0, &[ viewport ]) };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0,
            },
            extent: extent,
            ..Default::default()
        };

        unsafe { self.device.device.cmd_set_scissor(self.command_buffer, 0, &[ scissor ]) };

        let elements: Vec<InstructionPacket> = self.buffer.drain(..).collect();

        for packet in elements {
            match packet {
                InstructionPacket::Draw(packet) => push_draw(self.device.clone(), self.resources.clone(), self.command_buffer, packet)?
            }
        }

        unsafe { self.device.device.end_command_buffer(self.command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            CommandBufferError
        }) }?;

        Ok(())
    }
}
