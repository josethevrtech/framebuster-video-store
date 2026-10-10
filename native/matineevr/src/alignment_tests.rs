use super::*;

#[test]
fn motion_is_continuous_frame_rate_independent_and_bounded() {
    assert_eq!(response(STICK_DEAD_ZONE), 0.0);
    assert_eq!(response(1.0), 1.0);
    assert_eq!(response(-0.6), -response(0.6));
    assert!(response(0.6) < response(0.9));
    let identity = [0.0, 0.0, 0.0, 1.0];
    for rate in [60, 90, 120] {
        let mut a = Alignment::default();
        for _ in 0..rate {
            a.adjust(
                Mode::Orientation,
                [[1.0, -1.0], [1.0, 1.0]],
                1.0 / rate as f32,
                Stereo::Mono,
            );
        }
        assert!((a.yaw + 45.0).abs() < 0.001);
        assert!((a.pitch - 45.0).abs() < 0.001);
        assert!((a.roll + 45.0).abs() < 0.001);
        assert!((a.zoom_log2 - 0.75).abs() < 0.001);
        let q = a.rotation(identity, 0.7);
        assert!((q.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 0.00001);
        a.pitch = 0.0;
        a.roll = 0.0;
        assert_eq!(a.rotation(identity, a.yaw.to_radians()), identity);
    }
    let mut a = Alignment::default();
    assert_eq!(a.rotation(identity, 0.0), identity);
    a.adjust(Mode::Orientation, [[1.0; 2]; 2], 100.0, Stereo::Mono);
    assert!((-180.0..180.0).contains(&a.yaw));
    assert_eq!(a.pitch, -90.0);
    assert_eq!(a.zoom_log2, 2.0);
    a.adjust(Mode::Position, [[1.0; 2]; 2], 100.0, Stereo::Mono);
    assert_eq!(a.stereo_offset, [0.0; 2]);
    assert_eq!(a.position, [-POSITION_LIMIT; 2]);
    a.adjust(
        Mode::Position,
        [[1.0, -1.0], [0.0; 2]],
        100.0,
        Stereo::SideBySide,
    );
    assert_eq!(a.stereo_offset, [0.1, -0.1]);
}
