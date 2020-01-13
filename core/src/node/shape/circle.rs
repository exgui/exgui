use crate::node::{Real, RealValue, Fill, Stroke, Transform};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Circle {
    pub id: Option<String>,
    pub cx: RealValue,
    pub cy: RealValue,
    pub r: RealValue,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
}

impl Circle {
    pub const NAME: &'static str = "circle";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    #[inline]
    pub fn intersect(&self, x: Real, y: Real) -> bool {
        // TODO: check all transform
        let (x, y) = self.transform.as_ref()
            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
            .unwrap_or((x, y));
        ((x - self.cx.val()).powi(2) + (y - self.cy.val()).powi(2)).sqrt() <= self.r.val()
    }
}