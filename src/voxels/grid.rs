/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::voxels::Coord;
use crate::render::Voxel;

/*
 * Simple struct for grid errors
 */
#[derive(Debug)]
pub struct GridError;

/*
 * Trait conceptually wraps some way of accessing voxels
 * universally across chunk boundaries
 */
pub trait Grid {
    fn get_voxel(&self, coord: Coord) -> Option<Voxel>;

    fn set_voxel(&mut self, coord: Coord, voxel: Voxel) -> Result<(), GridError>;
}