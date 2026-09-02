use crate::render_context::RenderContext;
use crate::renderer::RenderTexture;
use crate::{display::Frame, util::pipeline::HotReloadPipeline};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, FilterMode, LoadOp, LoadOpDontCare,
    Operations, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp,
    TextureSampleType, TextureView, TextureViewDimension,
};

pub struct Blitter {
    context: RenderContext,
    pipeline: HotReloadPipeline<RenderPipeline>,
    sampler: Sampler,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Blitter {
    pub fn new(context: RenderContext, source: &RenderTexture) -> Self {
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

        let bind_group =
            Self::create_bind_group(&context, &bind_group_layout, &sampler, &source.output_view);

        let layout = context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Blit Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = HotReloadPipeline::new(&context, layout, "blit").unwrap();

        Self {
            context,
            pipeline,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, source_view: &TextureView) {
        self.bind_group = Self::create_bind_group(
            &self.context,
            &self.bind_group_layout,
            &self.sampler,
            source_view,
        );
    }

    pub fn blit(&self, frame: &mut Frame) {
        let mut render_pass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
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

        let mut render_pass = frame.profiler.scope("Blit", &mut render_pass);

        render_pass.set_pipeline(&self.pipeline.acquire());
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
