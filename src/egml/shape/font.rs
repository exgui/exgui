use std::any::Any;
use crate::egml::{Real, RealValue, Converter, Fill, Stroke, Transform};

#[derive(Default, Clone)]
pub struct Font {
    pub x: RealValue,
    pub y: RealValue,
    pub name: String,
    pub size: RealValue,
    pub align: (AlignHor, AlignVer),
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Font, &dyn Any)>,
}

impl Font {
    #[inline]
    pub fn intersect(&self, _x: Real, _y: Real) -> bool {
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

impl<'a> Converter<(AlignHor, AlignVer)> for AlignHor {
    fn convert(self) -> (AlignHor, AlignVer) {
        (self, AlignVer::default())
    }
}

impl<'a> Converter<(AlignHor, AlignVer)> for AlignVer {
    fn convert(self) -> (AlignHor, AlignVer) {
        (AlignHor::default(), self)
    }
}