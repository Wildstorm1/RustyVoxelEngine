/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::hal::resources::{ Resources };
use super::command_buffer::{CommandBuffer};

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
     * The implementor must choose the type which is for the same backend
     */
    type E: Resources;

    /*
     * Creates a new buffer
     */
    fn get_resources(&'a self) -> Result<&'a Self::E, ContextError>;

    /*
     * Creates a new command buffer
     */
    fn create_command_buffer(&'a self, args: CommandBufferArgs) -> Result<Self::T, ContextError>;

    /*
     * Submits the given command buffer for processing
     */
    fn submit_command_buffer(&'a mut self, buffer: Self::T) -> Result<Self::T, ContextError>;

    /*
     * Begins a new frame for recording
     */
    fn begin_frame(&'a mut self);

    /*
     * Ends the current frame's recording
     */
    fn end_frame(&'a mut self);

    /*
     * Presents the current frame for display
     */
    fn present_frame(&'a mut self);
}