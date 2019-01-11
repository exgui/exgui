use crate::egml::{Real, RealValue, AnyModel, Fill, Stroke, Transform};

#[derive(Default)]
pub struct Circle {
    pub id: Option<String>,
    pub cx: RealValue,
    pub cy: RealValue,
    pub r: RealValue,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Circle, &dyn AnyModel)>,
}

impl Circle {
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