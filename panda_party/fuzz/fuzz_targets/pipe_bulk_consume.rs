#![no_main]
use libfuzzer_sys::fuzz_target;

use core::cmp::min;

use panda_pile::sync::*;
use panda_party::sync::*;

fuzz_target!(|data: (Box<[u8]>, Box<[u8]>)| {
    let (a, mut b) = data;
    let mut o = pro::cursor(&a[..]);
    let mut i = con::cursor(&mut b[..]);

    pipe_bulk_consume(&mut o, &mut i);

    let m = min(o.as_ref().len(), i.as_ref().len());
    assert_eq!(&i.as_ref()[..m], &o.as_ref()[..m]);
});