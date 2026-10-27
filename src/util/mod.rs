use glam::{UVec3, ivec3};
use std::{collections::HashSet, fmt::Debug, process::abort};
use tracing::error;

use crate::{
    util::constants::{WORLD_SIZE, WORLD_SIZE_HALF},
    world::ChunkPos,
};

pub mod constants;
pub mod free_list;
pub mod pipeline;
pub mod thread;

#[must_use]
pub const fn flatten(x: u32, y: u32, z: u32, size: UVec3) -> u32 {
    z * size.y * size.x + y * size.x + x
}

#[must_use]
pub fn torodial_pos(pos: ChunkPos) -> usize {
    let offsetted_coords = pos.0 + WORLD_SIZE_HALF.as_ivec3();
    let index = (offsetted_coords.rem_euclid(WORLD_SIZE.as_ivec3())).as_uvec3();
    flatten(index.x, index.y, index.z, WORLD_SIZE) as usize
}

#[must_use]
pub fn get_chunks_in_radius(center: ChunkPos, radius: i32) -> HashSet<ChunkPos> {
    let mut chunks = HashSet::new();
    let radius_sq = radius * radius;

    for x in -radius..=radius {
        for z in -radius..=radius {
            let dist_sq = x * x + z * z;
            if dist_sq <= radius_sq {
                chunks.insert(ChunkPos(center.0 + ivec3(x, 0, z)));
            }
        }
    }

    chunks
}

pub trait ResultExt<T> {
    fn unwrap_or_abort<S: Into<String> + Debug>(self, msg: S) -> T;
}

impl<T, E: Debug> ResultExt<T> for Result<T, E> {
    fn unwrap_or_abort<S: Into<String> + Debug>(self, msg: S) -> T {
        self.unwrap_or_else(|err| {
            error!("{msg:?} Error: {err:?}");
            abort()
        })
    }
}
