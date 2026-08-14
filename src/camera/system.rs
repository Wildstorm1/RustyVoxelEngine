/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::input::{ KeyCode, KeyboardButtonInput, MouseMotion, SdlContext };
use crate::render::{ RenderContext, VulkanContext };
use crate::timer::Timer;
use bevy_ecs::message::MessageReader;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{ NonSend, NonSendMut, Res, ResMut };
use glm::Vector3;

#[derive(Resource)]
pub struct PlayerCamera {
    yaw: f32,
    pitch: f32,
    pos: Vector3<f32>,
}

impl PlayerCamera {
    pub fn new() -> Self {
        Self {
            yaw: 2.3220053f32,
            pitch: -0.7900002f32,
            pos: glm::vec3(84.0f32, 108.0f32, 68.0f32),
        }
    }

    pub fn look_direction(&self) -> Vector3<f32> {
        let direction = glm::vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        );

        glm::normalize(direction)
    }

    pub fn up_direction(&self) -> Vector3<f32> {
        glm::vec3(0.0f32, 1.0f32, 0.0f32)
    }

    pub fn position(&self) -> Vector3<f32> {
        self.pos
    }
}

/*
 * System looks for presses to the keyboard and updates the
 * position of the camera accordingly
 */
pub fn update_camera_position(
    keyboard: Res<KeyboardButtonInput>,
    mut camera: ResMut<PlayerCamera>,
    timer: Res<Timer>
) {
    let camera_front = camera.look_direction();
    let camera_up = camera.up_direction();

    if keyboard.is_pressed(KeyCode::KeyW) {
        camera.pos = camera.pos + (camera_front * 20.0f32 * timer.dt().as_secs_f32());
    }

    if keyboard.is_pressed(KeyCode::KeyS) {
        camera.pos = camera.pos - (camera_front * 20.0f32 * timer.dt().as_secs_f32());
    }

    if keyboard.is_pressed(KeyCode::KeyA) {
        camera.pos = camera.pos - (glm::normalize(glm::cross(camera_front, camera_up)) * 20.0f32 * timer.dt().as_secs_f32());
    }

    if keyboard.is_pressed(KeyCode::KeyD) {
        camera.pos = camera.pos + (glm::normalize(glm::cross(camera_front, camera_up)) * 20.0f32 * timer.dt().as_secs_f32());
    }
}

/*
 * System look for movement of the mouse and updates the yaw/pitch
 * of the camera accordingly
 */
pub fn update_camera_direction(
    mut sdl_context: NonSendMut<SdlContext>,
    keyboard: Res<KeyboardButtonInput>,
    render_context: NonSend<RenderContext>,
    mut reader: MessageReader<MouseMotion>,
    mut camera: ResMut<PlayerCamera>
) {
    if keyboard.is_pressed(KeyCode::KeyE) {
        return;
    }

    let mut dx_mov = 0.0f32;
    let mut dy_mov = 0.0f32;

    for MouseMotion { x, y, dx, dy } in reader.read() {
        // Ignore if the mouse is located at center (happens when moving mouse back to center)
        let extent = render_context.extent();

        println!("Mouse motion: {:?} {:?} {:?} {:?} {:?} {:?}", x, y, dx, dy, extent.width, extent.height);

        if !(*x != extent.width as f32 / 2.0 || *y != extent.height as f32 / 2.0) {
            continue;
        }
            
        if dx.abs() >= 1.0f32 {
            dx_mov = dx_mov + dx;
        }

        if dy.abs() >= 1.0f32 {
            dy_mov = dy_mov + dy;
        }

        sdl_context.move_mouse(extent.width as f32 / 2.0, extent.height as f32 / 2.0);
    }

    let sensitivity = 0.01f32;
    let dx = dx_mov * sensitivity;
    let dy = dy_mov * sensitivity;

    camera.yaw = camera.yaw + dx;
    camera.pitch = camera.pitch - dy;

    if camera.pitch > 1.55f32 { // pi / 2 - 0.017
        camera.pitch = 1.55f32;
    }

    if camera.pitch < -1.55f32 { // - pi / 2 + 0.017
        camera.pitch = -1.55f32;
    }
}