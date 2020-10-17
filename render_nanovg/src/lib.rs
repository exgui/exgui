use std::{ops::Mul, path::Path};

use exgui_core::{
    AlignHor, AlignVer, Clip, Color, CompositeShape, Fill, GlyphPos, Gradient, LineCap, LineJoin, Padding, Paint, Real,
    Render, Shape, Stroke, Text, TextMetrics, Transform, TransformMatrix,
};
use nanovg::{
    Alignment, Clip as NanovgClip, Color as NanovgColor, Context, ContextBuilder, CreateFontError, Font as NanovgFont,
    Frame, Gradient as NanovgGradient, LineCap as NanovgLineCap, LineJoin as NanovgLineJoin, Paint as NanovgPaint,
    PathOptions, Scissor as NanovgScissor, StrokeOptions, TextOptions, Transform as NanovgTransform,
};

struct ToNanovgPaint(Paint);

impl ToNanovgPaint {
    fn to_nanovg_color(color: Color) -> NanovgColor {
        let [r, g, b, a] = color.as_arr();
        NanovgColor::new(r, g, b, a)
    }

    fn to_nanovg_gradient(gradient: Gradient) -> NanovgGradient {
        match gradient {
            Gradient::Linear {
                start: (start_x, start_y),
                end: (end_x, end_y),
                start_color,
                end_color,
            } => NanovgGradient::Linear {
                start: (start_x as f32, start_y as f32),
                end: (end_x as f32, end_y as f32),
                start_color: Self::to_nanovg_color(start_color),
                end_color: Self::to_nanovg_color(end_color),
            },
            Gradient::Box {
                position: (x, y),
                size: (width, height),
                radius,
                feather,
                start_color,
                end_color,
            } => NanovgGradient::Box {
                position: (x as f32, y as f32),
                size: (width as f32, height as f32),
                radius: radius as f32,
                feather: feather as f32,
                start_color: Self::to_nanovg_color(start_color),
                end_color: Self::to_nanovg_color(end_color),
            },
            Gradient::Radial {
                center: (x, y),
                inner_radius,
                outer_radius,
                start_color,
                end_color,
            } => NanovgGradient::Radial {
                center: (x as f32, y as f32),
                inner_radius: inner_radius as f32,
                outer_radius: outer_radius as f32,
                start_color: Self::to_nanovg_color(start_color),
                end_color: Self::to_nanovg_color(end_color),
            },
        }
    }
}

impl NanovgPaint for ToNanovgPaint {
    fn fill(&self, context: &Context) {
        match self.0 {
            Paint::Color(ref color) => Self::to_nanovg_color(*color).fill(context),
            Paint::Gradient(ref gradient) => Self::to_nanovg_gradient(*gradient).fill(context),
        }
    }

    fn stroke(&self, context: &Context) {
        match self.0 {
            Paint::Color(ref color) => Self::to_nanovg_color(*color).stroke(context),
            Paint::Gradient(ref gradient) => Self::to_nanovg_gradient(*gradient).stroke(context),
        }
    }
}

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
pub enum NanovgRenderError {
    ContextIsNotInit,
    InitNanovgContextFailed,
    CreateFontError(CreateFontError, String),
}

#[derive(Debug, Default)]
pub struct NanovgRender {
    pub context: Option<Context>,
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

impl Render for NanovgRender {
    type Error = NanovgRenderError;

    fn init(&mut self, _background_color: Color) -> Result<(), Self::Error> {
        if self.context.is_none() {
            let context = ContextBuilder::new()
                .stencil_strokes()
                .build()
                .map_err(|_| NanovgRenderError::InitNanovgContextFailed)?;
            self.context = Some(context);
        }
        Ok(())
    }

    fn set_dimensions(&mut self, physical_width: u32, physical_height: u32, device_pixel_ratio: f64) {
        self.width = physical_width as f32;
        self.height = physical_height as f32;
        self.device_pixel_ratio = device_pixel_ratio as f32;
    }

    fn render(&mut self, node: &mut dyn CompositeShape) -> Result<(), Self::Error> {
        let shared_self = &*self;
        shared_self
            .context
            .as_ref()
            .ok_or(NanovgRenderError::ContextIsNotInit)?
            .frame(
                (shared_self.width, shared_self.height),
                shared_self.device_pixel_ratio,
                move |frame| {
                    let bound = BoundingBox {
                        min_x: 0.0,
                        min_y: 0.0,
                        max_x: shared_self.width as Real,
                        max_y: shared_self.height as Real,
                    };

                    if node.need_recalc().unwrap_or(true) {
                        let mut defaults = ShapeDefaults::default();
                        Self::recalc_composite(&frame, node, bound, TransformMatrix::identity(), &mut defaults);
                    }
                    let mut defaults = ShapeDefaults::default();
                    Self::render_composite(&frame, node, None, &mut defaults);
                },
            );
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct ShapeDefaults {
    pub transparency: Real,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub clip: Clip,
}

impl NanovgRender {
    pub fn new(context: Context, width: f32, height: f32, device_pixel_ratio: f32) -> Self {
        Self {
            context: Some(context),
            width,
            height,
            device_pixel_ratio,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn with_device_pixel_ratio(mut self, device_pixel_ratio: f32) -> Self {
        self.device_pixel_ratio = device_pixel_ratio;
        self
    }

    pub fn load_font(
        &mut self, name: impl Into<String>, path: impl AsRef<Path>,
    ) -> Result<(), <Self as Render>::Error> {
        let name = name.into();
        let display_path = path.as_ref().display();
        NanovgFont::from_file(
            self.context.as_ref().ok_or(NanovgRenderError::ContextIsNotInit)?,
            name.as_str(),
            path.as_ref(),
        )
        .map_err(|e| NanovgRenderError::CreateFontError(e, format!("{}", display_path)))?;
        Ok(())
    }

    fn recalc_composite(
        frame: &Frame, composite: &mut dyn CompositeShape, parent_bound: BoundingBox,
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

                    let nanovg_font = NanovgFont::find(frame.context(), &text.font_name)
                        .expect(&format!("Font '{}' not found", text.font_name));
                    let text_options = Self::text_options(text, defaults);

                    let metrics = frame.text_metrics(nanovg_font, text_options);
                    text.metrics = Some(TextMetrics {
                        ascender: metrics.ascender,
                        descender: metrics.descender,
                        line_height: metrics.line_height,
                    });

                    text.glyph_positions = frame
                        .text_glyph_positions((text.x.val() as f32, text.y.val() as f32), &text.content)
                        .map(|pos| {
                            let x = pos.x.min(pos.min_x);
                            GlyphPos {
                                x,
                                y: 0.0,
                                width: pos.max_x - x,
                            }
                        })
                        .collect();
                    bound = BoundingBox {
                        min_x: text.x.val(),
                        min_y: text.y.val(),
                        max_x: text.x.val() + text.glyph_positions.last().map(|pos| pos.max_x()).unwrap_or(0.0),
                        max_y: text.y.val() + metrics.line_height as Real,
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

        let inner_bound = Self::calc_inner_bound(frame, composite, bound, parent_global_transform, defaults);

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
        frame: &Frame, composite: &mut dyn CompositeShape, bound: BoundingBox,
        parent_global_transform: TransformMatrix, defaults: &mut ShapeDefaults,
    ) -> BoundingBox {
        let mut child_bounds = Vec::new();
        if let Some(children) = composite.children_mut() {
            for child in children {
                child_bounds.push(Self::recalc_composite(
                    frame,
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
        frame: &Frame, composite: &'a dyn CompositeShape, mut text: Option<&'a Text>, defaults: &mut ShapeDefaults,
    ) {
        if let Some(shape) = composite.shape() {
            match shape {
                Shape::Rect(rect) => {
                    frame.path(
                        |path| {
                            let rect_pos = (rect.x.val() as f32, rect.y.val() as f32);
                            let rect_size = (rect.width.val() as f32, rect.height.val() as f32);
                            if let Some(rounding) = rect.rounding {
                                path.rounded_rect_varying(
                                    rect_pos,
                                    rect_size,
                                    (rounding.top_left.val() as f32, rounding.top_right.val() as f32),
                                    (rounding.bottom_left.val() as f32, rounding.bottom_right.val() as f32),
                                );
                            } else {
                                path.rect(rect_pos, rect_size);
                            }
                            if let Some(fill) = rect.fill.as_ref().or(defaults.fill.as_ref()) {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = rect.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                path.stroke(ToNanovgPaint(stroke.paint), Self::stroke_option(&stroke));
                            }
                        },
                        Self::path_options(rect.transparency, rect.clip, &rect.transform, defaults),
                    );
                }
                Shape::Circle(circle) => {
                    frame.path(
                        |path| {
                            path.circle((circle.cx.val() as f32, circle.cy.val() as f32), circle.r.val() as f32);
                            if let Some(fill) = circle.fill.as_ref().or(defaults.fill.as_ref()) {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = circle.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                path.stroke(ToNanovgPaint(stroke.paint), Self::stroke_option(&stroke));
                            }
                        },
                        Self::path_options(circle.transparency, circle.clip, &circle.transform, defaults),
                    );
                }
                Shape::Path(path) => {
                    frame.path(
                        |nvg_path| {
                            use exgui_core::PathCommand::*;

                            let mut last_xy = [0.0, 0.0];
                            let mut bez_ctrls = [(0.0, 0.0), (0.0, 0.0)];

                            for cmd in path.cmd.iter() {
                                match cmd {
                                    Move(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.move_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    MoveRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.move_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    Line(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    LineRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    LineAlonX(ref x) => {
                                        last_xy[0] = *x;
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    LineAlonXRel(ref x) => {
                                        last_xy[0] += *x;
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    LineAlonY(ref y) => {
                                        last_xy[1] = *y;
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    LineAlonYRel(ref y) => {
                                        last_xy[1] += *y;
                                        nvg_path.line_to((last_xy[0] as f32, last_xy[1] as f32));
                                    }
                                    Close => nvg_path.close(),
                                    BezCtrl(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (xy[0], xy[1])];
                                    }
                                    BezCtrlRel(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (last_xy[0] + xy[0], last_xy[1] + xy[1])];
                                    }
                                    QuadBezTo(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.quad_bezier_to(
                                            (last_xy[0] as f32, last_xy[1] as f32),
                                            (bez_ctrls[1].0 as f32, bez_ctrls[1].1 as f32),
                                        );
                                    }
                                    QuadBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.quad_bezier_to(
                                            (last_xy[0] as f32, last_xy[1] as f32),
                                            (bez_ctrls[1].0 as f32, bez_ctrls[1].1 as f32),
                                        );
                                    }
                                    CubBezTo(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.cubic_bezier_to(
                                            (last_xy[0] as f32, last_xy[1] as f32),
                                            (bez_ctrls[0].0 as f32, bez_ctrls[0].1 as f32),
                                            (bez_ctrls[1].0 as f32, bez_ctrls[1].1 as f32),
                                        );
                                    }
                                    CubBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.cubic_bezier_to(
                                            (last_xy[0] as f32, last_xy[1] as f32),
                                            (bez_ctrls[0].0 as f32, bez_ctrls[0].1 as f32),
                                            (bez_ctrls[1].0 as f32, bez_ctrls[1].1 as f32),
                                        );
                                    }
                                    _ => panic!("Not impl rendering cmd {:?}", cmd), // TODO: need refl impl
                                }
                            }
                            if let Some(fill) = path.fill.as_ref().or(defaults.fill.as_ref()) {
                                nvg_path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = path.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                nvg_path.stroke(ToNanovgPaint(stroke.paint), Self::stroke_option(&stroke));
                            }
                        },
                        Self::path_options(path.transparency, path.clip, &path.transform, defaults),
                    );
                }
                Shape::Text(this_text) => {
                    text = Some(this_text);

                    let nanovg_font = NanovgFont::find(frame.context(), &this_text.font_name)
                        .expect(&format!("Font '{}' not found", this_text.font_name));
                    let text_options = Self::text_options(this_text, defaults);

                    frame.text(
                        nanovg_font,
                        (this_text.x.val() as f32, this_text.y.val() as f32),
                        &this_text.content,
                        text_options,
                    );
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
        if let Some(children) = composite.children() {
            for child in children {
                Self::render_composite(frame, child, text, defaults);
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

    fn nanovg_transform(transform: &Transform) -> Option<NanovgTransform> {
        if transform.is_not_exist() {
            None
        } else {
            let mut nanovg_transform = NanovgTransform::new();
            if transform.is_absolute() {
                nanovg_transform.absolute();
            }
            let matrix = transform
                .calculated_matrix()
                .unwrap_or_else(|| transform.matrix())
                .matrix;
            nanovg_transform.matrix = [
                matrix[0] as f32,
                matrix[1] as f32,
                matrix[2] as f32,
                matrix[3] as f32,
                matrix[4] as f32,
                matrix[5] as f32,
            ];
            Some(nanovg_transform)
        }
    }

    fn nanovg_clip(clip: &Clip) -> NanovgClip {
        match clip {
            Clip::Scissor(scissor) => NanovgClip::Scissor(NanovgScissor {
                x: scissor.x.val() as f32,
                y: scissor.y.val() as f32,
                width: scissor.width.val() as f32,
                height: scissor.height.val() as f32,
                transform: Self::nanovg_transform(&scissor.transform),
            }),
            Clip::None => NanovgClip::None,
        }
    }

    fn path_options(transparency: Real, clip: Clip, transform: &Transform, defaults: &ShapeDefaults) -> PathOptions {
        PathOptions {
            alpha: ((1.0 - transparency) * (1.0 - defaults.transparency)) as f32,
            clip: Self::nanovg_clip(&clip.or(defaults.clip)),
            transform: Self::nanovg_transform(transform),
            ..Default::default()
        }
    }

    fn stroke_option(stroke: &Stroke) -> StrokeOptions {
        let line_cap = match stroke.line_cap {
            LineCap::Butt => NanovgLineCap::Butt,
            LineCap::Round => NanovgLineCap::Round,
            LineCap::Square => NanovgLineCap::Square,
        };
        let line_join = match stroke.line_join {
            LineJoin::Miter => NanovgLineJoin::Miter,
            LineJoin::Round => NanovgLineJoin::Round,
            LineJoin::Bevel => NanovgLineJoin::Bevel,
        };
        StrokeOptions {
            width: stroke.width as f32,
            line_cap,
            line_join,
            miter_limit: stroke.miter_limit as f32,
            ..Default::default()
        }
    }

    fn text_options(text: &Text, defaults: &ShapeDefaults) -> TextOptions {
        let mut color = ToNanovgPaint::to_nanovg_color(
            text.fill
                .as_ref()
                .or(defaults.fill.as_ref())
                .and_then(|fill| {
                    if let Paint::Color(color) = fill.paint {
                        Some(color)
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
        );
        color.set_alpha(color.alpha() * (1.0 - defaults.transparency) * (1.0 - text.transparency));

        let mut align = Alignment::new();
        align = match text.align.0 {
            AlignHor::Left => align.left(),
            AlignHor::Right => align.right(),
            AlignHor::Center => align.center(),
        };
        align = match text.align.1 {
            AlignVer::Bottom => align.bottom(),
            AlignVer::Middle => align.middle(),
            AlignVer::Baseline => align.baseline(),
            AlignVer::Top => align.top(),
        };

        TextOptions {
            color,
            size: text.font_size.val() as f32,
            align,
            clip: Self::nanovg_clip(&text.clip.or(defaults.clip)),
            transform: Self::nanovg_transform(&text.transform),
            ..Default::default()
        }
    }
}
