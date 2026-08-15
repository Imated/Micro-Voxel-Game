pub mod constants;
pub mod free_list;

pub fn flatten(x: u32, y: u32, z: u32, size: u32) -> u32 {
    z * size * size + y * size + x
}
