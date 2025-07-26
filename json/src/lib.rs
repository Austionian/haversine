use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Json {
    pub pairs: Vec<Pair>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Pair {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}
