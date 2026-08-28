/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::input::engine::InternalAppEvents;
use super::engine::{EngineEvent};
use super::keyboard::{KeyCode, KeyboardButtonInput};
use super::mouse::{MouseMotion, MouseButtonInput, MouseCode};
use bevy_ecs::system::{NonSendMut, ResMut};
use bevy_ecs::message::{MessageReader, MessageWriter};
use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::video::Window;
use sdl3::{EventPump, Sdl};
use std::sync::{Arc};

/*
 * Non-theadsafe resource which contains the windowing event
 * pump
 */
pub struct SdlContext {
    context: Sdl,
    window: Arc<Window>,
    event_pump: EventPump,
}

impl SdlContext {
    pub fn new(
        context: Sdl,
        window: Arc<Window>,
        event_pump: EventPump
    ) -> Self {
        Self {
            context: context,
            window: window,
            event_pump: event_pump
        }
    }

    pub fn move_mouse(&mut self, x: f32, y: f32) {
        self.context.mouse().warp_mouse_in_window(&self.window, x, y);
    }
}

/*
 * System polls the sdl context for events and updates
 * the game engine resources which are made available to
 * systems for consumption
 */
pub fn update_inputs(
    mut inputs: NonSendMut<SdlContext>,
    mut keyboard: ResMut<KeyboardButtonInput>,
    mut mouse: ResMut<MouseButtonInput>,
    mut mouse_motion: MessageWriter<MouseMotion>,
    mut engine_events: MessageWriter<EngineEvent>
) {
    for event in inputs.event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => {
                engine_events.write(EngineEvent::Quit);
            },
            Event::Window { win_event, .. } => {
                match win_event {
                    WindowEvent::PixelSizeChanged(_, _) => {
                        engine_events.write(EngineEvent::WindowResize);
                    },
                    _ => {}
                }
            },
            Event::MouseMotion { x, y, xrel, yrel, .. } => {
                mouse_motion.write(MouseMotion { x, y, dx: xrel, dy: yrel });
            },
            Event::MouseButtonDown { mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left => mouse.press(MouseCode::LeftKey),
                    MouseButton::Right => mouse.press(MouseCode::RightKey),
                    _ => {}
                }
            },
            Event::MouseButtonUp { mouse_btn, .. } => {
                match mouse_btn {
                    MouseButton::Left => mouse.un_press(MouseCode::LeftKey),
                    MouseButton::Right => mouse.un_press(MouseCode::RightKey),
                    _ => {}
                }
            },
            Event::KeyDown { keycode, .. } => {
                match keycode {
                    Some(keycode) => {
                        match keycode {
                            Keycode::Escape => {
                                engine_events.write(EngineEvent::Quit);
                            },
                            Keycode::W => keyboard.press(KeyCode::KeyW),
                            Keycode::S => keyboard.press(KeyCode::KeyS),
                            Keycode::A => keyboard.press(KeyCode::KeyA),
                            Keycode::D => keyboard.press(KeyCode::KeyD),
                            Keycode::E => keyboard.press(KeyCode::KeyE),
                            Keycode::Q => keyboard.press(KeyCode::KeyQ),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            Event::KeyUp { keycode, .. } => {
                match keycode {
                    Some(keycode) => {
                        match keycode {
                            Keycode::Escape => {
                                engine_events.write(EngineEvent::Quit);
                            },
                            Keycode::W => keyboard.un_press(KeyCode::KeyW),
                            Keycode::S => keyboard.un_press(KeyCode::KeyS),
                            Keycode::A => keyboard.un_press(KeyCode::KeyA),
                            Keycode::D => keyboard.un_press(KeyCode::KeyD),
                            Keycode::E => keyboard.un_press(KeyCode::KeyE),
                            Keycode::Q => keyboard.un_press(KeyCode::KeyQ),
                            _ => {}
                        }
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}

/*
 * Translates any engine events into flags on an engine
 * resource monitored by the game loop
 */
pub fn update_engine_events(
    mut app_event: ResMut<InternalAppEvents>,
    mut reader: MessageReader<EngineEvent>,
) {
    for event in reader.read() {
        match event {
            EngineEvent::Quit => app_event.quit_app(),
            EngineEvent::WindowResize => app_event.recreate_window(),
        }
    }
}