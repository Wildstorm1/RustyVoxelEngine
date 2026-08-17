/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::component::Component;
use glm::{ Vector3, Vector4 };

#[derive(Component)]
pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Vector4<f32>,
    pub scale: Vector3<f32>
}

impl Transform {
    pub const IDENTITY: Self = Transform {
        translation: glm::vec3(0.0, 0.0, 0.0),
        rotation: glm::vec4(1.0, 0.0, 0.0, 0.0),
        scale: glm::vec3(1.0, 1.0, 1.0),
    };
}