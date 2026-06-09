use std::ops::{Add, Mul, Sub};

/// App units (Au) are used for layout calculations to avoid subpixel rounding errors.
/// 1px = 60 Au.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct Au(pub i32);

impl Au {
    /// The number of app units per CSS pixel.
    pub const PER_PX: i32 = 60;

    /// Creates an `Au` from a CSS pixel value (rounded).
    pub fn from_px(px: f32) -> Au {
        Au((px * Self::PER_PX as f32).round() as i32)
    }

    /// Converts the `Au` to a CSS pixel value.
    pub fn to_px(self) -> f32 {
        self.0 as f32 / Self::PER_PX as f32
    }

    /// Creates an `Au` from an integer CSS pixel value.
    pub fn from_px_i32(px: i32) -> Au {
        Au(px * Self::PER_PX)
    }
}

impl Add for Au {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Au(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Au {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Au(self.0.saturating_sub(rhs.0))
    }
}

impl Mul<i32> for Au {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Au(self.0.saturating_mul(rhs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_px() {
        assert_eq!(Au::from_px(1.0), Au(60));
        assert_eq!(Au::from_px(0.0), Au(0));
        assert_eq!(Au::from_px(0.5), Au(30));
        assert_eq!(Au::from_px(0.49), Au(29));
        assert_eq!(Au::from_px(0.51), Au(31));
        assert_eq!(Au::from_px(-1.0), Au(-60));
    }

    #[test]
    fn test_to_px() {
        assert_eq!(Au(60).to_px(), 1.0);
        assert_eq!(Au(0).to_px(), 0.0);
        assert_eq!(Au(30).to_px(), 0.5);
    }

    #[test]
    fn test_round_trip() {
        let cases = [0.0, 1.0, 0.5, 123.45, -10.2];
        for &px in &cases {
            let au = Au::from_px(px);
            let back = au.to_px();
            assert!((px - back).abs() < 1.0 / Au::PER_PX as f32);
        }
    }

    #[test]
    fn test_from_px_i32() {
        assert_eq!(Au::from_px_i32(1), Au(60));
        assert_eq!(Au::from_px_i32(0), Au(0));
        assert_eq!(Au::from_px_i32(-2), Au(-120));
    }

    #[test]
    fn test_add() {
        assert_eq!(Au(60) + Au(60), Au(120));
        assert_eq!(Au(i32::MAX) + Au(1), Au(i32::MAX));
    }

    #[test]
    fn test_sub() {
        assert_eq!(Au(120) - Au(60), Au(60));
        assert_eq!(Au(i32::MIN) - Au(1), Au(i32::MIN));
    }

    #[test]
    fn test_mul() {
        assert_eq!(Au(60) * 2, Au(120));
        assert_eq!(Au(i32::MAX) * 2, Au(i32::MAX));
        assert_eq!(Au(i32::MIN) * 2, Au(i32::MIN));
    }

    #[test]
    fn test_ordering() {
        assert!(Au(60) > Au(30));
        assert!(Au(-10) < Au(0));
        assert_eq!(Au(100), Au(100));
    }
}
