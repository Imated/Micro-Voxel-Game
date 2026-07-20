use crate::display::Display;
use crate::render_context::RenderContext;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use tracing::error;
use wgpu::CommandEncoderDescriptor;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowAttributes};

pub struct App {
    window: Arc<Window>,
    context: RenderContext,
    display: Display,
    frame_count: i64,
}

impl App {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("ea")
                        .with_inner_size(PhysicalSize::new(800, 600)),
                )
                .unwrap(),
        );

        let context = pollster::block_on(RenderContext::new()).expect("Failed to create renderer.");
        let display = Display::new(&context, window.clone()).expect("Failed to create display.");

        Self {
            context,
            window,
            display,
            frame_count: 0,
        }
    }

    pub fn on_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let prev = Instant::now();

                // acquire frame and skip if smth happened and hope it works next frame
                let Some(frame) = self.display.acquire_frame(&self.context) else {
                    return;
                };

                let mut encoder = self
                    .context
                    .device
                    .create_command_encoder(&CommandEncoderDescriptor::default());

                self.display.fullscreen_pass(&mut encoder, &frame);

                self.context.queue.submit([encoder.finish()]);
                self.window.pre_present_notify();
                self.context.queue.present(frame.surface_texture);

                // show fps
                if self.frame_count % 512 == 0 {
                    self.window.set_title(&format!(
                        "Micro Voxels - {:.1?} FPS",
                        1.0 / prev.elapsed().as_secs_f32()
                    ));
                }

                self.frame_count += 1;
                self.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return;
                };

                self.display.resize(&self.context, width, height);
            }

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (KeyCode::F11, true) => {
                    if self.window.fullscreen().is_some() {
                        self.window.set_fullscreen(None);
                    } else {
                        self.window
                            .set_fullscreen(Some(Fullscreen::Borderless(None)));
                    }
                }
                _ => {}
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
