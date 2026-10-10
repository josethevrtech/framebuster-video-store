use crate::{app::App, graphics::Graphics, input::Controls, store_catalog::{self, Movie}};
use anyhow::Result;
use openxr as xr;
use std::{fs, path::PathBuf, rc::Rc, time::{Duration, Instant}};

pub struct StoreRuntime {
    directory: PathBuf,
    movies: Vec<Movie>,
    selected: Option<usize>,
    catalog_path: String,
    request: String,
    next: Instant,
    sequence: u64,
    stick: bool,
    resume: Option<(PathBuf, f64)>,
    music_art: String,
    game_catalog_path: String,
}

pub fn update(runtime: &mut Option<StoreRuntime>,
    app: &mut App, video: &mut crate::xr_draw::Video, controls: Controls,
    aims: [Option<xr::Posef>; 2], hands: [Option<xr::Posef>; 2], dt: f32, active: bool) -> Result<bool> {
    let controls = crate::store_controls::normalize(controls);
    let selected = video.update_store(active, controls, aims, hands, dt);
    if let Some(runtime) = runtime { return runtime.update(app, video, controls, selected, active); }
    if active && controls.b { return Ok(true); }
    if selected.is_some() && let Some(path) = std::env::var_os("HALCYON_FRAME_STORE_SAMPLE") {
        app.queue_store_movie(path.into());
    }
    Ok(false)
}

impl StoreRuntime {
    pub fn new(_session: &xr::Session<xr::Vulkan>, _device: Rc<Graphics>) -> Result<Option<Self>> {
        let Some(directory) = std::env::var_os("HALCYON_FRAME_STORE_IPC") else { return Ok(None); };
        Ok(Some(Self { directory: directory.into(),
            movies: Vec::new(), selected: None,
            catalog_path: String::new(), request: String::new(), next: Instant::now(), sequence: 0,
            stick: false, resume: None,music_art:String::new(),game_catalog_path:String::new() }))
    }

    fn command(&mut self, action: &str, value: usize) -> Result<()> {
        self.sequence += 1;
        fs::write(self.directory.join("command"), format!("{}\n{action}\n{value}", self.sequence))?;
        Ok(())
    }

    pub fn update(&mut self, app: &mut App, video: &mut crate::xr_draw::Video,
        controls: Controls, selection: Option<usize>, active: bool) -> Result<bool> {
        if active { video.store_trailer(&self.directory); }
        if active {
            if let Some((action,index))=video.game_action() {self.selected=None;self.command(action,index)?;}
            if let Some(action)=video.music_action() { self.selected=None; self.command(action,0)?; }
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
        let [left,right]=video.store_audio();
        fs::write(self.directory.join("music-volume"),format!("[{left:.5},{right:.5}]"))?;
        if let Ok(path)=fs::read_to_string(self.directory.join("game-catalog-path")) {
            if path!=self.game_catalog_path && PathBuf::from(&path).parent()==Some(self.directory.as_path()) {
                let games=store_catalog::read(std::path::Path::new(&path))?;
                let types=fs::read(format!("{path}.types"))?;
                video.store_games(&games,&types)?;self.game_catalog_path=path;
            }
        }
        if let Ok(path)=fs::read_to_string(self.directory.join("music-art-path")) {
            let file=PathBuf::from(&path);
            if path!=self.music_art && file.parent()==Some(self.directory.as_path())
                && fs::metadata(&file).is_ok_and(|m| m.len()==384*384*4) {
                video.store_music(&file)?; self.music_art=path;
            }
        }
        if let Ok(path) = fs::read_to_string(self.directory.join("catalog-path")) {
            if path != self.catalog_path && PathBuf::from(&path).parent() == Some(self.directory.as_path()) {
                let movies = store_catalog::read(std::path::Path::new(&path))?;
                video.store_covers(&movies)?;
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
}
