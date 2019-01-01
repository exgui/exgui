use crate::egml::Converter;
use super::{Paint, Color, Gradient};

#[derive(Debug, Default, Clone, Copy)]
pub struct Fill {
    pub paint: Paint,
}

impl Fill {
    pub fn color<T: Into<Color>>(color: T) -> Self {
        Fill::from(color.into())
    }

    pub fn gradient<T: Into<Gradient>>(gradient: T) -> Self {
        Fill::from(gradient.into())
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Fill {
            paint: color.into(),
        }
    }
}

impl Converter<Option<Fill>> for Color {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}

impl From<(Color, f32)> for Fill {
    fn from((color, alpha): (Color, f32)) -> Self {
        Fill {
            paint: color.with_alpha(alpha).into(),
        }
    }
}

impl Converter<Option<Fill>> for (Color, f32) {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}

impl From<Gradient> for Fill {
    fn from(gradient: Gradient) -> Self {
        Fill {
            paint: gradient.into(),
        }
    }
}

impl Converter<Option<Fill>> for Gradient {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}