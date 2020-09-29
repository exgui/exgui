use exgui::{builder::*, ChangeView, Color, Comp, LineJoin, Model, Node, PathCommand::*, Stroke};
use exgui_controller_glutin::{glutin, App};
use exgui_render_nanovg::NanovgRender;

struct Smile {
    normal_face: bool,
}

#[derive(Clone)]
pub enum Msg {
    ToggleFace,
    Nope,
}

impl Model for Smile {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        Smile { normal_face: true }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::ToggleFace => {
                self.normal_face = !self.normal_face;
                ChangeView::Rebuild
            },
            Msg::Nope => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        group()
            .transform(translate(50.0, 50.0))
            .child(
                rect()
                    .left_top_pos(0, 0)
                    .width(300)
                    .height(300)
                    .stroke((Color::Black, 2, 0.5))
                    .child(
                        group()
                            .stroke((Color::Black, 5))
                            .child(
                                circle()
                                    .center(150, 150)
                                    .radius(100)
                                    .fill(if self.normal_face { Color::Yellow } else { Color::Red })
                                    .on_mouse_down(|_| Msg::ToggleFace),
                            )
                            .child(
                                group()
                                    .fill(if self.normal_face { Color::Black } else { Color::White })
                                    .child(circle().center(110, 130).radius(15))
                                    .child(circle().center(190, 130).radius(15))
                                    .child(self.view_mouth()),
                            ),
                    ),
            )
            .build()
    }
}

impl Smile {
    fn view_mouth(&self) -> Node<Self> {
        if self.normal_face {
            path(vec![
                Move([100.0, 180.0]),
                BezCtrl([150.0, 230.0]),
                QuadBezTo([200.0, 180.0]),
                BezCtrl([150.0, 210.0]),
                QuadBezTo([100.0, 180.0]),
            ])
            .stroke(Stroke {
                width: 5.0,
                line_join: LineJoin::Round,
                ..Default::default()
            })
            .build()
        } else {
            path(vec![
                Move([100.0, 205.0]),
                BezCtrl([150.0, 155.0]),
                QuadBezTo([200.0, 205.0]),
                BezCtrl([150.0, 175.0]),
                QuadBezTo([100.0, 205.0]),
            ])
            .stroke(Stroke {
                width: 5.0,
                line_join: LineJoin::Round,
                ..Default::default()
            })
            .build()
        }
    }
}

fn main() {
    let mut app = App::new(
        glutin::window::WindowBuilder::new()
            .with_title("ExGUI smile")
            .with_inner_size(glutin::dpi::PhysicalSize::new(480, 480)),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRender::default(),
    )
    .unwrap();
    app.init().unwrap();

    let comp = Comp::new(Smile::create(()));
    app.run(comp);
}
