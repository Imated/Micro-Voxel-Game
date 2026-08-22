use crate::array_buffer::TypedArrayBuffer;
use crate::render_context::RenderContext;
use crate::util::constants::WORLD_SIZE;
use crate::util::constants::WORLD_SIZE_HALF;
use crate::util::flatten;
use crate::world::brick_pool::BrickPool;
use crate::world::chunk::Chunk;
use crate::world::chunk::ChunkPos;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    pub brick_pool: BrickPool,
    chunk_grid: [Chunk; (WORLD_SIZE.x * WORLD_SIZE.y * WORLD_SIZE.z) as usize],
    // 32x1x32 chunk grid, eventually somehow get this from World struct,
    // oh and also to future me, make ts separate from the World struct Chunk type bc this chunk sohuld be like GpuChunk and store brick map and stuff idk u got this
    chunks: TypedArrayBuffer<[Chunk; (WORLD_SIZE.x * WORLD_SIZE.y * WORLD_SIZE.z) as usize]>,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
    is_dirty: bool,
}

impl WorldRenderer {
    pub fn new(context: &RenderContext) -> Self {
        let chunk_grid =
            [Chunk::new_from_empty(); (WORLD_SIZE.x * WORLD_SIZE.y * WORLD_SIZE.z) as usize];
        let buffer = TypedArrayBuffer::new_storage(context, &[chunk_grid]);

        let brick_pool = BrickPool::new(context);

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
        }
    }

    pub fn update(&mut self, context: &RenderContext) {
        if self.is_dirty {
            self.chunks.update(context, &[self.chunk_grid]);
            self.is_dirty = false;
        }

        if !self.brick_pool.update(context) {
            return;
        }

        self.chunks_bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: self.layout(),
            entries: &[
                self.chunks.as_bind_group_entry(0),
                self.brick_pool.as_bind_group_entry(1),
            ],
        });
    }

    pub fn load_chunk(&mut self, coords: ChunkPos) {
        let offsetted_coords = coords.0 + WORLD_SIZE_HALF.as_ivec3();
        let index = (offsetted_coords.rem_euclid(WORLD_SIZE.as_ivec3())).as_uvec3();
        self.chunk_grid[flatten(index.x, index.y, index.z, WORLD_SIZE) as usize] =
            self.brick_pool.gen_test_chunk(coords);
        self.is_dirty = true;
    }

    pub fn unload_chunk(&mut self, coords: ChunkPos) {
        let offsetted_coords = coords.0 + WORLD_SIZE_HALF.as_ivec3();
        let index = (offsetted_coords.rem_euclid(WORLD_SIZE.as_ivec3())).as_uvec3();
        self.chunk_grid[flatten(index.x, index.y, index.z, WORLD_SIZE) as usize] =
            Chunk::new_from_empty();
        self.is_dirty = true;
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.chunks_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.chunks_bind_group
    }
}
