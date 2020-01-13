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
use exgui_core::{Comp, Color, MouseInput, SystemMessage, Render};

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
        let mut mouse_controller = MouseInput::new();
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
                    WindowEvent::CursorMoved { position, .. } => {
                        mouse_controller.update_pos(position.x as f32, position.y as f32);
                    },
                    WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                        mouse_controller.left_pressed_comp(&mut comp);
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