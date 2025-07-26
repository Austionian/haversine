use clap::Parser;
use haversine::haversine;
use parser::{parse, Args};
use std::fs;

fn main() {
    let args = Args::parse();

    let input = String::from_utf8(fs::read(&args.json_path).expect("Unable to json read file"))
        .expect("Invalid utf-8 string");

    let parsed_json = parse(&input);

    let sum = parsed_json
        .pairs
        .iter()
        .map(|pair| haversine(pair.x0, pair.y0, pair.x1, pair.y1, 6372.8))
        .sum::<f64>()
        / parsed_json.pairs.len() as f64;

    let answer = fs::read(&args.answer_path)
        .expect("Unable to read answer file")
        .chunks(8)
        .skip(parsed_json.pairs.len())
        .map(|answer| {
            f64::from_le_bytes([
                answer[0], answer[1], answer[2], answer[3], answer[4], answer[5], answer[6],
                answer[7],
            ])
        })
        .collect::<Vec<f64>>()[0];

    println!(
        "RESULTS
input size: {}
Pair count: {}
Haversine sum: {sum}
",
        input.len(),
        parsed_json.pairs.len()
    );

    println!(
        "VALIDATION
Reference sum: {answer}
Difference: {}
",
        sum - answer
    );
}
