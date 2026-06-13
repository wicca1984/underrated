use crate::css::values::{AngleDeg, TransformFn};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    pub m: [f32; 6], // [a, b, c, d, e, f]
}

impl Affine {
    #[inline]
    pub fn identity() -> Affine {
        Affine {
            m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn translate(tx: f32, ty: f32) -> Affine {
        Affine {
            m: [1.0, 0.0, 0.0, 1.0, tx, ty],
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Affine {
        Affine {
            m: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn rotate(deg: f32) -> Affine {
        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Affine {
            m: [cos, sin, -sin, cos, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn multiply(&self, rhs: &Affine) -> Affine {
        let a1 = self.m[0];
        let b1 = self.m[1];
        let c1 = self.m[2];
        let d1 = self.m[3];
        let e1 = self.m[4];
        let f1 = self.m[5];

        let a2 = rhs.m[0];
        let b2 = rhs.m[1];
        let c2 = rhs.m[2];
        let d2 = rhs.m[3];
        let e2 = rhs.m[4];
        let f2 = rhs.m[5];

        Affine {
            m: [
                a1 * a2 + c1 * b2,      // a
                b1 * a2 + d1 * b2,      // b
                a1 * c2 + c1 * d2,      // c
                b1 * c2 + d1 * d2,      // d
                a1 * e2 + c1 * f2 + e1, // e
                b1 * e2 + d1 * f2 + f1, // f
            ],
        }
    }

    #[inline]
    pub fn apply_point(&self, x: f32, y: f32) -> (f32, f32) {
        let a = self.m[0];
        let b = self.m[1];
        let c = self.m[2];
        let d = self.m[3];
        let e = self.m[4];
        let f = self.m[5];

        (a * x + c * y + e, b * x + d * y + f)
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        let identity = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        self.m
            .iter()
            .zip(identity.iter())
            .all(|(&val, &id)| (val - id).abs() < 1e-6)
    }

    pub fn from_transform_fns(fns: &[TransformFn]) -> Affine {
        let mut result = Affine::identity();
        for func in fns {
            let next = match func {
                TransformFn::Scale { x, y } => Affine::scale(*x, *y),
                TransformFn::ScaleX(x) => Affine::scale(*x, 1.0),
                TransformFn::ScaleY(y) => Affine::scale(1.0, *y),
                TransformFn::Rotate(AngleDeg(deg)) => Affine::rotate(*deg),
                TransformFn::Translate { .. }
                | TransformFn::TranslateX(_)
                | TransformFn::TranslateY(_) => {
                    // TODO(spec): unify translate into the matrix
                    Affine::identity()
                }
            };
            result = result.multiply(&next);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::values::{LengthOrPercent, LengthUnit};

    #[test]
    fn test_identity_t0506() {
        let identity = Affine::identity();
        assert!(identity.is_identity());
        let p = identity.apply_point(3.0, 4.0);
        assert_eq!(p, (3.0, 4.0));
    }

    #[test]
    fn test_scale_t0506() {
        let s = Affine::scale(2.0, 3.0);
        let p = s.apply_point(1.0, 1.0);
        assert_eq!(p, (2.0, 3.0));
        assert!(!s.is_identity());
    }

    #[test]
    fn test_rotate_t0506() {
        let got = Affine::rotate(90.0).apply_point(1.0, 0.0);
        assert!(got.0.abs() < 1e-4, "expected x ≈ 0, got {}", got.0);
        assert!((got.1 - 1.0).abs() < 1e-4, "expected y ≈ 1, got {}", got.1);
    }

    #[test]
    fn test_multiply_order_t0506() {
        let scale = Affine::scale(2.0, 2.0);
        let translate = Affine::translate(1.0, 1.0);
        let combined = scale.multiply(&translate);
        let p = combined.apply_point(0.0, 0.0);
        assert_eq!(p, (2.0, 2.0));
    }

    #[test]
    fn test_from_transform_fns_t0506() {
        let fns = vec![TransformFn::Scale { x: 2.0, y: 3.0 }];
        let p1 = Affine::from_transform_fns(&fns).apply_point(1.0, 1.0);
        assert_eq!(p1, (2.0, 3.0));

        let fns2 = vec![
            TransformFn::Scale { x: 2.0, y: 2.0 },
            TransformFn::Rotate(AngleDeg(90.0)),
        ];
        let got = Affine::from_transform_fns(&fns2).apply_point(1.0, 0.0);
        assert!(got.0.abs() < 1e-4, "expected x ≈ 0, got {}", got.0);
        assert!((got.1 - 2.0).abs() < 1e-4, "expected y ≈ 2, got {}", got.1);

        // Test Translate variant contributes identity
        let fns3 = vec![
            TransformFn::Translate {
                x: LengthOrPercent {
                    value: 10.0,
                    unit: LengthUnit::Px,
                },
                y: LengthOrPercent {
                    value: 20.0,
                    unit: LengthUnit::Px,
                },
            },
            TransformFn::Scale { x: 3.0, y: 3.0 },
        ];
        let p3 = Affine::from_transform_fns(&fns3).apply_point(2.0, 2.0);
        assert_eq!(p3, (6.0, 6.0));
    }
}
