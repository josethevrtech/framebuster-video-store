use crate::{app::App, media::Frame, panel::Panel, preview, renderer::Renderer, seek_view};
use anyhow::Result;
use openxr as xr;
use std::{rc::Rc, time::Instant};

pub struct Thumbnail {
    pub renderer: Renderer,
    panel: Panel,
    seek: Option<Instant>,
    reported: bool,
    pts: Option<f64>,
    x: f32,
}

impl Thumbnail {
    pub fn new(
        session: &xr::Session<xr::Vulkan>,
        device: Rc<crate::graphics::Graphics>,
    ) -> Result<Self> {
        Ok(Self {
            renderer: Renderer::new(device.clone())?,
            panel: Panel::new(session, device, preview::SIZE)?,
            seek: None,
            reported: false,
            pts: None,
            x: 0.0,
        })
    }

    pub fn update(&mut self, app: &mut App, frame: Option<&Rc<Frame>>) -> Result<()> {
        if self.seek != app.seek_until {
            self.seek = app.seek_until;
            self.pts = None;
            self.reported = false;
        }
        let Some(player) = &mut app.playback else {
            return Ok(());
        };
        self.x = center(
            player.seek_target().unwrap_or(player.position),
            player.length,
        );
        let preview = player.take_preview().map(Rc::new);
        if self.seek.is_some()
            && self.pts.is_none()
            && let Some(frame) = preview.as_ref().or(frame)
        {
            self.renderer.upload(frame.clone())?;
            self.panel.render(|image| {
                preview::draw(&mut self.renderer, image, app.presentation).map(|_| ())
            })?;
            self.pts = Some(frame.pixels.pts);
        }
        Ok(())
    }

    pub fn submitted(&mut self, stats: bool) {
        if let Some(pts) = self.pts
            && !self.reported
        {
            self.reported = true;
            if stats {
                let elapsed = (self.seek.unwrap() - seek_view::VISIBLE_FOR).elapsed();
                eprintln!(
                    "Seek preview submitted: {:.3}ms, pts={:.3}s",
                    elapsed.as_secs_f64() * 1000.0,
                    pts
                );
            }
        }
    }

    pub fn layer<'a>(
        &'a self,
        space: &'a xr::Space,
    ) -> Option<xr::CompositionLayerQuad<'a, xr::Vulkan>> {
        let mut pose = seek_view::POSE;
        pose.position.x = self.x;
        pose.position.y += seek_view::WIDTH_METERS * (preview::SIZE[1] + seek_view::SIZE[1]) as f32
            / (2 * seek_view::SIZE[0]) as f32
            + 0.02;
        self.pts
            .is_some()
            .then(|| self.panel.layer(space, pose, seek_view::WIDTH_METERS))
    }
}

fn center(position: f64, length: Option<f64>) -> f32 {
    let width = seek_view::SIZE[0] as f32;
    let half = preview::SIZE[0] as f32 / 2.0;
    let marker = seek_view::marker(position, length).map_or(width / 2.0, |x| x as f32 + 1.0);
    (marker.clamp(half, width - half) / width - 0.5) * seek_view::WIDTH_METERS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graphics::Graphics, options::Options};
    use std::{thread, time::Duration};

    #[test]
    fn thumbnail_tracks_cursor_and_stays_inside_bar_width() {
        let limit = seek_view::WIDTH_METERS
            * (1.0 - preview::SIZE[0] as f32 / seek_view::SIZE[0] as f32)
            / 2.0;
        assert_eq!(center(0.0, None), 0.0);
        assert_eq!(center(50.0, Some(100.0)), 0.0);
        assert!((center(-1.0, Some(100.0)) + limit).abs() < 0.00001);
        assert!((center(101.0, Some(100.0)) - limit).abs() < 0.00001);
        assert!(center(25.0, Some(100.0)) < 0.0);
        assert!(center(75.0, Some(100.0)) > 0.0);
    }

    #[test]
    #[ignore = "requires Frame OpenXR, FRAME_TEST_VIDEO and SteamVR loader environment"]
    fn xr_preview_clears_on_seek_and_stop_and_falls_back_to_target() {
        let instance = Graphics::instance().unwrap();
        let system = instance
            .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
            .unwrap();
        let graphics = Graphics::new(&instance, system).unwrap();
        let (session, _, _) = graphics.session(&instance, system).unwrap();
        let space = session
            .create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)
            .unwrap();
        let mut thumbnail = Thumbnail::new(&session, graphics.clone()).unwrap();
        let options = Options::from_args([std::env::var("FRAME_TEST_VIDEO").unwrap()].into_iter())
            .unwrap()
            .unwrap();
        let mut app = App::new(&options);
        app.playback.as_mut().unwrap().toggle_pause();
        for target in [1.25, 0.0] {
            app.playback.as_mut().unwrap().seek(target).unwrap();
            app.seek_until = Some(Instant::now() + seek_view::VISIBLE_FOR);
            thumbnail.update(&mut app, None).unwrap();
            assert!(thumbnail.layer(&space).is_none());
            let deadline = Instant::now() + Duration::from_secs(20);
            while thumbnail.pts.is_none() {
                assert!(Instant::now() < deadline, "Preview timed out");
                app.synchronize(&mut [&mut thumbnail.renderer]).unwrap();
                let frame = app.frame().map(Rc::new);
                assert!(app.playback.is_some(), "{}", app.browser.message);
                thumbnail.update(&mut app, frame.as_ref()).unwrap();
                thread::sleep(Duration::from_millis(1));
            }
            assert!(thumbnail.layer(&space).is_some());
            if target > 0.0 {
                assert!(thumbnail.pts.unwrap() < target);
                assert_eq!(app.playback.as_ref().unwrap().seek_target(), Some(target));
            } else {
                assert_eq!(thumbnail.pts, Some(0.0));
                assert_eq!(app.playback.as_ref().unwrap().seek_target(), Some(0.0));
            }
        }
        app.stop();
        thumbnail.update(&mut app, None).unwrap();
        assert!(thumbnail.layer(&space).is_none());
        crate::app_tests::wait(&mut app, |app| !app.stopping());
    }
}
