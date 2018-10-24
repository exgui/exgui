pub mod macros;
pub mod unit;
pub mod shape;

pub use self::unit::*;
pub use self::shape::*;

use std::fmt::{self, Pointer};

pub enum Node<MC: ModelComponent> {
    Unit(Unit<MC>),
//    Comp(Comp),
//    List(List),
}

impl<MC: ModelComponent> From<Unit<MC>> for Node<MC> {
    fn from(unit: Unit<MC>) -> Self {
        Node::Unit(unit)
    }
}

impl<MC: ModelComponent> fmt::Debug for Node<MC> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Node::Unit(ref unit) => unit.fmt(f),
//            Node::Comp(_) => "Component<>".fmt(f),
//            Node::List(_) => "List<>".fmt(f),
//            Node::Text(ref text) => text.fmt(f),
        }
    }
}

pub type ShouldRender = bool;

/// An interface of a UI-component. Uses `self` as a model.
pub trait ModelComponent: Sized + 'static {
    /// Control message type which `update` loop get.
    type Message: 'static;
    /// Properties type of component implementation.
    /// It sould be serializable because it's sent to dynamicaly created
    /// component (layed under `Comp`) and must be restored for a component
    /// with unknown type.
    type Properties: Clone + PartialEq + Default;

    /// Initialization routine which could use a context.
    //fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self;
    /// Called everytime when a messages of `Msg` type received. It also takes a
    /// reference to a context.
    fn update(&mut self, msg: Self::Message) -> ShouldRender;
    /// This method called when properties changes, and once when component created.
    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        unimplemented!("you should implement `change` method for a component with properties")
    }
    /// Called for finalization on the final point of the component's lifetime.
    fn destroy(&mut self) { } // TODO Replace with `Drop`
}

/// Should be viewed relative to context and component environment.
pub trait Viewable<MC: ModelComponent> {
    /// Called by rendering loop.
    fn view(&self) -> Node<MC>;
}


#[cfg(test)]
mod tests {
    use super::*;

    struct Model;

    impl ModelComponent for Model {
        type Message = ();
        type Properties = ();

        fn update(&mut self, _msg: <Self as ModelComponent>::Message) -> bool {
            unimplemented!()
        }
    }

    impl Viewable<Model> for Model {
        fn view(&self) -> Node<Self> {
            let mut rect = Unit::new(
                "rect",
                Shape::Rect(Rect { x: 0.0, y: 0.0, width: 50.0, height: 80.0, ..Default::default() })
            );
            let mut group = Unit::new(
                "group",
                Shape::Group(Group {})
            );
            let circle = Unit::new(
                "circle",
                Shape::Circle(Circle { cx: 120.0, cy: 120.0, r: 40.0, ..Default::default() })
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
                        assert_eq!(0.0, rect.x);
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
                                                assert_eq!(40.0, circle.r);
                                            },
                                            _ => (),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
