use crate::{
    Fill, KeyboardEvent, Listener, Model, MouseDown, MouseScroll, Node, On, Real, RealValue, Stroke, Transform,
};

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
    fn transparency(self, transparency: impl Into<Real>) -> Self;
    fn stroke(self, stroke: impl Into<Stroke>) -> Self;
    fn fill(self, fill: impl Into<Fill>) -> Self;
    fn remove_stroke(self) -> Self;
    fn remove_fill(self) -> Self;
    fn clip(
        self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>,
        height: impl Into<RealValue>,
    ) -> Self;
}

pub trait EventHandler<M: Model>: Sized {
    fn add_listener(&mut self, listener: Listener<M>);

    fn on_click(self, _trigger: fn(()) -> M::Message) -> Self {
        self
    }

    fn on_mouse_down(mut self, trigger: fn(On<M, MouseDown>) -> M::Message) -> Self {
        self.add_listener(Listener::OnMouseDown(trigger));
        self
    }

    fn on_mouse_scroll(mut self, trigger: fn(On<M, MouseScroll>) -> M::Message) -> Self {
        self.add_listener(Listener::OnMouseScroll(trigger));
        self
    }

    fn on_key_down(mut self, trigger: fn(On<M, KeyboardEvent>) -> M::Message) -> Self {
        self.add_listener(Listener::OnKeyDown(trigger));
        self
    }

    fn on_key_up(mut self, trigger: fn(On<M, KeyboardEvent>) -> M::Message) -> Self {
        self.add_listener(Listener::OnKeyUp(trigger));
        self
    }

    fn on_input_char(mut self, trigger: fn(On<M, char>) -> M::Message) -> Self {
        self.add_listener(Listener::OnInputChar(trigger));
        self
    }

    fn on_blur(mut self, trigger: fn(On<M, MouseDown>) -> M::Message) -> Self {
        self.add_listener(Listener::OnBlur(trigger));
        self
    }
}
