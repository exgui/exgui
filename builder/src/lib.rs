use std::borrow::Cow;

use exgui_core::{Model, Node, Prim, Shape, Rect, Circle, Text, Group, Path, PathCommand, RealValue, Stroke, Fill, AlignHor, AlignVer, Transform, Comp};
pub use exgui_core::builder::*;

pub fn circle<M: Model>() -> CircleBuilder<M> {
    CircleBuilder {
        shape: Default::default(),
        children: Default::default(),
    }
}

pub struct CircleBuilder<M: Model> {
    shape: Circle,
    children: Vec<Node<M>>,
}

impl<M: Model> CircleBuilder<M> {
    pub fn center(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>) -> Self {
        self.shape.cx = x.into();
        self.shape.cy = y.into();
        self
    }

    pub fn radius(mut self, r: impl Into<RealValue>) -> Self {
        self.shape.r = r.into();
        self
    }
}

impl<M: Model> Builder<M> for CircleBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Circle::NAME),
            Shape::Circle(self.shape),
            self.children
        ))
    }
}

impl<M: Model> Entity for CircleBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = Some(transform.into());
        self
    }
}

impl<M: Model> Primitive<M> for CircleBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.children.extend(children);
        self
    }

    fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.shape.stroke = Some(stroke.into());
        self
    }

    fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.shape.fill = Some(fill.into());
        self
    }
}

impl<M: Model> EventHandler<M::Message> for CircleBuilder<M> {
}


pub fn rect<M: Model>() -> RectBuilder<M> {
    RectBuilder {
        shape: Default::default(),
        children: Default::default(),
    }
}

pub struct RectBuilder<M: Model> {
    shape: Rect,
    children: Vec<Node<M>>,
}

impl<M: Model> RectBuilder<M> {
}

impl<M: Model> Builder<M> for RectBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Rect::NAME),
            Shape::Rect(self.shape),
            self.children
        ))
    }
}

impl<M: Model> Entity for RectBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = Some(transform.into());
        self
    }
}

impl<M: Model> Primitive<M> for RectBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.children.extend(children);
        self
    }

    fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.shape.stroke = Some(stroke.into());
        self
    }

    fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.shape.fill = Some(fill.into());
        self
    }
}

impl<M: Model> EventHandler<M::Message> for RectBuilder<M> {
}


pub fn text<M: Model>(content: impl Into<String>) -> TextBuilder<M> {
    TextBuilder {
        shape: Text { content: content.into(), ..Text::default() },
        children: Default::default(),
    }
}

pub struct TextBuilder<M: Model> {
    shape: Text,
    children: Vec<Node<M>>,
}

impl<M: Model> TextBuilder<M> {
    pub fn pos(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>) -> Self {
        self.shape.x = x.into();
        self.shape.y = y.into();
        self
    }

    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.shape.font_name = name.into();
        self
    }

    pub fn font_size(mut self, size: impl Into<RealValue>) -> Self {
        self.shape.font_size = size.into();
        self
    }

    pub fn align(mut self, align: impl Into<(AlignHor, AlignVer)>) -> Self {
        self.shape.align = align.into();
        self
    }
}

impl<M: Model> Builder<M> for TextBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Text::NAME),
            Shape::Text(self.shape),
            self.children
        ))
    }
}

impl<M: Model> Entity for TextBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = Some(transform.into());
        self
    }
}

impl<M: Model> Primitive<M> for TextBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.children.extend(children);
        self
    }

    fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.shape.stroke = Some(stroke.into());
        self
    }

    fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.shape.fill = Some(fill.into());
        self
    }
}

impl<M: Model> EventHandler<M::Message> for TextBuilder<M> {
}


pub fn path<M: Model>(cmd: impl Into<Vec<PathCommand>>) -> PathBuilder<M> {
    PathBuilder {
        shape: Path { cmd: cmd.into(), ..Path::default() },
        children: Default::default(),
    }
}

pub struct PathBuilder<M: Model> {
    shape: Path,
    children: Vec<Node<M>>,
}

impl<M: Model> PathBuilder<M> {
}

impl<M: Model> Builder<M> for PathBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Path::NAME),
            Shape::Path(self.shape),
            self.children
        ))
    }
}

impl<M: Model> Entity for PathBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = Some(transform.into());
        self
    }
}

impl<M: Model> Primitive<M> for PathBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.children.extend(children);
        self
    }

    fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.shape.stroke = Some(stroke.into());
        self
    }

    fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.shape.fill = Some(fill.into());
        self
    }
}

impl<M: Model> EventHandler<M::Message> for PathBuilder<M> {
}


pub fn group<M: Model>() -> GroupBuilder<M> {
    GroupBuilder {
        shape: Default::default(),
        children: Default::default()
    }
}

pub struct GroupBuilder<M: Model> {
    shape: Group,
    children: Vec<Node<M>>,
}

impl<M: Model> GroupBuilder<M> {
}

impl<M: Model> Builder<M> for GroupBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Group::NAME),
            Shape::Group(self.shape),
            self.children
        ))
    }
}

impl<M: Model> Entity for GroupBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = Some(transform.into());
        self
    }
}

impl<M: Model> Primitive<M> for GroupBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.children.extend(children);
        self
    }

    fn stroke(mut self, stroke: impl Into<Stroke>) -> Self {
        self.shape.stroke = Some(stroke.into());
        self
    }

    fn fill(mut self, fill: impl Into<Fill>) -> Self {
        self.shape.fill = Some(fill.into());
        self
    }
}

impl<M: Model> EventHandler<M::Message> for GroupBuilder<M> {
}


pub fn comp(model: impl Model) -> CompBuilder {
    CompBuilder {
        comp: Comp::new(model)
    }
}

pub struct CompBuilder {
    comp: Comp,
}

impl CompBuilder {
}

impl<M: Model> Builder<M> for CompBuilder {
    fn build(self) -> Node<M> {
        Node::Comp(self.comp)
    }
}

impl Entity for CompBuilder {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.comp.set_id(id);
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.comp.set_transform(transform);
        self
    }
}

pub fn translate(x: impl Into<f32>, y: impl Into<f32>) -> Transform {
    Transform::new().with_translation(x.into(), y.into())
}

pub fn rotate(theta: impl Into<f32>) -> Transform {
    Transform::new().with_rotation(theta.into())
}