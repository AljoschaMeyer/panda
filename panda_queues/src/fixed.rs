use alloc::alloc::{Allocator, Global};
use alloc::boxed::Box;
use core::num::NonZeroUsize;
use core::mem::MaybeUninit;

use slice_n::Slice1;

use crate::Queue;

/// A queue holding up to a certain number of items. The capacity is set upon creation and remains fixed. Performs a single heap location on creation.
#[derive(Debug)]
pub struct Fixed<T, A = Global> where A: Allocator {
    data: Box<[MaybeUninit<T>], A>,
    // reading resumes from this position
    read: usize,
    // amount of valid data
    amount: usize,
}

impl<T> Fixed<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Fixed {
            data: Box::new_uninit_slice(capacity.get()),
            read: 0,
            amount: 0,
        }
    }
}

impl<T, A: Allocator> Fixed<T, A> {
    pub fn new_in(capacity: NonZeroUsize, alloc: A) -> Self {
        Fixed {
            data: Box::new_uninit_slice_in(capacity.get(), alloc),
            read: 0,
            amount: 0,
        }
    }

    pub fn get_capacity(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.data.len()) }
    }
}

impl<T: Copy, A: Allocator> Fixed<T, A> {
    fn is_data_contiguous(&self) -> bool {
        self.read + self.amount < self.capacity()
    }

    fn available_fst(&mut self) -> &mut [MaybeUninit<T>] {
        let cap = self.capacity();
        if self.is_data_contiguous() {
            return &mut self.data[self.read + self.amount..cap];
        } else {
            return &mut self.data[(self.read + self.amount) % cap..self.read];
        };
    }

    fn readable_fst(&mut self) -> &[MaybeUninit<T>] {
        if self.is_data_contiguous() {
            return &self.data[self.read..self.write_to()];
        } else {
            return &self.data[self.read..];
        }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }

    fn write_to(&self) -> usize {
        (self.read + self.amount) % self.capacity()
    }
}

impl<T: Copy, A: Allocator> Queue<T> for Fixed<T, A> {
    fn get_amount(&self) -> usize {
        self.amount
    }

    fn enqueue(&mut self, item: T) -> Option<T> {
        if self.amount == self.capacity() {
            return Some(item);
        } else {
            self.data[self.write_to()].write(item);
            self.amount += 1;
            return None;
        }
    }

    fn enqueue_slots(&mut self) -> Option<&mut Slice1<MaybeUninit<T>>> {
        if self.amount >= self.capacity() {
            return None;
        }

        return Some(unsafe {Slice1::from_slice_unchecked_mut(self.available_fst())});
    }

    unsafe fn did_enqueue(&mut self, amount: NonZeroUsize) {
        self.amount += amount.get();
    }

    fn dequeue(&mut self) -> Option<T> {
        if self.amount == 0 {
            return None;
        }

        let old_r = self.read;
        self.read = (self.read + 1) % self.capacity();
        self.amount -= 1;
        return Some(unsafe { self.data[old_r].assume_init() } );
    }

    fn dequeue_slots(&mut self) -> Option<&Slice1<T>> {
        if self.amount == 0 {
            return None;
        }

        Some(unsafe {Slice1::from_slice_unchecked(MaybeUninit::slice_assume_init_ref(self.readable_fst()))})
    }

    fn did_dequeue(&mut self, amount: NonZeroUsize) {
        self.read = (self.read + amount.get()) % self.capacity();
        self.amount -= amount.get();
    }
}

    

