//! This module contains macros which implements `egml!` macro
//! and JSX-like templates.
#![allow(non_camel_case_types, dead_code)]

use crate::egml::{Component, Node, Comp, Rect, Circle, Path, Group, Text, Word, Listener};

#[macro_export]
macro_rules! egml_impl {
    // Start of component tag
    ($state:ident (< $comp:ty : $($tail:tt)*)) => {
        #[allow(unused_mut)]
        let mut pair = $crate::egml::Comp::lazy::<$comp>();
        $crate::egml_impl! { @comp $state $comp, pair ($($tail)*) }
    };
    // Set a whole struct as a properties
    (@comp $state:ident $comp:ty, $pair:ident (with $props:ident, $($tail:tt)*)) => {
        $pair.0 = $props;
        $crate::egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (id = $val:expr, $($tail:tt)*)) => {
        ($pair.1).inner_mut::<$comp>().id = $crate::egml::Converter::convert($val);
        $crate::egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (modifier = | $this:pat, $model:ident | $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { @comp $state $comp, $pair (modifier = | $this, $model : $comp | $handler, $($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (modifier = | $this:pat, $model:ident : $pcm:ty | $handler:expr, $($tail:tt)*)) => {
        ($pair.1).modifier = Some(move |$this: &mut $crate::egml::Comp, $model: &dyn $crate::egml::AnyModel| {
            let $model = $model.as_any().downcast_ref::<$pcm>()
                .expect(concat!("Modifier of ", stringify!($comp), " can't downcast model to ", stringify!($pcm)));
            $handler
        });
        $crate::egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (modifier = for <$pcm:ty> $handler:ident, $($tail:tt)*)) => {
        $crate::egml_impl! { @comp $state $comp, $pair (modifier = | this, model : $pcm | $handler(this, model), $($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (modifier = for <$pcm:ty> $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { @comp $state $comp, $pair (modifier = | this, model : $pcm | ($handler)(this, model), $($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (pass_up = | $msg:ident | $handler:expr, $($tail:tt)*)) => {
        ($pair.1).inner_mut::<$comp>().pass_up_handler = Some(move |$msg: &dyn $crate::egml::AnyMessage| {
            let $msg = $msg.as_any().downcast_ref::<<$comp as $crate::egml::Component>::Message>()
                .expect(concat!("Pass up handler of ", stringify!($comp), " can't downcast msg to ", stringify!($comp::Message)))
                .clone();
            Box::new($handler) as Box<dyn $crate::egml::AnyMessage>
        });
        $crate::egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    // Set a specific field as a property.
    // It uses `Transformer` trait to convert a type used in template to a type of the field.
    (@comp $state:ident $comp:ty, $pair:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        // It cloned for ergonomics in templates. Attribute with
        // `self.param` value could be reused and sholdn't be cloned
        // by yourself
        ($pair.0).$attr = $crate::egml::comp::Transformer::<$comp, _, _>::transform(&mut $pair.1, $val);
        $crate::egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    // Self-closing of tag
    (@comp $state:ident $comp:ty, $pair:ident (/ > $($tail:tt)*)) => {
        let (props, mut comp) = $pair;
        comp.inner_mut::<$comp>().init(props);
        $state.init_inner_comp::<$comp>(&mut comp);
        $state.stack.push(comp.into());
        $crate::egml::macros::child_to_parent(&mut $state.stack, None);
        $crate::egml_impl! { $state ($($tail)*) }
    };

    // Start of opening prim tag
    ($state:ident (< $starttag:ident $($tail:tt)*)) => {
        let prim = $crate::egml::Prim::new(stringify!($starttag), $crate::egml::macros::$starttag::default().into());
        $state.stack.push(prim.into());
        $crate::egml_impl! { @prim $state $starttag ($($tail)*) }
    };
    (@prim $state:ident $shape:ident (modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { $state $shape (false, modifier = |$this, $model:$cm| $handler, $($tail)*) }
        $crate::egml_impl! { @prim $state $shape ($($tail)*) }
    };
    (@prim $state:ident $shape:ident (modifier = for <$cm:ty> $handler:ident, $($tail:tt)*)) => {
        $crate::egml_impl! { @prim $state $shape (modifier = |this, model:$cm| $handler(this, model), $($tail)*) }
    };
    (@prim $state:ident $shape:ident (modifier = for <$cm:ty> $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { @prim $state $shape (modifier = |this, model:$cm| ($handler)(this, model), $($tail)*) }
    };
    // Events:
    (@prim $state:ident $shape:ident (onclick = | $var:pat | $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { @prim $state $shape ((onclick) = move | $var: $crate::egml::event::ClickEvent | $handler, $($tail)*) }
    };
    // PATTERN: (action)=expression,
    (@prim $state:ident $shape:ident (($action:ident) = $handler:expr, $($tail:tt)*)) => {
        // Catch value to a separate variable for clear error messages
        let handler = $handler;
        let listener = $crate::egml::event::listener::$action(handler);
        $crate::egml::macros::attach_listener(&mut $state.stack, Box::new(listener));
        $crate::egml_impl! { @prim $state $shape ($($tail)*) }
    };
    (@prim $state:ident $shape:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        $crate::set_attr!($state, $shape.$attr = $crate::egml::Converter::convert($val));
        $crate::egml_impl! { @prim $state $shape ($($tail)*) }
    };
    // End of openging tag
    (@prim $state:ident $shape:ident (> $($tail:tt)*)) => {
        $crate::egml_impl! { $state ($($tail)*) }
    };
    // Self-closing of tag
    (@prim $state:ident $shape:ident (/ > $($tail:tt)*)) => {
        $crate::egml::macros::child_to_parent(&mut $state.stack, None);
        $crate::egml_impl! { $state ($($tail)*) }
    };
    // Traditional tag closing
    ($state:ident (< / $endtag:ident > $($tail:tt)*)) => {
        let endtag = stringify!($endtag);
        $crate::egml::macros::child_to_parent(&mut $state.stack, Some(endtag));
        $crate::egml_impl! { $state ($($tail)*) }
    };
    // PATTERN: { for expression }
    ($state:ident ({ for $eval:expr } $($tail:tt)*)) => {
        let nodes = $eval;
        let mut prim = $crate::egml::Prim::new("list group", $crate::egml::Group::default().into());
        for node in nodes {
            prim.add_child($crate::egml::Node::from(node));
        }
        $crate::egml::macros::add_child(&mut $state.stack, prim.into());
        $crate::egml_impl! { $state ($($tail)*) }
    };
    // PATTERN: { expression }
    ($state:ident ({ $eval:expr } $($tail:tt)*)) => {
        let node = $crate::egml::Node::from($eval);
        $crate::egml::macros::add_child(&mut $state.stack, node);
        $crate::egml_impl! { $state ($($tail)*) }
    };
    ($state:ident (. $shape:ident . modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        $crate::egml_impl! { $state $shape (true, modifier = |$this, $model:$cm| $handler, $($tail)*) }
        $crate::egml_impl! { $state ($($tail)*) }
    };
    ($state:ident $shape:ident ($for_child:expr, modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        $crate::set_child_attr!($state, $for_child, $shape.modifier = Some(move |$this: &mut $crate::egml::macros::$shape, $model: &dyn $crate::egml::AnyModel| {
            let $model = $model.as_any().downcast_ref::<$cm>()
                .expect(concat!("Modifier of ", stringify!($shape), " can't downcast model to ", stringify!($cm)));
            $handler
        }));
    };
    // "End of paring" rule
    ($state:ident ()) => {
        $crate::egml::macros::unpack($state.stack)
    };
    ($state:ident $($tail:tt)*) => {
        compile_error!("You should use curly bracets for text nodes: <a>{ \"Link\" }</a>");
    };
}

// This entrypoint and implementation had separated to prevent infinite recursion.
#[macro_export]
macro_rules! egml {
    ($($tail:tt)*) => {{
        let mut state = $crate::macros::State { stack: Vec::new() };
        $crate::egml_impl! { state ($($tail)*) }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! set_attr {
    ($state:ident, $shape:ident.$attr:ident = $val:expr) => {
        $crate::set_child_attr!($state, false, $shape.$attr = $val);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! set_child_attr {
    ($state:ident, $for_child:expr, $shape:ident.$attr:ident = $val:expr) => {
        {
            let last = $state.stack.last_mut()
                .and_then(|node| if $for_child {
                    if let &mut Node::Prim(ref mut prim) = node {
                        prim.childs.last_mut()
                    } else {
                        None
                    }
                } else {
                    Some(node)
                });
            if let Some(&mut Node::Prim(ref mut prim)) = last {
                if let Some(shape) = prim.shape.as_ref_mut().$shape() {
                    shape.$attr = $val;
                } else {
                    panic!("no shape '{}' to set attribute '{}'", stringify!($shape), stringify!($attr));
                }
            } else {
                panic!("no prim to set attribute: {}", stringify!($attr));
            }
        }
    };
}

pub type Stack<M> = Vec<Node<M>>;

pub struct State<M: Component> {
    pub stack: Stack<M>,
}

impl<M: Component> State<M> {
    pub fn init_inner_comp<CM: Component>(&self, comp: &mut Comp) {
        comp.inner_mut::<CM>().init_as_child::<M>();
    }
}

pub type rect = Rect;
pub type circle = Circle;
pub type path = Path;
pub type group = Group;
pub type text = Text;
pub type word = Word;

#[doc(hidden)]
pub fn unpack<MC: Component>(mut stack: Stack<MC>) -> Node<MC> {
    if stack.len() != 1 {
        panic!("exactly one element have to be in hgml!");
    }
    stack.pop().expect("no hgml elements in the stack")
}

#[doc(hidden)]
pub fn attach_listener<MC: Component>(stack: &mut Stack<MC>, listener: Box<dyn Listener<MC>>) {
    if let Some(&mut Node::Prim(ref mut prim)) = stack.last_mut() {
        prim.add_listener(listener);
    } else {
        panic!("no prim to attach listener: {:?}", listener);
    }
}

#[doc(hidden)]
pub fn add_child<MC: Component>(stack: &mut Stack<MC>, child: Node<MC>) {
    match stack.last_mut() {
        Some(&mut Node::Prim(ref mut prim)) => {
            prim.add_child(child);
        }
//        Some(&mut Node::VList(ref mut vlist)) => {
//            vlist.add_child(child);
//        }
        _ => {
            panic!("parent must be a prim or a fragment to add the node: {:?}", child);
        }
    }
}

#[doc(hidden)]
pub fn child_to_parent<MC: Component>(stack: &mut Stack<MC>, endtag: Option<&'static str>) {
    if let Some(mut node) = stack.pop() {
        // Check the enclosing prim
        // TODO Check it during compilation. Possible?
        if let (&mut Node::Prim(ref mut prim), Some(endtag)) = (&mut node, endtag) {
            let starttag = prim.name();
            if !starttag.eq_ignore_ascii_case(endtag) {
                panic!("wrong closing tag: <{}> -> </{}>", starttag, endtag);
            }
        }
        // Push the popped element to the last in the stack
        if !stack.is_empty() {
            match stack.last_mut() {
                Some(&mut Node::Prim(ref mut prim)) => {
                    prim.add_child(node);
                }
//                Some(&mut Node::VList(ref mut vlist)) => {
//                    vlist.add_child(node);
//                }
                _ => {
                    panic!("can't add child to this type of node");
                }
            }
        } else {
            // Keep the last node in the stack
            stack.push(node);
        }
    } else {
        panic!("redundant closing tag: {:?}", endtag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::egml::{self, Shapeable, Color, ChangeView};

    struct Model {
        val: f32,
    }

    #[derive(Copy, Clone)]
    enum Msg {
        InnerToggle(i32),
    }

    impl Component for Model {
        type Message = Msg;
        type Properties = ();

        fn create(_props: &Self::Properties) -> Self {
            Model {
                val: 0.0,
            }
        }

        fn update(&mut self, msg: Self::Message) -> ChangeView {
            match msg {
                Msg::InnerToggle(_) => ChangeView::None,
            }
        }

        fn view(&self) -> Node<Self> {
            egml! {
                <rect />
            }
        }
    }

    struct InnerModel {
        val: bool,
    }

    #[derive(Copy, Clone)]
    enum InnerMsg {
        Toggle(i32),
    }

    impl Component for InnerModel {
        type Message = InnerMsg;
        type Properties = ();

        fn create(_props: &Self::Properties) -> Self {
            InnerModel {
                val: false,
            }
        }

        fn update(&mut self, msg: Self::Message) -> ChangeView {
            match msg {
                InnerMsg::Toggle(_) => ChangeView::None,
            }
        }

        fn view(&self) -> Node<Self> {
            egml! {
                <rect />
            }
        }
    }

    #[test]
    fn set_attr() {
        let mut state = State { stack: Vec::<Node<Model>>::new() };

        let rect = egml::macros::rect::default();
        let prim = egml::Prim::new("rect", rect.into());

        state.stack.push(prim.into());
        set_attr!(state, rect.x = 1.2.into());
        match state.stack.last().unwrap() {
            Node::Prim(ref prim) => {
                let x = prim.rect().unwrap().x.val();
                assert_eq!(1.2, x);
            },
            _ => (),
        }

        let circle = egml::macros::circle::default();
        let prim = egml::Prim::new("circle", circle.into());
        state.stack.push(prim.into());
        set_attr!(state, circle.r = 2.5.into());
        match state.stack.last().unwrap() {
            Node::Prim(ref prim) => {
                let r = prim.circle().unwrap().r.val();
                assert_eq!(2.5, r);
            },
            _ => (),
        }
    }

    #[test]
    fn set_prim_modifier() {
        let _node: Node<Model> = egml! {
            <group translate = (50.0, 50.0), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = (Color::Black, 2.0, 0.5), >
                    <circle cx = 150.0, cy = 150.0, r = 20.0,
                            fill = Color::Blue,
                            modifier = |circle, model: Model| {
                                circle.cy = model.val.into();
                            }, />
                    <circle modifier = for <Model> get_prim_handler(), />
                    <circle modifier = for <Model> prim_handler, />
                </rect>
            </group>
        };
    }

    fn get_prim_handler() -> impl Fn(&mut Circle, &Model) {
        |circle: &mut Circle, model: &Model| {
            circle.cy = model.val.into();
        }
    }

    fn prim_handler(circle: &mut Circle, model: &Model) {
        circle.cy = model.val.into();
    }

    #[test]
    fn set_comp_modifier() {
        let _node: Node<Model> = egml! {
            <group translate = (50.0, 50.0), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = (Color::Black, 2.0, 0.5), >
                    <InnerModel : modifier = |this, model: Model| {
                            this.send_self(InnerMsg::Toggle(model.val as i32));
                        }, />
                    <InnerModel : modifier = for <Model> get_comp_handler(), />
                    <InnerModel : modifier = for <Model> comp_handler, />
                </rect>
            </group>
        };
    }

    fn get_comp_handler() -> impl Fn(&mut Comp, &Model) {
        |this: &mut Comp, model: &Model| {
            this.send_self(InnerMsg::Toggle(model.val as i32));
        }
    }

    fn comp_handler(this: &mut Comp, model: &Model) {
        this.send_self(InnerMsg::Toggle(model.val as i32));
    }

    #[test]
    fn set_pass_up() {
        let _node: Node<Model> = egml! {
            <group translate = (50.0, 50.0), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = (Color::Black, 2.0, 0.5), >
                    <InnerModel : pass_up = |msg| {
                            match msg { InnerMsg::Toggle(s) => Msg::InnerToggle(s) }
                        }, />
                </rect>
            </group>
        };
    }
}