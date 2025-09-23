#[cfg(target_os = "linux")]
pub fn get_os_time_freq() -> u64 {
    1_000_000
}

#[cfg(target_os = "windows")]
pub fn get_os_time_freq() -> u64 {
    let mut freq = 0i64;
    unsafe { windows_sys::Win32::System::Performance::QueryPerformanceFrequency(&mut freq) }
    freq.QuadPart
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "macos")]
pub fn read_os_timer() -> u64 {
    todo!()
}

#[cfg(target_os = "windows")]
pub fn read_os_timer() -> u64 {
    let mut value = 0i64;
    unsafe { windows_sys::Win32::System::Performance::QueryPerformanceCounter(&mut value) }
    value.QuadPart
}

#[cfg(target_arch = "x86_64")]
pub fn read_cpu_timer() -> u64 {
    unsafe { std::arch::x86_64::_rdtsc() }
}

#[cfg(target_arch = "arm")]
pub fn read_cpu_timer() -> u64 {
    unsafe { _rdtsc() }
}

pub fn estimate_cpu_freq() -> u64 {
    let ms_to_wait = 100u64;

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
