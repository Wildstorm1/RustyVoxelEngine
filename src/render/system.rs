/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::system::{NonSendMut, Res};
use crate::camera::PlayerCamera;
use crate::render::VulkanContext;
use crate::render::graphics::Camera;
use crate::render::graphics::hal::context::ImageExtent;
use crate::voxels::VoxelWorld;
use super::Renderer;

/*
 * Simple struct for renderer errors
 */
#[derive(Debug)]
pub struct RenderContextError;

/*
 * Non-send main thread resource that contains the
 * first version renderer (non-system based)
 */
pub struct RenderContext {
    renderer: Renderer<VulkanContext>,
}

impl RenderContext {
    pub fn new(
        renderer: Renderer<VulkanContext>
    ) -> Self {
        Self {
            renderer: renderer
        }
    }

    pub fn extent(&self) -> ImageExtent {
        self.renderer.extent()
    }

    pub fn recreate_window(&mut self) -> Result<(), RenderContextError> {
        self.renderer.window_resize().map_err(|err| {
            println!("Error resizing window {:?}", err);
            RenderContextError
        })
    }
}

/*
 * System renders the game
 */
pub fn render(
    mut render_context: NonSendMut<RenderContext>,
    mut camera: Res<PlayerCamera>,
    mut voxels: Res<VoxelWorld>,
) {
    let extent = render_context.extent();
    let position = camera.position();

    let mut camera = Camera {
        view: glm::ext::look_at(
            position,
            position + camera.look_direction(),
            camera.up_direction()
        ),
        proj: glm::ext::perspective(
            glm::radians(45.0f32),
            extent.width as f32 / extent.height as f32,
            0.1f32,
            1000.0f32
        )
    };

    camera.proj[1][1] *= -1.0f32;

    render_context.renderer.render(camera, voxels.get_cache(), voxels.get_visibility());
}