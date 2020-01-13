use std::any::{Any, type_name};

use crate::{Model, Node, Transform, ChangeViewState, CompositeShape, CompositeShapeIter, CompositeShapeIterMut, Shape, SystemMessage};

pub trait AsAny: Any {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

pub trait CompApi: AsAny {
    fn id(&self) -> Option<&str>;
    fn set_id(&mut self, id: String);
    fn transform(&self) -> Option<&Transform>;
    fn transform_mut(&mut self) -> Option<&mut Transform>;
    fn set_transform(&mut self, transform: Transform);
    fn as_composite_shape(&self) -> Option<&dyn CompositeShape>;
    fn as_composite_shape_mut(&mut self) -> Option<&mut dyn CompositeShape>;
    fn send_system_msg(&mut self, msg: SystemMessage);
    fn update_view(&mut self);
}

pub struct Comp {
    inner: Box<dyn CompApi>,
}

impl Comp {
    pub fn new(model: impl Model) -> Self {
        Self {
            inner: Box::new(CompInner::new(model))
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.inner.set_id(id.into());
    }

    pub fn transform(&self) -> Option<&Transform> {
        self.inner.transform()
    }

    pub fn transform_mut(&mut self) -> Option<&mut Transform> {
        self.inner.transform_mut()
    }

    pub fn set_transform(&mut self, transform: impl Into<Transform>) {
        self.inner.set_transform(transform.into());
    }

    #[inline]
    pub fn inner<M: Model>(&self) -> &CompInner<M> {
        (*self.inner)
            .as_any()
            .downcast_ref::<CompInner<M>>()
            .unwrap_or_else(|| panic!("Can't downcast CompInner to {}", type_name::<CompInner<M>>()))
    }

    #[inline]
    pub fn inner_mut<M: Model>(&mut self) -> &mut CompInner<M> {
        (*self.inner)
            .as_any_mut()
            .downcast_mut::<CompInner<M>>()
            .unwrap_or_else(|| panic!("Can't downcast mut CompInner to {}", type_name::<CompInner<M>>()))
    }

    #[inline]
    pub fn model<M: Model>(&self) -> &M {
        &self.inner::<M>().model
    }

    #[inline]
    pub fn model_mut<M: Model>(&mut self) -> &mut M {
        &mut self.inner_mut::<M>().model
    }

    pub fn send<M: Model>(&mut self, msg: M::Message) {
        let inner = self.inner_mut::<M>();
        inner.view_state.update(inner.model.update(msg));
    }

    pub fn send_system_msg(&mut self, msg: SystemMessage) {
        self.inner.send_system_msg(msg);
    }

    pub fn update_view(&mut self) {
        self.inner.update_view();
    }
}

impl CompositeShape for Comp {
    fn shape(&self) -> Option<&Shape> {
        self.inner.as_composite_shape()?.shape()
    }

    fn shape_mut(&mut self) -> Option<&mut Shape> {
        self.inner.as_composite_shape_mut()?.shape_mut()
    }

    fn children(&self) -> Option<CompositeShapeIter> {
        self.inner.as_composite_shape()?.children()
    }

    fn children_mut(&mut self) -> Option<CompositeShapeIterMut> {
        self.inner.as_composite_shape_mut()?.children_mut()
    }
}

pub struct CompInner<M: Model> {
    id: Option<String>,
    props: Option<M::Properties>,
    pub model: M,
    view: Option<Node<M>>,
    pub view_state: ChangeViewState,
    transform: Option<Transform>,
}

impl<M: Model> CompInner<M> {
    pub fn new(model: M) -> Self {
        let view = model.build_view();

        Self {
            id: None,
            props: None,
            model,
            view: Some(view),
            view_state: Default::default(),
            transform: None
        }
    }
}

impl<M: Model> CompApi for CompInner<M> {
    fn id(&self) -> Option<&str> {
        self.id.as_ref().map(|id| id.as_str())
    }

    fn set_id(&mut self, id: String) {
        self.id = Some(id.into());
    }

    fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    fn transform_mut(&mut self) -> Option<&mut Transform> {
        self.transform.as_mut()
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = Some(transform.into());
    }

    fn as_composite_shape(&self) -> Option<&dyn CompositeShape> {
        self.view.as_ref().map(|node| node as &dyn CompositeShape)
    }

    fn as_composite_shape_mut(&mut self) -> Option<&mut dyn CompositeShape> {
        self.view.as_mut().map(|node| node as &mut dyn CompositeShape)
    }

    fn send_system_msg(&mut self, msg: SystemMessage) {
        let mut outputs = vec![];
        if let Some(msg) = self.model.system_update(msg) {
            outputs.push(msg);
        }

        if let Some(view) = self.view.as_mut() {
            view.send_system_msg(msg, &mut outputs);
        }

        for msg in outputs {
            self.view_state.update(self.model.update(msg));
        }
    }

    fn update_view(&mut self) {
        let mut need_to_propagate_update = true;
        if self.view_state.need_rebuild {
            let view = self.model.build_view();
            self.view = Some(view);
            self.view_state.need_rebuild = false;
            need_to_propagate_update = false;
        }

        if self.view_state.need_modify {
            let mut view = self.view.take().unwrap();
            self.model.modify_view(&mut view);
            self.view = Some(view);
            self.view_state.need_modify = false;
        }

        if need_to_propagate_update {
            if let Some(view) = self.view.as_mut() {
                view.update_view();
            }
        }
    }
}
