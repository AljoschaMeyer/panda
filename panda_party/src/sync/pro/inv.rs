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
        active: true,
        exposed_slots: 0,
    }
}

pub struct Inv<I> {
    inner: I,
    active: bool,
    exposed_slots: usize,
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

impl<I> Inv<I> {
    fn check_inactive(&self) {
        if !self.active {
            panic!("may not call `Producer` methods after the sequence has ended");
        }
    }
}

impl<I, R, L, S> Producer for Inv<I> where
    I: Producer<Repeated = R, Last = L, Stopped = S>
{
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn produce(&mut self) -> SequenceState<Self::Repeated, Self::Last> {
        self.check_inactive();        
        match self.inner.produce() {
            More(r) => {
                self.exposed_slots = 0;
                return More(r);
            }
            Final(l) => {
                self.active = false;
                return Final(l);
            }
        }
    }

    fn slurp(&mut self) -> Option<Self::Last> {
        self.check_inactive();
        match self.inner.slurp() {
            Some(l) => {
                self.active = false;
                return Some(l);
            }
            None => {
                self.exposed_slots = 0;
                return None;
            }
        }
    }

    fn stop(&mut self, reason: Self::Stopped) -> () {
        self.check_inactive();
        self.active = false;
        return self.inner.stop(reason);
    }
}

impl<I, R, L, S>BulkProducer for Inv<I> where
    I: BulkProducer<Repeated = R, Last = L, Stopped = S>,
    R: Copy,
{
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        self.check_inactive();
        match self.inner.producer_slots() {
            More(s) => {
                self.exposed_slots = s.len().into();
                return More(s);
            }
            Final(l) => {
                self.active = false;
                return Final(l);
            }
        }
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.check_inactive();
        let as_usize: usize = amount.into();
        if as_usize > self.exposed_slots {
            panic!("may not call `did_produce` with a greater amount than slots have been exposed");
        } else {
            self.exposed_slots -= as_usize;
            return self.inner.did_produce(amount);
        }
    }
}
