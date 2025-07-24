use crate::Args;
use serde::Serialize;

#[derive(Serialize)]
pub struct Output {
    pub pairs: Vec<Pair>,
}

#[derive(Serialize)]
pub struct Pair {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

const X: [f64; 2] = [-180.0, 180.0];
const Y: [f64; 2] = [-90.0, 90.0];

fn create_value(bound: [f64; 2], seed: u64) -> f64 {
    todo!()
}

fn create_x(seed: u64) -> f64 {
    create_value(X, seed)
}

fn create_y(seed: u64) -> f64 {
    create_value(Y, seed)
}

pub struct Generator();

impl Generator {
    pub fn generate(args: Args) -> Output {
        let mut pairs = Vec::new();

        (0..args.number).for_each(|_| {
            pairs.push(Pair {
                x0: create_x(args.seed),
                y0: create_y(args.seed),
                x1: create_x(args.seed),
                y1: create_y(args.seed),
            })
        });

        Output { pairs }
    }
}
