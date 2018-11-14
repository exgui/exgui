use std::any::Any;
use egml::transform::Transform;

pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Path(Path),
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

#[derive(Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Rect, &dyn Any)>,
}

impl Rect {
    #[inline]
    pub fn intersect(&self, x: f32, y: f32) -> bool {
        // TODO: check all transform
        let (x, y) = self.transform.as_ref()
            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
            .unwrap_or((x, y));
        x >= self.x && x <= self.width && y >= self.y && y <= self.height
    }
}

#[derive(Default)]
pub struct Circle {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Circle, &dyn Any)>,
}

impl Circle {
    #[inline]
    pub fn intersect(&self, x: f32, y: f32) -> bool {
        // TODO: check all transform
        let (x, y) = self.transform.as_ref()
            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
            .unwrap_or((x, y));
        ((x - self.cx).powi(2) + (y - self.cy).powi(2)).sqrt() <= self.r
    }
}

#[derive(Default)]
pub struct Path {
    pub cmd: Vec<PathCommand>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Path, &dyn Any)>,
}

impl Path {
    pub fn intersect(&self, _x: f32, _y: f32) -> bool {
        false // TODO: need impl
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    Move([f32; 2]),
    MoveRel([f32; 2]),
    Line([f32; 2]),
    LineRel([f32; 2]),
    LineAlonX(f32),
    LineAlonXRel(f32),
    LineAlonY(f32),
    LineAlonYRel(f32),
    Close,
    BezCtrl([f32; 2]),
    BezCtrlRel([f32; 2]),
    BezReflectCtrl,
    QuadBezTo([f32; 2]),
    QuadBezToRel([f32; 2]),
    CubBezTo([f32; 2]),
    CubBezToRel([f32; 2]),
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

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub color: Color,
    pub transparent: f32,
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: Default::default(),
            transparent: 0.0,
            width: 1.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            miter_limit: 10.0,
        }
    }
}

impl From<Color> for Stroke {
    fn from(color: Color) -> Self {
        Stroke {
            color,
            ..Default::default()
        }
    }
}

impl From<(Color, f32)> for Stroke {
    fn from((color, width): (Color, f32)) -> Self {
        Stroke {
            color,
            width,
            ..Default::default()
        }
    }
}

impl From<(Color, f32, f32)> for Stroke {
    fn from((color, width, transparent): (Color, f32, f32)) -> Self {
        Stroke {
            color,
            width,
            transparent,
            ..Default::default()
        }
    }
}

/// Controls how the end of line is drawn.
#[derive(Clone, Copy, Debug)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

/// Controls how lines are joined together.
#[derive(Debug, Clone, Copy)]
pub enum LineJoin {
    Miter,
    Round,
    Bevel
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