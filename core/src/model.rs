use std::time::Duration;

use crate::{InputEvent, Node};

pub trait Model: Sized + 'static {
    type Message;
    type Properties;

    fn create(props: Self::Properties) -> Self;

    #[allow(unused_variables)]
    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        None
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView;

    fn build_view(&self) -> Node<Self>;

    #[allow(unused_variables)]
    fn modify_view(&mut self, view: &mut Node<Self>) {
        unimplemented!();
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChangeView {
    Rebuild,
    Modify,
    None,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ChangeViewState {
    pub need_rebuild: bool,
    pub need_modify: bool,
}

impl ChangeViewState {
    pub fn update(&mut self, change_view: ChangeView) {
        match change_view {
            ChangeView::Rebuild => self.need_rebuild = true,
            ChangeView::Modify => self.need_modify = true,
            ChangeView::None => (),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemMessage {
    WindowResized { width: u32, height: u32 },
    Draw(Duration),
    Input(InputEvent),
}
