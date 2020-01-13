pub use self::mouse::*;

pub mod mouse;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    MousePress(MousePress),
}

impl InputEvent {
    pub fn mouse_press(pos: MousePos) -> Self {
        Self::MousePress(MousePress { pos })
    }
}