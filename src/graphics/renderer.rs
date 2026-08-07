/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use crate::graphics::hal::command_buffer::{ CommandBuffer, CopyBufferPacket, DrawPacket };
use crate::graphics::hal::context::{ Context, ImageExtent };
use crate::graphics::hal::resources::{ BufferHandles, Resources, WriteModelBuffer, WriteVoxelBuffer };
use crate::graphics::hal::frame::{ Frame };
use crate::graphics::hal::job::{ Job };
use crate::graphics::hal::stucts::{ UniformBufferObject, Camera, Voxel };
use crate::voxels::{ Chunk, ChunkCache, ChunkCoord };
use std::sync::Arc;

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

    pub fn extent(&self) -> Result<ImageExtent, RendererError> {
        self.context.image_extent().map_err(|_| { RendererError })
    }

    pub fn render(&mut self, camera: Camera, chunks: &mut ChunkCache, coords: Arc<Vec<ChunkCoord>>) -> Result<(), RendererError> {
        let frame = self.context.next_frame().map_err(|err| {
            println!("Error getting next frame {:?}", err);
            RendererError
        })?;

        /* ---------------------------------------------------------------------------------------------------- */

        /*
         * Stage 0: Generate or cull data
         */

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

        coords.iter().enumerate().for_each(|(i, chunk)| {
            unsafe { resources.buffer_voxel_data_cpu_side(WriteVoxelBuffer {
                offset: i,
                voxels: chunks.get_chunk_mut(*chunk).unwrap()
            }).unwrap()};

            unsafe { resources.buffer_model_data_cpu_side(WriteModelBuffer {
                offset: i,
                model: *chunks.get_ubo(*chunk).unwrap()
            }).unwrap()};
        });

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

        let voxel_buffer = resources.get_buffer_handle(BufferHandles::VoxelsCPU).map_err(|err| {
            println!("Error getting voxel buffer {:?}", err);
            RendererError
        })?;

        let voxel_gpu_buffer = resources.get_buffer_handle(BufferHandles::VoxelsGPU).map_err(|err| {
            println!("Error getting voxel buffer {:?}", err);
            RendererError
        })?;

        command_buffer.copy_buffer(CopyBufferPacket {
            src: voxel_buffer,
            dst: voxel_gpu_buffer,
            src_offset: 0,
            dst_offset: 0,
            bytes: (coords.len() * 16 * 16 * 16 * std::mem::size_of::<Voxel>()) as u64
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

        let draw_job = frame.create_draw_job().map_err(|err| {
            println!("Error creating draw job {:?}", err);
            RendererError
        })?;

        draw_job.begin_job().map_err(|err| {
            println!("Error beginning the draw job {:?}", err);
            RendererError
        })?;

        let mut draw_command_buffer = draw_job.create_command_buffer().map_err(|err| {
            println!("Error creating draw command buffer {:?}", err);
            RendererError
        })?;

        coords.iter().enumerate().for_each(|(i, _)| {
            draw_command_buffer.draw(DrawPacket {
                camera: camera,
                model_uniform_index: i as u64,
                chunk_index: i as u64,
            }).unwrap();
        });

        draw_command_buffer.stage().map_err(|err| {
            println!("Error staging draw command buffer {:?}", err);
            RendererError
        })?;

        draw_job.submit_command_buffer(draw_command_buffer).map_err(|err| {
            println!("Error submitting draw command buffer {:?}", err);
            RendererError
        })?;

        draw_job.end_job().map_err(|err| {
            println!("Error ending draw job {:?}", err);
            RendererError
        })?;

        draw_job.run_job().map_err(|err| {
            println!("Error running draw job {:?}", err);
            RendererError
        })?;

        frame.submit_draw_job(draw_job).map_err(|err| {
            println!("Error submitting the draw job {:?}", err);
            RendererError
        })?;

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
