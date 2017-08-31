// Adapted from a rayon demo (commit b172eeedb44eab6fcd3200a33228e364f8ae6f83)
// see https://github.com/nikomatsakis/rayon

#![no_std]
#![allow(dead_code)]
extern crate nyuzi_support;

fn fib_rec(n: i32) -> i32 {
    if n < 2 {
        1
    } else {
        fib_rec(n - 1) + fib_rec(n - 2)
    }
}

fn fib_iter(n: i32) -> i32 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

const INPUT: [i32; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

#[no_mangle]
#[cfg(all(benchmark="fib_iter", variant="scalar"))]
pub extern "C" fn fib_iter_scalar() {
    nyuzi_support::run_scalar(&mut INPUT, |x| *x = fib_iter(*x));
}

#[no_mangle]
#[cfg(all(benchmark="fib_iter", variant="spmd"))]
pub extern "C" fn fib_iter_spmd() {
    nyuzi_support::run_vector(&mut INPUT, |x| *x = fib_iter(*x));
}

#[no_mangle]
#[cfg(all(benchmark="fib_rec", variant="scalar"))]
pub extern "C" fn fib_rec_scalar() {
    nyuzi_support::run_scalar(&mut INPUT, |x| *x = fib_rec(*x));
}

#[no_mangle]
#[cfg(all(benchmark="fib_rec", variant="spmd"))]
pub extern "C" fn fib_rec_spmd() {
    nyuzi_support::run_vector(&mut INPUT, |x| *x = fib_rec(*x));
}
