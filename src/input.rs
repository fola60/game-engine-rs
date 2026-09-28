use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Default)]
pub(crate) struct KeyboardState {
    down: HashSet<KeyCode>,
    pressed: HashSet<KeyCode>,
}

impl KeyboardState {
    pub(crate) fn set_key(&mut self, key: KeyCode, down: bool) {
        if down {
            if self.down.insert(key) {
                self.pressed.insert(key);
            }
        } else {
            self.down.remove(&key);
        }
    }

    pub(crate) fn key_down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    pub(crate) fn key_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub(crate) fn finish_frame(&mut self) {
        self.pressed.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.down.clear();
        self.pressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presses_last_one_frame_ignore_repeats_and_clear_on_focus_loss() {
        let mut keys = KeyboardState::default();
        keys.set_key(KeyCode::KeyA, true);
        assert!(keys.key_down(KeyCode::KeyA) && keys.key_pressed(KeyCode::KeyA));
        keys.finish_frame();
        keys.set_key(KeyCode::KeyA, true);
        assert!(keys.key_down(KeyCode::KeyA));
        assert!(!keys.key_pressed(KeyCode::KeyA));
        keys.set_key(KeyCode::KeyA, false);
        keys.set_key(KeyCode::KeyA, true);
        keys.set_key(KeyCode::KeyA, false);
        assert!(!keys.key_down(KeyCode::KeyA));
        assert!(keys.key_pressed(KeyCode::KeyA));
        keys.set_key(KeyCode::KeyD, true);
        keys.clear();
        assert!(!keys.key_down(KeyCode::KeyD));
        assert!(!keys.key_pressed(KeyCode::KeyD));
        assert!(!keys.key_pressed(KeyCode::KeyA));
    }
}
