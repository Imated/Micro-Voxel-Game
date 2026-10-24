use crate::array_buffer::TypedArrayBuffer;
use crate::render_context::RenderContext;
use crate::util::constants::WORLD_SIZE;
use crate::util::thread::ThreadPool;
use crate::util::torodial_pos;
use crate::world::ChunkPos;
use crate::world::chunk::Chunk;
use crate::world::render::brick::Brick;
use crate::world::render::brick_pool::BrickPool;
use crate::world::render::gpu_chunk::GpuChunk;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    context: RenderContext,
    thread_pool: ThreadPool<Vec<(usize, Brick)>>,

    pub brick_pool: BrickPool,
    chunk_grid: Box<[GpuChunk]>,
    chunks: TypedArrayBuffer<GpuChunk>,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
}

impl WorldRenderer {
    #[must_use]
    pub fn new(context: RenderContext) -> Self {
        let chunk_grid =
            vec![GpuChunk::default(); (WORLD_SIZE.x * WORLD_SIZE.y * WORLD_SIZE.z) as usize]
                .into_boxed_slice();

        let buffer = TypedArrayBuffer::new_storage(context.clone(), &chunk_grid);
        let brick_pool = BrickPool::new(&context);

        let layout = context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("World Renderer Bind Group Layout"),
                entries: &[
                    buffer.as_layout_entry(0, ShaderStages::COMPUTE),
                    brick_pool.as_layout_entry(1, ShaderStages::COMPUTE),
                ],
            });
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: &layout,
            entries: &[
                buffer.as_bind_group_entry(0),
                brick_pool.as_bind_group_entry(1),
            ],
        });

        Self {
            brick_pool,
            chunks: buffer,
            chunks_layout: layout,
            chunks_bind_group: bind_group,
            chunk_grid,
            context,
            thread_pool: ThreadPool::default(),
        }
    }

    pub fn update(&mut self, chunks: Option<HashMap<ChunkPos, Arc<Chunk>>>) {
        if let Some(chunks) = chunks {
            for (pos, chunk) in chunks {
                self.thread_pool
                    .queue(pos, move |_| GpuChunk::build_bricks(&chunk));
            }
        }

        self.thread_pool.create_data_threads();
        let finished_chunks = self.thread_pool.poll_data_threads();
        for (&pos, bricks) in &finished_chunks {
            let slot = &mut self.chunk_grid[torodial_pos(pos)];
            for &brick in &slot.bricks {
                if brick != Brick::EMPTY as u32 {
                    self.brick_pool.free(brick as usize);
                }
            }

            *slot = GpuChunk::from_bricks(bricks, &mut self.brick_pool);
        }
        if !finished_chunks.is_empty() {
            self.chunks.update(&self.chunk_grid);
        }

        if self.brick_pool.update() {
            self.chunks_bind_group = self.context.device.create_bind_group(&BindGroupDescriptor {
                label: Some("World Renderer Bind Group"),
                layout: self.layout(),
                entries: &[
                    self.chunks.as_bind_group_entry(0),
                    self.brick_pool.as_bind_group_entry(1),
                ],
            });
        }
    }

    #[must_use]
    pub const fn layout(&self) -> &BindGroupLayout {
        &self.chunks_layout
    }

    #[must_use]
    pub const fn bind_group(&self) -> &BindGroup {
        &self.chunks_bind_group
    }
}
