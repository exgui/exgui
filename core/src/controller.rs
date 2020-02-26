pub use self::mouse::*;

pub mod mouse;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    MouseDown(MouseDown),
}

impl InputEvent {
    pub fn mouse_down(pos: MousePos, button: MouseButton) -> Self {
        Self::MouseDown(MouseDown { pos, button })
    }
}