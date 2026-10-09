use anyhow::Result;
use ash::vk::{self, Handle};
use matineevr::{graphics::Graphics, renderer::Target, vk_readback};
use std::rc::Rc;
use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    path::Path,
};

pub fn save(device: &Rc<Graphics>, path: &Path, target: Target) -> Result<()> {
    let pixels = vk_readback::read(device, vk::Image::from_raw(target.image), target.size)?;
    let row = target.size.0 as usize * 4;
    let rows: Vec<_> = pixels.chunks_exact(row).rev().flatten().copied().collect();
    save_pixels(path, target.size.0, target.size.1, &rows)
}

pub fn save_pixels(path: &Path, width: i32, height: i32, pixels: &[u8]) -> Result<()> {
    let mut output = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    write!(output, "P6\n{width} {height}\n255\n")?;
    for row in pixels.chunks_exact((width * 4) as usize).rev() {
        for pixel in row.chunks_exact(4) {
            output.write_all(&pixel[..3])?;
        }
    }
    output.flush()?;
    eprintln!("Saved snapshot: {}", path.display());
    Ok(())
}
