/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::message::Message;

#[derive(Message)]
pub struct MouseMotion {
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
}