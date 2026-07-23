/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::command_buffer::{CommandBuffer};
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
 * Bitflags for the intended usage of the buffer
 */
pub struct BufferUsage(pub u32);

impl BufferUsage {
    pub const TRANSFER_DST: u32 = 0b00000001;
    pub const TRANSFER_SRC: u32 = 0b00000010;
    pub const VERTEX_BUFFER: u32 = 0b00000100;
    pub const INDEX_BUFFER: u32 = 0b00001000;
}

/*
 * Bitflags for memory properties
 */
pub struct BufferMemoryProperties(pub u32);

impl BufferMemoryProperties {
    pub const DEVICE_LOCAL: u32 = 0b00000001;
    pub const HOST_VISIBLE: u32 = 0b00000010;
    pub const HOST_COHERENT: u32 = 0b00000100;
}

/*
 * Inputs needed for creating a buffer
 */
pub struct UniformArgs {
    pub size: u64,
    pub count: u64,
    pub uniform_index: usize
}

/*
 * Inputs needed for creating a uniform
 */
pub struct BufferArgs {
    pub size: u64,
    pub usage: BufferUsage,
    pub properties: BufferMemoryProperties,
}

/*
 * Inputs needed for creating a command buffer
 */
pub struct CommandBufferArgs {}

/*
 * Simple struct for context errors
 */
#[derive(Debug)]
pub struct ContextError;

pub trait Context<'a> {
    /*
     * The implementor must choose the type which is for the same backend
     */
    type T: CommandBuffer;

    /*
     * Creates a new buffer
     */
    fn create_buffer(&'a self, args: BufferArgs) -> Result<Buffer, ContextError>;

    /*
     * Creates a new uniform value
     */
    fn create_uniform(&'a self, args: UniformArgs) -> Result<Uniform, ContextError>;

    /*
     * Creates a new command buffer
     */
    fn create_command_buffer(&'a self, args: CommandBufferArgs) -> Result<Self::T, ContextError>;
}