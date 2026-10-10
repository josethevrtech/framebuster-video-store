use crate::{media::Decoder, renderer::Renderer};
use std::{
    io::Write,
    process::{Command, Stdio},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn upload_luma(renderer: &mut Renderer, size: [i32; 2], luma: &[u8]) {
    assert_eq!(luma.len(), (size[0] * size[1]) as usize);
    renderer.reset().unwrap();
    let path = std::env::temp_dir().join(format!(
        "matineevr-pattern-{}-{}.mp4",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut encode = Command::new("ffmpeg")
        .env_remove("LD_LIBRARY_PATH")
        .args([
            "-v",
            "error",
            "-n",
            "-f",
            "rawvideo",
            "-pixel_format",
            "gray",
            "-video_size",
        ])
        .arg(format!("{}x{}", size[0], size[1]))
        .args([
            "-i",
            "pipe:0",
            "-vf",
            "scale=128:128:flags=neighbor:out_range=full,format=yuv420p",
            "-frames:v",
            "30",
            "-c:v",
            "libx264",
            "-qp",
            "1",
            "-color_range",
            "pc",
            "-colorspace",
            "bt709",
            "-chroma_sample_location",
            "left",
        ])
        .arg(&path)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = encode.stdin.take().unwrap();
    for _ in 0..30 {
        input.write_all(luma).unwrap();
    }
    drop(input);
    assert!(encode.wait().unwrap().success());
    let frame = Decoder::open(&path).unwrap().next_frame().unwrap().unwrap();
    renderer.upload(Rc::new(frame)).unwrap();
}
