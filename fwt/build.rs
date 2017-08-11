extern crate rand;
use rand::{XorShiftRng, Rng, SeedableRng};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;


const LENGTH: usize = 512;

const SEED: [u32; 4] = [1, 2, 3, 4];

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("input.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut rng = XorShiftRng::from_seed(SEED);
    let input: Vec<f32> = (0..LENGTH).map(|_| rng.gen_range(0.0, 255.0)).collect();
    write!(&mut f, "const LENGTH: usize = {};", LENGTH).unwrap();
    write!(&mut f,
           "static mut INPUT_INIT: [f32; LENGTH] = {:?};",
           input)
            .unwrap();
}
