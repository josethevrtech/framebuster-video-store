use crate::{
    app::App, hud_text::Canvas, metrics::Metrics, panel::Panel, performance::Performance,
    shortcuts::PanelKind, snapshot,
};
use anyhow::Result;
use openxr as xr;
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

pub struct Hud {
    pub panel: Panel,
    canvas: Canvas,
    metrics: Option<Metrics>,
    next_update: Instant,
    started: Instant,
    updates: u64,
    cost: Duration,
    max_cost: Duration,
}

impl Hud {
    pub fn hide(&mut self) {
        self.metrics = None;
    }

    pub fn new(
        session: &xr::Session<xr::Vulkan>,
        device: Rc<crate::graphics::Graphics>,
        path: &Path,
        size: [usize; 2],
    ) -> Result<Self> {
        let panel = Panel::new(session, device, size)?;
        eprintln!("HUD: {}x{}, 2 Hz idle updates", size[0], size[1]);
        Ok(Self {
            panel,
            canvas: Canvas::new(path, size),
            metrics: None,
            next_update: Instant::now(),
            started: Instant::now(),
            updates: 0,
            cost: Duration::ZERO,
            max_cost: Duration::ZERO,
        })
    }

    pub fn update(
        &mut self,
        app: &App,
        kind: PanelKind,
        changed: bool,
        stats: &Performance,
        snapshot_path: &mut Option<PathBuf>,
    ) -> Result<()> {
        if !changed && !app.changed && Instant::now() < self.next_update {
            return Ok(());
        }
        let start = Instant::now();
        if app.changed {
            self.canvas.set_path(&app.path);
        }
        let elapsed = self.started.elapsed().as_secs_f64();
        if kind != PanelKind::Information {
            self.metrics = None;
        }
        match kind {
            PanelKind::Information => {
                if let Some(player) = &app.playback {
                    let metrics = self.metrics.get_or_insert_with(Metrics::new);
                    metrics.refresh();
                    self.canvas.update(player, stats, metrics.latest, elapsed);
                }
            }
            PanelKind::Browser => self.canvas.browser(&app.browser, app.stopping(), elapsed),
            PanelKind::Settings => self.canvas.settings(app.presentation, app.settings_row),
            PanelKind::Alignment => self
                .canvas
                .alignment(app.presentation, app.adjustment.mode().unwrap()),
            PanelKind::Seek => {
                if let Some(player) = &app.playback {
                    self.canvas.seek(
                        player.seek_target().unwrap_or(player.position),
                        player.length,
                    );
                }
            }
        }
        self.panel.upload(&self.canvas.pixels)?;
        let cost = start.elapsed();
        self.updates += 1;
        self.cost += cost;
        self.max_cost = self.max_cost.max(cost);
        self.next_update = Instant::now() + Duration::from_millis(500);
        if kind != PanelKind::Seek
            && self.updates >= 5
            && !app.browser.loading()
            && let Some(path) = snapshot_path.take()
        {
            snapshot::save_pixels(
                &path,
                self.canvas.size[0] as i32,
                self.canvas.size[1] as i32,
                &self.canvas.pixels,
            )?;
        }
        Ok(())
    }
}

impl Drop for Hud {
    fn drop(&mut self) {
        if self.updates > 0 {
            eprintln!(
                "HUD: {} updates, mean {:.3} ms, max {:.3} ms (CPU raster + upload submission)",
                self.updates,
                self.cost.as_secs_f64() * 1000.0 / self.updates as f64,
                self.max_cost.as_secs_f64() * 1000.0
            );
        }
    }
}
