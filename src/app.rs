use crate::blit::Blitter;
use crate::camera::Camera;
use crate::display::{Display, Frame};
use crate::gui_renderer::GuiRenderer;
use crate::render_context::RenderContext;
use crate::renderer::{RenderTexture, Renderer};
use crate::world::{chunk::ChunkPos, world_renderer::WorldRenderer};
use egui::{Align2, Color32, FontId, RichText, Sense, vec2};
use glam::{Vec2, Vec3, ivec3};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
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
    profiler: GpuProfiler,

    profiled_passes: [(&'static str, f32); 3],
}

impl App {
    pub fn new(window: Arc<Window>) -> Self {
        let context = pollster::block_on(RenderContext::new()).expect("Failed to create renderer.");
        let display = Display::new(context.clone(), &window).expect("Failed to create display.");
        let output = RenderTexture::new(
            &context,
            window.inner_size().width,
            window.inner_size().height,
        );
        let camera = Camera::new(&context, Vec3::splat(0.0), 0.0_f32, 0.0_f32);
        let mut world_renderer = WorldRenderer::new(context.clone());
        let renderer = Renderer::new(context.clone(), &output, &camera, &world_renderer);
        let blitter = Blitter::new(context.clone(), &output);
        let gui_renderer =
            GuiRenderer::new(context.clone(), window.clone(), display.surface_format());
        let profiler = GpuProfiler::new(
            &context.device,
            GpuProfilerSettings {
                enable_timer_queries: true,
                enable_debug_groups: false,
                max_num_pending_frames: 2,
            },
        )
        .expect("Failed to crate GPU profiler.");

        for x in -4..4 {
            for y in 0..1 {
                for z in -4..4 {
                    world_renderer.load_chunk(&ChunkPos(ivec3(x, y, z)));
                }
            }
        }

        info!("bricks: {}", world_renderer.brick_pool.len());

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
            profiler,
            profiled_passes: [("Raytracing", 0.0), ("Blit", 0.0), ("UI", 0.0)],
        }
    }

    pub fn render(&mut self, delta_time: Duration) {
        // acquire frame and skip if smth happened and hope it works next frame
        let Some(mut frame) = self.display.acquire_frame(&self.profiler) else {
            return;
        };

        self.camera.update(delta_time);
        self.world_renderer.update();
        self.renderer
            .raytrace_pass(&mut frame, &self.output, &self.camera, &self.world_renderer);
        self.blitter.blit(&mut frame);

        // UI
        self.gui_renderer.run(&mut frame, |ui| {
            ui.style_mut().animation_time = 0.0;

            egui::Window::new("Profiler")
                .anchor(Align2::LEFT_TOP, vec2(8.0, 8.0))
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .show(ui, |ui| {
                    let total: f32 = self.profiled_passes.iter().map(|(_, t)| *t).sum();
                    let bar_width = 240.0;

                    ui.label(
                        RichText::new(format!("FPS: {}", 1.0 / delta_time.as_secs_f32()))
                            .heading()
                            .strong()
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(format!("GPU FPS: {}", 1.0 / (total / 1000.0)))
                            .heading()
                            .strong()
                            .monospace(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Frame time: {:?}ms",
                            delta_time.as_secs_f32() * 1000.0
                        ))
                        .heading()
                        .strong()
                        .monospace(),
                    );
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for (name, time) in self.profiled_passes {
                            let fraction = if total > 0.0 { time / total } else { 0.0 };
                            let box_width = (bar_width * fraction).max(40.0);
                            let (rect, _) =
                                ui.allocate_exact_size(vec2(box_width, 40.0), Sense::empty());
                            ui.painter()
                                .rect_filled(rect, 3.0, Color32::from_rgb(220, 90, 90));
                            ui.painter().text(
                                rect.center() - vec2(0.0, 7.0),
                                Align2::CENTER_CENTER,
                                name,
                                FontId::proportional(11.0),
                                Color32::BLACK,
                            );
                            ui.painter().text(
                                rect.center() + vec2(0.0, 7.0),
                                Align2::CENTER_CENTER,
                                format!("{time:.2} ms"),
                                FontId::proportional(10.0),
                                Color32::BLACK,
                            );
                        }
                    });
                });
        });

        let Frame {
            surface_texture,
            mut encoder,
            ..
        } = frame;

        self.profiler.resolve_queries(&mut encoder);

        self.gui_renderer.egui().request_repaint();
        self.window.pre_present_notify();
        self.context.queue.submit([encoder.finish()]);
        surface_texture.present();

        self.profiler.end_frame().unwrap();

        if let Some(profiling_data) = self
            .profiler
            .process_finished_frame(self.context.queue.get_timestamp_period())
        {
            for (i, profiling_pass) in profiling_data.iter().enumerate() {
                let time = profiling_pass.time.as_ref().unwrap();
                self.profiled_passes[i].1 = ((time.end - time.start) * 1000.0) as f32;
            }
        }
    }

    pub fn on_resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.output = RenderTexture::new(&self.context, width.get(), height.get());
        self.display.resize(width, height);
        self.renderer.resize(&self.output.output_view);
        self.blitter.resize(&self.output.output_view);
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
