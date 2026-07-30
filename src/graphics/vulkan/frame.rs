/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::frame::{ Frame, FrameError, CommandBufferArgs };
use super::frame_resources::{ VulkanFrameResources };
use super::command_buffer::{ VulkanCommandBuffer };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use super::application::pipeline::{ VulkanPipeline };
use std::sync::{ Arc, Mutex };
use ash::{ vk };

/*
 * Implementation of frame for vulkan
 */
pub struct VulkanFrame {
    device: Arc<VulkanDevice>,
    swap_chain: Arc<VulkanSwapchain>,
    resources: Arc<VulkanFrameResources>,
    pipeline: Arc<VulkanPipeline>,
    command_buffer: vk::CommandBuffer,
    submitted_command_buffers: Mutex<Vec<vk::CommandBuffer>>,
    image_available: vk::Semaphore,
    work_complete: vk::Semaphore,
    work_in_flight: vk::Fence
}

impl VulkanFrame {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        swap_chain: Arc<VulkanSwapchain>,
        pipeline: Arc<VulkanPipeline>,
        image_available: vk::Semaphore,
        work_complete: vk::Semaphore,
        work_in_flight: vk::Fence
    ) -> Result<Self, FrameError> {
        let frame_buffer_info = vk::CommandBufferAllocateInfo {
            command_pool: device.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_buffer =  match unsafe { device.device.allocate_command_buffers(&frame_buffer_info) } {
            Ok(buffers) => buffers[0],
            Err(err) => {
                println!("Error allocating frame command buffer {}", err);
                return Err(FrameError);
            }
        };

        Ok(Self {
            device: device,
            swap_chain: swap_chain,
            pipeline: pipeline,
            resources: resources,
            command_buffer: command_buffer,
            submitted_command_buffers: Mutex::new(Vec::new()),
            image_available: image_available,
            work_complete: work_complete,
            work_in_flight: work_in_flight
        })
    }
}

impl Frame for VulkanFrame {
    type T = VulkanCommandBuffer;

    type E = VulkanFrameResources;

    fn get_resources(&self) -> Result<Arc<Self::E>, FrameError> {
        Ok(self.resources.clone())
    }

    fn create_command_buffer(&self, args: CommandBufferArgs) -> Result<Self::T, FrameError> {
        VulkanCommandBuffer::new(
            self.device.clone(),
            self.resources.clone(),
            self.swap_chain.clone(),
            self.pipeline.clone()
        ).map_err(|err| {
            println!("Error creating command buffer {}", err);
            FrameError
        })
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), FrameError> {
        let mut submitted_buffers = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on command buffer is poisoned {}", err);
            FrameError
        })?;

        submitted_buffers.push(buffer.command_buffer);

        Ok(())
    }

    fn begin_frame(&self) -> Result<(), FrameError> {
        unsafe { self.device.device.reset_command_buffer(self.command_buffer, vk::CommandBufferResetFlags::empty()).map_err(|err| {
            println!("Error resetting command buffer {}", err);
            FrameError
        }) }?;

        let begin_info = vk::CommandBufferBeginInfo {
            ..Default::default()
        };

        unsafe { self.device.device.begin_command_buffer(self.command_buffer, &begin_info).map_err(|err| {
            println!("Error beginning command buffer {}", err);
            FrameError
        }) }?;

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [ 0.0f32, 0.0f32, 0.0f32, 1.0f32 ]
                }
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0f32,
                    stencil: 0
                }
            },
        ];

        let extent = self.swap_chain.swap_chain_extent;

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.resources.get_render_pass(),
            framebuffer: self.resources.get_frame_buffer(),
            render_area: vk::Rect2D {
                offset: vk::Offset2D {
                    x: 0,
                    y: 0
                },
                extent: extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.device.cmd_begin_render_pass(self.command_buffer, &render_pass_info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS) };

        Ok(())
    }

    fn end_frame(&self) -> Result<(), FrameError> {
        let submitted_command_buffers_guard = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on submitted command buffer is poisoned {}", err);
            FrameError
        })?;

        unsafe { self.device.device.cmd_execute_commands(self.command_buffer, &submitted_command_buffers_guard) };

        unsafe { self.device.device.cmd_end_render_pass(self.command_buffer); };

        unsafe { self.device.device.end_command_buffer(self.command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            FrameError
        }) }?;

        let wait_semaphores = [ self.image_available ];
        let signal_semaphores = [ self.work_complete ];
        let wait_stages = [ vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT ];
        let command_buffers = vec![ self.command_buffer ];

        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: 1,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.device.queue_submit(self.device.queues.graphics_queue, &[submit_info], self.work_in_flight).map_err(|err| {
            println!("Error submitting command buffer {}", err);
            FrameError
        }) }?;

        Ok(())
    }

    fn present_frame(&self) -> Result<(), FrameError> {
        let signal_semaphores = [ self.work_complete ];
        let swap_chains = [ self.swap_chain.swap_chain ];
        let image_indices = [ self.resources.get_presentation_index() ];

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swap_chains.as_ptr(),
            p_image_indices: image_indices.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.swapchain_extension.queue_present(self.device.queues.presentation_queue, &present_info).map_err(|err| {
            println!("Error presenting image {}", err);
            FrameError
        }) }?;

        Ok(())
    }
}

impl Drop for VulkanFrame {
    fn drop(&mut self) {
        unsafe {
            self.device.device.free_command_buffers(self.device.command_pool, &[self.command_buffer]);
        }

        match self.submitted_command_buffers.lock() {
            Ok(submitted_buffers) => {
                unsafe { self.device.device.free_command_buffers(self.device.command_pool, &submitted_buffers) };
            },
            Err(err) => {
                println!("Mutex on command buffer is poisoned {}", err);
            }
        }
    }
}