use crate::Real;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transform {
    Local(TransformMatrix),
    Global(TransformMatrix),
    Calculated {
        local: Option<TransformMatrix>,
        global: TransformMatrix,
    },
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    /// Construct a new transform with an identity matrix.
    pub fn new() -> Self {
        Transform::Local(TransformMatrix::identity())
    }

    /// Set the translation of the transform.
    pub fn with_translation(mut self, x: Real, y: Real) -> Self {
        *self.translate(x, y)
    }

    /// Set the scale of the transform.
    pub fn with_scale(mut self, x: Real, y: Real) -> Self {
        *self.scale(x, y)
    }

    /// Set the skew of the transform.
    pub fn with_skew(mut self, x: Real, y: Real) -> Self {
        *self.skew(x, y)
    }

    /// Set the rotation of the transform.
    pub fn with_rotation(mut self, theta: Real) -> Self {
        *self.rotate(theta)
    }

    pub fn transform(&mut self, modifier: impl Fn(&mut TransformMatrix)) {
        match self {
            Transform::Local(matrix) | Transform::Global(matrix) => modifier(matrix),
            Transform::Calculated { local: Some(local), .. } => {
                modifier(local);
                *self = Transform::Local(*local);
            },
            Transform::Calculated { global, .. } => {
                modifier(global);
                *self = Transform::Global(*global);
            },
        }
    }

    pub fn translate(&mut self, x: Real, y: Real) -> &mut Self {
        self.transform(|matrix| {
            matrix.translate(x, y);
        });
        self
    }

    pub fn translate_add(&mut self, x: Real, y: Real) -> &mut Self {
        self.transform(|matrix| {
            matrix.translate_add(x, y);
        });
        self
    }

    pub fn rotate(&mut self, theta: Real) -> &mut Self {
        self.transform(|matrix| {
            matrix.rotate(theta);
        });
        self
    }

    pub fn scale(&mut self, x: Real, y: Real) -> &mut Self {
        self.transform(|matrix| {
            matrix.scale(x, y);
        });
        self
    }

    pub fn skew(&mut self, x: Real, y: Real) -> &mut Self {
        self.transform(|matrix| {
            matrix.skew(x, y);
        });
        self
    }

    pub fn is_absolute(&self) -> bool {
        match self {
            Transform::Global(_) | Transform::Calculated { local: None, .. } => true,
            _ => false,
        }
    }

    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    pub fn is_not_exist(&self) -> bool {
        self.is_absolute() && self.matrix().is_identity()
    }

    pub fn matrix(&self) -> TransformMatrix {
        self.local_matrix().or_else(|| self.global_matrix()).unwrap()
    }

    pub fn local_matrix(&self) -> Option<TransformMatrix> {
        match self {
            Transform::Local(local) | Transform::Calculated { local: Some(local), .. } => Some(*local),
            _ => None,
        }
    }

    pub fn global_matrix(&self) -> Option<TransformMatrix> {
        match self {
            Transform::Global(global) | Transform::Calculated { global, .. } => Some(*global),
            _ => None,
        }
    }

    pub fn calculated_matrix(&self) -> Option<TransformMatrix> {
        match self {
            Transform::Calculated { global, .. } => Some(*global),
            _ => None,
        }
    }

    pub fn calculate_global(&mut self, parent_global: TransformMatrix) -> TransformMatrix {
        let local = self.local_matrix();
        let global = local
            .map(|local| parent_global * local)
            .or_else(|| self.global_matrix())
            .unwrap();
        *self = Transform::Calculated { local, global };
        global
    }
}

/// Represents a transformation in 2D space.
///
/// A transformation is a combination of translation (aka. position), skew and scale **or**
/// translation and rotation; implemented as a column-major matrix in the following form:
/// **[a c e]** - indices [0 2 4]
/// **[b d f]** - indices [1 3 5]
/// **[0 0 1]** - only theoretical / does not really exist. Logically it is always [0 0 1].
// TODO: need add transformation methods
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformMatrix {
    pub matrix: [Real; 6],
}

impl TransformMatrix {
    /// Construct a new transform matrix as an identity.
    pub fn identity() -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    /// Set the translation of the transform.
    pub fn with_translation(mut self, x: Real, y: Real) -> Self {
        *self.translate(x, y)
    }

    /// Set the scale of the transform.
    pub fn with_scale(mut self, x: Real, y: Real) -> Self {
        *self.scale(x, y)
    }

    /// Set the skew of the transform.
    pub fn with_skew(mut self, x: Real, y: Real) -> Self {
        *self.skew(x, y)
    }

    /// Set the rotation of the transform.
    pub fn with_rotation(mut self, theta: Real) -> Self {
        *self.rotate(theta)
    }

    pub fn translate(&mut self, x: Real, y: Real) -> &mut Self {
        self.matrix[4] = x;
        self.matrix[5] = y;
        self
    }

    pub fn translate_add(&mut self, x: Real, y: Real) -> &mut Self {
        self.matrix[4] += x;
        self.matrix[5] += y;
        self
    }

    pub fn translate_xy(&self) -> (Real, Real) {
        (self.matrix[4], self.matrix[5])
    }

    pub fn rotate(&mut self, theta: Real) -> &mut Self {
        self.matrix[0] = theta.cos();
        self.matrix[2] = -theta.sin();
        self.matrix[1] = theta.sin();
        self.matrix[3] = theta.cos();
        self
    }

    pub fn scale(&mut self, x: Real, y: Real) -> &mut Self {
        self.matrix[0] = x;
        self.matrix[3] = y;
        self
    }

    pub fn scale_xy(&self) -> (Real, Real) {
        (self.matrix[0], self.matrix[3])
    }

    pub fn skew(&mut self, x: Real, y: Real) -> &mut Self {
        self.matrix[2] = x;
        self.matrix[1] = y;
        self
    }

    pub fn inverse(mut self) -> Self {
        let inv_det = 1.0 / (self.matrix[0] * self.matrix[3] - self.matrix[2] * self.matrix[1]);
        self.matrix[0] = self.matrix[3] * inv_det;
        self.matrix[1] = -self.matrix[1] * inv_det;
        self.matrix[2] = -self.matrix[2] * inv_det;
        self.matrix[3] = self.matrix[0] * inv_det;
        self.matrix[4] = (self.matrix[2] * self.matrix[5] - self.matrix[3] * self.matrix[4]) * inv_det;
        self.matrix[5] = (self.matrix[1] * self.matrix[4] - self.matrix[0] * self.matrix[5]) * inv_det;
        self
    }

    pub fn is_identity(&self) -> bool {
        self.matrix == [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    }
}

/// Implementation of multiplication Trait for Transform.
/// The order in which you multiplicate matters (you are multiplicating matrices)
impl std::ops::Mul for TransformMatrix {
    type Output = TransformMatrix;

    /// Multiplies transform with other transform (the order matters).
    fn mul(self, rhs: TransformMatrix) -> Self::Output {
        TransformMatrix {
            matrix: [
                self.matrix[0] * rhs.matrix[0] + self.matrix[2] * rhs.matrix[1],
                self.matrix[1] * rhs.matrix[0] + self.matrix[3] * rhs.matrix[1],
                self.matrix[0] * rhs.matrix[2] + self.matrix[2] * rhs.matrix[3],
                self.matrix[1] * rhs.matrix[2] + self.matrix[3] * rhs.matrix[3],
                self.matrix[0] * rhs.matrix[4] + self.matrix[2] * rhs.matrix[5] + self.matrix[4],
                self.matrix[1] * rhs.matrix[4] + self.matrix[3] * rhs.matrix[5] + self.matrix[5],
            ],
        }
    }
}

impl std::ops::Mul<(Real, Real)> for TransformMatrix {
    type Output = (Real, Real);

    /// Multiplies transform with other transform (the order matters).
    fn mul(self, (x, y): (Real, Real)) -> Self::Output {
        (
            self.matrix[0] * x + self.matrix[2] * y + self.matrix[4],
            self.matrix[1] * x + self.matrix[3] * y + self.matrix[5],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! trans_eq_bool {
        ($t1:expr, $t2:expr) => {
            $t1.matrix[0] == $t2.matrix[0]
                && $t1.matrix[1] == $t2.matrix[1]
                && $t1.matrix[2] == $t2.matrix[2]
                && $t1.matrix[3] == $t2.matrix[3]
                && $t1.matrix[4] == $t2.matrix[4]
                && $t1.matrix[5] == $t2.matrix[5]
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
        trans_eq!(TransformMatrix::identity(), TransformMatrix {
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        });

        trans_eq!(
            TransformMatrix::identity().with_translation(11.1, 22.2),
            TransformMatrix {
                matrix: [1.0, 0.0, 0.0, 1.0, 11.1, 22.2],
            }
        );

        trans_eq!(TransformMatrix::identity().with_scale(11.1, 22.2), TransformMatrix {
            matrix: [11.1, 0.0, 0.0, 22.2, 0.0, 0.0],
        });

        trans_eq!(TransformMatrix::identity().with_skew(11.1, 22.2), TransformMatrix {
            matrix: [1.0, 22.2, 11.1, 1.0, 0.0, 0.0],
        });

        let angle = 90_f32.to_radians();
        trans_eq!(TransformMatrix::identity().with_rotation(angle), TransformMatrix {
            matrix: [angle.cos(), angle.sin(), -angle.sin(), angle.cos(), 0.0, 0.0],
        });

        // Multiplication
        let identity = TransformMatrix::identity();
        let trans = TransformMatrix::identity().with_translation(10.0, 20.0);
        trans_eq!(identity * trans, trans);
        trans_eq!(trans * identity, trans);
        trans_eq!(identity * identity, identity);
        let a = TransformMatrix::identity().with_rotation(123.0);
        let b = TransformMatrix::identity().with_skew(66.6, 1337.2);
        trans_not_eq!(a * b, b * a);
    }
}
