use crate::renderer::Renderer;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use tracing::error;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

pub struct App {
    window: Arc<Window>,
    renderer: Renderer,
    frame_count: i64,
}

impl App {
    pub fn new(event_loop: &ActiveEventLoop) -> Self {
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("ea").with_inner_size(PhysicalSize::new(800, 600)))
                .unwrap(),
        );
        Self {
            renderer: pollster::block_on(Renderer::new(window.clone()))
                .expect("Failed to create renderer."),
            window,
            frame_count: 0,
        }
    }

    pub fn on_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                let prev = Instant::now();
                self.window.pre_present_notify();
                match self.renderer.render() {
                    Ok(_) => {}
                    Err(err) => {
                        error!("{err}");
                        event_loop.exit();
                    }
                }
                if self.frame_count % 512 == 0 {
                    self.window.set_title(&format!("Micro Voxels - {:.1?} FPS", 1.0 / prev.elapsed().as_secs_f32()));
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

                self.renderer.resize(width, height);
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
                _ => {}
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}
