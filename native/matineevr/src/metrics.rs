use std::{fs, path::PathBuf, sync::mpsc, thread, time::Duration};

#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub cpu: Option<f64>,
    pub gpu: Option<f64>,
    pub memory: Option<(u64, u64)>,
}

pub struct Metrics {
    samples: mpsc::Receiver<Sample>,
    stop: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
    pub latest: Sample,
}

impl Metrics {
    pub fn new() -> Self {
        let (sender, samples) = mpsc::sync_channel(1);
        let (stop, stopped) = mpsc::channel();
        let worker = thread::spawn(move || {
            let gpu = gpu_counter();
            let mut previous = None;
            loop {
                let current = fs::read_to_string("/proc/stat")
                    .ok()
                    .and_then(|s| cpu_times(&s));
                let cpu = previous.zip(current).and_then(|(a, b)| cpu_percent(a, b));
                previous = current;
                let sample = Sample {
                    cpu,
                    gpu: gpu.as_ref().and_then(|path| {
                        let value: f64 = fs::read_to_string(path).ok()?.trim().parse().ok()?;
                        (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(value)
                    }),
                    memory: fs::read_to_string("/proc/meminfo")
                        .ok()
                        .and_then(|s| memory(&s)),
                };
                if matches!(
                    sender.try_send(sample),
                    Err(mpsc::TrySendError::Disconnected(_))
                ) {
                    break;
                }
                if !matches!(
                    stopped.recv_timeout(Duration::from_millis(500)),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    break;
                }
            }
        });
        Self {
            samples,
            stop,
            worker: Some(worker),
            latest: Sample::default(),
        }
    }

    pub fn refresh(&mut self) {
        if let Some(sample) = self.samples.try_iter().last() {
            self.latest = sample;
        }
    }
}

impl Drop for Metrics {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn gpu_counter() -> Option<PathBuf> {
    fs::read_dir("/sys/kernel/debug/dri")
        .ok()?
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().parse::<u32>().is_ok())
        .map(|entry| entry.path().join("perf_now"))
        .find(|path| path.is_file())
}

fn cpu_times(text: &str) -> Option<(u64, u64)> {
    let mut fields = text.lines().next()?.split_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }
    let values: Vec<u64> = fields
        .take(8)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    if values.len() != 8 {
        return None;
    }
    Some((values.iter().sum(), values[3] + values[4]))
}

fn cpu_percent(before: (u64, u64), after: (u64, u64)) -> Option<f64> {
    let total = after.0.checked_sub(before.0)?;
    let idle = after.1.checked_sub(before.1)?;
    let busy = total.checked_sub(idle)?;
    (total > 0).then_some(busy as f64 * 100.0 / total as f64)
}

fn memory(text: &str) -> Option<(u64, u64)> {
    let field = |name| -> Option<u64> {
        text.lines()
            .find_map(|line| line.strip_prefix(name))?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    };
    let total = field("MemTotal:")?;
    let available = field("MemAvailable:")?;
    Some((total.checked_sub(available)? * 1024, total * 1024))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_uses_all_cores_without_counting_guest_twice() {
        let a = cpu_times("cpu 100 10 20 500 30 5 5 0 50 1\ncpu0 0").unwrap();
        let b = cpu_times("cpu 140 10 20 540 50 5 5 0 80 1\n").unwrap();
        assert_eq!(cpu_percent(a, b), Some(40.0));
        assert_eq!(cpu_percent(a, a), None);
        assert_eq!(cpu_percent(b, a), None);
    }

    #[test]
    fn ram_counts_available_cache_as_available() {
        assert_eq!(
            memory("MemTotal: 1000 kB\nMemFree: 100 kB\nMemAvailable: 600 kB\n"),
            Some((400 * 1024, 1000 * 1024))
        );
        assert_eq!(memory("MemTotal: 1000 kB\n"), None);
    }
}
