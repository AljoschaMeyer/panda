use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Producer;
use panda_pile::sync::BulkProducer;

use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;

use slice_n::Slice1;
use wrapper::Wrapper;

pub fn map_last<I, F>(inner: I, f: F) -> MapLast<I, F> {
    MapLast(inner, f)
}

pub struct MapLast<I, F>(I, F);

impl<I, F> Wrapper<I> for MapLast<I, F> {
    fn into_inner(self) -> I {
        self.0
    }
}

impl<I, F> AsRef<I> for MapLast<I, F> {
    fn as_ref(&self) -> &I {
        &self.0
    }
}

impl<I, F> AsMut<I> for MapLast<I, F> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.0
    }
}

impl<I, R, L, S, L2, F> Producer for MapLast<I, F> where
    I: Producer<Repeated = R, Last = L, Stopped = S>,
    F: Fn(L) -> L2
{
    type Repeated = R;
    type Last = L2;
    type Stopped = S;

    fn produce(&mut self) -> SequenceState<Self::Repeated, Self::Last> {
        match self.0.produce() {
            More(item) => More(item),
            Final(l) => Final(self.1(l))
        }
    }

    fn slurp(&mut self) -> Option<Self::Last> {
        return self.0.slurp().map(&self.1);
    }

    fn stop(&mut self, reason: Self::Stopped) -> () {
        self.0.stop(reason)
    }
}

impl<I, R, L, S, L2, F>BulkProducer for MapLast<I, F> where
    I: BulkProducer<Repeated = R, Last = L, Stopped = S>,
    R: Copy,
    F: Fn(L) -> L2
{
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        match self.0.producer_slots() {
            More(s) => More(s),
            Final(l) => Final(self.1(l))
        }
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.0.did_produce(amount)
    }
}
