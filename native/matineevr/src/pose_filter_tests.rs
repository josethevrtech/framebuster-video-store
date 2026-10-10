use super::*;

fn time(seconds: f32) -> xr::Time {
    xr::Time::from_nanos((f64::from(seconds) * 1e9) as i64)
}

fn pose(position: f32, angle: f32) -> xr::Posef {
    let (z, w) = (angle / 2.0).sin_cos();
    make_pose([position, -position, position], [0.0, 0.0, z, w])
}

fn angle(pose: xr::Posef) -> f32 {
    2.0 * pose.orientation.z.atan2(pose.orientation.w)
}

#[test]
fn reduces_tremor_with_variable_frame_intervals() {
    for rate in [72, 90, 120] {
        let mut filter = PoseFilter::default();
        let mut seconds = 0.0;
        let mut energy = [0.0; 4];
        for frame in 0..rate * 3 {
            let dt = [1.0, 0.75, 1.25][frame % 3] / rate as f32;
            seconds += dt;
            let wave = (TAU * 10.0 * seconds).sin();
            let raw = pose(0.001 * wave, 0.5_f32.to_radians() * wave);
            let filtered = filter.update(Some(raw), time(seconds)).unwrap();
            if seconds > 1.0 {
                for (sum, value) in energy.iter_mut().zip([
                    raw.position.x,
                    filtered.position.x,
                    angle(raw),
                    angle(filtered),
                ]) {
                    *sum += value * value * dt;
                }
            }
        }
        let position_ratio = (energy[1] / energy[0]).sqrt();
        let rotation_ratio = (energy[3] / energy[2]).sqrt();
        eprintln!(
            "{rate} Hz tremor RMS ratios: position={position_ratio:.3}, rotation={rotation_ratio:.3}"
        );
        assert!(position_ratio < 0.6 && rotation_ratio < 0.6);
    }
}

#[test]
fn fast_motion_keeps_up_and_stops_without_overshoot() {
    for rate in [72, 90, 120] {
        let mut filter = PoseFilter::default();
        let mut max_lag = [0.0_f32; 2];
        let mut previous = pose(0.0, 0.0);
        for frame in 0..=rate * 2 {
            let seconds = frame as f32 / rate as f32;
            let raw = pose(2.0 * seconds.min(1.0), 4.0 * seconds.min(1.0));
            let filtered = filter.update(Some(raw), time(seconds)).unwrap();
            assert!(filtered.position.x >= previous.position.x - 1e-6);
            assert!(filtered.position.x <= raw.position.x + 1e-6);
            assert!(angle(filtered) >= angle(previous) - 1e-6);
            assert!(angle(filtered) <= angle(raw) + 1e-6);
            assert!((length(quaternion(filtered.orientation)) - 1.0).abs() < 1e-6);
            if (0.5..=1.0).contains(&seconds) {
                max_lag[0] = max_lag[0].max((raw.position.x - filtered.position.x) / 2.0);
                max_lag[1] = max_lag[1].max((angle(raw) - angle(filtered)) / 4.0);
            }
            previous = filtered;
        }
        eprintln!(
            "{rate} Hz fast-motion lag: position={:.1} ms, rotation={:.1} ms",
            max_lag[0] * 1000.0,
            max_lag[1] * 1000.0
        );
        assert!(max_lag.into_iter().all(|lag| lag < 0.025));
        assert!((previous.position.x - 2.0).abs() < 1e-5);
        assert!((angle(previous) - 4.0).abs() < 1e-5);
    }
}

#[test]
fn quaternion_sign_changes_and_half_turns_remain_continuous() {
    let mut filter = PoseFilter::default();
    for frame in 0..100 {
        let mut raw = pose(0.0, 179.0_f32.to_radians());
        if frame % 2 != 0 {
            raw.orientation =
                make_pose([0.0; 3], quaternion(raw.orientation).map(|v| -v)).orientation;
        }
        let filtered = filter.update(Some(raw), time(frame as f32 / 90.0)).unwrap();
        assert!((angle(filtered) - 179.0_f32.to_radians()).abs() < 1e-6);
    }
    let filtered = filter
        .update(Some(pose(0.0, -179.0_f32.to_radians())), time(100.0 / 90.0))
        .unwrap();
    assert!((179.0..181.0).contains(&angle(filtered).to_degrees()));
    let filtered = filter
        .update(Some(pose(0.0, 0.0)), time(101.0 / 90.0))
        .unwrap();
    assert!((length(quaternion(filtered.orientation)) - 1.0).abs() < 1e-6);
}

#[test]
fn missing_poses_and_discontinuous_times_restart_immediately() {
    let mut filter = PoseFilter::default();
    assert_eq!(
        filter.update(Some(pose(0.0, 0.0)), time(0.0)),
        Some(pose(0.0, 0.0))
    );
    let moved = filter.update(Some(pose(1.0, 0.0)), time(0.01)).unwrap();
    assert!(moved.position.x > 0.0 && moved.position.x < 1.0);
    assert_eq!(filter.update(None, time(0.02)), None);
    for (seconds, position) in [(0.03, 2.0), (1.0, 3.0), (1.0, 4.0), (0.5, 5.0)] {
        let raw = pose(position, 0.0);
        assert_eq!(filter.update(Some(raw), time(seconds)), Some(raw));
    }
}
