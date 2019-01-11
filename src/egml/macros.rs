//! This module contains macros which implements `egml!` macro
//! and JSX-like templates.
#![allow(non_camel_case_types, dead_code)]

use crate::egml::{Component, ViewableComponent, Node, Comp, Rect, Circle, Path, Group, Text, Word, Listener};

#[macro_export]
macro_rules! egml_impl {
//    ($stack:ident (< > $($tail:tt)*)) => {
//        let vlist = $crate::virtual_dom::VList::new();
//        $stack.push(vlist.into());
//        egml_impl! { $stack ($($tail)*) }
//    };
//    ($stack:ident (< / > $($tail:tt)*)) => {
//        $crate::macros::child_to_parent(&mut $stack, None);
//        egml_impl! { $stack ($($tail)*) }
//    };

    // Start of component tag
    ($state:ident (< $comp:ty : $($tail:tt)*)) => {
        #[allow(unused_mut)]
        let mut pair = $crate::egml::Comp::lazy::<$comp>();
        egml_impl! { @comp $state $comp, pair ($($tail)*) }
    };
    // Set a whole struct as a properties
    (@comp $state:ident $comp:ty, $pair:ident (with $props:ident, $($tail:tt)*)) => {
        $pair.0 = $props;
        egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (id = $val:expr, $($tail:tt)*)) => {
        ($pair.1).id = $crate::egml::Converter::convert($val);
        egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (modifier = | $this:pat, $model:ident : $pcm:ty | $handler:expr, $($tail:tt)*)) => {
        ($pair.1).modifier = Some(move |$this: &mut $crate::egml::Comp, $model: &dyn $crate::egml::AnyModel| {
            let $model = $model.downcast_ref::<$pcm>()
                .expect(concat!("Modifier of ", stringify!($comp), " can't downcast model to ", stringify!($pcm)));
            $handler
        });
        egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    (@comp $state:ident $comp:ty, $pair:ident (pass_up = | $msg:ident | $handler:expr, $($tail:tt)*)) => {
        ($pair.1).pass_up_handler = Some(move |$msg: &dyn $crate::egml::AnyMessage| {
            let $msg = $msg.as_any().downcast_ref::<<$comp as $crate::egml::Component>::Message>()
                .expect(concat!("Pass up handler of ", stringify!($comp), " can't downcast msg to ", stringify!($comp::Message)))
                .clone();
            Box::new($handler) as Box<dyn $crate::egml::AnyMessage>
        });
        egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    // Set a specific field as a property.
    // It uses `Transformer` trait to convert a type used in template to a type of the field.
    (@comp $state:ident $comp:ty, $pair:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        // It cloned for ergonomics in templates. Attribute with
        // `self.param` value could be reused and sholdn't be cloned
        // by yourself
        ($pair.0).$attr = $crate::egml::comp::Transformer::<$comp, _, _>::transform(&mut $pair.1, $val);
        egml_impl! { @comp $state $comp, $pair ($($tail)*) }
    };
    // Self-closing of tag
    (@comp $state:ident $comp:ty, $pair:ident (/ > $($tail:tt)*)) => {
        let (props, mut comp) = $pair;
        comp.init::<$comp>(props);
        $state.init_inner_comp::<$comp>(&mut comp);
        $state.stack.push(comp.into());
        $crate::egml::macros::child_to_parent(&mut $state.stack, None);
        egml_impl! { $state ($($tail)*) }
    };

    // Start of opening prim tag
    ($state:ident (< $starttag:ident $($tail:tt)*)) => {
        let prim = $crate::egml::Prim::new(stringify!($starttag), $crate::egml::macros::$starttag::default().into());
        $state.stack.push(prim.into());
        egml_impl! { @prim $state $starttag ($($tail)*) }
    };
//    // PATTERN: class=("class-1", "class-2", local_variable),
//    (@prim $state:ident (class = ($($class:expr),*), $($tail:tt)*)) => {
//        $( $crate::egml::macros::append_class(&mut $state.stack, $class); )*
//        egml_impl! { @prim $state ($($tail)*) }
//    };
//    (@prim $state:ident (class = $class:expr, $($tail:tt)*)) => {
//        $crate::macros::set_classes(&mut $state.stack, $class);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
//    // PATTERN: value="",
//    (@prim $state:ident (value = $value:expr, $($tail:tt)*)) => {
//        $crate::macros::set_value_or_attribute(&mut $state.stack, $value);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
//    // PATTERN: attribute=value, - workaround for `type` attribute
//    // because `type` is a keyword in Rust
//    (@prim $state:ident (type = $kind:expr, $($tail:tt)*)) => {
//        $crate::egml::macros::set_kind(&mut $state.stack, $kind);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
//    (@prim $state:ident (checked = $kind:expr, $($tail:tt)*)) => {
//        $crate::egml::macros::set_checked(&mut $state.stack, $kind);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
//    (@prim $state:ident (disabled = $kind:expr, $($tail:tt)*)) => {
//        if $kind {
//            $crate::egml::macros::add_attribute(&mut $state.stack, "disabled", "true");
//        }
//        egml_impl! { @prim $state ($($tail)*) }
//    };
    (@prim $state:ident $shape:ident (modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        egml_impl! { $state $shape (false, modifier = |$this, $model:$cm| $handler, $($tail)*) }
        egml_impl! { @prim $state $shape ($($tail)*) }
    };
    // Events:
    (@prim $state:ident $shape:ident (onclick = | $var:pat | $handler:expr, $($tail:tt)*)) => {
        egml_impl! { @prim $state $shape ((onclick) = move | $var: $crate::egml::event::ClickEvent | $handler, $($tail)*) }
    };
//    (@prim $state:ident (ondoubleclick = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondoubleclick) = move | $var: $crate::prelude::DoubleClickEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onkeypress = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onkeypress) = move | $var: $crate::prelude::KeyPressEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onkeydown = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onkeydown) = move | $var: $crate::prelude::KeyDownEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onkeyup = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onkeyup) = move | $var: $crate::prelude::KeyUpEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmousedown = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmousedown) = move | $var: $crate::prelude::MouseDownEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmousemove = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmousemove) = move | $var: $crate::prelude::MouseMoveEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmouseout = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmouseout) = move | $var: $crate::prelude::MouseOutEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmouseenter = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmouseenter) = move | $var: $crate::prelude::MouseEnterEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmouseleave = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmouseleave) = move | $var: $crate::prelude::MouseLeaveEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmousewheel = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmousewheel) = move | $var: $crate::prelude::MouseWheelEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmouseover = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmouseover) = move | $var: $crate::prelude::MouseOverEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onmouseup = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onmouseup) = move | $var: $crate::prelude::MouseUpEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onscroll = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onscroll) = move | $var: $crate::prelude::ScrollEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onblur = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onblur) = move | $var: $crate::prelude::BlurEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onfocus = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onfocus) = move | $var: $crate::prelude::FocusEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onsubmit = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onsubmit) = move | $var: $crate::prelude::SubmitEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (oninput = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((oninput) = move | $var: $crate::prelude::InputData | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (onchange = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((onchange) = move | $var: $crate::prelude::ChangeData | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragstart = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragstart) = move | $var: $crate::prelude::DragStartEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondrag = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondrag) = move | $var: $crate::prelude::DragEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragend = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragend) = move | $var: $crate::prelude::DragEndEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragenter = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragenter) = move | $var: $crate::prelude::DragEnterEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragleave = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragleave) = move | $var: $crate::prelude::DragLeaveEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragover = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragover) = move | $var: $crate::prelude::DragOverEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondragexit = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondragexit) = move | $var: $crate::prelude::DragExitEvent | $handler, $($tail)*) }
//    };
//    (@prim $state:ident (ondrop = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @prim $state ((ondrop) = move | $var: $crate::prelude::DragDropEvent | $handler, $($tail)*) }
//    };

    // PATTERN: (action)=expression,
    (@prim $state:ident $shape:ident (($action:ident) = $handler:expr, $($tail:tt)*)) => {
        // Catch value to a separate variable for clear error messages
        let handler = $handler;
        let listener = $crate::egml::event::listener::$action(handler);
        $crate::egml::macros::attach_listener(&mut $state.stack, Box::new(listener));
        egml_impl! { @prim $state $shape ($($tail)*) }
    };
//    // Attributes:
//    (@prim $state:ident (href = $href:expr, $($tail:tt)*)) => {
//        let href: $crate::html::Href = $href.into();
//        $crate::macros::add_attribute(&mut $state.stack, "href", href);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
    (@prim $state:ident $shape:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        set_attr!($state, $shape.$attr = $crate::egml::Converter::convert($val));
        egml_impl! { @prim $state $shape ($($tail)*) }
    };
    // End of openging tag
    (@prim $state:ident $shape:ident (> $($tail:tt)*)) => {
        egml_impl! { $state ($($tail)*) }
    };
    // Self-closing of tag
    (@prim $state:ident $shape:ident (/ > $($tail:tt)*)) => {
        $crate::egml::macros::child_to_parent(&mut $state.stack, None);
        egml_impl! { $state ($($tail)*) }
    };
//    (@prim $state:ident ($($attr:ident)-+ = $val:expr, $($tail:tt)*)) => {
//        let attr = vec![$(stringify!($attr).to_string()),+].join("-");
//        $crate::macros::add_attribute(&mut $state.stack, &attr, $val);
//        egml_impl! { @prim $state ($($tail)*) }
//    };
    // Traditional tag closing
    ($state:ident (< / $endtag:ident > $($tail:tt)*)) => {
        let endtag = stringify!($endtag);
        $crate::egml::macros::child_to_parent(&mut $state.stack, Some(endtag));
        egml_impl! { $state ($($tail)*) }
    };
    // PATTERN: { for expression }
    ($state:ident ({ for $eval:expr } $($tail:tt)*)) => {
        let nodes = $eval;
        let mut prim = $crate::egml::Prim::new("list group", $crate::egml::Group::default().into());
        for node in nodes {
            prim.add_child($crate::egml::Node::from(node));
        }
        $crate::egml::macros::add_child(&mut $state.stack, prim.into());
        egml_impl! { $state ($($tail)*) }
    };
//    // Support root text nodes: #313
//    // Provides `html!` blocks with only expression inside
//    ($state:ident ({ $eval:expr })) => {
//        let node = $crate::virtual_dom::VNode::from($eval);
//        $state.stack.push(node);
//        egml_impl! { $state () }
//    };
    // PATTERN: { expression }
    ($state:ident ({ $eval:expr } $($tail:tt)*)) => {
        let node = $crate::egml::Node::from($eval);
        $crate::egml::macros::add_child(&mut $state.stack, node);
        egml_impl! { $state ($($tail)*) }
    };
    ($state:ident (. $shape:ident . modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        egml_impl! { $state $shape (true, modifier = |$this, $model:$cm| $handler, $($tail)*) }
        egml_impl! { $state ($($tail)*) }
    };
    ($state:ident $shape:ident ($for_child:expr, modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        set_child_attr!($state, $for_child, $shape.modifier = Some(move |$this: &mut $crate::egml::macros::$shape, $model: &dyn $crate::egml::AnyModel| {
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
        egml_impl! { state ($($tail)*) }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! set_attr {
    ($state:ident, $shape:ident.$attr:ident = $val:expr) => {
        set_child_attr!($state, false, $shape.$attr = $val);
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

impl<M: ViewableComponent<M>> State<M> {
    pub fn init_inner_comp<CM: ViewableComponent<CM>>(&self, comp: &mut Comp) {
        comp.init_viewable::<M, CM>();
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

//#[doc(hidden)]
//pub fn set_value_or_attribute<COMP: Component, T: ToString>(stack: &mut Stack<COMP>, value: T) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        if vtag.tag().eq_ignore_ascii_case("option") {
//            vtag.add_attribute("value", &value)
//        } else {
//            vtag.set_value(&value)
//        }
//    } else {
//        panic!("no tag to set value: {}", value.to_string());
//    }
//}
//
//#[doc(hidden)]
//pub fn set_kind<COMP: Component, T: ToString>(stack: &mut Stack<COMP>, value: T) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        vtag.set_kind(&value);
//    } else {
//        panic!("no tag to set type: {}", value.to_string());
//    }
//}
//
//#[doc(hidden)]
//pub fn set_checked<COMP: Component>(stack: &mut Stack<COMP>, value: bool) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        vtag.set_checked(value);
//    } else {
//        panic!("no tag to set checked: {}", value);
//    }
//}
//
//#[doc(hidden)]
//pub fn add_attribute<COMP: Component, T: ToString>(
//    stack: &mut Stack<COMP>,
//    name: &str,
//    value: T,
//) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        vtag.add_attribute(name, &value);
//    } else {
//        panic!("no tag to set attribute: {}", name);
//    }
//}
//
//#[doc(hidden)]
//pub fn append_class<COMP: Component, T: AsRef<str>>(stack: &mut Stack<COMP>, class: T) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        vtag.add_class(class.as_ref());
//    } else {
//        panic!("no tag to attach class: {}", class.as_ref());
//    }
//}
//
//#[doc(hidden)]
//pub fn set_classes<COMP: Component, T: AsRef<str>>(stack: &mut Stack<COMP>, classes: T) {
//    if let Some(&mut VNode::VTag(ref mut vtag)) = stack.last_mut() {
//        vtag.set_classes(classes.as_ref());
//    } else {
//        panic!("no tag to set classes: {}", classes.as_ref());
//    }
//}

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
    use crate::egml::{self, Viewable, Shapeable, Color, ChangeView};

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
    }

    impl Viewable<Model> for Model {
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

        fn update(&mut self, msg: Self::Message) -> ChangeView {
            match msg {
                InnerMsg::Toggle(_) => ChangeView::None,
            }
        }

        fn create(_props: &Self::Properties) -> Self {
            InnerModel {
                val: false,
            }
        }
    }

    impl Viewable<InnerModel> for InnerModel {
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
    fn set_modifier() {
        let _node: Node<Model> = egml! {
            <group translate = (50.0, 50.0), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = (Color::Black, 2.0, 0.5), >
                    <circle cx = 150.0, cy = 150.0, r = 20.0,
                            fill = Color::Blue,
                            modifier = |circle, model: Model| {
                                circle.cy = model.val.into();
                            }, />
                </rect>
            </group>
        };
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