#![no_main]
use libfuzzer_sys::fuzz_target;

use core::cmp::min;
use core::num::NonZeroUsize;

use wrapper::Wrapper;

use panda_pile::sync::*;
use panda_party::sync::{*, pro::ProduceOperations};

fuzz_target!(|data: (Box<[u8]>, Box<[u8]>, ProduceOperations, ProduceOperations, NonZeroUsize, NonZeroUsize)| {
    let (a, mut b, ops_a, ops_b, cap_a, cap_b) = data;
    if b.len() < a.len() {
        return;
    }
    let cap_a = NonZeroUsize::new(min(cap_a.get(), 4)).unwrap();
    let cap_b = NonZeroUsize::new(min(cap_b.get(), 4)).unwrap();
    let mut o = pro::scramble(
        pro::scramble(
            pro::cursor(&a[..]),
            ops_b, cap_b),
        ops_a, cap_a
    );
    let mut i = con::cursor(&mut b[..]);

    pipe_bulk_produce(&mut o, &mut i);
    let _ = i.flush();

    let o = o.into_inner().into_inner();
    let m = min(o.as_ref().len(), i.as_ref().len());
    assert_eq!(&i.as_ref()[..m], &o.as_ref()[..m]);
});
