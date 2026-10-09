use crate::{app::App, graphics::Graphics, hud_text::WIDTH, input::Controls,
    panel::Panel, store_catalog::{self, Movie}};
use anyhow::Result;
use openxr as xr;
use std::{fs, path::PathBuf, rc::Rc, time::{Duration, Instant}};

pub struct StoreRuntime {
    directory: PathBuf,
    posters: Panel,
    movies: Vec<Movie>,
    selected: Option<usize>,
    catalog_path: String,
    request: String,
    next: Instant,
    sequence: u64,
    stick: bool,
    resume: Option<(PathBuf, f64)>,
}

pub fn update(runtime: &mut Option<StoreRuntime>,
    app: &mut App, video: &mut crate::xr_draw::Video, controls: Controls,
    aims: [Option<xr::Posef>; 2], hands: [Option<xr::Posef>; 2], dt: f32, active: bool) -> Result<bool> {
    let selected = video.update_store(active, controls, aims, hands, dt);
    if let Some(runtime) = runtime { return runtime.update(app, controls, selected, active); }
    if active && controls.b { return Ok(true); }
    if selected.is_some() && let Some(path) = std::env::var_os("HALCYON_FRAME_STORE_SAMPLE") {
        app.queue_store_movie(path.into());
    }
    Ok(false)
}

impl StoreRuntime {
    pub fn new(session: &xr::Session<xr::Vulkan>, device: Rc<Graphics>) -> Result<Option<Self>> {
        let Some(directory) = std::env::var_os("HALCYON_FRAME_STORE_IPC") else { return Ok(None); };
        Ok(Some(Self { directory: directory.into(),
            posters: Panel::new(session, device.clone(), [1440, 3240])?, movies: Vec::new(), selected: None,
            catalog_path: String::new(), request: String::new(), next: Instant::now(), sequence: 0,
            stick: false, resume: None }))
    }

    fn command(&mut self, action: &str, value: usize) -> Result<()> {
        self.sequence += 1;
        fs::write(self.directory.join("command"), format!("{}\n{action}\n{value}", self.sequence))?;
        Ok(())
    }

    pub fn update(&mut self, app: &mut App,
        controls: Controls, selection: Option<usize>, active: bool) -> Result<bool> {
        if active {
            if let Some(index) = selection.filter(|i| *i < self.movies.len()) {
                self.selected = Some(index);
            }
            if controls.b {
                if self.selected.take().is_none() { self.command("back", 0)?; }
            } else if controls.a && let Some(index) = self.selected {
                self.command("play", index)?;
                self.selected = None;
            } else if controls.y { self.command("search", 0)?; }
            let pressed = controls.sticks[1][1].abs() > 0.65;
            if pressed && !self.stick {
                self.command(if controls.sticks[1][1] > 0.0 { "previous" } else { "page" }, 1)?;
                self.selected = None;
            }
            self.stick = pressed;
        }
        if let Some((path, seconds)) = &self.resume {
            if &app.path == path && let Some(player) = &mut app.playback {
                if *seconds > 0.0 { player.seek(*seconds)?; }
                self.resume = None;
            }
        }
        if Instant::now() < self.next { return Ok(false); }
        self.next = Instant::now() + Duration::from_millis(250);
        if let Ok(path) = fs::read_to_string(self.directory.join("catalog-path")) {
            if path != self.catalog_path && PathBuf::from(&path).parent() == Some(self.directory.as_path()) {
                let movies = store_catalog::read(std::path::Path::new(&path))?;
                let canvas = crate::store_poster::atlas(&movies);
                self.posters.upload(&canvas.pixels)?;
                self.movies = movies; self.catalog_path = path; self.selected = None;
            }
        }
        if let Ok(request) = fs::read_to_string(self.directory.join("movie-request")) {
            if request != self.request {
                let lines: Vec<_> = request.lines().collect();
                if lines.len() == 3 {
                    let path = PathBuf::from(lines[1]);
                    let seconds: f64 = lines[2].parse()?;
                    anyhow::ensure!(matineevr::stream_path::is_stream(&path)
                        && seconds.is_finite() && (0.0..=1e7).contains(&seconds), "Invalid store playback request");
                    app.queue_store_movie(path.clone()); self.resume = Some((path, seconds)); self.request = request;
                }
            }
        }
        Ok(false)
    }

    pub fn layers<'a>(&'a self, space: &'a xr::Space, room: xr::Posef, scale: f32) -> Vec<xr::CompositionLayerQuad<'a, xr::Vulkan>> {
        let transform = |p: [f32; 3], yaw: f32| {
            let p = crate::store_scale::point(p, scale);
            let q = room.orientation;
            let c = 1.0 - 2.0 * q.y * q.y; let s = 2.0 * q.y * q.w;
            let angle = s.atan2(c) + yaw;
            xr::Posef { orientation: xr::Quaternionf { x: 0.0, y: (angle / 2.0).sin(), z: 0.0, w: (angle / 2.0).cos() }, position: xr::Vector3f {
                x: c * p[0] + s * p[2] + room.position.x, y: p[1] + room.position.y,
                z: -s * p[0] + c * p[2] + room.position.z } }
        };
        let mut layers = Vec::new();
        for bank in 0..crate::store_display::BAYS.len() {
            let section = bank % 3;
            if section * 18 >= self.movies.len() { continue; }
            let (p, yaw) = crate::store_geometry::bank_pose(bank);
            layers.push(self.posters.region_layer(space, transform(p, yaw),
                1.62 * scale * WIDTH as f32 / 1440.0, section * 1080, [1440, 1080]));
        }
        for (index, &(p, yaw)) in crate::store_endcaps::POSTERS.iter().enumerate().take(self.movies.len()) {
            layers.push(self.posters.crop_layer(space, transform(p, yaw),
                0.70 * scale * WIDTH as f32 / 192.0, 24 + index * 240, 56, [192, 288]));
        }
        layers
    }
}
