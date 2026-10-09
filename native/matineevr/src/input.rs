use crate::navigation::{Navigation, Repeat};
use anyhow::Result;
use openxr as xr;
use std::time::Instant;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Controls {
    pub x: bool,
    pub a: bool,
    pub b: bool,
    pub y: bool,
    pub grips: [bool; 2],
    pub triggers: [f32; 2],
    pub sticks: [[f32; 2]; 2],
    pub dpad: Navigation,
    pub held_vertical: i8,
    pub held_horizontal: [i8; 2],
    pub stick: Navigation,
}

pub struct Input {
    set: xr::ActionSet,
    actions: [xr::Action<bool>; 8],
    grips: [xr::Action<f32>; 2],
    poses: [xr::Action<xr::Posef>; 2],
    hands: [xr::Space; 2],
    controller_spaces: [xr::Space; 2],
    pub controller_poses: [Option<xr::Posef>; 2],
    pub panel_poses: [Option<xr::Posef>; 2],
    triggers: [xr::Action<f32>; 2],
    sticks: [xr::Action<xr::Vector2f>; 2],
    repeat: Repeat,
}

impl Input {
    pub fn new(
        instance: &xr::Instance,
        session: &xr::Session<xr::Vulkan>,
        offset: xr::Posef,
    ) -> Result<Self> {
        let set = instance.create_action_set("player", "Video player", 0)?;
        let actions = [
            set.create_action("previous_pause", "Previous / Pause", &[])?,
            set.create_action("open_recenter", "Open / Recenter", &[])?,
            set.create_action("back", "Stop / Parent folder", &[])?,
            set.create_action("next_hud", "Next / Show performance", &[])?,
            set.create_action("up", "Up", &[])?,
            set.create_action("down", "Down", &[])?,
            set.create_action("left", "Parent folder / Previous value", &[])?,
            set.create_action("right", "Open / Next value", &[])?,
        ];
        let paths = [
            "/user/hand/right/input/x/click",
            "/user/hand/right/input/a/click",
            "/user/hand/right/input/b/click",
            "/user/hand/right/input/y/click",
            "/user/hand/left/input/dpad_up/click",
            "/user/hand/left/input/dpad_down/click",
            "/user/hand/left/input/dpad_left/click",
            "/user/hand/left/input/dpad_right/click",
        ];
        let grips = [
            set.create_action("left_shortcut", "Left shortcut panel", &[])?,
            set.create_action("right_shortcut", "Right shortcut panel", &[])?,
        ];
        let poses = [
            set.create_action::<xr::Posef>("left_hand", "Left hand pose", &[])?,
            set.create_action::<xr::Posef>("right_hand", "Right hand pose", &[])?,
        ];
        let hands = [
            poses[0].create_space(session, xr::Path::NULL, offset)?,
            poses[1].create_space(session, xr::Path::NULL, offset)?,
        ];
        let triggers = [
            set.create_action("left_adjust", "Orientation / Zoom", &[])?,
            set.create_action("right_adjust", "Stereo / Position", &[])?,
        ];
        let sticks = [
            set.create_action("left_stick", "Left adjustment", &[])?,
            set.create_action("navigate", "Navigate / Seek / Adjust", &[])?,
        ];
        let mut bindings: Vec<_> = actions
            .iter()
            .zip(paths)
            .map(|(action, path)| Ok(xr::Binding::new(action, instance.string_to_path(path)?)))
            .collect::<Result<_>>()?;
        for (hand, name) in ["left", "right"].iter().enumerate() {
            let path = |field| instance.string_to_path(&format!("/user/hand/{name}/input/{field}"));
            bindings.extend([
                xr::Binding::new(&grips[hand], path("squeeze/value")?),
                xr::Binding::new(&poses[hand], path("grip/pose")?),
                xr::Binding::new(&triggers[hand], path("trigger/value")?),
                xr::Binding::new(&sticks[hand], path("thumbstick")?),
            ]);
        }
        instance.suggest_interaction_profile_bindings(
            instance.string_to_path("/interaction_profiles/valve/frame_controller_valve")?,
            &bindings,
        )?;
        session.attach_action_sets(&[&set])?;
        let controller_spaces = [
            poses[0].create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?,
            poses[1].create_space(session, xr::Path::NULL, xr::Posef::IDENTITY)?,
        ];
        Ok(Self {
            set,
            actions,
            grips,
            poses,
            hands,
            controller_spaces,
            controller_poses: [None; 2],
            panel_poses: [None; 2],
            triggers,
            sticks,
            repeat: Repeat::default(),
        })
    }

    pub fn poll(
        &mut self,
        session: &xr::Session<xr::Vulkan>,
        base: &xr::Space,
        time: xr::Time,
    ) -> Result<Controls> {
        session.sync_actions(&[(&self.set).into()])?;
        let mut pressed = [false; 8];
        let mut held = [false; 8];
        for ((action, pressed), held) in self.actions.iter().zip(&mut pressed).zip(&mut held) {
            let state = action.state(session, xr::Path::NULL)?;
            *held = state.is_active && state.current_state;
            *pressed = *held && state.changed_since_last_sync;
        }
        let [x, a, b, y, up, down, left, right] = pressed;
        let [_, _, _, _, held_up, held_down, held_left, held_right] = held;
        let mut grips = [false; 2];
        self.panel_poses = [None; 2];
        for (hand, (action, held)) in self.grips.iter().zip(&mut grips).enumerate() {
            let state = action.state(session, xr::Path::NULL)?;
            self.panel_poses[hand] = crate::input_tracking::locate(
                &self.poses[hand], session, &self.hands[hand], base, time)?;
            self.controller_poses[hand] = crate::input_tracking::locate(
                &self.poses[hand], session, &self.controller_spaces[hand], base, time)?;
            *held =
                state.is_active && state.current_state > 0.0 && self.panel_poses[hand].is_some();
        }
        let mut triggers = [0.0; 2];
        let mut sticks = [[0.0; 2]; 2];
        for hand in 0..2 {
            let trigger = self.triggers[hand].state(session, xr::Path::NULL)?;
            let stick = self.sticks[hand].state(session, xr::Path::NULL)?;
            if trigger.is_active {
                triggers[hand] = trigger.current_state;
            }
            if stick.is_active {
                sticks[hand] = [stick.current_state.x, stick.current_state.y];
            }
        }
        let direction = Navigation::stick(sticks[1][0], sticks[1][1]);
        Ok(Controls {
            x,
            a,
            b,
            y,
            grips,
            triggers,
            sticks,
            dpad: Navigation {
                horizontal: i8::from(right) - i8::from(left),
                vertical: i8::from(down) - i8::from(up),
            },
            held_vertical: i8::from(held_down) - i8::from(held_up),
            held_horizontal: [
                i8::from(held_right) - i8::from(held_left),
                direction.horizontal,
            ],
            stick: self.repeat.update(direction, Instant::now(), grips[1]),
        })
    }
}
