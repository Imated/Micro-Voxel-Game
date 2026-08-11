use crate::array_buffer::TypedArrayBuffer;
use crate::render_context::RenderContext;
use crate::world::brick_pool::BrickPool;
use crate::world::chunk::Chunk;
use crate::world::world::World;
use crate::{buffer::TypedBuffer, world::chunk::ChunkPos};

use glam::ivec3;
use tracing::info;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    brick_pool: BrickPool,
    chunk_grid: [[[Chunk; World::WORLD_SIZE.x as usize]; World::WORLD_SIZE.y as usize];
        World::WORLD_SIZE.z as usize],
    // 32x1x32 chunk grid, eventually somehow get this from World struct,
    // oh and also to future me, make ts separate from the World struct Chunk type bc this chunk sohuld be like GpuChunk and store brick map and stuff idk u got this
    chunks: TypedArrayBuffer<
        [[[Chunk; World::WORLD_SIZE.x as usize]; World::WORLD_SIZE.y as usize];
            World::WORLD_SIZE.z as usize],
    >,
    chunks_layout: BindGroupLayout,
    chunks_bind_group: BindGroup,
    is_dirty: bool,
}

impl WorldRenderer {
    pub fn new(context: &RenderContext) -> Self {
        let chunk_grid = [[[Chunk::new_from_empty(); 8]; 1]; 8];
        let buffer = TypedArrayBuffer::new_storage(context, &[chunk_grid]);

        let mut brick_pool = BrickPool::new(context);
        brick_pool.add_test_brick();

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
            info!("updating...");
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
        let offsetted_coords = coords.0 + World::WORLD_SIZE_HALF;
        let index = (offsetted_coords.rem_euclid(World::WORLD_SIZE)).as_usizevec3();
        self.chunk_grid[index.x][index.y][index.z] = self.brick_pool.gen_test_chunk();
        self.is_dirty = true;
    }

    pub fn unload_chunk(&mut self, coords: ChunkPos) {
        let offsetted_coords = coords.0 + World::WORLD_SIZE_HALF;
        let index = (offsetted_coords.rem_euclid(World::WORLD_SIZE)).as_usizevec3();
        self.chunk_grid[index.x][index.y][index.z] = Chunk::new_from_empty();
        self.is_dirty = true;
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.chunks_layout
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.chunks_bind_group
    }
}
