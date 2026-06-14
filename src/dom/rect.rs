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

    /// Static-like factory to create a new `DomRect` from a dictionary-like JSON representation.
    /// It takes an optional DOMRectInit-like object `{x, y, width, height}` (each defaulting to 0.0 when absent)
    /// and returns a new `DomRect` with those values.
    pub fn from_rect(other: Option<&serde_json::Value>) -> Self {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;

        if let Some(obj) = other.and_then(|v| v.as_object()) {
            if let Some(val) = obj.get("x").and_then(|v| v.as_f64()) {
                x = val;
            }
            if let Some(val) = obj.get("y").and_then(|v| v.as_f64()) {
                y = val;
            }
            if let Some(val) = obj.get("width").and_then(|v| v.as_f64()) {
                width = val;
            }
            if let Some(val) = obj.get("height").and_then(|v| v.as_f64()) {
                height = val;
            }
        }

        Self::new(x, y, width, height)
    }

    /// Non-snake-case alias of `from_rect` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromRect(other: Option<&serde_json::Value>) -> Self {
        Self::from_rect(other)
    }

    /// Returns a plain JSON object with keys x, y, width, height, top, right, bottom, left holding current numeric values.
    pub fn to_json(&self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(&self) -> serde_json::Value {
        self.serialize()
    }
}

/// `DomRectList` represents a list of `DomRect` objects, mirroring DOM's DOMRectList.
#[derive(Debug, Clone, PartialEq)]
pub struct DomRectList {
    rects: Vec<DomRect>,
}

impl DomRectList {
    /// Creates a new `DomRectList` with the given vector of rectangles.
    pub fn new(rects: Vec<DomRect>) -> Self {
        Self { rects }
    }

    /// Returns the number of rectangles in the list.
    pub fn length(&self) -> usize {
        self.rects.len()
    }

    /// Returns the `DomRect` at the specified index, or `None` if the index is out of range.
    pub fn item(&self, index: usize) -> Option<DomRect> {
        self.rects.get(index).copied()
    }

    /// Serializes this `DomRectList` to a JSON array of `DomRect` objects.
    pub fn serialize(&self) -> serde_json::Value {
        let serialized_rects: Vec<serde_json::Value> =
            self.rects.iter().map(|r| r.serialize()).collect();
        serde_json::Value::Array(serialized_rects)
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

    /// Returns the list of client rectangles for an element.
    ///
    /// For a non-fragmented box this is a single-item list equal to the bounding rect;
    /// returns an empty list for elements with no box or if the node is not an element.
    pub fn get_client_rects(&self, node: NodeId) -> DomRectList {
        use crate::dom::NodeData;
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let rect = self.get_bounding_client_rect(node);
            DomRectList::new(vec![rect])
        } else {
            DomRectList::new(Vec::new())
        }
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

    #[test]
    fn test_get_client_rects_basic() {
        let mut dom = Dom::new();
        use crate::dom::NodeData;
        let elem_node = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });

        let rects = dom.get_client_rects(elem_node);
        assert_eq!(rects.length(), 1);

        let bound_rect = dom.get_bounding_client_rect(elem_node);
        assert_eq!(rects.item(0), Some(bound_rect));
        assert_eq!(rects.item(1), None);
        assert_eq!(rects.item(999), None);

        // Serialize check
        let serialized_list = rects.serialize();
        assert!(serialized_list.is_array());
        assert_eq!(serialized_list.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_get_client_rects_non_element() {
        let mut dom = Dom::new();
        use crate::dom::NodeData;
        let text_node = dom.create_node(NodeData::Text("hello".to_string()));

        let rects = dom.get_client_rects(text_node);
        assert_eq!(rects.length(), 0);
        assert_eq!(rects.item(0), None);

        // Serialize check
        let serialized_list = rects.serialize();
        assert!(serialized_list.is_array());
        assert_eq!(serialized_list.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_domrect_from_rect_with_init() {
        let init = serde_json::json!({
            "x": 10.0,
            "y": 20.0,
            "width": 100.0,
            "height": 50.0
        });
        let rect = DomRect::from_rect(Some(&init));
        assert_eq!(rect.x(), 10.0);
        assert_eq!(rect.y(), 20.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);

        // Test non-snake-case alias fromRect
        let rect_camel = DomRect::fromRect(Some(&init));
        assert_eq!(rect_camel.x(), 10.0);
        assert_eq!(rect_camel.y(), 20.0);
        assert_eq!(rect_camel.width(), 100.0);
        assert_eq!(rect_camel.height(), 50.0);
    }

    #[test]
    fn test_domrect_from_rect_empty() {
        let rect_empty = DomRect::from_rect(None);
        assert_eq!(rect_empty.x(), 0.0);
        assert_eq!(rect_empty.y(), 0.0);
        assert_eq!(rect_empty.width(), 0.0);
        assert_eq!(rect_empty.height(), 0.0);

        let init_empty = serde_json::json!({});
        let rect_init_empty = DomRect::from_rect(Some(&init_empty));
        assert_eq!(rect_init_empty.x(), 0.0);
        assert_eq!(rect_init_empty.y(), 0.0);
        assert_eq!(rect_init_empty.width(), 0.0);
        assert_eq!(rect_init_empty.height(), 0.0);
    }

    #[test]
    fn test_domrect_to_json() {
        let rect = DomRect::new(10.0, 20.0, 100.0, 50.0);
        let json_val = rect.to_json();
        assert_eq!(json_val["x"], 10.0);
        assert_eq!(json_val["y"], 20.0);
        assert_eq!(json_val["width"], 100.0);
        assert_eq!(json_val["height"], 50.0);
        assert_eq!(json_val["top"], 20.0);
        assert_eq!(json_val["right"], 110.0);
        assert_eq!(json_val["bottom"], 70.0);
        assert_eq!(json_val["left"], 10.0);

        // Test non-snake-case alias toJSON
        let json_camel = rect.toJSON();
        assert_eq!(json_camel["x"], 10.0);
        assert_eq!(json_camel["y"], 20.0);
        assert_eq!(json_camel["width"], 100.0);
        assert_eq!(json_camel["height"], 50.0);
        assert_eq!(json_camel["top"], 20.0);
        assert_eq!(json_camel["right"], 110.0);
        assert_eq!(json_camel["bottom"], 70.0);
        assert_eq!(json_camel["left"], 10.0);
    }
}
