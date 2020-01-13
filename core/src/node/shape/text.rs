use crate::node::{Real, RealValue, ConvertTo, Fill, Stroke, Transform};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Text {
    pub id: Option<String>,
    pub content: String,
    pub x: RealValue,
    pub y: RealValue,
    pub font_name: String,
    pub font_size: RealValue,
    pub align: (AlignHor, AlignVer),
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
}

impl Text {
    pub const NAME: &'static str = "text";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

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

impl From<AlignHor> for (AlignHor, AlignVer) {
    fn from(align: AlignHor) -> Self {
        (align, AlignVer::default())
    }
}

impl From<AlignVer> for (AlignHor, AlignVer) {
    fn from(align: AlignVer) -> Self {
        (AlignHor::default(), align)
    }
}

impl ConvertTo<(AlignHor, AlignVer)> for AlignHor {
    fn convert(self) -> (AlignHor, AlignVer) {
        self.into()
    }
}

impl ConvertTo<(AlignHor, AlignVer)> for AlignVer {
    fn convert(self) -> (AlignHor, AlignVer) {
        self.into()
    }
}