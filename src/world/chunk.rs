use crate::world::brick::Brick;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Chunk {
    pub bricks: [[[Brick; 8]; 8]; 8],
}

impl Chunk {
    // test fn to create a chunk thats entirely filled with stuff
    pub fn new_from_full() -> Self {
        Self {
            bricks: [[[Brick {
                empty: false as u32,
            }; 8]; 8]; 8],
        }
    }
}
