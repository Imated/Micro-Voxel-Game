use crate::blit::Blitter;
use crate::camera::Camera;
use crate::display::Display;
use crate::gui_renderer::GuiRenderer;
use crate::render_context::RenderContext;
use crate::renderer::{RenderTexture, Renderer};
use crate::world::world_renderer::WorldRenderer;
use glam::{Vec2, Vec3};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::KeyCode;
use winit::window::{CursorGrabMode, Fullscreen, Window};

pub struct App {
    window: Arc<Window>,
    context: RenderContext,
    display: Display,
    camera: Camera,
    renderer: Renderer,
    blitter: Blitter,
    output: RenderTexture,
    world_renderer: WorldRenderer,
    gui_renderer: GuiRenderer,
}

impl App {
    pub fn new(window: Arc<Window>) -> Self {
        let context = pollster::block_on(RenderContext::new()).expect("Failed to create renderer.");
        let display = Display::new(&context, window.clone()).expect("Failed to create display.");
        let output = RenderTexture::new(
            &context,
            window.inner_size().width,
            window.inner_size().height,
        );
        let camera = Camera::new(&context, Vec3::splat(0.0), 0.0, 0.0);
        let world_renderer = WorldRenderer::new(&context);
        let renderer = Renderer::new(&context, &output, &camera, &world_renderer);
        let blitter = Blitter::new(&context, &output, display.surface_format());
        let gui_renderer = GuiRenderer::new(&context, window.clone(), display.surface_format());

        Self {
            window,
            context,
            display,
            camera,
            renderer,
            blitter,
            output,
            world_renderer,
            gui_renderer,
        }
    }

    pub fn render(&mut self, delta_time: Duration) {
        // acquire frame and skip if smth happened and hope it works next frame
        let Some(mut frame) = self.display.acquire_frame(&self.context) else {
            return;
        };

        self.camera.update(&self.context, delta_time);
        self.renderer
            .raytrace_pass(&mut frame, &self.output, &self.camera, &self.world_renderer);
        self.blitter.blit(&mut frame);

        // UI
        self.gui_renderer.run(&mut frame, &self.context, |ui| {
            egui::Window::new("eee").show(ui.ctx(), |ui| {
                ui.label("eeea");
            });
        });
        self.gui_renderer.egui().request_repaint();

        self.window.pre_present_notify();
        self.context.queue.submit([frame.encoder.finish()]);
        frame.surface_texture.present();
    }

    pub fn on_resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.output = RenderTexture::new(&self.context, width.get(), height.get());
        self.display.resize(&self.context, width, height);
        self.renderer
            .resize(&self.context, &self.output.output_view);
        self.blitter.resize(&self.context, &self.output.output_view);
    }

    pub fn on_key_event(&mut self, key_code: KeyCode, is_pressed: bool) {
        self.camera.process_key_event(key_code, is_pressed);

        match (key_code, is_pressed) {
            (KeyCode::F11, true) => {
                if self.window.fullscreen().is_some() {
                    self.window.set_fullscreen(None);
                } else {
                    self.window
                        .set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
            (KeyCode::Escape, true) => {
                self.window
                    .set_cursor_grab(CursorGrabMode::None)
                    .expect("theres like no way ts will fail and if it does i quit rust");
                self.window.set_cursor_visible(true);
            }
            _ => {}
        }
    }

    pub fn on_mouse_moved(&mut self, delta: Vec2) {
        self.camera.process_mouse_movement(delta);
    }

    pub fn on_window_event(&mut self, event: &WindowEvent) {
        let response = self.gui_renderer.on_event(event);
        if response.consumed {
            return;
        }

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Left,
            ..
        } = event
        {
            self.window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined))
                .expect("theres like no way ts will fail and if it does i quit rust");
            self.window.set_cursor_visible(false);
        }
    }
}
