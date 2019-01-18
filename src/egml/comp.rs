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
    pub modifier: Option<fn(&mut Comp, &dyn AnyModel)>,
    pub pass_up_handler: Option<fn(&dyn AnyMessage) -> Box<dyn AnyMessage>>,
    as_drawable_closure: Option<fn(&Comp) -> &dyn Drawable>,
    as_drawable_mut_closure: Option<fn(&mut Comp) -> &mut dyn Drawable>,
    resolve_closure: Option<fn(&mut Comp) -> ChildrenProcessed>,
    input_closure: Option<fn(&mut Comp, InputEvent, Option<&mut dyn AnyVecMessages>)>,
    modify_interior_closure: Option<fn(&mut Comp)>,
    update_closure: Option<fn(&mut Comp, &mut dyn AnyVecMessages)>,
    get_comp_closure: Option<for <'a, 'b> fn(&'a Comp, Finger<'b>) -> Result<&'a Comp, GetError<'b>>>,
    get_comp_mut_closure: Option<for <'a, 'b> fn(&'a mut Comp, Finger<'b>) -> Result<&'a mut Comp, GetError<'b>>>,
    send_batch_in_depth_closure: Option<for <'a> fn(&mut Comp, Finger<'a>, &mut dyn AnyVecMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>>,
}

trait CompInside {
    fn modify_interior<SelfModel>(&mut self)
    where
        SelfModel: ViewableComponent<SelfModel>;

    fn change_view_if_necessary<SelfModel>(&mut self, should_change: ChangeView)
    where
        SelfModel: ViewableComponent<SelfModel>;

    fn update_msgs<SelfModel>(&mut self, msgs: &mut Vec<SelfModel::Message>)
    where
        SelfModel: ViewableComponent<SelfModel>;

    fn update_and_pass_up<ParentModel, SelfModel>(&mut self, messages: &mut Vec<SelfModel::Message>)
        -> Option<Vec<ParentModel::Message>>
    where
        ParentModel: ViewableComponent<ParentModel>,
        SelfModel: ViewableComponent<SelfModel>;
}

impl Comp {
    /// This method prepares a generator to make a new instance of the `Component`.
    #[inline]
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

        self.as_drawable_closure = Some(|comp: &Comp| {
            comp.view_node::<SelfModel>() as &dyn Drawable
        });

        self.as_drawable_mut_closure = Some(|comp: &mut Comp| {
            comp.view_node_mut::<SelfModel>() as &mut dyn Drawable
        });

        self.resolve_closure = Some(|comp: &mut Comp| {
            let defaults = comp.cloned_defaults();
            comp.view_node_mut::<SelfModel>().resolve(defaults)
        });

        self.input_closure = Some(|comp: &mut Comp, event: InputEvent, _parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<SelfModel>()
                .input(event, &mut messages);
            comp.update_msgs::<SelfModel>(&mut messages);
        });

        self.modify_interior_closure = Some(|comp: &mut Comp| {
            comp.modify_interior::<SelfModel>();
        });

        self.update_closure = Some(|comp: &mut Comp, messages: &mut dyn AnyVecMessages| {
            let messages = messages.as_any_mut().downcast_mut::<Vec<SelfModel::Message>>()
                .expect("Can't downcast AnyVecMessages to Vec<Message> for update");
            comp.update_msgs::<SelfModel>(messages);
        });

        self.get_comp_closure = Some(|comp: &Comp, finger: Finger| {
            comp.view_node::<SelfModel>().get_comp(finger)
        });

        self.get_comp_mut_closure = Some(|comp: &mut Comp, finger: Finger| {
            comp.view_node_mut::<SelfModel>().get_comp_mut(finger)
        });

        self.send_batch_in_depth_closure = Some(|comp: &mut Comp, finger: Finger, messages: &mut dyn AnyVecMessages| {
            comp.send_batch_in_depth::<SelfModel, _>(finger, messages, |comp, msgs| {
                comp.update(msgs);
                None
            })
        });
    }

    pub fn init_viewable<ParentModel, SelfModel>(&mut self)
        where
            ParentModel: ViewableComponent<ParentModel>,
            SelfModel: ViewableComponent<SelfModel>,
    {
        self.input_closure = Some(|comp: &mut Comp, event: InputEvent, parent_messages: Option<&mut dyn AnyVecMessages>| {
            let mut messages = Vec::new();
            comp.view_node_mut::<SelfModel>().input(event, &mut messages);
            let to_parent_messages = comp.update_and_pass_up::<ParentModel, SelfModel>(&mut messages);

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

        self.send_batch_in_depth_closure = Some(|comp: &mut Comp, finger: Finger, messages: &mut dyn AnyVecMessages| {
            comp.send_batch_in_depth::<SelfModel, _>(finger, messages, |comp, msgs| {
                let msgs = msgs.as_any_mut().downcast_mut::<Vec<SelfModel::Message>>()
                    .expect("Can't downcast messages for update and pass up");
                comp.update_and_pass_up::<ParentModel, SelfModel>(msgs)
                    .map(|vec| Box::new(vec) as Box<dyn AnyVecMessages>)
            })
        });
    }

    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|s| s.as_str())
    }

    #[inline]
    pub fn get_comp<'a>(&self, finger: Finger<'a>) -> Result<&Comp, GetError<'a>> {
        let getter = self.get_comp_closure
            .expect("Get comp closure must be not None");
        getter(self, finger)
    }

    #[inline]
    pub fn get_comp_mut<'a>(&mut self, finger: Finger<'a>) -> Result<&mut Comp, GetError<'a>> {
        let getter = self.get_comp_mut_closure
            .expect("Get comp mut closure must be not None");
        getter(self, finger)
    }

    #[inline]
    pub fn get_prim<'a, SelfModel: Component>(&self, finger: Finger<'a>) -> Result<&Prim<SelfModel>, GetError<'a>> {
        self.view_node::<SelfModel>().get_prim(finger)
    }

    #[inline]
    pub fn get_prim_mut<'a, SelfModel: Component>(&mut self, finger: Finger<'a>) -> Result<&mut Prim<SelfModel>, GetError<'a>> {
        self.view_node_mut::<SelfModel>().get_prim_mut(finger)
    }

    #[inline]
    pub fn view_node<SelfModel: Component>(&self) -> &Node<SelfModel> {
        let node = self.view_node.as_ref()
            .expect("Can't downcast node - it is None");
        node.as_any().downcast_ref::<Node<SelfModel>>()
            .expect("Can't downcast node")
    }

    #[inline]
    pub fn view_node_mut<SelfModel: Component>(&mut self) -> &mut Node<SelfModel> {
        let node = self.view_node.as_mut()
            .expect("Can't downcast node - it is None");
        node.as_any_mut().downcast_mut::<Node<SelfModel>>()
            .expect("Can't downcast node")
    }

    #[inline]
    pub fn model<SelfModel: Component>(&self) -> &SelfModel {
        let model = self.model.as_ref()
            .expect("Can't downcast model - it is None");
        model.as_any().downcast_ref::<SelfModel>()
            .expect("Can't downcast model")
    }

    #[inline]
    pub fn model_mut<SelfModel: Component>(&mut self) -> &mut SelfModel {
        let model = self.model.as_mut()
            .expect("Can't downcast model - it is None");
        model.as_any_mut().downcast_mut::<SelfModel>()
            .expect("Can't downcast model")
    }

    #[inline]
    pub fn cloned_defaults(&self) -> Option<Rc<NodeDefaults>> {
        self.defaults.as_ref().map(|d| Rc::clone(d))
    }

    #[inline]
    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        self.defaults = defaults;
        let resolver = self.resolve_closure
            .expect("Can't resolve with uninitialized resolver");
        resolver(self)
    }

    #[inline]
    pub fn input(&mut self, event: InputEvent, messages: Option<&mut dyn AnyVecMessages>) {
        self.input_closure.map(|inputer| {
            inputer(self, event, messages)
        });
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

    #[inline]
    pub fn pass_up<ParentModel: Component>(&mut self, msg: &dyn AnyMessage) -> Option<ParentModel::Message> {
        self.pass_up_handler.map(|pass_up_handler| {
            *pass_up_handler(msg).into_any().downcast::<ParentModel::Message>()
                .expect("Can't downcast pass up msg")
        })
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

    #[inline]
    fn send_batch_dyn<'a>(&mut self, finger: Finger<'a>, msgs: &mut dyn AnyVecMessages)
        -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    {
        let sender = self.send_batch_in_depth_closure
            .expect("Send in depth closure must be not None");
        sender(self, finger, msgs)
    }

    fn send_batch_in_depth<'a, SelfModel, SelfUpdateFn>(
        &mut self,
        finger: Finger<'a>,
        messages: &mut dyn AnyVecMessages,
        self_updater: SelfUpdateFn,
    ) -> Result<Option<Box<dyn AnyVecMessages>>, GetError<'a>>
    where
        SelfModel: ViewableComponent<SelfModel>,
        SelfUpdateFn: Fn(&mut Comp, &mut dyn AnyVecMessages) -> Option<Box<dyn AnyVecMessages>>,
    {
        match finger {
            Finger::Root | Finger::Location(&[]) => {
                Ok(self_updater(self, messages))
            },
            Finger::Location(loc) => {
                let mut loc = loc;
                match self.view_node_mut::<SelfModel>() {
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
                match self.view_node_mut::<SelfModel>() {
                    Node::Prim(prim) => {
                        for child in prim.childs.iter_mut() {
                            let child_comp = child.get_comp_mut(finger);
                            if let Ok(child_comp) = child_comp {
                                return Ok(self_updater(child_comp, messages));
                            }
                        }
                        Err(not_found)
                    },
                    Node::Comp(comp) => if id == comp.id().ok_or(not_found)? {
                        Ok(self_updater(comp, messages))
                    } else {
                        Err(not_found)
                    }
                }
            },
        }
    }

    #[inline]
    fn update(&mut self, msgs: &mut dyn AnyVecMessages) {
        let updater = self.update_closure
            .expect("Update closure must be not None");
        updater(self, msgs);
    }
}

impl CompInside for Comp {
    #[inline]
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

    #[inline]
    fn update_msgs<SelfModel>(&mut self, messages: &mut Vec<SelfModel::Message>)
    where
        SelfModel: ViewableComponent<SelfModel>,
    {
        let mut should_change = ChangeView::None;
        for msg in messages.drain(..) {
            should_change.up(self.model_mut::<SelfModel>().update(msg));
        }
        self.change_view_if_necessary::<SelfModel>(should_change);
    }

    fn update_and_pass_up<ParentModel, SelfModel>(&mut self, messages: &mut Vec<SelfModel::Message>)
        -> Option<Vec<ParentModel::Message>>
    where
        ParentModel: ViewableComponent<ParentModel>,
        SelfModel: ViewableComponent<SelfModel>,
    {
        let mut parent_msgs = Vec::new();
        for msg in messages.iter() {
            let parent_msg = self.pass_up::<ParentModel>(msg);
            if let Some(parent_msg) = parent_msg {
                parent_msgs.push(parent_msg);
            }
        }
        self.update_msgs::<SelfModel>(messages);

        if !parent_msgs.is_empty() {
            Some(parent_msgs)
        } else {
            None
        }
    }
}

impl Drawable for Comp {
    #[inline]
    fn shape(&self) -> Option<&Shape> {
        let drawable = self.as_drawable_closure?;
        drawable(self).shape()
    }

    #[inline]
    fn shape_mut(&mut self) -> Option<&mut Shape> {
        let drawable = self.as_drawable_mut_closure?;
        drawable(self).shape_mut()
    }

    #[inline]
    fn childs(&self) -> Option<DrawableChilds> {
        let drawable = self.as_drawable_closure?;
        drawable(self).childs()
    }

    #[inline]
    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        let drawable = self.as_drawable_mut_closure?;
        drawable(self).childs_mut()
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