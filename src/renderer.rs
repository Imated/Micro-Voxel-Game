use crate::display::Display;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::{Adapter, BackendOptions, Backends, CommandEncoderDescriptor, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Trace};
use winit::window::Window;

pub struct Renderer {
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,

    display: Display,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::debugging(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
                apply_limit_buckets: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::EXPERIMENTAL_RAY_TRACING_PIPELINES,
                experimental_features: unsafe { ExperimentalFeatures::enabled() },
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await?;

        let display = Display::new(&instance, &adapter, &device, window.clone())?;

        Ok(Self {
            instance,
            device,
            adapter,
            queue,
            display,
        })
    }

    pub fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.display.resize(&self.device, &self.queue, width, height);
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let Some(frame) = self.display.acquire_frame(&self.device) else {
            return Ok(()); // skip this frame
        };

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        self.display.fullscreen_pass(&mut encoder, &frame);

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame.surface_texture);

        Ok(())
    }
}
