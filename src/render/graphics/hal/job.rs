/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::resources::{ Resources };
use super::command_buffer::{ CommandBuffer };
use std::sync::Arc;

#[derive(Debug)]
pub enum JobErrorCode {
    GENERIC,
    WINDOW_RESIZE,
}

/*
 * Simple struct for frame errors
 */
#[derive(Debug)]
pub struct JobError {
    pub error: JobErrorCode,
}

pub trait Job {
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
    fn get_resources(&self) -> Result<Arc<Self::E>, JobError>;

    /*
     * Creates a new command buffer
     */
    fn create_command_buffer(&self) -> Result<Self::T, JobError>;

    /*
     * Submits the given command buffer for processing
     */
    fn submit_command_buffer(&self, buffer: Self::T) -> Result<(), JobError>;

    /*
     * Work needed to begin the job
     */
    fn begin_job(&self) -> Result<(), JobError>;

    /*
     * Work needed to ends the current job
     */
    fn end_job(&self) -> Result<(), JobError>;

    /*
     * Work needed to executes the job asynchronously
     */
    fn run_job(&self) -> Result<(), JobError>;
}