/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::command_buffer::{
    CommandBuffer, CommandBufferError, DrawIndexedPacket, CopyBufferPacket
};
use super::super::hal::stucts::{ Camera, UniformBufferObject, Vertex };
use super::frame_resources::{ VulkanFrameResources };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use super::application::pipeline::{ VulkanPipeline };
use super::application::render_pass::{ VulkanRenderPass };
use std::sync::Arc;
use core::slice;
use ash::vk::PipelineInputAssemblyStateCreateInfo;
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
pub struct VulkanCommandBuffer {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    swap_chain: Arc<VulkanSwapchain>,
    render_pass: Arc<VulkanRenderPass>,
    pipeline: Arc<VulkanPipeline>,
    pub command_buffer: vk::CommandBuffer,
    buffer: Vec<InstructionPacket>,
    hacky_image_index: usize,
    invalidated: bool,
}

impl VulkanCommandBuffer {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        swap_chain: Arc<VulkanSwapchain>,
        render_pass: Arc<VulkanRenderPass>,
        pipeline: Arc<VulkanPipeline>,
        hacky_image_index: usize,
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
            render_pass: render_pass,
            pipeline: pipeline,
            command_buffer: command_buffer[0],
            buffer: Vec::new(),
            hacky_image_index: hacky_image_index,
            invalidated: false,
        } )
    }
}

fn push_draw_indexed(
    device: Arc<VulkanDevice>,
    command_buffer: vk::CommandBuffer,
    packet: DrawIndexedPacket
) -> Result<(), CommandBufferError> {
    /*
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
    */

    /*
    unsafe { device.device.cmd_bind_vertex_buffers(
        command_buffer, 0, &[vertex_buffer], &[0]
    ) };

    unsafe { device.device.cmd_bind_index_buffer(
        command_buffer, index_buffer, 0, vk::IndexType::UINT32
    ) };
    */

    //self.device.device.cmd_bind_descriptor_sets(command_buffer, pipeline_bind_point, layout, first_set, descriptor_sets, dynamic_offsets);
    //self.device.device.cmd_draw_indexed(command_buffer, index_count, instance_count, first_index, vertex_offset, first_instance);

    Ok(())
}

fn push_copy_buffer(
    device: Arc<VulkanDevice>,
    command_buffer: vk::CommandBuffer,
    packet: CopyBufferPacket
) -> Result<(), CommandBufferError> {
    Ok(())
}

fn push_face(
    vertex_0: glm::Vec3,
    vertex_1: glm::Vec3,
    vertex_2: glm::Vec3,
    vertex_3: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let index = vertices.len() as u32;
    vertices.push(Vertex { pos: vertex_0, color: glm::vec3(1.0f32, 0.0f32, 0.0f32) });
    vertices.push(Vertex { pos: vertex_1, color: glm::vec3(0.0f32, 1.0f32, 0.0f32) });
    vertices.push(Vertex { pos: vertex_2, color: glm::vec3(0.0f32, 0.0f32, 1.0f32) });
    vertices.push(Vertex { pos: vertex_3, color: glm::vec3(1.0f32, 1.0f32, 1.0f32) });
    indices.push(index);
    indices.push(index + 1);
    indices.push(index + 2);
    indices.push(index + 2);
    indices.push(index + 1);
    indices.push(index + 3);
}

fn push_right_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_front_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_bottom_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_left_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_back_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_top_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn create_cube(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = glm::vec3(0.0f32, 0.0f32, 0.0f32);
    push_right_face(v0, vertices, indices);
    push_left_face(v0, vertices, indices);
    push_front_face(v0, vertices, indices);
    push_back_face(v0, vertices, indices);
    push_top_face(v0, vertices, indices);
    push_bottom_face(v0, vertices, indices);
}

fn copy_cpu_staging_buffer_to_gpu(
    device: Arc<VulkanDevice>,
    staging_buffer: vk::Buffer,
    buffer: vk::Buffer,
    bytes: usize
) -> Result<(), CommandBufferError> {
    let alloc_info = vk::CommandBufferAllocateInfo {
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
        command_pool: device.command_pool,
        ..Default::default()
    };

    let command_buffer = unsafe { device.device.allocate_command_buffers(&alloc_info).map_err(|err| {
        println!("Error creating transfer command buffer {}", err);
        CommandBufferError
    }) }?[0];

    let begin_buffer_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };

    unsafe {
        device.device.begin_command_buffer(command_buffer, &begin_buffer_info).map_err(|err| {
            println!("Error beginning transfer command buffer {}", err);
            CommandBufferError
        })?;
    }

    let copy_region = vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: bytes as u64,
        ..Default::default()
    };

    unsafe { device.device.cmd_copy_buffer(command_buffer, staging_buffer, buffer, &[copy_region]) };
    unsafe { device.device.end_command_buffer(command_buffer).map_err(|err| {
        println!("Error ending transfer command buffer {}", err);
        CommandBufferError
    }) }?;

    let submit_info = vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: [ command_buffer ].as_ptr(),
        ..Default::default()
    };

    unsafe { device.device.queue_submit(device.queues.graphics_queue, &[submit_info], vk::Fence::null()).map_err(|err| {
        println!("Error submitting command buffer to queue {}", err);
        CommandBufferError
    }) }?;

    unsafe { device.device.queue_wait_idle(device.queues.graphics_queue).map_err(|err| {
        println!("Error waiting for queue to be idle {}", err);
        CommandBufferError
    }) }?;

    unsafe { device.device.free_command_buffers(device.command_pool, &[ command_buffer ]) };

    Ok(())
}


impl CommandBuffer for VulkanCommandBuffer {
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
        /*
        let elements: Vec<InstructionPacket> = self.buffer.drain(..).collect();

        for packet in elements {
            match packet {
                InstructionPacket::DrawIndexed(packet) => push_draw_indexed(self.device.clone(), self.command_buffer, packet)?,
                InstructionPacket::CopyBuffer(packet) => push_copy_buffer(self.device.clone(), self.command_buffer, packet)?,
            }
        }
        */

        let inheritance_info = vk::CommandBufferInheritanceInfo {
            render_pass: self.render_pass.render_pass,
            subpass: 0,
            framebuffer: self.render_pass.frame_buffers[self.hacky_image_index],
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


        let mut camera = Camera {
            view: glm::ext::look_at(
                glm::vec3(2.0f32, 2.0f32, 2.0f32),
                glm::vec3(0.0f32, 0.0f32, 0.0f32),
                glm::vec3(0.0f32, 0.0f32, 1.0f32)
            ),
            proj: glm::ext::perspective(
                glm::radians(45.0f32),
                extent.width as f32 / extent.height as f32,
                0.1f32,
                1000.0f32
            )
        };

        camera.proj[1][1] *= -1.0f32;

        let camera_bytes = unsafe {
            slice::from_raw_parts(
                &camera as *const Camera as *const u8,
                std::mem::size_of::<Camera>()
            )
        };

        unsafe {
            self.device.device.cmd_push_constants(
                self.command_buffer,
                self.pipeline.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                camera_bytes
            );
        }

        let model_matrix = UniformBufferObject {
            model: glm::mat4(
                1.0f32, 0.0f32, 0.0f32, 0.0f32,
                0.0f32, 1.0f32, 0.0f32, 0.0f32,
                0.0f32, 0.0f32, 1.0f32, 0.0f32,
                0.0f32, 0.0f32, 0.0f32, 1.0f32
            )
        };

        let uniform_pointer = self.resources.get_model_buffer_pointer().map_err(|err| {
            println!("Error getting model buffer pointer {:?}", err);
            CommandBufferError
        })?;

        unsafe {
            *(uniform_pointer as *mut UniformBufferObject) = model_matrix;
        }

        let descriptor_set = self.resources.get_descriptor_set().map_err(|err| {
            println!("Error getting model buffer pointer {:?}", err);
            CommandBufferError
        })?;

        unsafe {
            let dynamic_offset = [ 0 ];
            let descriptor_set = [ descriptor_set ];
            self.device.device.cmd_bind_descriptor_sets(
                self.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline_layout,
                0,
                &descriptor_set,
                &dynamic_offset
            );
        }

        let mut vertex_list = Vec::new();
        let mut index_list = Vec::new();
        create_cube(&mut vertex_list, &mut index_list);

        let vertex_pointer = self.resources.get_vertex_buffer_pointer().map_err(|err| {
            println!("Error getting vertex buffer pointer {:?}", err);
            CommandBufferError
        })? as *mut Vertex;

        let index_pointer = self.resources.get_index_buffer_pointer().map_err(|err| {
            println!("Error getting index buffer pointer {:?}", err);
            CommandBufferError
        })? as *mut u32;

        unsafe {
            std::ptr::copy_nonoverlapping(vertex_list.as_ptr(), vertex_pointer, vertex_list.len());
            std::ptr::copy_nonoverlapping(index_list.as_ptr(), index_pointer, index_list.len());
        }

        copy_cpu_staging_buffer_to_gpu(
            self.device.clone(),
            self.resources.get_vulkan_vertex_staging_buffer_handle().unwrap(),
            self.resources.get_vulkan_vertex_buffer_handle().unwrap(),
            std::mem::size_of::<Vertex>() * vertex_list.len()
        )?;

        copy_cpu_staging_buffer_to_gpu(
            self.device.clone(),
            self.resources.get_vulkan_index_staging_buffer_handle().unwrap(),
            self.resources.get_vulkan_index_buffer_handle().unwrap(),
            std::mem::size_of::<u32>() * index_list.len()
        )?;

        unsafe {
            self.device.device.cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                &[self.resources.get_vulkan_vertex_buffer_handle().unwrap()],
                &[0]
            )
        };

        unsafe {
            self.device.device.cmd_bind_index_buffer(
                self.command_buffer,
                self.resources.get_vulkan_index_buffer_handle().unwrap(),
                0,
                vk::IndexType::UINT32
            )
        };

        unsafe { self.device.device.cmd_draw_indexed(
            self.command_buffer,
            index_list.len() as u32,
            1,
            0,
            0,
            0
        ) };

        unsafe { self.device.device.end_command_buffer(self.command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            CommandBufferError
        }) }?;

        Ok(())
    }
}