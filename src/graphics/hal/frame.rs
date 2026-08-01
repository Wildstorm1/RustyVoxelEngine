/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::job::{ Job };

/*
 * Simple struct for frame errors
 */
#[derive(Debug)]
pub struct FrameError;

pub trait Frame {
    /*
     * The implementor must choose the type which is for the same backend
     */
    type T: Job;

    /*
     * The implementor must choose the type which is for the same backend
     */
    type E: Job;

    /*
     * Creates the frames transfer job
     */
    fn create_transfer_job(&self) -> Result<Self::T, FrameError>;

    /*
     * Submits the frames transfer job
     */
    fn submit_transfer_job(&self, job: Self::T) -> Result<(), FrameError>;

    /*
     * Creates the frames draw job
     */
    fn create_draw_job(&self) -> Result<Self::E, FrameError>;
}