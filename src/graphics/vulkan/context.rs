/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

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
use ash::{ vk };

#[derive(Copy, Clone)]
struct FrameContext {
    image_available_gpu: vk::Semaphore,
    in_flight_fence: vk::Fence,
}

#[derive(Copy, Clone)]
struct SwapchainViewContext {
    work_complete_gpu: vk::Semaphore,
}

/*
 * Vulkan implementation of context
 */
pub struct VulkanContext {
    device: Arc<VulkanDevice>,
    swapchain: Arc<VulkanSwapchain>,
    resources: Arc<VulkanResources>,
    render_pass: Arc<VulkanRenderPass>,
    pipeline: Arc<VulkanPipeline>,
    frame_contexts: Vec<FrameContext>,
    swap_chain_contexts: Vec<SwapchainViewContext>,
    frames: Vec<Option<Arc<VulkanFrame>>>,
    frame_number: usize,
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

        let mut frame_contexts = Vec::new();
        let mut frames = Vec::new();

        // TODO: Magic 2 for 2 frames in flight
        for i in 0..2 {
            let semaphore_info = vk::SemaphoreCreateInfo {
                ..Default::default()
            };

            let image_available_on_gpu = match unsafe { device.device.create_semaphore(&semaphore_info, None) } {
                Ok(semaphore) => semaphore,
                Err(err) => {
                    println!("Error allocating image available semaphore {}", err);
                    return Err(ContextError);
                }
            };

            let fence_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };

            let in_flight_fence = match unsafe { device.device.create_fence(&fence_info, None) } {
                Ok(fence) => fence,
                Err(err) => {
                    println!("Error allocating in flight fence {}", err);
                    return Err(ContextError);
                }
            };

            frame_contexts.push(FrameContext {
                image_available_gpu: image_available_on_gpu,
                in_flight_fence: in_flight_fence,
            });

            frames.push(None);
        }

        let mut swap_chain_contexts = Vec::new();

        for i in 0..swap_chain.swap_chain_image_views.len() {
            let create_info = vk::SemaphoreCreateInfo {
                ..Default::default()
            };

            let semaphore = match unsafe { device.device.create_semaphore(&create_info, None) } {
                Ok(semaphore) => semaphore,
                Err(err) => {
                    println!("Could not create {}th semaphore! {}", i, err);
                    return Err(ContextError);
                }
            };

            swap_chain_contexts.push(SwapchainViewContext {
                work_complete_gpu: semaphore,
            });
        }

        Ok( Self {
            device: device,
            swapchain: swap_chain,
            resources: resources,
            render_pass: render_pass,
            pipeline: pipeline,
            frame_contexts: frame_contexts,
            swap_chain_contexts: swap_chain_contexts,
            frames: frames,
            frame_number: 0,
        } )
    }
}

impl Context for VulkanContext {
    type T = VulkanFrame;

    fn next_frame(&mut self) -> Result<Arc<Self::T>, ContextError> {
        let context = self.frame_contexts[self.frame_number];

        unsafe { self.device.device.wait_for_fences(&[ context.in_flight_fence ], true, u64::MAX).map_err(|err| {
            println!("Error waiting on frame fence {}", err);
            ContextError
        }) }?;

        unsafe { self.device.device.reset_fences(&[ context.in_flight_fence ]).map_err(|err| {
            println!("Error resetting frame fence {}", err);
            ContextError
        }) }?;

        let (image_index, _) = unsafe { self.device.swapchain_extension.acquire_next_image(self.swapchain.swap_chain, u64::MAX, context.image_available_gpu, vk::Fence::null()).map_err(|err| {
            println!("Error acquiring next frame from swap chain {}", err);
            ContextError
        }) }?;

        let frame_resources = Arc::new(VulkanFrameResources::new(self.device.clone(), self.resources.clone(), self.render_pass.clone(), self.frame_number, image_index));

        let next_frame = match VulkanFrame::new(
            self.device.clone(),
            frame_resources,
            self.swapchain.clone(),
            self.pipeline.clone(),
            context.image_available_gpu,
            self.swap_chain_contexts[image_index as usize].work_complete_gpu,
            context.in_flight_fence,
        ) {
            Ok(frame) => Arc::new(frame),
            Err(err) => {
                println!("Error creating vulkan frame {:?}", err);
                return Err(ContextError);
            }
        };

        self.frames[self.frame_number] = Some(next_frame.clone());
        self.frame_number = (self.frame_number + 1) % self.frame_contexts.len();
        Ok(next_frame)
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device.device_wait_idle().map_err(|err| {
                println!("Error waiting for device to be idle {:?}", err);
            });

            self.swap_chain_contexts.iter().for_each(|view_context| {
                self.device.device.destroy_semaphore(view_context.work_complete_gpu, None);
            });

            self.frame_contexts.iter().for_each(|frame_context| {
                self.device.device.destroy_semaphore(frame_context.image_available_gpu, None);
                self.device.device.destroy_fence(frame_context.in_flight_fence, None);
            });
        }
    }
}
