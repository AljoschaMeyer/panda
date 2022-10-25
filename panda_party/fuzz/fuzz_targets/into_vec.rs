#![no_main]
use libfuzzer_sys::fuzz_target;

use core::cmp::min;
use core::num::NonZeroUsize;

use wrapper::Wrapper;

use panda_pile::sync::*;
use panda_party::sync::{*, con::ConsumeOperations};

fuzz_target!(|data: (Box<[u8]>, ConsumeOperations, NonZeroUsize)| {
    let (a, ops, cap) = data;
    let cap = NonZeroUsize::new(min(cap.get(), 2048)).unwrap();
    let mut o = pro::cursor(&a[..]);
    let mut i = con::scramble(con::into_vec(), ops, cap);

    pipe_bulk_produce(&mut o, &mut i);
    let _ = i.flush();

    let vector = i.into_inner().into_inner();
    assert_eq!(&vector[..], &o.as_ref()[..]);
});