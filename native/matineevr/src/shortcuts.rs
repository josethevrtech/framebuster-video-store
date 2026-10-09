#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelKind {
    Browser,
    Settings,
    Information,
    Seek,
    Alignment,
}

pub struct Shortcuts {
    pub assignments: [PanelKind; 2],
    pub held: [bool; 2],
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            assignments: [PanelKind::Browser, PanelKind::Settings],
            held: [false; 2],
        }
    }
}

impl Shortcuts {
    pub fn panels(&self, playing: bool, information: bool) -> [Option<PanelKind>; 3] {
        if !playing {
            return [None, None, Some(PanelKind::Browser)];
        }
        let hands = std::array::from_fn::<_, 2, _>(|hand| {
            self.held[hand].then_some(self.assignments[hand])
        });
        [
            hands[0],
            hands[1],
            (information && hands.iter().all(Option::is_none)).then_some(PanelKind::Information),
        ]
    }

    pub fn targets(panels: [Option<PanelKind>; 3]) -> [Option<PanelKind>; 2] {
        let dpad = if panels.contains(&Some(PanelKind::Browser)) {
            Some(PanelKind::Browser)
        } else {
            panels.iter().flatten().copied().next()
        };
        [dpad, panels[1]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use PanelKind::*;

    #[test]
    fn holds_visibility_routing_and_hand_assignments() {
        let mut shortcuts = Shortcuts::default();
        assert_eq!(shortcuts.panels(false, false), [None, None, Some(Browser)]);
        assert_eq!(
            shortcuts.panels(true, true),
            [None, None, Some(Information)]
        );
        shortcuts.held = [false, true];
        let panels = shortcuts.panels(true, false);
        assert_eq!(panels, [None, Some(Settings), None]);
        assert_eq!(Shortcuts::targets(panels), [Some(Settings), Some(Settings)]);
        shortcuts.held = [true, true];
        assert_eq!(
            Shortcuts::targets(shortcuts.panels(true, false)),
            [Some(Browser), Some(Settings)]
        );
        shortcuts.held = [true, false];
        assert_eq!(shortcuts.panels(true, false), [Some(Browser), None, None]);
        shortcuts.held = [false, false];
        assert_eq!(shortcuts.panels(true, false), [None; 3]);
        shortcuts.assignments.swap(0, 1);
        shortcuts.held = [true, true];
        assert_eq!(
            shortcuts.panels(true, false),
            [Some(Settings), Some(Browser), None]
        );
        assert_eq!(
            Shortcuts::targets(shortcuts.panels(true, false)),
            [Some(Browser), Some(Browser)]
        );
    }
}
