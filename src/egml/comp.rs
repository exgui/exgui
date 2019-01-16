use std::mem;
use std::rc::Rc;
use crate::egml::{
    Component, ComponentMessage, ViewableComponent, Drawable, DrawableChilds, DrawableChildsMut,
    AnyModel, AnyMessage, AnyVecMessages, AnyProperties, AnyNode, Node, NodeDefaults, Prim, Shape,
    Finger, ChangeView, ChildrenProcessed, GetError,
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
    pub modifier: Option<fn(&mut Comp, &dyn AnyModel)>,
    pub pass_up_handler: Option<fn(&dyn AnyMessage) -> Box<dyn AnyMessage>>,
    modify_interior_closure: Option<fn(&mut Comp)>,
    update_closure: Option<fn(&mut Comp, &mut dyn AnyVecMessages)>,
    get_comp_closure: Option<fn(*const Comp, Finger) -> Result<*const Comp, GetError>>,
    get_comp_mut_closure: Option<fn(*mut Comp, Finger) -> Result<*mut Comp, GetError>>,
}

trait CompInside {
    fn modify_interior<SelfModel>(&mut self)
    where SelfModel: ViewableComponent<SelfModel>;

    fn change_view_if_necessary<SelfModel>(&mut self, should_change: ChangeView)
    where SelfModel: ViewableComponent<SelfModel>;
}

impl Comp {
    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn lazy<SelfModel>() -> (SelfModel::Properties, Self)
    where
        SelfModel: ViewableComponent<SelfModel>,
    {
        (Default::default(), Default::default())
    }

    pub fn new<SelfModel>(props: SelfModel::Properties) -> Self
    where
        SelfModel: ViewableComponent<SelfModel>,
    {
        let mut comp = Comp::default();
        comp.init::<SelfModel>(props);
        comp
    }

    /// Create model and attach properties associated with the component.
    pub fn init<SelfModel>(&mut self, props: SelfModel::Properties)
        where
            SelfModel: ViewableComponent<SelfModel>,
    {
        let model = SelfModel::create(&props);
        let node = model.view();
        self.model = Some(Box::new(model));
        self.view_node = Some(Box::new(node));
        self.props = Some(Box::new(props));
        self.resolver = Some(|comp: &mut Comp| {
            let defaults = comp.cloned_defaults();
            comp.view_node_mut::<SelfModel>().resolve(defaults)
        });
        self.drawer = Some(|comp: &Comp| {
            comp.view_node::<SelfModel>() as &dyn Drawable
        });
        self.drawer_mut = Some(|comp: &mut Comp| {
            comp.view_node_mut::<SelfModel>() as &mut dyn Drawable
        });

        self.inputer = Some(|comp: &mut Comp, event: InputEvent, _parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<SelfModel>()
                .input(event, &mut messages);
            comp.update_msgs(messages);
        });

        self.modify_interior_closure = Some(|comp: &mut Comp| {
            comp.modify_interior::<SelfModel>();
        });

        self.update_closure = Some(|comp: &mut Comp, messages: &mut dyn AnyVecMessages| {
            let messages = messages.as_any_mut().downcast_mut::<Vec<SelfModel::Message>>()
                .expect("Can't downcast AnyVecMessages to Vec<Message> for update");
            let mut should_change = ChangeView::None;
            for msg in messages.drain(..) {
                should_change.up(comp.model_mut::<SelfModel>().update(msg));
            }
            comp.change_view_if_necessary::<SelfModel>(should_change);
        });

        self.get_comp_closure = Some(|comp: *const Comp, finger: Finger| -> Result<*const Comp, GetError> {
            unsafe { &*comp }.view_node::<SelfModel>().get_comp(finger)
                .map(|c| c as *const _)
        });

        self.get_comp_mut_closure = Some(|comp: *mut Comp, finger: Finger| -> Result<*mut Comp, GetError> {
            unsafe { &mut *comp }.view_node_mut::<SelfModel>().get_comp_mut(finger)
                .map(|c| c as *mut _)
        });
    }

    pub fn init_viewable<PM, SelfModel>(&mut self)
        where
            PM: ViewableComponent<PM>,
            SelfModel: ViewableComponent<SelfModel>,
    {
        self.inputer = Some(|comp: &mut Comp, event: InputEvent, parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<SelfModel>()
                .input(event, &mut messages);
            let to_parent_messages = Self::send_pass_up::<PM, SelfModel, _>(comp, messages);
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

    pub fn get_comp<'a>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>> {
        let getter = self.get_comp_closure.expect("Get comp closure must be not None");
        getter(self as *const _, finger).map(|p| unsafe { &*p })
    }

    pub fn get_comp_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>> {
        let getter = self.get_comp_mut_closure.expect("Get comp mut closure must be not None");
        getter(self as *mut _, finger).map(|p| unsafe { &mut *p })
    }

    pub fn get_prim<'a, SelfModel: Component>(&self, finger: Finger<'a>) -> Result<&Prim<SelfModel>, GetError<'a>> {
        self.view_node::<SelfModel>().get_prim(finger)
    }

    pub fn get_prim_mut<'a, SelfModel: Component>(&mut self, finger: Finger<'a>) -> Result<&mut Prim<SelfModel>, GetError<'a>> {
        self.view_node_mut::<SelfModel>().get_prim_mut(finger)
    }

    pub fn view_node<SelfModel: Component>(&self) -> &Node<SelfModel> {
        let node = self.view_node.as_ref().expect("Can't downcast node - it is None");
        node.as_any().downcast_ref::<Node<SelfModel>>().expect("Can't downcast node")
    }

    pub fn view_node_mut<SelfModel: Component>(&mut self) -> &mut Node<SelfModel> {
        let node = self.view_node.as_mut().expect("Can't downcast node - it is None");
        node.as_any_mut().downcast_mut::<Node<SelfModel>>().expect("Can't downcast node")
    }

    pub fn model<SelfModel: Component>(&self) -> &SelfModel {
        let model = self.model.as_ref().expect("Can't downcast model - it is None");
        model.as_any().downcast_ref::<SelfModel>().expect("Can't downcast model")
    }

    pub fn model_mut<SelfModel: Component>(&mut self) -> &mut SelfModel {
        let model = self.model.as_mut().expect("Can't downcast model - it is None");
        model.as_any_mut().downcast_mut::<SelfModel>().expect("Can't downcast model")
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
        self.modify_content();
    }

    #[inline]
    pub fn modify_content(&mut self) {
        self.modify_interior_closure.map(|modifier| {
            modifier(self);
        });
    }

    pub fn pass_up<M: Component>(&mut self, msg: &dyn AnyMessage) -> Option<M::Message> {
        self.pass_up_handler.map(|pass_up_handler| {
            *pass_up_handler(msg).into_any().downcast::<M::Message>()
                .expect("Can't downcast pass up msg")
        })
    }

    #[inline]
    pub fn send_self<SelfModelMessage: ComponentMessage>(&mut self, msg: SelfModelMessage) {
        self.send_self_batch(vec![msg])
    }

    pub fn send_self_batch<Msgs>(&mut self, msgs: Msgs)
    where
        Msgs: AnyVecMessages,
    {
        self.update_msgs(msgs);
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
        let comp = self.get_comp_mut(to_child)
            .expect("Send batch: Comp not found");
        let parent_msgs = Self::send_pass_up::<M, CM, MS>(comp, msgs);
        self.update_msgs(parent_msgs);
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
            comp.send_self(msg);
            if let Some(parent_msg) = parent_msg {
                parent_msgs.push(parent_msg);
            }
        }
        parent_msgs
    }

    #[inline]
    fn update_msgs<Msgs>(&mut self, mut msgs: Msgs)
    where
        Msgs: AnyVecMessages,
    {
        let updater = self.update_closure.expect("Update closure must be not None");
        updater(self, &mut msgs);
    }
}

impl CompInside for Comp {
    fn modify_interior<SelfModel>(&mut self)
    where
        SelfModel: ViewableComponent<SelfModel>,
    {
        let boxed_model = mem::replace(&mut self.model, None)
            .expect("Can't extract model for modify interior");
        self.view_node_mut::<SelfModel>().modify(&(*boxed_model));
        mem::replace(&mut self.model, Some(boxed_model));
    }

    fn change_view_if_necessary<SelfModel>(&mut self, should_change: ChangeView)
    where
        SelfModel: ViewableComponent<SelfModel>,
    {
        match should_change {
            ChangeView::Rebuild => {
                let mut new_node = self.model::<SelfModel>().view();
                new_node.resolve(self.cloned_defaults());
                self.view_node = Some(Box::new(new_node));
            },
            ChangeView::Modify => {
                self.modify_interior::<SelfModel>();
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