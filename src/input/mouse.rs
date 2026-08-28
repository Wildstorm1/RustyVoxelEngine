/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::message::Message;
use bevy_ecs::resource::Resource;
use std::collections::HashSet;

#[derive(Message)]
pub struct MouseMotion {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum MouseCode {
    LeftKey,
    RightKey,
}

#[derive(Resource)]
pub struct MouseButtonInput {
    pressed: HashSet<MouseCode>,
}

impl MouseButtonInput {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new()
        }
    }

    pub fn press(&mut self, input: MouseCode) {
        self.pressed.insert(input);
    }

    pub fn un_press(&mut self, input: MouseCode) {
        self.pressed.remove(&input);
    }

    pub fn is_pressed(&self, input: MouseCode) -> bool {
        self.pressed.contains(&input)
    }
}