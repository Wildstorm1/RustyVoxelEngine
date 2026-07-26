/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::frame::{ Frame, FrameError, CommandBufferArgs };
use super::frame_resources::{ VulkanFrameResources };
use super::command_buffer::{ VulkanCommandBuffer };
use super::hardware::device::{ VulkanDevice };
use std::sync::{ Arc };

/*
 * Implementation of frame for vulkan
 */
pub struct VulkanFrame {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanFrameResources>,
}

impl VulkanFrame {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: VulkanFrameResources,
    ) -> Self {
        Self {
            device: device,
            resources: Arc::new(resources),
        }
    }
}

impl Frame for VulkanFrame {
    type T = VulkanCommandBuffer;

    type E = VulkanFrameResources;

    fn get_resources(&self) -> Result<Arc<Self::E>, FrameError> {
        Ok(self.resources.clone())
    }

    fn create_command_buffer(&self, args: CommandBufferArgs) -> Result<Self::T, FrameError> {
        VulkanCommandBuffer::new(self.device.clone()).map_err(|err| {
            println!("Error creating command buffer {}", err);
            FrameError
        })
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), FrameError> {
        unimplemented!()
    }

    fn begin_frame(&self) {
        unimplemented!()
    }

    fn end_frame(&self) {
        unimplemented!()
    }

    fn present_frame(&self) {
        unimplemented!()
    }
}
