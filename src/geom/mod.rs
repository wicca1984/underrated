#![forbid(unsafe_code)]
#![allow(dead_code)]

/// A 2D point with f32 coordinates.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// A 2D size with f32 dimensions.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// A 2D rectangle defined by an origin point and a size.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

/// A set of edges (e.g., for margin, padding, or border).
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            origin: Point { x, y },
            size: Size {
                width: w,
                height: h,
            },
        }
    }

    /// Returns the maximum X coordinate (right edge) of the rectangle.
    pub fn max_x(&self) -> f32 {
        self.origin.x + self.size.width
    }

    /// Returns the maximum Y coordinate (bottom edge) of the rectangle.
    pub fn max_y(&self) -> f32 {
        self.origin.y + self.size.height
    }

    /// Returns true if the given point is inside or on the edge of the rectangle.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.origin.x && p.x <= self.max_x() && p.y >= self.origin.y && p.y <= self.max_y()
    }

    /// Returns the intersection of two rectangles, or None if they do not intersect.
    /// Touching rectangles (with zero-area intersection) are considered to intersect.
    pub fn intersection(&self, other: Rect) -> Option<Rect> {
        let x1 = self.origin.x.max(other.origin.x);
        let y1 = self.origin.y.max(other.origin.y);
        let x2 = self.max_x().min(other.max_x());
        let y2 = self.max_y().min(other.max_y());

        if x1 <= x2 && y1 <= y2 {
            Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_rect_new() {
        let r = Rect::new(1.0, 2.0, 3.0, 4.0);
        assert!(approx_eq(r.origin.x, 1.0));
        assert!(approx_eq(r.origin.y, 2.0));
        assert!(approx_eq(r.size.width, 3.0));
        assert!(approx_eq(r.size.height, 4.0));
    }

    #[test]
    fn test_max_x_y() {
        let r = Rect::new(1.0, 2.0, 3.0, 4.0);
        assert!(approx_eq(r.max_x(), 4.0));
        assert!(approx_eq(r.max_y(), 6.0));
    }

    #[test]
    fn test_contains() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);

        // Inside
        assert!(r.contains(Point { x: 5.0, y: 5.0 }));

        // Edges
        assert!(r.contains(Point { x: 0.0, y: 0.0 }));
        assert!(r.contains(Point { x: 10.0, y: 10.0 }));
        assert!(r.contains(Point { x: 0.0, y: 5.0 }));
        assert!(r.contains(Point { x: 10.0, y: 5.0 }));
        assert!(r.contains(Point { x: 5.0, y: 0.0 }));
        assert!(r.contains(Point { x: 5.0, y: 10.0 }));

        // Outside
        assert!(!r.contains(Point { x: -0.1, y: 5.0 }));
        assert!(!r.contains(Point { x: 10.1, y: 5.0 }));
        assert!(!r.contains(Point { x: 5.0, y: -0.1 }));
        assert!(!r.contains(Point { x: 5.0, y: 10.1 }));
    }

    #[test]
    fn test_intersection() {
        let r1 = Rect::new(0.0, 0.0, 10.0, 10.0);

        // Overlapping
        let r2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        let inter = r1.intersection(r2).unwrap();
        assert!(approx_eq(inter.origin.x, 5.0));
        assert!(approx_eq(inter.origin.y, 5.0));
        assert!(approx_eq(inter.size.width, 5.0));
        assert!(approx_eq(inter.size.height, 5.0));

        // Touching (edge)
        let r3 = Rect::new(10.0, 0.0, 10.0, 10.0);
        let inter = r1.intersection(r3).unwrap();
        assert!(approx_eq(inter.origin.x, 10.0));
        assert!(approx_eq(inter.origin.y, 0.0));
        assert!(approx_eq(inter.size.width, 0.0));
        assert!(approx_eq(inter.size.height, 10.0));

        // Touching (corner)
        let r4 = Rect::new(10.0, 10.0, 10.0, 10.0);
        let inter = r1.intersection(r4).unwrap();
        assert!(approx_eq(inter.origin.x, 10.0));
        assert!(approx_eq(inter.origin.y, 10.0));
        assert!(approx_eq(inter.size.width, 0.0));
        assert!(approx_eq(inter.size.height, 0.0));

        // Disjoint
        let r5 = Rect::new(11.0, 0.0, 10.0, 10.0);
        assert!(r1.intersection(r5).is_none());
    }

    #[test]
    fn test_edges() {
        let e = Edges {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 4.0,
        };
        assert!(approx_eq(e.top, 1.0));
        assert!(approx_eq(e.right, 2.0));
        assert!(approx_eq(e.bottom, 3.0));
        assert!(approx_eq(e.left, 4.0));
    }

    #[test]
    fn test_default() {
        assert_eq!(Point::default(), Point { x: 0.0, y: 0.0 });
        assert_eq!(
            Size::default(),
            Size {
                width: 0.0,
                height: 0.0
            }
        );
        assert_eq!(
            Rect::default(),
            Rect {
                origin: Point::default(),
                size: Size::default()
            }
        );
        assert_eq!(
            Edges::default(),
            Edges {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0
            }
        );
    }
}
