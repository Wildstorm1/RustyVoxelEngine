/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::system::{ NonSendMut, Res, ResMut };
use glm::{GenMat, Matrix4, Vector3};
use crate::camera::PlayerCamera;
use crate::render::VulkanContext;
use crate::render::graphics::Camera;
use crate::render::graphics::hal::context::ImageExtent;
use crate::voxels::{Coord, VoxelWorld};
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

fn compute_camera(
    render_context: &NonSendMut<RenderContext>,
    camera: &Res<PlayerCamera>
) -> Camera {
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
    camera
}

pub fn point_in_frustum(
    chunk_coord: Coord,
    pvt_matrix: Matrix4<f32>
) -> bool {
    let point = glm::vec4(
        chunk_coord.x() as f32 * 16.0,
        chunk_coord.y() as f32 * 16.0,
        chunk_coord.z() as f32 * 16.0,
        1.0
    );

    // Normally this is 0.0, but I subtracted some space to account
    // for "close calls" near the corners of the camera
    glm::dot(point, pvt_matrix[3] + pvt_matrix[0]) > -10.0 &&
    glm::dot(point, pvt_matrix[3] - pvt_matrix[0]) > -10.0 &&
    glm::dot(point, pvt_matrix[3] + pvt_matrix[1]) > -10.0 &&
    glm::dot(point, pvt_matrix[3] - pvt_matrix[1]) > -10.0 &&
    glm::dot(point, pvt_matrix[2]) > -10.0 &&
    glm::dot(point, pvt_matrix[3] - pvt_matrix[2]) > -10.0
}

pub fn chunk_in_frustum(
    chunk_coord: Coord,
    pvt_matrix: Matrix4<f32>
) -> bool {
    point_in_frustum(chunk_coord + Coord::new(0, 0, 0), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(1, 0, 0), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(0, 1, 0), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(1, 1, 0), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(0, 0, 1), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(1, 0, 1), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(0, 1, 1), pvt_matrix) ||
    point_in_frustum(chunk_coord + Coord::new(1, 1, 1), pvt_matrix)
}

pub fn chunk_nearby_camera(
    camera_pos: Vector3<f32>,
    chunk_coord: Coord
) -> bool {
    let chunk_center = glm::vec3(
        16.0 * (chunk_coord.x() as f32 + 0.5f32),
        16.0 * (chunk_coord.y() as f32 + 0.5f32),
        16.0 * (chunk_coord.z() as f32 + 0.5f32)
    );

    glm::distance(chunk_center, camera_pos) <= 16f32 * 2.0f32
}

pub fn frustum_culling(
    mut render_context: NonSendMut<RenderContext>,
    mut camera_data: Res<PlayerCamera>,
    mut voxels: ResMut<VoxelWorld>
) {
    let camera = compute_camera(&render_context, &camera_data);
    let pvt_matrix = (camera.proj * camera.view).transpose();
    voxels.clear_visibility();

    voxels.get_cache().list_chunks().iter().for_each(|coord| {
        // TODO: Hacky this should be known elsewhere
        if coord.x() < 1 || coord.y() < 1 || coord.z() < 1 {
            return;
        }

        if coord.x() >= 10 - 1 || coord.y() >= 10 - 1 || coord.z() >= 10 - 1 {
            return;
        }

        let chunk = voxels.get_cache().get_chunk(*coord).unwrap();

        if chunk.is_air() {
            return;
        }

        if chunk_in_frustum(*coord, pvt_matrix) || chunk_nearby_camera(camera_data.position(), *coord) {
            voxels.set_visible(*coord);
        }
    });
}

/*
 * System renders the game
 */
pub fn render(
    mut render_context: NonSendMut<RenderContext>,
    mut camera: Res<PlayerCamera>,
    mut voxels: Res<VoxelWorld>
) {
    let camera = compute_camera(&render_context, &camera);
    render_context.renderer.render(camera, voxels.get_cache(), voxels.get_visibility());
}