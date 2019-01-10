use std::any::Any;
use std::fmt::{self, Pointer};
use std::rc::Rc;
use crate::egml::{
    Component, Viewable, ViewableComponent, Drawable, DrawableChilds, DrawableChildsMut,
    Unit, Comp, Shape, Word, Fill, Stroke, Translate, ChangeView,
};
use crate::controller::InputEvent;

pub enum Node<M: Component> {
    Unit(Unit<M>),
    Comp(Comp),
//    List(List),
}

#[derive(Default, Clone)]
pub struct NodeDefaults {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub translate: Option<Translate>,
}

impl<M: Component> Node<M> {
    pub fn input(&mut self, parent_comp: Option<*mut Comp>, event: InputEvent, messages: &mut Vec<M::Message>) {
        match self {
            Node::Unit(ref mut unit) => unit.input(parent_comp, event, messages),
            Node::Comp(ref mut comp) => comp.input(parent_comp, event),
        }
    }

    pub fn unit(&self) -> Option<&Unit<M>> {
        if let Node::Unit(ref unit) = self {
            Some(unit)
        } else {
            None
        }
    }

    pub fn unit_mut(&mut self) -> Option<&mut Unit<M>> {
        if let Node::Unit(ref mut unit) = self {
            Some(unit)
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
}

pub type ChildrenProcessed = bool;

impl<M: ViewableComponent<M>> Node<M> {
    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        match self {
            Node::Unit(ref mut unit) => {
                if !unit.resolve(defaults.as_ref().map(|d| Rc::clone(d))) {
                    for child in unit.childs.iter_mut() {
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

    pub fn modify(&mut self, model: &dyn Any) {
        match self {
            Node::Unit(ref mut unit) => {
                unit.modify(model);
                for child in unit.childs.iter_mut() {
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
            Node::Unit(ref unit) => unit.shape(),
            Node::Comp(ref comp) => comp.shape(),
        }
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        match self {
            Node::Unit(ref mut unit) => unit.shape_mut(),
            Node::Comp(ref mut comp) => comp.shape_mut(),
        }
    }

    fn childs(&self) -> Option<DrawableChilds> {
        match self {
            Node::Unit(ref unit) => Drawable::childs(unit),
            Node::Comp(ref comp) => Drawable::childs(comp),
        }
    }

    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        match self {
            Node::Unit(ref mut unit) => Drawable::childs_mut(unit),
            Node::Comp(ref mut comp) => Drawable::childs_mut(comp),
        }
    }
}

impl<M: Component> From<Unit<M>> for Node<M> {
    fn from(unit: Unit<M>) -> Self {
        Node::Unit(unit)
    }
}

impl<M: Component> From<Comp> for Node<M> {
    fn from(comp: Comp) -> Self {
        Node::Comp(comp)
    }
}

impl<M: Component, T: ToString> From<T> for Node<M> {
    fn from(value: T) -> Self {
        Node::Unit(Unit::new("text", Shape::Word(
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
            Node::Unit(ref unit) => unit.fmt(f),
            Node::Comp(ref _comp) => "Component<>".fmt(f),
//            Node::List(_) => "List<>".fmt(f),
//            Node::Text(ref text) => text.fmt(f),
        }
    }
}
