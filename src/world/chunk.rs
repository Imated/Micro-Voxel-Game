use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
pub struct Chunk {
    pub empty: u32, // fake bool bc pod annoying
}