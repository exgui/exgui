use std::any::{type_name, Any};

use crate::{
    ChangeViewState, CompositeShape, CompositeShapeIter, CompositeShapeIterMut, Model, Node, Shape, SystemMessage,
    Transform,
};

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
    fn transform(&self) -> &Transform;
    fn transform_mut(&mut self) -> &mut Transform;
    fn set_transform(&mut self, transform: Transform);
    fn as_composite_shape(&self) -> Option<&dyn CompositeShape>;
    fn as_composite_shape_mut(&mut self) -> Option<&mut dyn CompositeShape>;
    fn send_system_msg(&mut self, msg: SystemMessage);
    fn update_view(&mut self) -> bool;
    fn need_recalc(&self) -> bool;
}

pub struct Comp {
    inner: Box<dyn CompApi>,
}

impl Comp {
    pub fn new(model: impl Model) -> Self {
        Self {
            inner: Box::new(CompInner::new(model)),
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    pub fn set_id(&mut self, id: impl Into<String>) {
        self.inner.set_id(id.into());
    }

    pub fn transform(&self) -> &Transform {
        self.inner.transform()
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
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

    pub fn update_view(&mut self) -> bool {
        self.inner.update_view()
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

    fn need_recalc(&self) -> Option<bool> {
        Some(self.inner.need_recalc())
    }
}

pub struct CompInner<M: Model> {
    id: Option<String>,
    _props: Option<M::Properties>,
    model: M,
    view: Option<Node<M>>,
    view_state: ChangeViewState,
    transform: Transform,
}

impl<M: Model> CompInner<M> {
    pub fn new(model: M) -> Self {
        let view = model.build_view();

        Self {
            id: None,
            _props: None,
            model,
            view: Some(view),
            view_state: ChangeViewState {
                need_rebuild: true,
                ..Default::default()
            },
            transform: Default::default(),
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

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform.into();
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

    fn update_view(&mut self) -> bool {
        let mut need_to_propagate_update = true;
        let mut is_updated = false;
        if self.view_state.need_rebuild {
            let view = self.model.build_view();
            self.view = Some(view);
            self.view_state.need_rebuild = false;
            need_to_propagate_update = false;
            is_updated = true;
        }

        if self.view_state.need_modify {
            let mut view = self.view.take().unwrap();
            self.model.modify_view(&mut view);
            self.view = Some(view);
            self.view_state.need_modify = false;
            is_updated = true;
        }

        if need_to_propagate_update {
            if let Some(view) = self.view.as_mut() {
                is_updated = view.update_view() || is_updated;
            }
        }
        self.view_state.need_recalc = is_updated;
        is_updated
    }

    fn need_recalc(&self) -> bool {
        self.view_state.need_recalc
    }
}
