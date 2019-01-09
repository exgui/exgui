pub mod macros;
pub mod value;
pub mod converter;
pub mod unit;
pub mod comp;
pub mod shape;
pub mod transform;

pub use self::value::*;
pub use self::converter::*;
pub use self::unit::*;
pub use self::comp::*;
pub use self::shape::*;
pub use self::transform::*;

use std::any::Any;
use std::fmt::{self, Pointer};
use std::rc::Rc;
use crate::controller::InputEvent;

pub enum Node<M: Component> {
    Unit(Unit<M>),
    Comp(Comp),
//    List(List),
}

#[derive(Default, Clone)]
pub struct NodeDefaults {
    fill: Option<Fill>,
    stroke: Option<Stroke>,
    translate: Option<Translate>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChangeView {
    Rebuild,
    Modify,
    None,
}

impl ChangeView {
    pub fn is_rebuild(&self) -> bool {
        *self == ChangeView::Rebuild
    }

    pub fn is_modify(&self) -> bool {
        *self == ChangeView::Modify
    }

    pub fn is_none(&self) -> bool {
        *self == ChangeView::None
    }

    pub fn up(&mut self, other: ChangeView) {
        match (other, *self) {
            (ChangeView::Modify, ChangeView::None) => *self = ChangeView::Modify,
            (ChangeView::Rebuild, _) => *self = ChangeView::Rebuild,
            _ => (),
        }
    }
}

pub type ShouldChangeView = bool;

/// An interface of a UI-component. Uses `self` as a model.
pub trait Component: Sized + 'static {
    /// Control message type which `update` loop get.
    type Message: Clone + 'static;

    /// Properties type of component implementation.
    /// It sould be serializable because it's sent to dynamicaly created
    /// component (layed under `Comp`) and must be restored for a component
    /// with unknown type.
    type Properties: Clone + PartialEq + Default;

    /// Initialization routine which could use a context.
    fn create(props: &Self::Properties/*, link: ComponentLink<Self>*/) -> Self;

    /// Called everytime when a messages of `Msg` type received. It also takes a
    /// reference to a context.
    fn update(&mut self, msg: Self::Message) -> ChangeView;

    fn before_child_update(&mut self, _msg: Self::Message) -> ChangeView {
        ChangeView::None
    }

    fn after_child_update(&mut self, msg: Self::Message) -> ChangeView {
        self.update(msg)
    }

    /// This method called when properties changes, and once when component created.
    fn change(&mut self, _: Self::Properties) -> ChangeView {
        unimplemented!("you should implement `change` method for a component with properties")
    }

    /// Called for finalization on the final point of the component's lifetime.
    fn destroy(&mut self) { } // TODO Replace with `Drop`
}

/// Should be viewed relative to context and component environment.
pub trait Viewable<M: Component> {
    /// Called by rendering loop.
    fn view(&self) -> Node<M>;
}

pub trait ViewableComponent<M: Component>: Component + Viewable<M> {}
impl<T: Component + Viewable<T>> ViewableComponent<T> for T {}

pub type DrawableChilds<'a> = Box<dyn Iterator<Item=&'a dyn Drawable> + 'a>;
pub type DrawableChildsMut<'a> = Box<dyn Iterator<Item=&'a mut dyn Drawable> + 'a>;

pub trait Drawable {
    /// Called by rendering loop.
    fn shape(&self) -> Option<&Shape>;

    fn shape_mut(&mut self) -> Option<&mut Shape>;

    fn childs(&self) -> Option<DrawableChilds>;

    fn childs_mut(&mut self) -> Option<DrawableChildsMut>;

    fn intersect(&self, x: f32, y: f32) -> bool {
        if let Some(shape) = self.shape() {
            match shape {
                Shape::Rect(ref r) => r.intersect(x, y),
                Shape::Circle(ref c) => c.intersect(x, y),
                Shape::Path(ref p) => p.intersect(x, y),
                _ => false,
            }
        } else {
            false
        }
    }
}

/// `Listener` trait is an universal implementation of an event listener
/// which helps to bind Node-listener to input controller's output.
pub trait Listener<M: Component> {
    /// Returns event type.
    fn kind(&self) -> &'static str;

    fn handle(&self, event: event::Event) -> Option<<M as Component>::Message>;
}

impl<M: Component> fmt::Debug for dyn Listener<M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Listener {{ kind: {} }}", self.kind())
    }
}

pub mod event {
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub enum Event {
        Click(ClickEvent),
        DoubleClick,
    }

    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct ClickEvent;

    pub mod listener {
        use super::*;
        use crate::egml::{Listener, Viewable, Component};

        pub struct ClickListener<F, MSG>(pub F)
            where F: Fn(ClickEvent) -> MSG + 'static;

        impl<T, M> Listener<M> for ClickListener<T, <M as Component>::Message>
            where
                T: Fn(ClickEvent) -> <M as Component>::Message + 'static,
                M: Component + Viewable<M>,
        {
            fn kind(&self) -> &'static str {
                stringify!(ClickEvent)
            }

            fn handle(&self, event: Event) -> Option<<M as Component>::Message> {
                match event {
                    Event::Click(event) => Some((self.0)(event)),
                    _ => None,
                }
            }
        }

        pub fn onclick<F, MSG>(handler: F) -> ClickListener<F, MSG>
            where
                MSG: 'static,
                F: Fn(ClickEvent) -> MSG + 'static,
        {
            ClickListener(handler)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Model;

    impl Component for Model {
        type Message = ();
        type Properties = ();

        fn create(_props: &<Self as Component>::Properties) -> Self {
            Model
        }

        fn update(&mut self, _msg: <Self as Component>::Message) -> ChangeView {
            unimplemented!()
        }
    }

    impl Viewable<Model> for Model {
        fn view(&self) -> Node<Self> {
            let mut rect = Unit::new(
                "rect",
                Shape::Rect(
                    Rect { x: 0.into(), y: 0.into(), width: 50.into(), height: 80.into(), ..Default::default() }
                )
            );
            let mut group = Unit::new(
                "group",
                Shape::Group(Group::default())
            );
            let circle = Unit::new(
                "circle",
                Shape::Circle(
                    Circle { cx: 120.into(), cy: 120.into(), r: 40.into(), ..Default::default() }
                )
            );
            group.childs.push(Node::Unit(circle));
            rect.childs.push(Node::Unit(group));
            Node::Unit(rect)
        }
    }

    #[test]
    fn view() {
        let node = Model.view();
        match node {
            Node::Unit(unit) => {
                assert_eq!("rect", unit.name());
                assert_eq!(1, unit.childs.len());
                match unit.shape {
                    Shape::Rect(ref rect) => {
                        assert_eq!(0.0, rect.x.val());
                    },
                    _ => (),
                }
                for child in unit.childs.iter() {
                    match child {
                        Node::Unit(unit) => {
                            assert_eq!("group", unit.name());
                            assert_eq!(1, unit.childs.len());
                            assert!(match unit.shape {
                                Shape::Group(_) => true,
                                _ => false,
                            });
                            for child in unit.childs.iter() {
                                match child {
                                    Node::Unit(unit) => {
                                        assert_eq!("circle", unit.name());
                                        assert_eq!(0, unit.childs.len());
                                        match unit.shape {
                                            Shape::Circle(ref circle) => {
                                                assert_eq!(40.0, circle.r.val());
                                            },
                                            _ => (),
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }
}
