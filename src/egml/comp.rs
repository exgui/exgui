use std::mem;
use std::any::Any;
use std::rc::Rc;
use egml::{
    Component, ChangeView, Viewable, Drawable, DrawableChilds,
    Node, NodeDefaults, Shape, ChildrenProcessed,
};
use controller::InputEvent;

trait AsAny {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn downcast_ref<U: 'static>(&self) -> Option<&U>;

    fn downcast_mut<U: 'static>(&mut self) -> Option<&mut U>;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn downcast_ref<U: 'static>(&self) -> Option<&U> {
        self.as_any().downcast_ref::<U>()
    }

    fn downcast_mut<U: 'static>(&mut self) -> Option<&mut U> {
        self.as_any_mut().downcast_mut::<U>()
    }
}

#[derive(Default)]
pub struct Comp {
    pub model: Option<Box<dyn Any>>,
    pub props: Option<Box<dyn Any>>,
    pub view_node: Option<Box<dyn Any>>,
    pub defaults: Option<Rc<NodeDefaults>>,
    pub resolver: Option<fn(&mut Comp) -> ChildrenProcessed>,
    pub drawer: Option<fn(&Comp) -> &dyn Drawable>,
    pub inputer: Option<fn(&mut Comp, InputEvent) -> ChangeView>,
    pub modify_handler: Option<fn(&mut Comp)>,
    pub modifier: Option<fn(&mut Comp, &dyn Any)>,
}

impl Comp {
    /// This method prepares a generator to make a new instance of the `Component`.
    pub fn lazy<MYM>() -> (<MYM as Component>::Properties, Self)
        where
            MYM: Component + Viewable<MYM>,
    {
        (Default::default(), Default::default())
    }

    pub fn new<MYM>(props: <MYM as Component>::Properties) -> Self
        where
            MYM: Component + Viewable<MYM>,
    {
        let mut comp = Comp::default();
        comp.init::<MYM>(props);
        comp
    }

    /// Create model and attach properties associated with the component.
    pub fn init<MYM>(&mut self, props: <MYM as Component>::Properties)
        where
            MYM: Component + Viewable<MYM>,
    {
        let model = <MYM as Component>::create(&props);
        let node = model.view();
        self.model = Some(Box::new(model));
        self.view_node = Some(Box::new(node));
        self.props = Some(Box::new(props));
        self.resolver = Some(|comp: &mut Comp| {
            let defaults = comp.cloned_defaults();
            comp.view_node_mut::<MYM>().resolve(defaults)
        });
        self.drawer = Some(|comp: &Comp| {
            comp.view_node::<MYM>() as &dyn Drawable
        });
        self.inputer = Some(|comp: &mut Comp, event: InputEvent| {
            let mut view_node = mem::replace(&mut comp.view_node, None)
                .expect("Inputer can't extract node");
            {
                let defaults = comp.cloned_defaults();
                let model = comp.model_mut::<MYM>();
                let should_change = (*view_node)
                    .downcast_mut::<Node<MYM>>().expect("Inputer can't downcast node")
                    .input(event, model);

                match should_change {
                    ChangeView::Rebuild => {
                        let mut new_node = model.view();
                        new_node.resolve(defaults);
                        view_node = Box::new(new_node);
                    },
                    ChangeView::Modify => {
                        (*view_node)
                            .downcast_mut::<Node<MYM>>().expect("Inputer can't downcast node")
                            .modify(model);
                    },
                    ChangeView::None => (),
                }
            }
            mem::replace(&mut comp.view_node, Some(view_node));
            ChangeView::None
        });
        self.modify_handler = Some(|comp: &mut Comp| {
            let boxed_model = mem::replace(&mut comp.model, None)
                .expect("Modifier can't extract model");
            comp.view_node_mut::<MYM>().modify(&(*boxed_model));
            mem::replace(&mut comp.model, Some(boxed_model));
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

    pub fn input(&mut self, event: InputEvent) -> ChangeView {
        self.inputer.map(|inputer| {
            inputer(self, event)
        }).unwrap_or(ChangeView::None)
    }

    pub fn cloned_defaults(&self) -> Option<Rc<NodeDefaults>> {
        self.defaults.as_ref().map(|d| Rc::clone(d))
    }

    pub fn send<M: Component + Viewable<M>>(&mut self, msg: M::Message) {
        let should_change = self.model_mut::<M>().update(msg);
        match should_change {
            ChangeView::Rebuild => {
                let mut new_node = self.model::<M> ().view();
                new_node.resolve(self.cloned_defaults());
                self.view_node = Some(Box::new(new_node));
            },
            ChangeView::Modify => {
                self.modify_internal();
            },
            ChangeView::None => (),
        }
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
}

impl Drawable for Comp {
    fn shape(&self) -> Option<&Shape> {
        self.drawer.and_then(|drawer| {
            drawer(self).shape()
        })
    }

    fn childs(&self) -> Option<DrawableChilds> {
        self.drawer.and_then(|drawer| {
            drawer(self).childs()
        })
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

impl<'a, M> Transformer<M, &'a str, String> for Comp
    where
        M: Component,
{
    fn transform(&mut self, from: &'a str) -> String {
        from.to_owned()
    }
}
