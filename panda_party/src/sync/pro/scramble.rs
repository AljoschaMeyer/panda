use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Producer;
use panda_pile::sync::BulkProducer;
use panda_queues::{Queue, fixed::Fixed};

use core::{
    cmp::min,
    num::NonZeroUsize,
    fmt::Debug,
};
use std::marker::PhantomData;

use slice_n::Slice1;
use wrapper::Wrapper;

use arbitrary::{Arbitrary, Error, Unstructured};

#[derive(Debug, PartialEq, Eq, Arbitrary)]
pub enum ProduceOperation {
    Produce,
    ProducerSlots(NonZeroUsize),
    Slurp,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProduceOperations(Box<[ProduceOperation]>);

impl ProduceOperations {
    /// Checks that the operations contain at least one non-slurp operation, and that at most 256 different operations are being allocated.
    pub fn new(operations: Box<[ProduceOperation]>) -> Option<Self> {
        let mut found_non_slurp = false;
        for op in operations.iter() {
            if *op != ProduceOperation::Slurp {
                found_non_slurp = true;
                break;
            }
        }

        if found_non_slurp && operations.len() <= 256 {
            Some(ProduceOperations(operations))
        } else {
            return None;
        }
    }
}

impl<'a> Arbitrary<'a> for ProduceOperations {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        match Self::new(Arbitrary::arbitrary(u)?) {
            Some(ops) => Ok(ops),
            None => Err(Error::IncorrectFormat),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Box<[ProduceOperation]> as Arbitrary<'a>>::size_hint(depth)
    }
}

#[derive(Debug)]
pub struct Scramble<I, R, L, S>  {
    inner: I,
    buf: Fixed<R>,
    last: Option<L>,
    operations: Box<[ProduceOperation]>,
    operations_index: usize,
    s: PhantomData<S>,
}

pub fn scramble<I, R, L, S>(inner: I, operations: ProduceOperations, capacity: NonZeroUsize) -> Scramble<I, R, L, S> {
    Scramble::<I, R, L, S> {
        inner,
        buf: Fixed::new(capacity),
        last: None,
        operations: operations.0,
        operations_index: 0,
        s: PhantomData,
    }
}

// Operates by draining its `buf` and `last`, and filling them via the `operations` when they become empty.
impl<I: BulkProducer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Producer for Scramble<I, R, L, S> {
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn produce(&mut self) -> SequenceState<Self::Repeated, Self::Last> {
        if self.buf.get_amount() == 0 && self.last.is_some() {
            return Final(self.last.take().unwrap());
        }

        while self.buf.get_amount() == 0 {
            if let Some(l) = self.perform_operation() {
                return Final(l);
            }
        }

        return More(self.buf.dequeue().unwrap());
    }

    fn slurp(&mut self) -> Option<Self::Last> {
        if self.buf.get_amount() == 0 && self.last.is_some() {
            return Some(self.last.take().unwrap());
        }

        while self.buf.get_amount() < self.buf.get_capacity().get() {
            if let Some(l) = self.perform_operation() {
                self.last = Some(l);
                return None;
            }
        }

        return self.inner.slurp();
    }

    fn stop(&mut self, reason: Self::Stopped) -> () {
        return self.inner.stop(reason);
    }
}

impl<I: BulkProducer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> BulkProducer for Scramble<I, R, L, S> {
    fn producer_slots(&mut self) -> SequenceState<&Slice1<Self::Repeated>, Self::Last> {
        if self.buf.get_amount() == 0 && self.last.is_some() {
            return Final(self.last.take().unwrap());
        }

        while self.buf.get_amount() == 0 {
            if let Some(l) = self.perform_operation() {
                return Final(l);
            }
        }

        More(self.buf.dequeue_slots().unwrap())
    }

    fn did_produce(&mut self, amount: NonZeroUsize) {
        self.buf.did_dequeue(amount)
    }
}

impl<I: BulkProducer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Scramble<I, R, L, S> {
    fn perform_operation(&mut self) -> Option<L> {
        debug_assert!(self.buf.get_amount() < self.buf.get_capacity().get());

        match self.operations[self.operations_index] {
            ProduceOperation::Produce => {
                match self.inner.produce() {
                    Final(l) => return Some(l),
                    More(r) => {
                        if let Some(_) = self.buf.enqueue(r) {
                            panic!("Erronous scrambler implementation.");
                        }
                    }
                }
            }
            ProduceOperation::ProducerSlots(n) => {
                match self.inner.producer_slots() {
                    Final(l) => return Some(l),
                    More(slots) => {
                        let l = slots.len_();
                        let slots = unsafe { Slice1::from_slice_unchecked(&slots[..min(l, n.get())]) };
                        let consume_amount = self.buf.bulk_enqueue(slots);
                        self.inner.did_produce(NonZeroUsize::new(consume_amount).expect("Erronous scrambler implementation."));
                    },
                }
            }
            ProduceOperation::Slurp => {
                if let Some(l) = self.inner.slurp() {
                    return Some(l);
                }
            }
        }

        self.operations_index = (self.operations_index + 1) % self.operations.len();
        None
    }
}

impl<I, R, L, S> Wrapper<I> for Scramble<I, R, L, S> {
    fn into_inner(self) -> I {
        self.inner
    }
}

impl<I, R, L, S> AsRef<I> for Scramble<I, R, L, S> {
    fn as_ref(&self) -> &I {
        &self.inner
    }
}

impl<I, R, L, S> AsMut<I> for Scramble<I, R, L, S> {
    fn as_mut(&mut self) -> &mut I {
        &mut self.inner
    }
}
