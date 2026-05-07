use std::num::NonZeroU64;
use std::sync::Arc;

use anyhow::Result;
use cgmath::Vector3;
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use image::EncodableLayout;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferUsages, ColorWrites,
    CurrentSurfaceTexture, Device, Instance, InstanceDescriptor, MultisampleState, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModuleDescriptor, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages, VertexAttribute, VertexBufferLayout,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::{
    shape_util::{CircleVerticesInput, create_circle_vertices, create_f_vertices},
    ui::EguiRenderer,
};

mod handle_input;
mod shape_util;
mod ui;

const TRANSLATION_SPEED: f32 = 100.0;
const ROTATION_SPEED: f32 = 100.0;
const SCALE_SPEED: f32 = 1.0;

#[derive(PartialEq, Debug)]
enum Direction {
    Inc,
    Dec,
    None,
}

struct Translation {
    x: f32,
    y: f32,
    z: f32,
    x_speed: f32,
    y_speed: f32,
    z_speed: f32,
    x_direction: Direction,
    y_direction: Direction,
    z_direction: Direction,
}

#[derive(Debug)]
struct Rotation {
    angle: f32,
    direction: Direction,
}

#[derive(Debug)]
struct Scale {
    x: f32,
    y: f32,
    z: f32,
    x_direction: Direction,
    y_direction: Direction,
    z_direction: Direction,
}

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    window: Arc<Window>,
    num_vertices: u32,
    uniform_buf: Buffer,
    uniform_bind_group: BindGroup,
    vertex_buf: Buffer,
    index_buf: Buffer,
    translation: Translation,
    rotation: Rotation,
    scale: Scale,
    egui_renderer: ui::EguiRenderer,
    last_frame_time: Option<std::time::Instant>,
}

fn projection(width: f32, height: f32, depth: f32) -> cgmath::Matrix4<f32> {
    let res_matrix = cgmath::Matrix4::from_nonuniform_scale(1. / width, 1. / height, 0.5 / depth);
    let zero_to_two = cgmath::Matrix4::from_scale(2.);
    let flip_clip = cgmath::Matrix4::from_translation(Vector3::new(-1., -1., -1.));
    let clip = cgmath::Matrix4::from_nonuniform_scale(1., -1., 1.);
    res_matrix * zero_to_two * flip_clip * clip
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Default::default(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: Default::default(),
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                ..Default::default()
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let size = window.inner_size();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

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

        let translation = Translation {
            x: 0.,
            y: 0.,
            z: 0.,
            x_speed: 0.,
            y_speed: 0.,
            z_speed: 0.,
            x_direction: Direction::None,
            y_direction: Direction::None,
            z_direction: Direction::None,
        };

        let rotation = Rotation {
            angle: 0.,
            direction: Direction::None,
        };

        let scale = Scale {
            x: 1.,
            y: 1.,
            z: 1.,
            x_direction: Direction::None,
            y_direction: Direction::None,
            z_direction: Direction::None,
        };

        let mut contents: Vec<f32> = vec![
            1.0, 0.2, 0.2, 1.0, //color
        ];

        contents.extend([0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.]);
        contents.extend([0., 0., 0., 0., 0., 0., 0.]);

        let uniform_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("uniform buffer"),
            contents: contents.as_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let (vertex_data, index_data, num_vertices) = create_f_vertices();

        let vertex_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: vertex_data.as_bytes(),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let index_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        let egui_renderer = EguiRenderer::new(
            &device,
            TextureFormat::Bgra8UnormSrgb,
            RendererOptions::PREDICTABLE,
            &window,
        );

        Ok(Self {
            device,
            queue,
            window,
            pipeline,
            surface,
            config,
            num_vertices,
            uniform_buf,
            uniform_bind_group,
            vertex_buf,
            index_buf,
            translation,
            rotation,
            scale,
            egui_renderer,
            last_frame_time: None,
        })
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl App {
    fn render(&mut self) -> anyhow::Result<()> {
        let state = self.state.as_mut().unwrap();
        let now = std::time::Instant::now();

        if let Some(lft) = state.last_frame_time {
            let delta_time = now - lft;
            let delta_time_f32 = delta_time.as_secs_f32();
            let x_speed_step = state.translation.x_speed * delta_time_f32;

            state.translation.x += x_speed_step;
            state.translation.y += state.translation.y_speed * delta_time_f32;
            state.translation.z += state.translation.z_speed * delta_time_f32;

            state.rotation.angle += match state.rotation.direction {
                Direction::Inc => ROTATION_SPEED * delta_time_f32,
                Direction::Dec => -ROTATION_SPEED * delta_time_f32,
                _ => 0.,
            };

            state.scale.x += match state.scale.x_direction {
                Direction::Inc => SCALE_SPEED * delta_time_f32,
                Direction::Dec => -SCALE_SPEED * delta_time_f32,
                _ => 0.,
            };

            state.scale.y += match state.scale.y_direction {
                Direction::Inc => SCALE_SPEED * delta_time_f32,
                Direction::Dec => -SCALE_SPEED * delta_time_f32,
                _ => 0.,
            };

            state.scale.z += match state.scale.z_direction {
                Direction::Inc => SCALE_SPEED * delta_time_f32,
                Direction::Dec => -SCALE_SPEED * delta_time_f32,
                _ => 0.,
            };

            let _fps = 1.0 / delta_time.as_secs_f64();
            // println!("delta_time: {:?}, fps: {:?}", delta_time, fps);
        }

        state.last_frame_time = Some(now);

        {
            let mut temp_buf = state
                .queue
                .write_buffer_with(
                    &state.uniform_buf,
                    std::mem::size_of::<[f32; 4]>() as u64,
                    NonZeroU64::new(std::mem::size_of::<[f32; 16]>() as u64).unwrap(),
                )
                .unwrap();

            let projection = projection(
                state.config.width as f32,
                state.config.height as f32,
                state.config.width as f32,
            );
            let trans_matrix = cgmath::Matrix4::from_translation(Vector3::new(
                state.translation.x,
                state.translation.y,
                state.translation.z,
            ));
            let scale_matrix =
                cgmath::Matrix4::from_nonuniform_scale(state.scale.x, state.scale.y, state.scale.z);
            let rotat_matrix = cgmath::Matrix4::from_angle_z(cgmath::Deg(state.rotation.angle));
            let move_origin = cgmath::Matrix4::from_translation(Vector3::new(-50., -75., -50.));

            let matrix = projection * trans_matrix * scale_matrix * rotat_matrix * move_origin;

            let uniform_vals: [[f32; 4]; 4] = matrix.into();
            let mut res: Vec<f32> = vec![];

            for row in uniform_vals {
                res.extend(row.iter());
                // res.push(0.);
            }

            temp_buf.copy_from_slice(res.as_bytes());
        }

        let mut encoder = state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("command encoder"),
            });

        let texture = match state.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Suboptimal(e) => {
                println!("Reached suboptimal");
                e
            }
            CurrentSurfaceTexture::Occluded => return Ok(()),
            e => panic!("Oops, texture goofed {e:?}"),
        };

        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.config.width, state.config.height],
            pixels_per_point: 1.,
        };

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    ops: Default::default(),
                    resolve_target: None,
                })],
                ..Default::default()
            });
            pass.set_pipeline(&state.pipeline);
            pass.set_bind_group(0, &state.uniform_bind_group, &[]);
            pass.set_vertex_buffer(0, state.vertex_buf.slice(..));
            pass.set_index_buffer(state.index_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..state.num_vertices, 0, 0..1);
        }

        {
            let input = state.egui_renderer.state.take_egui_input(&state.window);
            state.egui_renderer.state.egui_ctx().begin_pass(input);

            egui::Window::new("test").resizable(true).show(
                state.egui_renderer.state.egui_ctx(),
                |ui| {
                    ui.label("Hello");
                },
            );

            state
                .egui_renderer
                .state
                .egui_ctx()
                .set_pixels_per_point(screen_descriptor.pixels_per_point);

            let full_output = state.egui_renderer.state.egui_ctx().end_pass();
            state
                .egui_renderer
                .state
                .handle_platform_output(&state.window, full_output.platform_output);

            let tris = state.egui_renderer.state.egui_ctx().tessellate(
                full_output.shapes,
                state.egui_renderer.state.egui_ctx().pixels_per_point(),
            );

            for (id, image_delta) in &full_output.textures_delta.set {
                state.egui_renderer.renderer.update_texture(
                    &state.device,
                    &state.queue,
                    *id,
                    image_delta,
                );
            }

            state.egui_renderer.renderer.update_buffers(
                &state.device,
                &state.queue,
                &mut encoder,
                &tris,
                &screen_descriptor,
            );

            let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    ops: Default::default(),
                    resolve_target: None,
                })],
                ..Default::default()
            });

            state.egui_renderer.renderer.render(
                &mut pass.forget_lifetime(),
                &tris,
                &screen_descriptor,
            );

            for id in full_output.textures_delta.free {
                state.egui_renderer.renderer.free_texture(&id);
            }
        }

        let command_buf = encoder.finish();
        state.queue.submit([command_buf]);

        texture.present();
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = self.state.as_mut().unwrap();

        let _ = state
            .egui_renderer
            .state
            .on_window_event(&state.window, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                state.window.request_redraw();
                match self.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Unable to render {e}");
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                state.config.width = new_size.width;
                state.config.height = new_size.height;
                state.surface.configure(&state.device, &state.config);
            }
            WindowEvent::KeyboardInput {
                device_id: _device_id,
                is_synthetic: _is_synthetic,
                event,
            } => {
                handle_input::handle_input(event_loop, &event, state);
            }
            _ => (),
        }
    }
}

pub fn run() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    create_circle_vertices(CircleVerticesInput {
        ..Default::default()
    });

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
