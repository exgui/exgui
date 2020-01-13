use crate::node::{Fill, Stroke, Transform};

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Group {
    pub id: Option<String>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
}

impl Group {
    pub const NAME: &'static str = "group";

    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    pub fn empty_overrides(&self) -> bool {
        self.stroke.is_none() && self.fill.is_none() && self.transform.is_none()
    }
}