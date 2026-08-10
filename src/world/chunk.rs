use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Chunk {
    pub bricks: [[[u32; 8]; 8]; 8],
}

impl Chunk {
    // test fn to create a chunk thats entirely filled with stuff
    pub fn new_from_full() -> Self {
        Self {
            bricks: [[[1; 8]; 8]; 8], // slot 1 in brick pool which we just hardcode to full rn
        }
    }
}
