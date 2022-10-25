use core::convert::AsRef;
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

use panda_pile::*;
use panda_pile::sync::*;

/// Creates a producer which produces the data in the given slice.
pub fn cursor<'a, T>(s: &'a [T]) -> Cursor<'a, T> {
    Cursor(Cursor_(s, 0))
}

/// Produces data from a slice.
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

impl<'a, T: Clone> Producer for Cursor<'a, T> {
    type Repeated = T;
    /// Emitted when the end of the slice has been reached.
    type Last = ();

    type Stopped = ();

    fn produce(&mut self) -> SequenceState<T, Self::Last> {
        self.0.produce()
    }

    fn slurp(&mut self) -> Option<()> {
        self.0.slurp()
    }

    fn stop(&mut self, _reason: Self::Stopped) {
        self.0.stop(_reason)
    }
}

impl<'a, T: Copy> BulkProducer for Cursor<'a, T> {
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        self.0.producer_slots()
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.0.did_produce(amount)
    }
}

pub struct Cursor_<'a, T>(&'a [T], usize);

impl<'a, T> Wrapper<&'a [T]> for Cursor_<'a, T> {
    fn into_inner(self) -> &'a [T] {
        self.0
    }
}

impl<'a, T> AsRef<[T]> for Cursor_<'a, T> {
    fn as_ref(&self) -> &[T] {
        self.0
    }
}

impl<'a, T: Clone> Producer for Cursor_<'a, T> {
    type Repeated = T;
    /// Emitted when the end of the slice has been reached.
    type Last = ();

    type Stopped = ();

    fn produce(&mut self) -> SequenceState<T, Self::Last> {
        if self.0.len() == self.1 {
            SequenceState::Final(())
        } else {
            let item = self.0[self.1].clone();
            self.1 += 1;
            SequenceState::More(item)
        }
    }

    fn slurp(&mut self) -> Option<()> {
        None
    }

    fn stop(&mut self, _reason: Self::Stopped) {}
}

impl<'a, T: Copy> BulkProducer for Cursor_<'a, T> {
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        match Slice1::from_slice(&self.0[self.1..]) {
            Some(s) => SequenceState::More(s),
            None => SequenceState::Final(())
        }
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.1 += amount.get();
    }
}
