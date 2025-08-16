use clap::Parser;
use platform_metrics::{get_os_time_freq, read_cpu_timer, read_os_timer};

#[derive(Parser)]
struct Args {
    #[arg(short)]
    ms_to_wait: Option<u64>,
}

fn main() {
    let ms_to_wait = Args::parse().ms_to_wait.unwrap_or(1_000);

    let os_freq = get_os_time_freq();
    println!("OS Freq: {} (reported)", os_freq);

    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_end = 0;
    let mut os_elasped = 0;
    let os_wait_time = os_freq * ms_to_wait / 1_000;

    while os_elasped < os_wait_time {
        os_end = read_os_timer();
        os_elasped = os_end - os_start;
    }

    let cpu_end = read_cpu_timer();
    let cpu_elapsed = cpu_end - cpu_start;
    let mut cpu_freq = 0;

    if os_elasped != 0 {
        cpu_freq = os_freq * cpu_elapsed / os_elasped;
    }

    println!("OS Timer: {} -> {} = {}", os_start, os_end, os_elasped);
    println!("OS Seconds: {}", os_elasped as f64 / os_freq as f64);
    println!("CPU Timer: {} -> {} = {}", cpu_start, cpu_end, cpu_elapsed);
    println!("CPU Freq: {} (guessed)", cpu_freq);
}
