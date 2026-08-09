use std::ops::{Deref, DerefMut};

use bitvec::vec::BitVec;

/// A free-list type data structure that has O(1) contains, and other stuff i need for a brick pool.
#[allow(dead_code)]
pub struct FreeList<T> {
    // Inner memory
    slots: Vec<T>,
    // Reserved slots for O(1) contains, quick next free slot and quick greatest used index lookup
    reserved: BitVec,
    // Greatest index that is in use, used for size of the buffer needed to send this to the gpu
    greatest_used_index: usize,
    // Next free index for pushing
    next: usize,
}

impl<T: Default> Default for FreeList<T> {
    fn default() -> Self {
        Self {
            slots: vec![],
            reserved: BitVec::new(),
            greatest_used_index: 0,
            next: 0,
        }
    }
}

impl<T: Default> DerefMut for FreeList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}

impl<T: Default> Deref for FreeList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}

impl<T: Default> FreeList<T> {
    pub fn push(&mut self, data: T) -> usize {
        let next = self.next;
        self.slots.insert(next, data);
        self.reserved.insert(next, true);
        self.greatest_used_index = self.greatest_used_index.max(self.next);
        self.next = self.reserved.leading_ones();

        next
    }

    #[allow(clippy::unused_unit)]
    pub fn free(&mut self, index: usize) -> () {
        self.slots.insert(index, T::default());
        self.reserved.insert(index, false);
        self.greatest_used_index = self.reserved.len() - self.reserved.trailing_zeros();
        self.next = self.reserved.leading_ones();
    }
}
