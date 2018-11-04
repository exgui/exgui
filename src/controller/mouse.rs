use egml::{ModelComponent, Node, ShouldRender, event::{Event, ClickEvent}};

pub struct MouseInput {
    last_mouse_pos: Option<(f64, f64)>,
    last_offset: Option<(f64, f64)>,
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

    pub fn left_pressed<MC: ModelComponent>(&self, model: &mut MC, node: &Node<MC>) -> ShouldRender {
        let (x, y) = if let Some((x, y)) = self.last_mouse_pos {
            (x as f32, y as f32)
        } else {
            (0.0, 0.0)
        };
        Self::intersect_event(x, y, Event::Click(ClickEvent), model, node)
    }

    fn intersect_event<MC: ModelComponent>(x: f32, y: f32, event: Event, model: &mut MC, node: &Node<MC>) -> ShouldRender {
        let mut should_render = false;
        match node {
            Node::Unit(ref unit) => {
                if unit.intersect(x, y) {
                    for listener in unit.listeners.iter() {
                        if let Some(msg) = listener.handle(event) {
                            if model.update(msg) {
                                should_render = true;
                            }
                        }
                    }
                }
                for child in unit.childs.iter() {
                    if Self::intersect_event(x, y, event, model, child) {
                        should_render = true;
                    }
                }
            },
        }
        should_render
    }
}