use bitvec::{bitvec, order::Lsb0};
use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, RwLock};

use crate::{
    util::{
        constants::{BRICK_SIZE, CHUNK_SIZE},
        flatten,
    },
    world::{
        chunk::Chunk,
        render::{brick::Brick, brick_pool::BrickPool},
    },
};

pub const BRICKS_PER_CHUNK: usize = (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize;
pub const BRICK_MASK_WORDS: usize = BRICKS_PER_CHUNK / 32;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct GpuChunk {
    pub brick_mask: [u32; BRICK_MASK_WORDS],
    pub nonempty: u32,
    pad: [u32; 3],
    pub bricks: [u32; BRICKS_PER_CHUNK],
}

impl Default for GpuChunk {
    fn default() -> Self {
        Self {
            // slot 0 in brick pool is always empty
            bricks: [0; BRICKS_PER_CHUNK],
            brick_mask: [0; BRICK_MASK_WORDS],
            nonempty: 0,
            pad: [0; 3],
        }
    }
}

impl GpuChunk {
    pub fn new(chunk: &Arc<RwLock<Chunk>>, brick_pool: &mut BrickPool) -> Self {
        let mut gpu_chunk = Self::default();
        let chunk_guard = chunk.read().unwrap();

        for bx in 0..CHUNK_SIZE.x {
            for by in 0..CHUNK_SIZE.y {
                for bz in 0..CHUNK_SIZE.z {
                    let mut occupancy = bitvec!(u32, Lsb0; 0; (BRICK_SIZE.x * BRICK_SIZE.y * BRICK_SIZE.z) as usize);
                    let mut any_solid = false;

                    for vx in 0..BRICK_SIZE.x {
                        for vy in 0..BRICK_SIZE.y {
                            for vz in 0..BRICK_SIZE.z {
                                let chunk_x = (bx * BRICK_SIZE.x + vx) as usize;
                                let chunk_y = (by * BRICK_SIZE.y + vy) as usize;
                                let chunk_z = (bz * BRICK_SIZE.z + vz) as usize;

                                let solid = chunk_guard.voxels[chunk_x][chunk_y][chunk_z] != 0;
                                any_solid |= solid;
                                occupancy.set(flatten(vx, vy, vz, BRICK_SIZE) as usize, solid);
                            }
                        }
                    }

                    if !any_solid {
                        continue;
                    }

                    let brick = Brick {
                        occupancy_mask: occupancy.as_raw_slice().try_into().unwrap_or_default(),
                    };

                    let brick_index = flatten(bx, by, bz, CHUNK_SIZE) as usize;
                    gpu_chunk.bricks[brick_index] = brick_pool.push(brick) as u32;
                }
            }
        }

        gpu_chunk.rebuild_metadata();
        gpu_chunk
    }

    pub fn rebuild_metadata(&mut self) {
        self.brick_mask = [0; BRICK_MASK_WORDS];
        for (slot, &index) in self.bricks.iter().enumerate() {
            if index != Brick::EMPTY as u32 {
                self.brick_mask[slot / 32] |= 1 << (slot % 32);
            }
        }
        self.nonempty = u32::from(self.brick_mask.iter().any(|&w| w != 0));
    }
}
