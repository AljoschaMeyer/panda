use panda_pile::SequenceState::{self, *};
use panda_pile::sync::Consumer;
use panda_pile::sync::BulkConsumer;
use panda_queues::{Queue, fixed::Fixed};

use core::{
    cmp::min,
    num::NonZeroUsize,
    fmt::Debug,
};
use core::marker::PhantomData;
use core::mem::MaybeUninit;

use slice_n::Slice1;
use wrapper::Wrapper;

use arbitrary::{Arbitrary, Error, Unstructured};

#[derive(Debug, PartialEq, Eq, Arbitrary)]
pub enum ConsumeOperation {
    Consume,
    ConsumerSlots(NonZeroUsize),
    Flush,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConsumeOperations(Box<[ConsumeOperation]>);

impl ConsumeOperations {
    /// Checks that the operations contain at least one non-flush operation, and that at most 256 different operations are being allocated.
    pub fn new(operations: Box<[ConsumeOperation]>) -> Option<Self> {
        let mut found_non_flush = false;
        for op in operations.iter() {
            if *op != ConsumeOperation::Flush {
                found_non_flush = true;
                break;
            }
        }

        if found_non_flush && operations.len() <= 256 {
            Some(ConsumeOperations(operations))
        } else {
            return None;
        }
    }
}

impl<'a> Arbitrary<'a> for ConsumeOperations {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self, Error> {
        match Self::new(Arbitrary::arbitrary(u)?) {
            Some(ops) => Ok(ops),
            None => Err(Error::IncorrectFormat),
        }
    }

    #[inline]
    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <Box<[ConsumeOperation]> as Arbitrary<'a>>::size_hint(depth)
    }
}

#[derive(Debug)]
pub struct Scramble<I, R, L, S>  {
    inner: I,
    buf: Fixed<R>,
    operations: Box<[ConsumeOperation]>,
    operations_index: usize,
    sl: PhantomData<(S, L)>,
}

pub fn scramble<I, R, L, S>(inner: I, operations: ConsumeOperations, capacity: NonZeroUsize) -> Scramble<I, R, L, S> {
    Scramble::<I, R, L, S> {
        inner,
        buf: Fixed::new(capacity),
        operations: operations.0,
        operations_index: 0,
        sl: PhantomData,
    }
}

// Operates by filming its `buf` and `last`, and letting the inner consumer consume them via the `operations` when they become full.
impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Consumer for Scramble<I, R, L, S> {
    type Repeated = R;
    type Last = L;
    type Stopped = S;

    fn consume(&mut self, item: Self::Repeated) -> Option<Self::Stopped> {
        match self.buf.enqueue(item) {
            None => return None,
            Some(item) => {
                while self.buf.get_amount() > 0 {
                    if let Some(s) = self.perform_operation() {
                        return Some(s);
                    }
                }

                return None;
            }
        }
    }

    fn flush(&mut self) -> Option<Self::Stopped> {
        while self.buf.get_amount() > 0 {
            if let Some(s) = self.perform_operation() {
                return Some(s);
            }
        }

        return self.inner.flush();
    }

    fn close(&mut self, last: Self::Last) -> Option<Self::Stopped> {
        while self.buf.get_amount() > 0 {
            if let Some(s) = self.perform_operation() {
                return Some(s);
            }
        }

        return self.inner.close(last);
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> BulkConsumer for Scramble<I, R, L, S> {
    fn consumer_slots(&mut self) -> SequenceState<&mut Slice1<MaybeUninit<Self::Repeated>>, Self::Stopped> {
        let current = self.buf.get_amount();
        let capacity = self.buf.get_capacity();

        if current < capacity.into() {
            return More(self.buf.enqueue_slots().unwrap());
        } else {
            while self.buf.get_amount() > 0 {
                if let Some(s) = self.perform_operation() {
                    return Final(s);
                }
            }

            return More(self.buf.enqueue_slots().unwrap());
        }
    }

    unsafe fn did_consume(&mut self, amount: NonZeroUsize) {
        self.buf.did_enqueue(amount)
    }
}

impl<I: BulkConsumer<Repeated = R, Last = L, Stopped = S>, R: Copy, L, S> Scramble<I, R, L, S> {
    fn perform_operation(&mut self) -> Option<S> {
        debug_assert!(self.buf.get_amount() > 0);

        match self.operations[self.operations_index] {
            ConsumeOperation::Consume => {
                let item = self.buf.dequeue().unwrap();
                self.operations_index = (self.operations_index + 1) % self.operations.len();
                return self.inner.consume(item);
            }
            ConsumeOperation::ConsumerSlots(n) => {
                match self.inner.consumer_slots() {
                    Final(s) => return Some(s),
                    More(slots) => {
                        let l = slots.len_();
                        let slots = unsafe { Slice1::from_slice_unchecked_mut(&mut slots[..min(l, n.get())]) };
                        let produce_amount = self.buf.bulk_dequeue(slots);
                        unsafe {self.inner.did_consume(NonZeroUsize::new(produce_amount).expect("Erronous scrambler implementation."));}
                        self.operations_index = (self.operations_index + 1) % self.operations.len();
                        return None;
                    },
                }
            }
            ConsumeOperation::Flush => {
                if let Some(s) = self.inner.flush() {
                    return Some(s);
                } else {
                    self.operations_index = (self.operations_index + 1) % self.operations.len();
                        return None;
                }
            }
        }
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
