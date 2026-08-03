use crate::render_context::RenderContext;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::wgt::CommandEncoderDescriptor;
use wgpu::{
    CommandEncoder, CurrentSurfaceTexture, PresentMode, Surface, SurfaceConfiguration,
    SurfaceTexture, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::window::Window;

pub struct Display {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    is_surface_configured: bool,
}

impl Display {
    pub fn new(context: &RenderContext, window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let surface = context.instance.create_surface(window.clone())?;

        let surface_caps = surface.get_capabilities(&context.adapter);
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
        };

        Ok(Self {
            surface,
            surface_config: config,
            is_surface_configured: false,
        })
    }

    pub fn resize(&mut self, context: &RenderContext, width: NonZeroU32, height: NonZeroU32) {
        let width = width.get();
        let height = height.get();

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface
            .configure(&context.device, &self.surface_config);

        self.is_surface_configured = true;
    }

    pub fn acquire_frame(&self, context: &RenderContext) -> Option<Frame> {
        if !self.is_surface_configured {
            return None;
        }

        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Suboptimal(texture)
            | CurrentSurfaceTexture::Success(texture) => texture,

            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation
            | CurrentSurfaceTexture::Lost => return None,

            CurrentSurfaceTexture::Outdated => {
                self.surface
                    .configure(&context.device, &self.surface_config);
                return None;
            }
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        Some(Frame {
            surface_texture: output,
            surface_view: view,
            encoder: context
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default()),
        })
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }
}

pub struct Frame {
    pub surface_texture: SurfaceTexture,
    pub surface_view: TextureView,
    pub encoder: CommandEncoder,
}
