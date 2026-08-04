/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::hal::stucts::UniformBufferObject;

use super::super::hal::resources::{
    Resources, ResourcesError, Buffer, Uniform, WriteVoxelBuffer, WriteModelBuffer, BufferHandles, UniformHandles
};
use super::super::hal::stucts::{ Voxel };
use super::application::pipeline::{ VulkanPipeline };
use super::application::render_pass::{ VulkanRenderPass };
use super::application::resources::{ VulkanResources };
use super::hardware::swapchain::{ VulkanSwapchain };
use super::hardware::device::{ VulkanDevice };
use std::sync::{ Arc };
use ash::vk;

/*
 * Wrapper implementation that partitions resources per frame
 */
pub struct VulkanFrameResources {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanResources>,
    render_pass: Arc<VulkanRenderPass>,
    swap_chain: Arc<VulkanSwapchain>,
    pipeline: Arc<VulkanPipeline>,
    frame_number: usize,
    swap_chain_index: u32,
}

impl VulkanFrameResources {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanResources>,
        render_pass: Arc<VulkanRenderPass>,
        swap_chain: Arc<VulkanSwapchain>,
        pipeline: Arc<VulkanPipeline>,
        frame_number: usize,
        swap_chain_index: u32,
    ) -> Self {
        Self {
            device: device,
            resources: resources,
            render_pass: render_pass,
            swap_chain: swap_chain,
            pipeline: pipeline,
            frame_number: frame_number,
            swap_chain_index: swap_chain_index
        }
    }
}

impl Resources for VulkanFrameResources {
    fn get_buffer_handle(&self, label: BufferHandles) -> Result<Buffer, ResourcesError> {
        self.resources.get_buffer_handle(label)
    }

    fn get_uniform_handle(&self, label: UniformHandles) -> Result<Uniform, ResourcesError> {
        self.resources.get_uniform_handle(label)
    }

    unsafe fn buffer_voxel_data_cpu_side(&self, args: WriteVoxelBuffer) -> Result<(), ResourcesError> {
        let voxel_buffer_start = self.resources.get_vulkan_uniform_pointer(UniformHandles::VoxelsCPU)? as *mut Voxel;
        let vertex_slot = unsafe { voxel_buffer_start.add(args.offset) };

        unsafe { std::ptr::copy_nonoverlapping(args.voxels.as_ptr(), vertex_slot, args.voxels.len()) };

        Ok(())
    }

    unsafe fn buffer_model_data_cpu_side(&self, args: WriteModelBuffer) -> Result<(), ResourcesError> {
        let uniform_buffer_start;

        if self.frame_number == 0 {
            uniform_buffer_start = self.resources.get_vulkan_uniform_pointer(UniformHandles::ModelsFrame1CPU)? as *mut UniformBufferObject;
        } else {
            uniform_buffer_start = self.resources.get_vulkan_uniform_pointer(UniformHandles::ModelsFrame2CPU)? as *mut UniformBufferObject;
        }

        let uniform_slot = unsafe { uniform_buffer_start.add(args.offset) };

        unsafe { *uniform_slot = args.model };

        Ok(())
    }
}

impl VulkanFrameResources {
    pub fn get_vulkan_buffer_handle(&self, handle: Buffer) -> Result<vk::Buffer, ResourcesError> {
        self.resources.get_vulkan_buffer_handle_from_handle(handle)
    }

    pub fn get_vulkan_uniform_handle(&self, handle: Uniform) -> Result<vk::Buffer, ResourcesError> {
        self.resources.get_vulkan_uniform_handle_from_handle(handle)
    }

    pub fn get_descriptor_set(&self) -> Result<vk::DescriptorSet, ResourcesError> {
        Ok(self.device.descriptor_sets[self.frame_number])
    }

    pub fn get_frame_buffer(&self) -> vk::Framebuffer {
        self.render_pass.frame_buffers[self.swap_chain_index as usize]
    }

    pub fn get_render_pass(&self) -> vk::RenderPass {
        self.render_pass.render_pass
    }

    pub fn get_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline.pipeline_layout
    }

    pub fn get_presentation_index(&self) -> u32 {
        self.swap_chain_index
    }
}