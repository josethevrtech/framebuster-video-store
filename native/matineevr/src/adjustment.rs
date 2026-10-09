use crate::alignment::{Mode, STICK_DEAD_ZONE};
use std::time::Instant;

pub const TRIGGER_PRESS: f32 = 0.55;
pub const TRIGGER_RELEASE: f32 = 0.35;
const MAX_STEP_SECONDS: f32 = 0.05;

#[derive(Default)]
pub struct Adjustment {
    owner: Option<Mode>,
    held: [bool; 2],
    armed: bool,
    blocked: bool,
    previous: Option<Instant>,
}

impl Adjustment {
    pub fn mode(&self) -> Option<Mode> {
        self.owner.filter(|mode| self.held[*mode as usize])
    }

    pub fn blocked(&self) -> bool {
        self.blocked
    }

    pub fn update(
        &mut self,
        triggers: [f32; 2],
        sticks: [[f32; 2]; 2],
        enabled: bool,
        now: Instant,
    ) -> f32 {
        let seconds = self.previous.replace(now).map_or(0.0, |last| {
            now.saturating_duration_since(last)
                .as_secs_f32()
                .min(MAX_STEP_SECONDS)
        });
        for (held, value) in self.held.iter_mut().zip(triggers) {
            *held = value >= TRIGGER_PRESS || *held && value >= TRIGGER_RELEASE;
        }
        let centered = sticks.iter().flatten().all(|v| v.abs() <= STICK_DEAD_ZONE);
        if !enabled || self.held == [false; 2] {
            self.owner = None;
        } else if self.owner.is_none() {
            self.owner = Some(if self.held[0] {
                Mode::Orientation
            } else {
                Mode::Position
            });
        }
        self.blocked = self.owner.is_some() || self.blocked && !centered;
        self.armed = self.mode().is_some() && (self.armed || centered);
        if self.armed { seconds } else { 0.0 }
    }
}

#[cfg(test)]
#[path = "adjustment_tests.rs"]
mod tests;
