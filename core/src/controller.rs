pub use self::{keyboard::*, mouse::*};

pub mod mouse;
pub mod keyboard;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputEvent {
    MouseDown(MouseDown),
    MouseScroll(MouseScroll),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    Char(char),
}

impl InputEvent {
    pub fn mouse_down(pos: MousePos, button: MouseButton) -> Self {
        Self::MouseDown(MouseDown { pos, button })
    }

    pub fn mouse_scroll(scroll: MouseScroll) -> Self {
        Self::MouseScroll(scroll)
    }

    pub fn key_down(event: KeyboardEvent) -> Self {
        Self::KeyDown(event)
    }

    pub fn key_up(event: KeyboardEvent) -> Self {
        Self::KeyUp(event)
    }

    pub fn char(ch: char) -> Self {
        Self::Char(ch)
    }
}