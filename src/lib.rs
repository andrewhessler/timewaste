use std::sync::Arc;

use anyhow::Result;
use wgpu::{
    ColorWrites, CurrentSurfaceTexture, Device, Instance, InstanceDescriptor, MultisampleState,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, Surface,
    SurfaceConfiguration, TextureUsages,
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    pipeline: RenderPipeline,
    window: Arc<Window>,
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

        Ok(Self {
            device,
            queue,
            window,
            pipeline,
            surface,
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
            pass.draw(0..3, 0..1);
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
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                self.state.as_ref().unwrap().window.request_redraw();
                match self.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Unable to render {e}");
                    }
                }
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
