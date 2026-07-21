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
        .window("Settle And Survive", 800, 600)
        .position_centered()
        .set_flags(WindowFlags::VULKAN | WindowFlags::RESIZABLE)
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let device = unsafe { match graphics::vulkan::device::VulkanDevice::new(&window) {
        Ok(device) => device,
        Err(err) => { println!("Error creating vulkan device {:?}", err); return; }
    } };

    let swap_chain = unsafe { match graphics::vulkan::swapchain::VulkanSwapchain::new(&device, &window) {
        Ok(swap_chain) => swap_chain,
        Err(err) => { println!("Error creating vulkan swapchain {:?}", err); return; }
    } };

    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        if window.is_maximized() {
            println!("Keep window in scope");
        }
        // App stuff
    }
}
