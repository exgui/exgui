pub mod macros;
pub mod value;
pub mod converter;
pub mod node;
pub mod prim;
pub mod comp;
pub mod shape;
pub mod transform;

pub use self::value::*;
pub use self::converter::*;
pub use self::node::*;
pub use self::prim::*;
pub use self::comp::*;
pub use self::shape::*;
pub use self::transform::*;

use std::fmt;
use std::any::Any;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Finger<'a> {
    Id(&'a str),
    Location(&'a [usize]),
    Root,
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
    type Message: ComponentMessage;

    /// Properties type of component implementation.
    /// It sould be serializable because it's sent to dynamicaly created
    /// component (layed under `Comp`) and must be restored for a component
    /// with unknown type.
    type Properties: ComponentProperties;

    /// Initialization routine which could use a context.
    fn create(props: &Self::Properties/*, link: ComponentLink<Self>*/) -> Self;

    /// Called everytime when a messages of `Msg` type received. It also takes a
    /// reference to a context.
    fn update(&mut self, msg: Self::Message) -> ChangeView;

    /// This method called when properties changes, and once when component created.
    fn change(&mut self, _: Self::Properties) -> ChangeView {
        unimplemented!("you should implement `change` method for a component with properties")
    }

    /// Called for finalization on the final point of the component's lifetime.
    fn destroy(&mut self) { } // TODO Replace with `Drop`
}

pub trait ComponentMessage: Clone + 'static {}
impl<T: Clone + 'static> ComponentMessage for T {}

pub trait ComponentProperties: Clone + PartialEq + Default + 'static {}
impl<T: Clone + PartialEq + Default + 'static> ComponentProperties for T {}

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

pub trait AsAny: Any {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

pub trait AnyModel: AsAny {}
impl<M: Component> AnyModel for M {}

pub trait AnyMessage: AsAny {}
impl<M: ComponentMessage> AnyMessage for M {}

pub trait AnyVecMessages: AsAny {
    fn get_msg(&self, i: usize) -> Option<&dyn AnyMessage>;
    fn msg_iter(&self) -> MsgIter;
}
impl<M: ComponentMessage> AnyVecMessages for Vec<M> {
    fn get_msg(&self, i: usize) -> Option<&dyn AnyMessage> {
        self.get(i).map(|msg| msg as _)
    }

    fn msg_iter(&self) -> MsgIter {
        MsgIter {
            idx: 0,
            vec: self,
        }
    }
}

pub struct MsgIter<'a> {
    idx: usize,
    vec: &'a dyn AnyVecMessages,
}

impl<'a> Iterator for MsgIter<'a> {
    type Item = &'a dyn AnyMessage;

    fn next(&mut self) -> Option<Self::Item> {
        self.idx += 1;
        self.vec.get_msg(self.idx - 1)
    }
}

pub trait AnyProperties: AsAny {}
impl<P: ComponentProperties> AnyProperties for P {}

pub trait AnyNode: AsAny {}
impl<N: Nodeable> AnyNode for N {}

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
            let mut rect = Prim::new(
                "rect",
                Shape::Rect(
                    Rect { x: 0.into(), y: 0.into(), width: 50.into(), height: 80.into(), ..Default::default() }
                )
            );
            let mut group = Prim::new(
                "group",
                Shape::Group(Group::default())
            );
            let circle = Prim::new(
                "circle",
                Shape::Circle(
                    Circle { cx: 120.into(), cy: 120.into(), r: 40.into(), ..Default::default() }
                )
            );
            group.childs.push(Node::Prim(circle));
            rect.childs.push(Node::Prim(group));
            Node::Prim(rect)
        }
    }

    #[test]
    fn view() {
        let node = Model.view();
        match node {
            Node::Prim(prim) => {
                assert_eq!("rect", prim.name());
                assert_eq!(1, prim.childs.len());
                match prim.shape {
                    Shape::Rect(ref rect) => {
                        assert_eq!(0.0, rect.x.val());
                    },
                    _ => (),
                }
                for child in prim.childs.iter() {
                    match child {
                        Node::Prim(prim) => {
                            assert_eq!("group", prim.name());
                            assert_eq!(1, prim.childs.len());
                            assert!(match prim.shape {
                                Shape::Group(_) => true,
                                _ => false,
                            });
                            for child in prim.childs.iter() {
                                match child {
                                    Node::Prim(prim) => {
                                        assert_eq!("circle", prim.name());
                                        assert_eq!(0, prim.childs.len());
                                        match prim.shape {
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
