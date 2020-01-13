use crate::node::{Real, RealValue, Fill, Stroke, Transform};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Rect {
    pub id: Option<String>,
    pub x: RealValue,
    pub y: RealValue,
    pub width: RealValue,
    pub height: RealValue,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
}

impl Rect {
    pub const NAME: &'static str = "rect";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    #[inline]
    pub fn intersect(&self, x: Real, y: Real) -> bool {
        // TODO: check all transform
        let (x, y) = self.transform.as_ref()
            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
            .unwrap_or((x, y));
        x >= self.x.val() && x <= self.width.val() && y >= self.y.val() && y <= self.height.val()
    }
}