/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::vulkan::hardware::device::{ VulkanDevice };
use crate::graphics::vulkan::hardware::swapchain::{ VulkanSwapchain };
use crate::graphics::hal::command_buffer::{ CommandBuffer, DrawIndexedPacket };
use crate::graphics::hal::context::{ Context, CommandBufferArgs, ContextError };
use crate::graphics::hal::resources::{ Resources };
use crate::graphics::hal::frame::{ Frame };
use crate::graphics::vulkan::context::{ VulkanContext };
use sdl3::video::Window;
use std::sync::Arc;
use std::thread;

/*
 * Simple struct for rendering errors
 */
#[derive(Debug)]
pub struct RendererError;

pub struct Renderer<T: Context> {
    context: Box<T>,
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

        frame.begin_frame();

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

        /*
         * Write data
         */

        /*
         * Draw commands
         */

        frame.end_frame();
        frame.present_frame();

        Ok(())
    }
}

/*
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