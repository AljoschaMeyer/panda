use core::convert::AsRef;
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

use panda_pile::*;
use panda_pile::sync::*;

use core::mem::MaybeUninit;

use crate::maybe_uninit_slice_mut;

/// Creates a consumes which places consumed data in the given slice.
pub fn cursor<'a, T>(s: &'a mut [T]) -> Cursor<'a, T> {
    Cursor(Cursor_(s, 0))
}

/// Consumes data into a mutable slice.
pub struct Cursor<'a, T>(Cursor_<'a, T>);

impl<'a, T> Wrapper<&'a [T]> for Cursor<'a, T> {
    fn into_inner(self) -> &'a [T] {
        self.0.into_inner()
    }
}

impl<'a, T> AsRef<[T]> for Cursor<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<'a, T> AsMut<[T]> for Cursor<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut()
    }
}

impl<'a, T> Consumer for Cursor<'a, T> {
    type Repeated = T;
    
    type Last = ();

    /// Emitted when the end of the slice has been reached.
    type Stopped = ();

    fn consume(&mut self, item: T) -> Option<Self::Stopped> {
        self.0.consume(item)
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        self.0.flush()
    }

    fn close(&mut self, _last: Self::Last) -> Option<Self::Stopped> {
        self.0.close(_last)
    }
}

impl<'a, T: Copy> BulkConsumer for Cursor<'a, T> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        self.0.consumer_slots()
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.0.did_consume(amount)
    }
}

pub struct Cursor_<'a, T>(&'a mut [T], usize);

impl<'a, T> Wrapper<&'a mut [T]> for Cursor_<'a, T> {
    fn into_inner(self) -> &'a mut [T] {
        self.0
    }
}

impl<'a, T> AsRef<[T]> for Cursor_<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T> AsMut<[T]> for Cursor_<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0
    }
}

impl<'a, T> Consumer for Cursor_<'a, T> {
    type Repeated = T;
    
    type Last = ();

    /// Emitted when the end of the slice has been reached.
    type Stopped = ();

    fn consume(&mut self, item: T) -> Option<Self::Stopped> {
        if self.0.len() == self.1 {
            Some(())
        } else {
            self.0[self.1] = item;
            self.1 += 1;
            None
        }
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        None
    }

    fn close(&mut self, _: Self::Last) -> Option<Self::Stopped> {
        None
    }
}

impl<'a, T: Copy> BulkConsumer for Cursor_<'a, T> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        if self.0.len() == self.1 {
            SequenceState::Final(())
        } else {
            SequenceState::More(unsafe { Slice1::from_slice_unchecked_mut(maybe_uninit_slice_mut(&mut self.0[self.1..])) })
        }
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.1 += amount.get();
    }
}
