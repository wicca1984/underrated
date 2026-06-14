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
    pub fn translate_x(tx: f32) -> Affine {
        Affine {
            m: [1.0, 0.0, 0.0, 1.0, tx, 0.0],
        }
    }

    #[inline]
    pub fn translate_y(ty: f32) -> Affine {
        Affine {
            m: [1.0, 0.0, 0.0, 1.0, 0.0, ty],
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32) -> Affine {
        Affine {
            m: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn scale_x(sx: f32) -> Affine {
        Affine {
            m: [sx, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn scale_y(sy: f32) -> Affine {
        Affine {
            m: [1.0, 0.0, 0.0, sy, 0.0, 0.0],
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
    pub fn skew_x(deg_x: f32) -> Affine {
        let rad_x = deg_x.to_radians();
        Affine {
            m: [1.0, 0.0, rad_x.tan(), 1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn skew_y(deg_y: f32) -> Affine {
        let rad_y = deg_y.to_radians();
        Affine {
            m: [1.0, rad_y.tan(), 0.0, 1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn skew(deg_x: f32, deg_y: f32) -> Affine {
        let rad_x = deg_x.to_radians();
        let rad_y = deg_y.to_radians();
        Affine {
            m: [1.0, rad_y.tan(), rad_x.tan(), 1.0, 0.0, 0.0],
        }
    }

    #[inline]
    pub fn matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Affine {
        Affine {
            m: [a, b, c, d, e, f],
        }
    }

    // TODO(spec): The following 3D transform functions are not supported in 2D Affine layout:
    // - translate3d(tx, ty, tz)
    // - translateZ(tz)
    // - scale3d(sx, sy, sz)
    // - scaleZ(sz)
    // - rotate3d(rx, ry, rz, deg)
    // - rotateX(deg)
    // - rotateY(deg)
    // - rotateZ(deg) (same as rotate(deg) in 2D)
    // - matrix3d(...)

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
                TransformFn::ScaleX(x) => Affine::scale_x(*x),
                TransformFn::ScaleY(y) => Affine::scale_y(*y),
                TransformFn::Rotate(AngleDeg(deg)) => Affine::rotate(*deg),
                TransformFn::Matrix(m) => Affine { m: *m },
                TransformFn::Translate { x, y } => {
                    Affine::translate(resolve_length(x), resolve_length(y))
                }
                TransformFn::TranslateX(x) => Affine::translate_x(resolve_length(x)),
                TransformFn::TranslateY(y) => Affine::translate_y(resolve_length(y)),
            };
            result = result.multiply(&next);
        }
        result
    }
}

fn resolve_length(lp: &crate::css::values::LengthOrPercent) -> f32 {
    use crate::css::values::LengthUnit;
    match lp.unit {
        LengthUnit::Px => lp.value,
        LengthUnit::Em => lp.value * 16.0,
        LengthUnit::Rem => lp.value * 16.0,
        LengthUnit::Pt => lp.value * 96.0 / 72.0,
        LengthUnit::Percent => 0.0, // TODO(spec): percentage translation requires layout box reference size
        _ => lp.value,
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
    fn test_translate_constructors() {
        let t = Affine::translate(10.0, 20.0);
        let p = t.apply_point(0.0, 0.0);
        assert_eq!(p, (10.0, 20.0));

        let tx = Affine::translate_x(5.0);
        let p_x = tx.apply_point(2.0, 3.0);
        assert_eq!(p_x, (7.0, 3.0));

        let ty = Affine::translate_y(-4.0);
        let p_y = ty.apply_point(2.0, 3.0);
        assert_eq!(p_y, (2.0, -1.0));
    }

    #[test]
    fn test_scale_t0506() {
        let s = Affine::scale(2.0, 3.0);
        let p = s.apply_point(1.0, 1.0);
        assert_eq!(p, (2.0, 3.0));
        assert!(!s.is_identity());
    }

    #[test]
    fn test_scale_constructors() {
        let sx = Affine::scale_x(4.0);
        let p_x = sx.apply_point(2.0, 3.0);
        assert_eq!(p_x, (8.0, 3.0));

        let sy = Affine::scale_y(0.5);
        let p_y = sy.apply_point(2.0, 4.0);
        assert_eq!(p_y, (2.0, 2.0));
    }

    #[test]
    fn test_rotate_t0506() {
        let got = Affine::rotate(90.0).apply_point(1.0, 0.0);
        assert!(got.0.abs() < 1e-4, "expected x ≈ 0, got {}", got.0);
        assert!((got.1 - 1.0).abs() < 1e-4, "expected y ≈ 1, got {}", got.1);
    }

    #[test]
    fn test_skew_constructors() {
        // skewX(45deg) -> tan(45) = 1
        let sx = Affine::skew_x(45.0);
        let p_x = sx.apply_point(1.0, 2.0); // x' = 1 + 2 * 1 = 3, y' = 2
        assert!((p_x.0 - 3.0).abs() < 1e-4, "expected x ≈ 3, got {}", p_x.0);
        assert!((p_x.1 - 2.0).abs() < 1e-4, "expected y ≈ 2, got {}", p_x.1);

        // skewY(45deg) -> tan(45) = 1
        let sy = Affine::skew_y(45.0);
        let p_y = sy.apply_point(2.0, 1.0); // x' = 2, y' = 1 + 2 * 1 = 3
        assert!((p_y.0 - 2.0).abs() < 1e-4, "expected x ≈ 2, got {}", p_y.0);
        assert!((p_y.1 - 3.0).abs() < 1e-4, "expected y ≈ 3, got {}", p_y.1);

        // skew(45deg, 45deg)
        let sk = Affine::skew(45.0, 45.0);
        let p_sk = sk.apply_point(2.0, 3.0); // x' = 2 + 3 * 1 = 5, y' = 3 + 2 * 1 = 5
        assert!(
            (p_sk.0 - 5.0).abs() < 1e-4,
            "expected x ≈ 5, got {}",
            p_sk.0
        );
        assert!(
            (p_sk.1 - 5.0).abs() < 1e-4,
            "expected y ≈ 5, got {}",
            p_sk.1
        );
    }

    #[test]
    fn test_matrix_constructor() {
        let m = Affine::matrix(2.0, 1.0, -1.0, 3.0, 5.0, 7.0);
        assert_eq!(m.m, [2.0, 1.0, -1.0, 3.0, 5.0, 7.0]);
        let p = m.apply_point(1.0, 1.0); // x' = 2*1 - 1*1 + 5 = 6, y' = 1*1 + 3*1 + 7 = 11
        assert_eq!(p, (6.0, 11.0));
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

        // Test Translate variant is correctly unified and composed
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
        assert_eq!(p3, (16.0, 26.0));
    }

    #[test]
    fn test_matrix_to_affine_t0508() {
        let fns = vec![TransformFn::Matrix([2.0, 0.0, 0.0, 3.0, 5.0, 7.0])];
        let aff = Affine::from_transform_fns(&fns);
        assert_eq!(aff.m, [2.0, 0.0, 0.0, 3.0, 5.0, 7.0]);
    }
}
