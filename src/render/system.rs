/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use bevy_ecs::system::{ NonSendMut, Query, Res, ResMut };
use glm::{GenMat, Matrix4, Vector3};
use crate::camera::PlayerCamera;
use crate::render::graphics::hal::command_buffer::{CommandBuffer, CopyBufferPacket, DrawPacket};
use crate::render::graphics::hal::frame::Frame;
use crate::render::graphics::hal::job::Job;
use crate::render::graphics::hal::resources::{BufferHandles, Resources, WriteModelBuffer, WriteVoxelBuffer};
use crate::render::{ Context, UniformBufferObject, Voxel, VulkanContext };
use crate::render::graphics::Camera;
use crate::render::graphics::hal::context::ImageExtent;
use crate::transform::Transform;
use crate::voxels::{Chunk, Coord};

/*
 * Simple struct for renderer errors
 */
#[derive(Debug)]
pub struct RenderContextError;

/*
 * Non-send main thread resource that contains the
 * first version renderer (non-system based)
 */
pub struct RenderContext<T: Context> {
    context: T,
}

impl <T: Context> RenderContext<T> {
    pub fn new(
        context: T
    ) -> Self {
        Self {
            context: context
        }
    }

    pub fn extent(&self) -> ImageExtent {
        self.context.image_extent()
    }

    pub fn recreate_window(&mut self) -> Result<(), RenderContextError> {
        self.context.window_resize().map_err(|err| {
            println!("Error resizing window {:?}", err);
            RenderContextError
        })
    }
}

fn compute_camera<T: Context>(
    render_context: &NonSendMut<RenderContext<T>>,
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
    chunk_coord: Vector3<f32>,
    pvt_matrix: Matrix4<f32>
) -> bool {
    let point = glm::vec4(
        chunk_coord.x,
        chunk_coord.y,
        chunk_coord.z,
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
    chunk_coord: &Transform,
    pvt_matrix: Matrix4<f32>
) -> bool {
    point_in_frustum(chunk_coord.translation + glm::vec3(0.0, 0.0, 0.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(16.0, 0.0, 0.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(0.0, 16.0, 0.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(16.0, 16.0, 0.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(0.0, 0.0, 16.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(16.0, 0.0, 16.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(0.0, 16.0, 16.0), pvt_matrix) ||
    point_in_frustum(chunk_coord.translation + glm::vec3(16.0, 16.0, 16.0), pvt_matrix)
}

pub fn chunk_nearby_camera(
    camera_pos: Vector3<f32>,
    chunk_coord: &Transform
) -> bool {
    let chunk_center = glm::vec3(
        chunk_coord.translation.x as f32 + 16.0 * (0.5f32),
        chunk_coord.translation.y as f32 + 16.0 * (0.5f32),
        chunk_coord.translation.z as f32 + 16.0 * (0.5f32)
    );

    glm::distance(chunk_center, camera_pos) <= 16f32 * 2.0f32
}

fn translation_to_ubo(
    transform: &Transform
) -> UniformBufferObject {
    let s_x = transform.scale.x;
    let s_y = transform.scale.y;
    let s_z = transform.scale.z;

    let r_x = transform.rotation.x;
    let r_y = transform.rotation.y;
    let r_z = transform.rotation.z;
    let r_w = transform.rotation.w;

    let t_x = transform.translation.x;
    let t_y = transform.translation.y;
    let t_z = transform.translation.z;

    let model_matrix = glm::mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        t_x, t_y, t_z, 1.0
    ) * glm::mat4(
        1.0 - 2.0 * r_y * r_y - 2.0 * r_z * r_z, 2.0 * (r_x * r_y + r_z * r_w), 2.0 * (r_x * r_z - r_y * r_w), 0.0,
        2.0 * (r_x * r_y - r_z * r_w), 1.0 - 2.0 * r_x * r_x - 2.0 * r_z * r_z, 2.0 * (r_y * r_z + r_x * r_w), 0.0,
        2.0 * (r_x * r_z + r_y * r_w), 2.0 * (r_y * r_z - r_x * r_w), 1.0 - 2.0 * r_x * r_x - 2.0 * r_y * r_y, 0.0,
        0.0, 0.0, 0.0, 1.0
    ) * glm::mat4(
        s_x, 0.0, 0.0, 0.0,
        0.0, s_y, 0.0, 0.0,
        0.0, 0.0, s_z, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    UniformBufferObject {
        model: model_matrix,
        size: glm::vec3(16.0, 16.0, 16.0)
    }
}

/*
 * System renders the game
 */
pub fn render<T: Context>(
    mut render_context: NonSendMut<RenderContext<T>>,
    mut camera_data: Res<PlayerCamera>,
    renderables: Query<(&Transform, &Chunk)>
) {
    let camera = compute_camera(&render_context, &camera_data);
    let pvt_matrix = (camera.proj * camera.view).transpose();
    let frame = render_context.context.next_frame().unwrap();

    /* ---------------------------------------------------------------------------------------------------- */

    /*
     * Stage 0: Generate or cull data
     */
    let mut visible = Vec::new();

    for (transform, chunk) in renderables.iter() {
        if chunk.is_air() {
            continue;
        }

        if !chunk_in_frustum(transform, pvt_matrix) && !chunk_nearby_camera(camera_data.position(), transform) {
            continue;
        }

        visible.push((transform, chunk));
    }

    /*
     * Stage 1: Copy data into local buffers
     */

    let transfer_job = frame.create_transfer_job().unwrap();
    let resources = transfer_job.get_resources().unwrap();

    let mut i = 0;

    for (transform, chunk) in visible.iter() {
        unsafe { resources.buffer_voxel_data_cpu_side(WriteVoxelBuffer {
            offset: i,
            voxels: *chunk
        }).unwrap()};

        unsafe { resources.buffer_model_data_cpu_side(WriteModelBuffer {
            offset: i,
            model: translation_to_ubo(*transform)
        }).unwrap()};

        i += 1;
    }

    /*
     * Stage 2: Copy cpu to gpu (next frame?)
     */

    transfer_job.begin_job().unwrap();
    let mut command_buffer = transfer_job.create_command_buffer().unwrap();
    let voxel_buffer = resources.get_buffer_handle(BufferHandles::VoxelsCPU).unwrap();
    let voxel_gpu_buffer = resources.get_buffer_handle(BufferHandles::VoxelsGPU).unwrap();

    // TODO: Hardcoded chunk size
    command_buffer.copy_buffer(CopyBufferPacket {
        src: voxel_buffer,
        dst: voxel_gpu_buffer,
        src_offset: 0,
        dst_offset: 0,
        bytes: (visible.len() * 16 * 16 * 16 * std::mem::size_of::<Voxel>()) as u64
    }).unwrap();

    command_buffer.stage().unwrap();
    transfer_job.submit_command_buffer(command_buffer).unwrap();
    transfer_job.end_job().unwrap();
    transfer_job.run_job().unwrap();
    frame.submit_transfer_job(transfer_job).unwrap();

    /*
     * Stage 3: Issue draw calls
     */
    let draw_job = frame.create_draw_job().unwrap();
    draw_job.begin_job().unwrap();
    let mut draw_command_buffer = draw_job.create_command_buffer().unwrap();

    for i in 0..visible.len() {
        draw_command_buffer.draw(DrawPacket {
            camera: camera,
            model_uniform_index: i as u64,
            chunk_index: i as u64,
        }).unwrap();
    };

    draw_command_buffer.stage().unwrap();
    draw_job.submit_command_buffer(draw_command_buffer).unwrap();
    draw_job.end_job().unwrap();
    draw_job.run_job().unwrap();
    frame.submit_draw_job(draw_job).unwrap();

    /* ---------------------------------------------------------------------------------------------------- */
}