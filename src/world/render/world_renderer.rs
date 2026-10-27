use crate::array_buffer::TypedArrayBuffer;
use crate::buffer::TypedBuffer;
use crate::render_context::RenderContext;
use crate::util::constants::WORLD_SIZE;
use crate::util::thread::ThreadPool;
use crate::util::torodial_pos;
use crate::world::render::brick::Brick;
use crate::world::render::brick_pool::BrickPool;
use crate::world::render::gpu_chunk::GpuChunk;
use crate::world::{ChunkPos, WorldEvent};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, ShaderStages,
};

pub struct WorldRenderer {
    context: RenderContext,
    thread_pool: ThreadPool<Vec<(usize, Brick)>>,

    pub brick_pool: BrickPool,
    chunk_grid: Box<[GpuChunk]>,
    chunks: TypedArrayBuffer<GpuChunk>,
    window: TypedBuffer<[i32; 4]>,
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
        let window = TypedBuffer::new_uniform(context.clone(), [0; 4]);

        let layout = context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("World Renderer Bind Group Layout"),
                entries: &[
                    buffer.as_layout_entry(0, ShaderStages::COMPUTE),
                    brick_pool.as_layout_entry(1, ShaderStages::COMPUTE),
                    window.as_layout_entry(2, ShaderStages::COMPUTE),
                ],
            });
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("World Renderer Bind Group"),
            layout: &layout,
            entries: &[
                buffer.as_bind_group_entry(0),
                brick_pool.as_bind_group_entry(1),
                window.as_bind_group_entry(2),
            ],
        });

        Self {
            brick_pool,
            chunks: buffer,
            chunks_layout: layout,
            chunks_bind_group: bind_group,
            chunk_grid,
            window,
            context,
            thread_pool: ThreadPool::default(),
            is_dirty: true,
        }
    }

    pub fn set_center(&self, center: ChunkPos) {
        self.window.update(center.0.extend(0).to_array());
    }

    // can get ts by packets if i do multiplayer
    pub fn update_world_events(&mut self, events: Vec<WorldEvent>) {
        for event in events {
            match event {
                WorldEvent::ChunkLoaded { pos, chunk } => {
                    self.thread_pool
                        .queue(pos, move |_| GpuChunk::build_bricks(&chunk));
                }
                WorldEvent::ChunkUnloaded(pos) => {
                    self.thread_pool.deque(pos);

                    let slot = &mut self.chunk_grid[torodial_pos(pos)];
                    for &brick in &slot.bricks {
                        if brick != Brick::EMPTY as u32 {
                            self.brick_pool.free(brick as usize);
                        }
                    }

                    *slot = GpuChunk::default();
                    self.is_dirty = true;
                }
            }
        }
    }

    pub fn update(&mut self) {
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
        self.is_dirty |= !finished_chunks.is_empty();
        if self.is_dirty {
            self.chunks.update(&self.chunk_grid);
            self.is_dirty = false;
        }

        if self.brick_pool.update() {
            self.chunks_bind_group = self.context.device.create_bind_group(&BindGroupDescriptor {
                label: Some("World Renderer Bind Group"),
                layout: self.layout(),
                entries: &[
                    self.chunks.as_bind_group_entry(0),
                    self.brick_pool.as_bind_group_entry(1),
                    self.window.as_bind_group_entry(2),
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
