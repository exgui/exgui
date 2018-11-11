//! This module contains macros which implements `egml!` macro
//! and JSX-like templates.
#![allow(non_camel_case_types, dead_code)]

use egml::{ModelComponent, Node, Rect, Circle, Path, Group, Listener};

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
    ($stack:ident (< $comp:ty : $($tail:tt)*)) => {
        #[allow(unused_mut)]
        let mut pair = $crate::egml::Comp::lazy::<$comp>();
        egml_impl! { @comp $stack $comp, pair ($($tail)*) }
    };
    // Set a whole struct as a properties
    (@comp $stack:ident $comp:ty, $pair:ident (with $props:ident, $($tail:tt)*)) => {
        $pair.0 = $props;
        egml_impl! { @comp $stack $comp, $pair ($($tail)*) }
    };
    // Set a specific field as a property.
    // It uses `Transformer` trait to convert a type used in template to a type of the field.
    (@comp $stack:ident $comp:ty, $pair:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        // It cloned for ergonomics in templates. Attribute with
        // `self.param` value could be reused and sholdn't be cloned
        // by yourself
        ($pair.0).$attr = $crate::egml::comp::Transformer::transform(&mut $pair.1, $val);
        egml_impl! { @comp $stack $comp, $pair ($($tail)*) }
    };
    // Self-closing of tag
    (@comp $stack:ident $comp:ty, $pair:ident (/ > $($tail:tt)*)) => {
        let (props, mut comp) = $pair;
        comp.init::<$comp>(props);
        $stack.push(comp.into());
        $crate::egml::macros::child_to_parent(&mut $stack, None);
        egml_impl! { $stack ($($tail)*) }
    };

    // Start of opening unit tag
    ($stack:ident (< $starttag:ident $($tail:tt)*)) => {
        let unit = $crate::egml::Unit::new(stringify!($starttag), $crate::egml::macros::$starttag::default().into());
        $stack.push(unit.into());
        egml_impl! { @unit $stack $starttag ($($tail)*) }
    };
//    // PATTERN: class=("class-1", "class-2", local_variable),
//    (@unit $stack:ident (class = ($($class:expr),*), $($tail:tt)*)) => {
//        $( $crate::egml::macros::append_class(&mut $stack, $class); )*
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
//    (@unit $stack:ident (class = $class:expr, $($tail:tt)*)) => {
//        $crate::macros::set_classes(&mut $stack, $class);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
//    // PATTERN: value="",
//    (@unit $stack:ident (value = $value:expr, $($tail:tt)*)) => {
//        $crate::macros::set_value_or_attribute(&mut $stack, $value);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
//    // PATTERN: attribute=value, - workaround for `type` attribute
//    // because `type` is a keyword in Rust
//    (@unit $stack:ident (type = $kind:expr, $($tail:tt)*)) => {
//        $crate::egml::macros::set_kind(&mut $stack, $kind);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
//    (@unit $stack:ident (checked = $kind:expr, $($tail:tt)*)) => {
//        $crate::egml::macros::set_checked(&mut $stack, $kind);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
//    (@unit $stack:ident (disabled = $kind:expr, $($tail:tt)*)) => {
//        if $kind {
//            $crate::egml::macros::add_attribute(&mut $stack, "disabled", "true");
//        }
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
    (@unit $stack:ident $shape:ident (modifier = | $this:pat, $model:ident : $cm:ty | $handler:expr, $($tail:tt)*)) => {
        set_attr!($stack, $shape.modifier = Some(move |$this: &mut $crate::egml::macros::$shape, $model: &dyn std::any::Any| {
            let $model = $model.downcast_ref::<$cm>()
                .expect(concat!("Modifier of ", stringify!($shape), " can't downcast model to ", stringify!($cm)));
            $handler
        }));
        egml_impl! { @unit $stack $shape ($($tail)*) }
    };
    // Events:
    (@unit $stack:ident $shape:ident (onclick = | $var:pat | $handler:expr, $($tail:tt)*)) => {
        egml_impl! { @unit $stack $shape ((onclick) = move | $var: $crate::egml::event::ClickEvent | $handler, $($tail)*) }
    };
//    (@unit $stack:ident (ondoubleclick = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondoubleclick) = move | $var: $crate::prelude::DoubleClickEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onkeypress = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onkeypress) = move | $var: $crate::prelude::KeyPressEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onkeydown = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onkeydown) = move | $var: $crate::prelude::KeyDownEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onkeyup = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onkeyup) = move | $var: $crate::prelude::KeyUpEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmousedown = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmousedown) = move | $var: $crate::prelude::MouseDownEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmousemove = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmousemove) = move | $var: $crate::prelude::MouseMoveEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmouseout = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmouseout) = move | $var: $crate::prelude::MouseOutEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmouseenter = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmouseenter) = move | $var: $crate::prelude::MouseEnterEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmouseleave = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmouseleave) = move | $var: $crate::prelude::MouseLeaveEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmousewheel = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmousewheel) = move | $var: $crate::prelude::MouseWheelEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmouseover = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmouseover) = move | $var: $crate::prelude::MouseOverEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onmouseup = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onmouseup) = move | $var: $crate::prelude::MouseUpEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onscroll = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onscroll) = move | $var: $crate::prelude::ScrollEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onblur = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onblur) = move | $var: $crate::prelude::BlurEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onfocus = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onfocus) = move | $var: $crate::prelude::FocusEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onsubmit = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onsubmit) = move | $var: $crate::prelude::SubmitEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (oninput = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((oninput) = move | $var: $crate::prelude::InputData | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (onchange = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((onchange) = move | $var: $crate::prelude::ChangeData | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragstart = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragstart) = move | $var: $crate::prelude::DragStartEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondrag = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondrag) = move | $var: $crate::prelude::DragEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragend = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragend) = move | $var: $crate::prelude::DragEndEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragenter = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragenter) = move | $var: $crate::prelude::DragEnterEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragleave = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragleave) = move | $var: $crate::prelude::DragLeaveEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragover = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragover) = move | $var: $crate::prelude::DragOverEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondragexit = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondragexit) = move | $var: $crate::prelude::DragExitEvent | $handler, $($tail)*) }
//    };
//    (@unit $stack:ident (ondrop = | $var:pat | $handler:expr, $($tail:tt)*)) => {
//        egml_impl! { @unit $stack ((ondrop) = move | $var: $crate::prelude::DragDropEvent | $handler, $($tail)*) }
//    };

    // PATTERN: (action)=expression,
    (@unit $stack:ident $shape:ident (($action:ident) = $handler:expr, $($tail:tt)*)) => {
        // Catch value to a separate variable for clear error messages
        let handler = $handler;
        let listener = $crate::egml::event::listener::$action(handler);
        $crate::egml::macros::attach_listener(&mut $stack, Box::new(listener));
        egml_impl! { @unit $stack $shape ($($tail)*) }
    };
//    // Attributes:
//    (@unit $stack:ident (href = $href:expr, $($tail:tt)*)) => {
//        let href: $crate::html::Href = $href.into();
//        $crate::macros::add_attribute(&mut $stack, "href", href);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
    (@unit $stack:ident $shape:ident ($attr:ident = $val:expr, $($tail:tt)*)) => {
        set_attr!($stack, $shape.$attr = $val);
        egml_impl! { @unit $stack $shape ($($tail)*) }
    };
    // End of openging tag
    (@unit $stack:ident $shape:ident (> $($tail:tt)*)) => {
        egml_impl! { $stack ($($tail)*) }
    };
    // Self-closing of tag
    (@unit $stack:ident $shape:ident (/ > $($tail:tt)*)) => {
        $crate::egml::macros::child_to_parent(&mut $stack, None);
        egml_impl! { $stack ($($tail)*) }
    };
//    (@unit $stack:ident ($($attr:ident)-+ = $val:expr, $($tail:tt)*)) => {
//        let attr = vec![$(stringify!($attr).to_string()),+].join("-");
//        $crate::macros::add_attribute(&mut $stack, &attr, $val);
//        egml_impl! { @unit $stack ($($tail)*) }
//    };
    // Traditional tag closing
    ($stack:ident (< / $endtag:ident > $($tail:tt)*)) => {
        let endtag = stringify!($endtag);
        $crate::egml::macros::child_to_parent(&mut $stack, Some(endtag));
        egml_impl! { $stack ($($tail)*) }
    };
//    // PATTERN: { for expression }
//    ($stack:ident ({ for $eval:expr } $($tail:tt)*)) => {
//        let nodes = $eval;
//        let mut vlist = $crate::virtual_dom::VList::new();
//        for node in nodes {
//            let node = $crate::virtual_dom::VNode::from(node);
//            vlist.add_child(node);
//        }
//        $stack.push(vlist.into());
//        $crate::macros::child_to_parent(&mut $stack, None);
//        egml_impl! { $stack ($($tail)*) }
//    };
//    // Support root text nodes: #313
//    // Provides `html!` blocks with only expression inside
//    ($stack:ident ({ $eval:expr })) => {
//        let node = $crate::virtual_dom::VNode::from($eval);
//        $stack.push(node);
//        egml_impl! { $stack () }
//    };
    // PATTERN: { expression }
    ($stack:ident ({ $eval:expr } $($tail:tt)*)) => {
        let node = $crate::egml::Node::from($eval);
        $crate::egml::macros::add_child(&mut $stack, node);
        egml_impl! { $stack ($($tail)*) }
    };
    // "End of paring" rule
    ($stack:ident ()) => {
        $crate::egml::macros::unpack($stack)
    };
    ($stack:ident $($tail:tt)*) => {
        compile_error!("You should use curly bracets for text nodes: <a>{ \"Link\" }</a>");
    };
}

// This entrypoint and implementation had separated to prevent infinite recursion.
#[macro_export]
macro_rules! egml {
    ($($tail:tt)*) => {{
        let mut stack = Vec::new();
        egml_impl! { stack ($($tail)*) }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! set_attr {
    ($stack:ident, $shape:ident.$attr:ident = $val:expr) => {
        {
            if let Some(&mut Node::Unit(ref mut unit)) = $stack.last_mut() {
                if let Some(shape) = unit.shape.as_ref_mut().$shape() {
                    shape.$attr = $val;
                } else {
                    panic!("no shape '{}' to set attribute '{}'", stringify!($shape), stringify!($attr));
                }
            } else {
                panic!("no unit to set attribute: {}", stringify!($attr));
            }
        }
    };
}

type Stack<MC> = Vec<Node<MC>>;

pub type rect = Rect;
pub type circle = Circle;
pub type path = Path;
pub type group = Group;

#[doc(hidden)]
pub fn unpack<MC: ModelComponent>(mut stack: Stack<MC>) -> Node<MC> {
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
pub fn attach_listener<MC: ModelComponent>(stack: &mut Stack<MC>, listener: Box<dyn Listener<MC>>) {
    if let Some(&mut Node::Unit(ref mut unit)) = stack.last_mut() {
        unit.add_listener(listener);
    } else {
        panic!("no unit to attach listener: {:?}", listener);
    }
}

#[doc(hidden)]
pub fn add_child<MC: ModelComponent>(stack: &mut Stack<MC>, child: Node<MC>) {
    match stack.last_mut() {
        Some(&mut Node::Unit(ref mut unit)) => {
            unit.add_child(child);
        }
//        Some(&mut Node::VList(ref mut vlist)) => {
//            vlist.add_child(child);
//        }
        _ => {
            panic!("parent must be a unit or a fragment to add the node: {:?}", child);
        }
    }
}

#[doc(hidden)]
pub fn child_to_parent<MC: ModelComponent>(stack: &mut Stack<MC>, endtag: Option<&'static str>) {
    if let Some(mut node) = stack.pop() {
        // Check the enclosing unit
        // TODO Check it during compilation. Possible?
        if let (&mut Node::Unit(ref mut unit), Some(endtag)) = (&mut node, endtag) {
            let starttag = unit.name();
            if !starttag.eq_ignore_ascii_case(endtag) {
                panic!("wrong closing tag: <{}> -> </{}>", starttag, endtag);
            }
        }
        // Push the popped element to the last in the stack
        if !stack.is_empty() {
            match stack.last_mut() {
                Some(&mut Node::Unit(ref mut unit)) => {
                    unit.add_child(node);
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
    use ::egml::Color;

    struct Model {
        val: f32,
    }

    impl ModelComponent for Model {
        type Message = ();
        type Properties = ();

        fn update(&mut self, _msg: <Self as ModelComponent>::Message) -> bool {
            unimplemented!()
        }

        fn create(_props: &<Self as ModelComponent>::Properties) -> Self {
            Model {
                val: 0.0,
            }
        }
    }

    #[test]
    fn set_attr() {
        let mut stack = Vec::<Node<Model>>::new();

        let rect = ::egml::macros::rect::default();
        let unit = ::egml::Unit::new("rect", rect.into());
        stack.push(unit.into());
        set_attr!(stack, rect.x = 1.2);
        match stack.last().unwrap() {
            Node::Unit(ref unit) => {
                let x = unit.shape.as_ref().rect().unwrap().x;
                assert_eq!(1.2, x);
            },
            _ => (),
        }

        let circle = ::egml::macros::circle::default();
        let unit = ::egml::Unit::new("circle", circle.into());
        stack.push(unit.into());
        set_attr!(stack, circle.r = 2.5);
        match stack.last().unwrap() {
            Node::Unit(ref unit) => {
                let r = unit.shape.as_ref().circle().unwrap().r;
                assert_eq!(2.5, r);
            },
            _ => (),
        }
    }

    #[test]
    fn set_modifier() {
        let _node: Node<Model> = egml! {
            <group translate = Some((50.0, 50.0).into()), >
                <rect x = 0.0, y = 0.0, width = 300.0, height = 300.0,
                        fill = None, stroke = Some((Color::Black, 2.0, 0.5).into()), >
                    <circle cx = 150.0, cy = 150.0, r = 20.0,
                            fill = Some(Color::Blue.into()),
                            modifier = |circle, model: Model| {
                                circle.cy = model.val;
                            }, />
                </rect>
            </group>
        };
    }
}