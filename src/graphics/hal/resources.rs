/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

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
     * Returns the large global index buffer
     */
    fn get_index_buffer_handle(&self) -> Result<Buffer, ResourcesError>;

    /*
     * Returns the large global model uniform buffer
     */
    fn get_model_buffer_handle(&self) -> Result<Uniform, ResourcesError>;
}