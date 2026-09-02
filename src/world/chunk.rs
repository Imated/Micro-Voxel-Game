use bytemuck::{Pod, Zeroable};
use glam::IVec3;

use crate::util::constants::CHUNK_SIZE;

pub struct ChunkPos(pub IVec3);

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Chunk {
    pub bricks: [u32; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize],
}

impl Chunk {
    // test fn to create a chunk thats entirely filled with stuff
    #[must_use]
    pub const fn new_from_full() -> Self {
        Self {
            // slot 1 in brick pool which we just hardcode to full rn
            bricks: [1; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize],
        }
    }

    #[must_use]
    pub const fn new_from_empty() -> Self {
        Self {
            // slot 0 in brick pool is always empty
            bricks: [0; (CHUNK_SIZE.x * CHUNK_SIZE.y * CHUNK_SIZE.z) as usize],
        }
    }
}
