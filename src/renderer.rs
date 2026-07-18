use std::num::NonZeroU32;
use std::sync::Arc;
use anyhow::bail;
use wgpu::{include_wgsl, Adapter, BackendOptions, Backends, BlendState, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Face, Features, FragmentState, FrontFace, Instance, InstanceDescriptor, InstanceFlags, Limits, LoadOp, LoadOpDontCare, MemoryBudgetThresholds, MemoryHints, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, StoreOp, Surface, SurfaceColorSpace, SurfaceConfiguration, TextureUsages, Trace, VertexState};
use wgpu::wgt::TextureViewDescriptor;
use winit::window::Window;

pub struct Renderer {
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,

    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    is_surface_configured: bool,

    fullscreen_pipeline: RenderPipeline
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::debugging(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
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

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        let fullscreen_shader = device.create_shader_module(include_wgsl!(".././res/fullscreen.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor::default());
        let fullscreen_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Fullscreen Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &fullscreen_shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &fullscreen_shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Front),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            instance,
            device,
            adapter,
            queue,
            surface,
            surface_config: config,
            is_surface_configured: false,
            fullscreen_pipeline,
        })
    }

    pub fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) {
        self.surface_config.width = width.into();
        self.surface_config.height = height.into();
        self.surface.configure(&self.device, &self.surface_config);
        self.is_surface_configured = true;
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Suboptimal(texture)
            | CurrentSurfaceTexture::Success(texture) => texture,
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => return Ok(()),

            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.surface_config);
                return Ok(());
            }
            CurrentSurfaceTexture::Lost => bail!("Device lost"),
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::DontCare(unsafe { LoadOpDontCare::enabled() }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.fullscreen_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(output);

        Ok(())
    }
}
