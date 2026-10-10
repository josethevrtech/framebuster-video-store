use crate::options::Options;
use anyhow::{Result, anyhow};
use matineevr::{
    media::{Decoded, Decoder},
    playback::Playback,
};

pub enum Source {
    Decoder(Decoder),
    Playback(Box<Playback>),
}

impl Source {
    pub fn open(options: &Options) -> Result<Self> {
        Ok(if options.decoder {
            Self::Decoder(Decoder::open(&options.file)?)
        } else {
            let mut player = Playback::start(options.file.clone());
            if options.stats {
                player.diagnostics();
            }
            Self::Playback(Box::new(player))
        })
    }

    pub fn next(
        &mut self,
        render: &mut Option<matineevr::offscreen::Offscreen>,
    ) -> Result<Decoded> {
        match self {
            Self::Playback(player) => {
                if let Some(render) = render {
                    render.synchronize(Some(player))?;
                } else {
                    matineevr::playback_render::synchronize(Some(player), &mut [])?;
                }
            }
            Self::Decoder(decoder) => {
                if decoder.reconfigure(false)? {
                    if let Some(render) = render {
                        render.synchronize(None)?;
                    }
                    decoder.reconfigure(true)?;
                }
            }
        }
        match self {
            Self::Decoder(decoder) => decoder.advance(),
            Self::Playback(player) => Ok(match player.due_frame()? {
                Some(frame) => Decoded::Frame(frame),
                None if player.finished() => Decoded::End,
                None => player
                    .take_preview()
                    .map_or(Decoded::Pending, Decoded::Preview),
            }),
        }
    }

    pub fn seek(&mut self, target: f64) -> Result<()> {
        match self {
            Self::Decoder(decoder) => decoder.seek(target),
            Self::Playback(player) => player.seek(target),
        }
    }

    pub fn skipped(&self) -> u64 {
        match self {
            Self::Decoder(_) => 0,
            Self::Playback(player) => player.skipped,
        }
    }

    pub fn take_trace(&self) -> Option<matineevr::media_trace::SeekTrace> {
        match self {
            Self::Playback(player) => player.take_trace(),
            Self::Decoder(_) => None,
        }
    }

    pub fn stop(self) -> Result<()> {
        if let Self::Playback(player) = self {
            player
                .stop()
                .join()
                .map_err(|_| anyhow!("Decoder worker panicked"))?;
        }
        Ok(())
    }
}
