use crate::Args;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize)]
pub struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

fn create_x() -> f64 {
    rand::rng().random_range(-180.0..180.0)
}

fn create_y() -> f64 {
    rand::rng().random_range(-90.0..90.0)
}

pub struct Generator();

impl Generator {
    pub fn generate(args: &Args) -> Output {
        let mut pairs = Vec::new();

        (0..args.number).for_each(|_| {
            pairs.push(Pair {
                x0: create_x(),
                y0: create_y(),
                x1: create_x(),
                y1: create_y(),
            })
        });

        Output { pairs }
    }
}
