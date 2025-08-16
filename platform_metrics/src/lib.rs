use clap::Parser;
use std::arch::x86_64::_rdtsc;

pub fn get_os_time_freq() -> u64 {
    1_000_000
}

pub fn read_os_timer() -> u64 {
    let mut value = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };

    unsafe {
        libc::gettimeofday(&mut value, std::ptr::null_mut());
    }

    get_os_time_freq() * value.tv_sec as u64 + value.tv_usec as u64
}

pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

#[derive(Parser)]
struct Args {
    #[arg(short)]
    ms_to_wait: Option<u64>,
}

pub fn estimate_cpu_freq() -> u64 {
    let ms_to_wait = Args::parse().ms_to_wait.unwrap_or(1_000);

    let os_freq = get_os_time_freq();

    let cpu_start = read_cpu_timer();
    let os_start = read_os_timer();
    let mut os_end;
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

    cpu_freq
}
