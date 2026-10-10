use crate::{
    measure::Sample,
    options::Options,
    thumbnail_csv::{HEADER, Request},
};
use matineevr::{
    media_trace::{NativeTrace, SeekTrace},
    presentation::{Projection, Stereo, StereoFormat},
};
use std::{
    collections::HashMap,
    process::Command,
    time::{Duration, Instant},
};

fn request() -> Request {
    Request {
        scenario: "thumbnail",
        run: 1,
        index: 1,
        operation: "seek",
        target: Some(123.4),
        repeated: false,
        started: Instant::now(),
    }
}

#[test]
fn thumbnail_arguments_require_playback_targets_and_share_projection_parsing() {
    let parse = |args: &str| Options::parse(args.split_whitespace().map(str::to_owned));
    let options = parse("video 1.25 3600.5 123.4 5000.25 --thumbnails --projection fisheye190 --stereo sbs --sbs-format half --seconds 1 --repeat 3").unwrap().unwrap();
    assert!(options.thumbnails);
    assert_eq!(options.targets, [1.25, 3600.5, 123.4, 5000.25]);
    assert_eq!(options.presentation.projection, Projection::Fisheye(190));
    assert_eq!(options.presentation.stereo, Stereo::SideBySide);
    assert_eq!(options.presentation.sbs_format, StereoFormat::Half);
    let p = parse("video --tb-format half")
        .unwrap()
        .unwrap()
        .presentation;
    assert_eq!(p.tb_format, StereoFormat::Half);
    assert_eq!(p.sbs_format, StereoFormat::Full);
    assert_eq!(options.repeat, 3);
    for args in [
        "video --thumbnails",
        "video 1 --thumbnails --decoder",
        "video 1 --thumbnails --render 64 64",
        "video 1 --projection unknown",
        "video 1 --sbs-format",
        "video 1 --sbs-format unknown",
        "video 1 --tb-format",
        "video 1 --tb-format unknown",
    ] {
        assert!(parse(args).is_err(), "{args}");
    }
}

#[test]
fn csv_preserves_units_handoff_and_missing_measurements() {
    let request = request();
    let sample = Sample {
        elapsed: Duration::from_millis(20),
        preview: Some(Duration::from_millis(14)),
        preview_ready: Some(Duration::from_millis(19)),
        preview_pts: Some(123.0),
        preview_gpu: 1.5,
        trace: Some(SeekTrace {
            requested: request.started,
            released: request.started + Duration::from_millis(2),
            worker: request.started + Duration::from_millis(3),
            published: request.started + Duration::from_millis(10),
            native: NativeTrace {
                seek_us: 1500,
                packets: 42,
                packet_bytes: 123456,
                ..Default::default()
            },
        }),
        consumed: Some(request.started + Duration::from_millis(14)),
        ..Default::default()
    };
    for (status, sample) in [("ok", Some(&sample)), ("error", None), ("timeout", None)] {
        let values = request.fields(status, sample);
        assert_eq!(values.len(), HEADER.split(',').count());
        let row: HashMap<_, _> = HEADER
            .split(',')
            .zip(values.iter().map(String::as_str))
            .collect();
        assert_eq!(row["status"], status);
        assert_eq!(row["cache"], "uncontrolled");
        if status == "ok" {
            for (key, expected) in [
                ("elapsed_ms", "20.000000"),
                ("container_seek_ms", "1.500000"),
                ("handoff_ms", "4.000000"),
                ("packets", "42"),
                ("packet_bytes", "123456"),
                ("gpu_ms", "1.500000"),
            ] {
                assert_eq!(row[key], expected);
            }
        } else {
            assert_eq!(row["gpu_ms"], "");
            assert_eq!(row["packets"], "");
        }
    }
}

#[test]
fn watchdog_reports_a_stalled_operation_and_exits() {
    if std::env::var_os("FRAME_PROFILE_TIMEOUT_TEST").is_some() {
        request()
            .bounded::<()>(Duration::from_millis(20), || {
                loop {
                    std::thread::park();
                }
            })
            .unwrap();
        return;
    }
    let mut command = Command::new("timeout");
    command.arg("10s");
    if cfg!(target_arch = "aarch64")
        && let Ok(runner) = std::env::var("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER")
    {
        command.args(runner.split_whitespace());
    }
    let output = command
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "thumbnail_tests::watchdog_reports_a_stalled_operation_and_exits",
            "--nocapture",
        ])
        .env("FRAME_PROFILE_TIMEOUT_TEST", "1")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(124), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let row = stdout
        .lines()
        .find(|line| line.starts_with("1,1,seek,"))
        .unwrap();
    assert_eq!(row.split(',').count(), HEADER.split(',').count());
    assert_eq!(row.split(',').nth(6), Some("timeout"));
}
