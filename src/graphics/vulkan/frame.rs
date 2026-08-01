/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::frame::{ Frame, FrameError };
use super::transfer_job::{ VulkanTransferJob };
use super::draw_job::{ VulkanDrawJob };
use super::frame_resources::{ VulkanFrameResources };
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
    transfer_job: Mutex<Option<VulkanTransferJob>>,
    draw_job: Mutex<Option<VulkanDrawJob>>,
    image_available: vk::Semaphore,
    work_complete: vk::Semaphore,
    transfer_complete: vk::Semaphore,
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
        transfer_complete: vk::Semaphore,
        work_in_flight: vk::Fence
    ) -> Result<Self, FrameError> {
        Ok(Self {
            device: device,
            swap_chain: swap_chain,
            pipeline: pipeline,
            resources: resources,
            transfer_job: Mutex::new(None),
            draw_job: Mutex::new(None),
            image_available: image_available,
            work_complete: work_complete,
            work_in_flight: work_in_flight,
            transfer_complete: transfer_complete,
        })
    }
}

impl Frame for VulkanFrame {
    type T = VulkanTransferJob;

    type E = VulkanDrawJob;

    fn create_transfer_job(&self) -> Result<Self::T, FrameError> {
        VulkanTransferJob::new(self.device.clone(), self.resources.clone(), self.transfer_complete).map_err(|err| {
            println!("Error creating transfer job {:?}", err);
            return FrameError;
        })
    }

    fn submit_transfer_job(&self, job: Self::T) -> Result<(), FrameError> {
        let mut transfer_job_guard = self.transfer_job.lock().map_err(|err| {
            println!("Error submitting transfer job {:?}", err);
            return FrameError;
        })?;

        *transfer_job_guard = Some(job);

        Ok(())
    }

    fn create_draw_job(&self) -> Result<Self::E, FrameError> {
        VulkanDrawJob::new(
            self.device.clone(),
            self.resources.clone(),
            self.swap_chain.clone(),
            self.pipeline.clone(),
            self.image_available,
            self.work_complete,
            self.transfer_complete,
            self.work_in_flight
        ).map_err(|err| {
            println!("Error creating draw job {:?}", err);
            return FrameError;
        })
    }

    fn submit_draw_job(&self, job: Self::E) -> Result<(), FrameError> {
        let mut draw_job_guard = self.draw_job.lock().map_err(|err| {
            println!("Error submitting draw job {:?}", err);
            return FrameError;
        })?;

        *draw_job_guard = Some(job);

        Ok(())
    }
}

impl Drop for VulkanFrame {
    fn drop(&mut self) {
        println!("Destroying frame");
    }
}
