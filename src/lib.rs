use std::{
    mem,
    num::{NonZeroU32, NonZeroU64},
    sync::Arc,
    time::SystemTime,
};

use anyhow::Result;
use image::EncodableLayout;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages,
    ColorWrites, CurrentSurfaceTexture, Device, Instance, InstanceDescriptor, MultisampleState,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, Surface,
    SurfaceConfiguration, TextureUsages,
    util::{BufferInitDescriptor, DeviceExt},
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

struct TriangleProps {
    color_offset: f64,
    color_speed_offset: f64,
    x_offset: f32,
    y_offset: f32,
    height: f32,
    width: f32,
}

const TRIANGLE_COUNT: usize = 100;

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    window: Arc<Window>,
    triangle_props: Vec<TriangleProps>,
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
                buffers: &[],
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

        let mut triangle_props = Vec::new();

        for _ in 0..TRIANGLE_COUNT {
            triangle_props.push(TriangleProps {
                height: rand::random_range(0.0..1.0),
                width: rand::random_range(0.0..1.0),
                x_offset: rand::random_range(-0.9..0.9),
                y_offset: rand::random_range(-0.9..0.9),
                color_offset: rand::random_range(0.0..10000.0),
                color_speed_offset: rand::random_range(0.0..1000.0),
            });
        }

        Ok(Self {
            device,
            queue,
            window,
            pipeline,
            surface,
            config,
            triangle_props,
        })
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl App {
    fn render(&mut self) -> anyhow::Result<()> {
        let state = self.state.as_ref().unwrap();

        let mut bind_groups: Vec<BindGroup> = Vec::new();

        for i in 0..TRIANGLE_COUNT {
            let uniform_buf = state.device.create_buffer(&BufferDescriptor {
                label: Some(&format!("uniform buffer {i}")),
                size: (mem::size_of::<[f32; 4]>()
                    + mem::size_of::<[f32; 2]>()
                    + mem::size_of::<[f32; 2]>()) as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
                label: Some(&format!("bind group {i}")),
                layout: &state.pipeline.get_bind_group_layout(0),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
            });

            bind_groups.push(bind_group);

            let mut temp_buf = state
                .queue
                .write_buffer_with(&uniform_buf, 0, NonZeroU64::new(32).unwrap())
                .unwrap();

            let aspect = state.config.width as f32 / state.config.height as f32;

            let props = &state.triangle_props[i];
            let time_now = chrono::Utc::now().timestamp_millis() as f64;
            let base_color = time_now + props.color_offset;
            let r = ((base_color / (100.0 + props.color_speed_offset)).sin() + 1.0) / 2.0;
            let g = ((base_color / (400.0 + props.color_speed_offset)).sin() + 1.0) / 2.0;
            let b = ((base_color / (900.0 + props.color_speed_offset)).sin() + 1.0) / 2.0;

            // println!(
            //     "{} : {} : {} : {} : {}",
            //     time_now, g, aspect, state.config.width, state.config.height
            // );

            let offset_x = props.x_offset;
            let offset_y = props.y_offset;
            let width = props.width;
            let height = props.height;

            temp_buf.copy_from_slice(
                [
                    r as f32,
                    g as f32,
                    b as f32,
                    1.0,
                    width / aspect,
                    height,
                    offset_x,
                    offset_y,
                ]
                .as_bytes(),
            );
            // temp_buf drops, writes to uniform
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
            for bind_group in bind_groups {
                pass.set_bind_group(0, &bind_group, &[]);
                pass.draw(0..3, 0..1);
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
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }
            }
            _ => (),
        }
    }
}

pub fn run() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
