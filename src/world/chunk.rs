use crate::{
    util::constants::{BRICK_SIZE, CHUNK_SIZE},
    world::ChunkPos,
};
use fastnoise2::{
    SafeNode,
    generator::{Generator, simplex::Simplex},
};
use glam::USizeVec3;

pub const CHUNK_SIZE_IN_VOXELS: USizeVec3 = USizeVec3::new(
    (CHUNK_SIZE.x * BRICK_SIZE.x) as usize,
    (CHUNK_SIZE.y * BRICK_SIZE.y) as usize,
    (CHUNK_SIZE.z * BRICK_SIZE.z) as usize,
);

#[derive(Debug)]
pub struct Chunk {
    pub voxels: Vec<Vec<Vec<u32>>>,
}

impl Chunk {
    #[must_use]
    pub fn new(coords: &ChunkPos) -> Self {
        let node = SafeNode::from_encoded_node_tree("KQkIAACAqEIEGwAAgEIE")
            .unwrap_or_else(|_| Simplex::default().build().0);

        let chunk_voxels_x = (CHUNK_SIZE.x * BRICK_SIZE.x) as usize;
        let chunk_voxels_z = (CHUNK_SIZE.z * BRICK_SIZE.z) as usize;

        let mut noise_out = vec![0.0; chunk_voxels_x * chunk_voxels_z];
        node.gen_uniform_grid_2d(
            &mut noise_out,
            (coords.0.x * CHUNK_SIZE.x as i32 * BRICK_SIZE.x as i32) as f32,
            (coords.0.z * CHUNK_SIZE.z as i32 * BRICK_SIZE.z as i32) as f32,
            chunk_voxels_x as i32,
            chunk_voxels_z as i32,
            1.0,
            1.0,
            67676,
        );

        let mut voxels = vec![
            vec![vec![0; CHUNK_SIZE_IN_VOXELS.z]; CHUNK_SIZE_IN_VOXELS.y];
            CHUNK_SIZE_IN_VOXELS.x
        ];

        for x in 0..CHUNK_SIZE_IN_VOXELS.x {
            for z in 0..CHUNK_SIZE_IN_VOXELS.z {
                let height =
                    (noise_out[z * chunk_voxels_x + x] as usize).min(CHUNK_SIZE_IN_VOXELS.y);
                for y in 0..height {
                    voxels[x][y][z] = 1;
                }
            }
        }

        Self { voxels }
    }
}
