use anyhow::{Result, ensure};
#[repr(C)]
struct Timespec {
    seconds: i64,
    nanoseconds: i64,
}

unsafe extern "C" {
    fn clock_gettime(clock: i32, time: *mut Timespec) -> i32;
}

pub fn cpu_seconds() -> Result<f64> {
    let mut time = Timespec {
        seconds: 0,
        nanoseconds: 0,
    };
    ensure!(
        unsafe { clock_gettime(2, &mut time) } == 0,
        "Read process CPU clock: {}",
        std::io::Error::last_os_error()
    );
    Ok(time.seconds as f64 + time.nanoseconds as f64 * 1e-9)
}
