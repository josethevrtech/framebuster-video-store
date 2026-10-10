use crate::input::Controls;

pub fn normalize(mut controls: Controls) -> Controls {
    for axis in &mut controls.sticks[1] { *axis = -*axis; }
    controls
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn right_axes_are_corrected_while_left_axes_and_buttons_stay_native() {
        let mut raw = Controls::default();
        raw.sticks = [[-0.4,-0.9],[-0.7,0.8]]; raw.a = true; raw.sprint = true;
        let mapped = normalize(raw);
        assert_eq!(mapped.sticks,[[-0.4,-0.9],[0.7,-0.8]]);
        assert!(mapped.a && mapped.sprint);
    }
}
