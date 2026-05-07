use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};

pub struct EguiRenderer {
    pub renderer: Renderer,
    pub context: Context,
}

impl EguiRenderer {
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        options: RendererOptions,
    ) -> Self {
        let renderer = Renderer::new(device, output_color_format, options);
        let context = Context::default();
        Self { renderer, context }
    }
}
