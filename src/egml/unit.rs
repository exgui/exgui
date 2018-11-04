use std::borrow::Cow;
use std::marker::PhantomData;
use egml::{ModelComponent, Node, Shape, Listener};

#[derive(Debug)]
pub struct Unit<MC: ModelComponent> {
    pub name: Cow<'static, str>,
    pub shape: Shape,
    pub value: Option<String>,
    pub attrs: Attrs,
    pub childs: Vec<Node<MC>>,
    pub listeners: Vec<Box<dyn Listener<MC>>>,
    _phantom: PhantomData<MC>,
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
            _phantom: PhantomData,
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
}

#[derive(Debug, Default)]
pub struct Attrs {}