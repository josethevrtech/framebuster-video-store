use std::{collections::HashMap, process::Command};

#[test]
#[ignore = "requires Frame, FRAME_PROFILE_BIN and an eight-second FRAME_TEST_VIDEO"]
fn rendered_cli_reports_gpu_completion_and_exits_cleanly() {
    for decoder in [false, true] {
        let mut command = Command::new(std::env::var_os("FRAME_PROFILE_BIN").unwrap());
        command
            .arg(std::env::var_os("FRAME_TEST_VIDEO").unwrap())
            .args([
                "1.25",
                "20",
                "0",
                "--seconds",
                "0.1",
                "--repeat",
                "2",
                "--render",
                "64",
                "64",
            ]);
        if decoder {
            command.arg("--decoder");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let csv = String::from_utf8(output.stdout).unwrap();
        let mut lines = csv.lines();
        let header: Vec<_> = lines.next().unwrap().split(',').collect();
        let rows: Vec<HashMap<_, _>> = lines
            .map(|line| {
                let values: Vec<_> = line.split(',').collect();
                assert_eq!(values.len(), header.len());
                header.iter().copied().zip(values).collect()
            })
            .collect();
        for operation in ["graphics_init", "start"] {
            assert_eq!(
                rows.iter()
                    .filter(|row| row["operation"] == operation)
                    .count(),
                2
            );
        }
        assert_eq!(
            rows.iter().filter(|row| row["operation"] == "seek").count(),
            6
        );
        assert!(rows.iter().any(|row| !row["preview_pts_s"].is_empty()));
        for row in rows {
            assert_eq!(row["eye_width"], "64");
            assert_eq!(row["eye_height"], "64");
            let upload: f64 = row["import_ms"].parse().unwrap();
            let draw: f64 = row["draw_ms"].parse().unwrap();
            let frames: u64 = row["frames"].parse().unwrap();
            assert_eq!(upload > 0.0 && draw > 0.0, frames > 0);
            assert!(draw <= row["elapsed_ms"].parse::<f64>().unwrap());
            if row["operation"] == "seek" {
                let target: f64 = row["target_s"].parse().unwrap();
                if !row["preview_pts_s"].is_empty() {
                    assert!(row["preview_pts_s"].parse::<f64>().unwrap() < target + 0.05);
                    let preview = row["preview_ms"].parse::<f64>().unwrap();
                    let ready = row["preview_ready_ms"].parse::<f64>().unwrap();
                    assert!(preview > 0.0 && ready > preview);
                    assert!(ready < row["elapsed_ms"].parse::<f64>().unwrap());
                } else {
                    assert!(row["preview_ms"].is_empty() && row["preview_ready_ms"].is_empty());
                }
                if target == 20.0 {
                    assert_eq!(row["eof"], "true");
                    assert_eq!(frames, 0);
                } else {
                    let pts: f64 = row["first_pts_s"].parse().unwrap();
                    assert!((target..target + 0.05).contains(&pts));
                    assert_eq!(frames, 1);
                }
            }
        }
    }
}

#[test]
#[ignore = "requires Frame Vulkan and an eight-second FRAME_TEST_VIDEO"]
fn paused_seeks_release_rendered_frames_and_keep_the_latest_target() {
    use crate::{measure::measure, options::Options, source::Source};
    use matineevr::offscreen::Offscreen;
    use std::time::Instant;

    let options = Options::parse([std::env::var("FRAME_TEST_VIDEO").unwrap()].into_iter())
        .unwrap()
        .unwrap();
    let mut render = Some(Offscreen::new((64, 64), 2).unwrap());
    let mut source = Source::open(&options).unwrap();
    measure(
        &mut source,
        &mut render,
        Instant::now(),
        None,
        &options,
        false,
    )
    .unwrap();
    let Source::Playback(player) = &mut source else {
        unreachable!()
    };
    player.toggle_pause();
    for target in [1.25, 6.6, 0.25] {
        player.seek(target).unwrap();
    }
    let sample = measure(
        &mut source,
        &mut render,
        Instant::now(),
        None,
        &options,
        true,
    )
    .unwrap();
    assert!((0.25..0.30).contains(&sample.first.unwrap()));
    let Source::Playback(player) = &source else {
        unreachable!()
    };
    assert_eq!(player.status(), "Paused");
    render.as_mut().unwrap().synchronize(None).unwrap();
    source.stop().unwrap();
}

#[test]
#[ignore = "requires Frame, FRAME_PROFILE_BIN and an eight-second FRAME_TEST_VIDEO"]
fn thumbnails_report_first_and_repeated_positions_with_native_and_gpu_timings() {
    let output = Command::new(std::env::var_os("FRAME_PROFILE_BIN").unwrap())
        .arg(std::env::var_os("FRAME_TEST_VIDEO").unwrap())
        .args([
            "1.25",
            "0",
            "1.25",
            "--thumbnails",
            "--projection",
            "fisheye190",
            "--seconds",
            "0.1",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let csv = String::from_utf8(output.stdout).unwrap();
    let mut lines = csv.lines();
    let header: Vec<_> = lines.next().unwrap().split(',').collect();
    let lines: Vec<_> = lines.collect();
    assert_eq!(lines.len(), 6);
    for (pair, visit) in lines.chunks_exact(2).zip(["first", "first", "repeat"]) {
        let row: HashMap<_, _> = header.iter().copied().zip(pair[0].split(',')).collect();
        let target: HashMap<_, _> = header.iter().copied().zip(pair[1].split(',')).collect();
        assert_eq!(target["operation"], "seek_target");
        assert_eq!(target["request"], row["request"]);
        assert!(
            target["elapsed_ms"].parse::<f64>().unwrap()
                >= row["preview_ready_ms"].parse::<f64>().unwrap()
        );
        assert!(
            target["first_pts_s"].parse::<f64>().unwrap()
                >= row["target_s"].parse::<f64>().unwrap()
        );
        assert_eq!(row["status"], "ok");
        assert_eq!(row["visit"], visit);
        let number = |key| row[key].parse::<f64>().unwrap();
        assert!(number("preview_ready_ms") >= number("preview_ms"));
        assert!(number("preview_ms") >= number("worker_wait_ms"));
        assert!(number("worker_wait_ms") >= number("release_handshake_ms"));
        assert!(number("gpu_ms") > 0.0);
        assert!(number("decoded_frames") >= 1.0);
        assert!(number("receive_calls") >= number("decoded_frames"));
        if number("seek_calls") > 0.0 {
            assert!(number("packets") > 0.0);
        } else {
            assert_eq!(number("decoder_close_ms"), 0.0);
            assert_eq!(number("decoder_open_ms"), 0.0);
        }
    }
}
