use egui::{Context, viewport};
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use winit::window::Window;

pub struct EguiRenderer {
    pub renderer: Renderer,
    pub state: State,
}

impl EguiRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        options: RendererOptions,
        window: &Window,
    ) -> Self {
        let renderer = Renderer::new(device, output_color_format, options);
        let context = Context::default();
        let state = State::new(
            context,
            viewport::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024),
        );

        Self { renderer, state }
    }
}
