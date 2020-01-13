use crate::{Model, SystemMessage};
pub use self::{
    comp::*,
    prim::*,
    shape::*,
    value::*,
    converter::*,
    transform::*,
};

pub mod builder;
pub mod comp;
pub mod prim;
pub mod shape;
pub mod value;
pub mod converter;
pub mod transform;

pub enum Node<M: Model> {
    Prim(Prim<M>),
    Comp(Comp),
}

impl<M: Model> Node<M> {
//    pub fn add_child(&mut self, child: Node<M>) {
//        match self {
//            Node::Prim(prim) => prim.children.push(child),
//            Node::Comp(comp) => comp.
//        }
//    }
//
//    pub fn add_children(&mut self, children: impl IntoIterator<Item = Node<M>>) {
//        match self {
//            Node::Prim(prim) => prim.children.extend(children),
//            Node::Comp(comp) => comp.
//        }
//    }

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

    pub fn get_prim_mut(&mut self, id: impl AsRef<str>) -> Option<&mut Prim<M>> {
        let id = id.as_ref();
        match self {
            Node::Prim(prim) => if prim.id() == Some(id) {
                Some(prim)
            } else {
                for child in &mut prim.children {
                    if let Some(prim) = child.get_prim_mut(id) {
                        return Some(prim);
                    }
                }
                None
            },
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
            },
            _ => None,
        }
    }

    pub fn transform_mut(&mut self) -> Option<&mut Transform> {
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

    pub fn update_view(&mut self) {
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
}