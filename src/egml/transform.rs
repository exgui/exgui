/// Represents a transformation in 2D space.
///
/// A transformation is a combination of translation (aka. position), skew and scale **or**
/// translation and rotation; implemented as a column-major matrix in the following form:
/// **[a c e]** - indices [0 2 4]
/// **[b d f]** - indices [1 3 5]
/// **[0 0 1]** - only theoretical / does not really exist. Logically it is always [0 0 1].
// TODO: need add transformation methods
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub matrix: [f32; 6],
    /// Controls whether paths or texts that gets transformed by this Transform
    /// are drawn in absolute coordinate space or coordinate space relative to the one
    /// previously active (relative positioning is default)
    /// This is just flag to tell drawing functions to use this Transform for drawing,
    /// it does not modify the underlying matrix.
    pub absolute: bool,
}

impl Transform {
    /// Construct a new transform with an identity matrix.
    pub fn new() -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            absolute: false,
        }
    }

    /// Set flag on this transform to use it in absolute coordinate space. Only applies to text.
    pub fn absolute(mut self) -> Self {
        self.absolute = true;
        self
    }

    /// Set flag on this transform to use it in local (relative) coordinate space. Only applies to text.
    pub fn relative(mut self) -> Self {
        self.absolute = false;
        self
    }

    /// Set the translation of the transform.
    pub fn with_translation(mut self, x: f32, y: f32) -> Self {
        *self.translate(x, y)
    }

    /// Set the scale of the transform.
    pub fn with_scale(mut self, x: f32, y: f32) -> Self {
        self.matrix[0] = x;
        self.matrix[3] = y;
        self
    }

    /// Set the skew of the transform.
    pub fn with_skew(mut self, x: f32, y: f32) -> Self {
        self.matrix[2] = x;
        self.matrix[1] = y;
        self
    }

    /// Set the rotation of the transform.
    pub fn with_rotation(mut self, theta: f32) -> Self {
        *self.rotate(theta)
    }

    pub fn translate(&mut self, x: f32, y: f32) -> &mut Self {
        self.matrix[4] = x;
        self.matrix[5] = y;
        self
    }

    pub fn translate_add(&mut self, x: f32, y: f32) -> &mut Self {
        self.matrix[4] += x;
        self.matrix[5] += y;
        self
    }

    pub fn rotate(&mut self, theta: f32) -> &mut Self {
        self.matrix[0] = theta.cos();
        self.matrix[2] = -theta.sin();
        self.matrix[1] = theta.sin();
        self.matrix[3] = theta.cos();
        self
    }
}
