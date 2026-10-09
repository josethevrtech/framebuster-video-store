use crate::{
    hud_text::{Canvas, HEIGHT, WIDTH},
    media::Frame,
    metrics::Metrics,
    performance::Performance,
    playback::Playback,
    snapshot,
};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

pub fn due_frame(player: &mut Playback) -> anyhow::Result<Option<Frame>> {
    matineevr::playback_render::synchronize(Some(player), &mut [])?;
    player.due_frame()
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_VIDEO"]
fn native_playback_position_pause_and_end() {
    let path = PathBuf::from(std::env::var_os("FRAME_TEST_VIDEO").expect("FRAME_TEST_VIDEO"));
    let mut player = Playback::start(path.clone());
    let mut stats = Performance::new(false);
    let mut metrics = Metrics::new();
    let timeout = Instant::now() + Duration::from_secs(30);
    loop {
        assert!(Instant::now() < timeout, "Decoder did not produce a frame");
        if let Some(frame) = crate::playback_tests::due_frame(&mut player).unwrap() {
            stats.size = (frame.pixels.width, frame.pixels.height);
            break;
        }
        thread::sleep(Duration::from_millis(2));
    }
    let length = player.length.expect("Fixture needs a known duration");
    assert!(length > 0.0 && length < 20.0, "Use a short fixture");
    let position = player.position;
    player.toggle_pause();
    thread::sleep(Duration::from_millis(250));
    assert!(
        crate::playback_tests::due_frame(&mut player)
            .unwrap()
            .is_none()
    );
    assert_eq!(player.position, position);
    assert_eq!(player.status(), "Paused");
    player.toggle_pause();
    while player.status() != "Ended" {
        assert!(Instant::now() < timeout, "Playback did not finish");
        let previous = player.position;
        crate::playback_tests::due_frame(&mut player).unwrap();
        assert!(player.position >= previous);
        thread::sleep(Duration::from_millis(2));
    }
    assert!(player.position <= length && player.position > length - 1.0);
    let final_position = player.position;
    thread::sleep(Duration::from_millis(200));
    assert!(
        crate::playback_tests::due_frame(&mut player)
            .unwrap()
            .is_none()
    );
    assert_eq!(player.position, final_position);
    metrics.refresh();
    thread::sleep(Duration::from_millis(550));
    metrics.refresh();
    assert!(metrics.latest.cpu.is_some());
    assert!(metrics.latest.memory.is_some());
    if let Some(output_path) = std::env::var_os("FRAME_TEST_HUD_SNAPSHOT") {
        let mut canvas = Canvas::new(&path, [WIDTH, HEIGHT]);
        canvas.update(&player, &stats, metrics.latest, 0.0);
        snapshot::save_pixels(
            &PathBuf::from(output_path),
            crate::hud_text::WIDTH as i32,
            crate::hud_text::HEIGHT as i32,
            &canvas.pixels,
        )
        .unwrap();
    }
    eprintln!(
        "Verified paused and EOF position; end={:.6}, length={length:.6}, GPU={:?}",
        player.position, metrics.latest.gpu
    );
    player.stop().join().unwrap();
}

fn decoder_handles() -> usize {
    let device = std::fs::canonicalize("/dev/video-dec0").unwrap();
    std::fs::read_dir("/proc/self/fd")
        .unwrap()
        .filter_map(|entry| std::fs::read_link(entry.ok()?.path()).ok())
        .filter(|path| path == &device)
        .count()
}

#[path = "../examples/verify/dmabuf_check.rs"]
mod dmabuf_check;
#[path = "../examples/verify/dmabuf_pipeline.rs"]
mod dmabuf_pipeline;

pub fn pixels(frame: &Frame) -> Vec<u8> {
    use matineevr::graphics::Graphics;
    let instance = Graphics::instance().unwrap();
    let system = instance
        .system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)
        .unwrap();
    let device = Graphics::new(&instance, system).unwrap();
    dmabuf_check::read(device, frame, false, || Ok(()))
        .unwrap()
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect()
}

#[test]
#[ignore = "requires Frame and FRAME_TEST_VIDEO"]
fn retained_frame_keeps_decoder_and_pixels_alive_after_worker_stops() {
    let before = decoder_handles();
    let mut player = Playback::start(PathBuf::from(std::env::var_os("FRAME_TEST_VIDEO").unwrap()));
    let deadline = Instant::now() + Duration::from_secs(20);
    let frame = loop {
        assert!(Instant::now() < deadline, "First frame timed out");
        if let Some(frame) = crate::playback_tests::due_frame(&mut player).unwrap() {
            break frame;
        }
        thread::sleep(Duration::from_millis(2));
    };
    let expected = pixels(&frame);
    player.stop().join().unwrap();
    assert_eq!(decoder_handles(), before + 1);
    assert_eq!(pixels(&frame), expected);
    drop(frame);
    assert_eq!(decoder_handles(), before);
}
