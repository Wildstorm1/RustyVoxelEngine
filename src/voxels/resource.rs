/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct VoxelMapSettings {
    width: usize,
    depth: usize
}

impl VoxelMapSettings {
    pub fn new(width: usize, depth: usize) -> Self {
        Self {
            width: width,
            depth: depth,
        }
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn width(&self) -> usize {
        self.width
    }
}