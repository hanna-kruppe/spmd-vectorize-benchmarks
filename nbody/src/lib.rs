// Adapted from a rayon demo (commit b172eeedb44eab6fcd3200a33228e364f8ae6f83)
// see https://github.com/nikomatsakis/rayon
// Original license reproduced below:
//
// Rust source (c) 2016 by the Rayon developers. This is ported from
// [JavaScript sources][1] developed by Intel as part of the Parallel
// JS project. The copyright for the original source is reproduced
// below.
//
// Copyright (c) 2011, Intel Corporation
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// - Redistributions of source code must retain the above copyright notice,
//   this list of conditions and the following disclaimer.
// - Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
// THE POSSIBILITY OF SUCH DAMAGE.
//
// [1]: https://github.com/IntelLabs/RiverTrail/blob/master/examples/nbody-webgl/NBody.js
#![no_std]
#![allow(dead_code)]
extern crate nyuzi_support;

use nyuzi_support::{black_box, spmd_zip2, sin, cos, sqrt};
use core::f32::consts::PI;

pub struct NBodyBenchmark<'a> {
    time: usize,
    bodies: (&'a mut [Body], &'a mut [Body]),
}

#[derive(Copy, Clone)]
pub struct Body {
    pub position: Vector3,
    pub velocity: Vector3,
    pub velocity2: Vector3,
}

impl<'a> NBodyBenchmark<'a> {
    fn tick_seq(&mut self) {
        let (in_bodies, out_bodies) = if (self.time & 1) == 0 {
            (&*self.bodies.0, &mut *self.bodies.1)
        } else {
            (&*self.bodies.1, &mut *self.bodies.0)
        };

        let time = self.time;
        for (out, prev) in out_bodies.iter_mut().zip(in_bodies.iter()) {
            let (vel, vel2) = next_velocity(time, prev, in_bodies);
            out.velocity = vel;
            out.velocity2 = vel2;

            let next_velocity = vel - vel2;
            out.position = prev.position + next_velocity;
        }

        self.time += 1;
    }

    fn tick_par(&mut self) {
        let (in_bodies, out_bodies) = if (self.time & 1) == 0 {
            (&*self.bodies.0, &mut *self.bodies.1)
        } else {
            (&*self.bodies.1, &mut *self.bodies.0)
        };

        let time = self.time;
        spmd_zip2(out_bodies, in_bodies, |out, prev| {
            let (vel, vel2) = next_velocity(time, prev, in_bodies);
            out.velocity = vel;
            out.velocity2 = vel2;

            let next_velocity = vel - vel2;
            out.position = prev.position + next_velocity;
        });

        self.time += 1;
    }
}

fn next_velocity(time: usize, prev: &Body, bodies: &[Body]) -> (Vector3, Vector3) {
    let time = time as f32;
    let center = Vector3 {
        x: cos(time / 22.0) * -4200.0,
        y: sin(time / 14.0) * 9200.0,
        z: sin(time / 27.0) * 6000.0,
    };

    // pull to center
    let max_distance = 3400.0;
    let pull_strength = 0.042;

    // zones
    let zone = 400.0;
    let repel = 100.0;
    let align = 300.0;
    let attract = 100.0;

    let (speed_limit, attract_power);
    if time < 500.0 {
        speed_limit = 2000.0;
        attract_power = 100.9;
    } else {
        speed_limit = 0.2;
        attract_power = 20.9;
    }

    let zone_sqrd = 3.0 * (zone * zone);

    let mut acc = Vector3::zero();
    let mut acc2 = Vector3::zero();

    let dir_to_center = center - prev.position;
    let dist_to_center = dir_to_center.magnitude();

    // orient to center
    if dist_to_center > max_distance {
        let velc = if time < 200.0 {
            0.2
        } else {
            (dist_to_center - max_distance) * pull_strength
        };

        let diff = (dir_to_center / dist_to_center) * velc;
        acc += diff;
    }

    let zero: Vector3 = Vector3::zero();
    let (diff, diff2) = bodies
        .iter()
        .fold((zero, zero), |(mut diff, mut diff2), body| {
            let r = body.position - prev.position;

            // make sure we are not testing the particle against its own position
            let are_same = r == Vector3::zero();

            let dist_sqrd = r.magnitude2();

            if dist_sqrd < zone_sqrd && !are_same {
                let length = sqrt(dist_sqrd);
                let percent = dist_sqrd / zone_sqrd;

                if dist_sqrd < repel {
                    let f = (repel / percent - 1.0) * 0.025;
                    let normal = (r / length) * f;
                    diff += normal;
                    diff2 += normal;
                } else if dist_sqrd < align {
                    let thresh_delta = align - repel;
                    let adjusted_percent = (percent - repel) / thresh_delta;
                    let q = (0.5 - cos(adjusted_percent * PI * 2.0) * 0.5 + 0.5) * 100.9;

                    // normalize vel2 and multiply by factor
                    let vel2_length = body.velocity2.magnitude();
                    let vel2 = (body.velocity2 / vel2_length) * q;

                    // normalize own velocity
                    let vel_length = prev.velocity.magnitude();
                    let vel = (prev.velocity / vel_length) * q;

                    diff += vel2;
                    diff2 += vel;
                }

                if dist_sqrd > attract {
                    // attract
                    let thresh_delta2 = 1.0 - attract;
                    let adjusted_percent2 = (percent - attract) / thresh_delta2;
                    let c = (1.0 - (cos(adjusted_percent2 * PI * 2.0) * 0.5 + 0.5)) * attract_power;

                    // normalize the distance vector
                    let d = (r / length) * c;

                    diff += d;
                    diff2 -= d;
                }
            }
            (diff, diff2)
        });

    acc += diff;
    acc2 += diff2;

    // Speed limits
    if time > 500.0 {
        let acc_squared = acc.magnitude2();
        if acc_squared > speed_limit {
            acc *= 0.015;
        }

        let acc_squared2 = acc2.magnitude2();
        if acc_squared2 > speed_limit {
            acc2 *= 0.015;
        }
    }

    let mut new = prev.velocity + acc;
    let mut new2 = prev.velocity2 + acc2;

    if time < 500.0 {
        let acs = new2.magnitude2();
        if acs > speed_limit {
            new2 *= 0.15;
        }

        let acs2 = new.magnitude2();
        if acs2 > speed_limit {
            new *= 0.15;
        }
    }

    (new, new2)
}

// Custom implementation (no code shared) of cgmath interfaces,
// to make the above code work with less modification
use core::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div};

#[derive(Copy, Clone, PartialEq)]
pub struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vector3 {
    fn zero() -> Self {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn magnitude2(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    fn magnitude(&self) -> f32 {
        sqrt(self.magnitude2())
    }
}

impl Add for Vector3 {
    type Output = Vector3;

    fn add(mut self, rhs: Vector3) -> Vector3 {
        self += rhs;
        self
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vector3 {
    type Output = Vector3;

    fn sub(mut self, rhs: Vector3) -> Vector3 {
        self -= rhs;
        self
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    fn mul(mut self, rhs: f32) -> Vector3 {
        self *= rhs;
        self
    }
}
impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    fn div(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

// Bench data

const BENCH_BODIES: usize = 32;

const BENCH_TICKS: usize = 10;

include!(concat!(env!("OUT_DIR"), "/bodies.rs"));

// Bench interface

const DUMMY_BODY: Body = Body {
    position: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    velocity: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    velocity2: Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
};
static mut BODIES1: [Body; BENCH_BODIES] = [DUMMY_BODY; BENCH_BODIES];
static mut BODIES2: [Body; BENCH_BODIES] = [DUMMY_BODY; BENCH_BODIES];

unsafe fn get_data() -> NBodyBenchmark<'static> {
    BODIES1.copy_from_slice(&BODIES_INIT);
    BODIES2.copy_from_slice(&BODIES_INIT);
    black_box(NBodyBenchmark {
                  time: 0,
                  bodies: (&mut BODIES1, &mut BODIES2),
              })
}

#[no_mangle]
#[cfg(variant="scalar")]
pub extern "C" fn nbody_scalar() {
    let mut nbody = unsafe { get_data() };

    black_box(&mut nbody);
    for _ in 0..BENCH_TICKS {
        nbody.tick_seq();
    }
    black_box(&mut nbody);
}

#[no_mangle]
#[cfg(variant="spmd")]
pub extern "C" fn nbody_spmd() {
    let mut nbody = unsafe { get_data() };

    black_box(&mut nbody);
    for _ in 0..BENCH_TICKS {
        nbody.tick_par();
    }
    black_box(&mut nbody);
}
