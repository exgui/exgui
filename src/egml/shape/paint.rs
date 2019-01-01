
#[derive(Copy, Clone, Debug)]
pub enum Paint {
    Color(Color),
    Gradient(Gradient),
}

impl Default for Paint {
    fn default() -> Self {
        Paint::Color(Color::default())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    White,
    Black,
    RGB(f32, f32, f32),
    RGBA(f32, f32, f32, f32),
}

impl Color {
    pub fn with_alpha(&self, alpha: f32) -> Color {
        let [r, g, b, _] = self.as_arr();
        Color::RGBA(r, g, b, alpha)
    }

    pub fn as_arr(&self) -> [f32; 4] {
        match *self {
            Color::Red => [1.0, 0.0, 0.0, 1.0],
            Color::Green => [0.0, 1.0, 0.0, 1.0],
            Color::Blue => [0.0, 0.0, 1.0, 1.0],
            Color::Yellow => [1.0, 1.0, 0.0, 1.0],
            Color::White => [1.0, 1.0, 1.0, 1.0],
            Color::Black => [0.0, 0.0, 0.0, 1.0],
            Color::RGB(r, g, b) => [r, g, b, 1.0],
            Color::RGBA(r, g, b, a) => [r, g, b, a],
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

impl From<(Color, f32)> for Color {
    fn from((color, alpha): (Color, f32)) -> Self {
        color.with_alpha(alpha)
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Paint::Color(color)
    }
}

/// Gradient paint used to fill or stroke paths with gradient.
#[derive(Copy, Clone, Debug)]
pub enum Gradient {
    Linear {
        start: (f32, f32),
        end: (f32, f32),
        start_color: Color,
        end_color: Color,
    },
    Box {
        position: (f32, f32),
        size: (f32, f32),
        radius: f32,
        feather: f32,
        start_color: Color,
        end_color: Color,
    },
    Radial {
        center: (f32, f32),
        inner_radius: f32,
        outer_radius: f32,
        start_color: Color,
        end_color: Color,
    }
}

impl From<Gradient> for Paint {
    fn from(gradient: Gradient) -> Self {
        Paint::Gradient(gradient)
    }
}