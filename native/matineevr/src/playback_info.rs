use super::Playback;

impl Playback {
    pub fn audio_clock(&self) -> Option<f64> {
        self.worker.clock()
    }

    pub fn profile(&self) {
        self.worker.profile();
    }

    pub fn take_trace(&self) -> Option<crate::media_trace::SeekTrace> {
        self.worker.take_trace()
    }

    pub fn release_required(&mut self) -> bool {
        if !self.worker.release_required() {
            return false;
        }
        self.pending = None;
        self.preview = None;
        true
    }

    pub fn released(&self) {
        self.worker.released();
    }

    pub fn take_metadata(&self) -> Option<crate::video_settings::Hints> {
        self.worker.take_metadata()
    }

    pub fn finished(&self) -> bool {
        self.ended && self.pending.is_none()
    }

    pub fn status(&self) -> &'static str {
        if self.finished() {
            "Ended"
        } else if self.target.is_some() {
            "Seeking"
        } else if self.pause_started.is_some() {
            "Paused"
        } else {
            "Playing"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipping_outward_at_a_boundary_does_not_restart_playback() {
        let mut player = Playback::start(std::env::temp_dir().join("missing-seek-test.mp4"));
        player.length = Some(30.0);
        player.toggle_pause();
        player.skip(-10);
        assert_eq!(player.seek_target(), None);
        assert_eq!(player.status(), "Paused");
        player.position = 30.0;
        player.ended = true;
        player.skip(10);
        assert!(player.finished());
        assert_eq!(player.seek_target(), None);
        player.skip(-10);
        assert_eq!(player.seek_target(), Some(20.0));
        assert!(!player.finished());
        player.stop().join().unwrap();
    }
}
