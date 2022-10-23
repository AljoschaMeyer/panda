#![feature(maybe_uninit_write_slice)]
#![feature(maybe_uninit_slice)]
#![feature(maybe_uninit_uninit_array)]
#![feature(never_type)]

#![feature(allocator_api)]
#![feature(new_uninit)]

use core::mem::MaybeUninit;

pub mod sync;

pub(crate) fn maybe_uninit_slice<'a, T>(s: &'a [T]) -> &'a [MaybeUninit<T>] {
    let ptr = s.as_ptr().cast::<MaybeUninit<T>>();
    unsafe { core::slice::from_raw_parts(ptr, s.len()) }
}

pub(crate) fn maybe_uninit_slice_mut<'a, T>(s: &'a mut [T]) -> &'a mut [MaybeUninit<T>] {
    let ptr = s.as_mut_ptr().cast::<MaybeUninit<T>>();
    unsafe { core::slice::from_raw_parts_mut(ptr, s.len()) }
}
