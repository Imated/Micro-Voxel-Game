use crate::buffer::TypedBuffer;
use crate::render_context::RenderContext;
use crate::world::chunk::Chunk;
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages};

pub struct WorldRenderer {
    // 32x1x32 chunk grid, eventually somehow get this from World struct,
    // oh and also to future me, make ts separate from the World struct Chunk type bc this chunk sohuld be like GpuChunk and store brick map and stuff idk u got this
    chunks: TypedBuffer<[[[Chunk; 32]; 1]; 32]>,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
}

impl WorldRenderer {
    pub fn new(context: &RenderContext) -> Self {
        let buffer = TypedBuffer::new_storage(context, [[[Chunk { empty: 0 }; 32]; 1]; 32]);
        let layout = context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("World Renderer Bind Group Layout"),
                entries: &[buffer.as_layout_entry(0, ShaderStages::COMPUTE)],
            });
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: &layout,
            entries: &[buffer.as_bind_group_entry(0)],
        });

        Self {
            chunks: buffer,
            chunks_layout: layout,
            chunks_bind_group: bind_group,
        }
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.chunks_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.chunks_bind_group
    }
}