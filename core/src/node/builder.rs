use super::{Node, Model, Stroke, Fill, Transform};

pub trait Builder<M: Model> {
    fn build(self) -> Node<M>;
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
}

pub trait EventHandler<Msg> {
    fn on_click(self, _trigger: impl Fn(()) -> Msg) -> Self where Self: Sized {
        self
    }
}
