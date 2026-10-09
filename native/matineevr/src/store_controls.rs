use crate::input::Controls;

pub fn normalize(mut controls: Controls) -> Controls {
    for stick in &mut controls.sticks { for axis in stick { *axis = -*axis; } }
    controls
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_store_axes_are_corrected_without_changing_buttons() {
        let mut raw = Controls::default();
        raw.sticks = [[-0.4,-0.9],[-0.7,0.8]]; raw.a = true; raw.sprint = true;
        let mapped = normalize(raw);
        assert_eq!(mapped.sticks,[[0.4,0.9],[0.7,-0.8]]);
        assert!(mapped.a && mapped.sprint);
    }
}
