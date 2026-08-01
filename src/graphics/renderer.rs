/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::hal::command_buffer::{ CommandBuffer, CopyBufferPacket, DrawIndexedPacket };
use crate::graphics::hal::context::{ Context };
use crate::graphics::hal::resources::{ Resources, WriteVertexBuffer, WriteIndexBuffer, WriteModelBuffer, BufferHandles, UniformHandles };
use crate::graphics::hal::frame::{ Frame };
use crate::graphics::hal::job::{ Job };
use crate::graphics::hal::stucts::{ Vertex, UniformBufferObject };

/*
 * Simple struct for rendering errors
 */
#[derive(Debug)]
pub struct RendererError;

pub struct Renderer<T: Context> {
    context: Box<T>,
}

fn push_face(
    vertex_0: glm::Vec3,
    vertex_1: glm::Vec3,
    vertex_2: glm::Vec3,
    vertex_3: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let index = vertices.len() as u32;
    vertices.push(Vertex { pos: vertex_0, color: glm::vec3(1.0f32, 0.0f32, 0.0f32) });
    vertices.push(Vertex { pos: vertex_1, color: glm::vec3(0.0f32, 1.0f32, 0.0f32) });
    vertices.push(Vertex { pos: vertex_2, color: glm::vec3(0.0f32, 0.0f32, 1.0f32) });
    vertices.push(Vertex { pos: vertex_3, color: glm::vec3(1.0f32, 1.0f32, 1.0f32) });
    indices.push(index);
    indices.push(index + 1);
    indices.push(index + 2);
    indices.push(index + 2);
    indices.push(index + 1);
    indices.push(index + 3);
}

fn push_right_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_front_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_bottom_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_left_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 0.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_back_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 0.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn push_top_face(
    voxel_offset: glm::Vec3,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 0.0f32);
    let v1 = voxel_offset + glm::vec3(0.0f32, 1.0f32, 1.0f32);
    let v2 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 0.0f32);
    let v3 = voxel_offset + glm::vec3(1.0f32, 1.0f32, 1.0f32);
    push_face(v0, v1, v2, v3, vertices, indices);
}

fn create_cube(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>
) {
    let v0 = glm::vec3(0.0f32, 0.0f32, 0.0f32);
    push_right_face(v0, vertices, indices);
    push_left_face(v0, vertices, indices);
    push_front_face(v0, vertices, indices);
    push_back_face(v0, vertices, indices);
    push_top_face(v0, vertices, indices);
    push_bottom_face(v0, vertices, indices);
}

impl <T: Context> Renderer<T> {
    pub fn new(context: Box<T>) -> Self {
        Self {
            context: context,
        }
    }

    pub fn render(&mut self) -> Result<(), RendererError> {
        let frame = self.context.next_frame().map_err(|err| {
            println!("Error getting next frame {:?}", err);
            RendererError
        })?;

        /* ---------------------------------------------------------------------------------------------------- */

        /*
         * Stage 0: Generate or cull data
         */

        let mut vertex_list = Vec::new();
        let mut index_list = Vec::new();
        create_cube(&mut vertex_list, &mut index_list);

        /*
         * Stage 1: Copy data into local buffers
         */

        let transfer_job = frame.create_transfer_job().map_err(|err| {
            println!("Error creating transfer job {:?}", err);
            RendererError
        })?;

        let resources = transfer_job.get_resources().map_err(|err| {
            println!("Error getting resources api {:?}", err);
            RendererError
        })?;

        unsafe { resources.buffer_vertex_data_cpu_side(WriteVertexBuffer {
            offset: 0,
            vertices: &vertex_list
        }).map_err(|err| {
            println!("Error buffering vertices {:?}", err);
            RendererError
        }) }?;

        unsafe { resources.buffer_index_data_cpu_side(WriteIndexBuffer {
            offset: 0,
            indices: &index_list
        }).map_err(|err| {
            println!("Error buffering indices {:?}", err);
            RendererError
        }) }?;

        let model_matrix = UniformBufferObject {
            model: glm::mat4(
                1.0f32, 0.0f32, 0.0f32, 0.0f32,
                0.0f32, 1.0f32, 0.0f32, 0.0f32,
                0.0f32, 0.0f32, 1.0f32, 0.0f32,
                0.0f32, 0.0f32, 0.0f32, 1.0f32
            )
        };

        unsafe { resources.buffer_model_data_cpu_side(WriteModelBuffer {
            offset: 0,
            model: model_matrix
        }).map_err(|err| {
            println!("Error buffering model {:?}", err);
            RendererError
        }) }?;

        /*
         * Stage 2: Copy cpu to gpu (next frame?)
         */

        transfer_job.begin_job().map_err(|err| {
            println!("Error beginning the sync job {:?}", err);
            RendererError
        })?;

        let mut command_buffer = transfer_job.create_command_buffer().map_err(|err| {
            println!("Error creating command buffer {:?}", err);
            RendererError
        })?;

        let vertex_buffer = resources.get_buffer_handle(BufferHandles::VertexCPU).map_err(|err| {
            println!("Error getting vertex buffer {:?}", err);
            RendererError
        })?;

        let vertex_gpu_buffer = resources.get_buffer_handle(BufferHandles::VertexGPU).map_err(|err| {
            println!("Error getting vertex buffer {:?}", err);
            RendererError
        })?;

        command_buffer.copy_buffer(CopyBufferPacket {
            src: vertex_buffer,
            dst: vertex_gpu_buffer,
            src_offset: 0,
            dst_offset: 0,
            bytes: (vertex_list.len() * std::mem::size_of::<Vertex>()) as u64
        }).map_err(|err| {
            println!("Error submitting copy buffer command {:?}", err);
            RendererError
        })?;

        let index_buffer = resources.get_buffer_handle(BufferHandles::IndexCPU).map_err(|err| {
            println!("Error getting index buffer {:?}", err);
            RendererError
        })?;

        let index_gpu_buffer = resources.get_buffer_handle(BufferHandles::IndexGPU).map_err(|err| {
            println!("Error getting index buffer {:?}", err);
            RendererError
        })?;

        command_buffer.copy_buffer(CopyBufferPacket {
            src: index_buffer,
            dst: index_gpu_buffer,
            src_offset: 0,
            dst_offset: 0,
            bytes: (index_list.len() * std::mem::size_of::<u32>()) as u64
        }).map_err(|err| {
            println!("Error submitting copy buffer command {:?}", err);
            RendererError
        })?;

        command_buffer.stage().map_err(|err| {
            println!("Error staging command buffer {:?}", err);
            RendererError
        })?;

        transfer_job.submit_command_buffer(command_buffer).map_err(|err| {
            println!("Error submitting command buffer {:?}", err);
            RendererError
        })?;

        transfer_job.end_job().map_err(|err| {
            println!("Error ending the sync job {:?}", err);
            RendererError
        })?;

        transfer_job.run_job().map_err(|err| {
            println!("Error running the sync job {:?}", err);
            RendererError
        })?;

        frame.submit_transfer_job(transfer_job).map_err(|err| {
            println!("Error submitting the sync job {:?}", err);
            RendererError
        })?;

        /*
         * Stage 3: Issue draw calls
         */

        /*
        frame.begin_frame().map_err(|err| {
            println!("Error beginning frame {:?}", err);
            RendererError
        })?;
        */

        /*
        match command_buffer.draw_indexed(DrawIndexedPacket {
            vertex: vertex_buffer,
            index: index_buffer,
            uniform: uniform_buffer,
            vertex_offset: 0,
            index_offset: 0,
            size: 36,
        }) {
            Ok(result) => result,
            Err(err) => {
                println!("Error recording command buffer {:?}", err);
                return Err(RendererError);
            }
        };
        */

        /*
        frame.end_frame().map_err(|err| {
            println!("Error ending frame {:?}", err);
            RendererError
        })?;

        frame.present_frame().map_err(|err| {
            println!("Error presenting frame {:?}", err);
            RendererError
        })?;
        */

        /* ---------------------------------------------------------------------------------------------------- */

        Ok(())
    }

    pub fn window_resize(&mut self) -> Result<(), RendererError> {
        self.context.window_resize().map_err(|err| {
            println!("Error resizing window {:?}", err);
            RendererError
        })?;

        Ok(())
    }
}
