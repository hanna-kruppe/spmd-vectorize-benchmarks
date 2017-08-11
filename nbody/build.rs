extern crate cgmath;
extern crate rand;
use cgmath::{Point3, Vector3};
use rand::{Rand, Rng, SeedableRng, XorShiftRng};
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

const INITIAL_VELOCITY: f32 = 8.0; // set to 0.0 to turn off.

pub struct Body {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub velocity2: Vector3<f32>,
}

fn create_data<R: Rng>(num_bodies: usize, rng: &mut R) -> Vec<Body> {
    (0..num_bodies)
        .map(|_| {
            let position = Point3 {
                x: f32::rand(rng).floor() * 40_000.0,
                y: f32::rand(rng).floor() * 20_000.0,
                z: (f32::rand(rng).floor() - 0.25) * 50_000.0,
            };

            let velocity = Vector3 {
                x: (f32::rand(rng) - 0.5) * INITIAL_VELOCITY,
                y: (f32::rand(rng) - 0.5) * INITIAL_VELOCITY,
                z: f32::rand(rng) * INITIAL_VELOCITY + 10.0,
            };

            let velocity2 = Vector3 {
                x: (f32::rand(rng) - 0.5) * INITIAL_VELOCITY,
                y: (f32::rand(rng) - 0.5) * INITIAL_VELOCITY,
                z: f32::rand(rng) * INITIAL_VELOCITY,
            };

            Body {
                position: position,
                velocity: velocity,
                velocity2: velocity2,
            }
        })
        .collect()
}

fn write_bodies<W: Write>(w: &mut W, bodies: &[Body]) -> Result<(), io::Error> {
    writeln!(w, "[")?;
    for &Body {
             position: p,
             velocity: v,
             velocity2: v2,
         } in bodies {
        writeln!(w, "    Body {{")?;
        writeln!(w,
                 "         position: Vector3 {{ x: {:?}f32, y: {:?}f32, z: {:?}f32 }},",
                 p.x,
                 p.y,
                 p.z)?;
        writeln!(w,
                 "         velocity: Vector3 {{ x: {:?}f32, y: {:?}f32, z: {:?}f32 }},",
                 v.x,
                 v.y,
                 v.z)?;
        writeln!(w,
                 "         velocity2: Vector3 {{ x: {:?}f32, y: {:?}f32, z: {:?}f32 }},",
                 v2.x,
                 v2.y,
                 v2.z)?;
        writeln!(w, "    }},")?;
    }
    writeln!(w, "];")?;
    Ok(())
}


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bodies.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut rng = XorShiftRng::from_seed([0, 1, 2, 3]);
    let bodies = create_data(16, &mut rng);
    write!(&mut f, "static BODIES_INIT: [Body; BENCH_BODIES] = ").unwrap();
    write_bodies(&mut f, &bodies).unwrap();
}
