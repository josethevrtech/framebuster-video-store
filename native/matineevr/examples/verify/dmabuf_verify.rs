use super::{Options, dmabuf_check, dmabuf_reference};
use anyhow::{Context, Result};
use matineevr::media::{Decoded, Decoder};
use matineevr::vk_device::Device;
use std::rc::Rc;

pub fn run(device: Rc<Device>, options: &Options) -> Result<()> {
    let mut decoder = Decoder::open(&options.file)?;
    for target in std::iter::once(None).chain(options.seeks.iter().copied().map(Some)) {
        if let Some(target) = target {
            decoder.seek(target)?;
        }
        loop {
            decoder.reconfigure(true)?;
            let decoded = decoder.advance()?;
            let preview = matches!(decoded, Decoded::Preview(_));
            let frame = match decoded {
                Decoded::Frame(frame) | Decoded::Preview(frame) => frame,
                Decoded::Pending => continue,
                Decoded::End => anyhow::bail!("No frame at verification target"),
            };
            eprintln!(
                "Verify: requested={target:?} pts={} preview={preview}",
                frame.pixels.pts
            );
            for color in [false, true] {
                let pixels = dmabuf_check::read(device.clone(), &frame, color, || Ok(()))?;
                dmabuf_reference::compare(
                    &options.file,
                    &frame,
                    &pixels,
                    color,
                    device.limits.sub_texel_precision_bits,
                )?;
            }
            if !preview {
                let pixels = dmabuf_check::read(device.clone(), &frame, false, || {
                    for _ in 0..16 {
                        decoder
                            .next_frame()?
                            .context("Need 16 frames after target for reuse verification")?;
                    }
                    Ok(())
                })?;
                dmabuf_reference::compare(
                    &options.file,
                    &frame,
                    &pixels,
                    false,
                    device.limits.sub_texel_precision_bits,
                )?;
                break;
            }
        }
    }
    Ok(())
}
