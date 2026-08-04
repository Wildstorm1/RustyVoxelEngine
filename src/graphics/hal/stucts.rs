/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use glm::{ Matrix4 };

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Voxel {
    pub data: u64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct UniformBufferObject {
    pub model: Matrix4<f32>,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Camera {
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}