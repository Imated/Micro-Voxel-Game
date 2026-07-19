use bytemuck::checked::cast_slice;
use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::TextureViewDescriptor;
use wgpu::{include_wgsl, Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, CurrentSurfaceTexture, Device, FragmentState, FrontFace, Instance, LoadOp, LoadOpDontCare, MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages, StoreOp, Surface, SurfaceColorSpace, SurfaceConfiguration, SurfaceTexture, TextureUsages, TextureView, VertexState};
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Pod, Zeroable)]
pub struct DisplayUniform {
    size_aspect: Vec4,
}

pub struct Display {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    is_surface_configured: bool,
    pub pipeline: RenderPipeline,

    display_uniform: DisplayUniform,
    display_buffer: Buffer,
    display_bind_group: BindGroup,
}

impl Display {
    pub fn new(instance: &Instance, adapter: &Adapter, device: &Device, window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let surface = instance.create_surface(window.clone())?;

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

        let mut display_uniform = DisplayUniform::default();
        display_uniform.size_aspect = Vec4::new(size.width as f32, size.height as f32, size.width as f32 / size.height as f32, 0.0);
        let display_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Display Variables Buffer"),
            contents: cast_slice(&[display_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let display_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Display Variables Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let display_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Display Variables Bind Group"),
            layout: &display_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: display_buffer.as_entire_binding(),
                }
            ],
        });

        let shader = device.create_shader_module(include_wgsl!(".././res/fullscreen.wgsl"));
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Fullscreen Pipeline Layout"),
            bind_group_layouts: &[Some(&display_bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Fullscreen Pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shader,
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
                cull_mode: None,
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
            surface,
            surface_config: config,
            is_surface_configured: false,
            pipeline,
            display_uniform,
            display_buffer,
            display_bind_group,
        })
    }

    pub fn resize(&mut self, device: &Device, queue: &Queue, width: NonZeroU32, height: NonZeroU32) {
        let width = width.get() as f32;
        let height = height.get() as f32;

        self.surface_config.width = width as u32;
        self.surface_config.height = height as u32;
        self.surface.configure(device, &self.surface_config);
        self.display_uniform.size_aspect = Vec4::new(width, height, width / height, 0.0);
        queue.write_buffer(&self.display_buffer, 0, cast_slice(&[self.display_uniform]));
        self.is_surface_configured = true;
    }

    pub fn acquire_frame(&self, device: &Device) -> Option<Frame> {
        if !self.is_surface_configured {
            return None;
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Suboptimal(texture)
            | CurrentSurfaceTexture::Success(texture) => texture,

            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation
            | CurrentSurfaceTexture::Lost=> return None,

            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(device, &self.surface_config);
                return None;
            }
        };

        let view = output.texture.create_view(&TextureViewDescriptor::default());

        Some(Frame {
            surface_texture: output,
            surface_view: view,
        })
    }

    pub fn fullscreen_pass(&self, encoder: &mut CommandEncoder, frame: &Frame) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &frame.surface_view,
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.display_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}

pub struct Frame {
    pub surface_texture: SurfaceTexture,
    pub surface_view: TextureView,
}