use crate::geom::Rect;

/// A bounding-box union of all invalidated (dirty) areas.
///
/// // TODO(spec): The engine will feed per-node dirty rects here and later use `bounds()` to scope partial re-paint.
#[derive(Debug, Clone, Default)]
pub struct DirtyRegion {
    bounds: Option<Rect>,
}

impl DirtyRegion {
    /// Creates a new empty `DirtyRegion`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a rectangle to the dirty region, extending the bounding box to cover it.
    pub fn add(&mut self, rect: Rect) {
        if let Some(current) = self.bounds {
            self.bounds = Some(current.union(rect));
        } else {
            self.bounds = Some(rect);
        }
    }

    /// Returns the current bounding box of the invalidated region, or `None` if empty.
    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }

    /// Returns true if the dirty region is currently empty.
    pub fn is_empty(&self) -> bool {
        self.bounds.is_none()
    }

    /// Clears the dirty region, resetting it to empty.
    pub fn clear(&mut self) {
        self.bounds = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_dirty_region() {
        let region = DirtyRegion::new();
        assert!(region.is_empty());
        assert_eq!(region.bounds(), None);

        let region_default = DirtyRegion::default();
        assert!(region_default.is_empty());
        assert_eq!(region_default.bounds(), None);
    }

    #[test]
    fn test_single_add() {
        let mut region = DirtyRegion::new();
        let rect = Rect::new(5.0, 10.0, 50.0, 100.0);
        region.add(rect);

        assert!(!region.is_empty());
        let bounds = region.bounds().unwrap();
        assert_eq!(bounds.origin.x, 5.0);
        assert_eq!(bounds.origin.y, 10.0);
        assert_eq!(bounds.size.width, 50.0);
        assert_eq!(bounds.size.height, 100.0);
    }

    #[test]
    fn test_disjoint_adds() {
        let mut region = DirtyRegion::new();
        let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let rect2 = Rect::new(20.0, 20.0, 10.0, 10.0);

        region.add(rect1);
        region.add(rect2);

        assert!(!region.is_empty());
        let bounds = region.bounds().unwrap();
        assert_eq!(bounds.origin.x, 0.0);
        assert_eq!(bounds.origin.y, 0.0);
        assert_eq!(bounds.size.width, 30.0);
        assert_eq!(bounds.size.height, 30.0);
    }

    #[test]
    fn test_clear() {
        let mut region = DirtyRegion::new();
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        region.add(rect);
        assert!(!region.is_empty());

        region.clear();
        assert!(region.is_empty());
        assert_eq!(region.bounds(), None);
    }
}
