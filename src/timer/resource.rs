/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::time::Duration;
use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct Timer {
    dt: Duration
}

impl Timer {
    pub fn new() -> Self {
        Self {
            dt: Duration::new(0, 0)
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.dt = dt;
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }
}