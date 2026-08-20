use bytemuck::{Pod, Zeroable};

use crate::util::constants::BRICK_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Pod, Zeroable)]
pub struct Brick {
    pub voxels: [u32; (BRICK_SIZE.x * BRICK_SIZE.y * BRICK_SIZE.z) as usize],
    pub occupancy_mask: [u32; BRICK_SIZE.z as usize * 2],
}

impl Default for Brick {
    fn default() -> Self {
        Self {
            occupancy_mask: [0; BRICK_SIZE.z as usize * 2],
            voxels: [0; (BRICK_SIZE.x * BRICK_SIZE.y * BRICK_SIZE.z) as usize],
        }
    }
}

impl Brick {
    pub const EMPTY: usize = 0;
}
