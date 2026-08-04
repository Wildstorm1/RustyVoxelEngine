/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::stucts::{ Voxel, UniformBufferObject };
use std::fmt;

/*
 * List of buffers with buffer handles
 */
#[derive(Hash, PartialEq, Eq, Debug)]
pub enum BufferHandles {
    VertexCPU,
    VertexGPU,
    IndexCPU,
    IndexGPU
}

/*
 * List of uniforms with handles
 */
#[derive(Hash, PartialEq, Eq, Debug)]
pub enum UniformHandles {
    VoxelsCPU,
    VoxelsGPU,
    ModelsFrame1CPU,
    ModelsFrame2CPU,
}

/*
 * Handle to an underlying implementations buffer type
 */
#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Buffer {
    pub handle: u64,
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer: {}", self.handle)
    }
}

/*
 * Handle to an underlying implementations uniform type
 */
#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Uniform {
    pub handle: u64,
}

impl fmt::Display for Uniform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uniform: {}", self.handle)
    }
}

/*
 * Arguments writing to the voxel buffer
 * 
 * offset: starting index of where to write voxel data to in Voxel units
 * voxels: list of voxels to copy
 */
pub struct WriteVoxelBuffer<'a> {
    pub offset: usize,
    pub voxels: &'a Vec<Voxel>,
}

/*
 * Arguments writing to the model buffer
 * 
 * offset: starting index of where to write model data to in UniformBufferObject units
 * model: the model to write
 */
pub struct WriteModelBuffer {
    pub offset: usize,
    pub model: UniformBufferObject,
}

/*
 * Simple struct for context errors
 */
#[derive(Debug)]
pub struct ResourcesError;

pub trait Resources {
    /*
     * Returns the large global vertex buffer
     */
    fn get_buffer_handle(&self, label: BufferHandles) -> Result<Buffer, ResourcesError>;

    /*
     * Returns the large global model uniform buffer
     */
    fn get_uniform_handle(&self, label: UniformHandles) -> Result<Uniform, ResourcesError>;

    /*
     * Writes voxel data to the cpu side voxel buffer
     */
    unsafe fn buffer_voxel_data_cpu_side(&self, args: WriteVoxelBuffer) -> Result<(), ResourcesError>;

    /*
     * Writes model data to the cpu side model buffer
     */
    unsafe fn buffer_model_data_cpu_side(&self, args: WriteModelBuffer) -> Result<(), ResourcesError>;
}