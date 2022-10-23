#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use std::collections::VecDeque;
use core::num::NonZeroUsize;

use slice_n::Slice1;

use panda_queues::Queue;
use panda_queues::fixed::Fixed;

#[derive(Debug, Arbitrary)]
enum Operation<T> {
    En(T),
    De,
    BulkEn(Vec<T>),
    BulkDe(u8),
}
use Operation::*;

fuzz_target!(|data: Vec<Operation<u8>>| {
    let cap = 32;
    let mut control = VecDeque::new();
    let mut test = Fixed::new(NonZeroUsize::new(32).unwrap());

    for op in data {
        match op {
            En(t) => {
                let control_result = if control.len() >= cap {
                    Some(t.clone())
                } else {
                    control.push_back(t.clone());
                    None
                };
                let test_result = test.enqueue(t.clone());
                assert_eq!(test_result, control_result);
            }
            De => {
                let control_result = control.pop_front();
                let test_result = test.dequeue();
                assert_eq!(test_result, control_result);
            }
            BulkEn(ts) => {
                if let Some(ts) = Slice1::from_slice(&ts) {
                    let test_result = test.bulk_enqueue(ts);
                    for (i, t) in ts.iter().enumerate() {
                        if i >= test_result {
                            break;
                        } else {
                            control.push_back(t.clone());
                        }
                    }
                }                
            }
            BulkDe(n) => {
                let n = n as usize;
                if n > 0 {
                    let mut test_buffer = vec![];
                    test_buffer.resize(n, 0);
                    let test_result = test.bulk_dequeue(Slice1::from_slice_mut(&mut test_buffer).unwrap().as_maybe_uninit_mut());
                    let mut control_buffer = vec![];
                    for _ in 0..test_result {
                        if let Some(t) = control.pop_front() {
                            control_buffer.push(t.clone());
                        }
                    }
                    assert_eq!(&test_buffer[..test_result], &control_buffer[..test_result]);
                }
            }
        }
    }
});
