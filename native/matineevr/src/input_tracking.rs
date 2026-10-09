use anyhow::Result;
use openxr as xr;

pub fn locate(
    action: &xr::Action<xr::Posef>,
    session: &xr::Session<xr::Vulkan>,
    hand: &xr::Space,
    base: &xr::Space,
    time: xr::Time,
) -> Result<Option<xr::Posef>> {
    if !action.is_active(session, xr::Path::NULL)? {
        return Ok(None);
    }
    let location = hand.locate(base, time)?;
    let valid = xr::SpaceLocationFlags::POSITION_VALID | xr::SpaceLocationFlags::ORIENTATION_VALID;
    Ok(location.location_flags.contains(valid).then_some(location.pose))
}
