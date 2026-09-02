use static_assertions::assert_impl_all;
use wgpu::{
    Adapter, BackendOptions, Backends, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints,
    PowerPreference, Queue, RequestAdapterOptions, Trace,
};

/// Stores render info for creation/usage of the gpu
/// Internally atomically reference counted so its safe to clone!!!
#[derive(Debug, Clone)]
pub struct RenderContext {
    pub instance: Instance,
    pub device: Device,
    pub adapter: Adapter,
    pub queue: Queue,
}

assert_impl_all!(RenderContext: Send, Sync);

impl RenderContext {
    pub async fn new() -> anyhow::Result<Self> {
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
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::TIMESTAMP_QUERY
                    | Features::TIMESTAMP_QUERY_INSIDE_PASSES
                    | Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: Limits::default(),
                memory_hints: MemoryHints::Performance,
                trace: Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            device,
            adapter,
            queue,
        })
    }
}
