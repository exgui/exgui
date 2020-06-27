use std::path::Path;

use nanovg::{
    Context, ContextBuilder, Font as NanovgFont, CreateFontError, Frame,
    Color as NanovgColor, Gradient as NanovgGradient, Paint as NanovgPaint,
    StrokeOptions, PathOptions, TextOptions, Alignment,
    LineCap as NanovgLineCap, LineJoin as NanovgLineJoin, Transform as NanovgTransform,
};
use exgui_core::{
    Real, GlyphPos, CompositeShape, Shape, Paint, Color, Gradient, Stroke, Fill, Text, AlignHor,
    AlignVer, Transform, LineCap, LineJoin, Render, TransformMatrix, TextMetrics,
};

struct ToNanovgPaint(Paint);

impl ToNanovgPaint {
    fn to_nanovg_color(color: Color) -> NanovgColor {
        let [r, g, b, a] = color.as_arr();
        NanovgColor::new(r, g, b, a)
    }

    fn to_nanovg_gradient(gradient: Gradient) -> NanovgGradient {
        match gradient {
            Gradient::Linear { start, end, start_color, end_color } =>
                NanovgGradient::Linear {
                    start, end,
                    start_color: Self::to_nanovg_color(start_color),
                    end_color: Self::to_nanovg_color(end_color),
                },
            Gradient::Box { position, size, radius, feather, start_color, end_color } =>
                NanovgGradient::Box {
                    position, size, radius, feather,
                    start_color: Self::to_nanovg_color(start_color),
                    end_color: Self::to_nanovg_color(end_color),
                },
            Gradient::Radial { center, inner_radius, outer_radius, start_color, end_color } =>
                NanovgGradient::Radial {
                    center, inner_radius, outer_radius,
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

    fn init(&mut self) -> Result<(), Self::Error> {
        if self.context.is_none() {
            let context = ContextBuilder::new()
                .stencil_strokes()
                .build()
                .map_err(|_| NanovgRenderError::InitNanovgContextFailed)?;
            self.context = Some(context);
        }
        Ok(())
    }

    fn set_dimensions(&mut self, width: u32, height: u32, device_pixel_ratio: f64) {
        self.width = width as f32;
        self.height = height as f32;
        self.device_pixel_ratio = device_pixel_ratio as f32;
    }

    fn render(&self, node: &mut dyn CompositeShape) -> Result<(), Self::Error> {
        self.context
            .as_ref()
            .ok_or(NanovgRenderError::ContextIsNotInit)?
            .frame(
                (self.width, self.height),
                self.device_pixel_ratio,
                move |frame| {
                    let bound = BoundingBox {
                        min_x: 0.0,
                        min_y: 0.0,
                        max_x: self.width,
                        max_y: self.height,
                    };

                    let mut defaults = ShapeDefaults::default();
                    Self::recalc_composite(&frame, node, bound, None, TransformMatrix::identity(), &mut defaults);

                    let mut defaults = ShapeDefaults::default();
                    Self::render_composite(&frame, node, None, &mut defaults);
                }
            );
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct ShapeDefaults {
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
}

impl NanovgRender {
    pub fn new(context: Context, width: f32, height: f32, device_pixel_ratio: f32) -> Self {
        Self {
            context: Some(context),
            width,
            height,
            device_pixel_ratio
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

    pub fn load_font(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> Result<(), <Self as Render>::Error> {
        let name = name.into();
        let display_path = path.as_ref().display();
        NanovgFont::from_file(
            self.context.as_ref().ok_or(NanovgRenderError::ContextIsNotInit)?,
            name.as_str(),
            path.as_ref()
        ).map_err(|e| NanovgRenderError::CreateFontError(e, format!("{}", display_path)))?;
        Ok(())
    }

    fn recalc_composite(
        frame: &Frame,
        composite: &mut dyn CompositeShape,
        parent_bound: BoundingBox,
        text: Option<&Text>,
        mut parent_global_transform: TransformMatrix,
        defaults: &mut ShapeDefaults,
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
                    parent_global_transform = rect.recalculate_transform(parent_global_transform);

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
                    parent_global_transform = circle.recalculate_transform(parent_global_transform);

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

                    text.glyph_positions = frame.text_glyph_positions(
                        (text.x.val(), text.y.val()),
                        &text.content,
                    ).map(|pos| GlyphPos { x: pos.x, min_x: pos.min_x, max_x: pos.max_x }).collect();

                    let text = text.clone();
                    return Self::calc_inner_bound(frame, composite, bound, Some(&text), parent_global_transform, defaults);
                }
                Shape::Path(path) => {
                    parent_global_transform = path.recalculate_transform(parent_global_transform);
                }
                Shape::Group(group) => {
                    parent_global_transform = group.recalculate_transform(parent_global_transform);

                    if let Some(ref fill) = group.fill {
                        defaults.fill = Some(fill.clone());
                    }
                    if let Some(ref stroke) = group.stroke {
                        defaults.stroke = Some(stroke.clone());
                    }
                }
//                Shape::Word(word) => {
//                    if let Some(text) = text {
//                        let nanovg_font = NanovgFont::find(frame.context(), text.font_name.as_str())
//                            .expect(&format!("Font '{}' not found", text.font_name));
//
//                        let text_options = if let AlignHor::Center = text.align.0 {
//                            // Fix nanovg text_bounds bug for centered text
//                            let mut text = text.clone();
//                            text.align.0 = AlignHor::Left;
//                            Self::text_options(&text)
//                        } else {
//                            Self::text_options(text)
//                        };
//
//                        let mut text_bounds = frame.text_box_bounds(
//                            nanovg_font,
//                            (text.x.val(), text.y.val()),
//                            word,
//                            text_options,
//                        );
//
//                        // Fix nanovg text_bounds bug for centered text
//                        if let AlignHor::Center = text.align.0 {
//                            let half_width = (text_bounds.max_x - text_bounds.min_x) / 2.0;
//                            text_bounds.min_x -= half_width;
//                            text_bounds.max_x -= half_width;
//                        }
//
//                        bound = BoundingBox {
//                            min_x: text_bounds.min_x,
//                            min_y: text_bounds.min_y,
//                            max_x: text_bounds.max_x,
//                            max_y: text_bounds.max_y,
//                        };
//                    }
//                }
            }
        }

        let inner_bound = Self::calc_inner_bound(frame, composite, bound, text, parent_global_transform, defaults);

        if let Some(shape) = composite.shape_mut() {
            match shape {
                Shape::Rect(rect) => {
                    rect.x.set_by_auto(inner_bound.min_x);
                    rect.y.set_by_auto(inner_bound.min_y);
                    rect.width.set_by_auto(inner_bound.width());
                    rect.height.set_by_auto(inner_bound.height());

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
                    circle.r.set_by_auto(inner_bound.width().max(inner_bound.height()) / 2.0);

                    let (cx, cy, r) = (circle.cx.val(), circle.cy.val(), circle.r.val());
                    bound = BoundingBox {
                        min_x: cx - r,
                        min_y: cy - r,
                        max_x: cx + r,
                        max_y: cy + r,
                    };
                }
                _ => (),
            }
        }
        bound
    }

    fn calc_inner_bound(
        frame: &Frame,
        composite: &mut dyn CompositeShape,
        bound: BoundingBox,
        text: Option<&Text>,
        parent_global_transform: TransformMatrix,
        defaults: &mut ShapeDefaults,
    ) -> BoundingBox {
        let mut child_bounds = Vec::new();
        if let Some(children) = composite.children_mut() {
            for child in children {
                child_bounds.push(
                    Self::recalc_composite(frame, child, bound, text, parent_global_transform, defaults)
                );
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

    fn render_composite<'a>(frame: &Frame, composite: &'a dyn CompositeShape, mut text: Option<&'a Text>, defaults: &mut ShapeDefaults) {
        if let Some(shape) = composite.shape() {
            match shape {
                Shape::Rect(rect) => {
                    frame.path(
                        |path| {
                            path.rect((rect.x.val(), rect.y.val()), (rect.width.val(), rect.height.val()));
                            if let Some(fill) = rect.fill.as_ref().or(defaults.fill.as_ref()) {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = rect.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(&rect.transform),
                    );
                }
                Shape::Circle(circle) => {
                    frame.path(
                        |path| {
                            path.circle((circle.cx.val(), circle.cy.val()), circle.r.val());
                            if let Some(fill) = circle.fill.as_ref().or(defaults.fill.as_ref()) {
                                path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = circle.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(&circle.transform),
                    );
                }
                Shape::Path(path) => {
                    frame.path(
                        |nvg_path| {
                            use exgui_core::PathCommand::*;

                            let mut last_xy = [0.0_f32, 0.0];
                            let mut bez_ctrls = [(0.0_f32, 0.0), (0.0_f32, 0.0)];

                            for cmd in path.cmd.iter() {
                                match cmd {
                                    Move(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    MoveRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.move_to((last_xy[0], last_xy[1]));
                                    },
                                    Line(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonX(ref x) => {
                                        last_xy[0] = *x;
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonXRel(ref x) => {
                                        last_xy[0] += *x;
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonY(ref y) => {
                                        last_xy[1] = *y;
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    LineAlonYRel(ref y) => {
                                        last_xy[1] += *y;
                                        nvg_path.line_to((last_xy[0], last_xy[1]));
                                    },
                                    Close => nvg_path.close(),
                                    BezCtrl(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (xy[0], xy[1])];
                                    },
                                    BezCtrlRel(ref xy) => {
                                        bez_ctrls = [bez_ctrls[1], (last_xy[0] + xy[0], last_xy[1] + xy[1])];
                                    },
                                    QuadBezTo(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    QuadBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.quad_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[1]);
                                    },
                                    CubBezTo(ref xy) => {
                                        last_xy = *xy;
                                        nvg_path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    CubBezToRel(ref xy) => {
                                        last_xy = [last_xy[0] + xy[0], last_xy[1] + xy[1]];
                                        nvg_path.cubic_bezier_to((last_xy[0], last_xy[1]), bez_ctrls[0], bez_ctrls[1]);
                                    },
                                    _ => panic!("Not impl rendering cmd {:?}", cmd), // TODO: need refl impl
                                }
                            }
                            if let Some(fill) = path.fill.as_ref().or(defaults.fill.as_ref()) {
                                nvg_path.fill(ToNanovgPaint(fill.paint), Default::default());
                            };
                            if let Some(stroke) = path.stroke.as_ref().or(defaults.stroke.as_ref()) {
                                nvg_path.stroke(
                                    ToNanovgPaint(stroke.paint),
                                    Self::stroke_option(&stroke)
                                );
                            }
                        },
                        Self::path_options(&path.transform),
                    );
                }
                Shape::Text(this_text) => {
                    text = Some(this_text);

                    let nanovg_font = NanovgFont::find(frame.context(), &this_text.font_name)
                        .expect(&format!("Font '{}' not found", this_text.font_name));
                    let text_options = Self::text_options(this_text, defaults);

                    frame.text(
                        nanovg_font,
                        (this_text.x.val(), this_text.y.val()),
                        &this_text.content,
                        text_options,
                    );
                }
//                Shape::Word(word) => {
//                    if let Some(text) = full_text {
//                        let nanovg_font = NanovgFont::find(frame.context(), text.font_name.as_str())
//                            .expect(&format!("Font '{}' not found", text.font_name));
//                        let text_options = Self::text_options(text);
//
//                        frame.text(
//                            nanovg_font,
//                            (text.x.val(), text.y.val()),
//                            word,
//                            text_options,
//                        );
//                    }
//                }
                Shape::Group(group) => {
                    if let Some(ref fill) = group.fill {
                        defaults.fill = Some(fill.clone());
                    }
                    if let Some(ref stroke) = group.stroke {
                        defaults.stroke = Some(stroke.clone());
                    }
                },
            }
        }
        if let Some(children) = composite.children() {
            for child in children {
                Self::render_composite(frame, child, text, defaults);
            }
        }
    }

    fn nanovg_transform(transform: &Transform) -> Option<NanovgTransform> {
        if !transform.is_not_exist() {
            let mut nanovg_transform = NanovgTransform::new();
            if transform.is_absolute() {
                nanovg_transform.absolute();
            }
            nanovg_transform.matrix = transform
                .calculated_matrix()
                .unwrap_or_else(|| transform.matrix())
                .matrix;
            Some(nanovg_transform)
        } else {
            None
        }
    }

    fn path_options(transform: &Transform) -> PathOptions {
        PathOptions {
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
            width: stroke.width,
            line_cap,
            line_join,
            miter_limit: stroke.miter_limit,
            ..Default::default()
        }
    }

    fn text_options(text: &Text, defaults: &ShapeDefaults) -> TextOptions {
        let color = ToNanovgPaint::to_nanovg_color(
            text.fill.as_ref().or(defaults.fill.as_ref()).and_then(|fill| if let Paint::Color(color) = fill.paint {
                Some(color)
            } else {
                None
            }).unwrap_or_default()
        );
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
            size: text.font_size.val(),
            align,
            transform: Self::nanovg_transform(&text.transform),
            ..Default::default()
        }
    }
}
