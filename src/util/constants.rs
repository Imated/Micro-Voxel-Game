use glam::{UVec3, uvec3};

/// how many chunks are loaded at a time
pub const WORLD_SIZE: UVec3 = uvec3(8, 1, 8);
/// how many bricks are in a chunk
pub const CHUNK_SIZE: UVec3 = uvec3(8, 8, 8);
/// how many voxels are in a brick
pub const BRICK_SIZE: UVec3 = uvec3(8, 8, 8);
pub const WORLD_SIZE_HALF: UVec3 = uvec3(4, 1, 4);
