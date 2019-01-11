use std::mem;
use std::rc::Rc;
use crate::egml::{
    Component, ViewableComponent, Drawable, DrawableChilds, DrawableChildsMut, AnyModel, AnyMessage,
    AnyVecMessages, AnyProperties, AnyNode, Node, NodeDefaults, Prim, Shape, Finger, ChangeView,
    ChildrenProcessed, GetError,
};
use crate::controller::InputEvent;

#[derive(Default)]
pub struct Comp {
    pub id: Option<String>,
    pub model: Option<Box<dyn AnyModel>>,
    pub props: Option<Box<dyn AnyProperties>>,
    pub view_node: Option<Box<dyn AnyNode>>,
    pub defaults: Option<Rc<NodeDefaults>>,
    pub resolver: Option<fn(&mut Comp) -> ChildrenProcessed>,
    pub drawer: Option<fn(&Comp) -> &dyn Drawable>,
    pub drawer_mut: Option<fn(&mut Comp) -> &mut dyn Drawable>,
    pub inputer: Option<fn(&mut Comp, InputEvent, Option<&mut dyn AnyVecMessages>)>,
    pub modify_handler: Option<fn(&mut Comp)>,
    pub modifier: Option<fn(&mut Comp, &dyn AnyModel)>,
    pub pass_up_handler: Option<fn(&dyn AnyMessage) -> Box<dyn AnyMessage>>,
}

impl Comp {
    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

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
        let model = M::create(&props);
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
        self.inputer = Some(|comp: &mut Comp, event: InputEvent, _parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<M>()
                .input(event, &mut messages);
            comp.update_msgs::<M, _>(messages);
        });
    }

    pub fn init_viewable<PM, M>(&mut self)
        where
            PM: ViewableComponent<PM>,
            M: ViewableComponent<M>,
    {
        self.inputer = Some(|comp: &mut Comp, event: InputEvent, parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<M>()
                .input(event, &mut messages);
            let to_parent_messages = Self::send_pass_up::<PM, M, _>(comp, messages);
            if let Some(parent_messages) = parent_messages {
                let parent_messages = parent_messages.as_any_mut().downcast_mut::<Vec<PM::Message>>()
                    .expect("Inputer can't downcast Vec<PM::Message>");
                for msg in to_parent_messages.into_iter() {
                    parent_messages.push(msg);
                }
            }
        });
    }

    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        self.defaults = defaults;
        (self.resolver.expect("Can't resolve with uninitialized resolver"))(
            self
        )
    }

    pub fn get_comp<'a, M: Component>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>> {
        self.view_node::<M>().get_comp(finger)
    }

    pub fn get_comp_mut<'a, M: Component>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>> {
        self.view_node_mut::<M>().get_comp_mut(finger)
    }

    pub fn get_prim<'a, M: Component>(&self, finger: Finger<'a>) -> Result<&Prim<M>, GetError<'a>> {
        self.view_node::<M>().get_prim(finger)
    }

    pub fn get_prim_mut<'a, M: Component>(&mut self, finger: Finger<'a>) -> Result<&mut Prim<M>, GetError<'a>> {
        self.view_node_mut::<M>().get_prim_mut(finger)
    }

    pub fn view_node<M: Component>(&self) -> &Node<M> {
        let node = self.view_node.as_ref().expect("Can't downcast node - it is None");
        node.as_any().downcast_ref::<Node<M>>().expect("Can't downcast node")
    }

    pub fn view_node_mut<M: Component>(&mut self) -> &mut Node<M> {
        let node = self.view_node.as_mut().expect("Can't downcast node - it is None");
        node.as_any_mut().downcast_mut::<Node<M>>().expect("Can't downcast node")
    }

    pub fn model<M: Component>(&self) -> &M {
        let model = self.model.as_ref().expect("Can't downcast model - it is None");
        model.as_any().downcast_ref::<M>().expect("Can't downcast model")
    }

    pub fn model_mut<M: Component>(&mut self) -> &mut M {
        let model = self.model.as_mut().expect("Can't downcast model - it is None");
        model.as_any_mut().downcast_mut::<M>().expect("Can't downcast model")
    }

    pub fn input(&mut self, event: InputEvent, messages: Option<&mut dyn AnyVecMessages>) {
        self.inputer.map(|inputer| {
            inputer(self, event, messages)
        });
    }

    pub fn cloned_defaults(&self) -> Option<Rc<NodeDefaults>> {
        self.defaults.as_ref().map(|d| Rc::clone(d))
    }

    pub fn modify(&mut self, model: Option<&dyn AnyModel>) {
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

    pub fn pass_up<M: Component>(&mut self, msg: &dyn AnyMessage) -> Option<M::Message> {
        self.pass_up_handler.map(|pass_up_handler| {
            *pass_up_handler(msg).into_any().downcast::<M::Message>()
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
        let comp = self.get_comp_mut::<M>(to_child)
            .expect("Send batch: Comp not found");
        let parent_msgs = Self::send_pass_up::<M, CM, MS>(comp, msgs);
        self.update_msgs::<M, _>(parent_msgs);
    }

    fn send_pass_up<M, CM, CMS>(comp: &mut Comp, msgs: CMS) -> Vec<M::Message>
        where
            M: ViewableComponent<M>,
            CM: ViewableComponent<CM>,
            CMS: IntoIterator<Item = CM::Message>,
    {
        let mut parent_msgs = Vec::new();
        for msg in msgs.into_iter() {
            let parent_msg = comp.pass_up::<M>(&msg);
            comp.send_self::<CM>(msg);
            if let Some(parent_msg) = parent_msg {
                parent_msgs.push(parent_msg);
            }
        }
        parent_msgs
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