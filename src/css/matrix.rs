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
    pub fn pre_multiply(&self, lhs: &Affine) -> Affine {
        lhs.multiply(self)
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        let a = self.m[0];
        let b = self.m[1];
        let c = self.m[2];
        let d = self.m[3];
        a * d - b * c
    }

    #[inline]
    pub fn is_invertible(&self) -> bool {
        self.determinant().abs() >= 1e-12
    }

    #[inline]
    pub fn invert(&self) -> Option<Affine> {
        let det = self.determinant();
        if det.abs() < 1e-12 {
            return None;
        }
        let inv_det = 1.0 / det;
        let a = self.m[0];
        let b = self.m[1];
        let c = self.m[2];
        let d = self.m[3];
        let e = self.m[4];
        let f = self.m[5];

        Some(Affine {
            m: [
                d * inv_det,
                -b * inv_det,
                -c * inv_det,
                a * inv_det,
                (c * f - d * e) * inv_det,
                (b * e - a * f) * inv_det,
            ],
        })
    }

    #[inline]
    pub fn multiply_3d(&self, rhs: &Matrix3d) -> Matrix3d {
        Matrix3d::from(*self).multiply(rhs)
    }

    #[inline]
    pub fn translated(&self, tx: f32, ty: f32) -> Affine {
        self.multiply(&Affine::translate(tx, ty))
    }

    #[inline]
    pub fn scaled(&self, sx: f32, sy: f32) -> Affine {
        self.multiply(&Affine::scale(sx, sy))
    }

    #[inline]
    pub fn rotated(&self, deg: f32) -> Affine {
        self.multiply(&Affine::rotate(deg))
    }

    #[inline]
    pub fn skewed_x(&self, deg_x: f32) -> Affine {
        self.multiply(&Affine::skew_x(deg_x))
    }

    #[inline]
    pub fn skewed_y(&self, deg_y: f32) -> Affine {
        self.multiply(&Affine::skew_y(deg_y))
    }

    #[inline]
    pub fn skewed(&self, deg_x: f32, deg_y: f32) -> Affine {
        self.multiply(&Affine::skew(deg_x, deg_y))
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

    /// Composes a sequence of `Affine` matrices from left to right.
    pub fn compose(list: &[Affine]) -> Self {
        let mut result = Self::identity();
        for m in list {
            result = result.multiply(m);
        }
        result
    }

    /// Parses a CSS transform list string into an Affine matrix.
    /// Returns None if parsing fails, or if any 3D function is present.
    pub fn parse(s: &str) -> Option<Self> {
        Matrix3d::parse(s)?.to_2d()
    }
}

impl std::str::FromStr for Affine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
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

fn parse_number_arg(s: &str) -> Option<f32> {
    s.trim().parse::<f32>().ok()
}

fn parse_angle_arg(s: &str) -> Option<f32> {
    let s = s.trim().to_ascii_lowercase();
    if s == "0" {
        return Some(0.0);
    }
    if let Some(stripped) = s.strip_suffix("deg") {
        stripped.trim().parse::<f32>().ok()
    } else if let Some(stripped) = s.strip_suffix("grad") {
        let grad = stripped.trim().parse::<f32>().ok()?;
        Some(grad * 0.9)
    } else if let Some(stripped) = s.strip_suffix("rad") {
        let rad = stripped.trim().parse::<f32>().ok()?;
        Some(rad.to_degrees())
    } else if let Some(stripped) = s.strip_suffix("turn") {
        let turn = stripped.trim().parse::<f32>().ok()?;
        Some(turn * 360.0)
    } else {
        s.parse::<f32>().ok()
    }
}

fn parse_length_arg(s: &str) -> Option<f32> {
    let s = s.trim().to_ascii_lowercase();
    if s == "0" {
        return Some(0.0);
    }
    if let Some(stripped) = s.strip_suffix("px") {
        stripped.trim().parse::<f32>().ok()
    } else if let Some(stripped) = s.strip_suffix("rem") {
        let val = stripped.trim().parse::<f32>().ok()?;
        Some(val * 16.0)
    } else if let Some(stripped) = s.strip_suffix("em") {
        let val = stripped.trim().parse::<f32>().ok()?;
        Some(val * 16.0)
    } else if let Some(stripped) = s.strip_suffix("pt") {
        let val = stripped.trim().parse::<f32>().ok()?;
        Some(val * 96.0 / 72.0)
    } else if s.ends_with('%') {
        Some(0.0)
    } else {
        s.parse::<f32>().ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3d {
    pub m: [f32; 16],
}

impl Matrix3d {
    #[inline]
    pub fn identity() -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn translate(tx: f32, ty: f32, tz: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, tx, 0.0, 1.0, 0.0, ty, 0.0, 0.0, 1.0, tz, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn translate_x(tx: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, tx, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn translate_y(ty: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, ty, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn translate_z(tz: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, tz, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        Matrix3d {
            m: [
                sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, sz, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn scale_x(sx: f32) -> Self {
        Matrix3d {
            m: [
                sx, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn scale_y(sy: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn scale_z(sz: f32) -> Self {
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, sz, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn skew_x(deg_x: f32) -> Self {
        let rad_x = deg_x.to_radians();
        Matrix3d {
            m: [
                1.0,
                rad_x.tan(),
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    #[inline]
    pub fn skew_y(deg_y: f32) -> Self {
        let rad_y = deg_y.to_radians();
        Matrix3d {
            m: [
                1.0,
                0.0,
                0.0,
                0.0,
                rad_y.tan(),
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    #[inline]
    pub fn skew(deg_x: f32, deg_y: f32) -> Self {
        let rad_x = deg_x.to_radians();
        let rad_y = deg_y.to_radians();
        Matrix3d {
            m: [
                1.0,
                rad_x.tan(),
                0.0,
                0.0,
                rad_y.tan(),
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    #[inline]
    pub fn from_col_major(m: [f32; 16]) -> Self {
        Matrix3d {
            m: [
                m[0], m[4], m[8], m[12], m[1], m[5], m[9], m[13], m[2], m[6], m[10], m[14], m[3],
                m[7], m[11], m[15],
            ],
        }
    }

    #[inline]
    pub fn rotate_x(deg: f32) -> Self {
        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Matrix3d {
            m: [
                1.0, 0.0, 0.0, 0.0, 0.0, cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn rotate_y(deg: f32) -> Self {
        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Matrix3d {
            m: [
                cos, 0.0, sin, 0.0, 0.0, 1.0, 0.0, 0.0, -sin, 0.0, cos, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    #[inline]
    pub fn rotate_z(deg: f32) -> Self {
        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        Matrix3d {
            m: [
                cos, -sin, 0.0, 0.0, sin, cos, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn rotate_3d(rx: f32, ry: f32, rz: f32, deg: f32) -> Self {
        let len_sq = rx * rx + ry * ry + rz * rz;
        if len_sq < 1e-12 {
            return Self::identity();
        }
        let len = len_sq.sqrt();
        let ux = rx / len;
        let uy = ry / len;
        let uz = rz / len;

        let rad = deg.to_radians();
        let cos = rad.cos();
        let sin = rad.sin();
        let t = 1.0 - cos;

        Matrix3d {
            m: [
                t * ux * ux + cos,
                t * ux * uy - sin * uz,
                t * ux * uz + sin * uy,
                0.0,
                t * ux * uy + sin * uz,
                t * uy * uy + cos,
                t * uy * uz - sin * ux,
                0.0,
                t * ux * uz - sin * uy,
                t * uy * uz + sin * ux,
                t * uz * uz + cos,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        }
    }

    #[inline]
    pub fn perspective(d: f32) -> Self {
        let mut m = Self::identity();
        if d > 0.0 {
            m.m[14] = -1.0 / d; // row 3, col 2: w' gets -z/d
        }
        m
    }

    #[inline]
    pub fn multiply(&self, rhs: &Matrix3d) -> Self {
        let mut out = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[r * 4 + k] * rhs.m[k * 4 + c];
                }
                out[r * 4 + c] = sum;
            }
        }
        Matrix3d { m: out }
    }

    #[inline]
    pub fn pre_multiply(&self, lhs: &Matrix3d) -> Self {
        lhs.multiply(self)
    }

    #[inline]
    pub fn translated(&self, tx: f32, ty: f32, tz: f32) -> Matrix3d {
        self.multiply(&Matrix3d::translate(tx, ty, tz))
    }

    #[inline]
    pub fn scaled(&self, sx: f32, sy: f32, sz: f32) -> Matrix3d {
        self.multiply(&Matrix3d::scale(sx, sy, sz))
    }

    #[inline]
    pub fn rotated_x(&self, deg: f32) -> Matrix3d {
        self.multiply(&Matrix3d::rotate_x(deg))
    }

    #[inline]
    pub fn rotated_y(&self, deg: f32) -> Matrix3d {
        self.multiply(&Matrix3d::rotate_y(deg))
    }

    #[inline]
    pub fn rotated_z(&self, deg: f32) -> Matrix3d {
        self.multiply(&Matrix3d::rotate_z(deg))
    }

    #[inline]
    pub fn rotated_3d(&self, rx: f32, ry: f32, rz: f32, deg: f32) -> Matrix3d {
        self.multiply(&Matrix3d::rotate_3d(rx, ry, rz, deg))
    }

    #[inline]
    pub fn skewed_x(&self, deg_x: f32) -> Matrix3d {
        self.multiply(&Matrix3d::skew_x(deg_x))
    }

    #[inline]
    pub fn skewed_y(&self, deg_y: f32) -> Matrix3d {
        self.multiply(&Matrix3d::skew_y(deg_y))
    }

    #[inline]
    pub fn skewed(&self, deg_x: f32, deg_y: f32) -> Matrix3d {
        self.multiply(&Matrix3d::skew(deg_x, deg_y))
    }

    #[inline]
    pub fn perspectived(&self, d: f32) -> Matrix3d {
        self.multiply(&Matrix3d::perspective(d))
    }

    #[inline]
    pub fn apply_point_3d(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let px = self.m[0] * x + self.m[1] * y + self.m[2] * z + self.m[3];
        let py = self.m[4] * x + self.m[5] * y + self.m[6] * z + self.m[7];
        let pz = self.m[8] * x + self.m[9] * y + self.m[10] * z + self.m[11];
        let pw = self.m[12] * x + self.m[13] * y + self.m[14] * z + self.m[15];

        if pw != 0.0 && pw != 1.0 {
            (px / pw, py / pw, pz / pw)
        } else {
            (px, py, pz)
        }
    }

    #[inline]
    pub fn apply_point(&self, x: f32, y: f32) -> (f32, f32) {
        let p = self.apply_point_3d(x, y, 0.0);
        (p.0, p.1)
    }

    #[inline]
    pub fn is_identity(&self) -> bool {
        let identity = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        self.m
            .iter()
            .zip(identity.iter())
            .all(|(&val, &id)| (val - id).abs() < 1e-6)
    }

    #[inline]
    pub fn determinant(&self) -> f32 {
        det4x4(&self.m)
    }

    #[inline]
    pub fn is_invertible(&self) -> bool {
        self.determinant().abs() >= 1e-12
    }

    #[inline]
    pub fn invert(&self) -> Option<Matrix3d> {
        inverse4x4(&self.m).map(|m| Matrix3d { m })
    }

    #[inline]
    pub fn is_2d(&self) -> bool {
        self.to_2d().is_some()
    }

    #[inline]
    pub fn multiply_affine(&self, rhs: &Affine) -> Matrix3d {
        self.multiply(&Matrix3d::from(*rhs))
    }

    pub fn to_2d(&self) -> Option<Affine> {
        let eps = 1e-5;
        if self.m[2].abs() < eps && // m02
           self.m[6].abs() < eps && // m12
           self.m[8].abs() < eps && // m20
           self.m[9].abs() < eps && // m21
           (self.m[10] - 1.0).abs() < eps && // m22
           self.m[11].abs() < eps && // m23
           self.m[12].abs() < eps && // m30
           self.m[13].abs() < eps && // m31
           self.m[14].abs() < eps && // m32
           (self.m[15] - 1.0).abs() < eps
        {
            // m33
            Some(Affine {
                m: [
                    self.m[0], // a
                    self.m[4], // b
                    self.m[1], // c
                    self.m[5], // d
                    self.m[3], // e
                    self.m[7], // f
                ],
            })
        } else {
            None
        }
    }

    pub fn from_transform_fns(fns: &[TransformFn]) -> Self {
        let mut result = Matrix3d::identity();
        for func in fns {
            let next = match func {
                TransformFn::Scale { x, y } => Matrix3d::scale(*x, *y, 1.0),
                TransformFn::ScaleX(x) => Matrix3d::scale_x(*x),
                TransformFn::ScaleY(y) => Matrix3d::scale_y(*y),
                TransformFn::Rotate(AngleDeg(deg)) => Matrix3d::rotate_z(*deg),
                TransformFn::Matrix(m) => Matrix3d::from(Affine { m: *m }),
                TransformFn::Translate { x, y } => {
                    Matrix3d::translate(resolve_length(x), resolve_length(y), 0.0)
                }
                TransformFn::TranslateX(x) => Matrix3d::translate_x(resolve_length(x)),
                TransformFn::TranslateY(y) => Matrix3d::translate_y(resolve_length(y)),
            };
            result = result.multiply(&next);
        }
        result
    }

    /// Composes a sequence of `Matrix3d` matrices from left to right.
    pub fn compose(list: &[Matrix3d]) -> Self {
        let mut result = Self::identity();
        for m in list {
            result = result.multiply(m);
        }
        result
    }

    /// Parses a CSS transform list string into a Matrix3d matrix.
    /// Returns None if parsing fails.
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            return Some(Matrix3d::identity());
        }

        let mut result = Matrix3d::identity();
        let mut chars = trimmed.char_indices().peekable();

        while let Some(&(i, c)) = chars.peek() {
            if c.is_whitespace() || c == ',' {
                chars.next();
                continue;
            }

            // Parse identifier (function name)
            let start = i;
            let mut end = start;
            while let Some(&(idx, ch)) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    chars.next();
                    end = idx + ch.len_utf8();
                } else {
                    break;
                }
            }

            if start == end {
                return None; // Expected function name
            }

            let func_name = &trimmed[start..end];

            // Skip whitespace
            while let Some((_, ch)) = chars.peek() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            // Expect '('
            if let Some((_, '(')) = chars.next() {
                // Read arguments inside matching parentheses
                let arg_start = match chars.peek() {
                    Some(&(idx, _)) => idx,
                    None => return None,
                };

                let mut paren_count = 1;
                let mut arg_end = arg_start;

                for (idx, ch) in chars.by_ref() {
                    if ch == '(' {
                        paren_count += 1;
                    } else if ch == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            arg_end = idx;
                            break;
                        }
                    }
                }

                if paren_count != 0 {
                    return None; // Unmatched parentheses
                }

                let args_str = &trimmed[arg_start..arg_end];
                // Split arguments by comma and/or whitespace
                let args: Vec<&str> = args_str
                    .split(',')
                    .map(|a| a.trim())
                    .filter(|a| !a.is_empty())
                    .collect();

                let next_matrix = match func_name.to_ascii_lowercase().as_str() {
                    "matrix" => {
                        if args.len() != 6 {
                            return None;
                        }
                        let a = parse_number_arg(args[0])?;
                        let b = parse_number_arg(args[1])?;
                        let c = parse_number_arg(args[2])?;
                        let d = parse_number_arg(args[3])?;
                        let e = parse_number_arg(args[4])?;
                        let f = parse_number_arg(args[5])?;
                        Matrix3d::from(Affine {
                            m: [a, b, c, d, e, f],
                        })
                    }
                    "matrix3d" => {
                        if args.len() != 16 {
                            return None;
                        }
                        let mut m = [0.0; 16];
                        for (idx, arg) in args.iter().enumerate() {
                            m[idx] = parse_number_arg(arg)?;
                        }
                        Matrix3d::from_col_major(m)
                    }
                    "translate" => {
                        if args.len() == 1 {
                            let tx = parse_length_arg(args[0])?;
                            Matrix3d::translate(tx, 0.0, 0.0)
                        } else if args.len() == 2 {
                            let tx = parse_length_arg(args[0])?;
                            let ty = parse_length_arg(args[1])?;
                            Matrix3d::translate(tx, ty, 0.0)
                        } else {
                            return None;
                        }
                    }
                    "translate3d" => {
                        if args.len() != 3 {
                            return None;
                        }
                        let tx = parse_length_arg(args[0])?;
                        let ty = parse_length_arg(args[1])?;
                        let tz = parse_length_arg(args[2])?;
                        Matrix3d::translate(tx, ty, tz)
                    }
                    "translatex" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let tx = parse_length_arg(args[0])?;
                        Matrix3d::translate_x(tx)
                    }
                    "translatey" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let ty = parse_length_arg(args[0])?;
                        Matrix3d::translate_y(ty)
                    }
                    "translatez" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let tz = parse_length_arg(args[0])?;
                        Matrix3d::translate_z(tz)
                    }
                    "scale" => {
                        if args.len() == 1 {
                            let s = parse_number_arg(args[0])?;
                            Matrix3d::scale(s, s, 1.0)
                        } else if args.len() == 2 {
                            let sx = parse_number_arg(args[0])?;
                            let sy = parse_number_arg(args[1])?;
                            Matrix3d::scale(sx, sy, 1.0)
                        } else {
                            return None;
                        }
                    }
                    "scale3d" => {
                        if args.len() != 3 {
                            return None;
                        }
                        let sx = parse_number_arg(args[0])?;
                        let sy = parse_number_arg(args[1])?;
                        let sz = parse_number_arg(args[2])?;
                        Matrix3d::scale(sx, sy, sz)
                    }
                    "scalex" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let sx = parse_number_arg(args[0])?;
                        Matrix3d::scale_x(sx)
                    }
                    "scaley" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let sy = parse_number_arg(args[0])?;
                        Matrix3d::scale_y(sy)
                    }
                    "scalez" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let sz = parse_number_arg(args[0])?;
                        Matrix3d::scale_z(sz)
                    }
                    "rotate" | "rotatez" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let deg = parse_angle_arg(args[0])?;
                        Matrix3d::rotate_z(deg)
                    }
                    "rotatex" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let deg = parse_angle_arg(args[0])?;
                        Matrix3d::rotate_x(deg)
                    }
                    "rotatey" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let deg = parse_angle_arg(args[0])?;
                        Matrix3d::rotate_y(deg)
                    }
                    "rotate3d" => {
                        if args.len() != 4 {
                            return None;
                        }
                        let rx = parse_number_arg(args[0])?;
                        let ry = parse_number_arg(args[1])?;
                        let rz = parse_number_arg(args[2])?;
                        let deg = parse_angle_arg(args[3])?;
                        Matrix3d::rotate_3d(rx, ry, rz, deg)
                    }
                    "skew" => {
                        if args.len() == 1 {
                            let deg_x = parse_angle_arg(args[0])?;
                            Matrix3d::skew_x(deg_x)
                        } else if args.len() == 2 {
                            let deg_x = parse_angle_arg(args[0])?;
                            let deg_y = parse_angle_arg(args[1])?;
                            Matrix3d::skew(deg_x, deg_y)
                        } else {
                            return None;
                        }
                    }
                    "skewx" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let deg_x = parse_angle_arg(args[0])?;
                        Matrix3d::skew_x(deg_x)
                    }
                    "skewy" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let deg_y = parse_angle_arg(args[0])?;
                        Matrix3d::skew_y(deg_y)
                    }
                    "perspective" => {
                        if args.len() != 1 {
                            return None;
                        }
                        let d = parse_length_arg(args[0])?;
                        Matrix3d::perspective(d)
                    }
                    _ => return None,
                };

                result = result.multiply(&next_matrix);
            } else {
                return None;
            }
        }

        Some(result)
    }
}

impl std::str::FromStr for Matrix3d {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl From<Affine> for Matrix3d {
    fn from(a: Affine) -> Self {
        Matrix3d {
            m: [
                a.m[0], a.m[2], 0.0, a.m[4], a.m[1], a.m[3], 0.0, a.m[5], 0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decomposed2d {
    pub translate: (f32, f32),
    pub scale: (f32, f32),
    pub angle: f32, // degrees
    pub skew: f32,  // skew factor (tan of skew angle)
}

impl Decomposed2d {
    pub fn recompose(&self) -> Affine {
        let translate = Affine::translate(self.translate.0, self.translate.1);
        let rotate = Affine::rotate(self.angle);
        let skew_aff = Affine {
            m: [1.0, 0.0, self.skew, 1.0, 0.0, 0.0],
        };
        let scale = Affine::scale(self.scale.0, self.scale.1);

        translate
            .multiply(&rotate)
            .multiply(&skew_aff)
            .multiply(&scale)
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let translate = (
            self.translate.0 + (other.translate.0 - self.translate.0) * t,
            self.translate.1 + (other.translate.1 - self.translate.1) * t,
        );
        let scale = (
            self.scale.0 + (other.scale.0 - self.scale.0) * t,
            self.scale.1 + (other.scale.1 - self.scale.1) * t,
        );
        let skew = self.skew + (other.skew - self.skew) * t;

        // Angle interpolation using shortest path
        let mut diff = (other.angle - self.angle) % 360.0;
        if diff > 180.0 {
            diff -= 360.0;
        } else if diff < -180.0 {
            diff += 360.0;
        }
        let angle = self.angle + diff * t;

        Decomposed2d {
            translate,
            scale,
            angle,
            skew,
        }
    }
}

impl Affine {
    pub fn decompose(&self) -> Option<Decomposed2d> {
        let a = self.m[0];
        let b = self.m[1];
        let c = self.m[2];
        let d = self.m[3];
        let e = self.m[4];
        let f = self.m[5];

        let translate = (e, f);

        let mut scale_x = (a * a + b * b).sqrt();
        let mut scale_y = (c * c + d * d).sqrt();

        let det = a * d - b * c;
        if det.abs() < 1e-12 {
            return None;
        }

        if det < 0.0 {
            if a < d {
                scale_x = -scale_x;
            } else {
                scale_y = -scale_y;
            }
        }

        let norm_a = if scale_x.abs() >= 1e-12 {
            a / scale_x
        } else {
            0.0
        };
        let norm_b = if scale_x.abs() >= 1e-12 {
            b / scale_x
        } else {
            0.0
        };
        let norm_c = if scale_y.abs() >= 1e-12 {
            c / scale_y
        } else {
            0.0
        };
        let norm_d = if scale_y.abs() >= 1e-12 {
            d / scale_y
        } else {
            0.0
        };

        let rad = norm_b.atan2(norm_a);
        let angle = rad.to_degrees();

        let sn = (-rad).sin();
        let cs = (-rad).cos();
        let row1x = cs * norm_c - sn * norm_d;
        let row1y = sn * norm_c + cs * norm_d;

        let skew = if row1y.abs() >= 1e-12 {
            row1x / row1y
        } else {
            0.0
        };
        let scale_y = scale_y * row1y;

        Some(Decomposed2d {
            translate,
            scale: (scale_x, scale_y),
            angle,
            skew,
        })
    }

    /// Interpolate this matrix with `other` by progress `t` [0.0, 1.0].
    /// If either matrix cannot be decomposed, we fall back to linear interpolation
    /// of each component of the matrix directly.
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        if let (Some(d1), Some(d2)) = (self.decompose(), other.decompose()) {
            d1.interpolate(&d2, t).recompose()
        } else {
            let mut m = [0.0; 6];
            for (i, val) in m.iter_mut().enumerate() {
                *val = self.m[i] + (other.m[i] - self.m[i]) * t;
            }
            Affine { m }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decomposed3d {
    pub translate: (f32, f32, f32),
    pub scale: (f32, f32, f32),
    pub skew: (f32, f32, f32),             // xy, xz, yz
    pub perspective: (f32, f32, f32, f32), // px, py, pz, pw
    pub quaternion: (f32, f32, f32, f32),  // x, y, z, w
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalize(&self) -> Self {
        let l = self.len();
        if l < 1e-12 {
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            Vec3 {
                x: self.x / l,
                y: self.y / l,
                z: self.z / l,
            }
        }
    }

    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn combine(&self, other: &Vec3, ascl: f32, bscl: f32) -> Vec3 {
        Vec3 {
            x: ascl * self.x + bscl * other.x,
            y: ascl * self.y + bscl * other.y,
            z: ascl * self.z + bscl * other.z,
        }
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn det3x3(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32, i: f32) -> f32 {
    a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
}

fn det4x4(m: &[f32; 16]) -> f32 {
    let sub0 = det3x3(m[5], m[6], m[7], m[9], m[10], m[11], m[13], m[14], m[15]);
    let sub1 = det3x3(m[4], m[6], m[7], m[8], m[10], m[11], m[12], m[14], m[15]);
    let sub2 = det3x3(m[4], m[5], m[7], m[8], m[9], m[11], m[12], m[13], m[15]);
    let sub3 = det3x3(m[4], m[5], m[6], m[8], m[9], m[10], m[12], m[13], m[14]);

    m[0] * sub0 - m[1] * sub1 + m[2] * sub2 - m[3] * sub3
}

fn submatrix3x3(m: &[f32; 16], r: usize, c: usize) -> [f32; 9] {
    let mut out = [0.0; 9];
    let mut out_idx = 0;
    for row in 0..4 {
        if row == r {
            continue;
        }
        for col in 0..4 {
            if col == c {
                continue;
            }
            out[out_idx] = m[row * 4 + col];
            out_idx += 1;
        }
    }
    out
}

fn cofactor(m: &[f32; 16], r: usize, c: usize) -> f32 {
    let sub = submatrix3x3(m, r, c);
    let d = det3x3(
        sub[0], sub[1], sub[2], sub[3], sub[4], sub[5], sub[6], sub[7], sub[8],
    );
    if (r + c) % 2 == 1 { -d } else { d }
}

fn inverse4x4(m: &[f32; 16]) -> Option<[f32; 16]> {
    let det = det4x4(m);
    if det.abs() < 1e-12 {
        return None;
    }
    let mut adj = [0.0; 16];
    for r in 0..4 {
        for c in 0..4 {
            adj[c * 4 + r] = cofactor(m, r, c);
        }
    }
    for val in adj.iter_mut() {
        *val /= det;
    }
    Some(adj)
}

impl Matrix3d {
    pub fn decompose(&self) -> Option<Decomposed3d> {
        let mut t = [0.0; 16];
        for r in 0..4 {
            for c in 0..4 {
                t[r * 4 + c] = self.m[c * 4 + r];
            }
        }

        let t15 = t[15];
        if t15.abs() < 1e-12 {
            return None;
        }
        for val in t.iter_mut() {
            *val /= t15;
        }

        let mut perspective_matrix = t;
        perspective_matrix[3] = 0.0;
        perspective_matrix[7] = 0.0;
        perspective_matrix[11] = 0.0;
        perspective_matrix[15] = 1.0;

        if det4x4(&perspective_matrix).abs() < 1e-12 {
            return None;
        }

        let mut perspective = (0.0, 0.0, 0.0, 1.0);
        if t[3] != 0.0 || t[7] != 0.0 || t[11] != 0.0 {
            let right_hand_side = [t[3], t[7], t[11], t[15]];
            if let Some(inv_perspective_matrix) = inverse4x4(&perspective_matrix) {
                let mut trans_inv = [0.0; 16];
                for r in 0..4 {
                    for c in 0..4 {
                        trans_inv[r * 4 + c] = inv_perspective_matrix[c * 4 + r];
                    }
                }
                let px = trans_inv[0] * right_hand_side[0]
                    + trans_inv[1] * right_hand_side[1]
                    + trans_inv[2] * right_hand_side[2]
                    + trans_inv[3] * right_hand_side[3];
                let py = trans_inv[4] * right_hand_side[0]
                    + trans_inv[5] * right_hand_side[1]
                    + trans_inv[6] * right_hand_side[2]
                    + trans_inv[7] * right_hand_side[3];
                let pz = trans_inv[8] * right_hand_side[0]
                    + trans_inv[9] * right_hand_side[1]
                    + trans_inv[10] * right_hand_side[2]
                    + trans_inv[11] * right_hand_side[3];
                let pw = trans_inv[12] * right_hand_side[0]
                    + trans_inv[13] * right_hand_side[1]
                    + trans_inv[14] * right_hand_side[2]
                    + trans_inv[15] * right_hand_side[3];
                perspective = (px, py, pz, pw);
            } else {
                return None;
            }
        }

        let translate = (t[12], t[13], t[14]);

        let mut row0 = Vec3::new(t[0], t[1], t[2]);
        let mut row1 = Vec3::new(t[4], t[5], t[6]);
        let mut row2 = Vec3::new(t[8], t[9], t[10]);

        let mut scale_x = row0.len();
        row0 = row0.normalize();

        let mut skew_xy = row0.dot(&row1);
        row1 = row1.combine(&row0, 1.0, -skew_xy);

        let mut scale_y = row1.len();
        row1 = row1.normalize();
        if scale_y != 0.0 {
            skew_xy /= scale_y;
        }

        let mut skew_xz = row0.dot(&row2);
        row2 = row2.combine(&row0, 1.0, -skew_xz);

        let mut skew_yz = row1.dot(&row2);
        row2 = row2.combine(&row1, 1.0, -skew_yz);

        let mut scale_z = row2.len();
        row2 = row2.normalize();
        if scale_z != 0.0 {
            skew_xz /= scale_z;
            skew_yz /= scale_z;
        }

        let pdum3 = row1.cross(&row2);
        if row0.dot(&pdum3) < 0.0 {
            scale_x = -scale_x;
            scale_y = -scale_y;
            scale_z = -scale_z;
            row0.x = -row0.x;
            row0.y = -row0.y;
            row0.z = -row0.z;
            row1.x = -row1.x;
            row1.y = -row1.y;
            row1.z = -row1.z;
            row2.x = -row2.x;
            row2.y = -row2.y;
            row2.z = -row2.z;
        }

        let row00 = row0.x;
        let row01 = row0.y;
        let row02 = row0.z;
        let row10 = row1.x;
        let row11 = row1.y;
        let row12 = row1.z;
        let row20 = row2.x;
        let row21 = row2.y;
        let row22 = row2.z;

        let quaternion;
        let tr = row00 + row11 + row22;
        if tr > 0.0 {
            let s = 0.5 / (tr + 1.0).sqrt();
            quaternion = (
                (row21 - row12) * s,
                (row02 - row20) * s,
                (row10 - row01) * s,
                0.25 / s,
            );
        } else if row00 > row11 && row00 > row22 {
            let s = (1.0 + row00 - row11 - row22).max(0.0).sqrt() * 2.0;
            quaternion = (
                0.25 * s,
                (row01 + row10) / s,
                (row02 + row20) / s,
                (row21 - row12) / s,
            );
        } else if row11 > row22 {
            let s = (1.0 + row11 - row00 - row22).max(0.0).sqrt() * 2.0;
            quaternion = (
                (row01 + row10) / s,
                0.25 * s,
                (row12 + row21) / s,
                (row02 - row20) / s,
            );
        } else {
            let s = (1.0 + row22 - row00 - row11).max(0.0).sqrt() * 2.0;
            quaternion = (
                (row02 + row20) / s,
                (row12 + row21) / s,
                0.25 * s,
                (row10 - row01) / s,
            );
        }

        Some(Decomposed3d {
            translate,
            scale: (scale_x, scale_y, scale_z),
            skew: (skew_xy, skew_xz, skew_yz),
            perspective,
            quaternion,
        })
    }

    /// Interpolate this matrix with `other` by progress `t` [0.0, 1.0].
    /// If either matrix cannot be decomposed, we fall back to linear interpolation
    /// of each component of the matrix directly.
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        if let (Some(d1), Some(d2)) = (self.decompose(), other.decompose()) {
            d1.interpolate(&d2, t).recompose()
        } else {
            let mut m = [0.0; 16];
            for (i, val) in m.iter_mut().enumerate() {
                *val = self.m[i] + (other.m[i] - self.m[i]) * t;
            }
            Matrix3d { m }
        }
    }
}

impl Decomposed3d {
    pub fn recompose(&self) -> Matrix3d {
        let mut persp_m = Matrix3d::identity();
        persp_m.m[12] = self.perspective.0;
        persp_m.m[13] = self.perspective.1;
        persp_m.m[14] = self.perspective.2;
        persp_m.m[15] = self.perspective.3;

        let trans_m = Matrix3d::translate(self.translate.0, self.translate.1, self.translate.2);

        let qx = self.quaternion.0;
        let qy = self.quaternion.1;
        let qz = self.quaternion.2;
        let qw = self.quaternion.3;
        let len = (qx * qx + qy * qy + qz * qz + qw * qw).sqrt();
        let (qx, qy, qz, qw) = if len > 1e-12 {
            (qx / len, qy / len, qz / len, qw / len)
        } else {
            (0.0, 0.0, 0.0, 1.0)
        };
        let x2 = qx + qx;
        let y2 = qy + qy;
        let z2 = qz + qz;
        let xx = qx * x2;
        let xy = qx * y2;
        let xz = qx * z2;
        let yy = qy * y2;
        let yz = qy * z2;
        let zz = qz * z2;
        let wx = qw * x2;
        let wy = qw * y2;
        let wz = qw * z2;

        let rot_m = Matrix3d {
            m: [
                1.0 - (yy + zz),
                xy + wz,
                xz - wy,
                0.0,
                xy - wz,
                1.0 - (xx + zz),
                yz + wx,
                0.0,
                xz + wy,
                yz - wx,
                1.0 - (xx + yy),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ],
        };

        let mut skew_m = Matrix3d::identity();
        skew_m.m[1] = self.skew.0;
        skew_m.m[2] = self.skew.1;
        skew_m.m[6] = self.skew.2;

        let scale_m = Matrix3d::scale(self.scale.0, self.scale.1, self.scale.2);

        persp_m
            .multiply(&trans_m)
            .multiply(&rot_m)
            .multiply(&skew_m)
            .multiply(&scale_m)
    }

    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let translate = (
            self.translate.0 + (other.translate.0 - self.translate.0) * t,
            self.translate.1 + (other.translate.1 - self.translate.1) * t,
            self.translate.2 + (other.translate.2 - self.translate.2) * t,
        );
        let scale = (
            self.scale.0 + (other.scale.0 - self.scale.0) * t,
            self.scale.1 + (other.scale.1 - self.scale.1) * t,
            self.scale.2 + (other.scale.2 - self.scale.2) * t,
        );
        let skew = (
            self.skew.0 + (other.skew.0 - self.skew.0) * t,
            self.skew.1 + (other.skew.1 - self.skew.1) * t,
            self.skew.2 + (other.skew.2 - self.skew.2) * t,
        );
        let perspective = (
            self.perspective.0 + (other.perspective.0 - self.perspective.0) * t,
            self.perspective.1 + (other.perspective.1 - self.perspective.1) * t,
            self.perspective.2 + (other.perspective.2 - self.perspective.2) * t,
            self.perspective.3 + (other.perspective.3 - self.perspective.3) * t,
        );

        let q1 = self.quaternion;
        let q2 = other.quaternion;
        let mut dot = q1.0 * q2.0 + q1.1 * q2.1 + q1.2 * q2.2 + q1.3 * q2.3;
        dot = dot.clamp(-1.0, 1.0);

        let mut q2_adj = q2;
        if dot < 0.0 {
            dot = -dot;
            q2_adj = (-q2.0, -q2.1, -q2.2, -q2.3);
        }

        let quaternion = if dot > 0.9995 {
            // Linear interpolation + normalize
            let x = q1.0 + (q2_adj.0 - q1.0) * t;
            let y = q1.1 + (q2_adj.1 - q1.1) * t;
            let z = q1.2 + (q2_adj.2 - q1.2) * t;
            let w = q1.3 + (q2_adj.3 - q1.3) * t;
            let len = (x * x + y * y + z * z + w * w).sqrt();
            if len > 0.0 {
                (x / len, y / len, z / len, w / len)
            } else {
                q1
            }
        } else {
            // Spherical Linear Interpolation (Slerp)
            let theta_0 = dot.acos();
            let theta = theta_0 * t;
            let sin_theta = theta.sin();
            let sin_theta_0 = theta_0.sin();

            if sin_theta_0.abs() < 1e-6 {
                // Fallback to LERP if sin_theta_0 is somehow extremely small
                let x = q1.0 + (q2_adj.0 - q1.0) * t;
                let y = q1.1 + (q2_adj.1 - q1.1) * t;
                let z = q1.2 + (q2_adj.2 - q1.2) * t;
                let w = q1.3 + (q2_adj.3 - q1.3) * t;
                let len = (x * x + y * y + z * z + w * w).sqrt();
                if len > 0.0 {
                    (x / len, y / len, z / len, w / len)
                } else {
                    q1
                }
            } else {
                let s0 = (theta_0 - theta).sin() / sin_theta_0;
                let s1 = sin_theta / sin_theta_0;

                (
                    q1.0 * s0 + q2_adj.0 * s1,
                    q1.1 * s0 + q2_adj.1 * s1,
                    q1.2 * s0 + q2_adj.2 * s1,
                    q1.3 * s0 + q2_adj.3 * s1,
                )
            }
        };

        Decomposed3d {
            translate,
            scale,
            skew,
            perspective,
            quaternion,
        }
    }

    /// Converts this 3D decomposition to a 2D decomposition, if possible.
    /// Returns `None` if any 3D-specific transform features are non-identity.
    pub fn to_2d(&self) -> Option<Decomposed2d> {
        let eps = 1e-5;
        if self.perspective.0.abs() > eps
            || self.perspective.1.abs() > eps
            || self.perspective.2.abs() > eps
            || (self.perspective.3 - 1.0).abs() > eps
        {
            return None;
        }

        if self.translate.2.abs() > eps {
            return None;
        }

        if (self.scale.2 - 1.0).abs() > eps {
            return None;
        }

        if self.skew.1.abs() > eps || self.skew.2.abs() > eps {
            return None;
        }

        if self.quaternion.0.abs() > eps || self.quaternion.1.abs() > eps {
            return None;
        }

        let half_rad = self.quaternion.2.atan2(self.quaternion.3);
        let angle = (half_rad * 2.0).to_degrees();

        Some(Decomposed2d {
            translate: (self.translate.0, self.translate.1),
            scale: (self.scale.0, self.scale.1),
            angle,
            skew: self.skew.0,
        })
    }
}

impl From<Decomposed2d> for Decomposed3d {
    fn from(d: Decomposed2d) -> Self {
        let half_rad = (d.angle / 2.0).to_radians();
        let qz = half_rad.sin();
        let qw = half_rad.cos();

        Decomposed3d {
            translate: (d.translate.0, d.translate.1, 0.0),
            scale: (d.scale.0, d.scale.1, 1.0),
            skew: (d.skew, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, qz, qw),
        }
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

    #[test]
    fn test_matrix3d_identity() {
        let m = Matrix3d::identity();
        assert!(m.is_identity());
        let p = m.apply_point_3d(1.0, 2.0, 3.0);
        assert_eq!(p, (1.0, 2.0, 3.0));
    }

    #[test]
    fn test_matrix3d_translate_scale() {
        let t = Matrix3d::translate(1.0, 2.0, 3.0);
        let s = Matrix3d::scale(2.0, 3.0, 4.0);
        let combined = s.multiply(&t); // apply t first, then s

        let p1 = combined.apply_point_3d(0.0, 0.0, 0.0);
        assert_eq!(p1, (2.0, 6.0, 12.0));

        let p2 = combined.apply_point(0.0, 0.0);
        assert_eq!(p2, (2.0, 6.0));
    }

    #[test]
    fn test_matrix3d_rotations() {
        // Rotate Z 90 deg: (1, 0, 0) -> (0, 1, 0)
        let rz = Matrix3d::rotate_z(90.0);
        let p = rz.apply_point_3d(1.0, 0.0, 0.0);
        assert!(p.0.abs() < 1e-5);
        assert!((p.1 - 1.0).abs() < 1e-5);
        assert_eq!(p.2, 0.0);

        // Rotate X 90 deg: (0, 1, 0) -> (0, 0, 1)
        let rx = Matrix3d::rotate_x(90.0);
        let p = rx.apply_point_3d(0.0, 1.0, 0.0);
        assert_eq!(p.0, 0.0);
        assert!(p.1.abs() < 1e-5);
        assert!((p.2 - 1.0).abs() < 1e-5);

        // Rotate Y 90 deg: (0, 0, 1) -> (1, 0, 0)
        let ry = Matrix3d::rotate_y(90.0);
        let p = ry.apply_point_3d(0.0, 0.0, 1.0);
        assert!((p.0 - 1.0).abs() < 1e-5);
        assert_eq!(p.1, 0.0);
        assert!(p.2.abs() < 1e-5);

        // Rotate 3D around (1, 1, 1) normalized
        let r3d = Matrix3d::rotate_3d(1.0, 1.0, 1.0, 120.0);
        let p = r3d.apply_point_3d(1.0, 0.0, 0.0);
        assert!(p.0.abs() < 1e-5);
        assert!((p.1 - 1.0).abs() < 1e-5);
        assert!(p.2.abs() < 1e-5);
    }

    #[test]
    fn test_matrix3d_perspective() {
        let p_mat = Matrix3d::perspective(10.0);
        // perspective(10) on (0, 0, -5.0) -> w' = -(-5)/10 + 1 = 1.5
        // apply_point_3d of (2.0, 3.0, -5.0) -> (2/1.5, 3/1.5, -5/1.5)
        let p = p_mat.apply_point_3d(2.0, 3.0, -5.0);
        assert!((p.0 - 2.0 / 1.5).abs() < 1e-5);
        assert!((p.1 - 3.0 / 1.5).abs() < 1e-5);
        assert!((p.2 - -5.0 / 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_matrix3d_conversions() {
        let aff = Affine::scale(2.0, 3.0).multiply(&Affine::translate(4.0, 5.0));
        let m3d = Matrix3d::from(aff);
        let aff_back = m3d.to_2d().unwrap();
        assert_eq!(aff, aff_back);

        let non_2d = Matrix3d::translate(0.0, 0.0, 5.0);
        assert!(non_2d.to_2d().is_none());
    }

    #[test]
    fn test_decompose_2d() {
        let orig = Affine::translate(10.0, 20.0)
            .multiply(&Affine::rotate(45.0))
            .multiply(&Affine::skew_x(30.0))
            .multiply(&Affine::scale(2.0, 3.0));

        let decomp = orig.decompose().unwrap();
        assert!((decomp.translate.0 - 10.0).abs() < 1e-5);
        assert!((decomp.translate.1 - 20.0).abs() < 1e-5);
        assert!((decomp.scale.0 - 2.0).abs() < 1e-5);
        assert!((decomp.scale.1 - 3.0).abs() < 1e-5);
        assert!((decomp.angle - 45.0).abs() < 1e-5);
        assert!((decomp.skew - 30.0f32.to_radians().tan()).abs() < 1e-5);

        let recomposed = decomp.recompose();
        for i in 0..6 {
            assert!((orig.m[i] - recomposed.m[i]).abs() < 1e-4);
        }
    }

    #[test]
    fn test_decompose_3d() {
        let orig = Matrix3d::translate(10.0, 20.0, 30.0)
            .multiply(&Matrix3d::rotate_y(45.0))
            .multiply(&Matrix3d::scale(2.0, 3.0, 4.0));

        let decomp = orig.decompose().unwrap();
        assert!((decomp.translate.0 - 10.0).abs() < 1e-5);
        assert!((decomp.translate.1 - 20.0).abs() < 1e-5);
        assert!((decomp.translate.2 - 30.0).abs() < 1e-5);
        assert!((decomp.scale.0 - 2.0).abs() < 1e-5);
        assert!((decomp.scale.1 - 3.0).abs() < 1e-5);
        assert!((decomp.scale.2 - 4.0).abs() < 1e-5);

        let recomposed = decomp.recompose();
        for i in 0..16 {
            assert!((orig.m[i] - recomposed.m[i]).abs() < 1e-4);
        }
    }

    #[test]
    fn test_matrix3d_new_constructors_t0918() {
        // translate_x/y/z
        let tx = Matrix3d::translate_x(5.0);
        assert_eq!(tx.apply_point_3d(1.0, 1.0, 1.0), (6.0, 1.0, 1.0));

        let ty = Matrix3d::translate_y(6.0);
        assert_eq!(ty.apply_point_3d(1.0, 1.0, 1.0), (1.0, 7.0, 1.0));

        let tz = Matrix3d::translate_z(7.0);
        assert_eq!(tz.apply_point_3d(1.0, 1.0, 1.0), (1.0, 1.0, 8.0));

        // scale_x/y/z
        let sx = Matrix3d::scale_x(2.0);
        assert_eq!(sx.apply_point_3d(1.0, 2.0, 3.0), (2.0, 2.0, 3.0));

        let sy = Matrix3d::scale_y(3.0);
        assert_eq!(sy.apply_point_3d(1.0, 2.0, 3.0), (1.0, 6.0, 3.0));

        let sz = Matrix3d::scale_z(4.0);
        assert_eq!(sz.apply_point_3d(1.0, 2.0, 3.0), (1.0, 2.0, 12.0));

        // skew_x/y/skew
        let skx = Matrix3d::skew_x(45.0);
        let p_skx = skx.apply_point_3d(1.0, 2.0, 3.0); // x' = 1 + 2 * 1 = 3, y' = 2, z' = 3
        assert!((p_skx.0 - 3.0).abs() < 1e-4);
        assert!((p_skx.1 - 2.0).abs() < 1e-4);
        assert!((p_skx.2 - 3.0).abs() < 1e-4);

        let sky = Matrix3d::skew_y(45.0);
        let p_sky = sky.apply_point_3d(2.0, 1.0, 3.0); // x' = 2, y' = 1 + 2 * 1 = 3, z' = 3
        assert!((p_sky.0 - 2.0).abs() < 1e-4);
        assert!((p_sky.1 - 3.0).abs() < 1e-4);
        assert!((p_sky.2 - 3.0).abs() < 1e-4);

        let sk = Matrix3d::skew(45.0, 45.0);
        let p_sk = sk.apply_point_3d(2.0, 3.0, 4.0); // x' = 2 + 3 * 1 = 5, y' = 3 + 2 * 1 = 5, z' = 4
        assert!((p_sk.0 - 5.0).abs() < 1e-4);
        assert!((p_sk.1 - 5.0).abs() < 1e-4);
        assert!((p_sk.2 - 4.0).abs() < 1e-4);

        // matrix3d constructor
        let m3d = Matrix3d::from_col_major([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);
        assert_eq!(
            m3d.m,
            [
                1.0, 5.0, 9.0, 13.0, 2.0, 6.0, 10.0, 14.0, 3.0, 7.0, 11.0, 15.0, 4.0, 8.0, 12.0,
                16.0,
            ]
        );
    }

    #[test]
    fn test_matrix3d_from_transform_fns_direct_t0918() {
        let fns = vec![
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
        let aff = Affine::from_transform_fns(&fns);
        let m3d = Matrix3d::from_transform_fns(&fns);
        let aff_back = m3d.to_2d().unwrap();
        assert_eq!(aff, aff_back);
    }

    #[test]
    fn test_affine_invert_and_determinant_t0955() {
        // Identity matrix
        let identity = Affine::identity();
        assert_eq!(identity.determinant(), 1.0);
        assert!(identity.is_invertible());
        let inv_identity = identity.invert().unwrap();
        assert!(inv_identity.is_identity());

        // Standard scaling/translation
        let scale_trans = Affine::scale(2.0, 3.0).multiply(&Affine::translate(4.0, 5.0));
        assert_eq!(scale_trans.determinant(), 6.0);
        assert!(scale_trans.is_invertible());
        let inv = scale_trans.invert().unwrap();
        let back = scale_trans.multiply(&inv);
        assert!(back.is_identity());

        // Singular matrix
        let singular = Affine::matrix(1.0, 2.0, 2.0, 4.0, 5.0, 6.0);
        assert_eq!(singular.determinant(), 0.0);
        assert!(!singular.is_invertible());
        assert!(singular.invert().is_none());
    }

    #[test]
    fn test_matrix3d_invert_and_determinant_t0955() {
        // Identity matrix
        let identity = Matrix3d::identity();
        assert_eq!(identity.determinant(), 1.0);
        assert!(identity.is_invertible());
        let inv_identity = identity.invert().unwrap();
        assert!(inv_identity.is_identity());

        // Standard 3D translation & scale
        let t = Matrix3d::translate(2.0, 3.0, 4.0);
        let s = Matrix3d::scale(2.0, 0.5, 5.0);
        let orig = t.multiply(&s);
        assert_eq!(orig.determinant(), 5.0);
        assert!(orig.is_invertible());
        let inv = orig.invert().unwrap();
        let back = orig.multiply(&inv);
        assert!(back.is_identity());

        // Singular 3D matrix (row-major of scale with 0)
        let singular = Matrix3d::scale(1.0, 0.0, 1.0);
        assert_eq!(singular.determinant(), 0.0);
        assert!(!singular.is_invertible());
        assert!(singular.invert().is_none());
    }

    #[test]
    fn test_is_2d_t0955() {
        let m2d = Matrix3d::from(Affine::scale(2.0, 3.0));
        assert!(m2d.is_2d());

        let m3d = Matrix3d::translate(0.0, 0.0, 5.0);
        assert!(!m3d.is_2d());
    }

    #[test]
    fn test_multiply_helpers_t0955() {
        let aff = Affine::scale(2.0, 3.0);
        let m3d = Matrix3d::translate(0.0, 0.0, 5.0);

        let res1 = aff.multiply_3d(&m3d);
        let expected1 = Matrix3d::from(aff).multiply(&m3d);
        assert_eq!(res1, expected1);

        let res2 = m3d.multiply_affine(&aff);
        let expected2 = m3d.multiply(&Matrix3d::from(aff));
        assert_eq!(res2, expected2);
    }

    #[test]
    fn test_instance_composition_and_pre_multiply_t0955() {
        // Test Affine pre_multiply
        let a = Affine::scale(2.0, 3.0);
        let b = Affine::translate(10.0, 20.0);
        let pm = a.pre_multiply(&b); // translate, then scale (b.multiply(a))
        let expected_pm = b.multiply(&a);
        assert_eq!(pm, expected_pm);

        // Test Affine instance composition
        let aff_comp = Affine::identity()
            .translated(10.0, 20.0)
            .scaled(2.0, 3.0)
            .rotated(90.0)
            .skewed_x(45.0)
            .skewed_y(45.0)
            .skewed(30.0, 30.0);
        let expected_aff_comp = Affine::identity()
            .multiply(&Affine::translate(10.0, 20.0))
            .multiply(&Affine::scale(2.0, 3.0))
            .multiply(&Affine::rotate(90.0))
            .multiply(&Affine::skew_x(45.0))
            .multiply(&Affine::skew_y(45.0))
            .multiply(&Affine::skew(30.0, 30.0));
        assert_eq!(aff_comp, expected_aff_comp);

        // Test Matrix3d pre_multiply
        let m1 = Matrix3d::scale(2.0, 3.0, 4.0);
        let m2 = Matrix3d::translate(10.0, 20.0, 30.0);
        let m3d_pm = m1.pre_multiply(&m2);
        let expected_m3d_pm = m2.multiply(&m1);
        assert_eq!(m3d_pm, expected_m3d_pm);

        // Test Matrix3d instance composition
        let m3d_comp = Matrix3d::identity()
            .translated(10.0, 20.0, 30.0)
            .scaled(2.0, 3.0, 4.0)
            .rotated_x(90.0)
            .rotated_y(90.0)
            .rotated_z(90.0)
            .rotated_3d(1.0, 1.0, 1.0, 120.0)
            .skewed_x(45.0)
            .skewed_y(45.0)
            .skewed(30.0, 30.0)
            .perspectived(10.0);
        let expected_m3d_comp = Matrix3d::identity()
            .multiply(&Matrix3d::translate(10.0, 20.0, 30.0))
            .multiply(&Matrix3d::scale(2.0, 3.0, 4.0))
            .multiply(&Matrix3d::rotate_x(90.0))
            .multiply(&Matrix3d::rotate_y(90.0))
            .multiply(&Matrix3d::rotate_z(90.0))
            .multiply(&Matrix3d::rotate_3d(1.0, 1.0, 1.0, 120.0))
            .multiply(&Matrix3d::skew_x(45.0))
            .multiply(&Matrix3d::skew_y(45.0))
            .multiply(&Matrix3d::skew(30.0, 30.0))
            .multiply(&Matrix3d::perspective(10.0));
        assert_eq!(m3d_comp, expected_m3d_comp);
    }

    #[test]
    fn test_interpolate_2d_decomposed_t0974() {
        let d1 = Decomposed2d {
            translate: (10.0, 20.0),
            scale: (1.0, 2.0),
            angle: 10.0,
            skew: 0.1,
        };
        let d2 = Decomposed2d {
            translate: (20.0, 40.0),
            scale: (3.0, 4.0),
            angle: 350.0,
            skew: 0.5,
        };

        let res = d1.interpolate(&d2, 0.5);
        assert!((res.translate.0 - 15.0).abs() < 1e-5);
        assert!((res.translate.1 - 30.0).abs() < 1e-5);
        assert!((res.scale.0 - 2.0).abs() < 1e-5);
        assert!((res.scale.1 - 3.0).abs() < 1e-5);
        assert!((res.skew - 0.3).abs() < 1e-5);
        assert!(res.angle.abs() < 1e-5);

        let start = d1.interpolate(&d2, 0.0);
        assert!((start.angle - 10.0).abs() < 1e-5);
        let end = d1.interpolate(&d2, 1.0);
        assert!((end.angle - -10.0).abs() < 1e-5 || (end.angle - 350.0).abs() < 1e-5);
    }

    #[test]
    fn test_interpolate_2d_affine_t0974() {
        let a1 = Affine::translate(10.0, 20.0).multiply(&Affine::scale(2.0, 2.0));
        let a2 = Affine::translate(30.0, 40.0).multiply(&Affine::scale(4.0, 4.0));

        let res = a1.interpolate(&a2, 0.5);
        let expected = Affine::translate(20.0, 30.0).multiply(&Affine::scale(3.0, 3.0));
        for i in 0..6 {
            assert!((res.m[i] - expected.m[i]).abs() < 1e-4);
        }

        let singular1 = Affine::matrix(1.0, 2.0, 2.0, 4.0, 0.0, 0.0);
        let singular2 = Affine::matrix(2.0, 4.0, 4.0, 8.0, 10.0, 10.0);

        let fallback_res = singular1.interpolate(&singular2, 0.5);
        let expected_fallback = Affine::matrix(1.5, 3.0, 3.0, 6.0, 5.0, 5.0);
        assert_eq!(fallback_res.m, expected_fallback.m);
    }

    #[test]
    fn test_interpolate_3d_decomposed_t0974() {
        let d1 = Decomposed3d {
            translate: (10.0, 20.0, 30.0),
            scale: (1.0, 2.0, 3.0),
            skew: (0.1, 0.2, 0.3),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.0),
        };
        let sin45 = 45.0f32.to_radians().sin();
        let cos45 = 45.0f32.to_radians().cos();
        let d2 = Decomposed3d {
            translate: (20.0, 40.0, 60.0),
            scale: (3.0, 4.0, 5.0),
            skew: (0.5, 0.6, 0.7),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, sin45, cos45),
        };

        let res = d1.interpolate(&d2, 0.5);
        assert!((res.translate.0 - 15.0).abs() < 1e-5);
        assert!((res.translate.1 - 30.0).abs() < 1e-5);
        assert!((res.translate.2 - 45.0).abs() < 1e-5);
        assert!((res.scale.0 - 2.0).abs() < 1e-5);
        assert!((res.scale.1 - 3.0).abs() < 1e-5);
        assert!((res.scale.2 - 4.0).abs() < 1e-5);

        let sin22_5 = 22.5f32.to_radians().sin();
        let cos22_5 = 22.5f32.to_radians().cos();
        assert!(res.quaternion.0.abs() < 1e-5);
        assert!(res.quaternion.1.abs() < 1e-5);
        assert!((res.quaternion.2 - sin22_5).abs() < 1e-5);
        assert!((res.quaternion.3 - cos22_5).abs() < 1e-5);

        let res_collinear = d1.interpolate(&d1, 0.5);
        assert_eq!(res_collinear.quaternion, d1.quaternion);
    }

    #[test]
    fn test_interpolate_3d_matrix_t0974() {
        let m1 = Matrix3d::translate(10.0, 20.0, 30.0).multiply(&Matrix3d::scale(2.0, 2.0, 2.0));
        let m2 = Matrix3d::translate(30.0, 40.0, 50.0).multiply(&Matrix3d::scale(4.0, 4.0, 4.0));

        let res = m1.interpolate(&m2, 0.5);
        let expected =
            Matrix3d::translate(20.0, 30.0, 40.0).multiply(&Matrix3d::scale(3.0, 3.0, 3.0));
        for i in 0..16 {
            assert!((res.m[i] - expected.m[i]).abs() < 1e-4);
        }

        let singular1 = Matrix3d::scale(1.0, 0.0, 1.0);
        let singular2 = Matrix3d::scale(3.0, 0.0, 5.0);
        let fallback_res = singular1.interpolate(&singular2, 0.5);
        let expected_fallback = Matrix3d::scale(2.0, 0.0, 3.0);
        assert_eq!(fallback_res.m, expected_fallback.m);
    }

    #[test]
    fn test_matrix_edge_cases_t1021() {
        // 1. Tiny non-zero vector normalization
        let tiny_vec = Vec3::new(1e-20, 1e-20, 1e-20);
        let normalized = tiny_vec.normalize();
        assert_eq!(normalized.x, 0.0);
        assert_eq!(normalized.y, 0.0);
        assert_eq!(normalized.z, 0.0);

        // 2. 2D decomposition with near-zero scale/skew factor checks
        let m2d = Affine::matrix(1e-15, 0.0, 0.0, 1e-15, 0.0, 0.0);
        // determinant is 1e-30, which is < 1e-12, so decomposition should return None
        assert!(m2d.decompose().is_none());

        // 3. Quaternion interpolation with dot > 1.0 clamping
        let d1 = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.00001), // Slightly unnormalized
        };
        let d2 = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.00001),
        };
        // This triggers slerp or lerp. With our clamp, it shouldn't produce NaN.
        let interpolated = d1.interpolate(&d2, 0.5);
        assert!(!interpolated.quaternion.3.is_nan());
        assert!((interpolated.quaternion.3 - 1.0).abs() < 1e-4);

        // 4. Recomposing a denormalized quaternion
        let denormalized = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 5.0), // Far from normalized
        };
        let recomposed = denormalized.recompose();
        let expected_identity = Matrix3d::identity();
        for i in 0..16 {
            assert!((recomposed.m[i] - expected_identity.m[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_matrix_parsing_and_composition_t1063() {
        // 1. String parsing checks
        let m_none = Matrix3d::parse("none").unwrap();
        assert!(m_none.is_identity());

        let m_empty = Matrix3d::parse("   ").unwrap();
        assert!(m_empty.is_identity());

        let m_translate = Matrix3d::parse("translate(10px, 20px)").unwrap();
        assert_eq!(m_translate.apply_point(0.0, 0.0), (10.0, 20.0));

        // single arg translate
        let m_translate_single = Matrix3d::parse("translate(15px)").unwrap();
        assert_eq!(m_translate_single.apply_point(0.0, 0.0), (15.0, 0.0));

        let m_translate_em_rem = Matrix3d::parse("translate(2em, 3rem)").unwrap();
        assert_eq!(m_translate_em_rem.apply_point(0.0, 0.0), (32.0, 48.0));

        let m_translate_pt = Matrix3d::parse("translate(72pt, 0)").unwrap();
        assert_eq!(m_translate_pt.apply_point(0.0, 0.0), (96.0, 0.0));

        // translate3d
        let m_translate3d = Matrix3d::parse("translate3d(5px, 10px, 15px)").unwrap();
        assert_eq!(
            m_translate3d.apply_point_3d(0.0, 0.0, 0.0),
            (5.0, 10.0, 15.0)
        );

        // translateX / translateY / translateZ
        let m_sub_translates =
            Matrix3d::parse("translateX(1px) translateY(2px) translateZ(3px)").unwrap();
        assert_eq!(
            m_sub_translates.apply_point_3d(0.0, 0.0, 0.0),
            (1.0, 2.0, 3.0)
        );

        // scale / scale3d / scaleX / scaleY / scaleZ
        let m_scale = Matrix3d::parse("scale(2)").unwrap();
        assert_eq!(m_scale.apply_point(1.0, 1.0), (2.0, 2.0));

        let m_scale_double = Matrix3d::parse("scale(2, 3)").unwrap();
        assert_eq!(m_scale_double.apply_point(1.0, 1.0), (2.0, 3.0));

        let m_scale3d = Matrix3d::parse("scale3d(2, 3, 4)").unwrap();
        assert_eq!(m_scale3d.apply_point_3d(1.0, 1.0, 1.0), (2.0, 3.0, 4.0));

        let m_sub_scales = Matrix3d::parse("scaleX(1.5) scaleY(2.5) scaleZ(3.5)").unwrap();
        assert_eq!(m_sub_scales.apply_point_3d(1.0, 1.0, 1.0), (1.5, 2.5, 3.5));

        // rotate / rotateZ / rotateX / rotateY / rotate3d
        let m_rotate_deg = Matrix3d::parse("rotate(90deg)").unwrap();
        let p_deg = m_rotate_deg.apply_point(1.0, 0.0);
        assert!(p_deg.0.abs() < 1e-4);
        assert!((p_deg.1 - 1.0).abs() < 1e-4);

        let m_rotate_rad = Matrix3d::parse("rotate(1.5707963rad)").unwrap(); // approx 90deg
        let p_rad = m_rotate_rad.apply_point(1.0, 0.0);
        assert!(p_rad.0.abs() < 1e-3);
        assert!((p_rad.1 - 1.0).abs() < 1e-3);

        let m_rotate_grad = Matrix3d::parse("rotate(100grad)").unwrap(); // 90deg
        let p_grad = m_rotate_grad.apply_point(1.0, 0.0);
        assert!(p_grad.0.abs() < 1e-4);
        assert!((p_grad.1 - 1.0).abs() < 1e-4);

        let m_rotate_turn = Matrix3d::parse("rotate(0.25turn)").unwrap(); // 90deg
        let p_turn = m_rotate_turn.apply_point(1.0, 0.0);
        assert!(p_turn.0.abs() < 1e-4);
        assert!((p_turn.1 - 1.0).abs() < 1e-4);

        let m_rotate_3d = Matrix3d::parse("rotate3d(0, 0, 1, 90deg)").unwrap();
        let p_r3d = m_rotate_3d.apply_point(1.0, 0.0);
        assert!(p_r3d.0.abs() < 1e-4);
        assert!((p_r3d.1 - 1.0).abs() < 1e-4);

        // skew / skewX / skewY
        let m_skew_x = Matrix3d::parse("skewX(45deg)").unwrap();
        let p_skx = m_skew_x.apply_point(1.0, 2.0);
        assert!((p_skx.0 - 3.0).abs() < 1e-4);
        assert!((p_skx.1 - 2.0).abs() < 1e-4);

        let m_skew_y = Matrix3d::parse("skewY(45deg)").unwrap();
        let p_sky = m_skew_y.apply_point(2.0, 1.0);
        assert!((p_sky.0 - 2.0).abs() < 1e-4);
        assert!((p_sky.1 - 3.0).abs() < 1e-4);

        let m_skew = Matrix3d::parse("skew(45deg, 45deg)").unwrap();
        let p_sk = m_skew.apply_point(2.0, 3.0);
        assert!((p_sk.0 - 5.0).abs() < 1e-4);
        assert!((p_sk.1 - 5.0).abs() < 1e-4);

        // matrix / matrix3d
        let m_matrix = Matrix3d::parse("matrix(2.0, 1.0, -1.0, 3.0, 5.0, 7.0)").unwrap();
        assert_eq!(m_matrix.apply_point(1.0, 1.0), (6.0, 11.0));

        let m_matrix3d = Matrix3d::parse("matrix3d(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 30.0, 1.0)").unwrap();
        assert_eq!(m_matrix3d.apply_point_3d(1.0, 2.0, 3.0), (11.0, 22.0, 33.0));

        // perspective
        let m_perspective = Matrix3d::parse("perspective(100px)").unwrap();
        assert!((m_perspective.m[14] - -1.0 / 100.0).abs() < 1e-5);

        // multi-function list composition Order
        let m_list = Matrix3d::parse("translate(10px, 20px) scale(2)").unwrap();
        assert_eq!(m_list.apply_point(0.0, 0.0), (10.0, 20.0));
        assert_eq!(m_list.apply_point(1.0, 1.0), (12.0, 22.0));

        // malformed parsing inputs should return None
        assert!(Matrix3d::parse("translate(10px").is_none()); // unclosed paren
        assert!(Matrix3d::parse("unknown_func(10px)").is_none()); // invalid name
        assert!(Matrix3d::parse("translate(10px, 20px, 30px, 40px)").is_none()); // wrong args count
        assert!(Matrix3d::parse("scale(abc)").is_none()); // non-numeric arg

        // 2. FromStr parsing checks
        let parsed_aff: Affine = "translate(10px, 20px)".parse().unwrap();
        assert_eq!(parsed_aff.apply_point(0.0, 0.0), (10.0, 20.0));

        // Affine parse fails on 3D functions
        assert!("translate3d(10px, 20px, 30px)".parse::<Affine>().is_err());

        let parsed_m3d: Matrix3d = "translate3d(10px, 20px, 30px)".parse().unwrap();
        assert_eq!(parsed_m3d.apply_point_3d(0.0, 0.0, 0.0), (10.0, 20.0, 30.0));

        // 3. Compose sequence of matrices checks
        let aff_seq = vec![Affine::translate(10.0, 20.0), Affine::scale(2.0, 2.0)];
        let aff_composed = Affine::compose(&aff_seq);
        assert_eq!(aff_composed.apply_point(0.0, 0.0), (10.0, 20.0));
        assert_eq!(aff_composed.apply_point(1.0, 1.0), (12.0, 22.0));

        let m3d_seq = vec![
            Matrix3d::translate(10.0, 20.0, 30.0),
            Matrix3d::scale(2.0, 2.0, 2.0),
        ];
        let m3d_composed = Matrix3d::compose(&m3d_seq);
        assert_eq!(
            m3d_composed.apply_point_3d(0.0, 0.0, 0.0),
            (10.0, 20.0, 30.0)
        );
        assert_eq!(
            m3d_composed.apply_point_3d(1.0, 1.0, 1.0),
            (12.0, 22.0, 32.0)
        );

        // 4. Conversions between Decomposed2d and Decomposed3d checks
        let d2 = Decomposed2d {
            translate: (10.0, 20.0),
            scale: (2.0, 3.0),
            angle: 45.0,
            skew: 0.5,
        };
        let d3 = Decomposed3d::from(d2);
        assert_eq!(d3.translate, (10.0, 20.0, 0.0));
        assert_eq!(d3.scale, (2.0, 3.0, 1.0));
        assert_eq!(d3.skew, (0.5, 0.0, 0.0));

        let d2_back = d3.to_2d().unwrap();
        assert_eq!(d2_back.translate, d2.translate);
        assert_eq!(d2_back.scale, d2.scale);
        assert!((d2_back.angle - d2.angle).abs() < 1e-4);
        assert!((d2_back.skew - d2.skew).abs() < 1e-4);

        // A non-2D 3D decomposition should return None on to_2d()
        let d3_non2d_trans = Decomposed3d {
            translate: (0.0, 0.0, 5.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.0),
        };
        assert!(d3_non2d_trans.to_2d().is_none());

        let d3_non2d_scale = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 2.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.0),
        };
        assert!(d3_non2d_scale.to_2d().is_none());

        let d3_non2d_skew = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 1.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.0),
        };
        assert!(d3_non2d_skew.to_2d().is_none());

        let d3_non2d_perspective = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.1, 0.0, 0.0, 1.0),
            quaternion: (0.0, 0.0, 0.0, 1.0),
        };
        assert!(d3_non2d_perspective.to_2d().is_none());

        let d3_non2d_quaternion = Decomposed3d {
            translate: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            skew: (0.0, 0.0, 0.0),
            perspective: (0.0, 0.0, 0.0, 1.0),
            quaternion: (1.0, 0.0, 0.0, 0.0), // rotates around X
        };
        assert!(d3_non2d_quaternion.to_2d().is_none());
    }
}
