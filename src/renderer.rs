use crate::display::Display;
use crate::render_context::RenderContext;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, CommandEncoder, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, PipelineCompilationOptions,
    PipelineLayoutDescriptor, ShaderStages, StorageTextureAccess, Texture, TextureFormat,
    TextureView, TextureViewDimension, include_wgsl,
};

pub struct RenderTexture {
    pub output: Texture,
    pub output_view: TextureView,
    pub width: u32,
    pub height: u32,
}

pub struct Renderer {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Renderer {
    pub fn new(context: &RenderContext, display: &Display) -> Self {
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Raytracer Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let bind_group =
            Self::create_bind_group(context, &bind_group_layout, &display.output().output_view);

        let shader = context
            .device
            .create_shader_module(include_wgsl!(".././res/raytracer.wgsl"));
        let layout = context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Raytracing Pipeline Layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });

        let pipeline = context
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Raytracing Pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, context: &RenderContext, output_view: &TextureView) {
        self.bind_group = Self::create_bind_group(context, &self.bind_group_layout, output_view);
    }

    pub fn raytrace_pass(&self, encoder: &mut CommandEncoder, display: &Display) {
        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Raytracer Compute Pass"),
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(
            display.output().width.div_ceil(8),
            display.output().height.div_ceil(8),
            1,
        )
    }

    fn create_bind_group(
        context: &RenderContext,
        layout: &BindGroupLayout,
        output_view: &TextureView,
    ) -> BindGroup {
        context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Raytracer Bind Group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(output_view),
            }],
        })
    }
}
