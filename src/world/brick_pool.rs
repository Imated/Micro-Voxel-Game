use std::ops::{Deref, DerefMut};

use bitvec::{bitvec, order::Lsb0};
use fastnoise2::SafeNode;

use crate::{
    array_buffer::TypedArrayBuffer,
    render_context::RenderContext,
    util::{
        constants::{BRICK_SIZE, CHUNK_SIZE},
        flatten,
        free_list::FreeList,
    },
    world::{
        brick::Brick,
        chunk::{Chunk, ChunkPos},
    },
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

    pub fn gen_test_chunk(&mut self, coords: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new_from_empty();

        let node = SafeNode::from_encoded_node_tree("KQkIAACArEIEGw@AEEE").unwrap();

        let chunk_voxels_x = (CHUNK_SIZE.x * BRICK_SIZE.x) as usize;
        let chunk_voxels_z = (CHUNK_SIZE.z * BRICK_SIZE.z) as usize;

        let mut noise_out = vec![0.0; chunk_voxels_x * chunk_voxels_z];
        node.gen_uniform_grid_2d(
            &mut noise_out,
            (coords.0.x * CHUNK_SIZE.x as i32 * BRICK_SIZE.x as i32) as f32,
            (coords.0.z * CHUNK_SIZE.z as i32 * BRICK_SIZE.z as i32) as f32,
            chunk_voxels_x as i32,
            chunk_voxels_z as i32,
            1.0,
            1.0,
            67676,
        );

        for bx in 0..8 {
            for by in 0..1 {
                for bz in 0..8 {
                    let mut voxels = [[[0; 8]; 8]; 8];
                    let mut occupancy = bitvec!(u32, Lsb0; 0; (BRICK_SIZE.x * BRICK_SIZE.y * BRICK_SIZE.z) as usize);

                    for vx in 0..BRICK_SIZE.x as usize {
                        for vz in 0..BRICK_SIZE.z as usize {
                            let world_x = bx * BRICK_SIZE.x as usize + vx;
                            let world_z = bz * BRICK_SIZE.z as usize + vz;
                            for vy in 0..BRICK_SIZE.y as usize {
                                if (vy as f32) < noise_out[world_z * chunk_voxels_x + world_x] {
                                    voxels[vx][vy][vz] = 1;
                                    occupancy.set(
                                        flatten(vx as u32, vy as u32, vz as u32, BRICK_SIZE)
                                            as usize,
                                        true,
                                    );
                                }
                            }
                        }
                    }

                    let mut flattened_voxels =
                        [0; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize];

                    for (x, plane) in voxels.iter().enumerate() {
                        for (y, row) in plane.iter().enumerate() {
                            for (z, voxel) in row.iter().enumerate() {
                                flattened_voxels
                                    [flatten(x as u32, y as u32, z as u32, CHUNK_SIZE) as usize] =
                                    *voxel;
                            }
                        }
                    }

                    let brick = Brick {
                        voxels: flattened_voxels,
                        occupancy_mask: occupancy.as_raw_slice().try_into().unwrap(),
                    };

                    let brick_index = flatten(bx as u32, by as u32, bz as u32, CHUNK_SIZE);
                    chunk.bricks[brick_index as usize] = self.pool.push(brick) as u32;
                }
            }
        }

        self.is_dirty = true;

        chunk
    }

    /// Returns if the update did smth, false if it wasnt dirty.
    pub fn update(&mut self, context: &RenderContext) -> bool {
        if !self.is_dirty {
            return false;
        }
        self.is_dirty = false;
        self.buffer.update(context, &self.pool)
    }

    pub fn len(&self) -> usize {
        self.pool.greatest_used_index() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.pool.reserved_slots().not_any()
    }
}
