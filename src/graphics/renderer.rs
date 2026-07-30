/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::hal::command_buffer::{ CommandBuffer, DrawIndexedPacket };
use crate::graphics::hal::context::{ Context };
use crate::graphics::hal::resources::{ Resources };
use crate::graphics::hal::frame::{ Frame, CommandBufferArgs };
use crate::graphics::hal::stucts::{ Vertex };

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

        frame.begin_frame().map_err(|err| {
            println!("Error beginning frame {:?}", err);
            RendererError
        })?;

        let mut command_buffer = frame.create_command_buffer(CommandBufferArgs {}).map_err(|err| {
            println!("Error creating command buffer {:?}", err);
            RendererError
        })?;

        let resources = frame.get_resources().map_err(|err| {
            println!("Error getting resources api {:?}", err);
            RendererError
        })?;

        let vertex_buffer = resources.get_vertex_buffer_handle().map_err(|err| {
            println!("Error getting vertex buffer {:?}", err);
            RendererError
        })?;

        let index_buffer = resources.get_index_buffer_handle().map_err(|err| {
            println!("Error getting index buffer {:?}", err);
            RendererError
        })?;

        let uniform_buffer = resources.get_model_buffer_handle().map_err(|err| {
            println!("Error getting uniform buffer {:?}", err);
            RendererError
        })?;

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

        command_buffer.stage().map_err(|err| {
            println!("Error staging command buffer {:?}", err);
            RendererError
        })?;

        frame.submit_command_buffer(command_buffer).map_err(|err| {
            println!("Error submitting command buffer {:?}", err);
            RendererError
        })?;

        frame.end_frame().map_err(|err| {
            println!("Error ending frame {:?}", err);
            RendererError
        })?;

        frame.present_frame().map_err(|err| {
            println!("Error presenting frame {:?}", err);
            RendererError
        })?;

        Ok(())
    }
}

/*
        /*
        let resources = frame.get_resources().map_err(|err| {
            println!("Error getting resources api {:?}", err);
            RendererError
        })?;

        let vertex_buffer = resources.get_vertex_buffer_handle().map_err(|err| {
            println!("Error getting vertex buffer {:?}", err);
            RendererError
        })?;

        let index_buffer = resources.get_index_buffer_handle().map_err(|err| {
            println!("Error getting index buffer {:?}", err);
            RendererError
        })?;

        let uniform_buffer = resources.get_model_buffer_handle().map_err(|err| {
            println!("Error getting uniform buffer {:?}", err);
            RendererError
        })?;
        */

    let mut command_buffer = match context.create_command_buffer(CommandBufferArgs {}) {
        Ok(command_buffer) => command_buffer,
        Err(err) => {
            println!("Error creating command buffer {:?}", err);
            return Err(ContextError);
        }
    };

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
            return Err(ContextError);
        }
    };

    context.end_frame();
    context.present_frame();
*/

    /*
    let vertex_buffer = match context.create_buffer(BufferArgs {
        size: 1000000,
        usage: BufferUsage(BufferUsage::VERTEX_BUFFER),
        properties: BufferMemoryProperties(BufferMemoryProperties::DEVICE_LOCAL)
    }) {
        Ok(vertex_buffer) => vertex_buffer,
        Err(err) => {
            println!("Error creating vulkan vertex buffer {:?}", err);
            return;
        }
    };

    let index_buffer = match context.create_buffer(BufferArgs {
        size: 1000000,
        usage: BufferUsage(BufferUsage::INDEX_BUFFER),
        properties: BufferMemoryProperties(BufferMemoryProperties::DEVICE_LOCAL)
    }) {
        Ok(index_buffer) => index_buffer,
        Err(err) => {
            println!("Error creating vulkan index buffer {:?}", err);
            return;
        }
    };

    let vb1 = vertex_buffer.clone();
    let vb2 = vertex_buffer.clone();
    let ib1 = index_buffer.clone();
    let ib2 = index_buffer.clone();

    let context1 = context.clone();
    let context2 = context.clone();

    let mut command_buffer_1 = match context1.create_command_buffer(CommandBufferArgs {}) {
        Ok(command_buffer) => command_buffer,
        Err(err) => {
            println!("Error creating vulkan command buffer {:?}", err);
            return;
        }
    };    

    let handle1 = thread::spawn(move || {
        command_buffer_1.draw_indexed(DrawIndexedPacket {
            vertex: vb1,
            index: ib1,
            vertex_offset: 0,
            index_offset: 0,
            size: 2000,
        });
    });

    let mut command_buffer_2 = match context2.create_command_buffer(CommandBufferArgs {}) {
        Ok(command_buffer) => command_buffer,
        Err(err) => {
            println!("Error creating vulkan command buffer {:?}", err);
            return;
        }
    };

    let handle2 = thread::spawn(move || {
        command_buffer_2.draw_indexed(DrawIndexedPacket {
            vertex: vb2,
            index: ib2,
            vertex_offset: 0,
            index_offset: 0,
            size: 2000,
        });
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
    */