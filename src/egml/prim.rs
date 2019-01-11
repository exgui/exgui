use std::borrow::Cow;
use std::rc::Rc;
use crate::egml::{
    Component, Viewable, Drawable, DrawableChilds, DrawableChildsMut, AnyModel,
    Node, NodeDefaults, Shape, Shapeable, Listener, ChildrenProcessed, Transform,
    shape::*, event::{Event, ClickEvent}
};
use crate::controller::{InputEvent, MousePos};

pub struct Prim<M: Component> {
    pub name: Cow<'static, str>,
    pub shape: Shape,
    pub value: Option<String>,
    pub attrs: Attrs,
    pub childs: Vec<Node<M>>,
    pub listeners: Vec<Box<dyn Listener<M>>>,
}

impl<M: Component> Prim<M> {
    pub fn new(name: &'static str, shape: Shape) -> Self {
        Prim {
            name: name.into(),
            shape,
            value: None,
            attrs: Attrs {},
            childs: Vec::new(),
            listeners: Vec::new(),
        }
    }

    /// Returns name of an `Prim`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add `Node` child.
    pub fn add_child(&mut self, child: Node<M>) {
        self.childs.push(child);
    }

    /// Adds new listener to the node.
    /// It's boxed because we want to keep it in a single list.
    pub fn add_listener(&mut self, listener: Box<dyn Listener<M>>) {
        self.listeners.push(listener);
    }

    pub fn input(&mut self, event: InputEvent, messages: &mut Vec<M::Message>) {
        match event {
            InputEvent::MousePress(pos) => {
                self.mouse_press(pos, messages)
            }
        }
    }

    pub fn mouse_press(&mut self, pos: MousePos, messages: &mut Vec<M::Message>) {
        if self.intersect(pos.x, pos.y) {
            for listener in self.listeners.iter() {
                if let Some(msg) = listener.handle(Event::Click(ClickEvent)) {
                    messages.push(msg);
                }
            }
        }
        for child in self.childs.iter_mut() {
            child.input(InputEvent::MousePress(pos), messages);
        }
    }

    pub fn modify(&mut self, model: &dyn AnyModel) {
        match self.shape {
            Shape::Rect(ref mut r) => {
                if let Some(modifier) = r.modifier {
                    (modifier)(r, model);
                }
            },
            Shape::Circle(ref mut c) => {
                if let Some(modifier) = c.modifier {
                    (modifier)(c, model);
                }
            },
            Shape::Path(ref mut p) => {
                if let Some(modifier) = p.modifier {
                    (modifier)(p, model);
                }
            },
            Shape::Text(ref mut t) => {
                if let Some(modifier) = t.modifier {
                    (modifier)(t, model);
                }
            },
            Shape::Word(ref mut w) => {
                if let Some(modifier) = w.modifier {
                    (modifier)(w, model);
                }
            },
            Shape::Group(_) => {},
        }
    }
}

impl<M: Component + Viewable<M>> Prim<M> {
    pub fn resolve(&mut self, defaults: Option<Rc<NodeDefaults>>) -> ChildrenProcessed {
        match self.shape {
            Shape::Rect(ref mut r) => {
                if let Some(defaults) = defaults {
                    if defaults.fill.is_some() && r.fill.is_none() {
                        r.fill = defaults.fill;
                    }
                    if defaults.stroke.is_some() && r.stroke.is_none() {
                        r.stroke = defaults.stroke;
                    }
                    if defaults.translate.is_some() {
                        let tx = defaults.translate.unwrap().x.val();
                        let ty = defaults.translate.unwrap().y.val();

                        if r.transform.is_none() {
                            r.transform = Some(Transform::new());
                        }
                        r.transform.as_mut().map(|transform| {
                            transform.translate_add(tx, ty);
                        });
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
                        let tx = defaults.translate.unwrap().x.val();
                        let ty = defaults.translate.unwrap().y.val();

                        if c.transform.is_none() {
                            c.transform = Some(Transform::new());
                        }
                        c.transform.as_mut().map(|transform| {
                            transform.translate_add(tx, ty);
                        });
                    }
                }
            },
            Shape::Path(ref mut p) => {
                if let Some(defaults) = defaults {
                    if defaults.fill.is_some() && p.fill.is_none() {
                        p.fill = defaults.fill;
                    }
                    if defaults.stroke.is_some() && p.stroke.is_none() {
                        p.stroke = defaults.stroke;
                    }
                    if defaults.translate.is_some() {
                        let tx = defaults.translate.unwrap().x.val();
                        let ty = defaults.translate.unwrap().y.val();

                        if p.transform.is_none() {
                            p.transform = Some(Transform::new());
                        }
                        p.transform.as_mut().map(|transform| {
                            transform.translate_add(tx, ty);
                        });
                    }
                }
            },
            Shape::Group(ref g) => {
                if !g.empty_overrides() {
                    let mut defaults = defaults
                        .map(|d| (*d).clone())
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

                    let defaults = Rc::new(defaults);
                    for child in self.childs.iter_mut() {
                        child.resolve(Some(Rc::clone(&defaults)));
                    }
                    return true;
                }
            },
            Shape::Text(ref mut t) => {
                if let Some(defaults) = defaults {
                    if defaults.fill.is_some() && t.fill.is_none() {
                        t.fill = defaults.fill;
                    }
                    if defaults.stroke.is_some() && t.stroke.is_none() {
                        t.stroke = defaults.stroke;
                    }
                    if defaults.translate.is_some() {
                        let tx = defaults.translate.unwrap().x.val();
                        let ty = defaults.translate.unwrap().y.val();

                        if t.transform.is_none() {
                            t.transform = Some(Transform::new());
                        }
                        t.transform.as_mut().map(|transform| {
                            transform.translate_add(tx, ty);
                        });
                    }
                }
            },
            Shape::Word(_) => {},
        }
        false
    }
}

impl<M: Component> Drawable for Prim<M> {
    fn shape(&self) -> Option<&Shape> {
        Some(&self.shape)
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        Some(&mut self.shape)
    }

    fn childs(&self) -> Option<DrawableChilds> {
        Some(Box::new(self.childs.iter().map(|node| node as &dyn Drawable)))
    }

    fn childs_mut(&mut self) -> Option<DrawableChildsMut> {
        Some(Box::new(self.childs.iter_mut().map(|node| node as &mut dyn Drawable)))
    }
}

impl<M: Component> Shapeable for Prim<M> {
    #[inline]
    fn rect(&self) -> Option<&Rect> {
        self.shape.rect()
    }

    #[inline]
    fn rect_mut(&mut self) -> Option<&mut Rect> {
        self.shape.rect_mut()
    }

    #[inline]
    fn circle(&self) -> Option<&Circle> {
        self.shape.circle()
    }

    #[inline]
    fn circle_mut(&mut self) -> Option<&mut Circle> {
        self.shape.circle_mut()
    }

    #[inline]
    fn path(&self) -> Option<&Path> {
        self.shape.path()
    }

    #[inline]
    fn path_mut(&mut self) -> Option<&mut Path> {
        self.shape.path_mut()
    }

    #[inline]
    fn group(&self) -> Option<&Group> {
        self.shape.group()
    }

    #[inline]
    fn group_mut(&mut self) -> Option<&mut Group> {
        self.shape.group_mut()
    }

    #[inline]
    fn text(&self) -> Option<&Text> {
        self.shape.text()
    }

    #[inline]
    fn text_mut(&mut self) -> Option<&mut Text> {
        self.shape.text_mut()
    }

    #[inline]
    fn word(&self) -> Option<&Word> {
        self.shape.word()
    }

    #[inline]
    fn word_mut(&mut self) -> Option<&mut Word> {
        self.shape.word_mut()
    }
}

#[derive(Debug, Default)]
pub struct Attrs {}