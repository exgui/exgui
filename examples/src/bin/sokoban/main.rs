use std::{env, mem, time::Duration};

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    builder::*, Model, ChangeView, Node, Comp, Color, PathCommand::*, Transform,
    MousePos, Shaped, Real, VirtualKeyCode, SystemMessage, Pct, Stroke, LineJoin,
};

use self::levels::Level;

mod levels;

struct Canvas {
    width: f32,
    height: f32,
    cell_size: f32,
    scale_factor: f32,
}

impl Canvas {
    const WIDTH: f32 = 800.0;
    const HEIGHT: f32 = 600.0;

    fn calc_cell_size(_width: f32, height: f32) -> f32 {
        height / 24.0
    }

    fn new() -> Self {
        Self {
            width: Self::WIDTH,
            height: Self::HEIGHT,
            cell_size: Self::calc_cell_size(Self::WIDTH, Self::HEIGHT),
            scale_factor: 1.0,
        }
    }

    fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.cell_size = Self::calc_cell_size(self.width, self.height);
    }
}

struct Game {
    canvas: Canvas,
    level: Level,
}

enum Msg {
    Resize {
        width: f32,
        height: f32,
    },
    Scroll(f32),
}

impl Model for Game {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        Self {
            canvas: Canvas::new(),
            level: Level::new(),
        }
    }

    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        match msg {
            SystemMessage::WindowResized { width, height } => Some(Msg::Resize {
                width: width as f32,
                height: height as f32,
            }),
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::Resize { width, height } => {
                self.canvas.resize(width, height);
                ChangeView::Rebuild
            },
            Msg::Scroll(delta) => {
                self.canvas.scale_factor = (self.canvas.scale_factor + delta * 0.1).max(0.0);
                ChangeView::Rebuild
            }
            _ => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        let mut cells = vec![];
        let field_x = (self.canvas.width - self.level.cols() as f32 * self.canvas.cell_size) / 2.0;
        let field_y = (self.canvas.height - self.level.rows() as f32 * self.canvas.cell_size) / 2.0;

        for (row, line) in self.level.field().into_iter().enumerate() {
            for (col, &cell) in line.as_bytes().into_iter().enumerate() {
                let x = field_x + col as f32 * self.canvas.cell_size;
                let y = field_y + row as f32 * self.canvas.cell_size;
                match cell {
                    b'O' => cells.push(self.build_wall(x, y)),
                    b'#' => cells.push(self.build_box(x, y)),
                    b'*' => cells.push(self.build_docker(x, y)),
                    b'+' => cells.push(self.build_place(x, y)),
                    _ => (),
                }
            }
        }

        rect()
            .width(Pct(100))
            .height(Pct(100))
            .fill(Color::RGB(0.8, 0.9, 1.0))
            .on_mouse_scroll(|case| Msg::Scroll(case.event.delta.1))
            .child(group()
                .transform(Transform::new()
                    .with_scale(self.canvas.scale_factor, self.canvas.scale_factor)
                    .with_translation(
                        -(self.canvas.scale_factor * self.canvas.width - self.canvas.width) / 2.0,
                        -(self.canvas.scale_factor * self.canvas.height - self.canvas.height) / 2.0,
                    )
                )
                .children(cells)
            )
            .build()
    }
}

impl Game {
    fn build_wall(&self, x: f32, y: f32) -> Node<Self> {
        let brick_color = Color::RGB(1.0, 0.4, 0.2);
        let brick_space = self.canvas.cell_size / 15.0;
        let brick_height = self.canvas.cell_size / 2.0 - brick_space;
        let brick_chunk_size = (self.canvas.cell_size - brick_space) / 3.0;
        let round_radius = brick_space / 1.5;

        rect()
            .id("wall")
            .width(self.canvas.cell_size)
            .height(self.canvas.cell_size)
            .transparency(1.0)
            .transform(translate(x, y))
            .child(rect()
                .width(brick_chunk_size)
                .height(brick_height)
                .fill(brick_color)
                .rounding_top_right(round_radius)
                .rounding_bottom_right(round_radius)
                .transform(translate(0.0, brick_space / 2.0))
            )
            .child(rect()
                .width(brick_chunk_size * 2.0)
                .height(brick_height)
                .fill(brick_color)
                .rounding_top_left(round_radius)
                .rounding_bottom_left(round_radius)
                .transform(translate(brick_chunk_size + brick_space, brick_space / 2.0))
            )
            .child(rect()
                .width(brick_chunk_size * 2.0)
                .height(brick_height)
                .fill(brick_color)
                .rounding_top_right(round_radius)
                .rounding_bottom_right(round_radius)
                .transform(translate(0.0, brick_height + brick_space * 1.5))
            )
            .child(rect()
                .width(brick_chunk_size)
                .height(brick_height)
                .fill(brick_color)
                .rounding_top_left(round_radius)
                .rounding_bottom_left(round_radius)
                .transform(translate(brick_chunk_size * 2.0 + brick_space, brick_height + brick_space * 1.5))
            )
            .build()
    }

    fn build_box(&self, x: f32, y: f32) -> Node<Self> {
        let board_color = Color::RGB(1.0, 0.7, 0.1);
        let board_space = self.canvas.cell_size / 15.0;
        let board_chunk_size = (self.canvas.cell_size - board_space * 2.0) / 3.0;
        let round_radius = 1.0;

        rect()
            .id("box")
            .width(self.canvas.cell_size)
            .height(self.canvas.cell_size)
            .transparency(1.0)
            .transform(translate(x, y))
            .child(rect()
                .width(self.canvas.cell_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(0.0, 0.0))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(0.0, board_chunk_size + board_space))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_chunk_size + board_space, board_chunk_size + board_space))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_chunk_size * 2.0 + board_space * 2.0, board_chunk_size + board_space))
            )
            .child(rect()
                .width(self.canvas.cell_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(0.0, board_chunk_size * 2.0 + board_space * 2.0))
            )
            .build()
    }

    fn build_docker(&self, x: f32, y: f32) -> Node<Self> {
        let docker_color = Color::RGB(0.4, 0.4, 0.4);
        let docker_brush_size = self.canvas.cell_size / 10.0;
        let head_radius = self.canvas.cell_size / 5.0;

        rect()
            .id("docker")
            .width(self.canvas.cell_size)
            .height(self.canvas.cell_size)
            .transparency(1.0)
            .transform(translate(x, y))
            .child(circle()
                .radius(head_radius)
                .fill(docker_color)
                .transform(translate(self.canvas.cell_size / 2.0, head_radius + docker_brush_size))
            )
            .child(path(vec![
                    Move([docker_brush_size * 2.2, self.canvas.cell_size - docker_brush_size * 2.0]),
                    BezCtrl([self.canvas.cell_size / 2.0, head_radius * 2.0]),
                    QuadBezTo([self.canvas.cell_size - docker_brush_size * 2.2, self.canvas.cell_size - docker_brush_size * 2.0]),
                    LineAlonX(docker_brush_size * 2.2)
                ])
                .fill(docker_color)
                .stroke(Stroke {
                    paint: docker_color.into(),
                    width: docker_brush_size,
                    line_join: LineJoin::Round,
                    ..Default::default()
                })
            )
            .build()
    }

    fn build_place(&self, x: f32, y: f32) -> Node<Self> {
        let place_color = Color::RGB(0.15, 0.5, 0.9);
        let place_size = self.canvas.cell_size * 0.5;
        let place_diagonal = (2.0 * place_size.powi(2)).sqrt();
        let round_radius = 1.0;

        rect()
            .id("place")
            .width(self.canvas.cell_size)
            .height(self.canvas.cell_size)
            .transparency(1.0)
            .transform(translate(x, y))
            .child(rect()
                .width(place_size)
                .height(place_size)
                .fill(place_color)
                .rounding(round_radius)
                .transform(Transform::new()
                    .with_rotation(std::f32::consts::PI / 4.0)
                    .with_translation(self.canvas.cell_size / 2.0, (self.canvas.cell_size - place_diagonal) / 2.0)
                )
            )
            .build()
    }
}

fn main() {
    let mut app = App::new(
        glutin::window::WindowBuilder::new()
            .with_title("ExGUI Sokoban")
            .with_inner_size(glutin::dpi::PhysicalSize::new(Canvas::WIDTH, Canvas::HEIGHT)),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRender::default()
    ).unwrap();
    app.init().unwrap();

    let font_path = env::current_dir().unwrap().join("examples").join("resources").join("Roboto-Regular.ttf");
    app.renderer_mut().load_font("Roboto", font_path).unwrap();

    let comp = Comp::new(Game::create(()));
    app.run(comp);
}