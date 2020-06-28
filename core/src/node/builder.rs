use crate::{MouseDown, Node, Model, Stroke, Fill, Transform, Listener, KeyboardEvent};

pub trait Builder<M: Model> {
    fn build(self) -> Node<M>;
}

impl<M: Model> Builder<M> for Node<M> {
    fn build(self) -> Node<M> {
        self
    }
}

pub trait Entity {
    fn id(self, id: impl Into<String>) -> Self;
    fn transform(self, transform: impl Into<Transform>) -> Self;
}

pub trait Primitive<M: Model> {
    fn child(self, child: impl Builder<M>) -> Self;
    fn children(self, children: impl IntoIterator<Item = Node<M>>) -> Self;
    fn stroke(self, stroke: impl Into<Stroke>) -> Self;
    fn fill(self, fill: impl Into<Fill>) -> Self;
    fn remove_stroke(self) -> Self;
    fn remove_fill(self) -> Self;
}

pub trait EventHandler<Msg>: Sized {
    fn add_listener(&mut self, listener: Listener<Msg>);

    fn on_click(self, _trigger: fn(()) -> Msg) -> Self {
        self
    }

    fn on_mouse_down(mut self, trigger: fn(MouseDown) -> Msg) -> Self {
        self.add_listener(Listener::OnMouseDown(trigger));
        self
    }

    fn on_key_down(mut self, trigger: fn(KeyboardEvent) -> Msg) -> Self {
        self.add_listener(Listener::OnKeyDown(trigger));
        self
    }

    fn on_key_up(mut self, trigger: fn(KeyboardEvent) -> Msg) -> Self {
        self.add_listener(Listener::OnKeyUp(trigger));
        self
    }

    fn on_input_char(mut self, trigger: fn(char) -> Msg) -> Self {
        self.add_listener(Listener::OnInputChar(trigger));
        self
    }
}
