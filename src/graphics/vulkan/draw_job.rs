/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::job::{ Job, JobError };
use super::hardware::device::{ VulkanDevice };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::application::pipeline::{ VulkanPipeline };
use super::command_buffer::{ VulkanCommandBuffer };
use super::frame_resources::{ VulkanFrameResources };
use std::sync::{ Arc, Mutex };
use ash::vk;

/*
 * Implementation of transfer job for vulkan
 */
pub struct VulkanDrawJob {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    submitted_command_buffers: Mutex<Vec<vk::CommandBuffer>>,
    graphics_command_buffer: vk::CommandBuffer,
    swap_chain: Arc<VulkanSwapchain>,
    pipeline: Arc<VulkanPipeline>,
    image_available: vk::Semaphore,
    work_complete: vk::Semaphore,
    transfer_complete: vk::Semaphore,
    work_in_flight: vk::Fence
}

impl VulkanDrawJob {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        swap_chain: Arc<VulkanSwapchain>,
        pipeline: Arc<VulkanPipeline>,
        image_available: vk::Semaphore,
        work_complete: vk::Semaphore,
        transfer_complete: vk::Semaphore,
        work_in_flight: vk::Fence
    ) -> Result<Self, JobError> {
        let frame_buffer_info = vk::CommandBufferAllocateInfo {
            command_pool: device.command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };

        let command_buffers =  match unsafe { device.device.allocate_command_buffers(&frame_buffer_info) } {
            Ok(buffers) => buffers,
            Err(err) => {
                println!("Error allocating frame command buffer {}", err);
                return Err(JobError);
            }
        };

        Ok(Self {
            device: device,
            resources: resources,
            submitted_command_buffers: Mutex::new(Vec::new()),
            graphics_command_buffer: command_buffers[0],
            swap_chain: swap_chain,
            pipeline: pipeline,
            image_available: image_available,
            work_complete: work_complete,
            transfer_complete: transfer_complete,
            work_in_flight: work_in_flight,
        })
    }
}

impl Job for VulkanDrawJob {
    type T = VulkanCommandBuffer;

    type E = VulkanFrameResources;

    fn get_resources(&self) -> Result<Arc<Self::E>, JobError> {
        Ok(self.resources.clone())
    }

    fn create_command_buffer(&self) -> Result<Self::T, JobError> {
        VulkanCommandBuffer::new(
            self.device.clone(),
            self.resources.clone(),
            self.swap_chain.clone(),
            self.pipeline.clone()
        ).map_err(|err| {
            println!("Error creating command buffer {}", err);
            JobError
        })
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), JobError> {
        let mut submitted_buffers = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on command buffer is poisoned {}", err);
            JobError
        })?;

        submitted_buffers.push(buffer.command_buffer);

        Ok(())
    }

    fn begin_job(&self) -> Result<(), JobError> {
        unsafe { self.device.device.reset_command_buffer(self.graphics_command_buffer, vk::CommandBufferResetFlags::empty()).map_err(|err| {
            println!("Error resetting command buffer {}", err);
            JobError
        }) }?;

        let begin_info = vk::CommandBufferBeginInfo {
            ..Default::default()
        };

        unsafe { self.device.device.begin_command_buffer(self.graphics_command_buffer, &begin_info).map_err(|err| {
            println!("Error beginning command buffer {}", err);
            JobError
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

        unsafe { self.device.device.cmd_begin_render_pass(self.graphics_command_buffer, &render_pass_info, vk::SubpassContents::SECONDARY_COMMAND_BUFFERS) };

        Ok(())
    }

    fn end_job(&self) -> Result<(), JobError> {
        let submitted_command_buffers_guard = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on submitted command buffer is poisoned {}", err);
            JobError
        })?;

        unsafe { self.device.device.cmd_execute_commands(self.graphics_command_buffer, &submitted_command_buffers_guard) };

        unsafe { self.device.device.cmd_end_render_pass(self.graphics_command_buffer); };

        unsafe { self.device.device.end_command_buffer(self.graphics_command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            JobError
        }) }?;

        let wait_semaphores = [ self.image_available, self.transfer_complete ];
        let signal_semaphores = [ self.work_complete ];
        let wait_stages = [ vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT ];
        let command_buffers = vec![ self.graphics_command_buffer ];

        let submit_info = vk::SubmitInfo {
            wait_semaphore_count: 2,
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
            JobError
        }) }?;

        Ok(())
    }

    fn run_job(&self) -> Result<(), JobError> {
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
            JobError
        }) }?;

        Ok(())
    }
}

impl Drop for VulkanDrawJob {
    fn drop(&mut self) {
        unsafe {
            self.device.device.free_command_buffers(self.device.command_pool, &[self.graphics_command_buffer]);
        }

        match self.submitted_command_buffers.lock() {
            Ok(submitted_buffers) => {
                if submitted_buffers.len() > 0 {
                    unsafe { self.device.device.free_command_buffers(self.device.command_pool, &submitted_buffers) };
                }
            },
            Err(err) => {
                println!("Mutex on command buffer is poisoned {}", err);
            }
        }
    }
}
