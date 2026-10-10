use matineevr::{renderer::Renderer, statistics::Statistics};
use std::time::{Duration, Instant};

#[path = "performance_xr.rs"]
mod xr;

pub struct Performance {
    pub xr_fps: f64,
    pub video_fps: f64,
    pub target_fps: f64,
    pub size: (i32, i32),
    period: Instant,
    frames: u64,
    videos: u64,
    report: Instant,
    previous: Option<Instant>,
    pub stats: Option<Statistics>,
}

impl Performance {
    pub fn new(diagnostic: bool) -> Self {
        Self {
            xr_fps: 0.0,
            video_fps: 0.0,
            target_fps: 0.0,
            size: (0, 0),
            period: Instant::now(),
            frames: 0,
            videos: 0,
            report: Instant::now(),
            previous: None,
            stats: diagnostic.then(Statistics::default),
        }
    }

    pub fn record(&mut self, uploaded: bool, period_ns: i64, work: Duration, hud: bool) {
        let now = Instant::now();
        self.frames += 1;
        self.videos += u64::from(uploaded);
        self.target_fps = 1e9 / period_ns.max(1) as f64;
        if let Some(stats) = &mut self.stats {
            if let Some(previous) = self.previous {
                stats.sample(
                    "interval_ms",
                    now.duration_since(previous).as_secs_f64() * 1000.0,
                );
            }
            stats.sample("work_ms", work.as_secs_f64() * 1000.0);
            stats.count("submitted", 1);
            stats.count("fresh_video", uploaded.into());
            stats.count("hud", hud.into());
        }
        self.previous = Some(now);
        let elapsed = self.period.elapsed().as_secs_f64();
        if elapsed >= 0.5 {
            self.xr_fps = self.frames as f64 / elapsed;
            self.video_fps = self.videos as f64 / elapsed;
            self.frames = 0;
            self.videos = 0;
            self.period = now;
        }
    }

    pub fn suspend(&mut self) {
        self.previous = None;
        self.frames = 0;
        self.videos = 0;
        self.period = Instant::now();
    }

    pub fn report(&mut self, renderer: &mut Renderer, final_report: bool) {
        let Some(stats) = &mut self.stats else { return };
        let seconds = self.report.elapsed().as_secs_f64();
        if !final_report && seconds < 5.0 {
            return;
        }
        if let Some(gpu) = &mut renderer.stats {
            stats.merge(std::mem::take(gpu));
        }
        eprintln!("{}", stats.report("xr", seconds));
        self.report = Instant::now();
    }
}
