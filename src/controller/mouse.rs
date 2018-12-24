use egml::{ModelComponent, Node, Comp, ChangeView};
use controller::InputEvent;

pub struct MouseInput {
    last_mouse_pos: Option<(f64, f64)>,
    last_offset: Option<(f64, f64)>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct MousePos {
    pub x: f32,
    pub y: f32,
}

impl MouseInput {
    pub fn new() -> Self {
        MouseInput {
            last_mouse_pos: None,
            last_offset: None,
        }
    }

    pub fn update_pos(&mut self, x_pos: f64, y_pos: f64) {
        if self.last_mouse_pos.is_none() {
            self.last_mouse_pos = Some((x_pos, y_pos));
        }

        let (x_last, y_last) = self.last_mouse_pos.unwrap();
        let x_offset = x_pos - x_last;
        let y_offset = y_last - y_pos; // reversed since y-coordinates go from bottom to top

        self.last_mouse_pos = Some((x_pos, y_pos));
        self.last_offset = Some((x_offset, y_offset));
    }

    pub fn last_pos(&self) -> MousePos {
        if let Some((x, y)) = self.last_mouse_pos {
            MousePos { x: x as f32, y: y as f32 }
        } else {
            MousePos { x: 0.0, y: 0.0 }
        }
    }

    pub fn left_pressed<MC: ModelComponent>(&self, model: &mut MC, node: &mut Node<MC>) -> ChangeView {
        let pos = self.last_pos();
        node.input(InputEvent::MousePress(pos), model)
    }

    pub fn left_pressed_comp(&self, comp: &mut Comp) -> ChangeView {
        let pos = self.last_pos();
        comp.input(InputEvent::MousePress(pos))
    }
}