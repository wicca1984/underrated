#![forbid(unsafe_code)]
#![allow(dead_code)]

mod au;

pub use au::Au;

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

    /// Returns the intersection of two rectangles, or None if they do not intersect.
    /// Touching rectangles (with zero-area intersection) are considered to intersect.
    ///
    /// // spec: CSSOM View intersection operation
    pub fn intersect(&self, other: Rect) -> Option<Rect> {
        self.intersection(other)
    }

    /// Returns true if this rectangle intersects with another rectangle.
    ///
    /// // spec: CSSOM View intersection check
    pub fn intersects(&self, other: Rect) -> bool {
        self.intersection(other).is_some()
    }

    /// Returns the union of two rectangles, which is the smallest rectangle that contains both.
    ///
    /// // spec: CSSOM View union operation
    pub fn union(&self, other: Rect) -> Rect {
        let x1 = self.origin.x.min(other.origin.x);
        let y1 = self.origin.y.min(other.origin.y);
        let x2 = self.max_x().max(other.max_x());
        let y2 = self.max_y().max(other.max_y());
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Returns true if this rectangle completely contains another rectangle.
    pub fn contains_rect(&self, other: Rect) -> bool {
        other.origin.x >= self.origin.x
            && other.max_x() <= self.max_x()
            && other.origin.y >= self.origin.y
            && other.max_y() <= self.max_y()
    }

    /// Returns a new rectangle translated by the given dx and dy offsets.
    ///
    /// // spec: CSSOM View translation helper
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(
            self.origin.x + dx,
            self.origin.y + dy,
            self.size.width,
            self.size.height,
        )
    }

    /// Returns a new rectangle translated by the coordinates of the given point.
    ///
    /// // spec: CSSOM View translation helper
    pub fn translate_by_point(&self, p: Point) -> Rect {
        self.translate(p.x, p.y)
    }

    /// Translates this rectangle in-place by the given dx and dy offsets.
    ///
    /// // spec: CSSOM View translation helper
    pub fn translate_mut(&mut self, dx: f32, dy: f32) {
        self.origin.x += dx;
        self.origin.y += dy;
    }

    /// Translates this rectangle in-place by the coordinates of the given point.
    ///
    /// // spec: CSSOM View translation helper
    pub fn translate_by_point_mut(&mut self, p: Point) {
        self.translate_mut(p.x, p.y);
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
    fn test_intersect_and_intersects() {
        let r1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        let r3 = Rect::new(15.0, 15.0, 5.0, 5.0);

        // intersect behavior
        let inter = r1.intersect(r2).unwrap();
        assert!(approx_eq(inter.origin.x, 5.0));
        assert!(approx_eq(inter.origin.y, 5.0));
        assert!(approx_eq(inter.size.width, 5.0));
        assert!(approx_eq(inter.size.height, 5.0));

        assert!(r1.intersect(r3).is_none());

        // intersects behavior
        assert!(r1.intersects(r2));
        assert!(!r1.intersects(r3));
    }

    #[test]
    fn test_union() {
        let r1 = Rect::new(1.0, 2.0, 3.0, 4.0);
        let r2 = Rect::new(5.0, 6.0, 2.0, 2.0);

        let u = r1.union(r2);
        assert!(approx_eq(u.origin.x, 1.0));
        assert!(approx_eq(u.origin.y, 2.0));
        assert!(approx_eq(u.size.width, 6.0)); // max_x is 7.0, min_x is 1.0, width = 6.0
        assert!(approx_eq(u.size.height, 6.0)); // max_y is 8.0, min_y is 2.0, height = 6.0
    }

    #[test]
    fn test_contains_rect() {
        let r1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = Rect::new(2.0, 2.0, 5.0, 5.0);
        let r3 = Rect::new(5.0, 5.0, 10.0, 10.0);

        assert!(r1.contains_rect(r2));
        assert!(!r1.contains_rect(r3));
        assert!(r1.contains_rect(r1));
    }

    #[test]
    fn test_translate() {
        let r1 = Rect::new(1.0, 2.0, 3.0, 4.0);

        // translate immutably
        let r2 = r1.translate(2.0, -1.0);
        assert!(approx_eq(r1.origin.x, 1.0)); // original unmodified
        assert!(approx_eq(r1.origin.y, 2.0));
        assert!(approx_eq(r2.origin.x, 3.0));
        assert!(approx_eq(r2.origin.y, 1.0));
        assert!(approx_eq(r2.size.width, 3.0));
        assert!(approx_eq(r2.size.height, 4.0));

        // translate by point
        let r3 = r1.translate_by_point(Point { x: -1.0, y: 3.0 });
        assert!(approx_eq(r3.origin.x, 0.0));
        assert!(approx_eq(r3.origin.y, 5.0));

        // translate mutably
        let mut r4 = r1;
        r4.translate_mut(2.0, -1.0);
        assert!(approx_eq(r4.origin.x, 3.0));
        assert!(approx_eq(r4.origin.y, 1.0));

        // translate mutably by point
        r4.translate_by_point_mut(Point { x: -3.0, y: 4.0 });
        assert!(approx_eq(r4.origin.x, 0.0));
        assert!(approx_eq(r4.origin.y, 5.0));
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
