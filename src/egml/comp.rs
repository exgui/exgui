use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use crate::egml::{
    Component, ComponentMessage, Drawable, DrawableChilds, DrawableChildsMut,
    AsAny, AnyModel, AnyMessage, AnyVecMessages, Node, NodeDefaults, Prim, Shape, Finger,
    ChangeView, ChildrenProcessed, GetError
};
use crate::controller::InputEvent;

pub struct Comp {
    pub inner: Box<dyn CompApi>,
    pub modifier: Option<fn(&mut Comp, &dyn AnyModel)>,
}

impl Comp {
    #[inline]
    pub fn new<SelfModel: Component>(props: SelfModel::Properties) -> Self {
        Comp::from(CompInner::<SelfModel>::new(props))
    }

    /// This method prepares a generator to make a new instance of the `Component`.
    #[inline]
    pub fn lazy<SelfModel: Component>() -> (SelfModel::Properties, Self) {
        (Default::default(), Comp::from(CompInner::<SelfModel>::default()))
    }

    #[inline]
    pub fn inner<SelfModel: Component>(&self) -> &CompInner<SelfModel> {
        (*self.inner)
            .as_any()
            .downcast_ref::<CompInner<SelfModel>>()
            .expect("Can't downcast CompInner")
    }

    #[inline]
    pub fn inner_mut<SelfModel: Component>(&mut self) -> &mut CompInner<SelfModel> {
        (*self.inner)
            .as_any_mut()
            .downcast_mut::<CompInner<SelfModel>>()
            .expect("Can't downcast mut CompInner")
    }

    #[inline]
    pub fn model<SelfModel: Component>(&self) -> &SelfModel {
        self.inner::<SelfModel>().model()
    }

    #[inline]
    pub fn model_mut<SelfModel: Component>(&mut self) -> &mut SelfModel {
        self.inner_mut::<SelfModel>().model_mut()
    }

    #[inline]
    pub fn view_node<SelfModel: Component>(&self) -> &Node<SelfModel> {
        self.inner::<SelfModel>().view_node()
    }

    #[inline]
    pub fn view_node_mut<SelfModel: Component>(&mut self) -> &mut Node<SelfModel> {
        self.inner_mut::<SelfModel>().view_node_mut()
    }

    #[inline]
    pub fn get_prim<'a, SelfModel: Component>(&self, finger: Finger<'a>) -> Result<&Prim<SelfModel>, GetError<'a>> {
        self.view_node::<SelfModel>().get_prim(finger)
    }

    #[inline]
    pub fn get_prim_mut<'a, SelfModel: Component>(&mut self, finger: Finger<'a>) -> Result<&mut Prim<SelfModel>, GetError<'a>> {
        self.view_node_mut::<SelfModel>().get_prim_mut(finger)
    }

    pub fn modify(&mut self, model: Option<&dyn AnyModel>) {
        self.modifier.map(|modifier| {
            modifier(self, model.expect("Call `Comp::modify` without model, but modifier is exists"))
        });
        self.modify_content();
    }

    #[inline]
    pub fn send_self<SelfModelMessage: ComponentMessage>(&mut self, msg: SelfModelMessage) {
        self.send_self_batch(vec![msg])
    }

    #[inline]
    pub fn send_self_batch<SelfModelMessages>(&mut self, mut msgs: SelfModelMessages)
        where
            SelfModelMessages: AnyVecMessages,
    {
        self.update(&mut msgs);
    }

    #[inline]
    pub fn send<'a, TargetMessage>(&mut self, to_child: Finger<'a>, msg: TargetMessage)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    where
        TargetMessage: ComponentMessage,
    {
        self.send_batch::<_>(to_child, vec![msg])
    }

    #[inline]
    pub fn send_batch<'a, TargetModelMessages>(&mut self, to_child: Finger<'a>, mut msgs: TargetModelMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    where
        TargetModelMessages: AnyVecMessages,
    {
        self.send_batch_dyn(to_child, &mut msgs)
    }
}

impl<M: Component> From<CompInner<M>> for Comp {
    fn from(inner: CompInner<M>) -> Self {
        Comp {
            inner: Box::new(inner),
            modifier: None
        }
    }
}

impl Deref for Comp {
    type Target = dyn CompApi;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl DerefMut for Comp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner
    }
}

impl Drawable for Comp {
    #[inline]
    fn shape(&self) -> Option<&Shape> {
        self.inner.view_node_as_drawable()?.shape()
    }

    #[inline]
    fn shape_mut(&mut self) -> Option<&mut Shape> {
        self.inner.view_node_as_drawable_mut()?.shape_mut()
    }

    #[inline]
    fn childs(&self) -> Option<DrawableChilds> {
        self.inner.view_node_as_drawable()?.childs()
    }

    #[inline]
    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        self.inner.view_node_as_drawable_mut()?.childs_mut()
    }
}

pub trait CompApi: AsAny {
    fn id(&self) -> Option<&str>;
    fn get_comp<'a>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>>;
    fn get_comp_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>>;
    fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed;
    fn update(&mut self, msgs: &mut dyn AnyVecMessages);
    fn input(&mut self, event: InputEvent, messages: Option<&mut dyn AnyVecMessages>);
    fn modify_content(&mut self);
    fn view_node_as_drawable(&self) -> Option<&dyn Drawable>;
    fn view_node_as_drawable_mut(&mut self) -> Option<&mut dyn Drawable>;
    fn send_batch_dyn<'a>(&mut self, finger: Finger<'a>, msgs: &mut dyn AnyVecMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>;
}

pub struct CompInner<SelfModel: Component> {
    pub id: Option<String>,
    pub model: Option<SelfModel>,
    pub props: Option<SelfModel::Properties>,
    pub view_node: Option<Node<SelfModel>>,
    pub defaults: Option<Rc<NodeDefaults>>,
    pub pass_up_handler: Option<fn(&dyn AnyMessage) -> Box<dyn AnyMessage>>,
    input_closure: Option<fn(&mut dyn CompApi, InputEvent, Option<&mut dyn AnyVecMessages>)>,
    send_batch_in_depth_closure: Option<for <'a> fn(&mut CompInner<SelfModel>, Finger<'a>, &mut dyn AnyVecMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>>,
}

impl<SelfModel: Component> Default for CompInner<SelfModel> {
    fn default() -> Self {
        CompInner {
            id: None,
            model: None,
            props: None,
            view_node: None,
            defaults: None,
            pass_up_handler: None,
            input_closure: None,
            send_batch_in_depth_closure: None
        }
    }
}

impl<SelfModel: Component> CompApi for CompInner<SelfModel> {
    #[inline]
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    #[inline]
    fn get_comp<'a>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>> {
        self.view_node().get_comp(finger)
    }

    #[inline]
    fn get_comp_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>> {
        self.view_node_mut().get_comp_mut(finger)
    }

    #[inline]
    fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        self.defaults = defaults;
        let defaults = self.cloned_defaults();
        self.view_node_mut().resolve(defaults)
    }

    #[inline]
    fn update(&mut self, messages: &mut dyn AnyVecMessages) {
        let messages = messages.as_any_mut().downcast_mut::<Vec<SelfModel::Message>>()
            .expect("Can't downcast AnyVecMessages to Vec<Message> for update");
        self.update_msgs(messages);
    }

    #[inline]
    fn input(&mut self, event: InputEvent, messages: Option<&mut dyn AnyVecMessages>) {
        self.input_closure.map(|inputer| {
            inputer(self, event, messages)
        });
    }

    #[inline]
    fn modify_content(&mut self) {
        let model = self.model.take()
            .expect("Can't extract model for modify content");
        self.view_node_mut().modify(&model as &dyn AnyModel);
        self.model.replace(model);
    }

    #[inline]
    fn view_node_as_drawable(&self) -> Option<&dyn Drawable> {
        self.view_node.as_ref().map(|node| node as &dyn Drawable)
    }

    #[inline]
    fn view_node_as_drawable_mut(&mut self) -> Option<&mut dyn Drawable> {
        self.view_node.as_mut().map(|node| node as &mut dyn Drawable)
    }

    #[inline]
    fn send_batch_dyn<'a>(&mut self, finger: Finger<'a>, msgs: &mut dyn AnyVecMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    {
        let sender = self.send_batch_in_depth_closure
            .expect("Send in depth closure must be not None");
        sender(self, finger, msgs)
    }
}

impl<SelfModel: Component> CompInner<SelfModel> {
    pub fn new(props: SelfModel::Properties) -> Self {
        let mut comp = Self::default();
        comp.init(props);
        comp
    }

    /// Create model and attach properties associated with the component.
    pub fn init(&mut self, props: SelfModel::Properties) {
        let model = SelfModel::create(&props);
        let view_node = model.view();
        self.model = Some(model);
        self.view_node = Some(view_node);
        self.props = Some(props);

        self.input_closure = Some(|comp: &mut dyn CompApi, event: InputEvent, _parent_messages: Option<&mut dyn AnyVecMessages>| {
            let comp = comp.as_any_mut()
                .downcast_mut::<CompInner<SelfModel>>()
                .expect("Can't downcast mut CompInner");
            let mut messages = Vec::new();
            comp.view_node_mut().input(event, &mut messages);
            comp.update_msgs(&mut messages);
        });

        self.send_batch_in_depth_closure = Some(|comp: &mut CompInner<SelfModel>, finger: Finger, messages: &mut dyn AnyVecMessages| {
            comp.send_batch_in_depth(finger, messages, |comp, msgs| {
                comp.update(msgs);
                None
            })
        });
    }

    pub fn init_as_child<ParentModel: Component>(&mut self) {
        self.input_closure = Some(|comp: &mut dyn CompApi, event: InputEvent, parent_messages: Option<&mut dyn AnyVecMessages>| {
            let comp = comp.as_any_mut()
                .downcast_mut::<CompInner<SelfModel>>()
                .expect("Can't downcast mut CompInner");

            let mut messages = Vec::new();
            comp.view_node_mut().input(event, &mut messages);
            let to_parent_messages = comp.update_and_pass_up::<ParentModel>(&mut messages);

            if let Some(to_parent_messages) = to_parent_messages {
                if let Some(parent_messages) = parent_messages {
                    let parent_messages = parent_messages.as_any_mut().downcast_mut::<Vec<ParentModel::Message>>()
                        .expect("Inputer can't downcast parent messages to Vec<Message>");

                    for msg in to_parent_messages.into_iter() {
                        parent_messages.push(msg);
                    }
                }
            }
        });

        self.send_batch_in_depth_closure = Some(|comp: &mut CompInner<SelfModel>, finger: Finger, messages: &mut dyn AnyVecMessages| {
            comp.send_batch_in_depth(finger, messages, |comp, msgs| {
                let msgs = msgs.as_any_mut().downcast_mut::<Vec<SelfModel::Message>>()
                    .expect("Can't downcast messages for update and pass up");
                comp.update_and_pass_up::<ParentModel>(msgs)
                    .map(|vec| Box::new(vec) as Box<dyn AnyVecMessages>)
            })
        });
    }

    #[inline]
    pub fn cloned_defaults(&self) -> Option<Rc<NodeDefaults>> {
        self.defaults.as_ref().map(|d| Rc::clone(d))
    }

    #[inline]
    pub fn pass_up<ParentModel: Component>(&mut self, msg: &dyn AnyMessage) -> Option<ParentModel::Message> {
        self.pass_up_handler.map(|pass_up_handler| {
            *pass_up_handler(msg).into_any().downcast::<ParentModel::Message>()
                .expect("Can't downcast pass up msg")
        })
    }

    fn send_batch_in_depth<'a, SelfUpdateFn>(
        &mut self,
        finger: Finger<'a>,
        messages: &mut dyn AnyVecMessages,
        self_updater: SelfUpdateFn,
    ) -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    where
        SelfUpdateFn: Fn(&mut CompInner<SelfModel>, &mut dyn AnyVecMessages) -> Option<Box<dyn AnyVecMessages>>,
    {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                Ok(self_updater(self, messages))
            },
            Finger::Location(loc) => {
                let mut loc = loc;
                match self.view_node_mut() {
                    Node::Prim(ref mut prim) => {
                        let mut prim = prim;
                        loop {
                            let idx = loc[0];
                            let len = prim.childs.len();
                            break match prim.childs.get_mut(idx) {
                                Some(node) => {
                                    loc = &loc[1..];
                                    match node {
                                        Node::Prim(child_prim) => if loc.len() != 0 {
                                            prim = child_prim;
                                            continue;
                                        } else {
                                            Err(GetError::LinkToPrimInsteadOfComp { tail: loc })
                                        },
                                        Node::Comp(child_comp) => {
                                            child_comp.send_batch_dyn(Finger::Location(loc), messages)
                                                .map(|opt_pass_up_msgs|
                                                    opt_pass_up_msgs
                                                        .and_then(|mut pass_up_msgs|
                                                            self_updater(self, &mut *pass_up_msgs)
                                                        )
                                                )
                                        }
                                    }
                                },
                                None => Err(GetError::IdxOutOfBounds { idx, len, tail: loc }),
                            }
                        }
                    },
                    Node::Comp(inner_comp) => {
                        if loc[0] == 0 {
                            inner_comp.send_batch_dyn(Finger::Location(&loc[1..]), messages)
                                .map(|opt_pass_up_msgs|
                                    opt_pass_up_msgs
                                        .and_then(|mut pass_up_msgs|
                                            self_updater(self, &mut *pass_up_msgs)
                                        )
                                )
                        } else {
                            Err(GetError::TopCompMustHaveZeroIdx { idx: loc[0], tail: loc })
                        }
                    },
                }
            },
            Finger::Id(id) => {
                let not_found = GetError::NotFound;
                match self.view_node_mut() {
                    Node::Prim(prim) => {
                        for child in prim.childs.iter_mut() {
                            if let Ok(child_comp) = child.get_comp_mut(finger) {
                                return child_comp.send_batch_dyn(Finger::Root, messages)
                                    .map(|opt_pass_up_msgs|
                                        opt_pass_up_msgs
                                            .and_then(|mut pass_up_msgs|
                                                self_updater(self, &mut *pass_up_msgs)
                                            )
                                    );
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(comp) => if id == comp.id().ok_or(not_found)? {
                        comp.send_batch_dyn(Finger::Root, messages)
                            .map(|opt_pass_up_msgs|
                                opt_pass_up_msgs
                                    .and_then(|mut pass_up_msgs|
                                        self_updater(self, &mut *pass_up_msgs)
                                    )
                            )
                    } else {
                        Err(not_found)
                    }
                }
            },
        }
    }

    #[inline]
    pub fn model(&self) -> &SelfModel {
        self.model
            .as_ref()
            .expect("CompInner model is None")
    }

    #[inline]
    pub fn model_mut(&mut self) -> &mut SelfModel {
        self.model
            .as_mut()
            .expect("CompInner model mut is None")
    }

    #[inline]
    pub fn view_node(&self) -> &Node<SelfModel> {
        self.view_node
            .as_ref()
            .expect("CompInner view node is None")
    }

    #[inline]
    pub fn view_node_mut(&mut self) -> &mut Node<SelfModel> {
        self.view_node
            .as_mut()
            .expect("CompInner view node mut is None")
    }

    fn change_view_if_necessary(&mut self, should_change: ChangeView) {
        match should_change {
            ChangeView::Rebuild => {
                let mut new_node = self.model().view();
                new_node.resolve(self.cloned_defaults());
                self.view_node = Some(new_node);
            },
            ChangeView::Modify => {
                self.modify_content();
            },
            ChangeView::None => (),
        }
    }

    #[inline]
    fn update_msgs(&mut self, messages: &mut Vec<SelfModel::Message>) {
        let mut should_change = ChangeView::None;
        for msg in messages.drain(..) {
            should_change.up(self.model_mut().update(msg));
        }
        self.change_view_if_necessary(should_change);
    }

    fn update_and_pass_up<ParentModel: Component>(&mut self, messages: &mut Vec<SelfModel::Message>)
            -> Option<Vec<ParentModel::Message>>
    {
        let mut parent_msgs = Vec::new();
        for msg in messages.iter() {
            let parent_msg = self.pass_up::<ParentModel>(msg);
            if let Some(parent_msg) = parent_msg {
                parent_msgs.push(parent_msg);
            }
        }
        self.update_msgs(messages);

        if !parent_msgs.is_empty() {
            Some(parent_msgs)
        } else {
            None
        }
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
    #[inline]
    fn transform(&mut self, from: T) -> T {
        from
    }
}

impl<'a, M, T> Transformer<M, &'a T, T> for Comp
where
    M: Component,
    T: Clone,
{
    #[inline]
    fn transform(&mut self, from: &'a T) -> T {
        from.clone()
    }
}

impl<M, T> Transformer<M, T, Option<T>> for Comp
where
    M: Component,
{
    #[inline]
    fn transform(&mut self, from: T) -> Option<T> {
        Some(from)
    }
}

impl<'a, M> Transformer<M, &'a str, String> for Comp
where
    M: Component,
{
    #[inline]
    fn transform(&mut self, from: &'a str) -> String {
        from.to_owned()
    }
}

impl<'a, M> Transformer<M, &'a str, Option<String>> for Comp
where
    M: Component,
{
    #[inline]
    fn transform(&mut self, from: &'a str) -> Option<String> {
        Some(from.to_owned())
    }
}