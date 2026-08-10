use std::ops::{Deref, DerefMut};

use crate::{
    array_buffer::TypedArrayBuffer, render_context::RenderContext, util::free_list::FreeList,
    world::brick::Brick,
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
    pub fn new(context: &RenderContext) -> Self {
        let mut pool = FreeList::default();
        pool.push(Brick::default()); // index 0 always empty
        let buffer = TypedArrayBuffer::new_storage(context, &pool);

        Self {
            pool,
            buffer,
            is_dirty: false,
        }
    }

    pub fn add_test_brick(&mut self) {
        self.pool.push(Brick {
            empty: false as u32,
            voxels: [[[1; 8]; 8]; 8],
        });

        self.is_dirty = true;
    }

    /// Returns if the update did smth, false if it wasnt dirty.
    pub fn update(&mut self, context: &RenderContext) -> bool {
        if !self.is_dirty {
            return false;
        }
        self.is_dirty = false;
        self.buffer.update(context, &self.pool)
    }
}
