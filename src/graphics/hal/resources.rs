/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::stucts::{ Vertex, UniformBufferObject, Camera };
use std::fmt;

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
 * Arguments writing to the vertex buffer
 * 
 * offset: starting index of where to write vertex data to in Vertex units
 * vertices: list of vertices to copy
 */
pub struct WriteVertexBuffer<'a> {
    pub offset: usize,
    pub vertices: &'a Vec<Vertex>,
}

/*
 * Arguments writing to the index buffer
 * 
 * offset: starting index of where to write index data to in u32 units
 * indices: list of indices to copy
 */
pub struct WriteIndexBuffer<'a> {
    pub offset: usize,
    pub indices: &'a Vec<u32>,
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
    fn get_vertex_buffer_handle(&self) -> Result<Buffer, ResourcesError>;

    /*
     * Writes vertex data to the cpu side vertex buffer
     */
    unsafe fn buffer_vertex_data_cpu_side(&self, args: WriteVertexBuffer) -> Result<(), ResourcesError>;

    /*
     * Returns the large global index buffer
     */
    fn get_index_buffer_handle(&self) -> Result<Buffer, ResourcesError>;

    /*
     * Writes index data to the cpu side index buffer
     */
    unsafe fn buffer_index_data_cpu_side(&self, args: WriteIndexBuffer) -> Result<(), ResourcesError>;

    /*
     * Returns the large global model uniform buffer
     */
    fn get_model_buffer_handle(&self) -> Result<Uniform, ResourcesError>;

    /*
     * Writes model data to the cpu side model buffer
     */
    unsafe fn buffer_model_data_cpu_side(&self, args: WriteModelBuffer) -> Result<(), ResourcesError>;
}