use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Brick {
    //pub voxels: [[[u32; 8]; 8]; 8],
    pub empty: u32, // u32 bc pod annoying
}

impl Default for Brick {
    fn default() -> Self {
        Self { empty: true as u32 }
    }
}

impl Brick {
    pub const EMPTY: usize = 0xFFFFFFFF;
}
