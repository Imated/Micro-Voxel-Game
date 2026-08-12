use std::ops::{Deref, DerefMut};

use bitvec::vec::BitVec;

/// A free-list type data structure that has O(1) contains, and other stuff i need for a brick pool.
#[allow(dead_code)]
pub struct FreeList<T> {
    // Inner memory
    slots: Vec<T>,
    // Mirror of slots except it stores how many times push has been called
    //  so u would need to call free all of those times to in order to prevent read after free
    // basically reference counting
    rc: Vec<u32>,
    // Reserved slots for O(1) contains, quick next free slot and quick greatest used index lookup
    reserved: BitVec,
    // Greatest index that is in use, used for size of the buffer needed to send this to the gpu
    greatest_used_index: usize,
    // Next free index for pushing
    next: usize,
}

impl<T> Default for FreeList<T> {
    fn default() -> Self {
        Self {
            slots: vec![],
            rc: vec![],
            reserved: BitVec::new(),
            greatest_used_index: 0,
            next: 0,
        }
    }
}

impl<T> DerefMut for FreeList<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots[0..self.greatest_used_index + 1]
    }
}

impl<T> Deref for FreeList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.slots[0..self.greatest_used_index + 1]
    }
}

impl<T: Default + PartialEq> FreeList<T> {
    pub fn push(&mut self, data: T) -> usize {
        if let Some(index) = self.contains(&data) {
            self.rc[index] += 1;
            return index;
        }

        let next = self.next;
        self.slots.insert(next, data);
        self.rc.insert(next, 1);
        self.reserved.insert(next, true);
        self.greatest_used_index = self.greatest_used_index.max(self.next);
        self.next = self.reserved.leading_ones();

        next
    }

    #[allow(clippy::unused_unit)]
    pub fn free(&mut self, index: usize) -> () {
        self.rc[index] -= 1;
        if self.rc[index] == 0 {
            self.slots.insert(index, T::default());
            self.reserved.insert(index, false);
            self.greatest_used_index = self.reserved.len() - self.reserved.trailing_zeros();
            self.next = self.next.min(index);
        }
    }

    pub fn contains(&self, data: &T) -> Option<usize> {
        for (index, slot) in self.slots.iter().enumerate() {
            if index > self.greatest_used_index {
                return None;
            }
            if *slot == *data {
                return Some(index);
            }
        }
        None
    }

    pub fn greatest_used_index(&self) -> usize {
        self.greatest_used_index
    }

    pub fn reserved_slots(&self) -> &BitVec {
        &self.reserved
    }
}
