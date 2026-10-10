use std::{collections::BTreeMap, fmt::Write, time::Instant};

#[derive(Default)]
struct Samples {
    count: u64,
    total: f64,
    min: f64,
    max: f64,
}

#[derive(Default)]
pub struct Statistics {
    counts: BTreeMap<&'static str, u64>,
    samples: BTreeMap<&'static str, Samples>,
}

impl Statistics {
    pub fn count(&mut self, name: &'static str, count: u64) {
        *self.counts.entry(name).or_default() += count;
    }

    pub fn sample(&mut self, name: &'static str, value: f64) {
        let sample = self.samples.entry(name).or_default();
        if sample.count == 0 {
            sample.min = value;
            sample.max = value;
        }
        sample.count += 1;
        sample.total += value;
        sample.min = sample.min.min(value);
        sample.max = sample.max.max(value);
    }

    pub fn merge(&mut self, other: Self) {
        for (name, count) in other.counts {
            self.count(name, count);
        }
        for (name, source) in other.samples {
            let target = self.samples.entry(name).or_default();
            if target.count == 0 {
                *target = source;
            } else {
                target.count += source.count;
                target.total += source.total;
                target.min = target.min.min(source.min);
                target.max = target.max.max(source.max);
            }
        }
    }

    pub fn report(&mut self, scope: &str, seconds: f64) -> String {
        let mut line = format!("Stats: scope={scope} seconds={seconds:.3}");
        for (name, count) in &self.counts {
            write!(line, " {name}={count}").unwrap();
        }
        for (name, sample) in &self.samples {
            write!(
                line,
                " {name}_n={} {name}_mean={:.3} {name}_min={:.3} {name}_max={:.3}",
                sample.count,
                sample.total / sample.count as f64,
                sample.min,
                sample.max,
            )
            .unwrap();
        }
        self.counts.clear();
        self.samples.clear();
        line
    }
}

pub fn timed<T>(stats: Option<&mut Statistics>, name: &'static str, call: impl FnOnce() -> T) -> T {
    let started = stats.as_ref().map(|_| Instant::now());
    let result = call();
    if let Some(stats) = stats {
        stats.sample(name, started.unwrap().elapsed().as_secs_f64() * 1000.0);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summaries_merge_signed_samples_and_reset_each_window() {
        let mut a = Statistics::default();
        a.sample("lead_ms", -9.0);
        a.count("frames", 2);
        let mut b = Statistics::default();
        b.sample("lead_ms", -3.0);
        b.count("frames", 3);
        a.merge(b);
        let report = a.report("test", 5.0);
        assert!(report.contains("frames=5"));
        assert!(
            report
                .contains("lead_ms_n=2 lead_ms_mean=-6.000 lead_ms_min=-9.000 lead_ms_max=-3.000")
        );
        assert_eq!(a.report("test", 1.0), "Stats: scope=test seconds=1.000");
        let result = timed(None, "unused", || Err::<(), _>("failure"));
        assert_eq!(result, Err("failure"));
        assert_eq!(
            timed(Some(&mut a), "call_ms", || Err::<(), _>("failure")),
            result
        );
        assert!(a.report("test", 1.0).contains("call_ms_n=1"));
    }
}
