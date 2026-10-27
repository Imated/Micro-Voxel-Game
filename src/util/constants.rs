use glam::{UVec3, uvec3};

/// size of one voxel
pub const VOXEL_SIZE_METERS: f32 = 0.1;
pub const VOXELS_PER_METER: f32 = 1.0 / VOXEL_SIZE_METERS;

/// how many chunks are loaded at a time
pub const WORLD_SIZE: UVec3 = uvec3(64, 1, 64);
/// how many bricks are in a chunk
pub const CHUNK_SIZE: UVec3 = uvec3(8, 8, 8);
/// how many voxels are in a brick
pub const BRICK_SIZE: UVec3 = uvec3(8, 8, 8);
pub const WORLD_SIZE_HALF: UVec3 = uvec3(WORLD_SIZE.x / 2, WORLD_SIZE.y / 2, WORLD_SIZE.z / 2);
