/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::render::{ Voxel };
use crate::voxels::{ Grid, GridError, Size };
use super::coord::Coord;
use super::component::Chunk;
use std::collections::HashMap;

pub struct ChunkWrapper {
    chunk: Chunk,
}

impl ChunkWrapper {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk: chunk
        }
    }

    pub fn to_owned(self) -> Chunk {
        self.chunk
    }
}

/*
 * Implementation returns air or ok when a voxel is not
 * available in the voxel space
 */
impl Grid for ChunkWrapper {
    fn get_voxel(&self, coord: Coord) -> Option<Voxel> {
        let size = self.chunk.size();
        let x = coord.x();
        let y = coord.y();
        let z = coord.z();

        if x >= 0 && x < size.x() as i64 && y >= 0 && y < size.y() as i64 && z >= 0 && z < size.z() as i64 {
            return Some(self.chunk.get_voxel(x as usize, y as usize, z as usize));
        }

        Some(Voxel {
            metadata: 0,
            data: 0,
        })
    }

    fn set_voxel(&mut self, coord: Coord, voxel: Voxel) -> Result<(), GridError> {
        let size = self.chunk.size();
        let x = coord.x();
        let y = coord.y();
        let z = coord.z();

        if x >= 0 && x < size.x() as i64 && y >= 0 && y < size.y() as i64 && z >= 0 && z < size.z() as i64 {
            self.chunk.set_voxel(x as usize, y as usize, z as usize, voxel);
        }

        Ok(())
    }
}