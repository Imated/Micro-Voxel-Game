use crate::world::chunk::Chunk;
use glam::IVec3;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub mod chunk;
pub mod render;

pub struct ChunkPos(pub IVec3);

pub struct World {
    loaded_chunks: HashMap<ChunkPos, Arc<RwLock<Chunk>>>,
}
