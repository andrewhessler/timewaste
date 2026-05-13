use cgmath::Vector3;
use wgpu::{
    ColorWrites, CommandEncoder, Device, MultisampleState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    SurfaceConfiguration, TextureView, VertexAttribute, VertexBufferLayout,
};
use winit::event::KeyEvent;

use crate::cube::Cube;

pub fn projection(width: f32, height: f32, depth: f32) -> cgmath::Matrix4<f32> {
    let res_matrix = cgmath::Matrix4::from_nonuniform_scale(1. / width, 1. / height, 0.5 / depth);
    let zero_to_two = cgmath::Matrix4::from_scale(2.);
    let flip_clip = cgmath::Matrix4::from_translation(Vector3::new(-1., -1., -1.));
    let clip = cgmath::Matrix4::from_nonuniform_scale(1., -1., 1.);
    res_matrix * zero_to_two * flip_clip * clip
}

pub struct WorldRenderer {
    pipeline: RenderPipeline,
    cube: Cube,
}

impl WorldRenderer {
    pub fn new(device: &Device) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("silly shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("the render pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                entry_point: Some("vs"),
                module: &shader,
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    step_mode: wgpu::VertexStepMode::Vertex,
                    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
                    attributes: &[VertexAttribute {
                        shader_location: 0,
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                entry_point: Some("fs"),
                module: &shader,
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::default(),
                })],
            }),
            cache: None,
            depth_stencil: None,
            multisample: MultisampleState {
                ..Default::default()
            },
            primitive: Default::default(),
            multiview_mask: Default::default(),
        });

        let cube = Cube::new(device, &pipeline);

        Self { pipeline, cube }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        queue: &Queue,
        delta_time: Option<f32>,
        config: &SurfaceConfiguration,
    ) {
        if let Some(delta_time) = delta_time {
            self.cube.animate(queue, config, delta_time);
        }
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                ops: Default::default(),
                resolve_target: None,
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.cube.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.cube.vertex_buf.slice(..));
        pass.set_index_buffer(self.cube.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.cube.num_vertices, 0, 0..1);
    }

    pub fn handle_input(&mut self, event: &KeyEvent) {
        self.cube.handle_input(event);
    }
}
