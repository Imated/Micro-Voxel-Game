use crate::{util::ResultExt, world::chunk::Chunk};
use glam::IVec3;
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

pub mod chunk;
pub mod render;

pub const MAX_DATA_THREADS: usize = 8;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChunkPos(pub IVec3);

#[derive(Default)]
pub struct World {
    loaded_chunks: HashMap<ChunkPos, Arc<Chunk>>,
    load_queue: Vec<ChunkPos>,
    load_tasks: HashMap<ChunkPos, Option<JoinHandle<Chunk>>>,
    is_dirty: bool,
}

impl World {
    pub fn load_chunk(&mut self, pos: ChunkPos) {
        self.load_queue.push(pos);
    }

    pub fn unload_chunk(&mut self, pos: &ChunkPos) {
        self.loaded_chunks.remove(pos);
        self.is_dirty = true;
    }

    pub fn update(&mut self) -> Option<HashMap<ChunkPos, Arc<Chunk>>> {
        self.create_data_threads();
        self.poll_data_threads();

        if self.is_dirty {
            self.is_dirty = false;
            return Some(self.loaded_chunks.clone());
        }
        None
    }

    // take some amount of chunk positions from the load queue and spawn threads for them
    fn create_data_threads(&mut self) {
        let remaining_threads = MAX_DATA_THREADS.saturating_sub(self.load_tasks.len());
        let to_spawn = remaining_threads.min(self.load_queue.len());
        for pos in self.load_queue.drain(0..to_spawn) {
            let handle = thread::spawn(move || Chunk::new(pos));
            self.load_tasks.insert(pos, Some(handle));
        }
    }

    // poll the active threads to see if they done and if so add them to loaded chunks
    fn poll_data_threads(&mut self) {
        self.load_tasks.retain(|&pos, thread_option| {
            if let Some(thread) = thread_option
                && thread.is_finished()
            {
                // take bc it gonna be removed anyways
                if let Some(thread) = thread_option.take() {
                    let chunk = thread.join().unwrap_or_abort("Failed to join thread :c");
                    self.loaded_chunks.insert(pos, Arc::new(chunk));
                    self.is_dirty = true;
                }
                return false;
            }
            true
        });
    }
}
