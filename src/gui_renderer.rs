use std::sync::Arc;

use crate::display::Frame;
use crate::render_context::RenderContext;
use egui::{Context, Ui};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::EventResponse;
use wgpu::{
    LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, TextureFormat,
};
use winit::{event::WindowEvent, window::Window};

pub struct GuiRenderer {
    state: egui_winit::State,
    renderer: Renderer,
    window: Arc<Window>,
}

impl GuiRenderer {
    pub fn new(context: &RenderContext, window: Arc<Window>, format: TextureFormat) -> Self {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024),
        );

        let egui_renderer = Renderer::new(&context.device, format, RendererOptions::default());

        Self {
            state: egui_state,
            renderer: egui_renderer,
            window,
        }
    }

    pub fn on_event(&mut self, event: &WindowEvent) -> EventResponse {
        self.state.on_window_event(&self.window, event)
    }

    pub fn run(&mut self, frame: &mut Frame, context: &RenderContext, ui: impl FnMut(&mut Ui)) {
        let raw_input = self.state.take_egui_input(&self.window);
        let full_output = self.state.egui_ctx().run_ui(raw_input, ui);

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [
                self.window.inner_size().width,
                self.window.inner_size().height,
            ],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        self.egui()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        self.state
            .handle_platform_output(&self.window, full_output.platform_output);

        let tris = self
            .egui()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            &mut frame.encoder,
            &tris,
            &screen_descriptor,
        );

        let mut render_pass = frame
            .encoder
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("eGUI Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &frame.surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            })
            .forget_lifetime();

        let mut render_pass = frame.profiler.scope("UI", &mut render_pass);

        self.renderer
            .render(&mut render_pass, &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x);
        }
    }

    pub fn egui(&self) -> &Context {
        self.state.egui_ctx()
    }
}
