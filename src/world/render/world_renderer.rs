use crate::array_buffer::TypedArrayBuffer;
use crate::render_context::RenderContext;
use crate::util::constants::WORLD_SIZE;
use crate::util::constants::WORLD_SIZE_HALF;
use crate::util::flatten;
use crate::world::ChunkPos;
use crate::world::chunk::Chunk;
use crate::world::render::brick_pool::BrickPool;
use crate::world::render::gpu_chunk::GpuChunk;
use std::sync::Arc;
use std::sync::RwLock;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    context: RenderContext,

    pub brick_pool: BrickPool,
    chunk_grid: Box<[GpuChunk]>,
    // 32x1x32 chunk grid, eventually somehow get this from World struct,
    // oh and also to future me, make ts separate from the World struct Chunk type bc this chunk sohuld be like GpuChunk and store brick map and stuff idk u got this
    chunks: TypedArrayBuffer<GpuChunk>,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
    is_dirty: bool,
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
            is_dirty: false,
            context,
        }
    }

    pub fn update(&mut self) {
        if self.is_dirty {
            self.chunks.update(&self.chunk_grid);
            self.is_dirty = false;
        }

        if !self.brick_pool.update() {
            return;
        }

        self.chunks_bind_group = self.context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: self.layout(),
            entries: &[
                self.chunks.as_bind_group_entry(0),
                self.brick_pool.as_bind_group_entry(1),
            ],
        });
    }

    pub fn load_chunk(&mut self, coords: &ChunkPos) {
        let offsetted_coords = coords.0 + WORLD_SIZE_HALF.as_ivec3();
        let index = (offsetted_coords.rem_euclid(WORLD_SIZE.as_ivec3())).as_uvec3();
        let chunk = Arc::new(RwLock::new(Chunk::new(coords)));
        self.chunk_grid[flatten(index.x, index.y, index.z, WORLD_SIZE) as usize] =
            GpuChunk::new(&chunk, &mut self.brick_pool);
        self.is_dirty = true;
    }

    pub fn unload_chunk(&mut self, coords: &ChunkPos) {
        let offsetted_coords = coords.0 + WORLD_SIZE_HALF.as_ivec3();
        let index = (offsetted_coords.rem_euclid(WORLD_SIZE.as_ivec3())).as_uvec3();
        self.chunk_grid[flatten(index.x, index.y, index.z, WORLD_SIZE) as usize] =
            GpuChunk::default();
        self.is_dirty = true;
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
