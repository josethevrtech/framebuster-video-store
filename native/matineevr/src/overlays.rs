use crate::{
    app::App, browser_view::panel_size, hud::Hud, performance::Performance,
    pose_filter::PoseFilter, seek_view, shortcuts::PanelKind, thumbnail::Thumbnail,
};
use anyhow::Result;
use openxr as xr;
use std::{path::PathBuf, rc::Rc, time::Instant};

const HAND_PANEL_TILT_DEGREES: f32 = -60.0;
const HAND_PANEL_WIDTH: f32 = 0.42;

pub struct Overlays {
    pub thumbnail: Thumbnail,
    view_space: xr::Space,
    hand_filters: [PoseFilter; 2],
    panels: [Option<Hud>; 4],
    visible: [Option<PanelKind>; 4],
}

impl Overlays {
    pub fn new(
        session: &xr::Session<xr::Vulkan>,
        device: Rc<crate::graphics::Graphics>,
    ) -> Result<Self> {
        Ok(Self {
            thumbnail: Thumbnail::new(session, device)?,
            view_space: session
                .create_reference_space(xr::ReferenceSpaceType::VIEW, xr::Posef::IDENTITY)?,
            panels: Default::default(),
            hand_filters: Default::default(),
            visible: Default::default(),
        })
    }

    pub fn hand_offset() -> xr::Posef {
        let (x, w) = (HAND_PANEL_TILT_DEGREES.to_radians() / 2.0).sin_cos();
        xr::Posef {
            orientation: xr::Quaternionf {
                x,
                y: 0.0,
                z: 0.0,
                w,
            },
            position: xr::Vector3f {
                x: 0.0,
                y: 0.1,
                z: -0.1,
            },
        }
    }

    pub fn update(
        &mut self,
        session: &xr::Session<xr::Vulkan>,
        device: &Rc<crate::graphics::Graphics>,
        app: &App,
        stats: &Performance,
        snapshot: &mut Option<PathBuf>,
    ) -> Result<()> {
        let [left, right, head] = app.panels();
        let seek = app.playback.as_ref().is_some_and(|player| {
            seek_view::visible(
                app.seek_until,
                player.seek_target().is_some(),
                Instant::now(),
            )
        });
        let visible = [left, right, head, seek.then_some(PanelKind::Seek)];
        for (slot, kind) in visible.iter().enumerate() {
            if let Some(kind) = kind {
                let size = if *kind == PanelKind::Seek {
                    seek_view::SIZE
                } else {
                    panel_size(*kind, app.playback.is_some())
                };
                if self.panels[slot]
                    .as_ref()
                    .is_none_or(|hud| hud.panel.size != size)
                {
                    self.panels[slot] = Some(Hud::new(session, device.clone(), &app.path, size)?);
                }
                self.panels[slot].as_mut().unwrap().update(
                    app,
                    *kind,
                    self.visible[slot] != Some(*kind),
                    stats,
                    snapshot,
                )?;
            } else if let Some(panel) = &mut self.panels[slot] {
                panel.hide();
            }
        }
        self.visible = visible;
        Ok(())
    }

    pub fn reset_hands(&mut self) {
        self.hand_filters = Default::default();
    }

    pub fn layers<'a>(
        &'a mut self,
        base: &'a xr::Space,
        hands: [Option<xr::Posef>; 2],
        time: xr::Time,
    ) -> Vec<xr::CompositionLayerQuad<'a, xr::Vulkan>> {
        let poses: [_; 2] = std::array::from_fn(|hand| {
            let pose = self.visible[hand].and(hands[hand]);
            self.hand_filters[hand].update(pose, time)
        });
        let mut layers: Vec<_> = self
            .visible
            .iter()
            .enumerate()
            .filter_map(|(slot, kind)| {
                let kind = kind.as_ref()?;
                let (space, pose, width) = if let Some(pose) = poses.get(slot) {
                    (base, (*pose)?, HAND_PANEL_WIDTH)
                } else {
                    let pose = if *kind == PanelKind::Seek {
                        seek_view::POSE
                    } else {
                        xr::Posef {
                            position: xr::Vector3f {
                                x: 0.0,
                                y: match kind {
                                    PanelKind::Browser => 0.0,
                                    _ => -0.43,
                                },
                                z: -1.4,
                            },
                            ..xr::Posef::IDENTITY
                        }
                    };
                    (&self.view_space, pose, seek_view::WIDTH_METERS)
                };
                Some(
                    self.panels[slot]
                        .as_ref()
                        .unwrap()
                        .panel
                        .layer(space, pose, width),
                )
            })
            .collect();
        if self.visible[3].is_some() {
            layers.extend(self.thumbnail.layer(&self.view_space));
        }
        layers
    }

    pub fn submitted(&mut self, stats: bool) {
        if self.visible[3].is_some() {
            self.thumbnail.submitted(stats);
        }
    }
}
