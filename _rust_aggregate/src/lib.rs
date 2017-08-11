#![feature(core_intrinsics, lang_items)]
#![no_std]

extern crate fib;
extern crate nbody;
extern crate fwt;

use core::intrinsics;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(_msg: core::fmt::Arguments,
                                   _file: &'static str,
                                   _line: u32)
                                   -> ! {
    unsafe { intrinsics::abort() }
}

// Not actually needed on nyuzi b/c panic=abort, but makes my editor plugin happy
#[cfg(not(target_arch="nyuzi"))]
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
