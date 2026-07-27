/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::vulkan::application::{pipeline, render_pass};

use super::super::hal::context::{
    Context, ContextError
};
use super::application::resources::{ VulkanResources };
use super::frame_resources::{ VulkanFrameResources };
use super::hardware::device::{ VulkanDevice };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::application::render_pass::{ VulkanRenderPass };
use super::application::pipeline::{ VulkanPipeline };
use super::frame::VulkanFrame;
use sdl3::video::Window;
use std::sync::{ Arc };

/*
 * Vulkan implementation of context
 */
pub struct VulkanContext {
    device: Arc<VulkanDevice>,
    swapchain: Arc<VulkanSwapchain>,
    resources: Arc<VulkanResources>,
    render_pass: Arc<VulkanRenderPass>,
    pipeline: Arc<VulkanPipeline>,
    frame_number: usize,
    frames: Vec<Arc<VulkanFrame>>,
}

impl VulkanContext {
    pub fn new(window: &Window) -> Result<Self, ContextError> {
        let device = unsafe { match VulkanDevice::new(&window) {
            Ok(device) => Arc::new(device),
            Err(err) => {
                println!("Error creating vulkan device {:?}", err);
                return Err(ContextError);
            }
        } };

        let swap_chain = unsafe { match VulkanSwapchain::new(device.clone(), &window) {
            Ok(swap_chain) => Arc::new(swap_chain),
            Err(err) => {
                println!("Error creating vulkan swapchain {:?}", err);
                return Err(ContextError);
            }
        } };

        let resources = match VulkanResources::new(device.clone()) {
            Ok(resources) => Arc::new(resources),
            Err(err) => {
                println!("Error creating vulkan resources {:?}", err);
                return Err(ContextError);
            }
        };

        let render_pass = match VulkanRenderPass::new(device.clone(), &swap_chain) {
            Ok(render_pass) => Arc::new(render_pass),
            Err(err) => {
                println!("Error creating vulkan render pass {:?}", err);
                return Err(ContextError);
            }
        };

        let pipeline = match VulkanPipeline::new(device.clone(), swap_chain.clone(), render_pass.clone()) {
            Ok(pipeline) => Arc::new(pipeline),
            Err(err) => {
                println!("Error creating vulkan pipeline {:?}", err);
                return Err(ContextError);
            }
        };

        let mut frames = Vec::new();

        // TODO: Magic 2 for 2 frames in flight
        for i in 0..2 {
            let frame_resources = Arc::new(VulkanFrameResources::new(resources.clone(), i));

            let frame = match VulkanFrame::new(
                device.clone(), frame_resources.clone(), swap_chain.clone(), render_pass.clone(), pipeline.clone()
            ) {
                Ok(frame) => frame,
                Err(err) => {
                    println!("Error creating vulkan frame {:?}", err);
                    return Err(ContextError);
                }
            };
            
            frames.push(Arc::new(frame));
        }

        Ok( Self {
            device: device,
            swapchain: swap_chain,
            resources: resources,
            render_pass: render_pass,
            pipeline: pipeline,
            frame_number: 0,
            frames: frames,
        } )
    }
}

impl Context for VulkanContext {
    type T = VulkanFrame;

    fn next_frame(&mut self) -> Result<Arc<Self::T>, ContextError> {
        // TODO: Manual 2 for 2 frames "in flight"
        let frame = self.frames[self.frame_number].clone();
        self.frame_number = (self.frame_number + 1) % 2;

        Ok(frame)
    }
}