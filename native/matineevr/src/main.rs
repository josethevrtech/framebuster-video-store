mod alignment_view;
mod app;
#[cfg(test)]
mod app_tests;
mod browser;
mod browser_repeat;
mod browser_view;
mod controller_depth;
mod controller_draw;
mod controller_mesh;
mod controller_pipeline;
mod font;
mod frame_playback;
#[cfg(test)]
mod forward_seek_tests;
mod hud;
mod hud_text;
mod input;
mod input_tracking;
mod metrics;
mod options;
mod overlays;
mod panel;
mod performance;
#[cfg(test)]
mod playback_tests;
mod pose_filter;
#[cfg(test)]
mod preview_tests;
mod probe;
#[cfg(test)]
mod seek_input_tests;
#[cfg(test)]
mod seek_tests;
mod seek_view;
mod settings_view;
#[cfg(test)]
mod shortcut_tests;
mod shortcuts;
mod snapshot;
mod startup;
mod store_geometry;
mod store_scene;
mod swapchains;
mod thumbnail;
mod xr;
mod xr_draw;

use matineevr::{
    adjustment, alignment, graphics, media, navigation, playback, presentation, preview, renderer,
    video_settings,
};

fn main() -> anyhow::Result<()> {
    let Some(options) = options::Options::parse()? else {
        return Ok(());
    };
    startup::initialize()?;
    if options.probe || options.render_probe {
        probe::run(&options)
    } else {
        xr::run(&options)
    }
}
