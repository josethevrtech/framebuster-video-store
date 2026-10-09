use crate::{playback::Playback, renderer::Renderer};
use anyhow::Result;

pub fn synchronize(player: Option<&mut Playback>, renderers: &mut [&mut Renderer]) -> Result<()> {
    if let Some(player) = player {
        if player.release_required() {
            for renderer in renderers {
                renderer.reset()?;
            }
            player.released();
        }
    } else {
        for renderer in renderers {
            if renderer.ready() {
                renderer.reset()?;
            }
        }
    }
    Ok(())
}
