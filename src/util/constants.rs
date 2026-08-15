use glam::{IVec3, ivec3};

/// how many chunks are loaded at a time
pub const WORLD_SIZE: IVec3 = ivec3(8, 1, 8);
/// how many bricks are in a chunk
pub const CHUNK_SIZE: IVec3 = ivec3(8, 8, 8);
/// how many voxels are in a brick
pub const BRICK_SIZE: IVec3 = ivec3(8, 8, 8);
pub const WORLD_SIZE_HALF: IVec3 = ivec3(4, 1, 4);
