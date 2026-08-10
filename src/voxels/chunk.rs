/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::render::Voxel;

/*
 * Chunk of voxel data
 */
pub struct Chunk {
    buffer: Vec<Voxel>,
}

impl Chunk {
    pub fn new() -> Self {
        // TODO: Hard coded chunk size (task shader)
        let mut vec = Vec::with_capacity(16 * 16 * 16);

        for _ in 0..(16 * 16 * 16) {
            vec.push(Voxel {
                metadata: 0,
                data: 0
            })
        }

        Self {
            buffer: vec,
        }
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        let index = x * 16 * 16 + y * 16 + z;
        self.buffer[index]
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let index = x * 16 * 16 + y * 16 + z;
        self.buffer[index] = voxel;
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn as_ptr(&self) -> *const Voxel {
        self.buffer.as_ptr()
    }
}