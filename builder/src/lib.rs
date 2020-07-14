use std::{borrow::Cow, collections::HashMap};

use exgui_core::{
    Model, Node, Prim, Shape, Rect, Circle, Text, Group, Path, PathCommand, RealValue, Stroke, Fill,
    AlignHor, AlignVer, Transform, Comp, EventName, Listener, Padding, Clip, Rounding, Real,
};
pub use exgui_core::builder::*;

pub struct PrimBuilder<M: Model> {
    pub children: Vec<Node<M>>,
    pub listeners: HashMap<EventName, Vec<Listener<M>>>,
}

impl<M: Model> Default for PrimBuilder<M> {
    fn default() -> Self {
        Self {
            children: Default::default(),
            listeners: Default::default(),
        }
    }
}

pub fn circle<M: Model>() -> CircleBuilder<M> {
    CircleBuilder {
        shape: Default::default(),
        prim: Default::default(),
    }
}

pub struct CircleBuilder<M: Model> {
    shape: Circle,
    prim: PrimBuilder<M>,
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

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.shape.padding = padding.into();
        self
    }

    pub fn padding_top(mut self, top: impl Into<RealValue>) -> Self {
        self.shape.padding.top = top.into();
        self
    }

    pub fn padding_left(mut self, left: impl Into<RealValue>) -> Self {
        self.shape.padding.left = left.into();
        self
    }

    pub fn padding_right(mut self, right: impl Into<RealValue>) -> Self {
        self.shape.padding.right = right.into();
        self
    }

    pub fn padding_bottom(mut self, bottom: impl Into<RealValue>) -> Self {
        self.shape.padding.bottom = bottom.into();
        self
    }

    pub fn padding_top_and_bottom(mut self, padding: impl Into<RealValue>) -> Self {
        let padding = padding.into();
        self.shape.padding.top = padding;
        self.shape.padding.bottom = padding;
        self
    }

    pub fn padding_left_and_right(mut self, padding: impl Into<RealValue>) -> Self {
        let padding = padding.into();
        self.shape.padding.left = padding;
        self.shape.padding.right = padding;
        self
    }
}

impl<M: Model> Builder<M> for CircleBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Circle::NAME),
            Shape::Circle(self.shape),
            self.prim.children,
            self.prim.listeners,
        ))
    }
}

impl<M: Model> Entity for CircleBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = transform.into();
        self
    }
}

impl<M: Model> Primitive<M> for CircleBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.prim.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.prim.children.extend(children);
        self
    }

    fn transparency(mut self, transparency: impl Into<Real>) -> Self {
        self.shape.transparency = transparency.into();
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

    fn remove_stroke(mut self) -> Self {
        self.shape.stroke = None;
        self
    }

    fn remove_fill(mut self) -> Self {
        self.shape.fill = None;
        self
    }

    fn clip(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>, height: impl Into<RealValue>) -> Self {
        self.shape.clip = Clip::new_scissor(x.into(), y.into(), width.into(), height.into());
        self
    }
}

impl<M: Model> EventHandler<M> for CircleBuilder<M> {
    fn add_listener(&mut self, listener: Listener<M>) {
        self.prim.listeners.entry(listener.event_name()).or_default().push(listener);
    }
}


pub fn rect<M: Model>() -> RectBuilder<M> {
    RectBuilder {
        shape: Default::default(),
        prim: Default::default(),
    }
}

pub struct RectBuilder<M: Model> {
    shape: Rect,
    prim: PrimBuilder<M>,
}

impl<M: Model> RectBuilder<M> {
    pub fn left_top_pos(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>) -> Self {
        self.shape.x = x.into();
        self.shape.y = y.into();
        self
    }

    pub fn width(mut self, width: impl Into<RealValue>) -> Self {
        self.shape.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<RealValue>) -> Self {
        self.shape.height = height.into();
        self
    }

    pub fn rounding(mut self, rounding: impl Into<Rounding>) -> Self {
        self.shape.rounding = Some(rounding.into());
        self
    }

    pub fn rounding_top_left(mut self, radius: impl Into<RealValue>) -> Self {
        if let Some(rounding) = self.shape.rounding.as_mut() {
            rounding.top_left = radius.into();
        } else {
            self.shape.rounding = Some(Rounding { top_left: radius.into(), ..Default::default() });
        }
        self
    }

    pub fn rounding_top_right(mut self, radius: impl Into<RealValue>) -> Self {
        if let Some(rounding) = self.shape.rounding.as_mut() {
            rounding.top_right = radius.into();
        } else {
            self.shape.rounding = Some(Rounding { top_right: radius.into(), ..Default::default() });
        }
        self
    }

    pub fn rounding_bottom_left(mut self, radius: impl Into<RealValue>) -> Self {
        if let Some(rounding) = self.shape.rounding.as_mut() {
            rounding.bottom_left = radius.into();
        } else {
            self.shape.rounding = Some(Rounding { bottom_left: radius.into(), ..Default::default() });
        }
        self
    }

    pub fn rounding_bottom_right(mut self, radius: impl Into<RealValue>) -> Self {
        if let Some(rounding) = self.shape.rounding.as_mut() {
            rounding.bottom_right = radius.into();
        } else {
            self.shape.rounding = Some(Rounding { bottom_right: radius.into(), ..Default::default() });
        }
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.shape.padding = padding.into();
        self
    }

    pub fn padding_top(mut self, top: impl Into<RealValue>) -> Self {
        self.shape.padding.top = top.into();
        self
    }

    pub fn padding_left(mut self, left: impl Into<RealValue>) -> Self {
        self.shape.padding.left = left.into();
        self
    }

    pub fn padding_right(mut self, right: impl Into<RealValue>) -> Self {
        self.shape.padding.right = right.into();
        self
    }

    pub fn padding_bottom(mut self, bottom: impl Into<RealValue>) -> Self {
        self.shape.padding.bottom = bottom.into();
        self
    }

    pub fn padding_top_and_bottom(mut self, padding: impl Into<RealValue>) -> Self {
        let padding = padding.into();
        self.shape.padding.top = padding;
        self.shape.padding.bottom = padding;
        self
    }

    pub fn padding_left_and_right(mut self, padding: impl Into<RealValue>) -> Self {
        let padding = padding.into();
        self.shape.padding.left = padding;
        self.shape.padding.right = padding;
        self
    }
}

impl<M: Model> Builder<M> for RectBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Rect::NAME),
            Shape::Rect(self.shape),
            self.prim.children,
            self.prim.listeners,
        ))
    }
}

impl<M: Model> Entity for RectBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = transform.into();
        self
    }
}

impl<M: Model> Primitive<M> for RectBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.prim.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.prim.children.extend(children);
        self
    }

    fn transparency(mut self, transparency: impl Into<Real>) -> Self {
        self.shape.transparency = transparency.into();
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

    fn remove_stroke(mut self) -> Self {
        self.shape.stroke = None;
        self
    }

    fn remove_fill(mut self) -> Self {
        self.shape.fill = None;
        self
    }

    fn clip(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>, height: impl Into<RealValue>) -> Self {
        self.shape.clip = Clip::new_scissor(x.into(), y.into(), width.into(), height.into());
        self
    }
}

impl<M: Model> EventHandler<M> for RectBuilder<M> {
    fn add_listener(&mut self, listener: Listener<M>) {
        self.prim.listeners.entry(listener.event_name()).or_default().push(listener);
    }
}


pub fn text<M: Model>(content: impl Into<String>) -> TextBuilder<M> {
    TextBuilder {
        shape: Text { content: content.into(), ..Text::default() },
        prim: Default::default(),
    }
}

pub struct TextBuilder<M: Model> {
    shape: Text,
    prim: PrimBuilder<M>,
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
            self.prim.children,
            self.prim.listeners,
        ))
    }
}

impl<M: Model> Entity for TextBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = transform.into();
        self
    }
}

impl<M: Model> Primitive<M> for TextBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.prim.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.prim.children.extend(children);
        self
    }

    fn transparency(mut self, transparency: impl Into<Real>) -> Self {
        self.shape.transparency = transparency.into();
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

    fn remove_stroke(mut self) -> Self {
        self.shape.stroke = None;
        self
    }

    fn remove_fill(mut self) -> Self {
        self.shape.fill = None;
        self
    }

    fn clip(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>, height: impl Into<RealValue>) -> Self {
        self.shape.clip = Clip::new_scissor(x.into(), y.into(), width.into(), height.into());
        self
    }
}

impl<M: Model> EventHandler<M> for TextBuilder<M> {
    fn add_listener(&mut self, listener: Listener<M>) {
        self.prim.listeners.entry(listener.event_name()).or_default().push(listener);
    }
}


pub fn path<M: Model>(cmd: impl Into<Vec<PathCommand>>) -> PathBuilder<M> {
    PathBuilder {
        shape: Path { cmd: cmd.into(), ..Path::default() },
        prim: Default::default(),
    }
}

pub struct PathBuilder<M: Model> {
    shape: Path,
    prim: PrimBuilder<M>,
}

impl<M: Model> PathBuilder<M> {
}

impl<M: Model> Builder<M> for PathBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Path::NAME),
            Shape::Path(self.shape),
            self.prim.children,
            self.prim.listeners,
        ))
    }
}

impl<M: Model> Entity for PathBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = transform.into();
        self
    }
}

impl<M: Model> Primitive<M> for PathBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.prim.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.prim.children.extend(children);
        self
    }

    fn transparency(mut self, transparency: impl Into<Real>) -> Self {
        self.shape.transparency = transparency.into();
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

    fn remove_stroke(mut self) -> Self {
        self.shape.stroke = None;
        self
    }

    fn remove_fill(mut self) -> Self {
        self.shape.fill = None;
        self
    }

    fn clip(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>, height: impl Into<RealValue>) -> Self {
        self.shape.clip = Clip::new_scissor(x.into(), y.into(), width.into(), height.into());
        self
    }
}

impl<M: Model> EventHandler<M> for PathBuilder<M> {
    fn add_listener(&mut self, listener: Listener<M>) {
        self.prim.listeners.entry(listener.event_name()).or_default().push(listener);
    }
}


pub fn group<M: Model>() -> GroupBuilder<M> {
    GroupBuilder {
        shape: Default::default(),
        prim: Default::default()
    }
}

pub struct GroupBuilder<M: Model> {
    shape: Group,
    prim: PrimBuilder<M>,
}

impl<M: Model> GroupBuilder<M> {
}

impl<M: Model> Builder<M> for GroupBuilder<M> {
    fn build(self) -> Node<M> {
        Node::Prim(Prim::new(
            Cow::Borrowed(Group::NAME),
            Shape::Group(self.shape),
            self.prim.children,
            self.prim.listeners,
        ))
    }
}

impl<M: Model> Entity for GroupBuilder<M> {
    fn id(mut self, id: impl Into<String>) -> Self {
        self.shape.id = Some(id.into());
        self
    }

    fn transform(mut self, transform: impl Into<Transform>) -> Self {
        self.shape.transform = transform.into();
        self
    }
}

impl<M: Model> Primitive<M> for GroupBuilder<M> {
    fn child(mut self, child: impl Builder<M>) -> Self {
        self.prim.children.push(child.build());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item=Node<M>>) -> Self {
        self.prim.children.extend(children);
        self
    }

    fn transparency(mut self, transparency: impl Into<Real>) -> Self {
        self.shape.transparency = Some(transparency.into());
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

    fn remove_stroke(mut self) -> Self {
        self.shape.stroke = None;
        self
    }

    fn remove_fill(mut self) -> Self {
        self.shape.fill = None;
        self
    }

    fn clip(mut self, x: impl Into<RealValue>, y: impl Into<RealValue>, width: impl Into<RealValue>, height: impl Into<RealValue>) -> Self {
        self.shape.clip = Clip::new_scissor(x.into(), y.into(), width.into(), height.into());
        self
    }
}

impl<M: Model> EventHandler<M> for GroupBuilder<M> {
    fn add_listener(&mut self, listener: Listener<M>) {
        self.prim.listeners.entry(listener.event_name()).or_default().push(listener);
    }
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

// todo: use RealValue's
pub fn translate(x: impl Into<Real>, y: impl Into<Real>) -> Transform {
    Transform::new().with_translation(x.into(), y.into())
}

pub fn rotate(theta: impl Into<Real>) -> Transform {
    Transform::new().with_rotation(theta.into())
}

pub fn scale(x: impl Into<Real>, y: impl Into<Real>) -> Transform {
    Transform::new().with_scale(x.into(), y.into())
}