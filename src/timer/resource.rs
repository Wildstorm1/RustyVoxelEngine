/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use std::time::Duration;
use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct Timer {
    dt: Duration,
    total_time: Duration
}

impl Timer {
    pub fn new() -> Self {
        Self {
            dt: Duration::new(0, 0),
            total_time: Duration::new(0, 0)
        }
    }

    pub fn update(&mut self, dt: Duration) {
        self.dt = dt;
        self.total_time = self.total_time + dt;
    }

    pub fn dt(&self) -> Duration {
        self.dt
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }
}