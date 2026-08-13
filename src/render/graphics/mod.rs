/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

pub mod vulkan;
pub mod hal;

pub use vulkan::context::{ VulkanContext };
pub use hal::context::{ Context };
pub use hal::stucts::{
    Camera,
    Voxel,
    UniformBufferObject,
    RIGHT_FACE_METADATA_BIT,
    LEFT_FACE_METADATA_BIT,
    FRONT_FACE_METADATA_BIT,
    BACK_FACE_METADATA_BIT,
    TOP_FACE_METADATA_BIT,
    BOTTOM_FACE_METADATA_BIT,
    VOXEL_P1_00_P1,
    VOXEL_00_P1_P1,
    VOXEL_P1_P1_P1,
    VOXEL_N1_00_P1,
    VOXEL_N1_P1_P1,
    VOXEL_00_N1_P1,
    VOXEL_P1_N1_P1,
    VOXEL_N1_N1_P1,
    VOXEL_P1_P1_00,
    VOXEL_P1_N1_00,
    VOXEL_P1_00_N1,
    VOXEL_P1_P1_N1,
    VOXEL_P1_N1_N1,
    VOXEL_00_P1_N1,
    VOXEL_00_N1_N1,
    VOXEL_N1_00_N1,
    VOXEL_N1_P1_N1,
    VOXEL_N1_N1_N1,
    VOXEL_N1_P1_00,
    VOXEL_N1_N1_00
};
