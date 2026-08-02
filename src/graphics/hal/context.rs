/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::frame::{ Frame };
use std::sync::{ Arc };

/*
 * Extent of the image
 */
#[derive(Hash, PartialEq, Eq, Debug)]
pub struct ImageExtent {
    pub width: u32,
    pub height: u32,
}

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

    /*
     * Returns the extent (width / height) of the image
     */
    fn image_extent(&self) -> Result<ImageExtent, ContextError>;

    /*
     * Handles rebuilding when the window resizes
     */
    fn window_resize(&mut self) -> Result<(), ContextError>;
}