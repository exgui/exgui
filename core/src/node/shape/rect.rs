use crate::{Clip, Real, RealValue, Fill, Padding, Stroke, Transform, TransformMatrix};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Rect {
    pub id: Option<String>,
    pub x: RealValue,
    pub y: RealValue,
    pub width: RealValue,
    pub height: RealValue,
    pub padding: Padding,
    pub transparency: Real,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub clip: Clip,
    pub transform: Transform,
}

impl Rect {
    pub const NAME: &'static str = "rect";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    pub fn recalculate_transform(&mut self, parent_global: TransformMatrix) -> TransformMatrix {
        if let Some(transform) = self.clip.transform_mut() {
            transform.calculate_global(parent_global);
        }
        self.transform.calculate_global(parent_global)
    }

    #[inline]
    pub fn intersect(&self, x: Real, y: Real) -> bool {
        let matrix = self.transform.global_matrix().unwrap_or_else(|| self.transform.matrix());
        let (x, y) = if !matrix.is_identity() {
            matrix.inverse() * (x, y)
        } else {
            (x, y)
        };
        x >= self.x.val() && x <= self.width.val() && y >= self.y.val() && y <= self.height.val()
    }
}