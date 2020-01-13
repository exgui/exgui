use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign},
};

use crate::ConvertTo;

#[derive(Debug, Default, Clone, Copy, PartialOrd, PartialEq)]
pub struct Pct<T>(pub T);

impl<T: Add> Add for Pct<T> {
    type Output = Pct<T::Output>;

    fn add(self, other: Self) -> Self::Output {
        Pct(self.0 + other.0)
    }
}

impl<T: AddAssign> AddAssign for Pct<T> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<T: Sub> Sub for Pct<T> {
    type Output = Pct<T::Output>;

    fn sub(self, other: Self) -> Self::Output {
        Pct(self.0 - other.0)
    }
}

impl<T: SubAssign> SubAssign for Pct<T> {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl<T: Mul> Mul for Pct<T> {
    type Output = Pct<T::Output>;

    fn mul(self, rhs: Self) -> Self::Output {
        Pct(self.0 * rhs.0)
    }
}

impl<T: MulAssign> MulAssign for Pct<T> {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl<T: Div> Div for Pct<T> {
    type Output = Pct<T::Output>;

    fn div(self, rhs: Self) -> Self::Output {
        Pct(self.0 / rhs.0)
    }
}

impl<T: DivAssign> DivAssign for Pct<T> {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl<T> From<T> for Pct<T> {
    fn from(v: T) -> Self {
        Pct(v)
    }
}

impl From<i32> for Pct<Real> {
    fn from(v: i32) -> Self {
        Pct(v as Real)
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Value<T: Debug + Default + Clone + Copy + PartialEq>(pub T, pub ValueType);

impl<T: Debug + Default + Clone + Copy + PartialEq> Value<T> {
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

impl ConvertTo<Real> for i32 {
    fn convert(self) -> Real {
        self as Real
    }
}

impl ConvertTo<Option<Real>> for i32 {
    fn convert(self) -> Option<Real> {
        Some(self as Real)
    }
}

impl ConvertTo<RealValue> for Real {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl ConvertTo<Option<RealValue>> for Real {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl ConvertTo<RealValue> for i32 {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl ConvertTo<Option<RealValue>> for i32 {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl ConvertTo<RealValue> for Pct<Real> {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl ConvertTo<Option<RealValue>> for Pct<Real> {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}

impl ConvertTo<RealValue> for Pct<i32> {
    fn convert(self) -> RealValue {
        self.into()
    }
}

impl ConvertTo<Option<RealValue>> for Pct<i32> {
    fn convert(self) -> Option<RealValue> {
        Some(self.into())
    }
}