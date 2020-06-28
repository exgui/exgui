use std::time::Instant;

pub use gl;
pub use glutin;
use glutin::{
    event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, MouseButton},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder, PossiblyCurrent, NotCurrent, WindowedContext,
    CreationError, ContextError,
};
use exgui_core::{Comp, Color, MouseController, KeyboardController, controller, SystemMessage, Render};

pub enum AppState {
    Exit,
    Continue,
}

#[derive(Debug)]
pub enum AppContext {
    NotCurrent(Option<WindowedContext<NotCurrent>>),
    PossiblyCurrent(Option<WindowedContext<PossiblyCurrent>>),
}

impl AppContext {
    pub fn take_not_current(&mut self) -> Option<WindowedContext<NotCurrent>> {
        match self {
            AppContext::NotCurrent(context) => context.take(),
            _ => None,
        }
    }

    pub fn take_current(&mut self) -> Option<WindowedContext<PossiblyCurrent>> {
        match self {
            AppContext::PossiblyCurrent(context) => context.take(),
            _ => None,
        }
    }

    pub fn current(&self) -> Option<&WindowedContext<PossiblyCurrent>> {
        match self {
            AppContext::PossiblyCurrent(context) => context.as_ref(),
            _ => None,
        }
    }
}

pub struct App<R: Render> {
    event_loop: EventLoop<()>,
    context: AppContext,
    render: R,
    background_color: Color,
    exit_by_escape: bool,
}

#[derive(Debug)]
pub enum AppError<RE> {
    CreationError(CreationError),
    ContextError(ContextError),
    PossiblyCurrentContextNotExist,
    RendererError(RE),
    WindowNoLongerExists,
    EventsLoopIsNone,
}

impl<RE> From<CreationError> for AppError<RE> {
    fn from(from: CreationError) -> Self {
        AppError::CreationError(from)
    }
}

impl<RE> From<ContextError> for AppError<RE> {
    fn from(from: ContextError) -> Self {
        AppError::ContextError(from)
    }
}

impl<R: Render + 'static> App<R> {
    pub fn new(
        window_builder: WindowBuilder,
        context_builder: ContextBuilder<NotCurrent>,
        render: R,
    ) -> Result<Self, AppError<R::Error>> {
        let event_loop = EventLoop::new();
        let context = AppContext::NotCurrent(Some(context_builder.build_windowed(window_builder, &event_loop)?));
        Ok(App {
            event_loop,
            context,
            render,
            background_color: Color::RGBA(0.8, 0.8, 0.8, 1.0),
            exit_by_escape: true,
        })
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    pub fn with_exit_by_escape(mut self, exit: bool) -> Self {
        self.exit_by_escape = exit;
        self
    }

    pub fn init(&mut self) -> Result<&mut Self, AppError<R::Error>> {
        if let Some(context) = self.context.take_not_current() {
            let context = unsafe { context.make_current().map_err(|(_, err)| err)? };
            self.context = AppContext::PossiblyCurrent(Some(context));
        }
        let context = self.context.current().ok_or(AppError::PossiblyCurrentContextNotExist)?;

        unsafe {
            gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
            let color = self.background_color.as_arr();
            gl::ClearColor(color[0], color[1], color[2], color[3]);
        }
        self.render.init().map_err(|err| AppError::RendererError(err))?;
        Ok(self)
    }

    #[inline]
    pub fn run(self, comp: Comp) -> ! {
        self.run_proc(comp, |_, _, _| AppState::Continue)
    }

    pub fn run_proc(
        self,
        mut comp: Comp,
        mut proc: impl FnMut(&mut Comp, &WindowedContext<PossiblyCurrent>, &mut R) -> AppState + 'static
    ) -> ! {
        let App { event_loop, mut context, mut render, exit_by_escape, .. } = self;
        let mut mouse_controller = MouseController::new();
        let keyboard_controller = KeyboardController::new();
        let context = context.take_current().expect("PossiblyCurrent context does not exist");//ok_or(AppError::PossiblyCurrentContextNotExist)?;
        let mut last_time = Instant::now();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        context.resize(size);
                        comp.send_system_msg(SystemMessage::WindowResized { width: size.width, height: size.height });
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    WindowEvent::ReceivedCharacter(ch) => {
                        keyboard_controller.input_char(&mut comp, ch);
                    }
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                        ..
                    } if exit_by_escape => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                    WindowEvent::KeyboardInput { input, ..} => {
                        let KeyboardInput { scancode, state, virtual_keycode, .. } = input;
                        if let ElementState::Pressed = state {
                            keyboard_controller.pressed_comp(&mut comp, convert_keyboard_event(scancode, virtual_keycode));
                        } else {
                            keyboard_controller.released_comp(&mut comp, convert_keyboard_event(scancode, virtual_keycode));
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_controller.update_pos(position.x as f32, position.y as f32);
                    },
                    WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                        mouse_controller.pressed_comp(&mut comp, convert_mouse_button(button));
                    },
                    _ => (),
                },
                Event::MainEventsCleared => {
                    context.window().request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let size = context.window().inner_size();
                    unsafe {
                        gl::Viewport(0, 0, size.width as i32, size.height as i32);
                        gl::Clear(
                            gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT,
                        );
                    }

                    if let AppState::Exit = proc(&mut comp, &context, &mut render) {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    let elapsed = last_time.elapsed();
                    last_time = Instant::now();
                    comp.send_system_msg(SystemMessage::Draw(elapsed));
                    comp.update_view();

                    render.set_dimensions(size.width, size.height, context.window().scale_factor());
                    render.render(&mut comp).unwrap_or_else(|err| panic!("Renderer error: {:?}", err));

                    context.swap_buffers().expect("Swap buffers fail");
                }
                _ => (),
            }
        })
    }

    pub fn context(&self) -> &AppContext {
        &self.context
    }

    pub fn render(&self) -> &R {
        &self.render
    }

    pub fn render_mut(&mut self) -> &mut R {
        &mut self.render
    }
}

fn convert_keyboard_event(scancode: u32, keycode: Option<VirtualKeyCode>) -> controller::KeyboardEvent {
    let keycode = keycode.map(|code| match code {
        VirtualKeyCode::Key1 => controller::VirtualKeyCode::Key1,
        VirtualKeyCode::Key2 => controller::VirtualKeyCode::Key2,
        VirtualKeyCode::Key3 => controller::VirtualKeyCode::Key3,
        VirtualKeyCode::Key4 => controller::VirtualKeyCode::Key4,
        VirtualKeyCode::Key5 => controller::VirtualKeyCode::Key5,
        VirtualKeyCode::Key6 => controller::VirtualKeyCode::Key6,
        VirtualKeyCode::Key7 => controller::VirtualKeyCode::Key7,
        VirtualKeyCode::Key8 => controller::VirtualKeyCode::Key8,
        VirtualKeyCode::Key9 => controller::VirtualKeyCode::Key9,
        VirtualKeyCode::Key0 => controller::VirtualKeyCode::Key0,
        VirtualKeyCode::A => controller::VirtualKeyCode::A,
        VirtualKeyCode::B => controller::VirtualKeyCode::B,
        VirtualKeyCode::C => controller::VirtualKeyCode::C,
        VirtualKeyCode::D => controller::VirtualKeyCode::D,
        VirtualKeyCode::E => controller::VirtualKeyCode::E,
        VirtualKeyCode::F => controller::VirtualKeyCode::F,
        VirtualKeyCode::G => controller::VirtualKeyCode::G,
        VirtualKeyCode::H => controller::VirtualKeyCode::H,
        VirtualKeyCode::I => controller::VirtualKeyCode::I,
        VirtualKeyCode::J => controller::VirtualKeyCode::J,
        VirtualKeyCode::K => controller::VirtualKeyCode::K,
        VirtualKeyCode::L => controller::VirtualKeyCode::L,
        VirtualKeyCode::M => controller::VirtualKeyCode::M,
        VirtualKeyCode::N => controller::VirtualKeyCode::N,
        VirtualKeyCode::O => controller::VirtualKeyCode::O,
        VirtualKeyCode::P => controller::VirtualKeyCode::P,
        VirtualKeyCode::Q => controller::VirtualKeyCode::Q,
        VirtualKeyCode::R => controller::VirtualKeyCode::R,
        VirtualKeyCode::S => controller::VirtualKeyCode::S,
        VirtualKeyCode::T => controller::VirtualKeyCode::T,
        VirtualKeyCode::U => controller::VirtualKeyCode::U,
        VirtualKeyCode::V => controller::VirtualKeyCode::V,
        VirtualKeyCode::W => controller::VirtualKeyCode::W,
        VirtualKeyCode::X => controller::VirtualKeyCode::X,
        VirtualKeyCode::Y => controller::VirtualKeyCode::Y,
        VirtualKeyCode::Z => controller::VirtualKeyCode::Z,
        VirtualKeyCode::Escape => controller::VirtualKeyCode::Escape,
        VirtualKeyCode::F1 => controller::VirtualKeyCode::F1,
        VirtualKeyCode::F2 => controller::VirtualKeyCode::F2,
        VirtualKeyCode::F3 => controller::VirtualKeyCode::F3,
        VirtualKeyCode::F4 => controller::VirtualKeyCode::F4,
        VirtualKeyCode::F5 => controller::VirtualKeyCode::F5,
        VirtualKeyCode::F6 => controller::VirtualKeyCode::F6,
        VirtualKeyCode::F7 => controller::VirtualKeyCode::F7,
        VirtualKeyCode::F8 => controller::VirtualKeyCode::F8,
        VirtualKeyCode::F9 => controller::VirtualKeyCode::F9,
        VirtualKeyCode::F10 => controller::VirtualKeyCode::F10,
        VirtualKeyCode::F11 => controller::VirtualKeyCode::F11,
        VirtualKeyCode::F12 => controller::VirtualKeyCode::F12,
        VirtualKeyCode::F13 => controller::VirtualKeyCode::F13,
        VirtualKeyCode::F14 => controller::VirtualKeyCode::F14,
        VirtualKeyCode::F15 => controller::VirtualKeyCode::F15,
        VirtualKeyCode::F16 => controller::VirtualKeyCode::F16,
        VirtualKeyCode::F17 => controller::VirtualKeyCode::F17,
        VirtualKeyCode::F18 => controller::VirtualKeyCode::F18,
        VirtualKeyCode::F19 => controller::VirtualKeyCode::F19,
        VirtualKeyCode::F20 => controller::VirtualKeyCode::F20,
        VirtualKeyCode::F21 => controller::VirtualKeyCode::F21,
        VirtualKeyCode::F22 => controller::VirtualKeyCode::F22,
        VirtualKeyCode::F23 => controller::VirtualKeyCode::F23,
        VirtualKeyCode::F24 => controller::VirtualKeyCode::F24,
        VirtualKeyCode::Snapshot => controller::VirtualKeyCode::Snapshot,
        VirtualKeyCode::Scroll => controller::VirtualKeyCode::Scroll,
        VirtualKeyCode::Pause => controller::VirtualKeyCode::Pause,
        VirtualKeyCode::Insert => controller::VirtualKeyCode::Insert,
        VirtualKeyCode::Home => controller::VirtualKeyCode::Home,
        VirtualKeyCode::Delete => controller::VirtualKeyCode::Delete,
        VirtualKeyCode::End => controller::VirtualKeyCode::End,
        VirtualKeyCode::PageDown => controller::VirtualKeyCode::PageDown,
        VirtualKeyCode::PageUp => controller::VirtualKeyCode::PageUp,
        VirtualKeyCode::Left => controller::VirtualKeyCode::Left,
        VirtualKeyCode::Up => controller::VirtualKeyCode::Up,
        VirtualKeyCode::Right => controller::VirtualKeyCode::Right,
        VirtualKeyCode::Down => controller::VirtualKeyCode::Down,
        VirtualKeyCode::Back => controller::VirtualKeyCode::Backspace,
        VirtualKeyCode::Return => controller::VirtualKeyCode::Enter,
        VirtualKeyCode::Space => controller::VirtualKeyCode::Space,
        VirtualKeyCode::Compose => controller::VirtualKeyCode::Compose,
        VirtualKeyCode::Caret => controller::VirtualKeyCode::Caret,
        VirtualKeyCode::Numlock => controller::VirtualKeyCode::Numlock,
        VirtualKeyCode::Numpad0 => controller::VirtualKeyCode::Numpad0,
        VirtualKeyCode::Numpad1 => controller::VirtualKeyCode::Numpad1,
        VirtualKeyCode::Numpad2 => controller::VirtualKeyCode::Numpad2,
        VirtualKeyCode::Numpad3 => controller::VirtualKeyCode::Numpad3,
        VirtualKeyCode::Numpad4 => controller::VirtualKeyCode::Numpad4,
        VirtualKeyCode::Numpad5 => controller::VirtualKeyCode::Numpad5,
        VirtualKeyCode::Numpad6 => controller::VirtualKeyCode::Numpad6,
        VirtualKeyCode::Numpad7 => controller::VirtualKeyCode::Numpad7,
        VirtualKeyCode::Numpad8 => controller::VirtualKeyCode::Numpad8,
        VirtualKeyCode::Numpad9 => controller::VirtualKeyCode::Numpad9,
        VirtualKeyCode::AbntC1 => controller::VirtualKeyCode::AbntC1,
        VirtualKeyCode::AbntC2 => controller::VirtualKeyCode::AbntC2,
        VirtualKeyCode::Add => controller::VirtualKeyCode::Add,
        VirtualKeyCode::Apostrophe => controller::VirtualKeyCode::Apostrophe,
        VirtualKeyCode::Apps => controller::VirtualKeyCode::Apps,
        VirtualKeyCode::At => controller::VirtualKeyCode::At,
        VirtualKeyCode::Ax => controller::VirtualKeyCode::Ax,
        VirtualKeyCode::Backslash => controller::VirtualKeyCode::Backslash,
        VirtualKeyCode::Calculator => controller::VirtualKeyCode::Calculator,
        VirtualKeyCode::Capital => controller::VirtualKeyCode::Capital,
        VirtualKeyCode::Colon => controller::VirtualKeyCode::Colon,
        VirtualKeyCode::Comma => controller::VirtualKeyCode::Comma,
        VirtualKeyCode::Convert => controller::VirtualKeyCode::Convert,
        VirtualKeyCode::Decimal => controller::VirtualKeyCode::Decimal,
        VirtualKeyCode::Divide => controller::VirtualKeyCode::Divide,
        VirtualKeyCode::Equals => controller::VirtualKeyCode::Equals,
        VirtualKeyCode::Grave => controller::VirtualKeyCode::Grave,
        VirtualKeyCode::Kana => controller::VirtualKeyCode::Kana,
        VirtualKeyCode::Kanji => controller::VirtualKeyCode::Kanji,
        VirtualKeyCode::LAlt => controller::VirtualKeyCode::LAlt,
        VirtualKeyCode::LBracket => controller::VirtualKeyCode::LBracket,
        VirtualKeyCode::LControl => controller::VirtualKeyCode::LControl,
        VirtualKeyCode::LShift => controller::VirtualKeyCode::LShift,
        VirtualKeyCode::LWin => controller::VirtualKeyCode::LWin,
        VirtualKeyCode::Mail => controller::VirtualKeyCode::Mail,
        VirtualKeyCode::MediaSelect => controller::VirtualKeyCode::MediaSelect,
        VirtualKeyCode::MediaStop => controller::VirtualKeyCode::MediaStop,
        VirtualKeyCode::Minus => controller::VirtualKeyCode::Minus,
        VirtualKeyCode::Multiply => controller::VirtualKeyCode::Multiply,
        VirtualKeyCode::Mute => controller::VirtualKeyCode::Mute,
        VirtualKeyCode::MyComputer => controller::VirtualKeyCode::MyComputer,
        VirtualKeyCode::NavigateForward => controller::VirtualKeyCode::NavigateForward,
        VirtualKeyCode::NavigateBackward => controller::VirtualKeyCode::NavigateBackward,
        VirtualKeyCode::NextTrack => controller::VirtualKeyCode::NextTrack,
        VirtualKeyCode::NoConvert => controller::VirtualKeyCode::NoConvert,
        VirtualKeyCode::NumpadComma => controller::VirtualKeyCode::NumpadComma,
        VirtualKeyCode::NumpadEnter => controller::VirtualKeyCode::NumpadEnter,
        VirtualKeyCode::NumpadEquals => controller::VirtualKeyCode::NumpadEquals,
        VirtualKeyCode::OEM102 => controller::VirtualKeyCode::OEM102,
        VirtualKeyCode::Period => controller::VirtualKeyCode::Period,
        VirtualKeyCode::PlayPause => controller::VirtualKeyCode::PlayPause,
        VirtualKeyCode::Power => controller::VirtualKeyCode::Power,
        VirtualKeyCode::PrevTrack => controller::VirtualKeyCode::PrevTrack,
        VirtualKeyCode::RAlt => controller::VirtualKeyCode::RAlt,
        VirtualKeyCode::RBracket => controller::VirtualKeyCode::RBracket,
        VirtualKeyCode::RControl => controller::VirtualKeyCode::RControl,
        VirtualKeyCode::RShift => controller::VirtualKeyCode::RShift,
        VirtualKeyCode::RWin => controller::VirtualKeyCode::RWin,
        VirtualKeyCode::Semicolon => controller::VirtualKeyCode::Semicolon,
        VirtualKeyCode::Slash => controller::VirtualKeyCode::Slash,
        VirtualKeyCode::Sleep => controller::VirtualKeyCode::Sleep,
        VirtualKeyCode::Stop => controller::VirtualKeyCode::Stop,
        VirtualKeyCode::Subtract => controller::VirtualKeyCode::Subtract,
        VirtualKeyCode::Sysrq => controller::VirtualKeyCode::Sysrq,
        VirtualKeyCode::Tab => controller::VirtualKeyCode::Tab,
        VirtualKeyCode::Underline => controller::VirtualKeyCode::Underline,
        VirtualKeyCode::Unlabeled => controller::VirtualKeyCode::Unlabeled,
        VirtualKeyCode::VolumeDown => controller::VirtualKeyCode::VolumeDown,
        VirtualKeyCode::VolumeUp => controller::VirtualKeyCode::VolumeUp,
        VirtualKeyCode::Wake => controller::VirtualKeyCode::Wake,
        VirtualKeyCode::WebBack => controller::VirtualKeyCode::WebBack,
        VirtualKeyCode::WebFavorites => controller::VirtualKeyCode::WebFavorites,
        VirtualKeyCode::WebForward => controller::VirtualKeyCode::WebForward,
        VirtualKeyCode::WebHome => controller::VirtualKeyCode::WebHome,
        VirtualKeyCode::WebRefresh => controller::VirtualKeyCode::WebRefresh,
        VirtualKeyCode::WebSearch => controller::VirtualKeyCode::WebSearch,
        VirtualKeyCode::WebStop => controller::VirtualKeyCode::WebStop,
        VirtualKeyCode::Yen => controller::VirtualKeyCode::Yen,
        VirtualKeyCode::Copy => controller::VirtualKeyCode::Copy,
        VirtualKeyCode::Paste => controller::VirtualKeyCode::Paste,
        VirtualKeyCode::Cut => controller::VirtualKeyCode::Cut,
    });
    controller::KeyboardEvent {
        scancode,
        keycode,
    }
}

fn convert_mouse_button(button: MouseButton) -> controller::MouseButton {
    match button {
        MouseButton::Left => controller::MouseButton::Left,
        MouseButton::Right => controller::MouseButton::Right,
        MouseButton::Middle => controller::MouseButton::Middle,
        MouseButton::Other(code) => controller::MouseButton::Other(code),
    }
}