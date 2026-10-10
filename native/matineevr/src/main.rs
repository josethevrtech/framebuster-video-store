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
mod store_display;
mod store_capture;
mod store_fixtures;
mod store_frontage;
mod store_scale;
mod store_signs;
mod store_endcaps;
mod store_carpet;
mod store_props;
mod store_scene;
mod store_catalog;
mod store_poster;
mod store_cover_mesh;
mod store_covers;
mod store_texture;
mod store_runtime;
mod store_motion;
mod store_controls;
mod store_layout;
mod store_collision;
mod store_decor;
mod store_arcade;
mod store_console;
mod store_games;
mod store_game_mesh;
mod store_game_catalog;
mod store_game_racks;
mod store_game_signs;
mod store_bounds;
mod store_crt_screen;
mod store_speakers;
mod store_albums;
mod store_album_catalog;
mod store_album_racks;
mod store_album_signs;
mod store_audio;
mod store_music_cover;
mod store_material_props;
mod swapchains;
mod thumbnail;
mod xr;
mod xr_draw;
mod xr_space;

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
mod store_night_frontage;
mod store_trailer;
