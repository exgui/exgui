use std::borrow::Cow;
use std::marker::PhantomData;
use egml::{ModelComponent, Node, Shape};

#[derive(Debug)]
pub struct Unit<MC: ModelComponent> {
    pub name: Cow<'static, str>,
    pub shape: Shape,
    pub value: Option<String>,
    pub attrs: Attrs,
    pub childs: Vec<Node<MC>>,
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
}

#[derive(Debug, Default)]
pub struct Attrs {}