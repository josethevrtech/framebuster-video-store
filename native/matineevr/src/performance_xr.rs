use super::Performance;
use anyhow::Result;
use matineevr::statistics::timed;
use openxr as xr;

impl Performance {
    pub fn wait(
        &mut self,
        waiter: &mut xr::FrameWaiter,
        stream: &mut xr::FrameStream<xr::Vulkan>,
        focused: bool,
        visible: bool,
    ) -> Result<xr::FrameState> {
        let timing = timed(self.stats.as_mut(), "xr_wait_ms", || waiter.wait())?;
        timed(self.stats.as_mut(), "xr_begin_ms", || stream.begin())?;
        if let Some(stats) = &mut self.stats {
            stats.sample(
                "application_period_ms",
                timing.predicted_display_period.as_nanos() as f64 / 1e6,
            );
            stats.count("should_render", timing.should_render.into());
            stats.count("focused", focused.into());
            stats.count("visible", visible.into());
        }
        Ok(timing)
    }

    pub fn empty(
        &mut self,
        stream: &mut xr::FrameStream<xr::Vulkan>,
        time: xr::Time,
    ) -> Result<()> {
        timed(self.stats.as_mut(), "xr_end_ms", || {
            stream.end(time, xr::EnvironmentBlendMode::OPAQUE, &[])
        })?;
        Ok(())
    }
}
