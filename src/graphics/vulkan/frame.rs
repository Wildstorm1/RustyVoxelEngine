/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::frame::{ Frame, FrameError, CommandBufferArgs };
use super::frame_resources::{ VulkanFrameResources };
use super::command_buffer::{ VulkanCommandBuffer };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use super::application::render_pass::{ VulkanRenderPass };
use super::application::pipeline::{ VulkanPipeline };
use std::sync::{ Arc, Mutex };
use ash::ext::image_compression_control_swapchain;
use ash::{ vk };

/*
 * Implementation of frame for vulkan
 */
pub struct VulkanFrame {
    device: Arc<VulkanDevice>,
    swap_chain: Arc<VulkanSwapchain>,
    resources: Arc<VulkanFrameResources>,
    render_pass: Arc<VulkanRenderPass>,
    pipeline: Arc<VulkanPipeline>,
    command_buffer: vk::CommandBuffer,
    image_available_gpu: vk::Semaphore,
    in_flight_fence: vk::Fence,
    image_index: Mutex<Option<u32>>
}

impl VulkanFrame {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanFrameResources>,
        swap_chain: Arc<VulkanSwapchain>,
        render_pass: Arc<VulkanRenderPass>,
        pipeline: Arc<VulkanPipeline>
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

        let semaphore_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };

        let image_available_on_gpu = match unsafe { device.device.create_semaphore(&semaphore_info, None) } {
            Ok(semaphore) => semaphore,
            Err(err) => {
                println!("Error allocating image available semaphore {}", err);
                return Err(FrameError);
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
                return Err(FrameError);
            }
        };

        Ok(Self {
            device: device,
            swap_chain: swap_chain,
            render_pass: render_pass,
            pipeline: pipeline,
            resources: resources,
            command_buffer: command_buffer,
            image_available_gpu: image_available_on_gpu,
            in_flight_fence: in_flight_fence,
            image_index: Mutex::new(None)
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
        VulkanCommandBuffer::new(self.device.clone()).map_err(|err| {
            println!("Error creating command buffer {}", err);
            FrameError
        })
    }

    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), FrameError> {
        unimplemented!()
    }

    fn begin_frame(&self) -> Result<(), FrameError> {
        let mut image_index_guard = self.image_index.lock().map_err(|err| {
            println!("Mutex on image index is poisoned {}", err);
            FrameError
        })?;

        if image_index_guard.is_some() {
            println!("Image index is something when nothing is expected");
            return Err(FrameError);
        }

        unsafe { self.device.device.wait_for_fences(&[ self.in_flight_fence ], true, u64::MAX).map_err(|err| {
            println!("Error waiting on frame fence {}", err);
            FrameError
        }) }?;

        unsafe { self.device.device.reset_fences(&[ self.in_flight_fence ]).map_err(|err| {
            println!("Error resetting frame fence {}", err);
            FrameError
        }) }?;

        let (image_index, _) = unsafe { self.device.swapchain_extension.acquire_next_image(self.swap_chain.swap_chain, u64::MAX, self.image_available_gpu, vk::Fence::null()).map_err(|err| {
            println!("Error acquiring next frame from swap chain {}", err);
            FrameError
        }) }?;

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

        let render_pass_info = vk::RenderPassBeginInfo {
            render_pass: self.render_pass.render_pass,
            framebuffer: self.render_pass.frame_buffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D {
                    x: 0,
                    y: 0
                },
                extent: self.swap_chain.swap_chain_extent,
            },
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.device.cmd_begin_render_pass(self.command_buffer, &render_pass_info, vk::SubpassContents::INLINE) };
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
            extent: self.swap_chain.swap_chain_extent,
            ..Default::default()
        };

        unsafe { self.device.device.cmd_set_scissor(self.command_buffer, 0, &[ scissor ]) };

        *image_index_guard = Some(image_index);

        Ok(())
    }

    fn end_frame(&self) -> Result<(), FrameError> {
        let image_index_guard = self.image_index.lock().map_err(|err| {
            println!("Mutex on image index is poisoned {}", err);
            FrameError
        })?;

        if image_index_guard.is_none() {
            println!("Image index is unexpectedly none when something is expected");
            return Err(FrameError);
        }

        unsafe { self.device.device.cmd_end_render_pass(self.command_buffer); };

        unsafe { self.device.device.end_command_buffer(self.command_buffer).map_err(|err| {
            println!("Error ending command buffer {}", err);
            FrameError
        }) }?;

        let wait_semaphores = [ self.image_available_gpu ];
        let signal_semaphores = [ self.render_pass.frame_buffer_semaphore[image_index_guard.unwrap() as usize] ];
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

        unsafe { self.device.device.queue_submit(self.device.queues.graphics_queue, &[submit_info], self.in_flight_fence).map_err(|err| {
            println!("Error submitting command buffer {}", err);
            FrameError
        }) }?;

        Ok(())
    }

    fn present_frame(&self) -> Result<(), FrameError> {
        let mut image_index_guard = self.image_index.lock().map_err(|err| {
            println!("Mutex on image index is poisoned {}", err);
            FrameError
        })?;

        if image_index_guard.is_none() {
            println!("Image index is unexpectedly none when something is expected");
            return Err(FrameError);
        }

        let signal_semaphores = [ self.render_pass.frame_buffer_semaphore[image_index_guard.unwrap() as usize] ];
        let swap_chains = [ self.swap_chain.swap_chain ];
        let image_indices = [ image_index_guard.unwrap() ];

        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swap_chains.as_ptr(),
            p_image_indices: image_indices.as_ptr(),
            ..Default::default()
        };

        unsafe { self.device.swapchain_extension.queue_present(self.device.queues.presentation_queue, &present_info) };

        *image_index_guard = None;

        Ok(())
    }
}
