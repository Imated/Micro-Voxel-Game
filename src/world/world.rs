use glam::{IVec3, ivec3};

pub struct World;

impl World {
    pub const WORLD_SIZE: IVec3 = ivec3(8, 1, 8);
    pub const WORLD_SIZE_HALF: IVec3 = ivec3(4, 1, 4);
}
