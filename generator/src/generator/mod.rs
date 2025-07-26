use crate::Args;
use json::{Json, Pair};
use rand::Rng;

fn create_x() -> f64 {
    rand::rng().random_range(-180.0..180.0)
}

fn create_y() -> f64 {
    rand::rng().random_range(-90.0..90.0)
}

pub struct Generator();

impl Generator {
    pub fn generate(args: &Args) -> Json {
        let mut pairs = Vec::new();

        (0..args.number).for_each(|_| {
            pairs.push(Pair {
                x0: create_x(),
                y0: create_y(),
                x1: create_x(),
                y1: create_y(),
            })
        });

        Json { pairs }
    }
}
