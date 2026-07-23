/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::vulkan::hardware::device::{ VulkanDevice };
use crate::graphics::vulkan::hardware::swapchain::{ VulkanSwapchain };
use crate::graphics::hal::command_buffer::{ CommandBuffer, DrawIndexedPacket };
use crate::graphics::hal::context::{ Context, CommandBufferArgs, BufferArgs, BufferUsage, BufferMemoryProperties };
use crate::graphics::vulkan::context::{ VulkanContext };
use sdl3::video::Window;
use std::sync::Arc;
use std::thread;

pub fn test(window: &Window) {
    let device = unsafe { match VulkanDevice::new(&window) {
        Ok(device) => device,
        Err(err) => {
            println!("Error creating vulkan device {:?}", err);
            return;
        }
    } };

    let swap_chain = unsafe { match VulkanSwapchain::new(&device, &window) {
        Ok(swap_chain) => swap_chain,
        Err(err) => {
            println!("Error creating vulkan swapchain {:?}", err);
            return;
        }
    } };

    let mut context = Arc::new(VulkanContext::new(device));

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
}