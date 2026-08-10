/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::resource::Resource;
use std::collections::HashSet;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub enum KeyCode {
    KeyA,
    KeyW,
    KeyS,
    KeyD,
    KeyE,
    KeyQ,
}

#[derive(Resource)]
pub struct KeyboardButtonInput {
    pressed: HashSet<KeyCode>,
}

impl KeyboardButtonInput {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new()
        }
    }

    pub fn press(&mut self, input: KeyCode) {
        self.pressed.insert(input);
    }

    pub fn un_press(&mut self, input: KeyCode) {
        self.pressed.remove(&input);
    }

    pub fn is_pressed(&self, input: KeyCode) -> bool {
        self.pressed.contains(&input)
    }
}