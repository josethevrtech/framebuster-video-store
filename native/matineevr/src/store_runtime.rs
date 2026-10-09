use crate::{app::App, graphics::Graphics, hud_text::{Canvas, WIDTH}, input::Controls,
    panel::Panel, store_catalog::{self, Movie}};
use anyhow::Result;
use openxr as xr;
use std::{fs, path::PathBuf, rc::Rc, time::{Duration, Instant}};

pub struct StoreRuntime {
    directory: PathBuf,
    status: Panel,
    detail: Panel,
    posters: Panel,
    sign: Panel,
    movies: Vec<Movie>,
    selected: Option<usize>,
    catalog_path: String,
    status_text: String,
    request: String,
    next: Instant,
    sequence: u64,
    stick: bool,
    resume: Option<(PathBuf, f64)>,
}

pub fn update(runtime: &mut Option<StoreRuntime>,
    app: &mut App, video: &mut crate::xr_draw::Video, controls: Controls,
    aims: [Option<xr::Posef>; 2], active: bool) -> Result<bool> {
    let selected = video.update_store(active, controls, aims);
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
        let mut sign = Panel::new(session, device.clone(), [960, 240])?;
        sign.upload(include_bytes!("../assets/movies-series-sign.rgba"))?;
        Ok(Some(Self { directory: directory.into(), status: Panel::new(session, device.clone(), [WIDTH, 144])?,
            detail: Panel::new(session, device.clone(), [WIDTH, 420])?,
            posters: Panel::new(session, device.clone(), [1440, 3240])?, sign, movies: Vec::new(), selected: None, catalog_path: String::new(),
            status_text: String::new(), request: String::new(), next: Instant::now(), sequence: 0,
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
                self.draw_detail(index)?;
            }
            if controls.b {
                if self.selected.take().is_none() { self.command("back", 0)?; }
            } else if controls.a && let Some(index) = self.selected {
                self.command("play", index)?;
                self.selected = None;
            } else if controls.y { self.command("search", 0)?; }
            let pressed = controls.sticks[1][0].abs() > 0.65;
            if pressed && !self.stick {
                self.command(if controls.sticks[1][0] < 0.0 { "previous" } else { "page" }, 1)?;
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
        if let Ok(text) = fs::read_to_string(self.directory.join("status")) {
            if text != self.status_text {
                let mut canvas = Canvas::new(std::path::Path::new(""), self.status.size);
                canvas.clear();
                for (row, line) in text.lines().take(6).enumerate() { canvas.line(row, line, [235, 240, 250, 255]); }
                self.status.upload(&canvas.pixels)?;
                self.status_text = text;
            }
        }
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

    fn draw_detail(&mut self, index: usize) -> Result<()> {
        let movie = &self.movies[index];
        let mut canvas = Canvas::new(std::path::Path::new(""), self.detail.size);
        canvas.clear();
        canvas.line(0, &movie.title, [255, 215, 110, 255]);
        canvas.line(2, if movie.title.ends_with(" [Series]") { "A: Browse episodes     B: Back to shelves" }
            else { "A: Play / Resume     B: Back to shelves" }, [110, 220, 255, 255]);
        let mut line = String::new(); let mut row = 4;
        for word in movie.overview.split_whitespace() {
            if line.chars().count() + word.chars().count() > 62 {
                canvas.line(row, &line, [235, 240, 250, 255]); row += 1; line.clear();
                if row >= 19 { break; }
            }
            if !line.is_empty() { line.push(' '); } line.push_str(word);
        }
        if row < canvas.rows() { canvas.line(row, &line, [235, 240, 250, 255]); }
        self.detail.upload(&canvas.pixels)
    }

    pub fn layers<'a>(&'a self, space: &'a xr::Space, room: xr::Posef, scale: f32,
        selection: ([f32;3], f32)) -> Vec<xr::CompositionLayerQuad<'a, xr::Vulkan>> {
        let transform = |p: [f32; 3], yaw: f32| {
            let p = crate::store_scale::point(p, scale);
            let q = room.orientation;
            let c = 1.0 - 2.0 * q.y * q.y; let s = 2.0 * q.y * q.w;
            let angle = s.atan2(c) + yaw;
            xr::Posef { orientation: xr::Quaternionf { x: 0.0, y: (angle / 2.0).sin(), z: 0.0, w: (angle / 2.0).cos() }, position: xr::Vector3f {
                x: c * p[0] + s * p[2] + room.position.x, y: p[1] + room.position.y,
                z: -s * p[0] + c * p[2] + room.position.z } }
        };
        let mut layers = vec![self.status.layer(space, transform([0.0, 1.35, -4.3], 0.0), 3.4 * scale)];
        layers.push(self.sign.layer(space, transform([0.0, 1.15, -1.8], 0.0), 2.4 * scale * WIDTH as f32 / 960.0));
        for bank in 0..crate::store_display::BAYS.len() {
            let section = bank % 3;
            if section * 18 >= self.movies.len() { continue; }
            let (p, yaw) = crate::store_geometry::bank_pose(bank);
            layers.push(self.posters.region_layer(space, transform(p, yaw),
                1.62 * scale * WIDTH as f32 / 1440.0, section * 1080, [1440, 1080]));
        }
        for (index, x) in [-5.8, 5.8].into_iter().enumerate().take(self.movies.len()) {
            layers.push(self.posters.crop_layer(space, transform([x, 0.15, 12.80], std::f32::consts::PI),
                1.05 * scale * WIDTH as f32 / 192.0, 24 + index * 240, 56, [192, 288]));
        }
        if self.selected.is_some() {
            let (mut p, yaw) = selection;
            p[0] += yaw.sin() * 0.35; p[2] += yaw.cos() * 0.35;
            layers.push(self.detail.layer(space, transform(p, yaw), 1.8 * scale));
        }
        layers
    }
}
