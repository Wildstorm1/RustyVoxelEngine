/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use glm::{ Vector3, Matrix4 };

#[repr(C)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub color: Vector3<f32>,
}

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