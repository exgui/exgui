pub use exgui_builder as builder;
pub use exgui_core::*;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::{ChangeView, Model, Node, Rect, Text};
    use exgui_core::Shaped;

    #[derive(Debug, PartialEq)]
    struct Counter(i32);

    enum Msg {
        Increment,
        Decrement,
    }

    impl Model for Counter {
        type Message = Msg;
        type Properties = i32;

        fn create(prop: Self::Properties) -> Self {
            Counter(prop)
        }

        fn update(&mut self, msg: Self::Message) -> ChangeView {
            match msg {
                Msg::Increment => self.0 += 1,
                Msg::Decrement => self.0 -= 1,
            }
            ChangeView::Modify
        }

        fn build_view(&self) -> Node<Self> {
            use crate::builder::*;

            rect()
                .children(vec![
                    rect().child(text("-")).on_click(|_| Msg::Decrement).build(),
                    text(format!("{}", self.0)).id("counter").build(),
                    rect().child(text("+")).on_click(|_| Msg::Increment).build(),
                ])
                .build()
        }

        fn modify_view(&mut self, view: &mut Node<Self>) {
            view.get_prim_mut("counter")
                .map(|prim| prim.set_text(format!("{}", self.0)));
        }
    }

    #[test]
    fn view() {
        let view = Counter::create(0).build_view();

        let root = view.as_prim().unwrap();
        assert_eq!(root.name, Cow::Borrowed(Rect::NAME));
        let shape = root.shape.rect().unwrap();
        assert_eq!(*shape, Rect::default());
        assert_eq!(root.children.len(), 3);

        let child = root.children[0].as_prim().unwrap();
        assert_eq!(child.name, Cow::Borrowed(Rect::NAME));
        let shape = child.shape.rect().unwrap();
        assert_eq!(*shape, Rect::default());
        assert_eq!(child.children.len(), 1);

        let child = child.children[0].as_prim().unwrap();
        assert_eq!(child.name, Cow::Borrowed(Text::NAME));
        let shape = child.shape.text().unwrap();
        assert_eq!(*shape, Text {
            content: "-".to_string(),
            ..Default::default()
        });
        assert_eq!(child.children.len(), 0);

        let child = root.children[1].as_prim().unwrap();
        assert_eq!(child.name, Cow::Borrowed(Text::NAME));
        let shape = child.shape.text().unwrap();
        assert_eq!(*shape, Text {
            id: Some("counter".to_string()),
            content: "0".to_string(),
            ..Default::default()
        });
        assert_eq!(child.children.len(), 0);

        let child = root.children[2].as_prim().unwrap();
        assert_eq!(child.name, Cow::Borrowed(Rect::NAME));
        let shape = child.shape.rect().unwrap();
        assert_eq!(*shape, Rect::default());
        assert_eq!(child.children.len(), 1);

        let child = child.children[0].as_prim().unwrap();
        assert_eq!(child.name, Cow::Borrowed(Text::NAME));
        let shape = child.shape.text().unwrap();
        assert_eq!(*shape, Text {
            content: "+".to_string(),
            ..Default::default()
        });
        assert_eq!(child.children.len(), 0);
    }
}
