/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::renderer::{ Renderer };
use crate::graphics::{ Camera, Context, UniformBufferObject, Voxel, VulkanContext };
use crate::voxels::{ Chunk };

use sdl3::{pixels::Color, video::WindowFlags};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use ash::{vk, Entry};
use noise::{NoiseFn, Perlin, Seedable};

mod graphics;
mod voxels;

#[derive(Hash, PartialEq, Eq, Debug, Copy, Clone)]
pub struct ChunkCoord {
    cx: usize,
    cy: usize,
    cz: usize,
}

impl ChunkCoord {
    pub fn new(
        cx: usize,
        cy: usize,
        cz: usize
    ) -> Self {
        Self {
            cx: cx,
            cy: cy,
            cz: cz
        }
    }
}

pub struct ChunkCache {
    buffer: HashMap<ChunkCoord, Chunk>,
    ubos: HashMap<ChunkCoord, UniformBufferObject>,
}

impl ChunkCache {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
            ubos: HashMap::new()
        }
    }

    pub fn cache(&mut self, coord: ChunkCoord, chunk: Chunk, ubo: UniformBufferObject) {
        self.buffer.insert(coord, chunk);
        self.ubos.insert(coord, ubo);
    }

    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.buffer.get_mut(&coord)
    }

    pub fn get_ubo(&mut self, coord: ChunkCoord) -> Option<&UniformBufferObject> {
        self.ubos.get(&coord)
    }
}

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
    let mut chunk_store = ChunkCache::new();
    let mut coords = Vec::new();

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

                let model = UniformBufferObject {
                    model: glm::ext::translate(
                        &glm::mat4(
                            1.0f32, 0.0f32, 0.0f32, 0.0f32,
                            0.0f32, 1.0f32, 0.0f32, 0.0f32,
                            0.0f32, 0.0f32, 1.0f32, 0.0f32,
                            0.0f32, 0.0f32, 0.0f32, 1.0f32
                        ),
                        glm::vec3((cx * 16) as f32, (cy * 16) as f32, (cz * 16) as f32)
                    )
                };

                chunk_store.cache(ChunkCoord::new(cx, cy, cz), chunk, model);
            }
        }
    }

    for cy in 1..4 {
        for cz in 1..9 {
            for cx in 1..9 {
                let coordinate = ChunkCoord::new(cx, cy, cz);
                coords.push(coordinate);

                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();
                            let mut voxel = chunk.get_voxel(x, y, z);

                            // Right face: +z
                            if z < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y, z + 1).data != 0 {
                                    voxel.metadata |= 0x10000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy, cz + 1)).unwrap();

                                if neighbor_chunk.get_voxel(x, y, 0).data != 0 {
                                    voxel.metadata |= 0x10000000;
                                }
                            }

                            // Left face: -z
                            if z > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y, z - 1).data != 0 {
                                    voxel.metadata |= 0x20000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy, cz - 1)).unwrap();

                                if neighbor_chunk.get_voxel(x, y, 15).data != 0 {
                                    voxel.metadata |= 0x20000000;
                                }
                            }

                            // Front face: +x
                            if x < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x + 1, y, z).data != 0 {
                                    voxel.metadata |= 0x80000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx + 1, cy, cz)).unwrap();

                                if neighbor_chunk.get_voxel(0, y, z).data != 0 {
                                    voxel.metadata |= 0x80000000;
                                }
                            }

                            // Back face: -x
                            if x > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x - 1, y, z).data != 0 {
                                    voxel.metadata |= 0x40000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx - 1, cy, cz)).unwrap();

                                if neighbor_chunk.get_voxel(15, y, z).data != 0 {
                                    voxel.metadata |= 0x40000000;
                                }
                            }

                            // Top face: +y
                            if y < 15 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y + 1, z).data != 0 {
                                    voxel.metadata |= 0x08000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy + 1, cz)).unwrap();

                                if neighbor_chunk.get_voxel(x, 0, z).data != 0 {
                                    voxel.metadata |= 0x08000000;
                                }
                            }

                            // Bottom face: -y
                            if y > 0 {
                                let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();

                                if chunk.get_voxel(x, y - 1, z).data != 0 {
                                    voxel.metadata |= 0x04000000;
                                }
                            } else {
                                let neighbor_chunk = chunk_store.get_chunk_mut(ChunkCoord::new(cx, cy - 1, cz)).unwrap();

                                if neighbor_chunk.get_voxel(x, 15, z).data != 0 {
                                    voxel.metadata |= 0x04000000;
                                }
                            }

                            let chunk = chunk_store.get_chunk_mut(coordinate).unwrap();
                            chunk.set_voxel(x, y, z, voxel);
                        }
                    }
                }
            }
        }
    }

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
