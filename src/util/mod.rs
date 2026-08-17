use glam::UVec3;

pub mod constants;
pub mod free_list;

pub fn flatten(x: u32, y: u32, z: u32, size: UVec3) -> u32 {
    z * size.y * size.x + y * size.x + x
}
