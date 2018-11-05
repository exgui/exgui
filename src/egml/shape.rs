#[derive(Debug)]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Group(Group),
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

    pub fn group(&self) -> Option<&Group> {
        match self.0 {
            Shape::Group(ref group) => Some(group),
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

    pub fn group(&mut self) -> Option<&mut Group> {
        match self.0 {
            Shape::Group(ref mut group) => Some(group),
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

impl From<Group> for Shape {
    fn from(group: Group) -> Self {
        Shape::Group(group)
    }
}

#[derive(Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
}

#[derive(Debug, Default)]
pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
}

#[derive(Debug, Default)]
pub struct Group {
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub translate: Option<Translate>,
}

impl Group {
    pub fn empty_overrides(&self) -> bool {
        self.stroke.is_none() && self.fill.is_none() && self.translate.is_none()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    White,
    Black,
    RGB(f32, f32, f32),
}

impl Color {
    pub fn as_arr(&self) -> [f32; 4] {
        match *self {
            Color::Red => [1.0, 0.0, 0.0, 1.0],
            Color::Green => [0.0, 1.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 1.0, 1.0],
            Color::Yellow => [1.0, 1.0, 0.0, 1.0],
            Color::White => [1.0, 1.0, 1.0, 1.0],
            Color::Black => [0.0, 0.0, 0.0, 1.0],
            Color::RGB(r, g, b) => [r, g, b, 1.0],
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub transparent: f32,
    pub width: f32,
}

impl From<Color> for Stroke {
    fn from(color: Color) -> Self {
        Stroke {
            color,
            width: 1.0,
            transparent: 0.0,
        }
    }
}

impl From<(Color, f32)> for Stroke {
    fn from((color, width): (Color, f32)) -> Self {
        Stroke {
            color,
            width,
            transparent: 0.0,
        }
    }
}

impl From<(Color, f32, f32)> for Stroke {
    fn from((color, width, transparent): (Color, f32, f32)) -> Self {
        Stroke {
            color,
            width,
            transparent,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Fill {
    pub color: Color,
    pub transparent: f32,
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill {
            color,
            transparent: 0.0,
        }
    }
}

impl From<(Color, f32)> for Fill {
    fn from((color, transparent): (Color, f32)) -> Self {
        Fill {
            color,
            transparent,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Translate {
    pub x: f32,
    pub y: f32,
}

impl From<(f32, f32)> for Translate {
    fn from((x, y): (f32, f32)) -> Self {
        Translate { x, y }
    }
}