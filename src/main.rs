/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::camera::{ PlayerCamera, update_camera_direction, update_camera_position };
use crate::render::{ RenderContext, VulkanContext, render };
use crate::input::{ EngineEvent, InternalAppEvents, KeyboardButtonInput, MouseButtonInput, MouseMotion, SdlContext, update_engine_events, update_inputs };
use crate::transform::propagate_transform_system;
use crate::voxels::{ VoxelMap, VoxelMapSettings, block_manipulation, generate_hermit_crab_chunks, generate_initial_chunks, rotate_chunks };
use crate::timer::Timer;

use bevy_ecs::message::{ MessageRegistry, message_update_system };
use bevy_ecs::schedule::{ IntoScheduleConfigs, Schedule };
use bevy_ecs::world::World;
use sdl3::video::WindowFlags;
use std::sync::{Arc};
use std::time::{Duration, Instant};

mod timer;
mod render;
mod camera;
mod input;
mod voxels;
mod transform;

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
        Ok(context) => context,
        Err(err) => {
            println!("Error setting up context {:?}", err);
            return;
        }
    };

    world.insert_non_send(SdlContext::new(
        sdl_context,
        window.clone(),
        event_pump
    ));

    world.insert_non_send(RenderContext::new(context));
    world.insert_resource(PlayerCamera::new());
    world.insert_resource(KeyboardButtonInput::new());
    world.insert_resource(MouseButtonInput::new());
    world.insert_resource(VoxelMapSettings::new(10, 10, 16));
    world.insert_resource(VoxelMap::new());
    world.insert_resource(InternalAppEvents::new());
    world.insert_resource(Timer::new());
    MessageRegistry::register_message::<MouseMotion>(&mut world);
    MessageRegistry::register_message::<EngineEvent>(&mut world);

    let mut init_schedule = Schedule::default();
    init_schedule.add_systems(generate_hermit_crab_chunks);
    init_schedule.add_systems(generate_initial_chunks);

    let mut input_schedule = Schedule::default();
    input_schedule.add_systems((
        update_inputs,
        message_update_system.after(update_inputs),
        update_engine_events.after(message_update_system)
    ));

    let mut pre_update = Schedule::default();
    pre_update.add_systems(propagate_transform_system);

    let mut update_schedule = Schedule::default();
    update_schedule.add_systems(update_camera_position);
    update_schedule.add_systems(update_camera_direction::<VulkanContext>);
    update_schedule.add_systems(rotate_chunks);
    update_schedule.add_systems(block_manipulation);

    let mut render_schedule = Schedule::default();
    render_schedule.add_systems((
        render::<VulkanContext>,
    ));

    init_schedule.run(&mut world);

    'running: loop {
        let start_time = Instant::now();

        input_schedule.run(&mut world);
        pre_update.run(&mut world);
        update_schedule.run(&mut world);
        render_schedule.run(&mut world);

        let mut app_events = world.get_resource_mut::<InternalAppEvents>().unwrap();
        let app_quit = app_events.app_is_quit();
        let window_recreate = app_events.app_needs_window_recreate();
        app_events.reset();

        let mut render_resource = world.get_non_send_mut::<RenderContext<VulkanContext>>().unwrap();

        if app_quit {
            break 'running
        }

        if window_recreate {
            render_resource.recreate_window().unwrap()
        }

        let end_time = Instant::now();
        let mut timer = world.get_resource_mut::<Timer>().unwrap();
        timer.update(end_time.duration_since(start_time));
    }
}
