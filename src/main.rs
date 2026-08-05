use crate::AppRunner::Running;
use crate::app::App;
use glam::Vec2;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey::Code;
use winit::window::{Window, WindowAttributes, WindowId};

mod app;
mod blit;
pub mod buffer;
pub mod camera;
pub mod display;
pub mod gui_renderer;
pub mod render_context;
pub mod renderer;
pub mod world;

#[derive(Default)]
pub enum AppRunner {
    #[default]
    Uninitialized,
    Running {
        app: Box<App>,
        frame_count: i64,
        window: Arc<Window>,
        delta_time: Duration,
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
            app: Box::new(App::new(window.clone())),
            frame_count: 0,
            window,
            delta_time: Duration::ZERO,
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Running {
            app,
            frame_count,
            window,
            delta_time,
            ..
        } = self
        else {
            return;
        };

        app.on_window_event(&event);

        match event {
            WindowEvent::RedrawRequested => {
                let prev = Instant::now();

                app.render(*delta_time);

                *delta_time = prev.elapsed();
                let threshold = (512.0 * (1.0 - delta_time.as_secs_f32())).max(5.0) as i64;
                if *frame_count % threshold == 0 {
                    window.set_title(&format!(
                        "Micro Voxels - {:.1?} FPS",
                        1.0 / delta_time.as_secs_f64()
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
                        physical_key: Code(code),
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => {
                app.on_key_event(code, state.is_pressed());
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let Running { app, .. } = self else {
            return;
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            app.on_mouse_moved(Vec2::new(delta.0 as f32, delta.1 as f32));
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
