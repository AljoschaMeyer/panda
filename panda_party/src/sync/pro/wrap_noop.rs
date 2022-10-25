use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Producer;
use panda_pile::sync::BulkProducer;

use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

pub fn inv<I>(inner: I) -> Inv<I> {
    Inv {
        inner,
    }
}

pub struct Inv<I> {
    inner: I,
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


impl<I, R, L, S> Producer for Inv<I> where
    I: Producer<Repeated = R, Last = L, Stopped = S>
{
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn produce(&mut self) -> SequenceState<Self::Repeated, Self::Last> {
        self.inner.produce()
    }

    fn slurp(&mut self) -> Option<Self::Last> {
        self.inner.slurp()
    }

    fn stop(&mut self, reason: Self::Stopped) -> () {
        return self.inner.stop(reason);
    }
}

impl<I, R, L, S>BulkProducer for Inv<I> where
    I: BulkProducer<Repeated = R, Last = L, Stopped = S>,
    R: Copy,
{
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        self.inner.producer_slots()
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        return self.inner.did_produce(amount);
    }
}
