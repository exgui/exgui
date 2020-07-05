use crate::node::{Clip, Real, RealValue, Fill, Padding, Stroke, Transform, TransformMatrix};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Circle {
    pub id: Option<String>,
    pub cx: RealValue,
    pub cy: RealValue,
    pub r: RealValue,
    pub padding: Padding,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub clip: Clip,
    pub transform: Transform,
}

impl Circle {
    pub const NAME: &'static str = "circle";

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
        ((x - self.cx.val()).powi(2) + (y - self.cy.val()).powi(2)).sqrt() <= self.r.val()
    }
}