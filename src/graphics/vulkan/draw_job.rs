/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::job::{ Job, JobError };
use super::hardware::device::{ VulkanDevice };
use super::transfer_buffer::{ VulkanTransferCommandBuffer };
use super::frame_resources::{ VulkanFrameResources };
use std::sync::{ Arc, Mutex };
use ash::vk;

/*
 * Implementation of transfer job for vulkan
 */
pub struct VulkanDrawJob {}

impl VulkanDrawJob {
    pub fn new() -> Result<Self, JobError> {
        unimplemented!()
    }
}

impl Job for VulkanDrawJob {
    type T = VulkanTransferCommandBuffer;

    type E = VulkanFrameResources;

    fn get_resources(&self) -> Result<Arc<Self::E>, JobError> {
        unimplemented!()
    }

    fn create_command_buffer(&self) -> Result<Self::T, JobError> {
        unimplemented!()
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), JobError> {
        unimplemented!()
    }

    fn begin_job(&self) -> Result<(), JobError> {
        unimplemented!()
    }

    fn end_job(&self) -> Result<(), JobError> {
        unimplemented!()
    }

    fn run_job(&self) -> Result<(), JobError> {
        unimplemented!()
    }
}

impl Drop for VulkanDrawJob {
    fn drop(&mut self) {
        /*
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
        */
    }
}