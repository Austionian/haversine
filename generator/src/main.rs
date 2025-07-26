use clap::Parser;
use generator::{Args, Generator};
use haversine::haversine;
use std::{io::Write, path::PathBuf, str::FromStr};

fn main() {
    let args = Args::parse();
    let haversine_output = Generator::generate(&args);

    // Write the json to a file
    match serde_json::to_string(&haversine_output) {
        Ok(json) => {
            let _ = std::fs::write(
                PathBuf::from_str(&format!("haversine_{}_input.json", args.number)).unwrap(),
                json,
            );
        }
        Err(e) => eprint!("{e}"),
    }

    let mut file = std::fs::File::create(format!("haversine_{}_data.f64", args.number)).unwrap();
    let mean = haversine_output
        .pairs
        .iter()
        .map(|pairs| {
            let distance = haversine(pairs.x0, pairs.y0, pairs.x1, pairs.y1, 6372.8);
            let _ = file.write_all(&distance.to_le_bytes());
            distance
        })
        .sum::<f64>()
        / haversine_output.pairs.len() as f64;

    // write the mean to the .f64 file
    file.write_all(&mean.to_le_bytes()).unwrap();

    println!("Method: {}", args.ty);
    println!("Pair count: {}", args.number);
    println!("Expected mean: {mean}");
}
