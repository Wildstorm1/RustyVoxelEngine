/*
 * Copyright (C) 2026 Charles Nitschke montyn09@gmail.com All Rights Reserved
 */

use super::super::super::hal::stucts::{ Vertex, Camera };
use super::super::hardware::device::{ VulkanDevice };
use super::super::hardware::swapchain::{ VulkanSwapchain };
use super::render_pass::{ VulkanRenderPass };
use std::ffi::{ CString };
use std::fs::{ File };
use std::path::{ Path };
use std::ptr;
use std::sync::{ Arc };
use core::mem::{ offset_of };
use ash::{ vk, util };

/*
 * Simple struct for initialization error
 */
#[derive(Debug)]
pub struct PipelineError;

/*
 * Vulkan implementation of a render pass
 */
pub struct VulkanPipeline {
    device: Arc<VulkanDevice>,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline
}

struct ShaderDetails {
    mesh_module: vk::ShaderModule,
    task_module: vk::ShaderModule,
    fragment_module: vk::ShaderModule
}

fn create_shader_module(
    device: Arc<VulkanDevice>,
    path: &str
) -> Result<vk::ShaderModule, PipelineError> {
    let mut shader_file = match File::open(Path::new(path)) {
        Ok(shader_file) => shader_file,
        Err(err) => {
            println!("Failed to open shader: {}", err);
            return Err(PipelineError);
        }
    };

    let shader = match util::read_spv(&mut shader_file) {
        Ok(shader) => shader,
        Err(err) => {
            println!("Failed to read shader: {}", err);
            return Err(PipelineError);
        }
    };

    let metadata = match shader_file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => {
            println!("Failed to get shader metadata: {}", err);
            return Err(PipelineError);
        }
    };

    let module_info = vk::ShaderModuleCreateInfo {
        code_size: metadata.len() as usize,
        p_code: shader.as_ptr(),
        ..Default::default()
    };

    let module = match unsafe { device.device.create_shader_module(&module_info, None) } {
        Ok(module) => module,
        Err(err) => {
            println!("Failed to create shader module: {}", err);
            return Err(PipelineError);
        }
    };

    Ok(module)
}

fn load_shaders(
    device: Arc<VulkanDevice>
) -> Result<ShaderDetails, PipelineError> {
    let mesh_module = create_shader_module(device.clone(), "./shaders/mesh.spv")?;
    let task_module = create_shader_module(device.clone(), "./shaders/task.spv")?;
    let fragment_module = create_shader_module(device.clone(), "./shaders/frag.spv")?;

    Ok(ShaderDetails {
        mesh_module: mesh_module,
        task_module: task_module,
        fragment_module: fragment_module,
    })
}

impl VulkanPipeline {
    pub fn new(
        device: Arc<VulkanDevice>,
        swap_chain: Arc<VulkanSwapchain>,
        render_pass: Arc<VulkanRenderPass>
    ) -> Result<Self, PipelineError> {
        println!("Loading shaders...");
        let shaders = load_shaders(device.clone())?;
        println!("Shaders loaded!");

        let range = vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::MESH_EXT,
            offset: 0,
            size: size_of::<Camera>() as u32,
            ..Default::default()
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &device.descriptor_set_layout,
            push_constant_range_count: 1,
            p_push_constant_ranges: &range,
            ..Default::default()
        };

        let pipeline_layout = match unsafe { device.device.create_pipeline_layout(&pipeline_layout_info, None) } {
            Ok(pipeline_layout) => pipeline_layout,
            Err(err) => {
                println!("Failed to create pipeline layout: {}", err);
                return Err(PipelineError);
            }
        };

        let main_string = match CString::new("main") {
            Ok(main) => main,
            Err(err) => {
                println!("Failed to create c main string: {}", err);
                return Err(PipelineError);
            }
        };

        let mesh_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::MESH_EXT,
            module: shaders.mesh_module,
            p_name: main_string.as_ptr(),
            ..Default::default()
        };

        let task_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::TASK_EXT,
            module: shaders.task_module,
            p_name: main_string.as_ptr(),
            ..Default::default()
        };

        let frag_shader_stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: shaders.fragment_module,
            p_name: main_string.as_ptr(),
            ..Default::default()
        };

        let shader_infos = vec![ mesh_shader_stage_info, task_shader_stage_info, frag_shader_stage_info ];
        let dynamic_states = vec![ vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR ];

        let dynamic_state = vk::PipelineDynamicStateCreateInfo {
            dynamic_state_count: dynamic_states.len() as u32,
            p_dynamic_states: dynamic_states.as_ptr(),
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: swap_chain.swap_chain_extent.width as f32,
            height: swap_chain.swap_chain_extent.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32,
            ..Default::default()
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0
            },
            extent: swap_chain.swap_chain_extent,
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0f32,
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            depth_bias_constant_factor: 0.0f32,
            depth_bias_clamp: 0.0f32,
            depth_bias_slope_factor: 0.0f32,
            ..Default::default()
        };

        let multi_sampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            min_sample_shading: 1.0f32,
            p_sample_mask: ptr::null(),
            alpha_to_coverage_enable: vk::FALSE,
            alpha_to_one_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
            ..Default::default()
        };

        let color_blending = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            blend_constants: [ 0.0f32, 0.0f32, 0.0f32, 0.0f32 ],
            ..Default::default()
        };

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            min_depth_bounds: 0.0f32,
            max_depth_bounds: 1.0f32,
            stencil_test_enable: vk::FALSE,
            front: vk::StencilOpState::default(),
            back: vk::StencilOpState::default(),
            ..Default::default()
        };

        let pipeline_infos = vec![
            vk::GraphicsPipelineCreateInfo {
                stage_count: 3,
                p_stages: shader_infos.as_ptr(),
                p_viewport_state: &viewport_state,
                p_rasterization_state: &rasterizer,
                p_multisample_state: &multi_sampling,
                p_depth_stencil_state: &depth_stencil,
                p_color_blend_state: &color_blending,
                p_dynamic_state: &dynamic_state,
                layout: pipeline_layout,
                render_pass: render_pass.render_pass,
                subpass: 0,
                base_pipeline_index: -1,
                ..Default::default()
            }
        ];

        let pipeline = match unsafe { device.device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None) } {
            Ok(pipeline) => pipeline[0],
            Err((_vec, err)) => {
                println!("Failed to create graphics pipeline: {}", err);
                return Err(PipelineError);
            }
        };

        unsafe {
            device.device.destroy_shader_module(shaders.mesh_module, None);
            device.device.destroy_shader_module(shaders.task_module, None);
            device.device.destroy_shader_module(shaders.fragment_module, None);
        }

        Ok(Self {
            device: device,
            pipeline_layout: pipeline_layout,
            pipeline: pipeline
        })
    }
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        println!("Destroying pipeline wrapper");

        unsafe {
            println!("Destroying vulkan pipeline: {:?}", self.pipeline);
            self.device.device.destroy_pipeline(self.pipeline, None);
            println!("Destroying vulkan pipeline layout: {:?}", self.pipeline_layout);
            self.device.device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}