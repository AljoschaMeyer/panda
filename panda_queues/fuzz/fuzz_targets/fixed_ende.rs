#![no_main]
use libfuzzer_sys::fuzz_target;
use arbitrary::Arbitrary;

use std::collections::VecDeque;
use core::num::NonZeroUsize;

use panda_queues::Queue;
use panda_queues::fixed::Fixed;

#[derive(Debug, Arbitrary)]
enum Operation<T> {
    En(T),
    De,
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
        }
    }
});
