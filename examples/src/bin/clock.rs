use std::f32::consts::PI;
use std::env;

use chrono::{Datelike, DateTime, Local, Timelike};

use exgui_render_nanovg::NanovgRender;
use exgui_controller_glutin::{App, glutin};
use exgui::{
    AlignHor::*,
    AlignVer::*, builder::*, ChangeView, Color, Comp, Gradient, Model, Node,
    PathCommand::*, SystemMessage
};

const INIT_WINDOW_SIZE: (u32, u32) = (480, 480);
const TWO_PI: f32 = 2.0 * PI;

#[derive(Debug, Default)]
struct Clock {
    clock_size: i32,
    dial_radius: f32,
    dial_center: (f32, f32),

    am: bool,
    hour: f32,
    minute: f32,
    second: f32,

    year: i32,
    month: u32,
    day: u32,
    day_changed: bool,

    hour_angle: f32,
    minute_angle: f32,
    second_angle: f32,
}

#[derive(Clone)]
pub enum Msg {
    ResizeWindow((u32, u32)),
    Tick,
}

impl Model for Clock {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties) -> Self {
        let (width, height) = INIT_WINDOW_SIZE;
        let mut clock = Clock::default();
        clock.size_recalc(width, height);
        clock
    }

    fn system_update(&mut self, msg: SystemMessage) -> Option<Self::Message> {
        match msg {
            SystemMessage::Draw(_) => Some(Msg::Tick),
            SystemMessage::WindowResized { width, height } => Some(Msg::ResizeWindow((width, height))),
            _ => None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            Msg::ResizeWindow((w, h)) => {
                self.size_recalc(w, h)
            },
            Msg::Tick => {
                let dt: DateTime<Local> = Local::now(); // e.g. `2018-11-28T21:45:59.324310806+09:00`

                let prev_second = self.second;
                self.second = f64::from(dt.second()) as f32;

                if (self.second - prev_second).abs() >= 1. {
                    let hour = dt.hour();

                    self.am = hour < 12;
                    self.hour = f64::from(hour % 12) as f32;
                    self.minute = f64::from(dt.minute()) as f32;


                    self.year = dt.year();
                    self.month = dt.month();

                    let day = dt.day();
                    if self.day == day {
                        self.day_changed = false;
                    } else {
                        self.day = day;
                        self.day_changed = true;
                    }

                    let radians_per_sec = TWO_PI / 60.0;

                    self.hour_angle = (((self.hour * 60.0 + self.minute) / 60.0) / 12.0) * TWO_PI;
                    self.minute_angle = self.minute * radians_per_sec;
                    self.second_angle = self.second * radians_per_sec;

                    ChangeView::Modify
                } else {
                    ChangeView::None
                }
            },
        }
    }

    fn build_view(&self) -> Node<Self> {
        let second_hand_len = self.dial_radius * 0.9;
        let second_hand_props = HandProperties {
            length: second_hand_len,
            width: 1.0,
            theta: self.second_angle,
        };
        let minute_hand_props = HandProperties {
            length: self.dial_radius * 0.8,
            width: 3.0,
            theta: self.minute_angle,
        };
        let hour_hand_props = HandProperties {
            length: self.dial_radius * 0.6,
            width: 5.0,
            theta: self.hour_angle,
        };

        let silver = Color::RGB(196.0 / 255.0,199.0 / 255.0,206.0 / 255.0);
        let darksilver = Color::RGB(148.0 / 255.0, 152.0 / 255.0, 161.0 / 255.0);
        let darkgray = Color::RGB(169.0 / 255.0, 169.0 / 255.0, 169.0 / 255.0);
        let boss_rad = 6.0_f32;

        let mut set = vec![];

        let dial = circle()
            .center(0, 0)
            .radius(self.dial_radius)
            .stroke((silver, 3))
            .fill(Color::RGB(0.2, 0.0, 0.8))
            .build();
        set.push(dial);

        for n in 1..=12 {
            let min_marker = self.build_num(n, second_hand_len, 24.0);
            set.push(min_marker);
        }

        for m in 1..=60 {
            if m % 5 != 0 {
                let tick_marker = self.build_tick(m as f32, 3.0, 1.0);
                set.push(tick_marker);
            }
        }

        let date = text(format!("{:4}-{:02}-{:02}", self.year, self.month, self.day))
            .id("date")
            .pos(0, self.dial_radius * 0.7)
            .font_name("Roboto")
            .font_size(24)
            .align((Center, Baseline))
            .fill(silver)
            .build();
        set.push(date);

        let second_hand = comp(Hand::create(second_hand_props))
            .id("second hand")
            .build();
        set.push(second_hand);

        let minute_hand = comp(Hand::create(minute_hand_props))
            .id("minute hand")
            .build();
        set.push(minute_hand);

        let hour_hand = comp(Hand::create(hour_hand_props))
            .id("hour hand")
            .build();
        set.push(hour_hand);

        let boss = circle()
            .center(0, 0)
            .radius(boss_rad)
            .stroke(darkgray)
            .fill(Gradient::Radial {
                center: (0.0, 0.0),
                inner_radius: 0.0,
                outer_radius: boss_rad,
                start_color: silver,
                end_color: darksilver,
            })
            .build();
        set.push(boss);

        group()
            .transform(translate(self.dial_center.0, self.dial_center.1))
            .children(set)
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        if self.day_changed {
            view.get_prim_mut("date")
                .map(|prim| prim.set_text(format!(
                    "{:4}-{:02}-{:02}", self.year, self.month, self.day
                )));
        }

        view.get_comp_mut("second hand")
            .map(|hand| {
                let hand_theta = hand.model::<Hand>().theta;
                if (hand_theta - self.second_angle).abs() > 0.00001 {
                    hand.send::<Hand>(HandMsg::ChangeTheta(self.second_angle));
                }
            });

        view.get_comp_mut("minute hand")
            .map(|hand| {
                let hand_theta = hand.model::<Hand>().theta;
                if (hand_theta - self.minute_angle).abs() > 0.00001 {
                    hand.send::<Hand>(HandMsg::ChangeTheta(self.minute_angle));
                }
            });

        view.get_comp_mut("hour hand")
            .map(|hand| {
                let hand_theta = hand.model::<Hand>().theta;
                if (hand_theta - self.hour_angle).abs() > 0.00001 {
                    hand.send::<Hand>(HandMsg::ChangeTheta(self.hour_angle));
                }
            });
    }
}

impl Clock {
    fn size_recalc(&mut self, width: u32, height: u32) -> ChangeView {
        let clock_size = width.min(height) as i32 - 2;
        let dial_center = ((width as f64 / 2.0) as f32, (height as f64 / 2.0) as f32);
        if self.clock_size != clock_size || self.dial_center != dial_center {
            self.clock_size = clock_size;
            self.dial_center = dial_center;
            self.dial_radius = (self.clock_size as f64 / 2.0) as f32;
            ChangeView::Rebuild
        } else {
            ChangeView::None
        }
    }

    fn build_num(&self, n: i32, len: f32, font_size: f32) -> Node<Clock> {
        let radians_per_hour = TWO_PI / 12.0;
        let x = len * (n as f32 * radians_per_hour).sin();
        let y = - len * (n as f32 * radians_per_hour).cos();
        let silver = Color::RGB(196.0 / 255.0,199.0 / 255.0,206.0 / 255.0);

        text(format!("{}", n))
            .pos(x, y)
            .font_name("Roboto")
            .font_size(font_size)
            .align((Center, Middle))
            .fill(silver)
            .build()
    }

    fn build_tick(&self, m: f32, len: f32, width: f32) -> Node<Clock> {
        let radians_per_sec = TWO_PI / 60.0;
        let ticks_radius = self.dial_radius * 0.925;

        path(vec![Move([0.0, -ticks_radius]), Line([0.0, -ticks_radius - len]), Close])
            .fill(Color::White)
            .stroke((Color::White, width))
            .transform(rotate(m * radians_per_sec))
            .build()
    }
}

struct Hand {
    props: HandProperties,
    theta: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
struct HandProperties {
    length: f32,
    width: f32,
    theta: f32,
}

#[derive(Clone)]
enum HandMsg {
    ChangeTheta(f32),
}

impl Model for Hand {
    type Message = HandMsg;
    type Properties = HandProperties;

    fn create(props: Self::Properties) -> Self {
        Hand {
            props,
            theta: props.theta,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ChangeView {
        match msg {
            HandMsg::ChangeTheta(theta) => {
                self.theta = theta;
                ChangeView::Modify
            }
        }
    }

    fn build_view(&self) -> Node<Hand> {
        path(vec![Move([0.0, 0.0]), Line([0.0, -self.props.length]), Close])
            .fill(Color::White)
            .stroke((Color::White, self.props.width))
            .transform(rotate(self.theta))
            .build()
    }

    fn modify_view(&mut self, view: &mut Node<Self>) {
        view.transform_mut().rotate(self.theta);
    }
}

fn main() {
    let mut app = App::new(
        glutin::window::WindowBuilder::new()
            .with_title("ExGUI clock")
            .with_inner_size(glutin::dpi::PhysicalSize::new(INIT_WINDOW_SIZE.0, INIT_WINDOW_SIZE.1)),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8)
            .with_srgb(true),
        NanovgRender::default()
    ).unwrap();
    app.init().unwrap();

    let font_path = env::current_dir().unwrap().join("examples").join("resources").join("Roboto-Regular.ttf");
    app.renderer_mut().load_font("Roboto", font_path).unwrap();

    let comp = Comp::new(Clock::create(()));
    app.run(comp);
}
