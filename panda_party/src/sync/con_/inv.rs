use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Consumer;
use panda_pile::sync::BulkConsumer;

use core::num::NonZeroUsize;

use core::mem::MaybeUninit;

use slice_n::Slice1;
use wrapper::Wrapper;


pub struct Inv<I>  {
    inner: I,
    active: bool,
    exposed_slots: usize,
}

pub fn inv<I>(inner: I) -> Inv<I> {
    Inv::<I> {
        inner,
        active: true,
        exposed_slots: 0,
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

impl<I> Inv<I> {
    fn check_inactive(&self) {
        if !self.active {
            panic!("may not call `Consumer` methods after the sequence has ended");
        }
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Consumer for Inv<I> {
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn consume(&mut self, item: Self::Repeated) -> Option<Self::Stopped> {
        self.check_inactive();
        match self.inner.consume(item) {
            Some(s) => {
                self.active = false;
                return Some(s);
            }
            None => {
                self.exposed_slots = 0;
                return None;
            }
        }
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        self.check_inactive();
        match self.inner.flush() {
            Some(s) => {
                self.active = false;
                return Some(s);
            }
            None => {
                self.exposed_slots = 0;
                return None;
            }
        }
    }

    fn close(&mut self, last: Self::Last) -> Option<Self::Stopped> {
        self.check_inactive();
        self.active = false;
        return self.inner.close(last);
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> BulkConsumer for Inv<I> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        self.check_inactive();
        match self.inner.consumer_slots() {
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

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.check_inactive();
        let as_usize: usize = amount.into();
        if as_usize > self.exposed_slots {
            panic!("may not call `did_The consumer` with a greater amount than slots have been exposed");
        } else {
            self.exposed_slots -= as_usize;
            return self.inner.did_consume(amount);
        }
    }
}
