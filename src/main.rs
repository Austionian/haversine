use clap::Parser;
use haversine_generator::{Args, Generator};

fn main() {
    match serde_json::to_string(&Generator::generate(Args::parse())) {
        Ok(json) => println!("{json}"),
        Err(e) => eprint!("{e}"),
    }
}
