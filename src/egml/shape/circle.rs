use std::any::Any;
use crate::egml::{Real, RealValue};
use super::{Fill, Stroke, Transform};

#[derive(Default)]
pub struct Circle {
    pub cx: RealValue,
    pub cy: RealValue,
    pub r: RealValue,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Circle, &dyn Any)>,
}

impl Circle {
    #[inline]
    pub fn intersect(&self, x: Real, y: Real) -> bool {
        // TODO: check all transform
        let (x, y) = self.transform.as_ref()
            .map(|t| (x - t.matrix[4], y - t.matrix[5]))
            .unwrap_or((x, y));
        ((x - self.cx.val()).powi(2) + (y - self.cy.val()).powi(2)).sqrt() <= self.r.val()
    }
}