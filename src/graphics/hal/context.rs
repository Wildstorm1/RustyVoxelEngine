/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::frame::{ Frame };
use std::sync::{ Arc };

/*
 * Inputs needed for creating a command buffer
 */
pub struct CommandBufferArgs {}

/*
 * Simple struct for context errors
 */
#[derive(Debug)]
pub struct ContextError;

pub trait Context {
    /*
     * The implementor must choose the type which is for the same backend
     */
    type T: Frame;

    /*
     * Returns the next frame for recording
     */
    fn next_frame(&mut self) -> Result<Arc<Self::T>, ContextError>;
}