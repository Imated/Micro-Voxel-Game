use crate::array_buffer::TypedArrayBuffer;
use crate::render_context::RenderContext;
use crate::world::chunk::Chunk;
use crate::{buffer::TypedBuffer, world::brick::Brick};
use slab::Slab;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    brick_pool: Slab<Brick>,
    brick_pool_len: usize,
    brick_pool_buffer: TypedArrayBuffer<Brick>,
    // 32x1x32 chunk grid, eventually somehow get this from World struct,
    // oh and also to future me, make ts separate from the World struct Chunk type bc this chunk sohuld be like GpuChunk and store brick map and stuff idk u got this
    chunks: TypedBuffer<[[[Chunk; 8]; 1]; 8]>,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
}

impl WorldRenderer {
    pub fn new(context: &RenderContext) -> Self {
        let buffer = TypedBuffer::new_storage(context, [[[Chunk::new_from_full(); 8]; 1]; 8]);

        let mut brick_pool = Slab::new();
        let _ = brick_pool.insert(Brick {
            empty: false as u32,
        });

        let mut vec: Vec<Brick> = Vec::with_capacity(1);
        for i in 0..1 {
            let brick = if let Some(&brick) = brick_pool.get(i) {
                brick
            } else {
                Brick { empty: true as u32 }
            };
            vec.push(brick);
        }

        let brick_pool_buffer = TypedArrayBuffer::new_storage(context, &vec);
        let layout = context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("World Renderer Bind Group Layout"),
                entries: &[
                    buffer.as_layout_entry(0, ShaderStages::COMPUTE),
                    brick_pool_buffer.as_layout_entry(1, ShaderStages::COMPUTE),
                ],
            });
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: &layout,
            entries: &[
                buffer.as_bind_group_entry(0),
                brick_pool_buffer.as_bind_group_entry(1),
            ],
        });

        Self {
            brick_pool,
            chunks: buffer,
            chunks_layout: layout,
            chunks_bind_group: bind_group,
            brick_pool_len: 1,
            brick_pool_buffer,
        }
    }

    pub fn update(&mut self, context: &RenderContext) {
        let mut vec: Vec<Brick> = Vec::with_capacity(self.brick_pool_len);
        for i in 0..self.brick_pool_len {
            let brick = if let Some(&brick) = self.brick_pool.get(i) {
                brick
            } else {
                Brick { empty: true as u32 }
            };
            vec.push(brick);
        }

        self.brick_pool_buffer.update(context, &vec);

        self.chunks_bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: self.layout(),
            entries: &[
                self.chunks.as_bind_group_entry(0),
                self.brick_pool_buffer.as_bind_group_entry(1),
            ],
        });
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.chunks_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.chunks_bind_group
    }
}
