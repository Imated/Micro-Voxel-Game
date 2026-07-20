use crate::render_context::RenderContext;
use crate::renderer::RenderTexture;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::{CurrentSurfaceTexture, Extent3d, PresentMode, Surface, SurfaceColorSpace, SurfaceConfiguration, SurfaceTexture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use winit::window::Window;

pub struct Display {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    is_surface_configured: bool,

    output: RenderTexture,
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
            color_space: SurfaceColorSpace::Auto,
        };

        let output = Self::create_output_texture(context, size.width, size.height);

        Ok(Self {
            surface,
            surface_config: config,
            is_surface_configured: false,
            output,
        })
    }

    pub fn resize(&mut self, context: &RenderContext, width: NonZeroU32, height: NonZeroU32) {
        let width = width.get();
        let height = height.get();

        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface
            .configure(&context.device, &self.surface_config);

        self.output = Self::create_output_texture(context, width, height);
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
        })
    }

    pub fn output(&self) -> &RenderTexture {
        &self.output
    }

    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    fn create_output_texture(context: &RenderContext, width: u32, height: u32) -> RenderTexture {
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
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let output_view = output.create_view(&TextureViewDescriptor::default());

        RenderTexture {
            output,
            output_view,
            width,
            height,
        }
    }
}

pub struct Frame {
    pub surface_texture: SurfaceTexture,
    pub surface_view: TextureView,
}
