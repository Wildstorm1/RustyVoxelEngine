/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::resources::{ Resources };
use super::command_buffer::{ CommandBuffer };
use std::sync::Arc;

/*
 * Inputs needed for creating a command buffer
 */
pub struct CommandBufferArgs {}

/*
 * Simple struct for frame errors
 */
#[derive(Debug)]
pub struct FrameError;

pub trait Frame {
    /*
     * The implementor must choose the type which is for the same backend
     */
    type T: CommandBuffer;

    /*
     * The implementor must choose the type which is for the same backend
     */
    type E: Resources;

    /*
     * Creates a new buffer
     */
    fn get_resources(&self) -> Result<Arc<Self::E>, FrameError>;

    /*
     * Creates a new command buffer
     */
    fn create_command_buffer(&self, args: CommandBufferArgs) -> Result<Self::T, FrameError>;

    /*
     * Submits the given command buffer for processing
     */
    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), FrameError>;

    /*
     * Ends the current frame's recording
     */
    fn begin_frame(&self) -> Result<(), FrameError>;

    /*
     * Ends the current frame's recording
     */
    fn end_frame(&self) -> Result<(), FrameError>;

    /*
     * Presents the current frame for display
     */
    fn present_frame(&self) -> Result<(), FrameError>;
}