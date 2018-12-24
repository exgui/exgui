use std::any::Any;
use std::borrow::Cow;
use std::rc::Rc;
use egml::{
    ModelComponent, ChangeView, Viewable, Drawable, DrawableChilds,
    Node, NodeDefaults, Shape, Listener, ChildrenProcessed, Transform,
    event::{Event, ClickEvent}
};
use controller::{InputEvent, MousePos};

pub struct Unit<MC: ModelComponent> {
    pub name: Cow<'static, str>,
    pub shape: Shape,
    pub value: Option<String>,
    pub attrs: Attrs,
    pub childs: Vec<Node<MC>>,
    pub listeners: Vec<Box<dyn Listener<MC>>>,
}

impl<MC: ModelComponent> Unit<MC> {
    pub fn new(name: &'static str, shape: Shape) -> Self {
        Unit {
            name: name.into(),
            shape,
            value: None,
            attrs: Attrs {},
            childs: Vec::new(),
            listeners: Vec::new(),
        }
    }

    /// Returns name of an `Unit`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add `Node` child.
    pub fn add_child(&mut self, child: Node<MC>) {
        self.childs.push(child);
    }

    /// Adds new listener to the node.
    /// It's boxed because we want to keep it in a single list.
    pub fn add_listener(&mut self, listener: Box<dyn Listener<MC>>) {
        self.listeners.push(listener);
    }

    pub fn input(&mut self, event: InputEvent, model: &mut MC) -> ChangeView {
        match event {
            InputEvent::MousePress(pos) => {
                self.mouse_press(pos, model)
            }
        }
    }

    pub fn mouse_press(&mut self, pos: MousePos, model: &mut MC) -> ChangeView {
        let mut should_change = ChangeView::None;

        if self.intersect(pos.x, pos.y) {
            for listener in self.listeners.iter() {
                if let Some(msg) = listener.handle(Event::Click(ClickEvent)) {
                    should_change.up(model.update(msg));
                }
            }
        }
        for child in self.childs.iter_mut() {
            should_change.up(child.input(InputEvent::MousePress(pos), model));
        }
        should_change
    }

    pub fn modify(&mut self, model: &dyn Any) {
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
            Shape::Font(ref mut f) => {
                if let Some(modifier) = f.modifier {
                    (modifier)(f, model);
                }
            },
            Shape::Text(ref mut t) => {
                if let Some(modifier) = t.modifier {
                    (modifier)(t, model);
                }
            },
            Shape::Group(_) => {},
        }
    }
}

impl<MC: ModelComponent + Viewable<MC>> Unit<MC> {
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
                        let tx = defaults.translate.unwrap().x;
                        let ty = defaults.translate.unwrap().y;

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
                        let tx = defaults.translate.unwrap().x;
                        let ty = defaults.translate.unwrap().y;

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
                        let tx = defaults.translate.unwrap().x;
                        let ty = defaults.translate.unwrap().y;

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
            Shape::Font(ref mut f) => {
                if let Some(defaults) = defaults {
                    if defaults.fill.is_some() && f.fill.is_none() {
                        f.fill = defaults.fill;
                    }
                    if defaults.stroke.is_some() && f.stroke.is_none() {
                        f.stroke = defaults.stroke;
                    }
                    if defaults.translate.is_some() {
                        let tx = defaults.translate.unwrap().x;
                        let ty = defaults.translate.unwrap().y;

                        if f.transform.is_none() {
                            f.transform = Some(Transform::new());
                        }
                        f.transform.as_mut().map(|transform| {
                            transform.translate_add(tx, ty);
                        });
                    }
                }
            },
            Shape::Text(_) => {},
        }
        false
    }
}

impl<MC: ModelComponent> Drawable for Unit<MC> {
    fn shape(&self) -> Option<&Shape> {
        Some(&self.shape)
    }

    fn childs(&self) -> Option<DrawableChilds> {
        Some(Box::new(self.childs.iter().map(|node| node as &dyn Drawable)))
    }
}

#[derive(Debug, Default)]
pub struct Attrs {}