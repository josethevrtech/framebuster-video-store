use anyhow::{Result, ensure};
use matineevr::media::Frame;
use std::{path::Path, process::Command};

pub fn compare(
    file: &Path,
    decoded: &Frame,
    pixels: &[f32],
    color: bool,
    filter_bits: u32,
) -> Result<()> {
    let frame = decoded.dmabuf();
    let ten_bit = frame.format.to_le_bytes() == *b"P010";
    let output = Command::new("ffmpeg")
        .env_remove("LD_LIBRARY_PATH")
        .args([
            "-nostdin",
            "-hide_banner",
            "-v",
            "info",
            "-hwaccel",
            "none",
            "-i",
        ])
        .arg(file)
        .args([
            "-vf",
            &format!("select=gte(t\\,{}),showinfo", decoded.pixels.pts - 0.000001),
        ])
        .args([
            "-map",
            &format!("0:{}", frame.stream),
            "-an",
            "-frames:v",
            "1",
            "-pix_fmt",
            if ten_bit { "yuv420p10le" } else { "yuv420p" },
            "-f",
            "rawvideo",
            "-",
        ])
        .output()?;
    ensure!(
        output.status.success(),
        "Software reference failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let log = String::from_utf8_lossy(&output.stderr);
    let pts: f64 = log
        .split("pts_time:")
        .nth(1)
        .and_then(|text| text.split_whitespace().next())
        .ok_or_else(|| anyhow::anyhow!("Software reference has no timestamp"))?
        .parse()?;
    ensure!(
        (pts - decoded.pixels.pts).abs() < 0.0001,
        "Reference timestamp differs: {pts}"
    );
    let width = (frame.width - frame.crop[0] - frame.crop[2]) as usize;
    let height = (frame.height - frame.crop[1] - frame.crop[3]) as usize;
    let chroma = width.div_ceil(2) * height.div_ceil(2);
    let bytes = if ten_bit { 2 } else { 1 };
    ensure!(
        output.stdout.len() == (width * height + 2 * chroma) * bytes,
        "Reference dimensions differ"
    );
    let sample = |index: usize| -> f32 {
        if ten_bit {
            u16::from_le_bytes(output.stdout[index * 2..index * 2 + 2].try_into().unwrap()) as f32
        } else {
            output.stdout[index] as f32
        }
    };
    let scale = if ten_bit { 1023.0 } else { 255.0 };
    let mut maximum = [0.0_f32; 3];
    let mut worst = [(0, 0, 0.0, 0.0); 3];
    let mut low_bits = 0;
    for y in 0..height {
        for x in 0..width {
            let index = ((y + frame.crop[1] as usize) * (frame.width - frame.crop[2]) as usize
                + x
                + frame.crop[0] as usize)
                * 4;
            let uv = y / 2 * width.div_ceil(2) + x / 2;
            let reference = if color {
                super::dmabuf_color::reference(
                    &decoded.pixels,
                    frame.chroma_location,
                    x,
                    y,
                    scale,
                    &sample,
                )
            } else {
                [
                    sample(y * width + x),
                    sample(width * height + uv),
                    sample(width * height + chroma + uv),
                ]
            };
            for (component, reference) in reference.into_iter().enumerate() {
                let value = if color {
                    pixels[index + component]
                } else {
                    pixels[index + [1, 2, 0][component]] * scale
                };
                ensure!(value.is_finite(), "Non-finite GPU readback");
                if (value - reference).abs() > maximum[component] {
                    maximum[component] = (value - reference).abs();
                    worst[component] = (x, y, reference, value);
                }
                low_bits += usize::from(ten_bit && reference as u16 & 3 != 0);
            }
        }
    }
    eprintln!(
        "Independent pixel check: color={color} max_error={maximum:?} nonzero_low_bits={low_bits}"
    );
    eprintln!("Worst pixels (x, y, reference, GPU): {worst:?}");
    ensure!(
        maximum.into_iter().all(|error| error
            <= if color {
                2.0 / scale + 2.0_f32.powi(1 - filter_bits as i32)
            } else {
                1.0
            }),
        "Hardware/GPU samples differ from software reference"
    );
    ensure!(
        color || !ten_bit || low_bits > 0,
        "Reference does not exercise ten-bit precision"
    );
    Ok(())
}
