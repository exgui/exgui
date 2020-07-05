use std::{env, mem};

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    builder::*, Model, ChangeView, Node, Comp, Color, PathCommand::*,
    MousePos, Shaped, Real, VirtualKeyCode, SystemMessage,
};

enum CaretAction {
    Put(Real),
    MoveLeft,
    MoveRight,
    Input(char),
    Delete,
    Backspace,
    Redraw,
    None,
}

impl CaretAction {
    fn take(&mut self) -> Self {
        mem::replace(self, CaretAction::None)
    }
}

struct EditBox {
    initial_text: String,
    editable: bool,
    focus: bool,
    caret_idx: usize,
    caret_action: CaretAction,
}

#[derive(Clone)]
pub enum Msg {
    OnFocus(MousePos),
    OnKeyDown(VirtualKeyCode),
    Input(char),
    Draw,
    None,
}

impl Model for EditBox {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        EditBox {
            initial_text: "Cos or sin".to_string(),
            editable: true,
            focus: false,
            caret_action: CaretAction::None,
            caret_idx: 0,
        }
    }

    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        match msg {
            SystemMessage::Draw(_) => Some(Msg::Draw),
            _ => None,
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
            Msg::OnKeyDown(keycode) if self.editable => {
                match keycode {
                    VirtualKeyCode::Left => {
                        self.caret_action = CaretAction::MoveLeft;
                        ChangeView::Modify
                    }
                    VirtualKeyCode::Right => {
                        self.caret_action = CaretAction::MoveRight;
                        ChangeView::Modify
                    }
                    VirtualKeyCode::Delete => {
                        self.caret_action = CaretAction::Delete;
                        ChangeView::Modify
                    }
                    VirtualKeyCode::Backspace => {
                        self.caret_action = CaretAction::Backspace;
                        ChangeView::Modify
                    }
                    _ => ChangeView::None,
                }
            }
            Msg::Input(ch) if self.editable => {
                if !(ch.is_ascii_control() || ch.is_control()) {
                    self.caret_action = CaretAction::Input(ch);
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            }
            Msg::Draw if self.editable => {
                if let CaretAction::Redraw = self.caret_action {
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            }
            _ => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        group()
            .transform(translate(50.0, 50.0))
            .child(rect()
                .id("field")
                .left_top_pos(0, 0)
                .padding_left_and_right(8.0)
                .padding_top_and_bottom(4.0)
                .width(400)
                .height(40)
                .stroke((Color::Blue, 2, 0.5))
                .on_mouse_down(|case| Msg::OnFocus(case.event.pos))
                .on_key_down(|case| {
                    if let Some(keycode) = case.event.keycode {
                        Msg::OnKeyDown(keycode)
                    } else {
                        Msg::None
                    }
                })
                .on_input_char(|case| Msg::Input(case.event))
                .child(group()
                    .id("clip_area")
                    .clip(-1, 0, 400 - 16, 40 - 8)
                    .child(text(&self.initial_text)
                        .id("text")
                        .font_name("Roboto")
                        .font_size(32)
                    )
                )
            )
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        let text = view
            .get_prim_mut("text")
            .and_then(|text| text.shape.text_mut())
            .expect("Text primitive expected");

        match self.caret_action.take() {
            CaretAction::Put(focus_pos) => {
                if let Some(idx) = text.glyph_positions.iter().position(|pos| focus_pos >= pos.min_x && focus_pos <= pos.max_x) {
                    let pos = text.glyph_positions[idx];
                    let before = focus_pos - pos.min_x;
                    let after = pos.max_x - focus_pos;
                    if before < after {
                        self.caret_idx = idx;
                    } else {
                        self.caret_idx = idx + 1;
                    }
                }
                if let Some(last) = text.glyph_positions.last() {
                    if focus_pos > last.max_x {
                        self.caret_idx = text.glyph_positions.len();
                    }
                }
                self.caret_action = CaretAction::Redraw;
            },
            CaretAction::MoveLeft => {
                self.caret_idx = self.caret_idx.checked_sub(1).unwrap_or(0);
                self.caret_action = CaretAction::Redraw;
            },
            CaretAction::MoveRight => {
                self.caret_idx = self.caret_idx + if text.glyph_positions.len() > self.caret_idx { 1 } else { 0 };
                self.caret_action = CaretAction::Redraw;
            },
            CaretAction::Input(ch) => {
                if self.caret_idx < text.glyph_positions.len() {
                    text.insert(self.caret_idx, ch);
                } else {
                    text.push(ch);
                }
                self.caret_idx += 1;
                self.caret_action = CaretAction::Redraw;
            }
            CaretAction::Delete => {
                if self.caret_idx < text.glyph_positions.len() {
                    text.remove(self.caret_idx);
                    self.caret_action = CaretAction::Redraw;
                }
            }
            CaretAction::Backspace => {
                if self.caret_idx > 0 {
                    self.caret_idx -= 1;
                    text.remove(self.caret_idx);
                    self.caret_action = CaretAction::Redraw;
                }
            }
            CaretAction::Redraw => {
                let caret_pos = if self.caret_idx > 0 {
                    text.glyph_positions[self.caret_idx - 1].max_x
                } else {
                    0.0
                };
                let text_end_pos = text.glyph_positions.last().map(|pos| pos.max_x).unwrap_or(0.0);
                let line_height = text.metrics.map(|m| m.line_height).unwrap_or(text.font_size.0);
                Self::draw_caret(view, caret_pos, text_end_pos, line_height);
            },
            CaretAction::None => (),
        }
    }
}

impl EditBox {
    fn draw_caret(view: &mut Node<Self>, caret_pos: Real, text_end_pos: Real, line_height: Real) {
        if let Some(path) = view
            .get_prim_mut("caret")
            .and_then(|caret| caret.shape.path_mut())
        {
            path.cmd[0] = Move([caret_pos, 0.0]);
            path.cmd[1] = Line([caret_pos, line_height]);
        } else if let Some(text) = view.get_prim_mut("text") {
            text.children.push(
                path(vec![Move([caret_pos, 0.0]), Line([caret_pos, line_height])])
                    .id("caret")
                    .stroke((Color::Black, 2, 0.5))
                    .build()
            );
        }

        let (min_x, max_x) = view
            .get_prim_mut("clip_area")
            .and_then(|clip_area|  clip_area.shape.group_mut())
            .expect("Clip area expected")
            .clip
            .scissor()
            .map(|scissor| {
                (scissor.x.val(), scissor.x.val() + scissor.width.val())
            })
            .expect("Clip scissor expected");

        let text_transform = view
            .get_prim_mut("text")
            .expect("Text expected")
            .transform_mut();
        let shift = text_transform
            .local_matrix()
            .expect("Local transform expected")
            .translate_xy()
            .0
            .abs();

        if caret_pos - shift > max_x {
            text_transform.translate(-caret_pos + max_x - 1.0, 0.0);
        } else if caret_pos - shift < min_x {
            text_transform.translate(min_x - caret_pos + 1.0, 0.0);
        } else if shift > 0.0 && text_end_pos - shift < max_x - 1.0 {
            text_transform.translate_add(shift.min(max_x - 1.0 - text_end_pos + shift), 0.0);
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
    app.renderer_mut().load_font("Roboto", font_path).unwrap();

    let comp = Comp::new(EditBox::create(()));
    app.run(comp);
}