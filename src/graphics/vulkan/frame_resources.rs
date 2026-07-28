/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::resources::{
    Resources, ResourcesError, Buffer, Uniform
};
use super::application::render_pass::{ VulkanRenderPass };
use super::application::resources::{ VulkanResources };
use super::hardware::device::{ VulkanDevice };
use std::ffi::{ c_void };
use std::sync::{ Arc };
use ash::vk;

/*
 * Wrapper implementation that partitions resources per frame
 */
pub struct VulkanFrameResources {
    device: Arc<VulkanDevice>,
    resources: Arc<VulkanResources>,
    //render_pass: Arc<VulkanRenderPass>,
    frame_number: usize
}

impl VulkanFrameResources {
    pub fn new(
        device: Arc<VulkanDevice>,
        resources: Arc<VulkanResources>,
        //render_pass: Arc<VulkanRenderPass>,
        frame_number: usize
    ) -> Self {
        Self {
            device: device,
            resources: resources,
            //render_pass: render_pass,
            frame_number: frame_number
        }
    }
}

impl Resources for VulkanFrameResources {
    fn get_vertex_buffer_handle(&self) -> Result<Buffer, ResourcesError> {
        self.resources.get_vertex_buffer_handle()
    }

    fn get_index_buffer_handle(&self) -> Result<Buffer, ResourcesError> {
        self.resources.get_index_buffer_handle()
    }

    fn get_model_buffer_handle(&self) -> Result<Uniform, ResourcesError> {
        if self.frame_number == 0 {
            self.resources.get_model_buffer_frame1_handle()
        } else if self.frame_number == 1 {
            self.resources.get_model_buffer_frame2_handle()
        } else {
            Err(ResourcesError)
        }
    }
}

impl VulkanFrameResources {
    pub fn get_model_buffer_pointer(&self) -> Result<*mut c_void, ResourcesError> {
        if self.frame_number == 0 {
            Ok(self.resources.frame1_uniform_data.mapping)
        } else if self.frame_number == 1 {
            Ok(self.resources.frame2_uniform_data.mapping)
        } else {
            Err(ResourcesError)
        }
    }

    pub fn get_vertex_buffer_pointer(&self) -> Result<*mut c_void, ResourcesError> {
        Ok(self.resources.vertex_data.mapping)
    }

    pub fn get_vulkan_vertex_buffer_handle(&self) -> Result<vk::Buffer, ResourcesError> {
        Ok(self.resources.vertex_data.buffer)
    }

    pub fn get_vulkan_vertex_staging_buffer_handle(&self) -> Result<vk::Buffer, ResourcesError> {
        Ok(self.resources.vertex_data.staging_buffer)
    }

    pub fn get_index_buffer_pointer(&self) -> Result<*mut c_void, ResourcesError> {
        Ok(self.resources.index_data.mapping)
    }

    pub fn get_vulkan_index_buffer_handle(&self) -> Result<vk::Buffer, ResourcesError> {
        Ok(self.resources.index_data.buffer)
    }

    pub fn get_vulkan_index_staging_buffer_handle(&self) -> Result<vk::Buffer, ResourcesError> {
        Ok(self.resources.index_data.staging_buffer)
    }

    pub fn get_descriptor_set(&self) -> Result<vk::DescriptorSet, ResourcesError> {
        Ok(self.device.descriptor_sets[self.frame_number])
    }

    // pub fn get_frame_buffer(&self) -> 
    // pub fn get_render_pass
}