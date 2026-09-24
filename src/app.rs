use crate::blit::Blitter;
use crate::camera::Camera;
use crate::display::{Display, Frame};
use crate::gui_renderer::GuiRenderer;
use crate::render_context::RenderContext;
use crate::renderer::{RenderTexture, Renderer};
use crate::util::constants::{BRICK_SIZE, CHUNK_SIZE, VOXELS_PER_METER, WORLD_SIZE};
use crate::world::{chunk::ChunkPos, world_renderer::WorldRenderer};
use ash::vk::PhysicalDeviceMemoryBudgetPropertiesEXT;
use egui::{Align2, Color32, FontId, RichText, Sense, vec2};
use glam::{Vec2, Vec3, ivec3};
use humanize_bytes::humanize_bytes_binary;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;
use wgpu::wgc::api::Vulkan;
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

    vk_physical_device: ash::vk::PhysicalDevice,
    vram_stats: (u64, u64),
    vram_last_query: Instant,
}

const VRAM_QUERY_INTERVAL: Duration = Duration::from_millis(250);

impl App {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let context = pollster::block_on(RenderContext::new())?;
        let display = Display::new(context.clone(), &window)?;
        let output = RenderTexture::new(
            &context,
            window.inner_size().width,
            window.inner_size().height,
        );
        let world_bottom = -((WORLD_SIZE.y * CHUNK_SIZE.y * BRICK_SIZE.y) as f32) / 2.0;
        let eye_height = 1.8 * VOXELS_PER_METER;
        let start_y = world_bottom + BRICK_SIZE.y as f32 + eye_height;
        let camera = Camera::new(&context, Vec3::new(0.0, start_y, 0.0), 0.0_f32, 0.0_f32);
        let mut world_renderer = WorldRenderer::new(context.clone());
        let renderer = Renderer::new(context.clone(), &output, &camera, &world_renderer)?;
        let blitter = Blitter::new(context.clone(), &output)?;
        let gui_renderer =
            GuiRenderer::new(context.clone(), window.clone(), display.surface_format());
        let profiler = GpuProfiler::new(
            &context.device,
            GpuProfilerSettings {
                enable_timer_queries: true,
                enable_debug_groups: false,
                max_num_pending_frames: 2,
            },
        )?;

        for x in -4..4 {
            for y in 0..1 {
                for z in -4..4 {
                    world_renderer.load_chunk(&ChunkPos(ivec3(x, y, z)));
                }
            }
        }

        info!("bricks: {}", world_renderer.brick_pool.len());

        let hal_instance = unsafe { context.instance.as_hal::<Vulkan>() }.expect("use vulkan noob");
        let vk_physical_device = unsafe {
            hal_instance
                .shared_instance()
                .raw_instance()
                .enumerate_physical_devices()
                .expect("fym u have no gpu")
                .first()
                .expect("fym u have no gpu")
                .clone()
        };

        Ok(Self {
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
            vk_physical_device,
            vram_stats: (1, 1),
            vram_last_query: Instant::now(),
        })
    }

    pub fn render(&mut self, delta_time: Duration) -> anyhow::Result<()> {
        self.camera.update(delta_time);
        self.world_renderer.update();
        self.update_vram_stats();

        // acquire frame and skip if smth happened and hope it works next frame
        let Some(mut frame) = self.display.acquire_frame() else {
            return Ok(());
        };

        self.renderer.raytrace_pass(
            &mut frame,
            &self.output,
            &self.camera,
            &self.world_renderer,
            &self.profiler,
        )?;
        self.blitter.blit(&mut frame, &self.profiler)?;
        self.render_gui(delta_time, &mut frame);

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

        self.profiler.end_frame()?;

        if let Some(profiling_data) = self
            .profiler
            .process_finished_frame(self.context.queue.get_timestamp_period())
        {
            for (i, profiling_pass) in profiling_data.iter().enumerate() {
                let time = profiling_pass.time.as_ref().unwrap_or(&(0.0..0.0));
                self.profiled_passes[i].1 = ((time.end - time.start) * 1000.0) as f32;
            }
        }

        Ok(())
    }

    pub fn render_gui(&mut self, delta_time: Duration, frame: &mut Frame) {
        self.gui_renderer.run(frame, &mut self.profiler, |ui| {
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
                    ui.separator();
                    ui.label(RichText::new("VRAM usage:").heading().strong().monospace());
                    ui.indent(67, |ui| {
                        let (usage, budget) = self.vram_stats;
                        let max_width = 340.0;
                        let (filled_rect, _) = ui.allocate_exact_size(
                            vec2((max_width * (usage / budget) as f32).max(10.0), 30.0),
                            Sense::empty(),
                        );
                        let background_rect = filled_rect.with_max_x(max_width);
                        ui.painter().rect_filled(
                            background_rect,
                            3.0,
                            Color32::from_black_alpha(125),
                        );
                        ui.painter()
                            .rect_filled(filled_rect, 3.0, Color32::from_rgb(220, 90, 90));
                        let vram_text = &format!(
                            "{} / {}",
                            humanize_bytes_binary!(usage),
                            humanize_bytes_binary!(budget),
                        );
                        ui.painter().text(
                            background_rect.center(),
                            Align2::CENTER_CENTER,
                            vram_text,
                            FontId::monospace(13.0),
                            Color32::WHITE,
                        );
                    });
                });
        });
    }

    fn update_vram_stats(&mut self) {
        if self.vram_last_query.elapsed() < VRAM_QUERY_INTERVAL {
            return;
        }
        self.vram_last_query = Instant::now();

        let Some(hal_instance) = (unsafe { self.context.instance.as_hal::<Vulkan>() }) else {
            return;
        };
        let instance = hal_instance.shared_instance().raw_instance();

        let mut memory_budget_properties = PhysicalDeviceMemoryBudgetPropertiesEXT::default();
        let mut memory_properties = ash::vk::PhysicalDeviceMemoryProperties2::default()
            .push_next(&mut memory_budget_properties);

        unsafe {
            instance.get_physical_device_memory_properties2(
                self.vk_physical_device,
                &mut memory_properties,
            );
        };

        self.vram_stats = memory_properties
            .memory_properties
            .memory_heaps
            .iter()
            .enumerate()
            .find(|(_, heap)| heap.flags.contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL))
            .map(|(i, _)| {
                (
                    memory_budget_properties.heap_usage[i],
                    memory_budget_properties.heap_budget[i],
                )
            })
            .expect("no vram? huh");
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
