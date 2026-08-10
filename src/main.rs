/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::camera::{ PlayerCamera, update_camera_direction, update_camera_position };
use crate::render::{ RenderContext, Renderer, VulkanContext, render };
use crate::input::{EngineEvent, InternalAppEvents, KeyboardButtonInput, MouseMotion, SdlContext, update_inputs, update_engine_events};
use crate::voxels::{ VoxelWorld };

use bevy_ecs::message::{MessageRegistry, message_update_system};
use bevy_ecs::schedule::Schedule;
use bevy_ecs::world::World;
use sdl3::video::WindowFlags;
use std::sync::{Arc};
use std::time::{Duration, Instant};

mod render;
mod camera;
mod input;
mod voxels;

fn main() {
    let mut world = World::new();

    let sdl_context = sdl3::init().map_err(|e| e.to_string()).unwrap();
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();

    let video_subsystem = sdl_context.video().map_err(|e| e.to_string()).unwrap();
    video_subsystem.vulkan_load_library_default().unwrap();

    let window = Arc::new(video_subsystem
        .window("Settle And Survive", 800, 600)
        .position_centered()
        .set_flags(WindowFlags::VULKAN | WindowFlags::RESIZABLE)
        .build()
        .map_err(|e| e.to_string())
        .unwrap()
    );

    let context = match VulkanContext::new(window.clone()) {
        Ok(context) => Box::new(context),
        Err(err) => {
            println!("Error setting up context {:?}", err);
            return;
        }
    };

    let mut renderer = Renderer::new(context);

    world.insert_non_send(SdlContext::new(
        sdl_context,
        window.clone(),
        event_pump
    ));

    world.insert_non_send(RenderContext::new(renderer));
    world.insert_resource(PlayerCamera::new());
    world.insert_resource(KeyboardButtonInput::new());
    world.insert_resource(VoxelWorld::new(25, 5));
    world.insert_resource(InternalAppEvents::new());
    MessageRegistry::register_message::<MouseMotion>(&mut world);
    MessageRegistry::register_message::<EngineEvent>(&mut world);

    let mut update = Schedule::default();
    update.add_systems(update_engine_events);
    update.add_systems(message_update_system);
    update.add_systems(update_camera_position);
    update.add_systems(update_camera_direction);
    update.add_systems(update_inputs);
    update.add_systems(render);

    let mut dt = Duration::new(0, 0);

    'running: loop {
        let start_time = Instant::now();

        //println!("Main game loop, previous frame {:?} ", dt);
        update.run(&mut world);

        let mut app_events = world.get_resource_mut::<InternalAppEvents>().unwrap();
        let app_quit = app_events.app_is_quit();
        let window_recreate = app_events.app_needs_window_recreate();
        app_events.reset();

        let mut render_resource = world.get_non_send_mut::<RenderContext>().unwrap();

        if app_quit {
            break 'running
        }

        if window_recreate {
            render_resource.recreate_window().unwrap()
        }

        let end_time = Instant::now();
        dt = end_time.duration_since(start_time);
    }
}
