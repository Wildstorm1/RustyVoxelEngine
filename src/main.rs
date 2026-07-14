/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use sdl3::{pixels::Color, video::WindowFlags};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::time::Duration;
use ash::{vk, Entry};

mod graphics;

fn main() {
    let sdl_context = sdl3::init().map_err(|e| e.to_string()).unwrap();
    let video_subsystem = sdl_context.video().map_err(|e| e.to_string()).unwrap();
    video_subsystem.vulkan_load_library_default().unwrap();

    let window = video_subsystem
        .window("SDL3 Rust Demo", 800, 600)
        .position_centered()
        .set_flags(WindowFlags::VULKAN)
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    //let device = unsafe { graphics::vulkan::device::VulkanDevice::new(window).unwrap() };

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        // App stuff
    }
}
