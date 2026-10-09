use anyhow::{Context, Result, ensure};
use matineevr::vk_memory::Buffer;
use crate::graphics::Graphics;
use std::rc::Rc;

pub struct ControllerMesh {
    pub buffer: Buffer,
    pub counts: [u32; 2],
}

impl ControllerMesh {
    pub fn new(device: Rc<Graphics>) -> Result<Self> {
        let path = std::env::var_os("HALCYON_FRAME_CONTROLLER_MESH")
            .context("Controller model cache path is missing")?;
        let data = std::fs::read(path).context("Read Steam controller models")?;
        ensure!(data.len() >= 16 && &data[..8] == b"HFCM0001", "Invalid controller model cache");
        let counts = [u32::from_le_bytes(data[8..12].try_into()?),
            u32::from_le_bytes(data[12..16].try_into()?)];
        ensure!(counts.iter().all(|n| *n > 0 && *n <= 500_000 && n % 3 == 0),
            "Invalid controller vertex counts");
        let size = (counts[0] as usize + counts[1] as usize) * 48;
        ensure!(data.len() == 16 + size, "Truncated controller model cache");
        ensure!(data[16..].chunks_exact(4).all(|bytes|
            f32::from_le_bytes(bytes.try_into().unwrap()).is_finite()), "Non-finite controller vertex");
        let buffer = Buffer::new(device, size)?;
        unsafe { std::ptr::copy_nonoverlapping(data[16..].as_ptr(), buffer.pointer, size); }
        eprintln!("Steam controller models: {} and {} triangles", counts[0] / 3, counts[1] / 3);
        Ok(Self { buffer, counts })
    }
}
