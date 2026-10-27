use crate::{util::thread::ThreadPool, world::chunk::Chunk};
use glam::IVec3;
use std::{
    collections::HashMap,
    mem,
    sync::Arc,
};

pub mod chunk;
pub mod render;

pub const MAX_DATA_THREADS: usize = 8;

#[derive(Debug, Clone)]
pub enum WorldEvent {
    ChunkLoaded { pos: ChunkPos, chunk: Arc<Chunk> },
    ChunkUnloaded(ChunkPos),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChunkPos(pub IVec3);

#[derive(Default)]
pub struct World {
    loaded_chunks: HashMap<ChunkPos, Arc<Chunk>>,
    thread_pool: ThreadPool<Chunk>,
    unloaded_events: Vec<WorldEvent>,
    is_dirty: bool,
}

impl World {
    pub fn load_chunk(&mut self, pos: ChunkPos) {
        if self.loaded_chunks.contains_key(&pos) || self.thread_pool.is_pending(&pos) {
            return;
        }
        self.thread_pool.queue(pos, Chunk::new);
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) {
        self.loaded_chunks.remove(&pos);
        self.unloaded_events.push(WorldEvent::ChunkUnloaded(pos));
        self.thread_pool.deque(pos);
        self.is_dirty = true;
    }

    /// returns the chunks that were unloaded and finished loading since the last update,
    /// unloads first so a slot is cleared before it gets reused
    pub fn update(&mut self) -> Vec<WorldEvent> {
        self.thread_pool.create_data_threads();
        let finished_chunks = self.thread_pool.poll_data_threads();
        self.is_dirty |= !finished_chunks.is_empty();

        if !self.is_dirty {
            return Vec::new();
        }
        self.is_dirty = false;

        let mut events = mem::take(&mut self.unloaded_events);
        for (pos, chunk) in finished_chunks {
            self.loaded_chunks.insert(pos, chunk.clone());
            events.push(WorldEvent::ChunkLoaded { pos, chunk });
        }
        events
    }
}
