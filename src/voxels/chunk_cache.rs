/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::render::{ Voxel };
use crate::voxels::{ Grid, GridError, Size };
use super::coord::Coord;
use super::component::Chunk;
use std::collections::HashMap;

/*
 * Simple struct for cache errors
 */
#[derive(Debug)]
pub struct ChunkCacheError;

pub struct ChunkCache {
    buffer: HashMap<Coord, Chunk>,
    size: Size
}

// TODO: Bounds checking, rollover, duplication with voxel world?
impl ChunkCache {
    pub fn new(size: Size) -> Self {
        Self {
            buffer: HashMap::new(),
            size: size,
        }
    }

    pub fn cache(&mut self, coord: Coord, chunk: Chunk) {
        // Note: Should check the size
        self.buffer.insert(coord, chunk);
    }

    pub fn remove_chunk(&mut self, coord: Coord) -> Option<Chunk> {
        self.buffer.remove(&coord)
    }

    pub fn get_chunk_mut(&mut self, coord: Coord) -> Option<&mut Chunk> {
        self.buffer.get_mut(&coord)
    }

    pub fn get_chunk(&self, coord: Coord) -> Option<&Chunk> {
        self.buffer.get(&coord)
    }

    pub fn list_chunks(&self) -> Vec<Coord> {
        self.buffer.keys().copied().collect()
    }
}

/*
 * Implementation returns None or GridError when a chunk is not
 * available in the voxel space
 */
impl Grid for ChunkCache {
    fn get_voxel(&self, coord: Coord) -> Option<Voxel> {
        let chunk_coord = Coord::new(
            coord.x() / self.size.x() as i64,
            coord.y() / self.size.y() as i64,
            coord.z() / self.size.z() as i64
        );

        let chunk = self.get_chunk(chunk_coord)?;

        Some(chunk.get_voxel(
            (coord.x() % self.size.x() as i64) as usize,
            (coord.y() % self.size.y() as i64) as usize,
            (coord.z() % self.size.z() as i64) as usize
        ))
    }

    fn set_voxel(&mut self, coord: Coord, voxel: Voxel) -> Result<(), GridError> {
        let chunk_coord = Coord::new(
            coord.x() / self.size.x() as i64,
            coord.y() / self.size.y() as i64,
            coord.z() / self.size.z() as i64
        );

        let chunk_size = self.size;
        let chunk = self.get_chunk_mut(chunk_coord).ok_or(GridError)?;

        chunk.set_voxel(
            (coord.x() % chunk_size.x() as i64) as usize,
            (coord.y() % chunk_size.y() as i64) as usize,
            (coord.z() % chunk_size.z() as i64) as usize,
            voxel
        );

        Ok(())
    }
}