use std::{ops::Deref, time::Duration};

use crate::{MouseDown, KeyboardEvent, Model, Prim};

pub struct On<'a, M: Model, E> {
    pub prim: &'a Prim<M>,
    pub event: E,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventName(&'static str);

impl EventName {
    pub const WINDOW_RESIZED: EventName = EventName("WindowResized");
    pub const DRAW: EventName = EventName("Draw");
    pub const ON_MOUSE_DOWN: EventName = EventName("OnMouseDown");
    pub const ON_KEY_DOWN: EventName = EventName("OnKeyDown");
    pub const ON_KEY_UP: EventName = EventName("OnKeyUp");
    pub const ON_CLICK: EventName = EventName("OnClick");
    pub const ON_INPUT_CHAR: EventName = EventName("OnInputChar");
    pub const ON_BLUR: EventName = EventName("OnBlur");
}

impl Deref for EventName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}


pub enum Listener<M: Model> {
    WindowResized(fn (u32, u32) -> M::Message),
    Draw(fn (Duration) -> M::Message),
    OnMouseDown(fn (On<M, MouseDown>) -> M::Message),
    OnKeyDown(fn (On<M, KeyboardEvent>) -> M::Message),
    OnKeyUp(fn (On<M, KeyboardEvent>) -> M::Message),
    OnClick(fn (On<M, MouseDown>) -> M::Message),
    OnInputChar(fn (On<M, char>) -> M::Message),
    OnBlur(fn (On<M, MouseDown>) -> M::Message),
}

impl<M: Model> Listener<M> {
    pub fn event_name(&self) -> EventName {
        match self {
            Listener::WindowResized(_) => EventName::WINDOW_RESIZED,
            Listener::Draw(_) => EventName::DRAW,
            Listener::OnMouseDown(_) => EventName::ON_MOUSE_DOWN,
            Listener::OnKeyDown(_) => EventName::ON_KEY_DOWN,
            Listener::OnKeyUp(_) => EventName::ON_KEY_UP,
            Listener::OnClick(_) => EventName::ON_CLICK,
            Listener::OnInputChar(_) => EventName::ON_INPUT_CHAR,
            Listener::OnBlur(_) => EventName::ON_BLUR,
        }
    }
}

//pub struct OnClickListener<Msg>(fn (MousePress) -> Msg);
//
//impl<Msg> OnClickListener<Msg> {
//    pub const EVENT_NAME: EventName = EventName("OnClick");
//}
//
//impl<Msg> Deref for OnClickListener<Msg> {
//    type Target = fn (MousePress) -> Msg;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}