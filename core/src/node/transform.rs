/// Represents a transformation in 2D space.
///
/// A transformation is a combination of translation (aka. position), skew and scale **or**
/// translation and rotation; implemented as a column-major matrix in the following form:
/// **[a c e]** - indices [0 2 4]
/// **[b d f]** - indices [1 3 5]
/// **[0 0 1]** - only theoretical / does not really exist. Logically it is always [0 0 1].
// TODO: need add transformation methods
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// Implementation of multiplication Trait for Transform.
/// The order in which you multiplicate matters (you are multiplicating matrices)
impl std::ops::Mul for Transform {
    type Output = Transform;
    /// Multiplies transform with other transform (the order matters).
    fn mul(self, rhs: Transform) -> Self::Output {
        Transform {
            matrix: [
                self.matrix[0] * rhs.matrix[0] + self.matrix[2] * rhs.matrix[1],
                self.matrix[1] * rhs.matrix[0] + self.matrix[3] * rhs.matrix[1],
                self.matrix[0] * rhs.matrix[2] + self.matrix[2] * rhs.matrix[3],
                self.matrix[1] * rhs.matrix[2] + self.matrix[3] * rhs.matrix[3],
                self.matrix[0] * rhs.matrix[4] + self.matrix[2] * rhs.matrix[5] + self.matrix[4],
                self.matrix[1] * rhs.matrix[4] + self.matrix[3] * rhs.matrix[5] + self.matrix[5],
            ],
            absolute: self.absolute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! trans_eq_bool {
        ($t1:expr, $t2:expr) => {
            $t1.matrix[0] == $t2.matrix[0] &&
            $t1.matrix[1] == $t2.matrix[1] &&
            $t1.matrix[2] == $t2.matrix[2] &&
            $t1.matrix[3] == $t2.matrix[3] &&
            $t1.matrix[4] == $t2.matrix[4] &&
            $t1.matrix[5] == $t2.matrix[5] &&
            $t1.absolute == $t2.absolute
        };
    }

    macro_rules! trans_eq {
        ($t1:expr, $t2:expr) => {
            assert!(trans_eq_bool!($t1, $t2))
        };
    }

    macro_rules! trans_not_eq {
        ($t1:expr, $t2:expr) => {
            assert!(!trans_eq_bool!($t1, $t2))
        };
    }

    #[test]
    fn test_transform() {
        // Contructors
        trans_eq!(Transform::new(), Transform {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            absolute: false,
        });

        trans_eq!(Transform::new().with_translation(11.1, 22.2), Transform {
            matrix: [1.0, 0.0, 0.0, 1.0, 11.1, 22.2],
            absolute: false,
        });

        trans_eq!(Transform::new().with_scale(11.1, 22.2), Transform {
            matrix: [11.1, 0.0, 0.0, 22.2, 0.0, 0.0],
            absolute: false,
        });

        trans_eq!(Transform::new().with_skew(11.1, 22.2), Transform {
            matrix: [1.0, 22.2, 11.1, 1.0, 0.0, 0.0],
            absolute: false,
        });

        let angle = 90f32.to_radians();
        trans_eq!(Transform::new().with_rotation(angle), Transform {
            matrix: [angle.cos(), angle.sin(), -angle.sin(), angle.cos(), 0.0, 0.0],
            absolute: false,
        });

        // Multiplication
        let identity = Transform::new();
        let trans = Transform::new().with_translation(10.0, 20.0);
        trans_eq!(identity * trans, trans);
        trans_eq!(trans * identity, trans);
        trans_eq!(identity * identity, identity);
        let a = Transform::new().with_rotation(123.0);
        let b = Transform::new().with_skew(66.6, 1337.2);
        trans_not_eq!(a * b, b * a);
    }
}