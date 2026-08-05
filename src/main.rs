/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::renderer::{ Renderer };
use crate::graphics::{ Camera, Context, UniformBufferObject, Voxel, VulkanContext };
use crate::voxels::{ Chunk };

use sdl3::{pixels::Color, video::WindowFlags};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ash::{vk, Entry};
use noise::{NoiseFn, Perlin, Seedable};

mod graphics;
mod voxels;

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
    let perlin = Perlin::new(1);
    let mut chunks = Vec::new();
    let mut ubos = Vec::new();

    for cy in 0..5 {
        for cz in 0..10 {
            for cx in 0..10 {
                let mut chunk = Chunk::new();

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let value = perlin.get([
                                (cx * 16 + x) as f64 / 16.0f64,
                                (cy * 16 + y) as f64 / 16.0f64,
                                (cz * 16 + z) as f64 / 16.0f64
                            ]);

                            if value > 0.0 {
                                chunk.set_voxel(x, y, z, Voxel {
                                    metadata: 0,
                                    data: 1,
                                });
                            }
                        }
                    }
                }

                chunks.push(Some(chunk));

                ubos.push(UniformBufferObject {
                    model: glm::ext::translate(
                        &glm::mat4(
                            1.0f32, 0.0f32, 0.0f32, 0.0f32,
                            0.0f32, 1.0f32, 0.0f32, 0.0f32,
                            0.0f32, 0.0f32, 1.0f32, 0.0f32,
                            0.0f32, 0.0f32, 0.0f32, 1.0f32
                        ),
                        glm::vec3((cx * 16) as f32, (cy * 16) as f32, (cz * 16) as f32)
                    )
                });
            }
        }
    }

    let mut filled_chunks = Vec::new();
    let mut filled_ubos = Vec::new();

    for cy in 1..4 {
        for cz in 1..9 {
            for cx in 1..9 {
                let mut chunk = std::mem::replace(&mut chunks[cy * 10 * 10 + cx * 10 + cz], None);
                let mut chunk = chunk.unwrap();
                let ubo = ubos[cy * 10 * 10 + cx * 10 + cz];

                /*
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                        }
                    }
                }
                */

                filled_chunks.push(chunk);
                filled_ubos.push(ubo);
            }
        }
    }

    let chunks = Arc::new(filled_chunks);
    let ubos = Arc::new(filled_ubos);

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
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                Event::MouseMotion { x, y, xrel, yrel, .. } => {
                    // Skip resetting the mouse to screen center
                    if x != extent.width as f32 / 2.0 || y != extent.height as f32 / 2.0 {
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

        match renderer.render(camera, chunks.clone(), ubos.clone()) {
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
