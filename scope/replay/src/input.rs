use piston_window::*;

use crate::state::*;

pub fn handle<'a>(state: &mut AppState<'a>, event: &Event) {
    if let Some(b) = event.press_args() {
        if let Button::Keyboard(key) = b {
            match key {
                Key::Left => state.player_rewind(),
                Key::Right => state.player_advance(),
                _ => (),
            }
        }
    }
}
