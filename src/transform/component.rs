/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::component::Component;
use glm::{ Vector3, Vector4, Matrix4 };

#[derive(Component, Copy, Clone)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Vector4<f32>,
    pub scale: f32
}

impl Transform {
    pub const IDENTITY: Self = Transform {
        translation: glm::vec3(0.0, 0.0, 0.0),
        rotation: glm::vec4(0.0, 0.0, 0.0, 1.0),
        scale: 1.0,
    };
}

#[derive(Component, Copy, Clone)]
pub struct GlobalTransform {
    pub p_world: Matrix4<f32>,
}

impl GlobalTransform {
    pub const IDENTITY: Self = GlobalTransform {
        p_world: glm::mat4(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0
        ),
    };
}