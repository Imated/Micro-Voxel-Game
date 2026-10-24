use crate::{util::thread::ThreadPool, world::chunk::Chunk};
use glam::IVec3;
use std::{collections::HashMap, sync::Arc};

pub mod chunk;
pub mod render;

pub const MAX_DATA_THREADS: usize = 8;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChunkPos(pub IVec3);

#[derive(Default)]
pub struct World {
    loaded_chunks: HashMap<ChunkPos, Arc<Chunk>>,
    thread_pool: ThreadPool<Chunk>,
    is_dirty: bool,
}

impl World {
    pub fn load_chunk(&mut self, pos: ChunkPos) {
        if self.loaded_chunks.contains_key(&pos) || self.thread_pool.is_pending(&pos) {
            return;
        }
        self.thread_pool.queue(pos, Chunk::new);
    }

    pub fn unload_chunk(&mut self, pos: &ChunkPos) {
        self.loaded_chunks.remove(pos);
        self.is_dirty = true;
    }

    /// returns the chunks that finished loading since the last update
    pub fn update(&mut self) -> Option<HashMap<ChunkPos, Arc<Chunk>>> {
        self.thread_pool.create_data_threads();
        let finished_chunks = self.thread_pool.poll_data_threads();
        self.is_dirty |= !finished_chunks.is_empty();
        self.loaded_chunks.extend(finished_chunks.clone());

        if self.is_dirty {
            self.is_dirty = false;
            return Some(finished_chunks);
        }
        None
    }
}
