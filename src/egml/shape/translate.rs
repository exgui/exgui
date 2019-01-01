use crate::egml::{Real, RealValue, Pct, Converter};

#[derive(Debug, Default, Clone, Copy)]
pub struct Translate {
    pub x: RealValue,
    pub y: RealValue,
}

impl From<(Real, Real)> for Translate {
    fn from((x, y): (Real, Real)) -> Self {
        Translate { x: RealValue::px(x), y: RealValue::px(y) }
    }
}

impl From<(i32, i32)> for Translate {
    fn from((x, y): (i32, i32)) -> Self {
        Translate { x: RealValue::px(x as Real), y: RealValue::px(y as Real) }
    }
}

impl From<(Pct, Real)> for Translate {
    fn from((Pct(x), y): (Pct, Real)) -> Self {
        Translate { x: RealValue::pct(x), y: RealValue::px(y) }
    }
}

impl From<(Real, Pct)> for Translate {
    fn from((x, Pct(y)): (Real, Pct)) -> Self {
        Translate { x: RealValue::px(x), y: RealValue::pct(y) }
    }
}

impl From<(Pct, Pct)> for Translate {
    fn from((Pct(x), Pct(y)): (Pct, Pct)) -> Self {
        Translate { x: RealValue::pct(x), y: RealValue::pct(y) }
    }
}

impl Converter<Option<Translate>> for (Real, Real) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (i32, i32) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Pct, Real) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Real, Pct) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Pct, Pct) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}