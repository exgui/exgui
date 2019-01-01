use std::any::Any;
use crate::egml::Real;
use super::{Fill, Stroke, Transform};

#[derive(Default)]
pub struct Path {
    pub cmd: Vec<PathCommand>,
    pub stroke: Option<Stroke>,
    pub fill: Option<Fill>,
    pub transform: Option<Transform>,
    pub modifier: Option<fn(&mut Path, &dyn Any)>,
}

impl Path {
    pub fn intersect(&self, _x: Real, _y: Real) -> bool {
        false // TODO: need impl
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathCommand {
    Move([Real; 2]),
    MoveRel([Real; 2]),
    Line([Real; 2]),
    LineRel([Real; 2]),
    LineAlonX(Real),
    LineAlonXRel(Real),
    LineAlonY(Real),
    LineAlonYRel(Real),
    Close,
    BezCtrl([Real; 2]),
    BezCtrlRel([Real; 2]),
    BezReflectCtrl,
    QuadBezTo([Real; 2]),
    QuadBezToRel([Real; 2]),
    CubBezTo([Real; 2]),
    CubBezToRel([Real; 2]),
}