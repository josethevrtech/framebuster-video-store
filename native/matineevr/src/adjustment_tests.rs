use super::*;
use std::time::Duration;

const CENTER: [[f32; 2]; 2] = [[0.0; 2]; 2];
const MOVING: [[f32; 2]; 2] = [[1.0; 2]; 2];

#[test]
fn hysteresis_ownership_neutral_handoff_and_frame_time() {
    let mut input = Adjustment::default();
    let mut now = Instant::now();
    let mut step = |input: &mut Adjustment, triggers, sticks| {
        now += Duration::from_millis(10);
        input.update(triggers, sticks, true, now)
    };
    assert_eq!(step(&mut input, [0.0, 0.54], MOVING), 0.0);
    assert!(!input.blocked());
    assert_eq!(step(&mut input, [0.0, 0.6], MOVING), 0.0);
    assert_eq!(input.mode(), Some(Mode::Position));
    assert_eq!(step(&mut input, [0.0, 0.4], MOVING), 0.0);
    step(&mut input, [0.0, 0.4], CENTER);
    assert_eq!(step(&mut input, [0.9, 0.4], MOVING), 0.01);
    assert_eq!(input.mode(), Some(Mode::Position));
    assert_eq!(step(&mut input, [0.9, 0.3], MOVING), 0.0);
    assert_eq!(input.mode(), None);
    assert!(input.blocked());
    assert_eq!(step(&mut input, [0.9, 0.9], MOVING), 0.0);
    assert_eq!(input.mode(), Some(Mode::Position));
    step(&mut input, [0.0; 2], MOVING);
    assert!(input.blocked());
    step(&mut input, [0.0; 2], CENTER);
    assert!(!input.blocked());
    step(&mut input, [0.9; 2], CENTER);
    assert_eq!(input.mode(), Some(Mode::Orientation));
    now += Duration::from_secs(1);
    assert_eq!(input.update([0.9; 2], MOVING, true, now), MAX_STEP_SECONDS);
    assert_eq!(input.update([0.9; 2], CENTER, false, now), 0.0);
    assert_eq!(input.mode(), None);
}
