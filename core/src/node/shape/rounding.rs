use crate::RealValue;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Rounding {
    pub top_left: RealValue,
    pub top_right: RealValue,
    pub bottom_left: RealValue,
    pub bottom_right: RealValue,
}

impl<T: Into<RealValue>> From<T> for Rounding {
    fn from(radius: T) -> Self {
        let radius = radius.into();
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}
