/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use glm::{ Matrix4 };

pub const RIGHT_FACE_METADATA_BIT: u32 = 0x10000000;
pub const LEFT_FACE_METADATA_BIT: u32 = 0x20000000;
pub const FRONT_FACE_METADATA_BIT: u32 = 0x80000000;
pub const BACK_FACE_METADATA_BIT: u32 = 0x40000000;
pub const TOP_FACE_METADATA_BIT: u32 = 0x08000000;
pub const BOTTOM_FACE_METADATA_BIT: u32 = 0x04000000;
pub const VOXEL_P1_00_P1: u32 = 0x02000000;
pub const VOXEL_00_P1_P1: u32 = 0x01000000;
pub const VOXEL_P1_P1_P1: u32 = 0x00800000;
pub const VOXEL_N1_00_P1: u32 = 0x00400000;
pub const VOXEL_N1_P1_P1: u32 = 0x00200000;
pub const VOXEL_00_N1_P1: u32 = 0x00100000;
pub const VOXEL_P1_N1_P1: u32 = 0x00080000;
pub const VOXEL_N1_N1_P1: u32 = 0x00040000;
pub const VOXEL_P1_P1_00: u32 = 0x00020000;
pub const VOXEL_P1_N1_00: u32 = 0x00010000;
pub const VOXEL_P1_00_N1: u32 = 0x00008000;
pub const VOXEL_P1_P1_N1: u32 = 0x00004000;
pub const VOXEL_P1_N1_N1: u32 = 0x00002000;
pub const VOXEL_00_P1_N1: u32 = 0x00001000;
pub const VOXEL_00_N1_N1: u32 = 0x00000800;
pub const VOXEL_N1_00_N1: u32 = 0x00000400;
pub const VOXEL_N1_P1_N1: u32 = 0x00000200;
pub const VOXEL_N1_N1_N1: u32 = 0x00000100;
pub const VOXEL_N1_P1_00: u32 = 0x00000080;
pub const VOXEL_N1_N1_00: u32 = 0x00000040;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Voxel {
    pub metadata: u32,
    pub data: u32
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