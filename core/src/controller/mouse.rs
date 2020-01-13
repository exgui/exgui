use crate::{Model, Node, Comp, Real, SystemMessage};
use super::InputEvent;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MousePress {
    pub pos: MousePos,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MouseInput {
    last_pos: Option<MousePos>,
    last_offset: Option<MousePos>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct MousePos {
    pub x: Real,
    pub y: Real,
}

impl MouseInput {
    pub fn new() -> Self {
        MouseInput {
            last_pos: None,
            last_offset: None,
        }
    }

    pub fn update_pos(&mut self, x: Real, y: Real) {
        let offset = self
            .last_pos
            .map(|last| MousePos {
                x: x - last.x,
                y: last.y - y, // reversed since y-coordinates go from bottom to top
            })
            .unwrap_or_default();

        self.last_pos = Some(MousePos { x, y });
        self.last_offset = Some(offset);
    }

    pub fn last_pos(&self) -> MousePos {
        self.last_pos.unwrap_or_default()
    }

    pub fn left_pressed_comp(&self, comp: &mut Comp) {
        let pos = self.last_pos();
        comp.send_system_msg(SystemMessage::Input(InputEvent::mouse_press(pos)))
    }

//    pub fn left_pressed_node<M: Model>(&self, node: &mut Node<M>) -> Vec<M::Message> {
//        let pos = self.last_pos();
//        let mut msgs = Vec::new();
//        node.input(InputEvent::MousePress(pos), &mut msgs);
//        msgs
//    }
}