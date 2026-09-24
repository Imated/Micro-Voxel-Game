use bytemuck::{Pod, Zeroable};
use glam::IVec3;

use crate::{util::constants::CHUNK_SIZE, world::brick::Brick};

pub struct ChunkPos(pub IVec3);

pub const BRICKS_PER_CHUNK: usize = (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize;
pub const BRICK_MASK_WORDS: usize = BRICKS_PER_CHUNK / 32;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Chunk {
    pub brick_mask: [u32; BRICK_MASK_WORDS],
    pub nonempty: u32,
    pad: [u32; 3],
    pub bricks: [u32; BRICKS_PER_CHUNK],
}

impl Chunk {
    // test fn to create a chunk thats entirely filled with stuff
    #[must_use]
    pub const fn new_from_full() -> Self {
        Self {
            // slot 1 in brick pool which we just hardcode to full rn
            bricks: [1; BRICKS_PER_CHUNK],
            brick_mask: [u32::MAX; BRICK_MASK_WORDS],
            nonempty: 1,
            pad: [0; 3],
        }
    }

    #[must_use]
    pub const fn new_from_empty() -> Self {
        Self {
            // slot 0 in brick pool is always empty
            bricks: [0; BRICKS_PER_CHUNK],
            brick_mask: [0; BRICK_MASK_WORDS],
            nonempty: 0,
            pad: [0; 3],
        }
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
