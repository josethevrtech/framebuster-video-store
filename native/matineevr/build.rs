use std::{env, path::PathBuf, process::Command};

fn media_flags(option: &str) -> Vec<String> {
    let output = Command::new("pkg-config")
        .args([
            option,
            "libavformat",
            "libavcodec",
            "libavutil",
            "libswresample",
            "libpulse",
        ])
        .output()
        .expect("Run pkg-config for target FFmpeg and PulseAudio");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn main() {
    for shader in ["video.vert", "video.frag", "dmabuf_check.comp", "controller.vert", "controller.frag"] {
        let source = format!("shaders/{shader}");
        println!("cargo:rerun-if-changed={source}");
        let output = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(format!("{shader}.spv"));
        let status = Command::new("glslc")
            .args(["-O", "-std=450", "--target-env=vulkan1.3"])
            .arg(&source)
            .arg("-o")
            .arg(output)
            .status()
            .expect("Run glslc");
        assert!(status.success(), "Compile {source}");
    }
    for path in [
        ".git/HEAD",
        ".git/index",
        ".git/refs",
        "src",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
    let version = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8(output.stdout).unwrap())
        .unwrap_or_else(|| {
            format!(
                "{} (Git version unavailable)",
                env::var("CARGO_PKG_VERSION").unwrap()
            )
        });
    println!("cargo:rustc-env=MATINEEVR_VERSION={}", version.trim());
    println!("cargo:rerun-if-changed=native/media.h");
    println!("cargo:rerun-if-changed=native/media_trace.h");
    println!("cargo:rerun-if-changed=native/media_internal.h");
    println!("cargo:rerun-if-changed=native/media_reader.h");
    println!("cargo:rerun-if-changed=native/audio_internal.h");
    println!("cargo:rerun-if-changed=native/audio_output.h");
    for variable in [
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_SYSROOT_DIR",
        "PKG_CONFIG_PATH",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    let compiler = if env::var("TARGET").unwrap() == "aarch64-unknown-linux-gnu" {
        "steam-frame-cc"
    } else {
        "cc"
    };
    for source in [
        "media",
        "media_frame",
        "media_seek",
        "media_trace",
        "media_packets",
        "media_reader",
        "media_metadata",
        "audio",
        "audio_decode",
        "audio_output",
        "audio_control",
    ] {
        println!("cargo:rerun-if-changed=native/{source}.c");
        let object = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(format!("{source}.o"));
        let status = Command::new(compiler)
            .args(["-O2", "-Wall", "-Wextra", "-Werror", "-fPIC", "-c"])
            .arg(format!("native/{source}.c"))
            .arg("-o")
            .arg(&object)
            .args(media_flags("--cflags"))
            .status()
            .expect("Run the target C compiler");
        assert!(status.success(), "Compile native media bridge");
        println!("cargo:rustc-link-arg={}", object.display());
    }
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/ffmpeg");
    for flag in media_flags("--libs") {
        if flag == "-pthread" {
            println!("cargo:rustc-link-lib=pthread");
        } else {
            println!("cargo:rustc-link-arg={flag}");
        }
    }
}
