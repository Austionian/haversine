use clap::Parser;
use haversine::haversine;
use parser::{Args, parse};
use platform_metrics::{get_os_time_freq, read_cpu_timer, read_os_timer};
use std::fs;

fn main() {
    let time_start = read_os_timer();
    let cpu_start = read_cpu_timer();
    let args = Args::parse();

    let cpu_startup_end = read_cpu_timer();
    let input = String::from_utf8(fs::read(&args.json_path).expect("Unable to json read file"))
        .expect("Invalid utf-8 string");
    let cpu_read_end = read_cpu_timer();

    let parsed_json = parse(&input);
    let cpu_parsing_end = read_cpu_timer();

    let sum = parsed_json
        .pairs
        .iter()
        .map(|pair| haversine(pair.x0, pair.y0, pair.x1, pair.y1, 6372.8))
        .sum::<f64>()
        / parsed_json.pairs.len() as f64;
    let cpu_sum_end = read_cpu_timer();

    let time_end = read_os_timer();
    let cpu_end = read_cpu_timer();

    println!(
        "RESULTS
input size: {}
Pair count: {}
Haversine sum: {sum}
",
        input.len(),
        parsed_json.pairs.len()
    );

    if let Some(answer_path) = &args.answer_path {
        let answer = fs::read(answer_path)
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
            "VALIDATION
Reference sum: {answer}
Difference: {}
",
            sum - answer
        );
    }

    println!("---------------");
    let total_cpu = cpu_end - cpu_start;
    println!(
        "Total time: {:.4}ms (CPU freq {:.0})",
        (time_end - time_start) as f64 / 1_000.0,
        get_os_time_freq() as f64 * (cpu_end - cpu_start) as f64 / (time_end - time_start) as f64
    );
    println!(
        "\tStartup: {} ({:.2}%)",
        cpu_startup_end - cpu_start,
        (cpu_startup_end - cpu_start) as f64 / total_cpu as f64 * 100.0
    );
    println!(
        "\tRead: {} ({:.2}%)",
        cpu_read_end - cpu_startup_end,
        (cpu_read_end - cpu_startup_end) as f64 / total_cpu as f64 * 100.0,
    );
    println!(
        "\tParse: {} ({:.2}%)",
        cpu_parsing_end - cpu_read_end,
        (cpu_parsing_end - cpu_read_end) as f64 / total_cpu as f64 * 100.0,
    );
    println!(
        "\tSum: {} ({:.2}%)",
        cpu_sum_end - cpu_parsing_end,
        (cpu_sum_end - cpu_parsing_end) as f64 / total_cpu as f64 * 100.0,
    );
}
