use super::{Color, Gradient, Paint};
use crate::node::ConvertTo;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Fill {
    pub paint: Paint,
}

impl Fill {
    pub fn color<T: Into<Color>>(color: T) -> Self {
        Self::from(color.into())
    }

    pub fn gradient<T: Into<Gradient>>(gradient: T) -> Self {
        Self::from(gradient.into())
    }
}

impl From<Color> for Fill {
    fn from(color: Color) -> Self {
        Self { paint: color.into() }
    }
}

impl ConvertTo<Option<Fill>> for Color {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}

impl From<(Color, f32)> for Fill {
    fn from((color, alpha): (Color, f32)) -> Self {
        Self {
            paint: color.with_alpha(alpha).into(),
        }
    }
}

impl ConvertTo<Option<Fill>> for (Color, f32) {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}

impl From<Gradient> for Fill {
    fn from(gradient: Gradient) -> Self {
        Self { paint: gradient.into() }
    }
}

impl ConvertTo<Option<Fill>> for Gradient {
    fn convert(self) -> Option<Fill> {
        Some(self.into())
    }
}
