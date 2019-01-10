use std::mem;
use std::any::Any;
use std::rc::Rc;
use crate::egml::{
    Component, ChangeView, ViewableComponent, Drawable, DrawableChilds, DrawableChildsMut,
    Node, NodeDefaults, Prim, Shape, ChildrenProcessed,
};
use crate::controller::InputEvent;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Finger<'a> {
    Id(&'a str),
    Location(&'a [usize]),
    None,
}

#[derive(Default)]
pub struct Comp {
    pub id: Option<String>,
    pub model: Option<Box<dyn Any>>,
    pub props: Option<Box<dyn Any>>,
    pub view_node: Option<Box<dyn Any>>,
    pub defaults: Option<Rc<NodeDefaults>>,
    pub resolver: Option<fn(&mut Comp) -> ChildrenProcessed>,
    pub drawer: Option<fn(&Comp) -> &dyn Drawable>,
    pub drawer_mut: Option<fn(&mut Comp) -> &mut dyn Drawable>,
    pub inputer: Option<fn(*mut Comp, &mut Comp, InputEvent)>,
    pub modify_handler: Option<fn(&mut Comp)>,
    pub modifier: Option<fn(&mut Comp, &dyn Any)>,
    pub pass_up_handler: Option<fn(&dyn Any) -> Box<dyn Any>>,
}

impl Comp {
    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn lazy<M>() -> (M::Properties, Self)
        where
            M: ViewableComponent<M>,
    {
        (Default::default(), Default::default())
    }

    pub fn new<M>(props: M::Properties) -> Self
        where
            M: ViewableComponent<M>,
    {
        let mut comp = Comp::default();
        comp.init::<M>(props);
        comp
    }

    /// Create model and attach properties associated with the component.
    pub fn init<M>(&mut self, props: M::Properties)
        where
            M: ViewableComponent<M>,
    {
        let model = <M as Component>::create(&props);
        let node = model.view();
        self.model = Some(Box::new(model));
        self.view_node = Some(Box::new(node));
        self.props = Some(Box::new(props));
        self.resolver = Some(|comp: &mut Comp| {
            let defaults = comp.cloned_defaults();
            comp.view_node_mut::<M>().resolve(defaults)
        });
        self.drawer = Some(|comp: &Comp| {
            comp.view_node::<M>() as &dyn Drawable
        });
        self.drawer_mut = Some(|comp: &mut Comp| {
            comp.view_node_mut::<M>() as &mut dyn Drawable
        });
        self.modify_handler = Some(|comp: &mut Comp| {
            let boxed_model = mem::replace(&mut comp.model, None)
                .expect("Modifier can't extract model");
            comp.view_node_mut::<M>().modify(&(*boxed_model));
            mem::replace(&mut comp.model, Some(boxed_model));
        });
        self.inputer = Some(|_parent_comp: *mut Comp, comp: &mut Comp, event: InputEvent| {
            let mut messages = Vec::new();
            let comp_ptr = comp as *mut Comp;
            comp.view_node_mut::<M>()
                .input(Some(comp_ptr), event, &mut messages);
            comp.update_msgs::<M, _>(messages);
        });
    }

    pub fn init_viewable<PM, M>(&mut self)
        where
            PM: ViewableComponent<PM>,
            M: ViewableComponent<M>,
    {
        self.inputer = Some(|parent_comp: *mut Comp, comp: &mut Comp, event: InputEvent| {
            let mut messages = Vec::new();
            let comp_ptr = comp as *mut Comp;
            comp.view_node_mut::<M>()
                .input(Some(comp_ptr), event, &mut messages);
            Self::send_pass_up::<PM, M, _>(parent_comp, comp, messages);
        });
    }

    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        self.defaults = defaults;
        (self.resolver.expect("Can't resolve with uninitialized resolver"))(
            self
        )
    }

    pub fn view_node<M: Component>(&self) -> &Node<M> {
        let node = self.view_node.as_ref().expect("Can't downcast node - it is None");
        (*(*node)).downcast_ref::<Node<M>>().expect("Can't downcast node")
    }

    pub fn view_node_mut<M: Component>(&mut self) -> &mut Node<M> {
        let node = self.view_node.as_mut().expect("Can't downcast node - it is None");
        (*(*node)).downcast_mut::<Node<M>>().expect("Can't downcast node")
    }

    pub fn model<M: Component>(&self) -> &M {
        let model = self.model.as_ref().expect("Can't downcast model - it is None");
        (*(*model)).downcast_ref::<M>().expect("Can't downcast model")
    }

    pub fn model_mut<M: Component>(&mut self) -> &mut M {
        let model = self.model.as_mut().expect("Can't downcast model - it is None");
        (*(*model)).downcast_mut::<M>().expect("Can't downcast model")
    }

    pub fn input(&mut self, parent_comp: Option<*mut Comp>, event: InputEvent) {
        self.inputer.map(|inputer| {
            inputer(parent_comp.unwrap_or(self as *mut Comp), self, event)
        });
    }

    pub fn cloned_defaults(&self) -> Option<Rc<NodeDefaults>> {
        self.defaults.as_ref().map(|d| Rc::clone(d))
    }

    pub fn modify(&mut self, model: Option<&dyn Any>) {
        self.modifier.map(|modifier| {
            modifier(self, model.expect("Call `Comp::modify` without model, but modifier is exists"))
        });
        self.modify_internal();
    }

    pub fn modify_internal(&mut self) {
        self.modify_handler.map(|modifier| {
            modifier(self)
        });
    }

    pub fn pass_up<M: Component>(&mut self, msg: &dyn Any) -> Option<M::Message> {
        self.pass_up_handler.map(|pass_up_handler| {
            *pass_up_handler(msg).downcast::<M::Message>()
                .expect("Can't downcast pass up msg")
        })
    }

    #[inline]
    pub fn send_self<M: ViewableComponent<M>>(&mut self, msg: M::Message) {
        self.send_self_batch::<M, _>(Some(msg));
    }

    pub fn send_self_batch<M, MS>(&mut self, msgs: MS)
        where
            M: ViewableComponent<M>,
            MS: IntoIterator<Item = M::Message>,
    {
        let mut should_change = ChangeView::None;
        for msg in msgs.into_iter() {
            should_change.up(self.model_mut::<M>().update(msg));
        }
        self.change_if_necessary::<M>(should_change);
    }

    #[inline]
    pub fn send<M, CM>(&mut self, to_child: Finger, msg: CM::Message)
        where
            M: ViewableComponent<M>,
            CM: ViewableComponent<CM>,
    {
        self.send_batch::<M, CM, _>(to_child, Some(msg))
    }

    pub fn send_batch<M, CM, MS>(&mut self, to_child: Finger, msgs: MS)
        where
            M: ViewableComponent<M>,
            CM: ViewableComponent<CM>,
            MS: IntoIterator<Item = CM::Message>,
    {
        match to_child {
            Finger::None | Finger::Location(&[]) => {
                self.update_msgs::<CM, _>(msgs);
            },
            Finger::Location(loc) => {
                let this = self as *mut Self;
                match self.view_node_mut::<M>() {
                    Node::Prim(prim) => Self::send_prim::<M, CM, MS>(this, prim, Finger::Location(loc), msgs),
                    Node::Comp(_) => panic!("Wrong location tail: {:?}, link to Comp instead Prim", loc),
                }
            },
            Finger::Id(id) => (),
        }
    }

    fn send_prim<M, CM, MS>(this: *mut Self, prim: &mut Prim<M>, to_child: Finger, msgs: MS)
        where
            M: ViewableComponent<M>,
            CM: ViewableComponent<CM>,
            MS: IntoIterator<Item = CM::Message>,
    {
        match to_child {
            Finger::None | Finger::Location(&[]) => {
                panic!("Wrong location, link to Prim instead Comp");
            },
            Finger::Location(loc) => {
                let idx = loc[0];
                let len = prim.childs.len();
                match prim.childs.get_mut(idx) {
                    Some(Node::Prim(ref mut prim)) => {
                        Self::send_prim::<M, CM, MS>(this, prim, Finger::Location(&loc[1..]), msgs)
                    },
                    Some(Node::Comp(ref mut comp)) => {
                        if len == 1 {
                            Self::send_pass_up::<M, CM, MS>(this, comp, msgs);
                        } else {
                            comp.send_batch::<M, CM, MS>(Finger::Location(&loc[1..]), msgs);
                        }
                    },
                    None => panic!("Wrong location tail: {:?}, idx {} out of bounds {}", loc, idx, len),
                }
            },
            Finger::Id(id) => (),
        }
    }

    fn send_pass_up<M, CM, MS>(parent_comp: *mut Comp, comp: &mut Comp, msgs: MS)
        where
            M: ViewableComponent<M>,
            CM: ViewableComponent<CM>,
            MS: IntoIterator<Item = CM::Message>,
    {
        let parent_comp = unsafe {&mut *parent_comp};
        for msg in msgs.into_iter() {
            let parent_msg = comp.pass_up::<M>(&msg);
            parent_msg.as_ref().map(|parent_msg| {
                let should_change = parent_comp
                    .model_mut::<M>()
                    .before_child_update(parent_msg.clone());
                parent_comp.change_if_necessary::<M>(should_change);
            });
            comp.send_self::<CM>(msg);
            if let Some(parent_msg) = parent_msg {
                let should_change = parent_comp
                    .model_mut::<M>()
                    .after_child_update(parent_msg);
                parent_comp.change_if_necessary::<M>(should_change);
            }
        }
    }

    fn update_msgs<M, MS>(&mut self, msgs: MS)
        where
            M: ViewableComponent<M>,
            MS: IntoIterator<Item = M::Message>,
    {
        let mut should_change = ChangeView::None;
        for msg in msgs.into_iter() {
            should_change.up(self.model_mut::<M>().update(msg));
        }
        self.change_if_necessary::<M>(should_change);
    }

    fn change_if_necessary<M: ViewableComponent<M>>(&mut self, should_change: ChangeView) {
        match should_change {
            ChangeView::Rebuild => {
                let mut new_node = self.model::<M>().view();
                new_node.resolve(self.cloned_defaults());
                self.view_node = Some(Box::new(new_node));
            },
            ChangeView::Modify => {
                self.modify_internal();
            },
            ChangeView::None => (),
        }
    }
}

impl Drawable for Comp {
    fn shape(&self) -> Option<&Shape> {
        self.drawer.and_then(|drawer| {
            drawer(self).shape()
        })
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        let drawer = self.drawer_mut?;
        drawer(self).shape_mut()
    }

    fn childs(&self) -> Option<DrawableChilds> {
        self.drawer.and_then(|drawer| {
            drawer(self).childs()
        })
    }

    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        let drawer = self.drawer_mut?;
        drawer(self).childs_mut()
    }
}

/// Converts property and attach lazy components to it.
pub trait Transformer<M: Component, FROM, TO> {
    /// Transforms one type to another.
    fn transform(&mut self, from: FROM) -> TO;
}

impl<M, T> Transformer<M, T, T> for Comp
    where
        M: Component,
{
    fn transform(&mut self, from: T) -> T {
        from
    }
}

impl<'a, M, T> Transformer<M, &'a T, T> for Comp
    where
        M: Component,
        T: Clone,
{
    fn transform(&mut self, from: &'a T) -> T {
        from.clone()
    }
}

impl<M, T> Transformer<M, T, Option<T>> for Comp
    where
        M: Component,
{
    fn transform(&mut self, from: T) -> Option<T> {
        Some(from)
    }
}

impl<'a, M> Transformer<M, &'a str, String> for Comp
    where
        M: Component,
{
    fn transform(&mut self, from: &'a str) -> String {
        from.to_owned()
    }
}

impl<'a, M> Transformer<M, &'a str, Option<String>> for Comp
    where
        M: Component,
{
    fn transform(&mut self, from: &'a str) -> Option<String> {
        Some(from.to_owned())
    }
}