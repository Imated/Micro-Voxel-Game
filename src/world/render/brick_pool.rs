use std::ops::{Deref, DerefMut};

use crate::{
    array_buffer::TypedArrayBuffer, render_context::RenderContext, util::free_list::FreeList,
    world::render::brick::Brick,
};

pub struct BrickPool {
    pool: FreeList<Brick>,
    buffer: TypedArrayBuffer<Brick>,
    is_dirty: bool,
}

impl DerefMut for BrickPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl Deref for BrickPool {
    type Target = TypedArrayBuffer<Brick>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl BrickPool {
    #[must_use]
    pub fn new(context: &RenderContext) -> Self {
        let mut pool = FreeList::default();
        pool.push(Brick::default()); // index 0 always empty
        let buffer = TypedArrayBuffer::new_storage(context.clone(), &pool);

        Self {
            pool,
            buffer,
            is_dirty: false,
        }
    }

    pub fn push(&mut self, data: Brick) -> usize {
        self.is_dirty = true;
        self.pool.push(data)
    }

    pub fn free(&mut self, index: usize) {
        self.is_dirty = true;
        self.pool.free(index);
    }

    /// Returns if the update did smth, false if it wasnt dirty.
    pub fn update(&mut self) -> bool {
        if !self.is_dirty {
            return false;
        }
        self.is_dirty = false;
        self.buffer.update(&self.pool)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.pool.greatest_used_index() + 1
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pool.reserved_slots().not_any()
    }
}
