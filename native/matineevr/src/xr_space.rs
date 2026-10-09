use anyhow::Result;
use openxr as xr;

pub fn create(session: &xr::Session<xr::Vulkan>, _store: bool) -> Result<(xr::Space, f32)> {
    let supported = session.enumerate_reference_spaces()?;
    let kind = xr::ReferenceSpaceType::LOCAL;
    eprintln!("OpenXR room reference: {kind:?}; available: {supported:?}");
    Ok((session.create_reference_space(kind, xr::Posef::IDENTITY)?, 0.0))
}
