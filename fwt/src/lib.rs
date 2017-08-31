// Benchmark translated and adapted from AMD APP SDK. Original license:
/**********************************************************************
Copyright ©2015 Advanced Micro Devices, Inc. All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

•   Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
•   Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or
 other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY
 DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS
 OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
 NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
********************************************************************/

#![no_std]
#![allow(dead_code, unused_imports)]
extern crate nyuzi_support;
use core::cell::Cell;
use core::mem::transmute;
use nyuzi_support::spmd_range;

/* tid = get_global_id(0) */
fn fwt_kernel(xs: &[Cell<f32>], step: usize, tid: usize) {
    let group = tid % step;
    let pair = 2 * step * (tid / step) + group;

    let partner = pair + step;

    unsafe {
        let t1 = xs.get_unchecked(pair).get();
        let t2 = xs.get_unchecked(partner).get();

        xs.get_unchecked(pair).set(t1 + t2);
        xs.get_unchecked(partner).set(t1 - t2);
    }
}

/* tid = get_global_id(0) */
fn fwt_nodivmod_kernel(xs: &[Cell<f32>], step: usize, step_log2: usize, tid: usize) {
    let group = tid & (step - 1);
    let pair = 2 * step * (tid >> step_log2) + group;

    let partner = pair + step;

    unsafe {
        let t1 = xs.get_unchecked(pair).get();
        let t2 = xs.get_unchecked(partner).get();

        xs.get_unchecked(pair).set(t1 + t2);
        xs.get_unchecked(partner).set(t1 - t2);
    }
}


include!(concat!(env!("OUT_DIR"), "/input.rs"));

static mut INPUT: [f32; LENGTH] = [0.0; LENGTH];

unsafe fn get_data() -> &'static [Cell<f32>] {
    INPUT.copy_from_slice(&INPUT_INIT);
    &*(&mut INPUT as *mut [f32] as *const [Cell<f32>])
}

#[no_mangle]
#[cfg(all(benchmark="fwt", variant="scalar"))]
pub extern fn fwt_scalar() {
    let xs = unsafe { get_data() };
    let mut step = 1;
    while step < xs.len() {
        for tid in 0..(xs.len() / 2) {
            fwt_kernel(xs, step, tid);
        }
        step <<= 1;
    }
}

#[no_mangle]
#[cfg(all(benchmark="fwt", variant="spmd"))]
pub extern fn fwt_spmd() {
    let xs = unsafe { get_data() };
    let mut step = 1;
    while step < xs.len() {
        spmd_range(0..xs.len() / 2, |tid: usize| {
            fwt_kernel(xs, step, tid);
        });
        step <<= 1;
    }
}

#[no_mangle]
#[cfg(all(benchmark="fwt_nodivmod", variant="scalar"))]
pub extern fn fwt_nodivmod_scalar() {
    let xs = unsafe { get_data() };
    let mut step = 1;
    let mut step_log2 = 1;
    while step < xs.len() {
        for tid in 0..(xs.len() / 2) {
            fwt_nodivmod_kernel(xs, step, step_log2, tid);
        }
        step <<= 1;
        step_log2 += 1;
    }
}

#[no_mangle]
#[cfg(all(benchmark="fwt_nodivmod", variant="spmd"))]
pub extern fn fwt_nodivmod_spmd() {
    let xs = unsafe { get_data() };
    let mut step = 1;
    let mut step_log2 = 0;
    while step < xs.len() {
        spmd_range(0..xs.len() / 2, |tid: usize| {
            fwt_nodivmod_kernel(xs, step, step_log2, tid);
        });
        step <<= 1;
        step_log2 += 1;
    }
}
