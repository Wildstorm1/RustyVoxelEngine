/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::stucts::{ Camera };
use super::resources::{ Buffer };
use std::fmt;

/*
 * Arguments for a draw mesh command
 */
pub struct DrawPacket {
    pub camera: Camera,
    pub model_uniform_index: u64,
    pub voxel_model_index: u64,
}

/*
 * Arguments for a copy buffer command
 */
pub struct CopyBufferPacket {
    pub src: Buffer,
    pub dst: Buffer,
    pub src_offset: u64,
    pub dst_offset: u64,
    pub bytes: u64,
}

/*
 * Simple struct for command buffer errors
 */
#[derive(Debug)]
pub struct CommandBufferError;

impl fmt::Display for CommandBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CommandBufferError")
    }
}

pub trait CommandBuffer {
    /*
     * Issues an indexed draw command
     */
    fn draw(&mut self, packet: DrawPacket) -> Result<(), CommandBufferError>;

    /*
     * Issues a copy buffer command
     */
    fn copy_buffer(&mut self, packet: CopyBufferPacket) -> Result<(), CommandBufferError>;

    /*
     * Convert the command buffer to native api commands
     */
    fn stage(&mut self) -> Result<(), CommandBufferError>;
}