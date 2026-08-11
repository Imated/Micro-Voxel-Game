use std::ops::{Deref, DerefMut};

use crate::{
    array_buffer::TypedArrayBuffer, render_context::RenderContext, util::free_list::FreeList,
    world::brick::Brick, world::chunk::Chunk,
};

pub struct BrickPool {
    pool: FreeList<Brick>,
    buffer: TypedArrayBuffer<Brick>,
    is_dirty: bool,
}

impl DerefMut for BrickPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl Deref for BrickPool {
    type Target = TypedArrayBuffer<Brick>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl BrickPool {
    pub fn new(context: &RenderContext) -> Self {
        let mut pool = FreeList::default();
        pool.push(Brick::default()); // index 0 always empty
        let buffer = TypedArrayBuffer::new_storage(context, &pool);

        Self {
            pool,
            buffer,
            is_dirty: false,
        }
    }

    pub fn gen_test_chunk(&mut self) -> Chunk {
        let mut chunk = Chunk::new_from_empty();

        const CENTER: usize = 8 * 8 / 2; // chunk size * brick size / 2
        const RADIUS: usize = 32;

        for bx in 0..8 {
            for by in 0..8 {
                for bz in 0..8 {
                    let mut voxels = [[[0; 8]; 8]; 8];
                    let mut empty = true;
                    for (vx, voxels2) in voxels.iter_mut().enumerate() {
                        for (vy, voxels3) in voxels2.iter_mut().enumerate() {
                            for (vz, voxel) in voxels3.iter_mut().enumerate() {
                                let dx = bx * 8 + vx - CENTER;
                                let dy = by * 8 + vy - CENTER;
                                let dz = bz * 8 + vz - CENTER;
                                if dx * dx + dy * dy + dz * dz <= RADIUS * RADIUS {
                                    *voxel = 1;
                                    empty = false;
                                }
                            }
                        }
                    }

                    let brick = Brick {
                        voxels,
                        empty: empty as u32,
                    };

                    chunk.bricks[bx][by][bz] = self.pool.push(brick) as u32;
                }
            }
        }

        self.is_dirty = true;

        chunk
    }

    pub fn add_test_brick(&mut self) {
        self.pool.push(Brick {
            empty: false as u32,
            voxels: [[[1; 8]; 8]; 8],
        });

        self.is_dirty = true;
    }

    /// Returns if the update did smth, false if it wasnt dirty.
    pub fn update(&mut self, context: &RenderContext) -> bool {
        if !self.is_dirty {
            return false;
        }
        self.is_dirty = false;
        self.buffer.update(context, &self.pool)
    }
}
