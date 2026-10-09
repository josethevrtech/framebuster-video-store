use std::time::{Duration, Instant};

pub struct HoldRepeat {
    direction: i8,
    started: Option<Instant>,
    next: Option<Instant>,
    delay: Duration,
    acceleration: f64,
    max_speed: f64,
}

impl HoldRepeat {
    pub const BROWSER: Self = Self {
        direction: 0,
        started: None,
        next: None,
        delay: Duration::from_millis(100),
        acceleration: 8.0,
        max_speed: 25.0,
    };
    pub const SEEK: Self = Self {
        delay: Duration::from_millis(250),
        acceleration: 2.0,
        max_speed: 20.0,
        ..Self::BROWSER
    };

    pub fn update(&mut self, direction: i8, pressed: bool, now: Instant) -> i8 {
        if direction != self.direction || pressed {
            self.direction = direction;
            self.started = Some(now + self.delay);
            self.next = self.started;
        } else if direction != 0 && self.next.is_some_and(|next| now >= next) {
            let elapsed = now.duration_since(self.started.unwrap()).as_secs_f64();
            let speed = (self.delay.as_secs_f64().recip() + elapsed * self.acceleration)
                .min(self.max_speed);
            self.next = Some(now + Duration::from_secs_f64(speed.recip()));
            return direction;
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taps_release_and_direction_changes_reset_the_delay() {
        for mut repeat in [HoldRepeat::BROWSER, HoldRepeat::SEEK] {
            let delay = repeat.delay;
            let mut now = Instant::now();
            for _ in 0..20 {
                assert_eq!(repeat.update(1, true, now), 0);
                assert_eq!(repeat.update(1, false, now + delay / 2), 0);
                assert_eq!(repeat.update(0, false, now + delay / 2), 0);
                now += delay;
            }
            for _ in 0..20 {
                assert_eq!(repeat.update(1, true, now), 0);
                now += delay / 2;
                assert_eq!(repeat.update(1, false, now), 0);
            }
            assert_eq!(repeat.update(1, true, now), 0);
            assert_eq!(repeat.update(1, false, now + delay), 1);
            now += delay;
            assert_eq!(repeat.update(-1, false, now), 0);
            assert_eq!(repeat.update(-1, false, now + delay / 2), 0);
            assert_eq!(repeat.update(-1, false, now + delay), -1);
            assert_eq!(repeat.update(0, false, now + delay), 0);
            assert_eq!(repeat.update(0, false, now + Duration::from_secs(10)), 0);
        }
    }

    #[test]
    fn held_movement_accelerates_to_the_cap_without_catch_up_bursts() {
        for mut repeat in [HoldRepeat::BROWSER, HoldRepeat::SEEK] {
            let delay = repeat.delay;
            let start = Instant::now();
            let mut events = Vec::new();
            for milliseconds in 0..=10000 {
                let now = start + Duration::from_millis(milliseconds);
                if repeat.update(1, false, now) != 0 {
                    events.push(now.duration_since(start));
                    assert_eq!(repeat.update(1, false, now), 0);
                }
            }
            assert_eq!(events[0], delay);
            let intervals: Vec<_> = events.windows(2).map(|pair| pair[1] - pair[0]).collect();
            assert_eq!(intervals[0], delay);
            assert!(intervals.windows(2).all(|pair| pair[1] <= pair[0]));
            assert!(intervals[1] < intervals[0]);
            let fastest = Duration::from_secs_f64(repeat.max_speed.recip());
            assert_eq!(*intervals.last().unwrap(), fastest);
            assert!(intervals.iter().all(|interval| *interval >= fastest));
            let late = start + Duration::from_secs(60);
            assert_eq!(repeat.update(1, false, late), 1);
            assert_eq!(repeat.update(1, false, late), 0);
            assert_eq!(repeat.update(0, false, late), 0);
            assert_eq!(repeat.update(1, false, late), 0);
            assert_eq!(repeat.update(1, false, late + delay / 2), 0);
            assert_eq!(repeat.update(1, false, late + delay), 1);
        }
    }
}
