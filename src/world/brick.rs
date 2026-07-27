use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Brick {
    pub empty: u32, // u32 bc pod annoying
}
