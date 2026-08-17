use crate::camera::Camera;
use crate::display::Frame;
use crate::render_context::RenderContext;
use crate::world::world_renderer::WorldRenderer;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Extent3d, PipelineCompilationOptions, PipelineLayoutDescriptor,
    ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    include_wgsl,
};

pub struct RenderTexture {
    pub output: Texture,
    pub output_view: TextureView,
    pub width: u32,
    pub height: u32,
}

impl RenderTexture {
    pub const FORMAT: TextureFormat = TextureFormat::Rgba16Float;
}

impl RenderTexture {
    pub fn new(context: &RenderContext, width: u32, height: u32) -> Self {
        let output = context.device.create_texture(&TextureDescriptor {
            label: Some("Output Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: RenderTexture::FORMAT,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_view = output.create_view(&TextureViewDescriptor::default());

        Self {
            output,
            output_view,
            width,
            height,
        }
    }
}

pub struct Renderer {
    pipeline: ComputePipeline,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Renderer {
    pub fn new(
        context: &RenderContext,
        output: &RenderTexture,
        camera: &Camera,
        world_renderer: &WorldRenderer,
    ) -> Self {
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
                            format: RenderTexture::FORMAT,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let bind_group = Self::create_bind_group(context, &bind_group_layout, &output.output_view);

        let shader = context
            .device
            .create_shader_module(include_wgsl!(".././res/compiled/raytracer.wgsl"));
        let layout = context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Raytracing Pipeline Layout"),
                bind_group_layouts: &[
                    Some(world_renderer.layout()),
                    Some(&camera.get_layout()),
                    Some(&bind_group_layout),
                ],
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

    pub fn raytrace_pass(
        &self,
        frame: &mut Frame,
        output: &RenderTexture,
        camera: &Camera,
        world_renderer: &WorldRenderer,
    ) {
        let mut compute_pass = frame.encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Raytracer Compute Pass"),
            timestamp_writes: None,
        });

        let mut compute_pass = frame.profiler.scope("Raytracing", &mut compute_pass);

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, world_renderer.bind_group(), &[]);
        compute_pass.set_bind_group(1, camera.get_bind_group(), &[]);
        compute_pass.set_bind_group(2, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(output.width.div_ceil(16), output.height.div_ceil(16), 1);
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
