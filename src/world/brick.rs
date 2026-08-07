use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Brick {
    //pub voxels: [[[u32; 8]; 8]; 8],
    pub empty: u32, // u32 bc pod annoying
}

impl Brick {
    pub const EMPTY: usize = 0xFFFFFFFF;
}
