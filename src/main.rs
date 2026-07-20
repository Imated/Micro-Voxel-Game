use crate::AppRunner::Running;
use crate::app::App;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

mod app;
pub mod buffer;
pub mod camera;
pub mod display;
pub mod render_context;

#[derive(Default)]
pub enum AppRunner {
    #[default]
    Uninitialized,
    Running {
        app: App,
        frame_count: i64,
        window: Arc<Window>,
        delta_time: f64,
    },
}

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("ea")
                        .with_inner_size(PhysicalSize::new(800, 600)),
                )
                .unwrap(),
        );

        *self = Running {
            app: App::new(window.clone()),
            frame_count: 0,
            window,
            delta_time: 0.0,
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Running {
            app,
            frame_count,
            window,
            delta_time,
        } = self
        else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                let prev = Instant::now();

                app.render(*delta_time);


                *delta_time = prev.elapsed().as_secs_f64();
                if *frame_count % 512 == 0 {
                    window.set_title(&format!(
                        "Micro Voxels - {:.1?} FPS",
                        1.0 / *delta_time
                    ));
                }

                *frame_count += 1;
                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                let (Some(width), Some(height)) =
                    (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
                else {
                    return;
                };

                app.on_resize(width, height);
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
                (code, is_pressed) => {
                    app.on_key_event(code, is_pressed);
                }
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();
    info!("Starting app...");
    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut AppRunner::default())
        .expect("Failed to run app");
}
