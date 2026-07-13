use sdl3::pixels::Color;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use std::time::Duration;
use ash::{vk, Entry};

fn main() {
    let sdl_context = sdl3::init().map_err(|e| e.to_string()).unwrap();
    let video_subsystem = sdl_context.video().map_err(|e| e.to_string()).unwrap();
    let window = video_subsystem
        .window("SDL3 Rust Demo", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();

    let mut canvas = window.into_canvas();
    let mut event_pump = sdl_context.event_pump().map_err(|e| e.to_string()).unwrap();

    let entry = Entry::linked();

    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 0, 0),
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };

    let instance = unsafe { entry.create_instance(&create_info, None).unwrap() };

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(30, 40, 60));
        canvas.clear();
        canvas.present();

        std::thread::sleep(Duration::from_millis(16));
    }
}
