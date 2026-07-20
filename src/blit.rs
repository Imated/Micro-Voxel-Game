use crate::display::{Display, Frame};
use crate::render_context::RenderContext;
use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState, ColorTargetState,
    ColorWrites, CommandEncoder, FilterMode, FragmentState, FrontFace, LoadOp, LoadOpDontCare,
    MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, StoreOp, TextureSampleType, TextureView, TextureViewDimension,
    VertexState,
};

pub struct Blitter {
    pipeline: RenderPipeline,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Blitter {
    pub fn new(context: &RenderContext, display: &Display) -> Self {
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Blit Bind Group Layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let sampler = context.device.create_sampler(&SamplerDescriptor {
            label: Some("Blit Sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = Self::create_bind_group(context, &bind_group_layout, &sampler, &display.output().output_view);

        let shader = context
            .device
            .create_shader_module(include_wgsl!(".././res/fullscreen.wgsl"));
        let layout = context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Blit Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = context
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Blit Pipeline"),
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
                        format: display.surface_format(),
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

        Self {
            pipeline,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, context: &RenderContext, source_view: &TextureView) {
        self.bind_group =
            Self::create_bind_group(context, &self.bind_group_layout, &self.sampler, source_view);
    }

    pub fn blit(&self, encoder: &mut CommandEncoder, frame: &Frame) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Blit Pass"),
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
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }

    fn create_bind_group(
        context: &RenderContext,
        layout: &BindGroupLayout,
        sampler: &Sampler,
        source_view: &TextureView,
    ) -> BindGroup {
        context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(source_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}
