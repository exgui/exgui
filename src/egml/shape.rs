pub mod rect;
pub mod circle;
pub mod path;
pub mod group;
pub mod font;
pub mod text;
pub mod paint;
pub mod stroke;
pub mod fill;
pub mod translate;

pub use self::rect::*;
pub use self::circle::*;
pub use self::path::*;
pub use self::group::*;
pub use self::font::*;
pub use self::text::*;
pub use self::paint::*;
pub use self::stroke::*;
pub use self::fill::*;
pub use self::translate::*;

use crate::egml::transform::Transform;

pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Path(Path),
    Group(Group),
    Font(Font),
    Text(Text),
}

pub struct ShapeRef<'a>(pub &'a Shape);
pub struct ShapeRefMut<'a>(pub &'a mut Shape);

impl Shape {
    pub fn as_ref(&self) -> ShapeRef {
        ShapeRef(self)
    }

    pub fn as_ref_mut(&mut self) -> ShapeRefMut {
        ShapeRefMut(self)
    }
}

impl<'a> ShapeRef<'a> {
    pub fn rect(&self) -> Option<&Rect> {
        match self.0 {
            Shape::Rect(ref rect) => Some(rect),
            _ => None,
        }
    }

    pub fn circle(&self) -> Option<&Circle> {
        match self.0 {
            Shape::Circle(ref circle) => Some(circle),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self.0 {
            Shape::Path(ref path) => Some(path),
            _ => None,
        }
    }

    pub fn group(&self) -> Option<&Group> {
        match self.0 {
            Shape::Group(ref group) => Some(group),
            _ => None,
        }
    }

    pub fn font(&self) -> Option<&Font> {
        match self.0 {
            Shape::Font(ref font) => Some(font),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&Text> {
        match self.0 {
            Shape::Text(ref text) => Some(text),
            _ => None,
        }
    }
}

impl<'a> ShapeRefMut<'a> {
    pub fn rect(&mut self) -> Option<&mut Rect> {
        match self.0 {
            Shape::Rect(ref mut rect) => Some(rect),
            _ => None,
        }
    }

    pub fn circle(&mut self) -> Option<&mut Circle> {
        match self.0 {
            Shape::Circle(ref mut circle) => Some(circle),
            _ => None,
        }
    }

    pub fn path(&mut self) -> Option<&mut Path> {
        match self.0 {
            Shape::Path(ref mut path) => Some(path),
            _ => None,
        }
    }

    pub fn group(&mut self) -> Option<&mut Group> {
        match self.0 {
            Shape::Group(ref mut group) => Some(group),
            _ => None,
        }
    }

    pub fn font(&mut self) -> Option<&mut Font> {
        match self.0 {
            Shape::Font(ref mut font) => Some(font),
            _ => None,
        }
    }

    pub fn text(&mut self) -> Option<&mut Text> {
        match self.0 {
            Shape::Text(ref mut text) => Some(text),
            _ => None,
        }
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

impl From<Font> for Shape {
    fn from(font: Font) -> Self {
        Shape::Font(font)
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