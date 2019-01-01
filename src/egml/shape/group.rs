use crate::egml::{Fill, Stroke, Translate};

#[derive(Debug, Default)]
pub struct Group {
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub translate: Option<Translate>,
}

impl Group {
    pub fn empty_overrides(&self) -> bool {
        self.stroke.is_none() && self.fill.is_none() && self.translate.is_none()
    }
}