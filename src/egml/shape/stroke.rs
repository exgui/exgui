use crate::egml::{Real, Converter};
use super::{Paint, Color, Gradient};

#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    pub paint: Paint,
    pub width: Real,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: Real,
}

impl Stroke {
    pub fn color<T: Into<Color>>(color: T) -> Self {
        Stroke::from(color.into())
    }

    pub fn gradient<T: Into<Gradient>>(gradient: T) -> Self {
        Stroke::from(gradient.into())
    }

    pub fn width(mut self, width: Real) -> Self {
        self.width = width;
        self
    }
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

impl From<(Color, Real)> for Stroke {
    fn from((color, width): (Color, Real)) -> Self {
        Stroke {
            paint: color.into(),
            width,
            ..Default::default()
        }
    }
}

impl From<(Color, i32)> for Stroke {
    fn from((color, width): (Color, i32)) -> Self {
        Stroke {
            paint: color.into(),
            width: width as Real,
            ..Default::default()
        }
    }
}

impl From<(Color, Real, f32)> for Stroke {
    fn from((color, width, alpha): (Color, Real, f32)) -> Self {
        Stroke {
            paint: color.with_alpha(alpha).into(),
            width,
            ..Default::default()
        }
    }
}

impl From<(Color, i32, f32)> for Stroke {
    fn from((color, width, alpha): (Color, i32, f32)) -> Self {
        Stroke {
            paint: color.with_alpha(alpha).into(),
            width: width as Real,
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

impl From<(Gradient, Real)> for Stroke {
    fn from((gradient, width): (Gradient, Real)) -> Self {
        Stroke {
            paint: gradient.into(),
            width,
            ..Default::default()
        }
    }
}

impl Converter<Option<Stroke>> for Color {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for (Color, Real) {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for (Color, i32) {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for (Color, Real, f32) {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for (Color, i32, f32) {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for Gradient {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
    }
}

impl Converter<Option<Stroke>> for (Gradient, Real) {
    fn convert(self) -> Option<Stroke> {
        Some(self.into())
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