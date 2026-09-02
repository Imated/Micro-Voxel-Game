use anyhow::anyhow;
use notify::{
    EventKind, RecommendedWatcher, RecursiveMode,
    event::{DataChange, ModifyKind},
};
use notify_debouncer_full::{DebounceEventResult, Debouncer, NoCache, new_debouncer};
use std::{
    borrow::Cow,
    iter::once,
    ops::Deref,
    path::Path,
    process::abort,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use tracing::{error, info};
use wesl::{ModulePath, Wesl};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, ComputePipeline, ComputePipelineDescriptor,
    ErrorFilter, FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayout,
    PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, TextureFormat, VertexState,
};

use crate::render_context::RenderContext;

pub trait Pipeline {
    // Create default pipeline that MY game uses
    fn create(context: &RenderContext, layout: &PipelineLayout, path: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
}

impl Pipeline for ComputePipeline {
    fn create(
        context: &RenderContext,
        layout: &PipelineLayout,
        path: &str,
    ) -> anyhow::Result<Self> {
        info!("Creating compute pipeline for {path:?}...");

        Ok(context
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: None,
                layout: Some(layout),
                module: &make_wesl_shader(context, path)?,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            }))
    }
}

impl Pipeline for RenderPipeline {
    fn create(
        context: &RenderContext,
        layout: &PipelineLayout,
        path: &str,
    ) -> anyhow::Result<Self> {
        info!("Creating render pipeline for {path:?}...");

        let shader = make_wesl_shader(context, path)?;

        Ok(context
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(layout),
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
                        format: TextureFormat::Bgra8UnormSrgb,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            }))
    }
}

pub struct HotReloadPipeline<P> {
    inner: Arc<Mutex<P>>,
    _debouncer: Debouncer<RecommendedWatcher, NoCache>,
}

impl<P: Pipeline + Send + Sync + 'static> HotReloadPipeline<P> {
    pub fn new<T: Into<String>>(
        context: &RenderContext,
        layout: PipelineLayout,
        path: T,
    ) -> anyhow::Result<Self> {
        let path = path.into();
        let inner_pipeline = Arc::new(Mutex::new(P::create(context, &layout, &path)?));

        let pipeline_handle = inner_pipeline.clone();
        let context_handle = context.clone();
        let path_handle = path.clone();
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    for event in events.iter().filter(|e| {
                        e.event.kind == EventKind::Modify(ModifyKind::Data(DataChange::Any))
                    }) {
                        if let Ok(mut guard) = pipeline_handle.lock() {
                            info!("Got notify event: {event:?}");
                            info!("File modified! Reloading...");
                            if let Ok(pipeline) = P::create(&context_handle, &layout, &path_handle)
                            {
                                *guard = pipeline;
                            } else {
                                error!("Failed to compile shader! Using previous shader...");
                            }
                        }
                    }
                } else {
                    error!("Failed to notify pipeline for hot reload!");
                }
            },
        )?;

        debouncer.watch(
            Path::new(&format!("res/{path}.wesl")),
            RecursiveMode::NonRecursive,
        )?;

        info!("Watching {path:?} for hot reloading...");

        Ok(Self {
            inner: inner_pipeline,
            _debouncer: debouncer,
        })
    }

    pub fn acquire(&self) -> MutexGuard<'_, P> {
        self.lock().expect("Welp something panicked :despair:")
    }
}

impl<P: Pipeline + Send + Sync + 'static> Deref for HotReloadPipeline<P> {
    type Target = Arc<Mutex<P>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn make_wesl_shader(context: &RenderContext, path: &str) -> anyhow::Result<ShaderModule> {
    let compiler = Wesl::new("res/");
    let src = compiler
        .compile(&ModulePath::new_root().join(once(path.into())))?
        .to_string();

    let guard = context.device.push_error_scope(ErrorFilter::Validation);
    let shader = context.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(Cow::Owned(src)),
    });

    let err = pollster::block_on(guard.pop());
    if let Some(_err) = err {
        return Err(anyhow!("Failed to compile shader!"));
    }

    info!("Compiled {path:?} shader!");
    Ok(shader)
}
