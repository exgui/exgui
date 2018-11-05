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

#[derive(Default, Clone)]
pub struct NodeDefaults {
    fill: Option<Fill>,
    stroke: Option<Stroke>,
    translate: Option<Translate>,
}

impl<MC: ModelComponent> Node<MC> {
    pub fn resolve(&mut self, defaults: Option<&NodeDefaults>) {
        match self {
            Node::Unit(ref mut unit) => {
                match unit.shape {
                    Shape::Rect(ref mut r) => {
                        if let Some(defaults) = defaults {
                            if defaults.fill.is_some() && r.fill.is_none() {
                                r.fill = defaults.fill;
                            }
                            if defaults.stroke.is_some() && r.stroke.is_none() {
                                r.stroke = defaults.stroke;
                            }
                            if defaults.translate.is_some() {
                                r.x += defaults.translate.unwrap().x;
                                r.y += defaults.translate.unwrap().y;
                            }
                        }
                    },
                    Shape::Circle(ref mut c) => {
                        if let Some(defaults) = defaults {
                            if defaults.fill.is_some() && c.fill.is_none() {
                                c.fill = defaults.fill;
                            }
                            if defaults.stroke.is_some() && c.stroke.is_none() {
                                c.stroke = defaults.stroke;
                            }
                            if defaults.translate.is_some() {
                                c.cx += defaults.translate.unwrap().x;
                                c.cy += defaults.translate.unwrap().y;
                            }
                        }
                    },
                    Shape::Group(ref g) => {
                        if !g.empty_overrides() {
                            let mut defaults = defaults
                                .map(|d| d.clone())
                                .unwrap_or(NodeDefaults::default());

                            if g.fill.is_some() {
                                defaults.fill = g.fill;
                            }
                            if g.stroke.is_some() {
                                defaults.stroke = g.stroke;
                            }
                            if g.translate.is_some() {
                                defaults.translate = g.translate;
                            }

                            for child in unit.childs.iter_mut() {
                                child.resolve(Some(&defaults));
                            }
                            return;
                        }
                    },
                }
                for child in unit.childs.iter_mut() {
                    child.resolve(defaults);
                }
            }
        }
    }
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

pub type ShouldChangeView = bool;

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
    fn update(&mut self, msg: Self::Message) -> ShouldChangeView;
    /// This method called when properties changes, and once when component created.
    fn change(&mut self, _: Self::Properties) -> ShouldChangeView {
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

/// `Listener` trait is an universal implementation of an event listener
/// which helps to bind Node-listener to input controller's output.
pub trait Listener<MC: ModelComponent> {
    /// Returns event type.
    fn kind(&self) -> &'static str;

    fn handle(&self, event: event::Event) -> Option<<MC as ModelComponent>::Message>;
}

impl<MC: ModelComponent> fmt::Debug for dyn Listener<MC> {
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
        use egml::{Listener, Viewable, ModelComponent};

        pub struct ClickListener<F, MSG>(pub F)
            where F: Fn(ClickEvent) -> MSG + 'static;

        impl<T, MC> Listener<MC> for ClickListener<T, <MC as ModelComponent>::Message>
            where
                T: Fn(ClickEvent) -> <MC as ModelComponent>::Message + 'static,
                MC: ModelComponent + Viewable<MC>,
        {
            fn kind(&self) -> &'static str {
                stringify!(ClickEvent)
            }

            fn handle(&self, event: Event) -> Option<<MC as ModelComponent>::Message> {
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
                Shape::Group(Group::default())
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
