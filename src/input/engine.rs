/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::message::Message;
use bevy_ecs::resource::Resource;

#[derive(Message)]
pub enum EngineEvent {
    WindowResize,
    Quit,
}

#[derive(Resource)]
pub struct InternalAppEvents {
    app_quit: bool,
    window_recreate: bool,
}

impl InternalAppEvents {
    pub fn new() -> Self {
        Self {
            app_quit: false,
            window_recreate: false,
        }
    }

    pub fn quit_app(&mut self) {
        self.app_quit = true;
    }

    pub fn app_is_quit(&self) -> bool {
        self.app_quit
    }

    pub fn recreate_window(&mut self) {
        self.window_recreate = true;
    }

    pub fn app_needs_window_recreate(&self) -> bool {
        self.window_recreate
    }

    pub fn reset(&mut self) {
        self.app_quit = false;
        self.window_recreate = false;
    }
}