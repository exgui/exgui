use std::{env, time::Duration};

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    builder::*, Model, ChangeView, Node, Comp, Color, PathCommand::*, Transform,
    MousePos, Shaped, Real, VirtualKeyCode, SystemMessage, Pct, Stroke, LineJoin,
};

use self::{
    animate::Animate,
    levels::{Cell, Level},
};

mod animate;
mod levels;

struct Canvas {
    width: f32,
    height: f32,
    cell_size: f32,
    scale_factor: Animate<f32>,
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
            scale_factor: Animate::new(0.0, 1.0, 0.002),
        }
    }

    fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.cell_size = Self::calc_cell_size(self.width, self.height);
    }
}

#[derive(Debug)]
struct SkewBox {
    id: String,
    row: usize,
    col: usize,
    x: Animate<f32>,
    y: Animate<f32>,
}

#[derive(Default)]
struct Docker {
    row: usize,
    col: usize,
    x: Animate<f32>,
    y: Animate<f32>,
    skew_box: Option<SkewBox>,
}

impl Docker {
    const SPEED: f32 = 0.3;

    fn is_transient(&self) -> bool {
        self.x.is_transient() || self.y.is_transient() || self.skew_box.as_ref().map(|skew_box| {
            skew_box.x.is_transient() || skew_box.y.is_transient()
        }).unwrap_or(false)
    }

    fn animate(&mut self, elapsed: Duration) {
        self.x.animate(elapsed);
        self.y.animate(elapsed);
        if let Some(skew_box) = self.skew_box.as_mut() {
            skew_box.x.animate(elapsed);
            skew_box.y.animate(elapsed);
        }
    }

    fn update(&mut self, x: f32, y: f32) -> (f32, f32) {
        self.x.to(x);
        self.y.to(y);
        (*self.x, *self.y)
    }
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

enum Msg {
    Resize {
        width: f32,
        height: f32,
    },
    Draw(Duration),
    Scroll(f32),
    KeyDown(VirtualKeyCode),
    None,
}

struct Game {
    canvas: Canvas,
    level: Level,
    level_complete: bool,
    docker: Docker,
}

impl Game {
    fn is_transient(&self) -> bool {
        self.canvas.scale_factor.is_transient()
        || self.docker.is_transient()
    }

    fn animate(&mut self, elapsed: Duration) {
        self.canvas.scale_factor.animate(elapsed);
        self.docker.animate(elapsed);
    }

    fn field_transform(&self) -> Transform {
        let scale_factor = self.canvas.scale_factor.val();
        Transform::new()
            .with_scale(scale_factor, scale_factor)
            .with_translation(
                -(scale_factor * self.canvas.width - self.canvas.width) / 2.0,
                -(scale_factor * self.canvas.height - self.canvas.height) / 2.0,
            )
    }

    fn field_pos(&self) -> (f32, f32) {
        let field_x = (self.canvas.width - self.level.cols() as f32 * self.canvas.cell_size) / 2.0;
        let field_y = (self.canvas.height - self.level.rows() as f32 * self.canvas.cell_size) / 2.0;
        (field_x, field_y)
    }

    fn reset_docker(&mut self) {
        let (row, col) = self.level.docker_pos();
        let (field_x, field_y) = self.field_pos();
        let x = field_x + col as f32 * self.canvas.cell_size;
        let y = field_y + row as f32 * self.canvas.cell_size;
        self.docker = Docker {
            row,
            col,
            x: Animate::new(x, x, Docker::SPEED),
            y: Animate::new(y, y, Docker::SPEED),
            skew_box: None,
        };
    }

    fn move_docker(&mut self, dir: Direction) {
        let (to_row, to_col) = match dir {
            Direction::Left => (self.docker.row, self.docker.col - 1),
            Direction::Right => (self.docker.row, self.docker.col + 1),
            Direction::Up => (self.docker.row - 1, self.docker.col),
            Direction::Down => (self.docker.row + 1, self.docker.col),
        };

        if self.level.cell(to_row, to_col).map(|cell| cell.contains_box()).unwrap_or(false) {
            let (to_box_row, to_box_col) = match dir {
                Direction::Left => (to_row, to_col - 1),
                Direction::Right => (to_row, to_col + 1),
                Direction::Up => (to_row - 1, to_col),
                Direction::Down => (to_row + 1, to_col),
            };
            if self.level.go_box(to_row, to_col, to_box_row, to_box_col) {
                let (field_x, field_y) = self.field_pos();
                let x = field_x + to_col as f32 * self.canvas.cell_size;
                let y = field_y + to_row as f32 * self.canvas.cell_size;
                self.docker.skew_box = Some(SkewBox {
                    id: format!("box_{}_{}", to_row, to_col),
                    row: to_box_row,
                    col: to_box_col,
                    x: Animate::new(x, field_x + to_box_col as f32 * self.canvas.cell_size, Docker::SPEED),
                    y: Animate::new(y, field_y + to_box_row as f32 * self.canvas.cell_size, Docker::SPEED),
                });

                if self.level.is_complete() {
                    self.level_complete = true;
                }
            }
        }

        if self.level.go_docker(to_row, to_col) {
            self.docker.row = to_row;
            self.docker.col = to_col;
            let (field_x, field_y) = self.field_pos();
            let x = field_x + self.docker.col as f32 * self.canvas.cell_size;
            let y = field_y + self.docker.row as f32 * self.canvas.cell_size;
            self.docker.update(x, y);
        }
    }
}

impl Model for Game {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        let mut game = Self {
            canvas: Canvas::new(),
            level: Level::new(),
            level_complete: false,
            docker: Default::default(),
        };
        game.reset_docker();
        game
    }

    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        match msg {
            SystemMessage::WindowResized { width, height } => Some(Msg::Resize {
                width: width as f32,
                height: height as f32,
            }),
            SystemMessage::Draw(elapsed) => Some(Msg::Draw(elapsed)),
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::Resize { width, height } => {
                self.canvas.resize(width, height);
                self.reset_docker();
                ChangeView::Rebuild
            },
            Msg::Draw(elapsed) => {
                if self.is_transient() {
                    self.animate(elapsed);
                    ChangeView::Modify
                } else {
                    if self.level_complete {
                        self.level.next();
                        self.level_complete = false;
                        self.reset_docker();
                        ChangeView::Rebuild
                    } else {
                        ChangeView::None
                    }
                }
            }
            Msg::Scroll(delta) => {
                self.canvas.scale_factor.set((self.canvas.scale_factor.val() + delta * 0.1).max(0.0));
                ChangeView::Rebuild
            }
            Msg::KeyDown(code) => {
                match code {
                    VirtualKeyCode::Left if !self.docker.is_transient() => self.move_docker(Direction::Left),
                    VirtualKeyCode::Right if !self.docker.is_transient() => self.move_docker(Direction::Right),
                    VirtualKeyCode::Up if !self.docker.is_transient() => self.move_docker(Direction::Up),
                    VirtualKeyCode::Down if !self.docker.is_transient() => self.move_docker(Direction::Down),
                    _ => (),
                };
                ChangeView::None
            }
            _ => ChangeView::None,
        }
    }

    fn build_view(&self) -> Node<Self> {
        let mut cells = vec![];
        let mut docker = None;
        let mut boxes = vec![];
        let (field_x, field_y) = self.field_pos();

        for row in 0..self.level.rows() {
            for col in 0..self.level.cols() {
                let x = field_x + col as f32 * self.canvas.cell_size;
                let y = field_y + row as f32 * self.canvas.cell_size;
                match self.level.cell(row, col).expect("Cell expected") {
                    Cell::Wall => cells.push(self.build_wall(x, y)),
                    Cell::Box => boxes.push(self.build_box(row, col, x, y)),
                    Cell::BoxOnPlace => {
                        cells.push(self.build_place(x, y));
                        boxes.push(self.build_box(row, col, x, y));
                    },
                    Cell::Docker => docker = Some(self.build_docker(x, y)),
                    Cell::DockerOnPlace => {
                        cells.push(self.build_place(x, y));
                        docker = Some(self.build_docker(x, y));
                    },
                    Cell::Place => cells.push(self.build_place(x, y)),
                    _ => (),
                }
            }
        }
        for cell_box in boxes {
            cells.push(cell_box);
        }
        if let Some(docker) = docker {
            cells.push(docker);
        }

        rect()
            .width(Pct(100))
            .height(Pct(100))
            .fill(Color::RGB(0.8, 0.9, 1.0))
            .on_mouse_scroll(|case| Msg::Scroll(case.event.delta.1))
            .child(group()
                .id("field")
                .transform(self.field_transform())
                .children(cells)
                .on_key_down(|case| if let Some(code) = case.event.keycode {
                    Msg::KeyDown(code)
                } else {
                    Msg::None
                })
            )
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        if let Some(field) = view.get_prim_mut("field") {
            *field.transform_mut() = self.field_transform();
        }
        if let Some(docker) = view.get_prim_mut("docker") {
            *docker.transform_mut() = translate(*self.docker.x, *self.docker.y);
            if let Some(skew_box) = &mut self.docker.skew_box {
                if let Some(cell_box) = view.get_prim_mut(&skew_box.id) {
                    *cell_box.transform_mut() = translate(*skew_box.x, *skew_box.y);
                    skew_box.id = format!("box_{}_{}", skew_box.row, skew_box.col);
                    if cell_box.id().map(|id| id != skew_box.id).unwrap_or(true) {
                        cell_box.set_id(&skew_box.id);
                    }
                }
            }
        }
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

    fn build_box(&self, row: usize, col: usize, x: f32, y: f32) -> Node<Self> {
        let board_color = Color::RGB(1.0, 0.7, 0.1);
        let board_space = self.canvas.cell_size / 15.0;
        let board_space_half = board_space / 2.0;
        let board_chunk_size = (self.canvas.cell_size - board_space * 3.0) / 3.0;
        let round_radius = 1.0;

        rect()
            .id(format!("box_{}_{}", row, col))
            .width(self.canvas.cell_size)
            .height(self.canvas.cell_size)
            .transparency(1.0)
            .transform(translate(x, y))
            .child(rect()
                .width(self.canvas.cell_size - board_space)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_space_half, board_space_half))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_space_half, board_space_half + board_chunk_size + board_space))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_space_half + board_chunk_size + board_space, board_space_half + board_chunk_size + board_space))
            )
            .child(rect()
                .width(board_chunk_size)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_space_half + (board_chunk_size + board_space) * 2.0, board_space_half + board_chunk_size + board_space))
            )
            .child(rect()
                .width(self.canvas.cell_size - board_space)
                .height(board_chunk_size)
                .fill(board_color)
                .rounding(round_radius)
                .transform(translate(board_space_half, board_space_half + board_chunk_size * 2.0 + board_space * 2.0))
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
                    Move([docker_brush_size * 1.8, self.canvas.cell_size - docker_brush_size * 1.2]),
                    BezCtrl([self.canvas.cell_size / 2.0, head_radius * 2.0]),
                    QuadBezTo([self.canvas.cell_size - docker_brush_size * 1.8, self.canvas.cell_size - docker_brush_size * 1.2]),
                    LineAlonX(docker_brush_size * 1.8)
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
        let place_color = Color::RGB(0.2, 0.6, 1.0);
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
            .with_double_buffer(Some(true))
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