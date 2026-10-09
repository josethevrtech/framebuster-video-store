use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Navigation {
    pub horizontal: i8,
    pub vertical: i8,
}

impl Navigation {
    pub fn stick(x: f32, y: f32) -> Self {
        if x.abs().max(y.abs()) < 0.6 {
            Self::default()
        } else if x.abs() > y.abs() {
            Self {
                horizontal: x.signum() as i8,
                vertical: 0,
            }
        } else {
            Self {
                horizontal: 0,
                vertical: -y.signum() as i8,
            }
        }
    }
}

#[derive(Default)]
pub struct Repeat {
    direction: Navigation,
    next: Option<Instant>,
}

impl Repeat {
    pub fn update(&mut self, direction: Navigation, now: Instant, repeat: bool) -> Navigation {
        if direction != self.direction {
            self.direction = direction;
            self.next = Some(now + Duration::from_millis(350));
            direction
        } else if repeat
            && direction != Navigation::default()
            && self.next.is_some_and(|next| now >= next)
        {
            self.next = Some(now + Duration::from_millis(140));
            direction
        } else {
            Navigation::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_dead_zone_dominant_axis_and_repeat() {
        let now = Instant::now();
        let up = Navigation {
            horizontal: 0,
            vertical: -1,
        };
        assert_eq!(Navigation::stick(0.2, 0.3), Navigation::default());
        assert_eq!(Navigation::stick(0.7, 0.9), up);
        let mut repeat = Repeat::default();
        assert_eq!(repeat.update(up, now, true), up);
        assert_eq!(
            repeat.update(up, now + Duration::from_millis(300), true),
            Navigation::default()
        );
        assert_eq!(
            repeat.update(up, now + Duration::from_millis(350), true),
            up
        );
        assert_eq!(
            repeat.update(up, now + Duration::from_millis(400), true),
            Navigation::default()
        );
        assert_eq!(
            repeat.update(up, now + Duration::from_secs(2), false),
            Navigation::default()
        );
        repeat.update(Navigation::default(), now, false);
        assert_eq!(repeat.update(up, now, true), up);
    }
}
