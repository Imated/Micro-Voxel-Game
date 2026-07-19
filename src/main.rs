use crate::AppRunner::Running;
use crate::app::App;
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

mod app;
pub mod renderer;
pub mod display;

#[derive(Default)]
pub enum AppRunner {
    #[default]
    Uninitialized,
    Running(App),
}

impl ApplicationHandler for AppRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        *self = Running(App::new(event_loop));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Running(app) = self else {
            return;
        };

        app.on_event(event_loop, event);
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
