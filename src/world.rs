use std::sync::{Arc, atomic::AtomicBool};

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, Buffer, BufferDescriptor,
    BufferUsages, ColorWrites, CommandEncoder, Device, MultisampleState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, SurfaceConfiguration, TextureView, VertexAttribute, VertexBufferLayout,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::event::KeyEvent;

use crate::cube::Cube;

pub fn projection(width: f32, height: f32, depth: f32) -> na::Matrix4<f32> {
    #[rustfmt::skip]
    let resolution = na::Matrix4::new(
        1. / width, 0., 0., 0.,
        0., 1. / height, 0., 0.,
        0., 0., 0.5 / depth, 0.,
        0., 0., 0., 1.,
    );

    #[rustfmt::skip]
    let scale_to_two = na::Matrix4::new(
        2., 0., 0., 0.,
        0., 2., 0., 0.,
        0., 0., 2., 0.,
        0., 0., 0., 1.,
    );

    #[rustfmt::skip]
    let flip_clip = na::Matrix4::new(
        1., 0., 0., 0.,
        0., -1., 0., 0.,
        0., 0., 1., 0.,
        0., 0., 0., 1.,
    );

    resolution * scale_to_two * flip_clip
}

pub struct WorldRenderer {
    pipeline: RenderPipeline,
    cube: Cube,
    debug_buf: Arc<Buffer>,
    debug_bg: BindGroup,
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
                        format: wgpu::VertexFormat::Float32x3,
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

        let debug_buf = device.create_buffer(&BufferDescriptor {
            label: Some("debug storage buffer"),
            size: std::mem::size_of::<[f32; 1024]>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let debug_bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("debug bind group"),
            layout: &pipeline.get_bind_group_layout(1),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: debug_buf.as_entire_binding(),
            }],
        });

        Self {
            pipeline,
            cube,
            debug_buf: debug_buf.into(),
            debug_bg,
        }
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
        pass.set_bind_group(1, &self.debug_bg, &[]);
        pass.set_vertex_buffer(0, self.cube.vertex_buf.slice(..));
        pass.set_index_buffer(self.cube.index_buf.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..self.cube.num_vertices, 0, 0..1);
    }

    pub fn handle_input(&mut self, event: &KeyEvent) {
        self.cube.handle_input(event);
    }

    pub fn read_debug_buffer(&mut self, encoder: &CommandEncoder) {
        let buffer_clone = self.debug_buf.clone();
        encoder.map_buffer_on_submit(&self.debug_buf, wgpu::MapMode::Read, 0..4096, move |res| {
            match res {
                Ok(_) => {
                    {
                        let contents = buffer_clone.get_mapped_range(0..128);
                        unsafe {
                            let rand_val: u8 = rand::random();
                            if rand_val < 12 {
                                println!("{rand_val} : {:?}", contents.align_to::<f32>());
                            }
                        }
                    }
                    buffer_clone.unmap();
                }
                Err(e) => {
                    println!("did an oops: {:?}", e);
                }
            }
        });
    }

    pub fn unmap_debug_buffer(&mut self) {
        self.debug_buf.unmap();
    }
}
