use super::Playback;
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

fn advance_to(player: &mut Playback, position: f64) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        assert!(Instant::now() < deadline, "Audio/video playback stalled");
        crate::playback_render::synchronize(Some(player), &mut []).unwrap();
        if let Some(frame) = player.due_frame().unwrap() {
            if let Some(clock) = player.worker.clock() {
                assert!(frame.pixels.pts <= clock + 0.05, "Video ran ahead of audio");
            }
            if frame.pixels.pts >= position {
                return;
            }
        }
        thread::sleep(Duration::from_millis(2));
    }
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_AUDIO_VIDEO: four-second video with two-second audio"]
fn audio_clock_pause_seek_and_video_tail() {
    let path = PathBuf::from(std::env::var_os("FRAME_TEST_AUDIO_VIDEO").unwrap());
    let mut player = Playback::start(path);
    player.diagnostics();
    advance_to(&mut player, 0.3);
    let (mut stats, native) = player.worker.take_stats();
    assert!(native.frames > 0 && native.audio_latency_samples > 0);
    assert!(
        stats
            .report("test", 1.0)
            .contains("audio_observation_age_ms_n=")
    );
    assert!(player.worker.clock().is_some());
    player.toggle_pause();
    thread::sleep(Duration::from_millis(100));
    let paused = player.worker.clock().unwrap();
    thread::sleep(Duration::from_millis(150));
    assert_eq!(player.worker.clock().unwrap(), paused);
    assert!(player.due_frame().unwrap().is_none());
    player.set_suspended(true);
    player.toggle_pause();
    for target in [1.0, 0.25, 1.25] {
        player.seek(target).unwrap();
        advance_to(&mut player, target);
        assert_eq!(player.status(), "Paused");
        assert_eq!(player.worker.clock().unwrap(), target);
    }
    player.set_suspended(false);
    advance_to(&mut player, 3.0);
    assert!(
        player.worker.clock().is_none(),
        "Audio did not drain before video tail"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while !player.finished() {
        assert!(Instant::now() < deadline);
        player.due_frame().unwrap();
        thread::sleep(Duration::from_millis(2));
    }
    player.seek(0.25).unwrap();
    advance_to(&mut player, 0.25);
    player.stop().join().unwrap();
}
