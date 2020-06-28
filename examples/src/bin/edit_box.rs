use std::{env, mem};

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    builder::*, Model, ChangeView, Node, Comp, Color, PathCommand::*,
    MousePos, Shaped, Real, VirtualKeyCode,
};

enum CaretAction {
    Put(Real),
    MoveLeft,
    MoveRight,
    None,
}

impl CaretAction {
    fn take(&mut self) -> Self {
        mem::replace(self, CaretAction::None)
    }
}

struct EditBox {
    text: String,
    editable: bool,
    focus: bool,
    caret_idx: usize,
    caret_action: CaretAction,
}

#[derive(Clone)]
pub enum Msg {
    OnFocus(MousePos),
    OnKeyDown(VirtualKeyCode),
    None,
}

impl Model for EditBox {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        EditBox {
            text: "Cos or sin".to_string(),
            editable: true,
            focus: false,
            caret_action: CaretAction::None,
            caret_idx: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::OnFocus(pos) => {
                self.focus = true;
                if self.editable {
                    self.caret_action = CaretAction::Put(pos.x - 50.0);
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
            Msg::OnKeyDown(keycode) => {
                match keycode {
                    VirtualKeyCode::Left => {
                        self.caret_action = CaretAction::MoveLeft;
                        ChangeView::Modify
                    }
                    VirtualKeyCode::Right => {
                        self.caret_action = CaretAction::MoveRight;
                        ChangeView::Modify
                    }
                    _ => ChangeView::None,
                }
            }
            Msg::None => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        group()
            .transform(translate(50.0, 50.0))
            .child(rect()
                .id("field")
                .left_top_pos(0, 0)
                .width(400)
                .height(40)
                .stroke((Color::Blue, 2, 0.5))
                .on_mouse_down(|event| Msg::OnFocus(event.pos))
                .on_key_down(|event| {
                    if let Some(keycode) = event.keycode {
                        Msg::OnKeyDown(keycode)
                    } else {
                        Msg::None
                    }
                })
                .on_input_char(|ch| { println!("Char: {:?}", ch); Msg::None })
                .child(text(&self.text)
                    .id("text")
                    .font_name("Roboto")
                    .font_size(32),
                )
            )
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        let mut caret_pos = 0.0;
        if let Some(text) = view.get_prim("text").and_then(|text| text.shape.text()) {
            match self.caret_action.take() {
                CaretAction::Put(focus_pos) => {
                    if let Some(idx) = text.glyph_positions.iter().position(|pos| focus_pos >= pos.min_x && focus_pos <= pos.max_x) {
                        let pos = text.glyph_positions[idx];
                        let before = focus_pos - pos.min_x;
                        let after = pos.max_x - focus_pos;
                        if before < after {
                            caret_pos = pos.min_x;
                            self.caret_idx = idx;
                        } else {
                            caret_pos = pos.max_x;
                            self.caret_idx = idx + 1;
                        }
                    }
                    if let Some(last) = text.glyph_positions.last() {
                        if focus_pos > last.max_x {
                            caret_pos = last.max_x;
                            self.caret_idx = text.glyph_positions.len();
                        }
                    }
                },
                CaretAction::MoveLeft => {
                    self.caret_idx = self.caret_idx.checked_sub(1).unwrap_or(0);
                    caret_pos = text.glyph_positions[self.caret_idx].min_x;
                },
                CaretAction::MoveRight => {
                    self.caret_idx = self.caret_idx + if self.text.chars().count() > self.caret_idx { 1 } else { 0 };
                    caret_pos = text
                        .glyph_positions
                        .get(self.caret_idx)
                        .map(|pos| pos.min_x)
                        .or_else(|| text.glyph_positions.last().map(|last| last.max_x))
                        .unwrap_or(caret_pos);
                },
                CaretAction::None => (),
            }
        }

        Self::draw_caret(view, caret_pos);
    }
}

impl EditBox {
    fn draw_caret(view: &mut Node<Self>, caret_pos: Real) {
        if let Some(path) = view
            .get_prim_mut("caret")
            .and_then(|caret| caret.shape.path_mut())
        {
            path.cmd[0] = Move([caret_pos, 5.0]);
            path.cmd[1] = Line([caret_pos, 35.0]);
        } else if let Some(field) = view.get_prim_mut("field") {
            field.children.push(
                path(vec![Move([caret_pos, 5.0]), Line([caret_pos, 35.0])])
                    .id("caret")
                    .stroke((Color::Black, 2, 0.5))
                    .build()
            );
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::window::WindowBuilder::new()
            .with_title("ExGUI edit box")
            .with_inner_size(glutin::dpi::PhysicalSize::new(480, 480)),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRender::default()
    ).unwrap();
    app.init().unwrap();

    let font_path = env::current_dir().unwrap().join("examples").join("resources").join("Roboto-Regular.ttf");
    app.render_mut().load_font("Roboto", font_path).unwrap();

    let comp = Comp::new(EditBox::create(()));
    app.run(comp);
}