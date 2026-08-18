/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::component::Component;
use crate::render::Voxel;

/*
 * Size is a measure of the number of voxels in a model
 */
#[derive(Component, Copy, Clone)]
pub struct Size {
    x_dim: usize,
    y_dim: usize,
    z_dim: usize
}

impl Size {
    pub fn new(
        x_dim: usize,
        y_dim: usize,
        z_dim: usize
    ) -> Self {
        Self {
            x_dim,
            y_dim,
            z_dim
        }
    }

    pub fn x(&self) -> usize {
        self.x_dim
    }

    pub fn y(&self) -> usize {
        self.y_dim
    }

    pub fn z(&self) -> usize {
        self.z_dim
    }
}

/*
 * Chunk of voxel data
 */
#[derive(Component)]
pub struct Chunk {
    is_all_air: bool,
    buffer: Vec<Voxel>,
    size: Size,
}

fn get_index(x: usize, y: usize, z: usize, size: Size) -> usize {
    y * size.x() * size.z() + z * size.x() + x
}

impl Chunk {
    pub fn new(size: Size) -> Self {
        let len = size.x() * size.y() * size.z();
        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            vec.push(Voxel {
                metadata: 0,
                data: 0
            })
        }

        Self {
            is_all_air: true,
            buffer: vec,
            size: size,
        }
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        let index = get_index(x, y, z, self.size);
        self.buffer[index]
    }

    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, voxel: Voxel) {
        let index = get_index(x, y, z, self.size);
        self.buffer[index] = voxel;
    }

    pub fn set_is_air(&mut self, is_air: bool) {
        self.is_all_air = is_air;
    }

    pub fn is_air(&self) -> bool {
        self.is_all_air
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn as_ptr(&self) -> *const Voxel {
        self.buffer.as_ptr()
    }
}