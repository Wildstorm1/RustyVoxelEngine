/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::context::{
    Context, ContextError, CommandBufferArgs
};
use super::super::hal::resources::{
    Buffer, Uniform
};
use super::resources::{ 
    VulkanResources
};
use super::command_buffer::{
    VulkanCommandBuffer
};
use super::hardware::device::{ VulkanDevice };
use super::hardware::swapchain::{ VulkanSwapchain };
use sdl3::video::Window;
use std::collections::{ HashMap };
use std::ffi::{ c_void };
use std::marker::{ PhantomData };
use std::sync::{ Arc, RwLock };
use ash::{ vk };

/*
 * Vulkan implementation of context
 */
pub struct VulkanContext<'a> {
    device: Arc<VulkanDevice>,
    swapchain: VulkanSwapchain,
    resources: VulkanResources,
    _lifetime: PhantomData<&'a ()>
}

impl <'a> VulkanContext<'a> {
    pub fn new(window: &Window) -> Result<Self, ContextError> {
        let device = unsafe { match VulkanDevice::new(&window) {
            Ok(device) => Arc::new(device),
            Err(err) => {
                println!("Error creating vulkan device {:?}", err);
                return Err(ContextError);
            }
        } };

        let swap_chain = unsafe { match VulkanSwapchain::new(&device, &window) {
            Ok(swap_chain) => swap_chain,
            Err(err) => {
                println!("Error creating vulkan swapchain {:?}", err);
                return Err(ContextError);
            }
        } };

        let resources = match VulkanResources::new(device.clone()) {
            Ok(resources) => resources,
            Err(err) => {
                println!("Error creating vulkan resources {:?}", err);
                return Err(ContextError);
            }
        };

        Ok( Self {
            device: device,
            swapchain: swap_chain,
            resources: resources,
            _lifetime: PhantomData
        } )
    }
}

impl <'a> Context<'a> for VulkanContext<'a> {
    type T = VulkanCommandBuffer<'a>;

    type E = VulkanResources;

    fn get_resources(&'a self) -> Result<&'a Self::E, ContextError> {
        Ok(&self.resources)
    }

    fn create_command_buffer(&'a self, args: CommandBufferArgs) -> Result<Self::T, ContextError> {
        match VulkanCommandBuffer::new(&self.device, self) {
            Ok(command_buffer) => Ok(command_buffer),
            Err(err) => {
                println!("Error creating command buffer {}", err);
                return Err(ContextError);
            }
        }
    }

    fn submit_command_buffer(&'a mut self, buffer: Self::T) -> Result<Self::T, ContextError> {
        unimplemented!()
    }

    fn begin_frame(&'a mut self) {
        unimplemented!()
    }

    fn end_frame(&'a mut self) {
        unimplemented!()
    }

    fn present_frame(&'a mut self) {
        unimplemented!()
    }
}