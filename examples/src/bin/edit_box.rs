use std::{env, mem, time::Duration};

use exgui::{
    builder::*, ChangeView, Color, Comp, Model, MousePos, Node, PathCommand::*, Real, Shaped, SystemMessage,
    VirtualKeyCode,
};
use exgui_controller_glutin::{glutin, App};
use exgui_render_nanovg::NanovgRender;

enum CaretAction {
    Put(Real),
    MoveLeft,
    MoveRight,
    MoveStart,
    MoveEnd,
    Input(char),
    Delete,
    Backspace,
    Redraw,
    Blink,
    None,
}

impl CaretAction {
    fn take(&mut self) -> Self {
        mem::replace(self, CaretAction::None)
    }
}

struct Caret {
    idx: usize,
    action: CaretAction,
    blink: Duration,
    show: bool,
}

impl Caret {
    fn new() -> Self {
        Self {
            action: CaretAction::None,
            idx: 0,
            blink: Duration::default(),
            show: true,
        }
    }

    fn reset(&mut self) {
        self.idx = 0;
        self.action = CaretAction::None;
        self.reset_blink();
        self.show = false;
    }

    fn reset_blink(&mut self) {
        self.blink = Duration::default();
    }

    fn update_blink(&mut self) -> bool {
        if self.blink.as_millis() >= 1000 {
            self.reset_blink();
            self.show = !self.show;
            true
        } else {
            false
        }
    }

    fn update_action(&mut self, action: CaretAction) {
        self.action = action;
        match &self.action {
            CaretAction::Blink | CaretAction::None => (),
            _ => {
                self.reset_blink();
                self.show = true;
            },
        }
    }
}

struct EditBox {
    initial_text: String,
    editable: bool,
    focus: bool,
    caret: Caret,
}

#[derive(Clone)]
pub enum Msg {
    OnFocus(MousePos),
    OnBlur,
    OnKeyDown(VirtualKeyCode),
    Input(char),
    Draw(Duration),
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
            caret: Caret::new(),
        }
    }

    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        match msg {
            SystemMessage::Draw(elapsed) => Some(Msg::Draw(elapsed)),
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::OnFocus(pos) => {
                self.focus = true;
                if self.editable {
                    self.caret.update_action(CaretAction::Put(pos.x));
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
            Msg::OnKeyDown(keycode) if self.focus => match keycode {
                VirtualKeyCode::Left => {
                    self.caret.update_action(CaretAction::MoveLeft);
                    ChangeView::Modify
                },
                VirtualKeyCode::Right => {
                    self.caret.update_action(CaretAction::MoveRight);
                    ChangeView::Modify
                },
                VirtualKeyCode::Home => {
                    self.caret.update_action(CaretAction::MoveStart);
                    ChangeView::Modify
                },
                VirtualKeyCode::End => {
                    self.caret.update_action(CaretAction::MoveEnd);
                    ChangeView::Modify
                },
                VirtualKeyCode::Delete if self.editable => {
                    self.caret.update_action(CaretAction::Delete);
                    ChangeView::Modify
                },
                VirtualKeyCode::Backspace if self.editable => {
                    self.caret.update_action(CaretAction::Backspace);
                    ChangeView::Modify
                },
                _ => ChangeView::None,
            },
            Msg::Input(ch) if self.editable => {
                if !(ch.is_ascii_control() || ch.is_control()) {
                    self.caret.update_action(CaretAction::Input(ch));
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
            Msg::Draw(elapsed) if self.focus => {
                self.caret.blink += elapsed;
                if let CaretAction::Redraw = self.caret.action {
                    ChangeView::Modify
                } else if self.caret.update_blink() {
                    self.caret.update_action(CaretAction::Blink);
                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
            Msg::OnBlur if self.focus => {
                self.focus = false;
                self.caret.reset();
                self.caret.action = CaretAction::Blink;
                ChangeView::Modify
            },
            _ => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        group()
            .transform(translate(50.0, 50.0))
            .child(
                rect()
                    .id("field")
                    .left_top_pos(0, 0)
                    .padding_left_and_right(8.0)
                    .padding_top_and_bottom(4.0)
                    .width(400)
                    .height(40)
                    .rounding(4)
                    .stroke((Color::Blue, 2, 0.5))
                    .on_mouse_down(|case| Msg::OnFocus(case.event.pos))
                    .on_blur(|_| Msg::OnBlur)
                    .on_key_down(|case| {
                        if let Some(keycode) = case.event.keycode {
                            Msg::OnKeyDown(keycode)
                        } else {
                            Msg::None
                        }
                    })
                    .on_input_char(|case| Msg::Input(case.event))
                    .child(
                        group()
                            .id("clip_area")
                            .clip(-1, 0, 400 - 16, 40 - 8)
                            .child(text(&self.initial_text).id("text").font_name("Roboto").font_size(32)),
                    ),
            )
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        let text = view
            .get_prim_mut("text")
            .and_then(|text| text.shape.text_mut())
            .expect("Text primitive expected");

        match self.caret.action.take() {
            CaretAction::Put(mut focus_pos) => {
                let matrix = text
                    .transform
                    .global_matrix()
                    .unwrap_or_else(|| text.transform.matrix());
                if !matrix.is_identity() {
                    focus_pos = focus_pos - matrix.translate_xy().0;
                }

                if let Some(idx) = text
                    .glyph_positions
                    .iter()
                    .position(|pos| focus_pos >= pos.min_x && focus_pos <= pos.max_x)
                {
                    let pos = text.glyph_positions[idx];
                    let before = focus_pos - pos.min_x;
                    let after = pos.max_x - focus_pos;
                    if before < after {
                        self.caret.idx = idx;
                    } else {
                        self.caret.idx = idx + 1;
                    }
                }
                if let Some(last) = text.glyph_positions.last() {
                    if focus_pos > last.max_x {
                        self.caret.idx = text.glyph_positions.len();
                    }
                }
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::MoveLeft => {
                self.caret.idx = self.caret.idx.checked_sub(1).unwrap_or(0);
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::MoveRight => {
                self.caret.idx = self.caret.idx
                    + if text.glyph_positions.len() > self.caret.idx {
                        1
                    } else {
                        0
                    };
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::MoveStart => {
                self.caret.idx = 0;
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::MoveEnd => {
                self.caret.idx = text.glyph_positions.len();
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::Input(ch) => {
                if self.caret.idx < text.glyph_positions.len() {
                    text.insert(self.caret.idx, ch);
                } else {
                    text.push(ch);
                }
                self.caret.idx += 1;
                self.caret.update_action(CaretAction::Redraw);
            },
            CaretAction::Delete => {
                if self.caret.idx < text.glyph_positions.len() {
                    text.remove(self.caret.idx);
                    self.caret.update_action(CaretAction::Redraw);
                }
            },
            CaretAction::Backspace => {
                if self.caret.idx > 0 {
                    self.caret.idx -= 1;
                    text.remove(self.caret.idx);
                    self.caret.update_action(CaretAction::Redraw);
                }
            },
            CaretAction::Redraw => {
                let caret_pos = if self.caret.idx > 0 {
                    text.glyph_positions[self.caret.idx - 1].max_x
                } else {
                    0.0
                };
                let text_end_pos = text.glyph_positions.last().map(|pos| pos.max_x).unwrap_or(0.0);
                let line_height = text.metrics.map(|m| m.line_height).unwrap_or(text.font_size.0);
                Self::draw_caret(view, caret_pos, text_end_pos, line_height, self.caret.show);
            },
            CaretAction::Blink => {
                if let Some(path) = view.get_prim_mut("caret").and_then(|caret| caret.shape.path_mut()) {
                    path.transparency = if self.caret.show { 0.0 } else { 1.0 };
                }
            },
            CaretAction::None => (),
        }
    }
}

impl EditBox {
    fn draw_caret(view: &mut Node<Self>, caret_pos: Real, text_end_pos: Real, line_height: Real, show: bool) {
        if let Some(path) = view.get_prim_mut("caret").and_then(|caret| caret.shape.path_mut()) {
            path.cmd[0] = Move([caret_pos, 0.0]);
            path.cmd[1] = Line([caret_pos, line_height]);
            path.transparency = if show { 0.0 } else { 1.0 };
        } else if let Some(text) = view.get_prim_mut("text") {
            text.children.push(
                path(vec![Move([caret_pos, 0.0]), Line([caret_pos, line_height])])
                    .id("caret")
                    .stroke((Color::Black, 2, 0.5))
                    .build(),
            );
        }

        let (min_x, max_x) = view
            .get_prim_mut("clip_area")
            .and_then(|clip_area| clip_area.shape.group_mut())
            .expect("Clip area expected")
            .clip
            .scissor()
            .map(|scissor| (scissor.x.val(), scissor.x.val() + scissor.width.val()))
            .expect("Clip scissor expected");

        let text_transform = view.get_prim_mut("text").expect("Text expected").transform_mut();
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
        NanovgRender::default(),
    )
    .unwrap();
    app.init().unwrap();

    let font_path = env::current_dir()
        .unwrap()
        .join("examples")
        .join("resources")
        .join("Roboto-Regular.ttf");
    app.renderer_mut().load_font("Roboto", font_path).unwrap();

    let comp = Comp::new(EditBox::create(()));
    app.run(comp);
}
