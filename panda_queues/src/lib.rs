#![no_std]
#![feature(maybe_uninit_write_slice)]

#![feature(allocator_api)]
#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]
//#![feature(maybe_uninit_uninit_array)]

extern crate alloc;

use core::num::NonZeroUsize;
use core::mem::MaybeUninit;
use core::cmp::min;

use slice_n::Slice1;

pub trait Queue<T: Copy> {
    fn get_amount(&self) -> usize;

    fn enqueue(&mut self, t: T) -> Option<T>;

    fn enqueue_slots(&mut self) -> Option<&mut Slice1<MaybeUninit<T>>>;

    unsafe fn did_enqueue(&mut self, amount: NonZeroUsize);

    fn bulk_enqueue(&mut self, buffer: &Slice1<T>) -> usize {
        match self.enqueue_slots() {
            None => 0,
            Some(e_slots) => {
                let amount = min(e_slots.len_(), buffer.len_());
                MaybeUninit::write_slice(&mut e_slots[..amount], &buffer[..amount]);
                unsafe {
                    let amount = NonZeroUsize::new_unchecked(amount);
                    self.did_enqueue(amount);
                    amount.into()
                }
            }
        }
    }

    fn dequeue(&mut self) -> Option<T>;

    fn dequeue_slots(&mut self) -> Option<&Slice1<T>>;

    fn did_dequeue(&mut self, amount: NonZeroUsize);

    fn bulk_dequeue(&mut self, buffer: &mut Slice1<MaybeUninit<T>>) -> usize {
    match self.dequeue_slots() {
        None => 0,
        Some(d_slots) => {
            let amount = min(d_slots.len_(), buffer.len_());
            MaybeUninit::write_slice(&mut buffer[..amount], &d_slots[..amount]);
            unsafe {
                let amount = NonZeroUsize::new_unchecked(amount);
                self.did_dequeue(amount);
                amount.into()
            }
        }
    }
}
}

pub mod fixed;