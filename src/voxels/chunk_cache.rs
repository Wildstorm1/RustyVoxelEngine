/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::render::{  UniformBufferObject, Voxel };
use super::coord::Coord;
use super::chunk::Chunk;
use std::collections::HashMap;

/*
 * Simple struct for cache errors
 */
#[derive(Debug)]
pub struct ChunkCacheError;

pub struct ChunkCache {
    buffer: HashMap<Coord, Chunk>,
}

// TODO: Bounds checking, rollover, duplication with voxel world?
impl ChunkCache {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
        }
    }

    pub fn cache(&mut self, coord: Coord, chunk: Chunk) {
        self.buffer.insert(coord, chunk);
    }

    pub fn remove_chunk(&mut self, coord: Coord) -> Option<Chunk> {
        self.buffer.remove(&coord)
    }

    pub fn get_chunk_mut(&mut self, coord: Coord) -> Option<&mut Chunk> {
        self.buffer.get_mut(&coord)
    }

    pub fn list_chunks(&self) -> Vec<Coord> {
        self.buffer.keys().copied().collect()
    }

    pub fn get_chunk(&self, coord: Coord) -> Option<&Chunk> {
        self.buffer.get(&coord)
    }

    pub fn get_voxel(&self, coord: Coord) -> Option<Voxel> {
        // TODO: Hardcoded chunk size
        let chunk_coord = Coord::new(
            coord.x() / 16,
            coord.y() / 16,
            coord.z() / 16
        );

        let chunk = self.get_chunk(chunk_coord)?;

        Some(chunk.get_voxel(
            (coord.x() % 16) as usize,
            (coord.y() % 16) as usize,
            (coord.z() % 16) as usize
        ))
    }

    pub fn set_voxel(&mut self, coord: Coord, voxel: Voxel) -> Result<(), ChunkCacheError> {
        // TODO: Hardcoded chunk size
        let chunk_coord = Coord::new(
            coord.x() / 16,
            coord.y() / 16,
            coord.z() / 16
        );

        let chunk = self.get_chunk_mut(chunk_coord).ok_or(ChunkCacheError)?;

        chunk.set_voxel(
            (coord.x() % 16) as usize,
            (coord.y() % 16) as usize,
            (coord.z() % 16) as usize,
            voxel
        );

        Ok(())
    }
}