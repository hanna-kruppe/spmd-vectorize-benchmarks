#![feature(core_intrinsics, lang_items, asm, linkage)]
#![no_std]

#[allow(unused_imports)]
use core::intrinsics;
use core::ops::Range;

// Taken from libtest
pub fn black_box<T>(dummy: T) -> T {
    // we need to "use" the argument in some way LLVM can't
    // introspect.
    unsafe { asm!("" : : "r"(&dummy)) }
    dummy
}

pub fn run_scalar<T, F>(scratch: &mut [T], f: F)
    where F: Sync + Fn(&mut T)
{
    black_box(&mut *scratch);
    for x in &mut *scratch {
        f(x);
    }
    black_box(&mut *scratch);
}

pub fn run_vector<T, F>(scratch: &mut [T], f: F)
    where F: Sync + Fn(&mut T)
{
    black_box(&mut *scratch);
    spmd(scratch, f);
    black_box(&mut *scratch);
}

#[cfg(target_arch="nyuzi")]
fn spmd<T, F>(x: &mut [T], kernel: F)
    where F: Sync + Fn(&mut T)
{
    struct KernelData<'a, T: 'a, F>(&'a mut [T], F);

    fn kernel_wrapper<T, F>(kernel_data: *mut u8)
        where F: Sync + Fn(&mut T)
    {
        unsafe {
            let kernel_data = &mut *(kernel_data as *mut KernelData<T, F>);
            let elems: &mut [T] = &mut kernel_data.0;
            let id = intrinsics::spmd_lane_id();
            (kernel_data.1)(&mut elems[id]);
        }
    }

    assert_eq!(x.len(), 16); // TODO

    let mut kernel_data = KernelData(x, kernel);
    unsafe {
        intrinsics::spmd_call(kernel_wrapper::<T, F>,
                              &mut kernel_data as *mut _ as *mut u8);
    }
}

#[cfg(target_arch="nyuzi")]
pub fn spmd_range<F>(x: Range<usize>, kernel: F)
    where F: Fn(usize)
{
    struct KernelData<F>(usize, F);

    fn kernel_wrapper<F>(kernel_data: *mut u8)
        where F: Fn(usize)
    {
        unsafe {
            let kernel_data = &*(kernel_data as *mut KernelData<F>);
            let id = kernel_data.0 + intrinsics::spmd_lane_id();
            (kernel_data.1)(id);
        }
    }

    assert_eq!(x.len() % 16, 0);

    let mut kernel_data = KernelData(0, kernel);
    while kernel_data.0 < x.end {
        unsafe {
            intrinsics::spmd_call(kernel_wrapper::<F>, &mut kernel_data as *mut _ as *mut u8);
        }
        kernel_data.0 += 16;
    }
}

#[cfg(not(target_arch="nyuzi"))]
pub fn spmd_range<F>(_: Range<usize>, _: F) {}

#[cfg(target_arch="nyuzi")]
pub fn spmd_zip2<T, F>(outs: &mut [T], ins: &[T], kernel: F)
    where F: Sync + Fn(&mut T, &T)
{

    struct KernelData<'a, 'b, T: 'a + 'b, F>(usize, &'a mut [T], &'b [T], F);

    fn kernel_wrapper<T, F>(kernel_data: *mut u8)
        where F: Sync + Fn(&mut T, &T)
    {
        unsafe {
            let kernel_data = &mut *(kernel_data as *mut KernelData<T, F>);
            let outs: &mut [T] = &mut kernel_data.1;
            let ins: &[T] = &mut kernel_data.2;
            let id = kernel_data.0 + intrinsics::spmd_lane_id();
            // TODO remove this and improve LowerSPMD's handling of panics
            (kernel_data.3)(&mut outs[id], &ins[id]);
        }
    }

    assert_eq!(ins.len() % 16, 0);
    assert_eq!(ins.len(), outs.len());

    let mut kernel_data = KernelData(0, outs, ins, kernel);
    while kernel_data.0 < ins.len() {
        unsafe {
            intrinsics::spmd_call(kernel_wrapper::<T, F>,
                                  &mut kernel_data as *mut _ as *mut u8);
        }
        kernel_data.0 += 16;
    }
}

#[cfg(not(target_arch="nyuzi"))]
pub fn spmd_zip2<T, F>(outs: &mut [T], ins: &[T], kernel: F)
    where F: Sync + Fn(&mut T, &T)
{
    for (out, inp) in outs.iter_mut().zip(ins.iter()) {
        kernel(out, inp);
    }
}

// Keeps the editor happy
#[cfg(not(target_arch="nyuzi"))]
fn spmd<T, F>(x: &mut [T], kernel: F)
    where F: Sync + Fn(&mut T)
{
    run_scalar(x, kernel);
}

mod math {
    use core::f32::consts::PI;
    /*
    extern {
        fn sinf(x: f32) -> f32;
        fn cosf(x: f32) -> f32;
        fn sqrtf(x: f32) -> f32;
    }

    pub fn sin_wrapper(x: f32) -> f32 {
        unsafe { sinf(x) }
    }

    pub fn cos_wrapper(x: f32) -> f32 {
        unsafe { cosf(x) }
    }

    pub fn sqrt_wrapper(x: f32) -> f32 {
        unsafe { sqrtf(x) }
    }*/

    // TODO evaluate perf with cross-module speculative call to vectorized libc

    #[inline]
    fn fmod(val1: f32, val2: f32) -> f32 {
        let whole = (val1 / val2) as i32;
        return val1 - (whole as f32 * val2);
    }

    const NUM_TERMS: usize = 6;

    const DENOMINATORS: [f32; NUM_TERMS] = [
        -0.166666666666667,  // 1 / 3!
        0.008333333333333,   // 1 / 5!
        -0.000198412698413,  // 1 / 7!
        0.000002755731922,   // 1 / 9!
        -2.50521084e-8,      // 1 / 11!
        1.6059044e-10        // 1 / 13!
    ];

    #[inline]
    #[allow(non_snake_case)]
    pub fn sin(mut angle: f32) -> f32 {
        // The approximation begins to diverge past 0-pi/2. To prevent
        // discontinuities, mirror or flip this function for the remaining
        // parts of the function.
        angle = fmod(angle, PI * 2.);
        let mut resultSign: i32;
        if angle < 0. {
            resultSign = -1;
        } else {
            resultSign = 1;
        }

        if angle < 0.0 {
            angle = -angle;
        };
        if angle > PI * 3. / 2. {
            angle = PI * 2. - angle;
            resultSign = -resultSign;
        } else if angle > PI {
            angle -= PI;
            resultSign = -resultSign;
        } else if angle > PI / 2. {
            angle = PI - angle;
        }

        let angleSquared = angle * angle;
        let mut numerator = angle;
        let mut result = angle;

        for denom in &DENOMINATORS {
            numerator *= angleSquared;
            result += numerator * denom;
        }

        return result * resultSign as f32;
    }

    #[inline]
    pub fn cos(angle: f32) -> f32 {
        sin(angle + PI * 0.5)
    }

    #[inline]
    pub fn sqrt(value: f32) -> f32 {
        let mut guess = value;
        for _ in 0..10 {
            guess = ((value / guess) + guess) / 2.0;
        }
        guess
    }
}

pub use math::{sin, cos, sqrt};
/*pub use math::sin_wrapper as sin;
pub use math::cos_wrapper as cos;
pub use math::sqrt_wrapper as sqrt;*/

#[macro_export]
macro_rules! printf {
    ($fmt: expr) => (printf!($fmt, /* no arguments */));
    ($fmt: expr, $($arg: expr),*) => ({
        $crate::printf(concat!($fmt, "\0").as_bytes().as_ptr(), $($arg),*);
    });
}

extern "C" {
    #[allow(dead_code)]
    pub fn printf(_: *const u8, ...) -> i32;
}
