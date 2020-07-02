pub use self::{
    rect::*,
    circle::*,
    path::*,
    group::*,
    padding::*,
    text::*,
    paint::*,
    stroke::*,
    fill::*,
    translate::*,
};
use crate::{Real, Transform};

pub mod rect;
pub mod circle;
pub mod path;
pub mod group;
pub mod padding;
pub mod text;
pub mod paint;
pub mod stroke;
pub mod fill;
pub mod translate;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Path(Path),
    Group(Group),
    Text(Text),
}

pub trait Shaped {
    fn rect(&self) -> Option<&Rect>;
    fn rect_mut(&mut self) -> Option<&mut Rect>;

    fn circle(&self) -> Option<&Circle>;
    fn circle_mut(&mut self) -> Option<&mut Circle>;

    fn path(&self) -> Option<&Path>;
    fn path_mut(&mut self) -> Option<&mut Path>;

    fn group(&self) -> Option<&Group>;
    fn group_mut(&mut self) -> Option<&mut Group>;

    fn text(&self) -> Option<&Text>;
    fn text_mut(&mut self) -> Option<&mut Text>;
}

pub struct ShapeRef<'a>(pub &'a Shape);
pub struct ShapeRefMut<'a>(pub &'a mut Shape);

impl Shape {
    pub fn id(&self) -> Option<&str> {
        match self {
            Shape::Rect(rect) => rect.id(),
            Shape::Circle(circle) => circle.id(),
            Shape::Path(path) => path.id(),
            Shape::Group(group) => group.id(),
            Shape::Text(text) => text.id(),
        }
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        let id = Some(id.into());
        match self {
            Shape::Rect(rect) => rect.id = id,
            Shape::Circle(circle) => circle.id = id,
            Shape::Path(path) => path.id = id,
            Shape::Group(group) => group.id = id,
            Shape::Text(text) => text.id = id,
        }
    }

    pub fn transform(&self) -> &Transform {
        match self {
            Shape::Rect(rect) => &rect.transform,
            Shape::Circle(circle) => &circle.transform,
            Shape::Path(path) => &path.transform,
            Shape::Group(group) => &group.transform,
            Shape::Text(text) => &text.transform,
        }
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            Shape::Rect(rect) => &mut rect.transform,
            Shape::Circle(circle) => &mut circle.transform,
            Shape::Path(path) => &mut path.transform,
            Shape::Group(group) => &mut group.transform,
            Shape::Text(text) => &mut text.transform,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> ShapeRef {
        ShapeRef(self)
    }

    #[inline]
    pub fn as_ref_mut(&mut self) -> ShapeRefMut {
        ShapeRefMut(self)
    }
}

impl Shaped for Shape {
    #[inline]
    fn rect(&self) -> Option<&Rect> {
        match self {
            Shape::Rect(rect) => Some(rect),
            _ => None,
        }
    }

    #[inline]
    fn rect_mut(&mut self) -> Option<&mut Rect> {
        match self {
            Shape::Rect(rect) => Some(rect),
            _ => None,
        }
    }

    #[inline]
    fn circle(&self) -> Option<&Circle> {
        match self {
            Shape::Circle(circle) => Some(circle),
            _ => None,
        }
    }

    #[inline]
    fn circle_mut(&mut self) -> Option<&mut Circle> {
        match self {
            Shape::Circle(circle) => Some(circle),
            _ => None,
        }
    }

    #[inline]
    fn path(&self) -> Option<&Path> {
        match self {
            Shape::Path(path) => Some(path),
            _ => None,
        }
    }

    #[inline]
    fn path_mut(&mut self) -> Option<&mut Path> {
        match self {
            Shape::Path(path) => Some(path),
            _ => None,
        }
    }

    #[inline]
    fn group(&self) -> Option<&Group> {
        match self {
            Shape::Group(group) => Some(group),
            _ => None,
        }
    }

    #[inline]
    fn group_mut(&mut self) -> Option<&mut Group> {
        match self {
            Shape::Group(group) => Some(group),
            _ => None,
        }
    }

    #[inline]
    fn text(&self) -> Option<&Text> {
        match self {
            Shape::Text(text) => Some(text),
            _ => None,
        }
    }

    #[inline]
    fn text_mut(&mut self) -> Option<&mut Text> {
        match self {
            Shape::Text(text) => Some(text),
            _ => None,
        }
    }
}

impl<'a> ShapeRef<'a> {
    #[inline]
    pub fn rect(&self) -> Option<&Rect> {
        self.0.rect()
    }

    #[inline]
    pub fn circle(&self) -> Option<&Circle> {
        self.0.circle()
    }

    #[inline]
    pub fn path(&self) -> Option<&Path> {
        self.0.path()
    }

    #[inline]
    pub fn group(&self) -> Option<&Group> {
        self.0.group()
    }

    #[inline]
    pub fn text(&self) -> Option<&Text> {
        self.0.text()
    }
}

impl<'a> ShapeRefMut<'a> {
    #[inline]
    pub fn rect(&mut self) -> Option<&mut Rect> {
        self.0.rect_mut()
    }

    #[inline]
    pub fn circle(&mut self) -> Option<&mut Circle> {
        self.0.circle_mut()
    }

    #[inline]
    pub fn path(&mut self) -> Option<&mut Path> {
        self.0.path_mut()
    }

    #[inline]
    pub fn group(&mut self) -> Option<&mut Group> {
        self.0.group_mut()
    }

    #[inline]
    pub fn text(&mut self) -> Option<&mut Text> {
        self.0.text_mut()
    }
}

impl From<Rect> for Shape {
    fn from(rect: Rect) -> Self {
        Shape::Rect(rect)
    }
}

impl From<Circle> for Shape {
    fn from(circle: Circle) -> Self {
        Shape::Circle(circle)
    }
}

impl From<Path> for Shape {
    fn from(path: Path) -> Self {
        Shape::Path(path)
    }
}

impl From<Group> for Shape {
    fn from(group: Group) -> Self {
        Shape::Group(group)
    }
}

impl From<Text> for Shape {
    fn from(text: Text) -> Self {
        Shape::Text(text)
    }
}

impl From<String> for Shape {
    fn from(text: String) -> Self {
        Shape::Text(Text { content: text, ..Default::default() })
    }
}

pub type CompositeShapeIter<'a> = Box<dyn Iterator<Item = &'a dyn CompositeShape> + 'a>;
pub type CompositeShapeIterMut<'a> = Box<dyn Iterator<Item = &'a mut dyn CompositeShape> + 'a>;

pub trait CompositeShape {
    fn shape(&self) -> Option<&Shape>;

    fn shape_mut(&mut self) -> Option<&mut Shape>;

    fn children(&self) -> Option<CompositeShapeIter>;

    fn children_mut(&mut self) -> Option<CompositeShapeIterMut>;

    fn need_recalc(&self) -> Option<bool>;

    fn intersect(&self, x: Real, y: Real) -> bool {
        if let Some(shape) = self.shape() {
            match shape {
                Shape::Rect(rect) => rect.intersect(x, y),
                Shape::Circle(circle) => circle.intersect(x, y),
                Shape::Path(path) => path.intersect(x, y),
                _ => false,
            }
        } else {
            false
        }
    }
}