use std::any::Any;
use std::convert::AsRef;
use egml::transform::Transform;
use egml::paint::{Paint, Color, Gradient};

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

#[derive(Default)]
pub struct Font {
    pub x: f32,
    pub y: f32,
    pub name: String,
    pub size: f32,
    pub align: (AlignHor, AlignVer),
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Font, &dyn Any)>,
}

impl Font {
    #[inline]
    pub fn intersect(&self, _x: f32, _y: f32) -> bool {
        // TODO: calvulate intersect
//        let (x, y) = self.transform.as_ref()
//            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
//            .unwrap_or((x, y));
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignHor {
    Left,
    Right,
    Center,
}

impl Default for AlignHor {
    fn default() -> Self {
        AlignHor::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignVer {
    Bottom,
    Middle,
    Baseline,
    Top,
}

impl Default for AlignVer {
    fn default() -> Self {
        AlignVer::Top
    }
}

#[derive(Default)]
pub struct Text {
    pub content: String,
    pub modifier: Option<fn(&mut Text, &dyn Any)>,
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.content.as_str()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub paint: Paint,
    pub width: f32,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            paint: Default::default(),
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
            paint: color.into(),
            ..Default::default()
        }
    }
}

impl From<(Color, f32)> for Stroke {
    fn from((color, width): (Color, f32)) -> Self {
        Stroke {
            paint: color.into(),
            width,
            ..Default::default()
        }
    }
}

impl From<(Color, f32, f32)> for Stroke {
    fn from((color, width, alpha): (Color, f32, f32)) -> Self {
        Stroke {
            paint: color.with_alpha(alpha).into(),
            width,
            ..Default::default()
        }
    }
}

impl From<Gradient> for Stroke {
    fn from(gradient: Gradient) -> Self {
        Stroke {
            paint: gradient.into(),
            ..Default::default()
        }
    }
}

impl From<(Gradient, f32)> for Stroke {
    fn from((gradient, width): (Gradient, f32)) -> Self {
        Stroke {
            paint: gradient.into(),
            width,
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
    pub paint: Paint,
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill {
            paint: color.into(),
        }
    }
}

impl From<(Color, f32)> for Fill {
    fn from((color, alpha): (Color, f32)) -> Self {
        Fill {
            paint: color.with_alpha(alpha).into(),
        }
    }
}

impl From<Gradient> for Fill {
    fn from(gradient: Gradient) -> Self {
        Fill {
            paint: gradient.into(),
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