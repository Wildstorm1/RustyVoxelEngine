/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::resources::{
    Resources, ResourcesError, Buffer, Uniform
};
use super::application::resources::{ VulkanResources };
use std::ffi::{ c_void };
use std::sync::{ Arc };
use ash::vk;

/*
 * Wrapper implementation that partitions resources per frame
 */
pub struct VulkanFrameResources {
    resources: Arc<VulkanResources>,
    frame_number: usize
}

impl VulkanFrameResources {
    pub fn new(
        resources: Arc<VulkanResources>,
        frame_number: usize
    ) -> Self {
        Self { 
            resources: resources,
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
}