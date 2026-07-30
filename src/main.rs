/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::renderer::{ Renderer };
use crate::graphics::{ CommandBuffer, Context, Resources, VulkanCommandBuffer, VulkanContext };

use sdl3::{pixels::Color, video::WindowFlags};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use std::time::Duration;
use ash::{vk, Entry};

mod graphics;

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

    let context = match VulkanContext::new(window) {
        Ok(context) => Box::new(context),
        Err(err) => {
            println!("Error setting up context {:?}", err);
            return;
        }
    };

    let mut renderer = Renderer::new(context);

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
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
                _ => {}
            }
        }

        match renderer.render() {
            Ok(_) => (),
            Err(err) => {
                println!("Error rendering in main loop {:?}", err);
                break 'running
            }
        }
    }
}
