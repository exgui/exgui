pub mod macros;
pub mod unit;
pub mod comp;
pub mod shape;
pub mod paint;
pub mod transform;

pub use self::unit::*;
pub use self::comp::*;
pub use self::shape::*;
pub use self::paint::*;
pub use self::transform::*;

use std::any::Any;
use std::fmt::{self, Pointer};
use std::rc::Rc;
use controller::InputEvent;

pub enum Node<MC: ModelComponent> {
    Unit(Unit<MC>),
    Comp(Comp),
//    List(List),
}

#[derive(Default, Clone)]
pub struct NodeDefaults {
    fill: Option<Fill>,
    stroke: Option<Stroke>,
    translate: Option<Translate>,
}

impl<MC: ModelComponent> Node<MC> {
    pub fn input(&mut self, event: InputEvent, model: &mut MC) -> ShouldChangeView {
        match self {
            Node::Unit(ref mut unit) => unit.input(event, model),
            Node::Comp(ref mut comp) => comp.input(event),
        }
    }
}

pub type ChildrenProcessed = bool;

impl<MC: ModelComponent + Viewable<MC>> Node<MC> {
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
}

impl<MC: ModelComponent> Drawable for Node<MC> {
    fn shape(&self) -> Option<&Shape> {
        match self {
            Node::Unit(ref unit) => unit.shape(),
            Node::Comp(ref comp) => comp.shape(),
        }
    }

    fn childs(&self) -> Option<DrawableChilds> {
        match self {
            Node::Unit(ref unit) => Drawable::childs(unit),
            Node::Comp(ref comp) => Drawable::childs(comp),
        }
    }
}

impl<MC: ModelComponent> From<Unit<MC>> for Node<MC> {
    fn from(unit: Unit<MC>) -> Self {
        Node::Unit(unit)
    }
}

impl<MC: ModelComponent> From<Comp> for Node<MC> {
    fn from(comp: Comp) -> Self {
        Node::Comp(comp)
    }
}

impl<MC: ModelComponent, T: ToString> From<T> for Node<MC> {
    fn from(value: T) -> Self {
        Node::Unit(Unit::new("text", Shape::Text(
            Text { content: value.to_string(), ..Default::default() }
        )))
    }
}

impl<'a, MC: ModelComponent> From<&'a dyn Viewable<MC>> for Node<MC> {
    fn from(value: &'a dyn Viewable<MC>) -> Self {
        value.view()
    }
}

impl<MC: ModelComponent> fmt::Debug for Node<MC> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Node::Unit(ref unit) => unit.fmt(f),
            Node::Comp(ref _comp) => "Component<>".fmt(f),
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
    fn create(props: &Self::Properties/*, link: ComponentLink<Self>*/) -> Self;

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

pub type DrawableChilds<'a> = Box<dyn Iterator<Item=&'a dyn Drawable> + 'a>;

pub trait Drawable {
    /// Called by rendering loop.
    fn shape(&self) -> Option<&Shape>;

    fn childs(&self) -> Option<DrawableChilds>;

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

/// Converts property and attach lazy components to it.
pub trait Converter<TO> {
    /// Convert one type to another.
    fn convert(self) -> TO;
}

impl<T> Converter<T> for T {
    fn convert(self) -> T {
        self
    }
}

impl<T> Converter<Option<T>> for T {
    fn convert(self) -> Option<T> {
        Some(self)
    }
}

impl<'a, T: Clone> Converter<T> for &'a T {
    fn convert(self) -> T {
        self.clone()
    }
}

impl<'a> Converter<String> for &'a str {
    fn convert(self) -> String {
        self.to_owned()
    }
}

impl<'a> Converter<(AlignHor, AlignVer)> for AlignHor {
    fn convert(self) -> (AlignHor, AlignVer) {
        (self, AlignVer::default())
    }
}

impl<'a> Converter<(AlignHor, AlignVer)> for AlignVer {
    fn convert(self) -> (AlignHor, AlignVer) {
        (AlignHor::default(), self)
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

        fn create(_props: &<Self as ModelComponent>::Properties) -> Self {
            Model
        }

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
