use egui::{Context, viewport};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{
    CommandEncoder, Device, Queue, RenderPassColorAttachment, RenderPassDescriptor, TextureView,
};
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

    pub fn begin_frame(&mut self, window: &Window) {
        let input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(input);
    }

    pub fn end_frame(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        screen_descriptor: &ScreenDescriptor,
    ) {
        self.state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();
        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        self.renderer
            .update_buffers(device, queue, encoder, &tris, screen_descriptor);

        let pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                ops: Default::default(),
                resolve_target: None,
            })],
            ..Default::default()
        });

        self.renderer
            .render(&mut pass.forget_lifetime(), &tris, screen_descriptor);

        for id in full_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }
    }
}
