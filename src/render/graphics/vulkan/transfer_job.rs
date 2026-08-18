/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::job::{ Job, JobError, JobErrorCode };
use super::hardware::device::{ VulkanDevice };
use super::transfer_buffer::{ VulkanTransferCommandBuffer };
use super::frame_resources::{ VulkanFrameResources };
use std::sync::{ Arc, Mutex };
use ash::vk;

/*
 * Implementation of transfer job for vulkan
 */
pub struct VulkanTransferJob {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
    submitted_command_buffers: Mutex<Vec<vk::CommandBuffer>>,
    sync_command_buffer: vk::CommandBuffer,
    transfer_complete: vk::Semaphore,
}

impl VulkanTransferJob {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        transfer_complete: vk::Semaphore,
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
                return Err(JobError { error: JobErrorCode::GENERIC });
            }
        };

        Ok(Self {
            device: device,
            resources: resources,
            submitted_command_buffers: Mutex::new(Vec::new()),
            sync_command_buffer: command_buffers[0],
            transfer_complete: transfer_complete,
        })
    }
}

impl Job for VulkanTransferJob {
    type T = VulkanTransferCommandBuffer;

    type E = VulkanFrameResources;

    fn get_resources(&self) -> Result<Arc<Self::E>, JobError> {
        Ok(self.resources.clone())
    }

    fn create_command_buffer(&self) -> Result<Self::T, JobError> {
        VulkanTransferCommandBuffer::new(
            self.device.clone(),
            self.resources.clone()
        ).map_err(|err| {
            println!("Error creating command buffer {}", err);
            JobError { error: JobErrorCode::GENERIC }
        })
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), JobError> {
        let mut submitted_buffers = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on command buffer is poisoned {}", err);
            JobError { error: JobErrorCode::GENERIC }
        })?;

        submitted_buffers.push(buffer.command_buffer);

        Ok(())
    }

    fn begin_job(&self) -> Result<(), JobError> {
        unsafe { self.device.device.reset_command_buffer(self.sync_command_buffer, vk::CommandBufferResetFlags::empty()).map_err(|err| {
            println!("Error resetting command buffer {}", err);
            JobError { error: JobErrorCode::GENERIC }
        }) }?;

        let begin_info = vk::CommandBufferBeginInfo {
            ..Default::default()
        };

        unsafe { self.device.device.begin_command_buffer(self.sync_command_buffer, &begin_info).map_err(|err| {
            println!("Error beginning command buffer {}", err);
            JobError { error: JobErrorCode::GENERIC }
        }) }?;

        Ok(())
    }

    fn end_job(&self) -> Result<(), JobError> {
        let submitted_command_buffers_guard = self.submitted_command_buffers.lock().map_err(|err| {
            println!("Mutex on submitted command buffer is poisoned {}", err);
            JobError { error: JobErrorCode::GENERIC }
        })?;

        unsafe { self.device.device.cmd_execute_commands(self.sync_command_buffer, &submitted_command_buffers_guard) };

        unsafe { self.device.device.end_command_buffer(self.sync_command_buffer).map_err(|err| {
            println!("Error ending transfer command buffer {}", err);
            JobError { error: JobErrorCode::GENERIC }
        }) }?;

        Ok(())
    }

    fn run_job(&self) -> Result<(), JobError> {
        let signal_semaphores = [ self.transfer_complete ];
        let command_buffers = [ self.sync_command_buffer ];

        let submit_info = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: command_buffers.as_ptr(),
            signal_semaphore_count: 1,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.device.queue_submit(self.device.queues.graphics_queue, &[submit_info], vk::Fence::null()).map_err(|err| {
            println!("Error submitting command buffer to queue {}", err);
            JobError { error: JobErrorCode::GENERIC }
        }) }?;

        Ok(())
    }
}

impl Drop for VulkanTransferJob {
    fn drop(&mut self) {
        //println!("Destroying transfer job");

        unsafe {
            self.device.device.free_command_buffers(self.device.command_pool, &[self.sync_command_buffer]);
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