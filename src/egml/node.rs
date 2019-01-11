use std::fmt::{self, Pointer};
use std::rc::Rc;
use crate::egml::{
    Component, Viewable, ViewableComponent, Drawable, DrawableChilds, DrawableChildsMut, AnyModel,
    AnyVecMessages, Prim, Comp, Shape, Word, Fill, Stroke, Translate, Finger, ChangeView,
};
use crate::controller::InputEvent;

pub enum Node<M: Component> {
    Prim(Prim<M>),
    Comp(Comp),
//    List(List),
}

#[derive(Default, Clone)]
pub struct NodeDefaults {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub translate: Option<Translate>,
}

#[derive(Debug, Copy, Clone)]
pub enum GetError<'a> {
    TargetIsPrim,
    TargetIsComp,
    IdxOutOfBounds {
        idx: usize,
        len: usize,
        tail: &'a [usize]
    },
    LinkToCompInsteadPrim {
        tail: &'a [usize]
    },
    NotFound,
}

impl<M: Component> Node<M> {
    pub fn input(&mut self, event: InputEvent, messages: &mut Vec<M::Message>) {
        match self {
            Node::Prim(ref mut prim) => prim.input(event, messages),
            Node::Comp(ref mut comp) => comp.input(event, Some(messages as &mut dyn AnyVecMessages)),
        }
    }

    pub fn prim(&self) -> Option<&Prim<M>> {
        if let Node::Prim(ref prim) = self {
            Some(prim)
        } else {
            None
        }
    }

    pub fn prim_mut(&mut self) -> Option<&mut Prim<M>> {
        if let Node::Prim(ref mut prim) = self {
            Some(prim)
        } else {
            None
        }
    }

    pub fn comp(&self) -> Option<&Comp> {
        if let Node::Comp(ref comp) = self {
            Some(comp)
        } else {
            None
        }
    }

    pub fn comp_mut(&mut self) -> Option<&mut Comp> {
        if let Node::Comp(ref mut comp) = self {
            Some(comp)
        } else {
            None
        }
    }

    pub fn get_comp<'a>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>> {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                match self {
                    Node::Prim(_) => Err(GetError::TargetIsPrim),
                    Node::Comp(comp) => Ok(comp)
                }
            },
            Finger::Location(loc) => {
                match self {
                    Node::Prim(prim) => {
                        let idx = loc[0];
                        let len = prim.childs.len();
                        match prim.childs.get(idx) {
                            Some(node) => {
                                node.get_comp(Finger::Location(&loc[1..]))
                            },
                            None => Err(GetError::IdxOutOfBounds { idx, len, tail: loc }),
                        }
                    },
                    Node::Comp(_) => Err(GetError::LinkToCompInsteadPrim { tail: loc }),
                }
            },
            Finger::Id(id) => {
                let not_found = GetError::NotFound;
                match self {
                    Node::Prim(prim) => {
                        for child in prim.childs.iter() {
                            let result = child.get_comp(finger);
                            if result.is_ok() {
                                return result;
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(comp) => if id == comp.id().ok_or(not_found)? {
                        Ok(comp)
                    } else {
                        Err(not_found)
                    }
                }
            },
        }
    }

    pub fn get_comp_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>> {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                match self {
                    Node::Prim(_) => Err(GetError::TargetIsPrim),
                    Node::Comp(comp) => Ok(comp)
                }
            },
            Finger::Location(loc) => {
                match self {
                    Node::Prim(prim) => {
                        let idx = loc[0];
                        let len = prim.childs.len();
                        match prim.childs.get_mut(idx) {
                            Some(node) => {
                                node.get_comp_mut(Finger::Location(&loc[1..]))
                            },
                            None => Err(GetError::IdxOutOfBounds { idx, len, tail: loc }),
                        }
                    },
                    Node::Comp(_) => Err(GetError::LinkToCompInsteadPrim { tail: loc }),
                }
            },
            Finger::Id(id) => {
                let not_found = GetError::NotFound;
                match self {
                    Node::Prim(prim) => {
                        for child in prim.childs.iter_mut() {
                            let result = child.get_comp_mut(finger);
                            if result.is_ok() {
                                return result;
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(comp) => if id == comp.id().ok_or(not_found)? {
                        Ok(comp)
                    } else {
                        Err(not_found)
                    }
                }
            },
        }
    }

    pub fn get_prim<'a>(&self, finger: Finger<'a>) -> Result<&Prim<M>, GetError<'a>> {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                match self {
                    Node::Prim(prim) => Ok(prim),
                    Node::Comp(_) => Err(GetError::TargetIsComp),
                }
            },
            Finger::Location(loc) => {
                match self {
                    Node::Prim(prim) => {
                        let idx = loc[0];
                        let len = prim.childs.len();
                        match prim.childs.get(idx) {
                            Some(node) => {
                                node.get_prim(Finger::Location(&loc[1..]))
                            },
                            None => Err(GetError::IdxOutOfBounds { idx, len, tail: loc }),
                        }
                    },
                    Node::Comp(_) => Err(GetError::LinkToCompInsteadPrim { tail: loc }),
                }
            },
            Finger::Id(id) => {
                let not_found = GetError::NotFound;
                match self {
                    Node::Prim(prim) => {
                        if let Some(prim_id) = prim.shape.id() {
                            if id == prim_id {
                                return Ok(prim);
                            }
                        }

                        for child in prim.childs.iter() {
                            let result = child.get_prim(finger);
                            if result.is_ok() {
                                return result;
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(_) => Err(not_found),
                }
            },
        }
    }

    pub fn get_prim_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Prim<M>, GetError<'a>> {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                match self {
                    Node::Prim(prim) => Ok(prim),
                    Node::Comp(_) => Err(GetError::TargetIsComp),
                }
            },
            Finger::Location(loc) => {
                match self {
                    Node::Prim(prim) => {
                        let idx = loc[0];
                        let len = prim.childs.len();
                        match prim.childs.get_mut(idx) {
                            Some(node) => {
                                node.get_prim_mut(Finger::Location(&loc[1..]))
                            },
                            None => Err(GetError::IdxOutOfBounds { idx, len, tail: loc }),
                        }
                    },
                    Node::Comp(_) => Err(GetError::LinkToCompInsteadPrim { tail: loc }),
                }
            },
            Finger::Id(id) => {
                let not_found = GetError::NotFound;
                match self {
                    Node::Prim(prim) => {
                        if let Some(prim_id) = prim.shape.id() {
                            if id == prim_id {
                                return Ok(prim);
                            }
                        }

                        for child in prim.childs.iter_mut() {
                            let result = child.get_prim_mut(finger);
                            if result.is_ok() {
                                return result;
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(_) => Err(not_found),
                }
            },
        }
    }
}

pub type ChildrenProcessed = bool;

impl<M: ViewableComponent<M>> Node<M> {
    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        match self {
            Node::Prim(ref mut prim) => {
                if !prim.resolve(defaults.as_ref().map(|d| Rc::clone(d))) {
                    for child in prim.childs.iter_mut() {
                        child.resolve(defaults.as_ref().map(|d| Rc::clone(d)));
                    }
                }
                true
            }
            Node::Comp(ref mut comp) => {
                comp.resolve(defaults);
                false
            }
        }
    }

    pub fn modify(&mut self, model: &dyn AnyModel) {
        match self {
            Node::Prim(ref mut prim) => {
                prim.modify(model);
                for child in prim.childs.iter_mut() {
                    child.modify(model);
                }
            }
            Node::Comp(ref mut comp) => {
                comp.modify(Some(model));
            }
        }
    }

    #[inline]
    pub fn send_self(&mut self, model: &mut M, msg: M::Message) {
        self.send_self_batch(model, Some(msg));
    }

    pub fn send_self_batch<MS>(&mut self, model: &mut M, msgs: MS)
        where
            MS: IntoIterator<Item = M::Message>,
    {
        let mut should_change = ChangeView::None;
        for msg in msgs.into_iter() {
            should_change.up(model.update(msg));
        }
        match should_change {
            ChangeView::Rebuild => {
                let mut new_node = model.view();
                new_node.resolve(None);
                *self = new_node;
            },
            ChangeView::Modify => {
                self.modify(model);
            },
            ChangeView::None => (),
        }
    }
}

impl<M: Component> Drawable for Node<M> {
    fn shape(&self) -> Option<&Shape> {
        match self {
            Node::Prim(ref prim) => prim.shape(),
            Node::Comp(ref comp) => comp.shape(),
        }
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        match self {
            Node::Prim(ref mut prim) => prim.shape_mut(),
            Node::Comp(ref mut comp) => comp.shape_mut(),
        }
    }

    fn childs(&self) -> Option<DrawableChilds> {
        match self {
            Node::Prim(ref prim) => Drawable::childs(prim),
            Node::Comp(ref comp) => Drawable::childs(comp),
        }
    }

    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        match self {
            Node::Prim(ref mut prim) => Drawable::childs_mut(prim),
            Node::Comp(ref mut comp) => Drawable::childs_mut(comp),
        }
    }
}

pub trait Nodeable: 'static {}
impl<M: Component> Nodeable for Node<M> {}

impl<M: Component> From<Prim<M>> for Node<M> {
    fn from(prim: Prim<M>) -> Self {
        Node::Prim(prim)
    }
}

impl<M: Component> From<Comp> for Node<M> {
    fn from(comp: Comp) -> Self {
        Node::Comp(comp)
    }
}

impl<M: Component, T: ToString> From<T> for Node<M> {
    fn from(value: T) -> Self {
        Node::Prim(Prim::new("text", Shape::Word(
            Word { content: value.to_string(), ..Default::default() }
        )))
    }
}

impl<'a, M: Component> From<&'a dyn Viewable<M>> for Node<M> {
    fn from(value: &'a dyn Viewable<M>) -> Self {
        value.view()
    }
}

impl<M: Component> fmt::Debug for Node<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Node::Prim(ref prim) => prim.fmt(f),
            Node::Comp(ref _comp) => "Component<>".fmt(f),
//            Node::List(_) => "List<>".fmt(f),
//            Node::Text(ref text) => text.fmt(f),
        }
    }
}
