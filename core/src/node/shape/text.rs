use crate::node::{Real, RealValue, ConvertTo, Fill, Stroke, Transform, TransformMatrix};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GlyphPos {
    pub x: Real,
    pub max_x: Real,
    pub min_x: Real,
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct TextMetrics {
    pub ascender: f32,
    pub descender: f32,
    pub line_height: f32,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Text {
    pub id: Option<String>,
    pub content: String,
    pub glyph_positions: Vec<GlyphPos>,
    pub metrics: Option<TextMetrics>,
    pub x: RealValue,
    pub y: RealValue,
    pub font_name: String,
    pub font_size: RealValue,
    pub align: (AlignHor, AlignVer),
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Transform,
}

impl Text {
    pub const NAME: &'static str = "text";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    pub fn recalculate_transform(&mut self, parent_global: TransformMatrix) -> TransformMatrix {
        self.transform.calculate_global(parent_global)
    }

    #[inline]
    pub fn intersect(&self, _x: Real, _y: Real) -> bool {
        // TODO: calvulate intersect
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