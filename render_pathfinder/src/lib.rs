use std::{
    fmt,
    fs::File,
    io::{self, Read},
    iter,
    ops::Mul,
    path::Path,
    sync::Arc,
};

use exgui_core::{
    AlignHor, AlignVer, Clip, Color, CompositeShape, Fill, GlyphPos, Gradient, LineCap, LineJoin, Padding, Paint, Real,
    Render, Rounding, Shape, Stroke, Text, TextMetrics, Transform, TransformMatrix,
};
use font_kit::handle::Handle;
use pathfinder_canvas::{
    vec2f, vec2i, Canvas, CanvasFontContext, CanvasRenderingContext2D, ColorF, FillRule, FillStyle,
    LineCap as PathfinderLineCap, LineJoin as PathfinderLineJoin, Path2D, RectF, TextAlign, TextBaseline, Transform2F,
    Vector2F, Vector2I,
};
use pathfinder_content::gradient::Gradient as PathfinderGradient;
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_renderer::{
    concurrent::{rayon::RayonExecutor, scene_proxy::SceneProxy},
    gpu::{
        options::{DestFramebuffer, RendererOptions},
        renderer::Renderer,
    },
    options::BuildOptions,
};
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use pathfinder_simd::default::F32x2;
use skribo::TextStyle;

const PI_2: f32 = std::f32::consts::PI * 2.0;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min_x: Real,
    pub min_y: Real,
    pub max_x: Real,
    pub max_y: Real,
}

impl BoundingBox {
    pub fn width(&self) -> Real {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> Real {
        self.max_y - self.min_y
    }
}

impl Mul<BoundingBox> for TransformMatrix {
    type Output = [(Real, Real); 4];

    fn mul(self, rhs: BoundingBox) -> Self::Output {
        [
            self * (rhs.min_x, rhs.min_y),
            self * (rhs.min_x, rhs.max_y),
            self * (rhs.max_x, rhs.min_y),
            self * (rhs.max_x, rhs.max_y),
        ]
    }
}

#[derive(Debug)]
pub enum PathfinderRenderError {
    ContextIsNotInit,
    CreateFontError(io::Error, String),
}

pub struct RendererContext {
    pub renderer: Renderer<GLDevice>,
    pub font_context: CanvasFontContext,
    pub font_handles: Vec<Handle>,
}

impl fmt::Debug for RendererContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(RendererContext))
            .field("renderer", &"...")
            .field("font_context", &"...")
            .field("font_handles", &self.font_handles)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct PathfinderRender {
    pub context: Option<RendererContext>,
    pub width: u32,
    pub height: u32,
    pub framebuffer_size: Vector2I,
}

impl Render for PathfinderRender {
    type Error = PathfinderRenderError;

    fn init(&mut self, background_color: Color) -> Result<(), Self::Error> {
        if self.context.is_none() {
            let colors = background_color.as_arr();
            assert_ne!(self.framebuffer_size, Vector2I::zero());
            let renderer = Renderer::new(
                GLDevice::new(GLVersion::GL3, 0),
                &EmbeddedResourceLoader::new(),
                DestFramebuffer::full_window(self.framebuffer_size),
                RendererOptions {
                    background_color: Some(ColorF::new(colors[0], colors[1], colors[2], colors[3])),
                },
            );
            let font_context = CanvasFontContext::from_system_source();
            self.context = Some(RendererContext {
                renderer,
                font_context,
                font_handles: vec![],
            });
        }
        Ok(())
    }

    fn set_dimensions(&mut self, physical_width: u32, physical_height: u32, _device_pixel_ratio: f64) {
        if self.width != physical_width || self.height != physical_height {
            let framebuffer_size = vec2i(physical_width as i32, physical_height as i32);
            self.width = physical_width;
            self.height = physical_height;
            self.framebuffer_size = framebuffer_size;
            if let Some(context) = &mut self.context {
                context
                    .renderer
                    .replace_dest_framebuffer(DestFramebuffer::full_window(framebuffer_size));
            }
        }
    }

    fn render(&mut self, node: &mut dyn CompositeShape) -> Result<bool, Self::Error> {
        let renderer_context = self.context.as_mut().ok_or(PathfinderRenderError::ContextIsNotInit)?;
        let mut canvas_context =
            Canvas::new(self.framebuffer_size.to_f32()).get_context_2d(renderer_context.font_context.clone());

        let bound = BoundingBox {
            min_x: 0.0,
            min_y: 0.0,
            max_x: self.width as Real,
            max_y: self.height as Real,
        };

        // Recalculate tree data and fill canvas
        if node.need_recalc().unwrap_or(true) {
            let mut defaults = ShapeDefaults::default();
            Self::recalc_composite(
                &mut canvas_context,
                node,
                bound,
                TransformMatrix::identity(),
                &mut defaults,
            );
        }

        if node.need_redraw().unwrap_or(true) {
            let mut defaults = ShapeDefaults::default();
            Self::render_composite(&mut canvas_context, node, None, &mut defaults);

            // Render the canvas to screen.
            let scene = SceneProxy::from_scene(canvas_context.into_canvas().into_scene(), RayonExecutor);
            scene.build_and_render(&mut renderer_context.renderer, BuildOptions::default());
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[derive(Default, Clone)]
pub struct ShapeDefaults {
    pub transparency: Real,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub clip: Clip,
}

impl PathfinderRender {
    pub fn new(physical_width: u32, physical_height: u32) -> Self {
        let mut render = Self::default();
        render.set_dimensions(physical_width, physical_height, 1.0);
        render
    }

    pub fn load_font(&mut self, _name: impl AsRef<str>, path: impl AsRef<Path>) -> Result<(), <Self as Render>::Error> {
        let context = self.context.as_mut().ok_or(PathfinderRenderError::ContextIsNotInit)?;

        let display_path = path.as_ref().display();
        let mut font_file_data = vec![];
        File::open(&path)
            .and_then(|mut file| file.read_to_end(&mut font_file_data))
            .map_err(|err| PathfinderRenderError::CreateFontError(err, format!("{}", display_path)))?;
        context
            .font_handles
            .push(Handle::from_memory(Arc::new(font_file_data), 0));
        context.font_context = CanvasFontContext::from_fonts(context.font_handles.clone().into_iter());

        Ok(())
    }

    fn recalc_composite(
        canvas: &mut CanvasRenderingContext2D, composite: &mut dyn CompositeShape, parent_bound: BoundingBox,
        mut parent_global_transform: TransformMatrix, defaults: &mut ShapeDefaults,
    ) -> BoundingBox {
        let mut bound = parent_bound;

        if let Some(shape) = composite.shape_mut() {
            match shape {
                Shape::Rect(rect) => {
                    if rect.x.set_by_pct(parent_bound.width()) {
                        rect.x.0 += parent_bound.min_x;
                    }
                    if rect.y.set_by_pct(parent_bound.height()) {
                        rect.y.0 += parent_bound.min_y;
                    }
                    rect.width.set_by_pct(parent_bound.width());
                    rect.height.set_by_pct(parent_bound.height());
                    if let Some(rounding) = &mut rect.rounding {
                        let radius = parent_bound.width().min(parent_bound.height());
                        rounding.top_left.set_by_pct(radius);
                        rounding.top_right.set_by_pct(radius);
                        rounding.bottom_left.set_by_pct(radius);
                        rounding.bottom_right.set_by_pct(radius);
                    }
                    Self::set_by_pct_padding(&mut rect.padding, &parent_bound);
                    Self::set_by_pct_clip(&mut rect.clip, &parent_bound);

                    parent_global_transform = rect.recalculate_transform(parent_global_transform);
                    let (scale_x, scale_y) = parent_global_transform.scale_xy();
                    parent_global_transform
                        .translate_add(rect.padding.left.val() * scale_x, rect.padding.top.val() * scale_y);

                    bound = BoundingBox {
                        min_x: rect.x.val(),
                        min_y: rect.y.val(),
                        max_x: rect.x.val() + rect.width.val(),
                        max_y: rect.y.val() + rect.height.val(),
                    };
                }
                Shape::Circle(circle) => {
                    if circle.cx.set_by_pct(parent_bound.width()) {
                        circle.cx.0 += parent_bound.min_x;
                    }
                    if circle.cy.set_by_pct(parent_bound.height()) {
                        circle.cy.0 += parent_bound.min_y;
                    }
                    circle.r.set_by_pct(parent_bound.width().min(parent_bound.height()));
                    Self::set_by_pct_padding(&mut circle.padding, &parent_bound);
                    Self::set_by_pct_clip(&mut circle.clip, &parent_bound);

                    parent_global_transform = circle.recalculate_transform(parent_global_transform);
                    let (scale_x, scale_y) = parent_global_transform.scale_xy();
                    parent_global_transform
                        .translate_add(circle.padding.left.val() * scale_x, circle.padding.top.val() * scale_y);

                    let (cx, cy, r) = (circle.cx.val(), circle.cy.val(), circle.r.val());
                    bound = BoundingBox {
                        min_x: cx - r,
                        min_y: cy - r,
                        max_x: cx + r,
                        max_y: cy + r,
                    };
                }
                Shape::Text(text) => {
                    if text.x.set_by_pct(parent_bound.width()) {
                        text.x.0 += parent_bound.min_x;
                    }
                    if text.y.set_by_pct(parent_bound.height()) {
                        text.y.0 += parent_bound.min_y;
                    }
                    Self::set_by_pct_clip(&mut text.clip, &parent_bound);

                    parent_global_transform = text.recalculate_transform(parent_global_transform);

                    canvas.save();
                    Self::set_text_options(canvas, text, defaults);

                    let metrics = canvas.measure_text(&text.content);
                    let ascend = metrics.font_bounding_box_ascent.abs();
                    let descend = metrics.font_bounding_box_descent.abs();
                    let line_height = ascend + descend;
                    text.metrics = Some(TextMetrics {
                        ascender: ascend,
                        descender: descend,
                        line_height,
                    });

                    // todo: caching glyph_positions
                    let layout = skribo::layout(
                        &TextStyle {
                            size: canvas.font_size(),
                        },
                        &canvas.font(),
                        &text.content,
                    );

                    text.glyph_positions.clear();
                    let mut prev_pos: Option<Vector2F> = None;
                    for pos in layout
                        .glyphs
                        .iter()
                        .map(|glyph| glyph.offset)
                        .chain(iter::once(layout.advance))
                    {
                        if let Some(prev_pos) = prev_pos {
                            text.glyph_positions.push(GlyphPos {
                                x: prev_pos.x(),
                                y: prev_pos.y(),
                                width: pos.x() - prev_pos.x(),
                            });
                        }
                        prev_pos = Some(pos);
                    }

                    canvas.restore();

                    bound = BoundingBox {
                        min_x: text.x.val(),
                        min_y: text.y.val(),
                        max_x: text.x.val() + text.glyph_positions.last().map(|pos| pos.max_x()).unwrap_or(0.0),
                        max_y: text.y.val() + line_height,
                    };
                }
                Shape::Path(path) => {
                    Self::set_by_pct_clip(&mut path.clip, &parent_bound);
                    parent_global_transform = path.recalculate_transform(parent_global_transform);
                }
                Shape::Group(group) => {
                    Self::set_by_pct_clip(&mut group.clip, &parent_bound);
                    parent_global_transform = group.recalculate_transform(parent_global_transform);

                    if let Some(transparency) = group.transparency {
                        defaults.transparency = transparency;
                    }
                    if let Some(fill) = group.fill {
                        defaults.fill = Some(fill);
                    }
                    if let Some(stroke) = group.stroke {
                        defaults.stroke = Some(stroke);
                    }
                    if !group.clip.is_none() {
                        defaults.clip = group.clip;
                    }
                }
            }
        }

        let inner_bound = Self::calc_inner_bound(canvas, composite, bound, parent_global_transform, defaults);

        if let Some(shape) = composite.shape_mut() {
            match shape {
                Shape::Rect(rect) => {
                    rect.x.set_by_auto(inner_bound.min_x);
                    rect.y.set_by_auto(inner_bound.min_y);
                    rect.width
                        .set_by_auto(inner_bound.max_x - rect.x.val() + rect.padding.left_and_right().val());
                    rect.height
                        .set_by_auto(inner_bound.max_y - rect.y.val() + rect.padding.top_and_bottom().val());

                    bound = BoundingBox {
                        min_x: rect.x.val(),
                        min_y: rect.y.val(),
                        max_x: rect.x.val() + rect.width.val(),
                        max_y: rect.y.val() + rect.height.val(),
                    };
                }
                Shape::Circle(circle) => {
                    circle.cx.set_by_auto(inner_bound.min_x + inner_bound.width() / 2.0);
                    circle.cy.set_by_auto(inner_bound.min_y + inner_bound.height() / 2.0);
                    circle.r.set_by_auto(
                        (inner_bound.width() + circle.padding.left_and_right().val())
                            .max(inner_bound.height() + circle.padding.top_and_bottom().val())
                            / 2.0,
                    );

                    let (cx, cy, r) = (circle.cx.val(), circle.cy.val(), circle.r.val());
                    bound = BoundingBox {
                        min_x: cx - r,
                        min_y: cy - r,
                        max_x: cx + r,
                        max_y: cy + r,
                    };
                }
                Shape::Text(text) => {
                    let transform = text.transform.matrix();
                    let inner_bound_points = transform * inner_bound;
                    let bound_points = transform * bound;

                    bound.min_x = bound_points[0].0;
                    bound.max_x = bound.min_x;
                    bound.min_y = bound_points[0].1;
                    bound.max_y = bound.min_y;
                    for idx in 0..4 {
                        bound.min_x = bound.min_x.min(bound_points[idx].0).min(inner_bound_points[idx].0);
                        bound.max_x = bound.max_x.max(bound_points[idx].0).max(inner_bound_points[idx].0);
                        bound.min_y = bound.min_y.min(bound_points[idx].1).min(inner_bound_points[idx].1);
                        bound.max_y = bound.max_y.max(bound_points[idx].1).max(inner_bound_points[idx].1);
                    }
                }
                _ => (),
            }
        }
        bound
    }

    fn calc_inner_bound(
        canvas: &mut CanvasRenderingContext2D, composite: &mut dyn CompositeShape, bound: BoundingBox,
        parent_global_transform: TransformMatrix, defaults: &mut ShapeDefaults,
    ) -> BoundingBox {
        let mut child_bounds = Vec::new();
        if let Some(children) = composite.children_mut() {
            for child in children {
                child_bounds.push(Self::recalc_composite(
                    canvas,
                    child,
                    bound,
                    parent_global_transform,
                    defaults,
                ));
            }
        }

        if child_bounds.is_empty() {
            BoundingBox::default()
        } else {
            let mut inner_bound = child_bounds[0];
            for bound in &child_bounds[1..] {
                if bound.min_x < inner_bound.min_x {
                    inner_bound.min_x = bound.min_x;
                }
                if bound.min_y < inner_bound.min_y {
                    inner_bound.min_y = bound.min_y;
                }
                if bound.max_x > inner_bound.max_x {
                    inner_bound.max_x = bound.max_x;
                }
                if bound.max_y > inner_bound.max_y {
                    inner_bound.max_y = bound.max_y;
                }
            }
            inner_bound
        }
    }

    fn render_composite<'a>(
        canvas: &mut CanvasRenderingContext2D, composite: &'a dyn CompositeShape, mut text: Option<&'a Text>,
        defaults: &mut ShapeDefaults,
    ) {
        canvas.save();
        if let Some(shape) = composite.shape() {
            match shape {
                Shape::Rect(rect) => {
                    let rect_pos = Vector2F::new(rect.x.val() as f32, rect.y.val() as f32);
                    let rect_size = Vector2F::new(rect.width.val() as f32, rect.height.val() as f32);

                    let rect_path = if let Some(rounding) = rect.rounding {
                        create_rounded_rect_path(rect_pos, rect_size, rounding)
                    } else {
                        let mut path = Path2D::new();
                        path.rect(RectF::new(rect_pos, rect_size));
                        path
                    };
                    Self::set_path_options(canvas, rect.transparency, rect.clip, &rect.transform, defaults);
                    if let Some(fill) = rect.fill.as_ref().or(defaults.fill.as_ref()) {
                        Self::set_fill_option(canvas, fill);
                        canvas.fill_path(rect_path.clone(), FillRule::Winding);
                    };
                    if let Some(stroke) = rect.stroke.as_ref().or(defaults.stroke.as_ref()) {
                        Self::set_stroke_option(canvas, stroke);
                        canvas.stroke_path(rect_path);
                    }
                }
                Shape::Circle(circle) => {
                    let center = Vector2F::new(circle.cx.val(), circle.cy.val());
                    let axes = Vector2F::new(circle.r.val(), circle.r.val());
                    let circle_path = {
                        let mut path = Path2D::new();
                        path.ellipse(center, axes, 0.0, 0.0, PI_2);
                        path
                    };

                    Self::set_path_options(canvas, circle.transparency, circle.clip, &circle.transform, defaults);
                    if let Some(fill) = circle.fill.as_ref().or(defaults.fill.as_ref()) {
                        Self::set_fill_option(canvas, fill);
                        canvas.fill_path(circle_path.clone(), FillRule::Winding);
                    };
                    if let Some(stroke) = circle.stroke.as_ref().or(defaults.stroke.as_ref()) {
                        Self::set_stroke_option(canvas, stroke);
                        canvas.stroke_path(circle_path);
                    }
                }
                Shape::Path(path) => {
                    use exgui_core::PathCommand::*;

                    let mut last_xy = Vector2F::new(0.0, 0.0);
                    let mut bez_ctrls = [Vector2F::new(0.0, 0.0), Vector2F::new(0.0, 0.0)];
                    let mut draw_path = Path2D::new();

                    for cmd in path.cmd.iter() {
                        match cmd {
                            Move(ref xy) => {
                                last_xy = Vector2F::new(xy[0], xy[1]);
                                draw_path.move_to(last_xy);
                            }
                            MoveRel(ref xy) => {
                                last_xy = Vector2F::new(last_xy.x() + xy[0], last_xy.y() + xy[1]);
                                draw_path.move_to(last_xy);
                            }
                            Line(ref xy) => {
                                last_xy = Vector2F::new(xy[0], xy[1]);
                                draw_path.line_to(last_xy);
                            }
                            LineRel(ref xy) => {
                                last_xy = Vector2F::new(last_xy.x() + xy[0], last_xy.y() + xy[1]);
                                draw_path.line_to(last_xy);
                            }
                            LineAlonX(ref x) => {
                                last_xy.set_x(*x);
                                draw_path.line_to(last_xy);
                            }
                            LineAlonXRel(ref x) => {
                                last_xy.set_x(last_xy.x() + *x);
                                draw_path.line_to(last_xy);
                            }
                            LineAlonY(ref y) => {
                                last_xy.set_y(*y);
                                draw_path.line_to(last_xy);
                            }
                            LineAlonYRel(ref y) => {
                                last_xy.set_y(last_xy.y() + *y);
                                draw_path.line_to(last_xy);
                            }
                            Close => draw_path.close_path(),
                            BezCtrl(ref xy) => {
                                bez_ctrls = [bez_ctrls[1], Vector2F::new(xy[0], xy[1])];
                            }
                            BezCtrlRel(ref xy) => {
                                bez_ctrls = [bez_ctrls[1], Vector2F::new(last_xy.x() + xy[0], last_xy.y() + xy[1])];
                            }
                            QuadBezTo(ref xy) => {
                                last_xy = Vector2F::new(xy[0], xy[1]);
                                draw_path.quadratic_curve_to(bez_ctrls[1], last_xy);
                            }
                            QuadBezToRel(ref xy) => {
                                last_xy = Vector2F::new(last_xy.x() + xy[0], last_xy.y() + xy[1]);
                                draw_path.quadratic_curve_to(bez_ctrls[1], last_xy);
                            }
                            CubBezTo(ref xy) => {
                                last_xy = Vector2F::new(xy[0], xy[1]);
                                draw_path.bezier_curve_to(bez_ctrls[0], bez_ctrls[1], last_xy);
                            }
                            CubBezToRel(ref xy) => {
                                last_xy = Vector2F::new(last_xy.x() + xy[0], last_xy.y() + xy[1]);
                                draw_path.bezier_curve_to(bez_ctrls[0], bez_ctrls[1], last_xy);
                            }
                            _ => panic!("Not impl rendering cmd {:?}", cmd), // TODO: need refl impl
                        }
                    }

                    Self::set_path_options(canvas, path.transparency, path.clip, &path.transform, defaults);
                    if let Some(fill) = path.fill.as_ref().or(defaults.fill.as_ref()) {
                        Self::set_fill_option(canvas, fill);
                        canvas.fill_path(draw_path.clone(), FillRule::Winding);
                    };
                    if let Some(stroke) = path.stroke.as_ref().or(defaults.stroke.as_ref()) {
                        Self::set_stroke_option(canvas, stroke);
                        canvas.stroke_path(draw_path);
                    }
                }
                Shape::Text(this_text) => {
                    text = Some(this_text);

                    let pos = Vector2F::new(this_text.x.val(), this_text.y.val());

                    Self::set_text_options(canvas, this_text, defaults);
                    if let Some(fill) = this_text.fill.as_ref().or(defaults.fill.as_ref()) {
                        Self::set_fill_option(canvas, fill);
                        canvas.fill_text(&this_text.content, pos);
                    };
                    if let Some(stroke) = this_text.stroke.as_ref().or(defaults.stroke.as_ref()) {
                        Self::set_stroke_option(canvas, stroke);
                        canvas.stroke_text(&this_text.content, pos);
                    }
                }
                Shape::Group(group) => {
                    if let Some(transparency) = group.transparency {
                        defaults.transparency = transparency;
                    }
                    if let Some(fill) = group.fill {
                        defaults.fill = Some(fill);
                    }
                    if let Some(stroke) = group.stroke {
                        defaults.stroke = Some(stroke);
                    }
                    if !group.clip.is_none() {
                        defaults.clip = group.clip;
                    }
                }
            }
        }
        canvas.restore();

        if let Some(children) = composite.children() {
            for child in children {
                Self::render_composite(canvas, child, text, defaults);
            }
        }
    }

    fn set_by_pct_padding(padding: &mut Padding, parent_bound: &BoundingBox) {
        padding.left.set_by_pct(parent_bound.width());
        padding.right.set_by_pct(parent_bound.width());
        padding.top.set_by_pct(parent_bound.height());
        padding.bottom.set_by_pct(parent_bound.height());
    }

    fn set_by_pct_clip(clip: &mut Clip, parent_bound: &BoundingBox) {
        if let Clip::Scissor(scissor) = clip {
            scissor.x.set_by_pct(parent_bound.width());
            scissor.y.set_by_pct(parent_bound.height());
            scissor.width.set_by_pct(parent_bound.width());
            scissor.height.set_by_pct(parent_bound.height());
        }
    }

    fn pathfinder_transform(transform: &Transform, current_transform: Transform2F) -> Option<Transform2F> {
        if transform.is_not_exist() {
            None
        } else {
            let matrix = transform
                .calculated_matrix()
                .unwrap_or_else(|| transform.matrix())
                .matrix;
            let pathfinder_transform = Transform2F::row_major(
                matrix[0] as f32,
                matrix[2] as f32,
                matrix[1] as f32,
                matrix[3] as f32,
                matrix[4] as f32,
                matrix[5] as f32,
            );
            Some(if !transform.is_absolute() {
                current_transform * pathfinder_transform
            } else {
                pathfinder_transform
            })
        }
    }

    fn clip_path(clip: &Clip, current_transform: Transform2F) -> Option<Path2D> {
        match clip {
            Clip::Scissor(scissor) => {
                let mut clip_rect = RectF::new(
                    vec2f(scissor.x.val() as f32, scissor.y.val() as f32),
                    vec2f(scissor.width.val() as f32, scissor.height.val() as f32),
                );
                if let Some(transform) = Self::pathfinder_transform(&scissor.transform, current_transform) {
                    clip_rect = transform * clip_rect;
                }

                let mut clip_path = Path2D::new();
                clip_path.rect(clip_rect);
                Some(clip_path)
            }
            Clip::None => None,
        }
    }

    fn set_path_options(
        canvas: &mut CanvasRenderingContext2D, transparency: Real, clip: Clip, transform: &Transform,
        defaults: &ShapeDefaults,
    ) {
        let transparency = if transparency != 0.0 {
            transparency
        } else {
            defaults.transparency
        };
        canvas.set_global_alpha(1.0 - transparency);
        let current_transform = canvas.transform();
        if let Some(clip_path) = Self::clip_path(&clip.or(defaults.clip), current_transform) {
            canvas.clip_path(clip_path, FillRule::Winding);
        }
        if let Some(transform) = Self::pathfinder_transform(transform, current_transform) {
            canvas.set_transform(&transform);
        }
    }

    fn set_fill_option(canvas: &mut CanvasRenderingContext2D, fill: &Fill) {
        canvas.set_fill_style(ToPathfinderPaint(fill.paint));
    }

    fn set_stroke_option(canvas: &mut CanvasRenderingContext2D, stroke: &Stroke) {
        canvas.set_stroke_style(ToPathfinderPaint(stroke.paint));
        canvas.set_line_width(stroke.width);
        canvas.set_miter_limit(stroke.miter_limit);
        let line_cap = match stroke.line_cap {
            LineCap::Butt => PathfinderLineCap::Butt,
            LineCap::Round => PathfinderLineCap::Round,
            LineCap::Square => PathfinderLineCap::Square,
        };
        let line_join = match stroke.line_join {
            LineJoin::Miter => PathfinderLineJoin::Miter,
            LineJoin::Round => PathfinderLineJoin::Round,
            LineJoin::Bevel => PathfinderLineJoin::Bevel,
        };
        canvas.set_line_cap(line_cap);
        canvas.set_line_join(line_join);
    }

    fn set_text_options(canvas: &mut CanvasRenderingContext2D, text: &Text, defaults: &ShapeDefaults) {
        let transparency = if text.transparency != 0.0 {
            text.transparency
        } else {
            defaults.transparency
        };
        canvas.set_global_alpha(1.0 - transparency);
        canvas.set_font(&[text.font_name.as_str()][..]);
        canvas.set_font_size(text.font_size.val());
        canvas.set_text_align(match text.align.0 {
            AlignHor::Left => TextAlign::Left,
            AlignHor::Right => TextAlign::Right,
            AlignHor::Center => TextAlign::Center,
        });
        canvas.set_text_baseline(match text.align.1 {
            AlignVer::Bottom => TextBaseline::Bottom,
            AlignVer::Middle => TextBaseline::Middle,
            AlignVer::Baseline => TextBaseline::Alphabetic,
            AlignVer::Top => TextBaseline::Top,
        });
        let current_transform = canvas.transform();
        if let Some(clip_path) = Self::clip_path(&text.clip.or(defaults.clip), current_transform) {
            canvas.clip_path(clip_path, FillRule::Winding);
        }
        if let Some(transform) = Self::pathfinder_transform(&text.transform, current_transform) {
            canvas.set_transform(&transform);
        }
    }
}

fn create_rounded_rect_path(rect_pos: Vector2F, rect_size: Vector2F, rounding: Rounding) -> Path2D {
    let rect = RectF::new(rect_pos, rect_size);
    let mut path = Path2D::new();

    let draw_segment = |path: &mut Path2D, to_point: Vector2F, radius: f32, vec: Vector2F| {
        if radius.abs() > 0.0 + f32::EPSILON {
            path.arc_to(to_point, to_point + vec, radius);
        } else {
            path.line_to(to_point);
        }
    };

    let radius = rounding.top_left.val();
    path.move_to(rect.origin() + vec2f(radius, 0.0));

    let radius = rounding.top_right.val();
    draw_segment(&mut path, rect.upper_right(), radius, vec2f(0.0, radius));

    let radius = rounding.bottom_right.val();
    draw_segment(&mut path, rect.lower_right(), radius, vec2f(-radius, 0.0));

    let radius = rounding.bottom_left.val();
    draw_segment(&mut path, rect.lower_left(), radius, vec2f(0.0, -radius));

    let radius = rounding.top_left.val();
    draw_segment(&mut path, rect.origin(), radius, vec2f(radius, 0.0));

    path.close_path();
    path
}

struct ToPathfinderPaint(Paint);

impl ToPathfinderPaint {
    fn to_color(color: Color) -> ColorF {
        let [r, g, b, a] = color.as_arr();
        ColorF::new(r, g, b, a)
    }

    fn to_gradient(gradient: Gradient) -> PathfinderGradient {
        match gradient {
            Gradient::Linear {
                start: (start_x, start_y),
                end: (end_x, end_y),
                start_color,
                end_color,
            } => {
                let mut gradient = PathfinderGradient::linear_from_points(
                    Vector2F::new(start_x as f32, start_y as f32),
                    Vector2F::new(end_x as f32, end_y as f32),
                );
                gradient.add_color_stop(Self::to_color(start_color).to_u8(), 0.0);
                gradient.add_color_stop(Self::to_color(end_color).to_u8(), 1.0);
                gradient
            }
            Gradient::Box { .. } => todo!("The Box gradient is not support"),
            Gradient::Radial {
                center: (x, y),
                inner_radius,
                outer_radius,
                start_color,
                end_color,
            } => {
                let mut gradient =
                    PathfinderGradient::radial(Vector2F::new(x, y), F32x2::new(inner_radius, outer_radius));
                gradient.add_color_stop(Self::to_color(start_color).to_u8(), 0.0);
                gradient.add_color_stop(Self::to_color(end_color).to_u8(), 1.0);
                gradient
            }
        }
    }

    fn into_fill_style(self) -> FillStyle {
        match self.0 {
            Paint::Color(color) => FillStyle::Color(Self::to_color(color).to_u8()),
            Paint::Gradient(gradient) => FillStyle::Gradient(Self::to_gradient(gradient)),
        }
    }
}

impl From<ToPathfinderPaint> for FillStyle {
    fn from(paint: ToPathfinderPaint) -> Self {
        paint.into_fill_style()
    }
}
