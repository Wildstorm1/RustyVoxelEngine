/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::collections::HashMap;
use bevy_ecs::resource::{ Resource };
use bevy_ecs::entity::{ Entity };
use crate::voxels::{ Coord };

#[derive(Resource)]
pub struct VoxelMapSettings {
    width: usize,
    depth: usize,
    chunk_size: usize
}

impl VoxelMapSettings {
    pub fn new(width: usize, depth: usize, size: usize) -> Self {
        Self {
            width: width,
            depth: depth,
            chunk_size: size
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

#[derive(Resource)]
pub struct VoxelMap {
    buffer: HashMap<Coord, Entity>,
}

impl VoxelMap {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new()
        }
    }

    pub fn insert(&mut self, coord: Coord, entity: Entity) {
        self.buffer.insert(coord, entity);
    }

    pub fn get(&self, coord: Coord) -> Option<Entity> {
        match self.buffer.get(&coord) {
            Some(entity) => Some(*entity),
            None => None
        }
    }

    pub fn remove(&mut self, coord: Coord) -> Option<Entity> {
        self.buffer.remove(&coord)
    }
}