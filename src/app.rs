use crate::blit::Blitter;
use crate::camera::Camera;
use crate::display::Display;
use crate::render_context::RenderContext;
use crate::renderer::Renderer;
use glam::Vec3;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::CommandEncoderDescriptor;
use winit::keyboard::KeyCode;
use winit::window::{Fullscreen, Window};

pub struct App {
    window: Arc<Window>,
    context: RenderContext,
    display: Display,
    camera: Camera,
    renderer: Renderer,
    blitter: Blitter,
}

impl App {
    pub fn new(window: Arc<Window>) -> Self {
        let context = pollster::block_on(RenderContext::new()).expect("Failed to create renderer.");
        let camera = Camera::new(&context, Vec3::splat(0.0), 0.0, 0.0);
        let display = Display::new(&context, window.clone()).expect("Failed to create display.");
        let renderer = Renderer::new(&context, &display);
        let blitter = Blitter::new(&context, &display);

        Self {
            window,
            context,
            display,
            camera,
            renderer,
            blitter,
        }
    }

    pub fn render(&mut self, delta_time: f64) {
        // acquire frame and skip if smth happened and hope it works next frame
        let Some(frame) = self.display.acquire_frame(&self.context) else {
            return;
        };

        let mut encoder = self
            .context
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        self.renderer.raytrace_pass(&mut encoder, &self.display);
        self.blitter.blit(&mut encoder, &frame);

        self.context.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.context.queue.present(frame.surface_texture);
    }

    pub fn on_resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.display.resize(&self.context, width, height);

        let output_view = &self.display.output().output_view;
        self.renderer.resize(&self.context, output_view);
        self.blitter.resize(&self.context, output_view);
    }

    pub fn on_key_event(&mut self, key_code: KeyCode, is_pressed: bool) {
        match (key_code, is_pressed) {
            (KeyCode::F11, true) => {
                if self.window.fullscreen().is_some() {
                    self.window.set_fullscreen(None);
                } else {
                    self.window
                        .set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
            _ => {
                self.camera.process_key_event(key_code, is_pressed);
            }
        }
    }
}
