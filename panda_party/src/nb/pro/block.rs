use panda_pile::SequenceState::{self, *};
use panda_pile::sync;
use panda_pile::nb::Producer;
use panda_pile::nb::BulkProducer;

use core::convert::{AsRef, AsMut};
use core::num::NonZeroUsize;
use core::future::{Ready, ready, Future};
use core::pin::Pin;
use core::task::{Context, Poll};

use slice_n::Slice1;
use wrapper::Wrapper;

pub struct Block<I> {
    inner: I,
    // state: State<R, L>,
}

impl<I> Unpin for Block<I> {}

// enum State<R, L> {
//     Produce(Ready<SequenceState<R, L>>),
//     Slurp(Ready<()>),
//     Stop(Ready<()>),
//     ProducerSlots,
//     // ProducerSlots(Ready<SequenceState<&Slice1<R>, L>>),
// }

pub struct Produce<I>(Block<I>);

// impl<I> Unpin for Produce<I> {}

impl<I, R, L, S> Future for Produce<I> where
    I: sync::Producer<Repeated = R, Last = L, Stopped = S> {
    type Output = SequenceState<R, L>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0.inner.produce())
    }
}

pub struct ProducerSlots<I>(Block<I>);

// impl<I> Unpin for ProducerSlots<I> {}

// impl<I, R, L, S> Future for ProducerSlots<I> where
//     I: sync::BulkProducer<Repeated = R, Last = L, Stopped = S>,
//     for<'s> R: Copy + 's, {
//     type Output = SequenceState<&'s Slice1<R>, L>;

//     fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
//         Poll::Ready(self.0.inner.producer_slots())
//     }
// }


struct Aggregate<T>(T);

impl<T> Aggregate<T> {
    fn project(&self) -> &T {
        &self.0
    }
}

// impl<'s, T> Future for Aggregate<&'s T> {
//     type Output = &'s T;

//     fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
//         Poll::Ready(self.project())
//     }
// }

impl<T> Future for Aggregate<T> {
    type Output<'s> = &'s T where T: 's;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output<'_>> {
        Poll::Ready(self.project())
    }
}



// type ProducerSlots<'s> = Ready<SequenceState<&'s Slice1<Self::Repeated>, Self::Last>> where Self: 's;

impl<I> Block<I> {
    fn as_produce(&mut self) -> &mut Produce<I> {
        unsafe{ &mut *(self as *mut Block<I> as *mut Produce<I>)}
    }

    fn as_producer_slots(&mut self) -> &mut ProducerSlots<I> {
        unsafe{ &mut *(self as *mut Block<I> as *mut ProducerSlots<I>)}
    }
}

// let ptr = &mut 0;
// let val_casts = unsafe { &mut *(ptr as *mut i32 as *mut u32) };

pub fn block<I>(inner: I) -> Block<I> {
    Block {
        inner,
        // state: State::Slurp(ready(())),
    }
}

impl<I> Wrapper<I> for Block<I> {
    fn into_inner(self) -> I {
        self.inner
    }
}

impl<I> AsRef<I> for Block<I> {
    fn as_ref(&self) -> &I {
        &self.inner
    }
}

impl<I> AsMut<I> for Block<I> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}

impl<I, R, L, S> Producer for Block<I> where
    I: sync::Producer<Repeated = R, Last = L, Stopped = S>
{
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    type Produce = Produce<I>;
    type Slurp = Ready<()>;
    type Stop = Ready<()>;

    fn produce(self: Pin<&mut Self>) -> Pin<&mut Self::Produce> {
        unsafe { self.map_unchecked_mut(Self::as_produce) }
    }

    fn slurp(self: Pin<&mut Self>) -> Pin<&mut Self::Slurp> {
        unimplemented!()
    }

    fn stop(self: Pin<&mut Self>, reason: Self::Stopped) -> Pin<&mut Self::Stop> {
        unimplemented!()
    }
}

// impl<I, R, L, S>BulkProducer for Block<I> where
//     I: sync::BulkProducer<Repeated = R, Last = L, Stopped = S>,
//     for<'s> R: Copy + 's,
// {
//     type ProducerSlots<'s> = &'s mut ProducerSlots<I> where Self: 's;

//     fn producer_slots<'s>(self: Pin<&'s mut Self>) -> Pin<&mut Self::ProducerSlots<'s>> {
//         unsafe { self.map_unchecked_mut(Self::as_producer_slots) }
//     }

//     fn did_produce(mut self: Pin<&mut Self>, amount: NonZeroUsize) {
//         self.inner.did_produce(amount)
//     }
// }

trait Food<T> {
    type Apple<'s>: Future<Output = &'s T> where Self:'s, T: 's;
    fn food<'s>(self: Pin<&mut &'s Self>) -> Pin<&mut Self::Apple<'s>>;
}

struct A<T>(T);

impl<T> Food<T> for A<T> {
    type Apple<'s> = &'s B<T> where T: 's;
    fn food<'s>(self: Pin<&mut &'s Self>) -> Pin<&mut Self::Apple<'s>> {
        unimplemented!()
    }
}

struct B<T>(T);

impl<'s, T> Future for &'s B<T> {
    type Output = &'s T;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(&self.0)
    }
}
