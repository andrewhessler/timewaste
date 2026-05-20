use std::sync::Arc;

use anyhow::Result;
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use wgpu::{
    CurrentSurfaceTexture, Device, Features, Instance, InstanceDescriptor, PollType, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
    wgt::{CommandEncoderDescriptor, DeviceDescriptor, TextureViewDescriptor},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

use crate::{ui::EguiRenderer, world::WorldRenderer};

extern crate nalgebra as na;

mod cube;
mod shape_util;
mod ui;
mod world;

#[derive(PartialEq, Debug)]
enum Direction {
    Inc,
    Dec,
    None,
}

struct State {
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    window: Arc<Window>,
    world_renderer: WorldRenderer,
    egui_renderer: EguiRenderer,
    last_frame_time: Option<std::time::Instant>,
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

        let required_features = Features::VERTEX_WRITABLE_STORAGE
            | Features::MAPPABLE_PRIMARY_BUFFERS
            | Features::PRIMITIVE_INDEX;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features,
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

        let world_renderer = WorldRenderer::new(&device, &config);

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
            surface,
            config,
            world_renderer,
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

        let delta_time = if let Some(lft) = state.last_frame_time {
            let delta_time = now - lft;
            let delta_time_f32 = delta_time.as_secs_f32();
            Some(delta_time_f32)
        } else {
            None
        };

        state.last_frame_time = Some(now);

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

        state
            .world_renderer
            .render(&mut encoder, &view, &state.queue, delta_time, &state.config);

        //------- ui render
        // state.egui_renderer.begin_frame(&state.window);
        //
        // egui::Window::new("test").resizable(true).show(
        //     state.egui_renderer.state.egui_ctx(),
        //     |ui| {
        //         ui.label("Hello");
        //     },
        // );
        //
        // state.egui_renderer.end_frame(
        //     &state.window,
        //     &state.device,
        //     &state.queue,
        //     &mut encoder,
        //     &view,
        //     &screen_descriptor,
        // );
        //-------

        state.world_renderer.read_debug_buffer(&encoder);

        let command_buf = encoder.finish();
        state.queue.submit([command_buf]);

        texture.present();
        let _ = state.device.poll(PollType::wait_indefinitely());
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
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    event_loop.exit();
                }

                state.world_renderer.handle_input(&event);
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
