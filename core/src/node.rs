pub use self::{clip::*, comp::*, converter::*, prim::*, shape::*, transform::*, value::*};
use crate::{Model, SystemMessage};

pub mod builder;
pub mod clip;
pub mod comp;
pub mod converter;
pub mod prim;
pub mod shape;
pub mod transform;
pub mod value;

pub enum Node<M: Model> {
    Prim(Prim<M>),
    Comp(Comp),
}

impl<M: Model> Node<M> {
    pub fn get_id(&self) -> Option<&str> {
        match self {
            Node::Prim(prim) => prim.id(),
            Node::Comp(comp) => comp.id(),
        }
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        match self {
            Node::Prim(prim) => prim.shape.set_id(id),
            Node::Comp(comp) => comp.set_id(id),
        }
    }

    pub fn as_prim(&self) -> Option<&Prim<M>> {
        match self {
            Node::Prim(prim) => Some(prim),
            _ => None,
        }
    }

    pub fn as_prim_mut(&mut self) -> Option<&mut Prim<M>> {
        match self {
            Node::Prim(prim) => Some(prim),
            _ => None,
        }
    }

    pub fn into_prim(self) -> Option<Prim<M>> {
        match self {
            Node::Prim(prim) => Some(prim),
            _ => None,
        }
    }

    pub fn as_comp(&self) -> Option<&Comp> {
        match self {
            Node::Comp(comp) => Some(comp),
            _ => None,
        }
    }

    pub fn as_comp_mut(&mut self) -> Option<&mut Comp> {
        match self {
            Node::Comp(comp) => Some(comp),
            _ => None,
        }
    }

    pub fn into_comp(self) -> Option<Comp> {
        match self {
            Node::Comp(comp) => Some(comp),
            _ => None,
        }
    }

    pub fn get(&self, id: impl AsRef<str>) -> Option<&Node<M>> {
        let id = id.as_ref();
        match self {
            Node::Prim(prim) if prim.id() == Some(id) => Some(self),
            Node::Prim(prim) => {
                for child in &prim.children {
                    if let Some(node) = child.get(id) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Comp(comp) if comp.id() == Some(id) => Some(self),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Node<M>> {
        let id = id.as_ref();
        match self {
            Node::Prim(prim) if prim.id() == Some(id) => Some(self),
            Node::Prim(prim) => {
                for child in &mut prim.children {
                    if let Some(node) = child.get_mut(id) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Comp(comp) if comp.id() == Some(id) => Some(self),
            _ => None,
        }
    }

    pub fn get_prim(&self, id: impl AsRef<str>) -> Option<&Prim<M>> {
        let id = id.as_ref();
        match self {
            Node::Prim(prim) => {
                if prim.id() == Some(id) {
                    Some(prim)
                } else {
                    for child in &prim.children {
                        if let Some(prim) = child.get_prim(id) {
                            return Some(prim);
                        }
                    }
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_prim_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Prim<M>> {
        let id = id.as_ref();
        match self {
            Node::Prim(prim) => {
                if prim.id() == Some(id) {
                    Some(prim)
                } else {
                    for child in &mut prim.children {
                        if let Some(prim) = child.get_prim_mut(id) {
                            return Some(prim);
                        }
                    }
                    None
                }
            }
            _ => None,
        }
    }

    pub fn get_comp_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Comp> {
        let id = id.as_ref();
        match self {
            Node::Comp(comp) if comp.id() == Some(id) => Some(comp),
            Node::Prim(prim) => {
                for child in &mut prim.children {
                    if let Some(comp) = child.get_comp_mut(id) {
                        return Some(comp);
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        match self {
            Node::Prim(prim) => prim.transform_mut(),
            Node::Comp(comp) => comp.transform_mut(),
        }
    }

    pub fn send_system_msg(&mut self, msg: SystemMessage, outputs: &mut Vec<M::Message>) {
        match self {
            Node::Prim(prim) => prim.send_system_msg(msg, outputs),
            Node::Comp(comp) => comp.send_system_msg(msg),
        }
    }

    pub fn update_view(&mut self) -> UpdateView {
        match self {
            Node::Prim(prim) => prim.update_view(),
            Node::Comp(comp) => comp.update_view(),
        }
    }
}

impl<M: Model> CompositeShape for Node<M> {
    fn shape(&self) -> Option<&Shape> {
        match self {
            Node::Prim(prim) => prim.shape(),
            Node::Comp(comp) => comp.shape(),
        }
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        match self {
            Node::Prim(prim) => prim.shape_mut(),
            Node::Comp(comp) => comp.shape_mut(),
        }
    }

    fn children(&self) -> Option<CompositeShapeIter> {
        match self {
            Node::Prim(prim) => CompositeShape::children(prim),
            Node::Comp(comp) => CompositeShape::children(comp),
        }
    }

    fn children_mut(&mut self) -> Option<CompositeShapeIterMut> {
        match self {
            Node::Prim(prim) => CompositeShape::children_mut(prim),
            Node::Comp(comp) => CompositeShape::children_mut(comp),
        }
    }

    fn need_recalc(&self) -> Option<bool> {
        match self {
            Node::Prim(prim) => CompositeShape::need_recalc(prim),
            Node::Comp(comp) => CompositeShape::need_recalc(comp),
        }
    }

    fn need_redraw(&self) -> Option<bool> {
        match self {
            Node::Prim(prim) => CompositeShape::need_redraw(prim),
            Node::Comp(comp) => CompositeShape::need_redraw(comp),
        }
    }
}
