pub mod mouse;

pub use self::mouse::*;

pub enum InputEvent {
    MousePress(MousePos),
}