#[path = "verify/dmabuf_check.rs"]
mod dmabuf_check;
#[path = "verify/dmabuf_color.rs"]
mod dmabuf_color;
#[path = "verify/dmabuf_pipeline.rs"]
mod dmabuf_pipeline;
#[path = "verify/dmabuf_reference.rs"]
mod dmabuf_reference;
#[path = "verify/dmabuf_verify.rs"]
mod dmabuf_verify;

use anyhow::{Context, Result, ensure};
use matineevr::graphics::Graphics;
use std::path::PathBuf;

pub struct Options {
    file: PathBuf,
    seeks: Vec<f64>,
}

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let file = args
        .next()
        .context("Usage: verify FILE [SEEK_SECONDS ...]")?;
    let seeks = args.map(|v| v.parse()).collect::<Result<Vec<f64>, _>>()?;
    ensure!(
        seeks.iter().all(|v| v.is_finite() && *v >= 0.0),
        "Invalid seek position"
    );
    let instance = Graphics::instance()?;
    let system = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    dmabuf_verify::run(
        Graphics::new(&instance, system)?,
        &Options {
            file: file.into(),
            seeks,
        },
    )
}
