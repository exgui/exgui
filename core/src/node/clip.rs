use crate::{RealValue, Transform};

/// A scissor defines a region on the screen in which drawing operations are allowed.
/// Pixels drawn outside of this region are clipped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scissor {
    pub x: RealValue,
    pub y: RealValue,
    pub width: RealValue,
    pub height: RealValue,
    pub transform: Transform,
}

/// Define how to clip specified region.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clip {
    Scissor(Scissor),
    None,
}

impl Clip {
    pub fn new_scissor(x: RealValue, y: RealValue, width: RealValue, height: RealValue) -> Self {
        Clip::Scissor(Scissor {
            x,
            y,
            width,
            height,
            transform: Transform::default(),
        })
    }

    pub fn is_none(&self) -> bool {
        if let Clip::None = self {
            true
        } else {
            false
        }
    }

    pub fn or(self, other: Self) -> Self {
        if self.is_none() {
            other
        } else {
            self
        }
    }

    pub fn scissor(&self) -> Option<&Scissor> {
        match self {
            Clip::Scissor(scissor) => Some(scissor),
            Clip::None => None,
        }
    }

    pub fn scissor_mut(&mut self) -> Option<&mut Scissor> {
        match self {
            Clip::Scissor(scissor) => Some(scissor),
            Clip::None => None,
        }
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.scissor().map(|scissor| &scissor.transform)
    }

    pub fn transform_mut(&mut self) -> Option<&mut Transform> {
        self.scissor_mut().map(|scissor| &mut scissor.transform)
    }
}

impl Default for Clip {
    fn default() -> Self {
        Clip::None
    }
}
