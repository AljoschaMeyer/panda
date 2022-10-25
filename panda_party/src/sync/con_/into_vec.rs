use core::num::NonZeroUsize;
use core::mem::MaybeUninit;
use std::alloc::{Allocator, Global};
use std::vec::Vec;

use slice_n::Slice1;
use wrapper::Wrapper;

use panda_pile::{*, SequenceState::*};
use panda_pile::sync::*;

pub fn into_vec<T>() -> IntoVec<T> {
    IntoVec { v: Vec::new() }
}

pub fn into_vec_in<T,  A: Allocator>(alloc: A) -> IntoVec<T, A> {
    IntoVec { v: Vec::new_in(alloc) }
}

/// Collects data and can at any point be converted into a `Vec<T>.
pub struct IntoVec<T, A = Global> where A: Allocator {
    v: Vec<T, A>
}

impl<T> IntoVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        self.v
    }
}
impl<T> Consumer for IntoVec<T> {
    type Repeated = T;
    
    type Last = ();

    /// Emitted when the end of the slice has been reached.
    type Stopped = ();

    fn consume(&mut self, item: T) -> Option<Self::Stopped> {
        self.v.push(item);
        return None;
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        None
    }

    fn close(&mut self, _: Self::Last) -> Option<Self::Stopped> {
        None
    }
}

impl<'a, T: Copy> BulkConsumer for IntoVec<T> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        if self.v.capacity() == self.v.len() {
            self.v.reserve((self.v.capacity() * 2) + 1);
        }

        let p = self.v.as_mut_ptr();
        let l = self.v.capacity() - self.v.len();
        More(unsafe { slice_n::from_raw_parts_unchecked_mut(p.add(self.v.len()), l).as_maybe_uninit_mut() })
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.v.set_len(self.v.len() + amount.get());
    }
}

impl<T> Wrapper<Vec<T>> for IntoVec<T> {
    fn into_inner(self) -> Vec<T> {
        self.v
    }
}

impl<T> AsRef<Vec<T>> for IntoVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.v
    }
}

impl<T> AsMut<Vec<T>> for IntoVec<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.v
    }
}
