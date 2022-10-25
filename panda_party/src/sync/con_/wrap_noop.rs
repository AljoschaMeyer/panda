use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Consumer;
use panda_pile::sync::BulkConsumer;

use core::num::NonZeroUsize;

use core::mem::MaybeUninit;

use slice_n::Slice1;
use wrapper::Wrapper;


pub struct Inv<I>  {
    inner: I,
}

pub fn inv<I>(inner: I) -> Inv<I> {
    Inv::<I> {
        inner,
    }
}

impl<I> Wrapper<I> for Inv<I> {
    fn into_inner(self) -> I {
        self.inner
    }
}

impl<I> AsRef<I> for Inv<I> {
    fn as_ref(&self) -> &I {
        &self.inner
    }
}

impl<I> AsMut<I> for Inv<I> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Consumer for Inv<I> {
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn consume(&mut self, item: Self::Repeated) -> Option<Self::Stopped> {
        self.inner.consume(item)
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        self.inner.flush()
    }

    fn close(&mut self, last: Self::Last) -> Option<Self::Stopped> {
        self.inner.close(last)
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> BulkConsumer for Inv<I> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        self.inner.consumer_slots()
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.inner.did_consume(amount)
    }
}
