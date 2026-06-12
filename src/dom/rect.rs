use crate::dom::Dom;
use crate::infra::NodeId;

/// `DomRect` represents a rectangle, which is the type of object returned by
/// `Element.getBoundingClientRect()`.
///
/// It provides read-only properties describing the size and position of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl DomRect {
    /// Creates a new `DomRect` with the given coordinates and dimensions.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the x-coordinate of the origin of the rectangle.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y-coordinate of the origin of the rectangle.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the width of the rectangle.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// Returns the height of the rectangle.
    pub fn height(&self) -> f64 {
        self.height
    }

    /// Returns the top coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative height,
    /// this is normalized as: `min(y, y + height)`.
    pub fn top(&self) -> f64 {
        self.y.min(self.y + self.height)
    }

    /// Returns the right coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `max(x, x + width)`.
    pub fn right(&self) -> f64 {
        self.x.max(self.x + self.width)
    }

    /// Returns the bottom coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative height,
    /// this is normalized as: `max(y, y + height)`.
    pub fn bottom(&self) -> f64 {
        self.y.max(self.y + self.height)
    }

    /// Returns the left coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `min(x, x + width)`.
    pub fn left(&self) -> f64 {
        self.x.min(self.x + self.width)
    }

    /// Serializes this `DomRect` to a JSON object.
    ///
    /// Matches the standard format used in this codebase for serialized rects.
    pub fn serialize(&self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x(),
            "y": self.y(),
            "width": self.width(),
            "height": self.height(),
            "top": self.top(),
            "right": self.right(),
            "bottom": self.bottom(),
            "left": self.left(),
        })
    }
}

impl Dom {
    /// Returns the bounding client rect of the element.
    ///
    /// // TODO(spec): Element::get_bounding_client_rect() will, in a future task,
    /// // read the element's laid-out box and return a `DomRect`. For now, this is
    /// // a DOM-side preparation placeholder.
    pub fn get_bounding_client_rect(&self, _node: NodeId) -> DomRect {
        DomRect::new(0.0, 0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domrect_basic_edges() {
        let rect = DomRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x(), 10.0);
        assert_eq!(rect.y(), 20.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);

        assert_eq!(rect.left(), 10.0);
        assert_eq!(rect.top(), 20.0);
        assert_eq!(rect.right(), 110.0);
        assert_eq!(rect.bottom(), 70.0);
    }

    #[test]
    fn test_domrect_negative_dimensions_normalize() {
        // Negative width
        let rect_neg_w = DomRect::new(10.0, 20.0, -100.0, 50.0);
        assert_eq!(rect_neg_w.left(), -90.0); // min(10.0, -90.0)
        assert_eq!(rect_neg_w.right(), 10.0); // max(10.0, -90.0)
        assert_eq!(rect_neg_w.top(), 20.0);
        assert_eq!(rect_neg_w.bottom(), 70.0);

        // Negative height
        let rect_neg_h = DomRect::new(10.0, 20.0, 100.0, -50.0);
        assert_eq!(rect_neg_h.left(), 10.0);
        assert_eq!(rect_neg_h.right(), 110.0);
        assert_eq!(rect_neg_h.top(), -30.0); // min(20.0, -30.0)
        assert_eq!(rect_neg_h.bottom(), 20.0); // max(20.0, -30.0)

        // Both negative
        let rect_both_neg = DomRect::new(10.0, 20.0, -100.0, -50.0);
        assert_eq!(rect_both_neg.left(), -90.0);
        assert_eq!(rect_both_neg.right(), 10.0);
        assert_eq!(rect_both_neg.top(), -30.0);
        assert_eq!(rect_both_neg.bottom(), 20.0);
    }

    #[test]
    fn test_domrect_serialize_shape() {
        let rect = DomRect::new(10.0, 20.0, 100.0, 50.0);
        let serialized = rect.serialize();

        assert_eq!(serialized["x"], 10.0);
        assert_eq!(serialized["y"], 20.0);
        assert_eq!(serialized["width"], 100.0);
        assert_eq!(serialized["height"], 50.0);
        assert_eq!(serialized["top"], 20.0);
        assert_eq!(serialized["right"], 110.0);
        assert_eq!(serialized["bottom"], 70.0);
        assert_eq!(serialized["left"], 10.0);
    }
}
