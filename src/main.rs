/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::renderer::{ Renderer };
use crate::graphics::{ Camera, Context, UniformBufferObject, Voxel, VulkanContext };
use crate::voxels::{ Chunk, ChunkCache, ChunkCoord, world_init };

use bevy_ecs::resource::Resource;
use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use glm::Vector3;
use sdl3::Sdl;
use sdl3::video::Window;
use sdl3::{pixels::Color, video::WindowFlags};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ash::{vk, Entry};
use noise::{NoiseFn, Perlin, Seedable};

mod graphics;
mod voxels;

#[derive(Resource)]
struct PlayerCamera {
    yaw: f32,
    pitch: f32,
    pos: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
}

/*
fn write_resource_system(mut resource: ResMut<MyResource>) {
    assert_eq!(resource.value, 42);
    resource.value = 0;
    assert_eq!(resource.value, 0);
}
*/

fn main() {
    let sdl_context = sdl3::init().map_err(|e| e.to_string()).unwrap();
    let video_subsystem = sdl_context.video().map_err(|e| e.to_string()).unwrap();
    video_subsystem.vulkan_load_library_default().unwrap();

    let window = video_subsystem
        .window("Settle And Survive", 800, 600)
        .position_centered()
        .set_flags(WindowFlags::VULKAN | WindowFlags::RESIZABLE)
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut world = World::new();

    world.insert_resource(PlayerCamera {
        yaw: 2.3220053f32,
        pitch: -0.7900002f32,
        pos: glm::vec3(84.0f32, 108.0f32, 68.0f32),
        front: glm::vec3(-0.5680655f32, -0.77073944f32, -0.28855217f32),
        up: glm::vec3(0.0f32, 1.0f32, 0.0f32)
    });

    let mut render = Schedule::default();

    let context = match VulkanContext::new(&window) {
        Ok(context) => Box::new(context),
        Err(err) => {
            println!("Error setting up context {:?}", err);
            return;
        }
    };

    let mut renderer = Renderer::new(context);
    let extent = renderer.extent().unwrap();
    sdl_context.mouse().warp_mouse_in_window(&window, extent.width as f32 / 2.0, extent.height as f32 / 2.0);

    let mut center_mouse = true;
    let mut dx_mov = 0.0f32;
    let mut dy_mov = 0.0f32;
    let mut yaw = 2.3220053f32; //4.712f32; // 3 * pi / 2
    let mut pitch = -0.7900002f32;
    let mut camera_pos = glm::vec3(84.0f32, 108.0f32, 68.0f32);
    let mut camera_front = glm::vec3(-0.5680655f32, -0.77073944f32, -0.28855217f32);
    let mut camera_up = glm::vec3(0.0f32, 1.0f32, 0.0f32);

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();
    let mut frame_num = 0;

    println!("Creating chunks...");
    let (mut chunk_store, coords) = world_init(25, 10);
    let coords = Arc::new(coords);
    println!("Chunks created!");

    'running: loop {
        let start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    println!("Game exit requested");
                    break 'running
                },
                Event::Window { win_event, .. } => {
                    match win_event {
                        WindowEvent::PixelSizeChanged(x, y) => {
                            println!("Window size changed to {}, {}", x, y);

                            match renderer.window_resize() {
                                Err(err) => {
                                    println!("Error resizing window in main loop {:?}", err);
                                    break 'running                                    
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                Event::KeyDown { keycode, .. } => {
                    match keycode {
                        Some(keycode) => {
                            match keycode {
                                Keycode::Escape => {
                                    println!("Game exit requested");
                                    break 'running
                                },
                                Keycode::W => {
                                    camera_pos = camera_pos + (camera_front * 0.2f32);
                                },
                                Keycode::S => {
                                    camera_pos = camera_pos - (camera_front * 0.2f32);
                                },
                                Keycode::A => {
                                    camera_pos = camera_pos - (glm::normalize(glm::cross(camera_front, camera_up)) * 0.2f32);
                                },
                                Keycode::D => {
                                    camera_pos = camera_pos + (glm::normalize(glm::cross(camera_front, camera_up)) * 0.2f32);
                                },
                                Keycode::E => {
                                    center_mouse = !center_mouse;
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    // Skip resetting the mouse to screen center
                    let extent = renderer.extent().unwrap();

                    if center_mouse && (x != extent.width as f32 / 2.0 || y != extent.height as f32 / 2.0) {
                        dx_mov = dx_mov + xrel;
                        dy_mov = dy_mov + yrel;
                        println!("Frame {} - {} {}", frame_num, x, y);

                        sdl_context.mouse().warp_mouse_in_window(&window, extent.width as f32 / 2.0, extent.height as f32 / 2.0);
                    }
                }
                _ => {}
            }
        }

        let extent = renderer.extent().unwrap();

        let sensitivity = 0.01f32;
        let dx = dx_mov * sensitivity;
        let dy = dy_mov * sensitivity;
        dx_mov = 0.0f32;
        dy_mov = 0.0f32;

        yaw = yaw + dx;
        pitch = pitch - dy;

        if pitch > 1.55f32 { // pi / 2 - 0.017
            pitch = 1.55f32;
        }

        if pitch < -1.55f32 { // - pi / 2 + 0.017
            pitch = -1.55f32;
        }

        let direction = glm::vec3(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
        camera_front = glm::normalize(direction);

        let mut camera = Camera {
            view: glm::ext::look_at(camera_pos, camera_pos + camera_front, camera_up),
            proj: glm::ext::perspective(
                glm::radians(45.0f32),
                extent.width as f32 / extent.height as f32,
                0.1f32,
                1000.0f32
            )
        };

        camera.proj[1][1] *= -1.0f32;

        match renderer.render(camera, &mut chunk_store, coords.clone()) {
            Ok(_) => (),
            Err(err) => {
                println!("Error rendering in main loop {:?}", err);
                break 'running
            }
        }

        frame_num = frame_num + 1;
        let end_time = Instant::now();
        //println!("Frame generation time (ms): {:?}", end_time.duration_since(start_time));
    }
}
