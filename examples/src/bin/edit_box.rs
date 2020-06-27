use std::env;

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    builder::*, Model, ChangeView, Node, Comp, Color, PathCommand::*,
    MousePos, Shaped, Real,
};

struct EditBox {
    text: String,
    focus_pos: Real,
    editable: bool,
    focus: bool,
}

#[derive(Clone)]
pub enum Msg {
    OnFocus(MousePos),
    Nope,
}

impl Model for EditBox {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        EditBox {
            text: "Cos or sin".to_string(),
            focus_pos: 2.0,
            editable: true,
            focus: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::OnFocus(pos) => {
                self.focus = true;
                if self.editable {
                    self.focus_pos = pos.x - 50.0;
                }
                ChangeView::Modify
            },
            Msg::Nope => ChangeView::None,
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
                .child(text(&self.text)
                    .id("text")
                    .font_name("Roboto")
                    .font_size(32),
                )
            )
            .build()
    }

    fn modify_view(&self, view: &mut Node<Self>) {
        let mut cursor_pos = self.focus_pos;
        if let Some(text) = view.get_prim("text").and_then(|text| text.shape.text()) {
            for pos in &text.glyph_positions {
                if cursor_pos >= pos.min_x && cursor_pos <= pos.max_x {
                    let before = cursor_pos - pos.min_x;
                    let after = pos.max_x - cursor_pos;
                    if before < after {
                        cursor_pos = pos.min_x;
                    } else {
                        cursor_pos = pos.max_x;
                    }
                }
            }
            if let Some(last) = text.glyph_positions.last() {
                if cursor_pos > last.max_x {
                    cursor_pos = last.max_x;
                }
            }
        }

        if let Some(path) = view
            .get_prim_mut("cursor")
            .and_then(|cursor| cursor.shape.path_mut())
        {
            path.cmd[0] = Move([cursor_pos, 5.0]);
            path.cmd[1] = Line([cursor_pos, 35.0]);
        } else if let Some(field) = view.get_prim_mut("field") {
            field.children.push(
                path(vec![Move([cursor_pos, 5.0]), Line([cursor_pos, 35.0])])
                    .id("cursor")
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