use std::fmt::Debug;
use crate::egml::Converter;

#[derive(Debug, Default, Clone, Copy)]
pub struct Pct<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Auto,
    Px,
    Pct(Real),
}

impl Default for ValueType {
    fn default() -> Self {
        ValueType::Auto
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Value<T: Debug + Default + Clone + Copy>(pub T, pub ValueType);

impl<T: Debug + Default + Clone + Copy> Value<T> {
    pub fn px(v: T) -> Self {
        Value(v, ValueType::Px)
    }

    pub fn pct(pct: Real) -> Self {
        Value(Default::default(), ValueType::Pct(pct))
    }

    pub fn auto() -> Self {
        Value(Default::default(), ValueType::Auto)
    }

    pub fn val(&self) -> T {
        self.0
    }

    pub fn set_val(&mut self, v: T) {
        self.0 = v
    }

    pub fn set_by_auto(&mut self, source: T) -> bool {
        if let Value(ref mut v, ValueType::Auto) = self {
            *v = source;
            true
        } else {
            false
        }
    }
}

impl Value<Real> {
    pub fn set_by_pct(&mut self, source: Real) -> bool {
        if let Value(ref mut v, ValueType::Pct(pct)) = self {
            *v = *pct / 100.0 * source;
            true
        } else {
            false
        }
    }
}

pub type Real = f32;
pub type RealValue = Value<Real>;

impl From<Real> for RealValue {
    fn from(v: Real) -> Self {
        RealValue::px(v)
    }
}

impl From<i32> for RealValue {
    fn from(v: i32) -> Self {
        RealValue::px(v as Real)
    }
}

impl From<Pct<Real>> for RealValue {
    fn from(v: Pct<Real>) -> Self {
        RealValue::pct(v.0)
    }
}

impl From<Pct<i32>> for RealValue {
    fn from(v: Pct<i32>) -> Self {
        RealValue::pct(v.0 as Real)
    }
}

impl Converter<Real> for i32 {
    fn convert(self) -> Real {
        self as Real
    }
}

impl Converter<Option<Real>> for i32 {
    fn convert(self) -> Option<Real> {
        Some(self as Real)
    }
}

impl Converter<RealValue> for Real {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl Converter<Option<RealValue>> for Real {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl Converter<RealValue> for i32 {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl Converter<Option<RealValue>> for i32 {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl Converter<RealValue> for Pct<Real> {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl Converter<Option<RealValue>> for Pct<Real> {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl Converter<RealValue> for Pct<i32> {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl Converter<Option<RealValue>> for Pct<i32> {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}