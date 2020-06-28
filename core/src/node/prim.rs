use std::{borrow::Cow, marker::PhantomData, collections::HashMap};

use crate::{
    InputEvent, EventName, Listener, Model, Node, Shape, CompositeShape, CompositeShapeIter,
    CompositeShapeIterMut, SystemMessage, Transform, On,
};

pub struct Prim<M: Model> {
    pub name: Cow<'static, str>,
    pub shape: Shape,
    pub children: Vec<Node<M>>,
    pub listeners: HashMap<EventName, Vec<Listener<M>>>,
    model: PhantomData<M>,
}

impl<M: Model> Prim<M> {
    pub fn new(
        name: Cow<'static, str>,
        shape: Shape,
        children: Vec<Node<M>>,
        listeners: HashMap<EventName, Vec<Listener<M>>>,
    ) -> Self {
        Self {
            name,
            shape,
            children,
            listeners,
            model: PhantomData
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.shape.id()
    }

    pub fn set_text(&mut self, content: impl Into<String>) -> bool {
        match self.shape {
            Shape::Text(ref mut text) => {
                text.content = content.into();
                true
            },
            _ => false,
        }
    }

    pub fn transform(&self) -> &Transform {
        self.shape.transform()
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        self.shape.transform_mut()
    }

    pub fn send_system_msg(&mut self, msg: SystemMessage, outputs: &mut Vec<M::Message>) {
        match msg {
            SystemMessage::Input(input) => {
                match input {
                    InputEvent::MouseDown(press) => if self.intersect(press.pos.x, press.pos.y) {
                        if let Some(listeners) = self.listeners.get(&EventName::ON_MOUSE_DOWN) {
                            for listener in listeners {
                                let msg = match listener {
                                    Listener::OnMouseDown(func) => func(On { prim: self, event: press }),
                                    _ => continue,
                                };
                                outputs.push(msg);
                            }
                        }
                    }
                    InputEvent::KeyDown(event) => {
                        if let Some(listeners) = self.listeners.get(&EventName::ON_KEY_DOWN) {
                            for listener in listeners {
                                let msg = match listener {
                                    Listener::OnKeyDown(func) => func(On { prim: self, event }),
                                    _ => continue,
                                };
                                outputs.push(msg);
                            }
                        }
                    }
                    InputEvent::KeyUp(event) => {
                        if let Some(listeners) = self.listeners.get(&EventName::ON_KEY_UP) {
                            for listener in listeners {
                                let msg = match listener {
                                    Listener::OnKeyUp(func) => func(On { prim: self, event }),
                                    _ => continue,
                                };
                                outputs.push(msg);
                            }
                        }
                    }
                    InputEvent::Char(ch) => {
                        if let Some(listeners) = self.listeners.get(&EventName::ON_INPUT_CHAR) {
                            for listener in listeners {
                                let msg = match listener {
                                    Listener::OnInputChar(func) => func(On { prim: self, event: ch }),
                                    _ => continue,
                                };
                                outputs.push(msg);
                            }
                        }
                    }
                }
            },
            SystemMessage::Draw(duration) => {
                if let Some(listeners) = self.listeners.get(&EventName::DRAW) {
                    for listener in listeners {
                        let msg = match listener {
                            Listener::Draw(func) => func(duration),
                            _ => continue,
                        };
                        outputs.push(msg);
                    }
                }
            },
            SystemMessage::WindowResized { width, height } => {
                if let Some(listeners) = self.listeners.get(&EventName::WINDOW_RESIZED) {
                    for listener in listeners {
                        let msg = match listener {
                            Listener::WindowResized(func) => func(width, height),
                            _ => continue,
                        };
                        outputs.push(msg);
                    }
                }
            },
        }

        for child in self.children.iter_mut() {
            child.send_system_msg(msg, outputs);
        }
    }

    pub fn update_view(&mut self) {
        for child in self.children.iter_mut() {
            child.update_view();
        }
    }
}

impl<M: Model> CompositeShape for Prim<M> {
    fn shape(&self) -> Option<&Shape> {
        Some(&self.shape)
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        Some(&mut self.shape)
    }

    fn children(&self) -> Option<CompositeShapeIter> {
        Some(Box::new(self.children.iter().map(|node| node as &dyn CompositeShape)))
    }

    fn children_mut(&mut self) -> Option<CompositeShapeIterMut> {
        Some(Box::new(self.children.iter_mut().map(|node| node as &mut dyn CompositeShape)))
    }
}