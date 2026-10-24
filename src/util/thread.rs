use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

use crate::{util::ResultExt, world::ChunkPos};

pub const MAX_DATA_THREADS: usize = 8;

type Task<T> = Box<dyn FnOnce(ChunkPos) -> T + Send + 'static>;

#[derive(Default)]
pub struct ThreadPool<T: Send + 'static> {
    task_queue: HashMap<ChunkPos, Task<T>>,
    tasks: HashMap<ChunkPos, JoinHandle<T>>,
}

impl<T: Send + 'static> ThreadPool<T> {
    pub fn queue<F: FnOnce(ChunkPos) -> T + Send + 'static>(&mut self, pos: ChunkPos, f: F) {
        self.task_queue.insert(pos, Box::new(f));
    }

    #[must_use]
    pub fn is_pending(&self, pos: &ChunkPos) -> bool {
        self.task_queue.contains_key(pos) || self.tasks.contains_key(pos)
    }

    // take some amount of chunk positions from the load queue and spawn threads for them
    pub fn create_data_threads(&mut self) {
        let remaining_threads = MAX_DATA_THREADS.saturating_sub(self.tasks.len());
        let positions: Vec<ChunkPos> = self
            .task_queue
            .keys()
            .filter(|pos| !self.tasks.contains_key(pos))
            .copied()
            .take(remaining_threads)
            .collect();
        for pos in positions {
            let Some(f) = self.task_queue.remove(&pos) else {
                continue;
            };
            self.tasks.insert(pos, thread::spawn(move || f(pos)));
        }
    }

    // poll the active threads and return the results of the finished ones
    pub fn poll_data_threads(&mut self) -> HashMap<ChunkPos, Arc<T>> {
        let finished: Vec<ChunkPos> = self
            .tasks
            .iter()
            .filter(|(_, handle)| handle.is_finished())
            .map(|(&pos, _)| pos)
            .collect();

        finished
            .into_iter()
            .filter_map(|pos| {
                let handle = self.tasks.remove(&pos)?;
                let result = handle.join().unwrap_or_abort("Failed to join thread :c");
                Some((pos, Arc::new(result)))
            })
            .collect()
    }
}
