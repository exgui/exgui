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

impl From<(Pct<Real>, Real)> for Translate {
    fn from((Pct(x), y): (Pct<Real>, Real)) -> Self {
        Translate { x: RealValue::pct(x), y: RealValue::px(y) }
    }
}

impl From<(Pct<i32>, Real)> for Translate {
    fn from((Pct(x), y): (Pct<i32>, Real)) -> Self {
        Translate { x: RealValue::pct(x as Real), y: RealValue::px(y) }
    }
}

impl From<(Real, Pct<Real>)> for Translate {
    fn from((x, Pct(y)): (Real, Pct<Real>)) -> Self {
        Translate { x: RealValue::px(x), y: RealValue::pct(y) }
    }
}

impl From<(Real, Pct<i32>)> for Translate {
    fn from((x, Pct(y)): (Real, Pct<i32>)) -> Self {
        Translate { x: RealValue::px(x), y: RealValue::pct(y as Real) }
    }
}

impl From<(Pct<Real>, Pct<Real>)> for Translate {
    fn from((Pct(x), Pct(y)): (Pct<Real>, Pct<Real>)) -> Self {
        Translate { x: RealValue::pct(x), y: RealValue::pct(y) }
    }
}

impl From<(Pct<i32>, Pct<i32>)> for Translate {
    fn from((Pct(x), Pct(y)): (Pct<i32>, Pct<i32>)) -> Self {
        Translate { x: RealValue::pct(x as Real), y: RealValue::pct(y as Real) }
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

impl Converter<Option<Translate>> for (Pct<Real>, Real) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Pct<i32>, Real) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Real, Pct<Real>) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Real, Pct<i32>) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Pct<Real>, Pct<Real>) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}

impl Converter<Option<Translate>> for (Pct<i32>, Pct<i32>) {
    fn convert(self) -> Option<Translate> {
        Some(self.into())
    }
}