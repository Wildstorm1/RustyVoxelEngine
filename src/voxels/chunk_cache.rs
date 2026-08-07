/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::UniformBufferObject;
use super::coord::ChunkCoord;
use super::chunk::Chunk;
use std::collections::HashMap;

pub struct ChunkCache {
    buffer: HashMap<ChunkCoord, Chunk>,
    ubos: HashMap<ChunkCoord, UniformBufferObject>,
}

impl ChunkCache {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
            ubos: HashMap::new()
        }
    }

    pub fn cache(&mut self, coord: ChunkCoord, chunk: Chunk, ubo: UniformBufferObject) {
        self.buffer.insert(coord, chunk);
        self.ubos.insert(coord, ubo);
    }

    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.buffer.get_mut(&coord)
    }

    pub fn get_ubo(&mut self, coord: ChunkCoord) -> Option<&UniformBufferObject> {
        self.ubos.get(&coord)
    }
}