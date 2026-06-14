use crate::dom::Dom;
use crate::infra::NodeId;
use serde::{Deserialize, Serialize};

/// Coerce a JSON value to f64 following WebIDL-like conversion principles.
fn coerce_to_f64(value: &serde_json::Value) -> f64 {
    match value {
        serde_json::Value::Number(num) => num.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        serde_json::Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        serde_json::Value::Null => 0.0,
        _ => f64::NAN,
    }
}

/// `DomRectReadOnly` represents a read-only rectangle, which is a standard base
/// interface for `DOMRect` per the CSS Geometry Interfaces standard.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DomRectReadOnly {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl DomRectReadOnly {
    /// Creates a new `DomRectReadOnly` with the given coordinates and dimensions.
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
    /// this is normalized as: `y` if `height >= 0`, and `y + height` otherwise.
    /// Handles NaN correctly per spec.
    pub fn top(&self) -> f64 {
        if self.height.is_nan() {
            f64::NAN
        } else if self.height >= 0.0 {
            self.y
        } else {
            self.y + self.height
        }
    }

    /// Returns the right coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `x + width` if `width >= 0`, and `x` otherwise.
    /// Handles NaN correctly per spec.
    pub fn right(&self) -> f64 {
        if self.width.is_nan() {
            f64::NAN
        } else if self.width >= 0.0 {
            self.x + self.width
        } else {
            self.x
        }
    }

    /// Returns the bottom coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative height,
    /// this is normalized as: `y + height` if `height >= 0`, and `y` otherwise.
    /// Handles NaN correctly per spec.
    pub fn bottom(&self) -> f64 {
        if self.height.is_nan() {
            f64::NAN
        } else if self.height >= 0.0 {
            self.y + self.height
        } else {
            self.y
        }
    }

    /// Returns the left coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `x` if `width >= 0`, and `x + width` otherwise.
    /// Handles NaN correctly per spec.
    pub fn left(&self) -> f64 {
        if self.width.is_nan() {
            f64::NAN
        } else if self.width >= 0.0 {
            self.x
        } else {
            self.x + self.width
        }
    }

    /// Serializes this `DomRectReadOnly` to a JSON object.
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

    /// Static-like factory to create a new `DomRectReadOnly` from a dictionary-like JSON representation
    /// or a JSON Array representation (e.g., `[x, y, width, height]` or `[width, height]`).
    pub fn from_rect(other: Option<&serde_json::Value>) -> Self {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut width = 0.0;
        let mut height = 0.0;

        if let Some(val) = other {
            if let Some(obj) = val.as_object() {
                if let Some(val) = obj.get("x") {
                    x = coerce_to_f64(val);
                }
                if let Some(val) = obj.get("y") {
                    y = coerce_to_f64(val);
                }
                if let Some(val) = obj.get("width") {
                    width = coerce_to_f64(val);
                }
                if let Some(val) = obj.get("height") {
                    height = coerce_to_f64(val);
                }
            } else if let Some(arr) = val.as_array() {
                if arr.len() >= 4 {
                    x = coerce_to_f64(&arr[0]);
                    y = coerce_to_f64(&arr[1]);
                    width = coerce_to_f64(&arr[2]);
                    height = coerce_to_f64(&arr[3]);
                } else if arr.len() == 2 {
                    width = coerce_to_f64(&arr[0]);
                    height = coerce_to_f64(&arr[1]);
                } else if arr.len() == 3 {
                    x = coerce_to_f64(&arr[0]);
                    y = coerce_to_f64(&arr[1]);
                    width = coerce_to_f64(&arr[2]);
                }
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
    pub fn to_json(self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(self) -> serde_json::Value {
        self.serialize()
    }

    /// Returns a mutable `DomRect` copy.
    pub fn to_mutable(self) -> DomRect {
        DomRect::new(self.x, self.y, self.width, self.height)
    }

    /// Returns true if the given point is inside or on the edge of the rectangle.
    /// Handles NaN coordinates correctly (always returns false if any value is NaN).
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        if px.is_nan() || py.is_nan() {
            return false;
        }
        let left = self.left();
        let right = self.right();
        let top = self.top();
        let bottom = self.bottom();
        if left.is_nan() || right.is_nan() || top.is_nan() || bottom.is_nan() {
            return false;
        }
        px >= left && px <= right && py >= top && py <= bottom
    }

    /// Returns true if this rectangle intersects with another rectangle.
    /// Overlapping on the edge is considered an intersection.
    /// Handles NaN coordinates correctly (returns false if any coordinate is NaN).
    pub fn intersects(&self, other: &Self) -> bool {
        let s_left = self.left();
        let s_right = self.right();
        let s_top = self.top();
        let s_bottom = self.bottom();
        let o_left = other.left();
        let o_right = other.right();
        let o_top = other.top();
        let o_bottom = other.bottom();

        if s_left.is_nan()
            || s_right.is_nan()
            || s_top.is_nan()
            || s_bottom.is_nan()
            || o_left.is_nan()
            || o_right.is_nan()
            || o_top.is_nan()
            || o_bottom.is_nan()
        {
            return false;
        }

        s_left <= o_right && s_right >= o_left && s_top <= o_bottom && s_bottom >= o_top
    }

    /// Returns the smallest rectangle that contains both this rectangle and another rectangle.
    ///
    /// If either rectangle contains NaN values, returns a default (zeroed) rectangle.
    pub fn union(&self, other: &Self) -> Self {
        let s_left = self.left();
        let s_right = self.right();
        let s_top = self.top();
        let s_bottom = self.bottom();
        let o_left = other.left();
        let o_right = other.right();
        let o_top = other.top();
        let o_bottom = other.bottom();

        if s_left.is_nan()
            || s_right.is_nan()
            || s_top.is_nan()
            || s_bottom.is_nan()
            || o_left.is_nan()
            || o_right.is_nan()
            || o_top.is_nan()
            || o_bottom.is_nan()
        {
            return Self::default();
        }

        let min_x = s_left.min(o_left);
        let min_y = s_top.min(o_top);
        let max_x = s_right.max(o_right);
        let max_y = s_bottom.max(o_bottom);

        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Translates the origin of the rectangle by `dx` and `dy`, returning a new `DomRectReadOnly`.
    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    /// Scales both the position and the size of the rectangle by `sx` and `sy`, returning a new `DomRectReadOnly`.
    pub fn scale(&self, sx: f64, sy: f64) -> Self {
        Self::new(self.x * sx, self.y * sy, self.width * sx, self.height * sy)
    }

    /// Returns true if the rectangle is empty (i.e. has a width or height less than or equal to 0.0, or is NaN).
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0 || self.width.is_nan() || self.height.is_nan()
    }

    /// Returns a normalized copy of the rectangle where the width and height are guaranteed to be non-negative,
    /// while preserving the same spatial boundary coordinates.
    pub fn normalize(&self) -> Self {
        let (new_x, new_width) = if self.width.is_nan() {
            (self.x, self.width)
        } else {
            (self.left(), self.width.abs())
        };

        let (new_y, new_height) = if self.height.is_nan() {
            (self.y, self.height)
        } else {
            (self.top(), self.height.abs())
        };

        Self::new(new_x, new_y, new_width, new_height)
    }

    /// Returns the intersection of this rectangle and another rectangle.
    /// If they do not intersect, or if any coordinate is NaN, returns a default (zeroed) rectangle.
    pub fn intersection(&self, other: &Self) -> Self {
        if !self.intersects(other) {
            return Self::default();
        }
        let s_left = self.left();
        let s_right = self.right();
        let s_top = self.top();
        let s_bottom = self.bottom();
        let o_left = other.left();
        let o_right = other.right();
        let o_top = other.top();
        let o_bottom = other.bottom();

        let left = s_left.max(o_left);
        let right = s_right.min(o_right);
        let top = s_top.max(o_top);
        let bottom = s_bottom.min(o_bottom);

        if left > right || top > bottom {
            Self::default()
        } else {
            Self::new(left, top, right - left, bottom - top)
        }
    }
}

impl Serialize for DomRectReadOnly {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DomRectReadOnly", 8)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("top", &self.top())?;
        state.serialize_field("right", &self.right())?;
        state.serialize_field("bottom", &self.bottom())?;
        state.serialize_field("left", &self.left())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DomRectReadOnly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_rect(Some(&value)))
    }
}

/// `DomRect` represents a rectangle, which is the type of object returned by
/// `Element.getBoundingClientRect()`.
///
/// It provides read-write properties describing the size and position of a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

    /// Sets the x-coordinate of the origin of the rectangle.
    pub fn set_x(&mut self, val: f64) {
        self.x = val;
    }

    /// Sets the y-coordinate of the origin of the rectangle.
    pub fn set_y(&mut self, val: f64) {
        self.y = val;
    }

    /// Sets the width of the rectangle.
    pub fn set_width(&mut self, val: f64) {
        self.width = val;
    }

    /// Sets the height of the rectangle.
    pub fn set_height(&mut self, val: f64) {
        self.height = val;
    }

    /// Returns the top coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative height,
    /// this is normalized as: `y` if `height >= 0`, and `y + height` otherwise.
    /// Handles NaN correctly per spec.
    pub fn top(&self) -> f64 {
        if self.height.is_nan() {
            f64::NAN
        } else if self.height >= 0.0 {
            self.y
        } else {
            self.y + self.height
        }
    }

    /// Returns the right coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `x + width` if `width >= 0`, and `x` otherwise.
    /// Handles NaN correctly per spec.
    pub fn right(&self) -> f64 {
        if self.width.is_nan() {
            f64::NAN
        } else if self.width >= 0.0 {
            self.x + self.width
        } else {
            self.x
        }
    }

    /// Returns the bottom coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative height,
    /// this is normalized as: `y + height` if `height >= 0`, and `y` otherwise.
    /// Handles NaN correctly per spec.
    pub fn bottom(&self) -> f64 {
        if self.height.is_nan() {
            f64::NAN
        } else if self.height >= 0.0 {
            self.y + self.height
        } else {
            self.y
        }
    }

    /// Returns the left coordinate value of the rectangle.
    ///
    /// According to the DOMRect spec, for a possibly-negative width,
    /// this is normalized as: `x` if `width >= 0`, and `x + width` otherwise.
    /// Handles NaN correctly per spec.
    pub fn left(&self) -> f64 {
        if self.width.is_nan() {
            f64::NAN
        } else if self.width >= 0.0 {
            self.x
        } else {
            self.x + self.width
        }
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

    /// Static-like factory to create a new `DomRect` from a dictionary-like JSON representation
    /// or a JSON Array representation (e.g., `[x, y, width, height]` or `[width, height]`).
    pub fn from_rect(other: Option<&serde_json::Value>) -> Self {
        DomRectReadOnly::from_rect(other).to_mutable()
    }

    /// Non-snake-case alias of `from_rect` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromRect(other: Option<&serde_json::Value>) -> Self {
        Self::from_rect(other)
    }

    /// Returns a plain JSON object with keys x, y, width, height, top, right, bottom, left holding current numeric values.
    pub fn to_json(self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(self) -> serde_json::Value {
        self.serialize()
    }

    /// Returns a read-only `DomRectReadOnly` copy.
    pub fn to_readonly(self) -> DomRectReadOnly {
        DomRectReadOnly::new(self.x, self.y, self.width, self.height)
    }

    /// Returns true if the given point is inside or on the edge of the rectangle.
    /// Handles NaN coordinates correctly (always returns false if any value is NaN).
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        if px.is_nan() || py.is_nan() {
            return false;
        }
        let left = self.left();
        let right = self.right();
        let top = self.top();
        let bottom = self.bottom();
        if left.is_nan() || right.is_nan() || top.is_nan() || bottom.is_nan() {
            return false;
        }
        px >= left && px <= right && py >= top && py <= bottom
    }

    /// Returns true if this rectangle intersects with another rectangle.
    /// Overlapping on the edge is considered an intersection.
    /// Handles NaN coordinates correctly (returns false if any coordinate is NaN).
    pub fn intersects(&self, other: &Self) -> bool {
        let s_left = self.left();
        let s_right = self.right();
        let s_top = self.top();
        let s_bottom = self.bottom();
        let o_left = other.left();
        let o_right = other.right();
        let o_top = other.top();
        let o_bottom = other.bottom();

        if s_left.is_nan()
            || s_right.is_nan()
            || s_top.is_nan()
            || s_bottom.is_nan()
            || o_left.is_nan()
            || o_right.is_nan()
            || o_top.is_nan()
            || o_bottom.is_nan()
        {
            return false;
        }

        s_left <= o_right && s_right >= o_left && s_top <= o_bottom && s_bottom >= o_top
    }

    /// Returns the smallest rectangle that contains both this rectangle and another rectangle.
    ///
    /// If either rectangle contains NaN values, returns a default (zeroed) rectangle.
    pub fn union(&self, other: &Self) -> Self {
        let s_left = self.left();
        let s_right = self.right();
        let s_top = self.top();
        let s_bottom = self.bottom();
        let o_left = other.left();
        let o_right = other.right();
        let o_top = other.top();
        let o_bottom = other.bottom();

        if s_left.is_nan()
            || s_right.is_nan()
            || s_top.is_nan()
            || s_bottom.is_nan()
            || o_left.is_nan()
            || o_right.is_nan()
            || o_top.is_nan()
            || o_bottom.is_nan()
        {
            return Self::default();
        }

        let min_x = s_left.min(o_left);
        let min_y = s_top.min(o_top);
        let max_x = s_right.max(o_right);
        let max_y = s_bottom.max(o_bottom);

        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// Translates the origin of the rectangle by `dx` and `dy`, returning a new `DomRect`.
    pub fn translate(&self, dx: f64, dy: f64) -> Self {
        Self::new(self.x + dx, self.y + dy, self.width, self.height)
    }

    /// Scales both the position and the size of the rectangle by `sx` and `sy`, returning a new `DomRect`.
    pub fn scale(&self, sx: f64, sy: f64) -> Self {
        Self::new(self.x * sx, self.y * sy, self.width * sx, self.height * sy)
    }

    /// Returns true if the rectangle is empty (i.e. has a width or height less than or equal to 0.0, or is NaN).
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0 || self.width.is_nan() || self.height.is_nan()
    }

    /// Returns a normalized copy of the rectangle where the width and height are guaranteed to be non-negative,
    /// while preserving the same spatial boundary coordinates.
    pub fn normalize(&self) -> Self {
        self.to_readonly().normalize().to_mutable()
    }

    /// Returns the intersection of this rectangle and another rectangle.
    /// If they do not intersect, or if any coordinate is NaN, returns a default (zeroed) rectangle.
    pub fn intersection(&self, other: &Self) -> Self {
        self.to_readonly()
            .intersection(&other.to_readonly())
            .to_mutable()
    }
}

impl Serialize for DomRect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DomRect", 8)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("width", &self.width)?;
        state.serialize_field("height", &self.height)?;
        state.serialize_field("top", &self.top())?;
        state.serialize_field("right", &self.right())?;
        state.serialize_field("bottom", &self.bottom())?;
        state.serialize_field("left", &self.left())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DomRect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self::from_rect(Some(&value)))
    }
}

impl From<DomRect> for DomRectReadOnly {
    fn from(rect: DomRect) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }
}

impl From<DomRectReadOnly> for DomRect {
    fn from(rect: DomRectReadOnly) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }
}

impl From<crate::geom::Rect> for DomRect {
    fn from(rect: crate::geom::Rect) -> Self {
        Self::new(
            rect.origin.x as f64,
            rect.origin.y as f64,
            rect.size.width as f64,
            rect.size.height as f64,
        )
    }
}

impl From<crate::geom::Rect> for DomRectReadOnly {
    fn from(rect: crate::geom::Rect) -> Self {
        Self::new(
            rect.origin.x as f64,
            rect.origin.y as f64,
            rect.size.width as f64,
            rect.size.height as f64,
        )
    }
}

impl From<DomRect> for crate::geom::Rect {
    fn from(rect: DomRect) -> Self {
        crate::geom::Rect::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        )
    }
}

impl From<DomRectReadOnly> for crate::geom::Rect {
    fn from(rect: DomRectReadOnly) -> Self {
        crate::geom::Rect::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        )
    }
}

/// `DomRectList` represents a list of `DomRect` objects, mirroring DOM's DOMRectList.
#[derive(Debug, Clone, PartialEq, Default)]
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

    /// Returns an iterator over the rectangles in the list.
    pub fn iter(&self) -> std::slice::Iter<'_, DomRect> {
        self.rects.iter()
    }

    /// Returns a plain JSON array of rect objects.
    pub fn to_json(&self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(&self) -> serde_json::Value {
        self.serialize()
    }
}

impl std::ops::Index<usize> for DomRectList {
    type Output = DomRect;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rects[index]
    }
}

impl FromIterator<DomRect> for DomRectList {
    fn from_iter<T: IntoIterator<Item = DomRect>>(iter: T) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for DomRectList {
    type Item = DomRect;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rects.into_iter()
    }
}

impl<'a> IntoIterator for &'a DomRectList {
    type Item = &'a DomRect;
    type IntoIter = std::slice::Iter<'a, DomRect>;

    fn into_iter(self) -> Self::IntoIter {
        self.rects.iter()
    }
}

impl Extend<DomRect> for DomRectList {
    fn extend<T: IntoIterator<Item = DomRect>>(&mut self, iter: T) {
        self.rects.extend(iter);
    }
}

impl From<Vec<DomRect>> for DomRectList {
    fn from(rects: Vec<DomRect>) -> Self {
        Self::new(rects)
    }
}

impl From<DomRectList> for Vec<DomRect> {
    fn from(list: DomRectList) -> Self {
        list.rects
    }
}

fn min4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if a.is_nan() || b.is_nan() || c.is_nan() || d.is_nan() {
        f64::NAN
    } else {
        a.min(b).min(c).min(d)
    }
}

fn max4(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if a.is_nan() || b.is_nan() || c.is_nan() || d.is_nan() {
        f64::NAN
    } else {
        a.max(b).max(c).max(d)
    }
}

/// `DomPointReadOnly` represents a read-only 2D or 3D point, which is a standard base
/// interface for `DOMPoint` per the CSS Geometry Interfaces standard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomPointReadOnly {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Default for DomPointReadOnly {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl DomPointReadOnly {
    /// Creates a new `DomPointReadOnly` with the given coordinates.
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    /// Returns the x-coordinate of the point.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y-coordinate of the point.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the z-coordinate of the point.
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Returns the w-coordinate of the point.
    pub fn w(&self) -> f64 {
        self.w
    }

    /// Returns a mutable `DomPoint` copy.
    pub fn to_mutable(self) -> DomPoint {
        DomPoint::new(self.x, self.y, self.z, self.w)
    }

    /// Serializes this `DomPointReadOnly` to a JSON object.
    pub fn serialize(&self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
            "w": self.w,
        })
    }

    /// Returns a plain JSON object with keys x, y, z, w holding current numeric values.
    pub fn to_json(self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(self) -> serde_json::Value {
        self.serialize()
    }

    /// Static-like factory to create a new `DomPointReadOnly` from a dictionary-like JSON representation.
    pub fn from_point(other: Option<&serde_json::Value>) -> Self {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut z = 0.0;
        let mut w = 1.0;

        if let Some(obj) = other.and_then(|v| v.as_object()) {
            if let Some(val) = obj.get("x") {
                x = coerce_to_f64(val);
            }
            if let Some(val) = obj.get("y") {
                y = coerce_to_f64(val);
            }
            if let Some(val) = obj.get("z") {
                z = coerce_to_f64(val);
            }
            if let Some(val) = obj.get("w") {
                w = coerce_to_f64(val);
            }
        }

        Self::new(x, y, z, w)
    }

    /// Non-snake-case alias of `from_point` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromPoint(other: Option<&serde_json::Value>) -> Self {
        Self::from_point(other)
    }
}

/// `DomPoint` represents a 2D or 3D point, which is a standard read-write interface
/// per the CSS Geometry Interfaces standard.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DomPoint {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Default for DomPoint {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl DomPoint {
    /// Creates a new `DomPoint` with the given coordinates.
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    /// Returns the x-coordinate of the point.
    pub fn x(&self) -> f64 {
        self.x
    }

    /// Returns the y-coordinate of the point.
    pub fn y(&self) -> f64 {
        self.y
    }

    /// Returns the z-coordinate of the point.
    pub fn z(&self) -> f64 {
        self.z
    }

    /// Returns the w-coordinate of the point.
    pub fn w(&self) -> f64 {
        self.w
    }

    /// Sets the x-coordinate of the point.
    pub fn set_x(&mut self, val: f64) {
        self.x = val;
    }

    /// Sets the y-coordinate of the point.
    pub fn set_y(&mut self, val: f64) {
        self.y = val;
    }

    /// Sets the z-coordinate of the point.
    pub fn set_z(&mut self, val: f64) {
        self.z = val;
    }

    /// Sets the w-coordinate of the point.
    pub fn set_w(&mut self, val: f64) {
        self.w = val;
    }

    /// Returns a read-only `DomPointReadOnly` copy.
    pub fn to_readonly(self) -> DomPointReadOnly {
        DomPointReadOnly::new(self.x, self.y, self.z, self.w)
    }

    /// Serializes this `DomPoint` to a JSON object.
    pub fn serialize(&self) -> serde_json::Value {
        serde_json::json!({
            "x": self.x,
            "y": self.y,
            "z": self.z,
            "w": self.w,
        })
    }

    /// Returns a plain JSON object with keys x, y, z, w holding current numeric values.
    pub fn to_json(self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(self) -> serde_json::Value {
        self.serialize()
    }

    /// Static-like factory to create a new `DomPoint` from a dictionary-like JSON representation.
    pub fn from_point(other: Option<&serde_json::Value>) -> Self {
        let readonly = DomPointReadOnly::from_point(other);
        Self::new(readonly.x, readonly.y, readonly.z, readonly.w)
    }

    /// Non-snake-case alias of `from_point` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromPoint(other: Option<&serde_json::Value>) -> Self {
        Self::from_point(other)
    }
}

impl From<DomPoint> for DomPointReadOnly {
    fn from(pt: DomPoint) -> Self {
        Self::new(pt.x, pt.y, pt.z, pt.w)
    }
}

impl From<DomPointReadOnly> for DomPoint {
    fn from(pt: DomPointReadOnly) -> Self {
        Self::new(pt.x, pt.y, pt.z, pt.w)
    }
}

impl From<crate::geom::Point> for DomPoint {
    fn from(pt: crate::geom::Point) -> Self {
        Self::new(pt.x as f64, pt.y as f64, 0.0, 1.0)
    }
}

impl From<crate::geom::Point> for DomPointReadOnly {
    fn from(pt: crate::geom::Point) -> Self {
        Self::new(pt.x as f64, pt.y as f64, 0.0, 1.0)
    }
}

impl From<DomPoint> for crate::geom::Point {
    fn from(pt: DomPoint) -> Self {
        crate::geom::Point {
            x: pt.x as f32,
            y: pt.y as f32,
        }
    }
}

impl From<DomPointReadOnly> for crate::geom::Point {
    fn from(pt: DomPointReadOnly) -> Self {
        crate::geom::Point {
            x: pt.x as f32,
            y: pt.y as f32,
        }
    }
}

impl From<(f64, f64)> for DomPoint {
    fn from(coords: (f64, f64)) -> Self {
        Self::new(coords.0, coords.1, 0.0, 1.0)
    }
}

impl From<(f64, f64)> for DomPointReadOnly {
    fn from(coords: (f64, f64)) -> Self {
        Self::new(coords.0, coords.1, 0.0, 1.0)
    }
}

impl From<(f64, f64, f64, f64)> for DomPoint {
    fn from(coords: (f64, f64, f64, f64)) -> Self {
        Self::new(coords.0, coords.1, coords.2, coords.3)
    }
}

impl From<(f64, f64, f64, f64)> for DomPointReadOnly {
    fn from(coords: (f64, f64, f64, f64)) -> Self {
        Self::new(coords.0, coords.1, coords.2, coords.3)
    }
}

impl From<DomPoint> for (f64, f64) {
    fn from(pt: DomPoint) -> Self {
        (pt.x, pt.y)
    }
}

impl From<DomPointReadOnly> for (f64, f64) {
    fn from(pt: DomPointReadOnly) -> Self {
        (pt.x, pt.y)
    }
}

impl From<DomPoint> for (f64, f64, f64, f64) {
    fn from(pt: DomPoint) -> Self {
        (pt.x, pt.y, pt.z, pt.w)
    }
}

impl From<DomPointReadOnly> for (f64, f64, f64, f64) {
    fn from(pt: DomPointReadOnly) -> Self {
        (pt.x, pt.y, pt.z, pt.w)
    }
}

/// `DomQuad` represents a quadrilateral with four corners (p1, p2, p3, p4),
/// which is a standard interface per the CSS Geometry Interfaces standard.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DomQuad {
    pub p1: DomPoint,
    pub p2: DomPoint,
    pub p3: DomPoint,
    pub p4: DomPoint,
}

impl DomQuad {
    /// Creates a new `DomQuad` with the four given corners.
    pub fn new(p1: DomPoint, p2: DomPoint, p3: DomPoint, p4: DomPoint) -> Self {
        Self { p1, p2, p3, p4 }
    }

    /// Returns a new `DomQuad` with corner boundaries computed from a `DomRectReadOnly` or `DomRect` (or anything converting to DomRectReadOnly).
    pub fn from_rect_readonly(rect: DomRectReadOnly) -> Self {
        let p1 = DomPoint::new(rect.x(), rect.y(), 0.0, 1.0);
        let p2 = DomPoint::new(rect.x() + rect.width(), rect.y(), 0.0, 1.0);
        let p3 = DomPoint::new(rect.x() + rect.width(), rect.y() + rect.height(), 0.0, 1.0);
        let p4 = DomPoint::new(rect.x(), rect.y() + rect.height(), 0.0, 1.0);
        Self::new(p1, p2, p3, p4)
    }

    /// Non-snake-case alias of `from_rect_readonly` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromRectReadOnly(rect: DomRectReadOnly) -> Self {
        Self::from_rect_readonly(rect)
    }

    /// Static-like factory to create a new `DomQuad` from a dictionary-like JSON representation (DOMRectInit).
    pub fn from_rect(other: Option<&serde_json::Value>) -> Self {
        let rect = DomRectReadOnly::from_rect(other);
        Self::from_rect_readonly(rect)
    }

    /// Non-snake-case alias of `from_rect` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromRect(other: Option<&serde_json::Value>) -> Self {
        Self::from_rect(other)
    }

    /// Static-like factory to create a new `DomQuad` from a dictionary-like JSON representation (DOMQuadInit/DOMQuad).
    pub fn from_quad(other: Option<&serde_json::Value>) -> Self {
        let mut p1 = DomPoint::new(0.0, 0.0, 0.0, 1.0);
        let mut p2 = DomPoint::new(0.0, 0.0, 0.0, 1.0);
        let mut p3 = DomPoint::new(0.0, 0.0, 0.0, 1.0);
        let mut p4 = DomPoint::new(0.0, 0.0, 0.0, 1.0);

        if let Some(obj) = other.and_then(|v| v.as_object()) {
            if let Some(val) = obj.get("p1") {
                p1 = DomPoint::from_point(Some(val));
            }
            if let Some(val) = obj.get("p2") {
                p2 = DomPoint::from_point(Some(val));
            }
            if let Some(val) = obj.get("p3") {
                p3 = DomPoint::from_point(Some(val));
            }
            if let Some(val) = obj.get("p4") {
                p4 = DomPoint::from_point(Some(val));
            }
        }

        Self::new(p1, p2, p3, p4)
    }

    /// Non-snake-case alias of `from_quad` for compatibility.
    #[allow(non_snake_case)]
    pub fn fromQuad(other: Option<&serde_json::Value>) -> Self {
        Self::from_quad(other)
    }

    /// Returns the bounding box of the quadrilateral as a `DomRectReadOnly`.
    pub fn bounds(&self) -> DomRectReadOnly {
        let left = min4(self.p1.x, self.p2.x, self.p3.x, self.p4.x);
        let right = max4(self.p1.x, self.p2.x, self.p3.x, self.p4.x);
        let top = min4(self.p1.y, self.p2.y, self.p3.y, self.p4.y);
        let bottom = max4(self.p1.y, self.p2.y, self.p3.y, self.p4.y);

        DomRectReadOnly::new(left, top, right - left, bottom - top)
    }

    /// Serializes this `DomQuad` to a JSON object containing its four corners.
    pub fn serialize(&self) -> serde_json::Value {
        serde_json::json!({
            "p1": self.p1.serialize(),
            "p2": self.p2.serialize(),
            "p3": self.p3.serialize(),
            "p4": self.p4.serialize(),
        })
    }

    /// Returns a plain JSON object representation of the quad.
    pub fn to_json(self) -> serde_json::Value {
        self.serialize()
    }

    /// Non-snake-case alias of `to_json` for compatibility.
    #[allow(non_snake_case)]
    pub fn toJSON(self) -> serde_json::Value {
        self.serialize()
    }
}

impl From<DomRectReadOnly> for DomQuad {
    fn from(rect: DomRectReadOnly) -> Self {
        Self::from_rect_readonly(rect)
    }
}

impl From<DomRect> for DomQuad {
    fn from(rect: DomRect) -> Self {
        Self::from_rect_readonly(rect.to_readonly())
    }
}

impl From<crate::geom::Rect> for DomQuad {
    fn from(rect: crate::geom::Rect) -> Self {
        let dom_rect = DomRectReadOnly::from(rect);
        Self::from_rect_readonly(dom_rect)
    }
}

std::thread_local! {
    pub static CURRENT_LAYOUT: std::cell::RefCell<Option<crate::layout::LayoutBox>> = const { std::cell::RefCell::new(None) };
    pub static CURRENT_STYLES: std::cell::RefCell<Option<std::collections::HashMap<crate::infra::NodeId, crate::style::CategorizedComputedStyle>>> = const { std::cell::RefCell::new(None) };
}

impl Dom {
    /// Returns the bounding client rect of the element.
    ///
    /// Reads the element's laid-out box and return a `DomRect` from the currently
    /// active `CURRENT_LAYOUT` and `CURRENT_STYLES`, or falls back to a 0-rect
    /// if none is active or matching.
    pub fn get_bounding_client_rect(&self, node: NodeId) -> DomRect {
        let rects = self.get_client_rects(node);
        if rects.length() == 0 {
            return DomRect::new(0.0, 0.0, 0.0, 0.0);
        }

        let mut min_left = f64::INFINITY;
        let mut max_right = -f64::INFINITY;
        let mut min_top = f64::INFINITY;
        let mut max_bottom = -f64::INFINITY;

        for i in 0..rects.length() {
            if let Some(r) = rects.item(i) {
                let left = r.left();
                let right = r.right();
                let top = r.top();
                let bottom = r.bottom();

                if left < min_left {
                    min_left = left;
                }
                if right > max_right {
                    max_right = right;
                }
                if top < min_top {
                    min_top = top;
                }
                if bottom > max_bottom {
                    max_bottom = bottom;
                }
            }
        }

        if min_left.is_infinite() || min_top.is_infinite() {
            DomRect::new(0.0, 0.0, 0.0, 0.0)
        } else {
            DomRect::new(
                min_left,
                min_top,
                max_right - min_left,
                max_bottom - min_top,
            )
        }
    }

    /// Returns the list of client rectangles for an element.
    ///
    /// Recursively computes transformed client rectangles for each fragment of
    /// the element in tree-order. Handles nested transforms correctly.
    pub fn get_client_rects(&self, node: NodeId) -> DomRectList {
        use crate::dom::NodeData;
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let mut rects = Vec::new();
            let mut layout_active = false;

            CURRENT_LAYOUT.with(|layout_cell| {
                if let Some(root) = layout_cell.borrow().as_ref() {
                    layout_active = true;
                    CURRENT_STYLES.with(|styles_cell| {
                        let empty_styles = std::collections::HashMap::new();
                        let styles_borrow = styles_cell.borrow();
                        let styles = styles_borrow.as_ref().unwrap_or(&empty_styles);

                        let mut stack = vec![(root, crate::css::matrix::Affine::identity())];
                        while let Some((layout_box, parent_matrix)) = stack.pop() {
                            let mut local_matrix = crate::css::matrix::Affine::identity();
                            if let Some(node_id) = layout_box.node
                                && let Some(style) = styles.get(&node_id)
                                && !style.reset_effects.transform.is_empty()
                            {
                                let cx =
                                    layout_box.rect.origin.x + layout_box.rect.size.width / 2.0;
                                let cy =
                                    layout_box.rect.origin.y + layout_box.rect.size.height / 2.0;
                                let trans_origin = crate::css::matrix::Affine::translate(cx, cy);
                                let trans_origin_inv =
                                    crate::css::matrix::Affine::translate(-cx, -cy);
                                let matrix_fns = crate::css::matrix::Affine::from_transform_fns(
                                    &style.reset_effects.transform,
                                );
                                local_matrix = trans_origin
                                    .multiply(&matrix_fns)
                                    .multiply(&trans_origin_inv);
                            }

                            let accumulated_matrix = parent_matrix.multiply(&local_matrix);

                            if layout_box.node == Some(node) {
                                let x0 = layout_box.rect.origin.x;
                                let y0 = layout_box.rect.origin.y;
                                let w = layout_box.rect.size.width;
                                let h = layout_box.rect.size.height;

                                let p1 = accumulated_matrix.apply_point(x0, y0);
                                let p2 = accumulated_matrix.apply_point(x0 + w, y0);
                                let p3 = accumulated_matrix.apply_point(x0, y0 + h);
                                let p4 = accumulated_matrix.apply_point(x0 + w, y0 + h);

                                let xs = [p1.0, p2.0, p3.0, p4.0];
                                let ys = [p1.1, p2.1, p3.1, p4.1];

                                let mut min_x = xs[0];
                                let mut max_x = xs[0];
                                let mut min_y = ys[0];
                                let mut max_y = ys[0];

                                for i in 1..4 {
                                    if xs[i] < min_x {
                                        min_x = xs[i];
                                    }
                                    if xs[i] > max_x {
                                        max_x = xs[i];
                                    }
                                    if ys[i] < min_y {
                                        min_y = ys[i];
                                    }
                                    if ys[i] > max_y {
                                        max_y = ys[i];
                                    }
                                }

                                rects.push(DomRect::new(
                                    min_x as f64,
                                    min_y as f64,
                                    (max_x - min_x) as f64,
                                    (max_y - min_y) as f64,
                                ));
                            }

                            for child in layout_box.children.iter().rev() {
                                stack.push((child, accumulated_matrix));
                            }
                        }
                    });
                }
            });

            if layout_active {
                DomRectList::new(rects)
            } else {
                // Backward-compatibility fallback: return a single zero-rect
                DomRectList::new(vec![DomRect::new(0.0, 0.0, 0.0, 0.0)])
            }
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

    #[test]
    fn test_domrect_setters_and_derived() {
        let mut rect = DomRect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x(), 10.0);
        assert_eq!(rect.y(), 20.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);

        // Test mutators
        rect.set_x(15.0);
        rect.set_y(25.0);
        rect.set_width(120.0);
        rect.set_height(60.0);

        assert_eq!(rect.x(), 15.0);
        assert_eq!(rect.y(), 25.0);
        assert_eq!(rect.width(), 120.0);
        assert_eq!(rect.height(), 60.0);

        assert_eq!(rect.left(), 15.0);
        assert_eq!(rect.top(), 25.0);
        assert_eq!(rect.right(), 135.0);
        assert_eq!(rect.bottom(), 85.0);

        // Test mutators with negative dimensions
        rect.set_width(-120.0);
        rect.set_height(-60.0);

        assert_eq!(rect.left(), -105.0); // min(15.0, -105.0)
        assert_eq!(rect.top(), -35.0); // min(25.0, -35.0)
        assert_eq!(rect.right(), 15.0); // max(15.0, -105.0)
        assert_eq!(rect.bottom(), 25.0); // max(25.0, -35.0)
    }

    #[test]
    fn test_domrectreadonly_basic() {
        let readonly = DomRectReadOnly::new(5.0, 10.0, 50.0, 30.0);
        assert_eq!(readonly.x(), 5.0);
        assert_eq!(readonly.y(), 10.0);
        assert_eq!(readonly.width(), 50.0);
        assert_eq!(readonly.height(), 30.0);

        assert_eq!(readonly.left(), 5.0);
        assert_eq!(readonly.top(), 10.0);
        assert_eq!(readonly.right(), 55.0);
        assert_eq!(readonly.bottom(), 40.0);

        // Test serialization
        let s = readonly.serialize();
        assert_eq!(s["x"], 5.0);
        assert_eq!(s["y"], 10.0);
        assert_eq!(s["width"], 50.0);
        assert_eq!(s["height"], 30.0);

        // Test JSON aliases
        let json_val = readonly.to_json();
        assert_eq!(json_val["x"], 5.0);
        let json_camel = readonly.toJSON();
        assert_eq!(json_camel["x"], 5.0);

        // Test negative dimensions
        let readonly_neg = DomRectReadOnly::new(5.0, 10.0, -50.0, -30.0);
        assert_eq!(readonly_neg.left(), -45.0);
        assert_eq!(readonly_neg.top(), -20.0);
        assert_eq!(readonly_neg.right(), 5.0);
        assert_eq!(readonly_neg.bottom(), 10.0);

        // Test from_rect
        let init = serde_json::json!({
            "x": 8.0,
            "y": 12.0,
            "width": 80.0,
            "height": 40.0
        });
        let from_json = DomRectReadOnly::from_rect(Some(&init));
        assert_eq!(from_json.x(), 8.0);
        assert_eq!(from_json.y(), 12.0);
        assert_eq!(from_json.width(), 80.0);
        assert_eq!(from_json.height(), 40.0);

        // Test fromRect alias
        let from_json_camel = DomRectReadOnly::fromRect(Some(&init));
        assert_eq!(from_json_camel.x(), 8.0);

        // Test empty/None from_rect
        let from_none = DomRectReadOnly::from_rect(None);
        assert_eq!(from_none.x(), 0.0);
        assert_eq!(from_none.y(), 0.0);
        assert_eq!(from_none.width(), 0.0);
        assert_eq!(from_none.height(), 0.0);
    }

    #[test]
    fn test_conversions() {
        let rect = DomRect::new(2.0, 4.0, 20.0, 10.0);
        let readonly = rect.to_readonly();
        assert_eq!(readonly.x(), 2.0);
        assert_eq!(readonly.y(), 4.0);
        assert_eq!(readonly.width(), 20.0);
        assert_eq!(readonly.height(), 10.0);

        let mutable = readonly.to_mutable();
        assert_eq!(mutable.x(), 2.0);
        assert_eq!(mutable.y(), 4.0);
        assert_eq!(mutable.width(), 20.0);
        assert_eq!(mutable.height(), 10.0);

        // Test From traits
        let from_mutable: DomRectReadOnly = DomRectReadOnly::from(rect);
        assert_eq!(from_mutable.x(), 2.0);

        let from_readonly: DomRect = DomRect::from(readonly);
        assert_eq!(from_readonly.x(), 2.0);
    }

    #[test]
    fn test_geom_rect_conversions() {
        let g_rect = crate::geom::Rect::new(1.0, 2.0, 3.0, 4.0);

        let dom_rect: DomRect = DomRect::from(g_rect);
        assert_eq!(dom_rect.x(), 1.0);
        assert_eq!(dom_rect.y(), 2.0);
        assert_eq!(dom_rect.width(), 3.0);
        assert_eq!(dom_rect.height(), 4.0);

        let dom_readonly_rect: DomRectReadOnly = DomRectReadOnly::from(g_rect);
        assert_eq!(dom_readonly_rect.x(), 1.0);
        assert_eq!(dom_readonly_rect.y(), 2.0);
        assert_eq!(dom_readonly_rect.width(), 3.0);
        assert_eq!(dom_readonly_rect.height(), 4.0);

        let back_g_rect: crate::geom::Rect = crate::geom::Rect::from(dom_rect);
        assert_eq!(back_g_rect.origin.x, 1.0);
        assert_eq!(back_g_rect.origin.y, 2.0);
        assert_eq!(back_g_rect.size.width, 3.0);
        assert_eq!(back_g_rect.size.height, 4.0);

        let back_g_rect2: crate::geom::Rect = crate::geom::Rect::from(dom_readonly_rect);
        assert_eq!(back_g_rect2.origin.x, 1.0);
        assert_eq!(back_g_rect2.origin.y, 2.0);
        assert_eq!(back_g_rect2.size.width, 3.0);
        assert_eq!(back_g_rect2.size.height, 4.0);
    }

    #[test]
    fn test_t0909_geometry_enhancements() {
        // test contains_point
        let rect = DomRect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains_point(15.0, 25.0));
        assert!(rect.contains_point(10.0, 20.0)); // boundary
        assert!(rect.contains_point(110.0, 70.0)); // boundary
        assert!(!rect.contains_point(5.0, 25.0));
        assert!(!rect.contains_point(15.0, 75.0));
        assert!(!rect.contains_point(f64::NAN, 25.0));

        let rect_neg = DomRectReadOnly::new(10.0, 20.0, -100.0, -50.0);
        // left: -90.0, right: 10.0, top: -30.0, bottom: 20.0
        assert!(rect_neg.contains_point(0.0, 0.0));
        assert!(rect_neg.contains_point(-90.0, -30.0));
        assert!(!rect_neg.contains_point(15.0, 0.0));

        // test intersects
        let r1 = DomRect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = DomRect::new(5.0, 5.0, 10.0, 10.0);
        let r3 = DomRect::new(20.0, 20.0, 5.0, 5.0);
        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));

        let r_neg1 = DomRectReadOnly::new(0.0, 0.0, -10.0, -10.0); // -10 to 0
        let r_neg2 = DomRectReadOnly::new(-5.0, -5.0, -10.0, -10.0); // -15 to -5
        let r_neg3 = DomRectReadOnly::new(5.0, 5.0, -2.0, -2.0); // 3 to 5
        assert!(r_neg1.intersects(&r_neg2));
        assert!(!r_neg1.intersects(&r_neg3));

        // test union
        let u = r1.union(&r2);
        assert_eq!(u.x(), 0.0);
        assert_eq!(u.y(), 0.0);
        assert_eq!(u.width(), 15.0);
        assert_eq!(u.height(), 15.0);

        let u_neg = r_neg1.union(&r_neg2);
        assert_eq!(u_neg.left(), -15.0);
        assert_eq!(u_neg.right(), 0.0);
        assert_eq!(u_neg.top(), -15.0);
        assert_eq!(u_neg.bottom(), 0.0);

        // test translate & scale
        let t = r1.translate(2.0, 3.0);
        assert_eq!(t.x(), 2.0);
        assert_eq!(t.y(), 3.0);
        assert_eq!(t.width(), 10.0);
        assert_eq!(t.height(), 10.0);

        let s = r1.scale(2.0, 3.0);
        assert_eq!(s.x(), 0.0);
        assert_eq!(s.y(), 0.0);
        assert_eq!(s.width(), 20.0);
        assert_eq!(s.height(), 30.0);

        // test DomRectList iter, into_iter, extend
        let list1 = DomRectList::new(vec![r1, r2]);
        let mut count = 0;
        for _r in &list1 {
            count += 1;
        }
        assert_eq!(count, 2);

        let mut list2 = DomRectList::default();
        list2.extend(list1);
        assert_eq!(list2.length(), 2);

        let mut count_owned = 0;
        for _r in list2 {
            count_owned += 1;
        }
        assert_eq!(count_owned, 2);
    }

    #[test]
    fn test_t0889_nan_handling() {
        // According to CSS Geometry Interfaces spec, NaN width/height propagates to top, bottom, left, right as NaN
        let rect_nan_w = DomRect::new(10.0, 20.0, f64::NAN, 50.0);
        assert!(rect_nan_w.left().is_nan());
        assert!(rect_nan_w.right().is_nan());
        assert_eq!(rect_nan_w.top(), 20.0);
        assert_eq!(rect_nan_w.bottom(), 70.0);

        let rect_nan_h = DomRect::new(10.0, 20.0, 100.0, f64::NAN);
        assert_eq!(rect_nan_h.left(), 10.0);
        assert_eq!(rect_nan_h.right(), 110.0);
        assert!(rect_nan_h.top().is_nan());
        assert!(rect_nan_h.bottom().is_nan());

        let readonly_nan_w = DomRectReadOnly::new(10.0, 20.0, f64::NAN, 50.0);
        assert!(readonly_nan_w.left().is_nan());
        assert!(readonly_nan_w.right().is_nan());
        assert_eq!(readonly_nan_w.top(), 20.0);
        assert_eq!(readonly_nan_w.bottom(), 70.0);
    }

    #[test]
    fn test_t0889_coercion_parsing() {
        let init = serde_json::json!({
            "x": "12.5",
            "y": true,
            "width": null,
            "height": false
        });
        let rect = DomRect::from_rect(Some(&init));
        assert_eq!(rect.x(), 12.5);
        assert_eq!(rect.y(), 1.0);
        assert_eq!(rect.width(), 0.0);
        assert_eq!(rect.height(), 0.0);

        let init_invalid = serde_json::json!({
            "x": "not a number"
        });
        let rect_invalid = DomRect::from_rect(Some(&init_invalid));
        assert!(rect_invalid.x().is_nan());
        assert_eq!(rect_invalid.y(), 0.0);
    }

    #[test]
    fn test_t0889_trait_implementations() {
        // Default traits
        let def_rect = DomRect::default();
        assert_eq!(def_rect.x(), 0.0);
        assert_eq!(def_rect.y(), 0.0);

        let def_readonly = DomRectReadOnly::default();
        assert_eq!(def_readonly.x(), 0.0);

        let def_list = DomRectList::default();
        assert_eq!(def_list.length(), 0);

        // Index trait
        let r1 = DomRect::new(1.0, 2.0, 3.0, 4.0);
        let r2 = DomRect::new(5.0, 6.0, 7.0, 8.0);
        let list = DomRectList::new(vec![r1, r2]);
        assert_eq!(list[0], r1);
        assert_eq!(list[1], r2);

        // FromIterator trait
        let collected_list: DomRectList = vec![r1, r2].into_iter().collect();
        assert_eq!(collected_list.length(), 2);
        assert_eq!(collected_list[0], r1);
    }

    #[test]
    fn test_dompoint_basic() {
        let mut pt = DomPoint::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(pt.x(), 1.0);
        assert_eq!(pt.y(), 2.0);
        assert_eq!(pt.z(), 3.0);
        assert_eq!(pt.w(), 4.0);

        pt.set_x(10.0);
        pt.set_y(20.0);
        pt.set_z(30.0);
        pt.set_w(40.0);
        assert_eq!(pt.x(), 10.0);
        assert_eq!(pt.y(), 20.0);
        assert_eq!(pt.z(), 30.0);
        assert_eq!(pt.w(), 40.0);

        let pt_readonly = pt.to_readonly();
        assert_eq!(pt_readonly.x(), 10.0);
        assert_eq!(pt_readonly.y(), 20.0);
        assert_eq!(pt_readonly.z(), 30.0);
        assert_eq!(pt_readonly.w(), 40.0);

        let pt_mut = pt_readonly.to_mutable();
        assert_eq!(pt_mut, pt);
    }

    #[test]
    fn test_dompoint_defaults() {
        let pt_def = DomPoint::default();
        assert_eq!(pt_def.x(), 0.0);
        assert_eq!(pt_def.y(), 0.0);
        assert_eq!(pt_def.z(), 0.0);
        assert_eq!(pt_def.w(), 1.0);

        let pt_ro_def = DomPointReadOnly::default();
        assert_eq!(pt_ro_def.x(), 0.0);
        assert_eq!(pt_ro_def.y(), 0.0);
        assert_eq!(pt_ro_def.z(), 0.0);
        assert_eq!(pt_ro_def.w(), 1.0);
    }

    #[test]
    fn test_dompoint_json() {
        let init = serde_json::json!({
            "x": 5.0,
            "y": 10.0,
            "z": 15.0,
            "w": 20.0
        });
        let pt = DomPoint::from_point(Some(&init));
        assert_eq!(pt.x(), 5.0);
        assert_eq!(pt.y(), 10.0);
        assert_eq!(pt.z(), 15.0);
        assert_eq!(pt.w(), 20.0);

        let serialized = pt.serialize();
        assert_eq!(serialized["x"], 5.0);
        assert_eq!(serialized["y"], 10.0);
        assert_eq!(serialized["z"], 15.0);
        assert_eq!(serialized["w"], 20.0);

        let pt_camel = DomPointReadOnly::fromPoint(Some(&init));
        assert_eq!(pt_camel.x(), 5.0);
    }

    #[test]
    fn test_domquad_basic() {
        let p1 = DomPoint::new(0.0, 0.0, 0.0, 1.0);
        let p2 = DomPoint::new(10.0, 0.0, 0.0, 1.0);
        let p3 = DomPoint::new(10.0, 20.0, 0.0, 1.0);
        let p4 = DomPoint::new(0.0, 20.0, 0.0, 1.0);

        let quad = DomQuad::new(p1, p2, p3, p4);
        assert_eq!(quad.p1, p1);
        assert_eq!(quad.p2, p2);
        assert_eq!(quad.p3, p3);
        assert_eq!(quad.p4, p4);

        let bounds = quad.bounds();
        assert_eq!(bounds.x(), 0.0);
        assert_eq!(bounds.y(), 0.0);
        assert_eq!(bounds.width(), 10.0);
        assert_eq!(bounds.height(), 20.0);
    }

    #[test]
    fn test_domquad_from_rect() {
        let rect = DomRect::new(5.0, 10.0, 50.0, 100.0);
        let quad = DomQuad::from_rect_readonly(rect.to_readonly());

        assert_eq!(quad.p1, DomPoint::new(5.0, 10.0, 0.0, 1.0));
        assert_eq!(quad.p2, DomPoint::new(55.0, 10.0, 0.0, 1.0));
        assert_eq!(quad.p3, DomPoint::new(55.0, 110.0, 0.0, 1.0));
        assert_eq!(quad.p4, DomPoint::new(5.0, 110.0, 0.0, 1.0));

        let bounds = quad.bounds();
        assert_eq!(bounds.x(), 5.0);
        assert_eq!(bounds.y(), 10.0);
        assert_eq!(bounds.width(), 50.0);
        assert_eq!(bounds.height(), 100.0);
    }

    #[test]
    fn test_domquad_json() {
        let init = serde_json::json!({
            "p1": {"x": 1.0, "y": 2.0},
            "p2": {"x": 3.0, "y": 4.0},
            "p3": {"x": 5.0, "y": 6.0},
            "p4": {"x": 7.0, "y": 8.0}
        });
        let quad = DomQuad::from_quad(Some(&init));
        assert_eq!(quad.p1.x(), 1.0);
        assert_eq!(quad.p1.y(), 2.0);
        assert_eq!(quad.p2.x(), 3.0);
        assert_eq!(quad.p2.y(), 4.0);
        assert_eq!(quad.p3.x(), 5.0);
        assert_eq!(quad.p3.y(), 6.0);
        assert_eq!(quad.p4.x(), 7.0);
        assert_eq!(quad.p4.y(), 8.0);

        let serialized = quad.serialize();
        assert_eq!(serialized["p1"]["x"], 1.0);
        assert_eq!(serialized["p2"]["y"], 4.0);
    }

    #[test]
    fn test_domquad_nan_propagation() {
        let p1 = DomPoint::new(f64::NAN, 0.0, 0.0, 1.0);
        let p2 = DomPoint::new(10.0, 0.0, 0.0, 1.0);
        let p3 = DomPoint::new(10.0, 20.0, 0.0, 1.0);
        let p4 = DomPoint::new(0.0, 20.0, 0.0, 1.0);

        let quad = DomQuad::new(p1, p2, p3, p4);
        let bounds = quad.bounds();
        assert!(bounds.x().is_nan());
        assert!(bounds.width().is_nan());
        assert_eq!(bounds.y(), 0.0);
        assert_eq!(bounds.height(), 20.0);
    }

    #[test]
    fn test_is_empty() {
        let r1 = DomRect::new(0.0, 0.0, 10.0, 10.0);
        assert!(!r1.is_empty());

        let r2 = DomRect::new(0.0, 0.0, 0.0, 10.0);
        assert!(r2.is_empty());

        let r3 = DomRectReadOnly::new(0.0, 0.0, 10.0, -5.0);
        assert!(r3.is_empty());

        let r4 = DomRect::new(0.0, 0.0, f64::NAN, 10.0);
        assert!(r4.is_empty());
    }

    #[test]
    fn test_normalize() {
        let r = DomRect::new(10.0, 20.0, -100.0, -50.0);
        let norm = r.normalize();
        assert_eq!(norm.x(), -90.0);
        assert_eq!(norm.y(), -30.0);
        assert_eq!(norm.width(), 100.0);
        assert_eq!(norm.height(), 50.0);
        assert!(!norm.is_empty());

        let ro = DomRectReadOnly::new(10.0, 20.0, -100.0, -50.0);
        let norm_ro = ro.normalize();
        assert_eq!(norm_ro.x(), -90.0);
        assert_eq!(norm_ro.width(), 100.0);

        // One dimension is NaN
        let r_nan_w = DomRect::new(10.0, 20.0, f64::NAN, -50.0);
        let norm_nan_w = r_nan_w.normalize();
        assert!(norm_nan_w.width().is_nan());
        assert_eq!(norm_nan_w.x(), 10.0);
        assert_eq!(norm_nan_w.height(), 50.0);
        assert_eq!(norm_nan_w.y(), -30.0);

        let r_nan_h = DomRectReadOnly::new(10.0, 20.0, -100.0, f64::NAN);
        let norm_nan_h = r_nan_h.normalize();
        assert!(norm_nan_h.height().is_nan());
        assert_eq!(norm_nan_h.y(), 20.0);
        assert_eq!(norm_nan_h.width(), 100.0);
        assert_eq!(norm_nan_h.x(), -90.0);
    }

    #[test]
    fn test_intersection() {
        let r1 = DomRect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = DomRect::new(5.0, 5.0, 10.0, 10.0);
        let inter = r1.intersection(&r2);
        assert_eq!(inter.x(), 5.0);
        assert_eq!(inter.y(), 5.0);
        assert_eq!(inter.width(), 5.0);
        assert_eq!(inter.height(), 5.0);

        let r3 = DomRect::new(20.0, 20.0, 10.0, 10.0);
        let inter_empty = r1.intersection(&r3);
        assert_eq!(inter_empty, DomRect::default());
    }

    #[test]
    fn test_from_rect_array_parsing() {
        let arr_4 = serde_json::json!([10.0, 20.0, 30.0, 40.0]);
        let r4 = DomRect::from_rect(Some(&arr_4));
        assert_eq!(r4.x(), 10.0);
        assert_eq!(r4.y(), 20.0);
        assert_eq!(r4.width(), 30.0);
        assert_eq!(r4.height(), 40.0);

        let arr_2 = serde_json::json!([50.0, 60.0]);
        let r2 = DomRectReadOnly::from_rect(Some(&arr_2));
        assert_eq!(r2.x(), 0.0);
        assert_eq!(r2.y(), 0.0);
        assert_eq!(r2.width(), 50.0);
        assert_eq!(r2.height(), 60.0);

        let arr_3 = serde_json::json!([5.0, 10.0, 15.0]);
        let r3 = DomRect::from_rect(Some(&arr_3));
        assert_eq!(r3.x(), 5.0);
        assert_eq!(r3.y(), 10.0);
        assert_eq!(r3.width(), 15.0);
        assert_eq!(r3.height(), 0.0);
    }

    #[test]
    fn test_domrectlist_conversions_and_json() {
        let r1 = DomRect::new(0.0, 0.0, 10.0, 10.0);
        let r2 = DomRect::new(5.0, 5.0, 5.0, 5.0);
        let list = DomRectList::from(vec![r1, r2]);
        assert_eq!(list.length(), 2);

        let vec_back = Vec::<DomRect>::from(list.clone());
        assert_eq!(vec_back.len(), 2);

        let json = list.to_json();
        assert!(json.is_array());
        assert_eq!(json[0]["width"], 10.0);

        let json_camel = list.toJSON();
        assert_eq!(json_camel[1]["width"], 5.0);
    }

    #[test]
    fn test_domquad_from_conversions() {
        let r = DomRect::new(10.0, 20.0, 30.0, 40.0);
        let quad1: DomQuad = DomQuad::from(r);
        assert_eq!(quad1.bounds().x(), 10.0);
        assert_eq!(quad1.bounds().width(), 30.0);

        let ro = DomRectReadOnly::new(5.0, 15.0, 25.0, 35.0);
        let quad2: DomQuad = DomQuad::from(ro);
        assert_eq!(quad2.bounds().x(), 5.0);
        assert_eq!(quad2.bounds().width(), 25.0);

        let geom_r = crate::geom::Rect::new(2.0, 4.0, 6.0, 8.0);
        let quad3: DomQuad = DomQuad::from(geom_r);
        assert_eq!(quad3.bounds().x(), 2.0);
        assert_eq!(quad3.bounds().width(), 6.0);
    }

    #[test]
    fn test_dompoint_tuple_conversions() {
        let pt1 = DomPoint::from((3.0, 4.0));
        assert_eq!(pt1.x(), 3.0);
        assert_eq!(pt1.y(), 4.0);
        assert_eq!(pt1.z(), 0.0);
        assert_eq!(pt1.w(), 1.0);

        let tuple_2: (f64, f64) = pt1.into();
        assert_eq!(tuple_2, (3.0, 4.0));

        let pt2 = DomPointReadOnly::from((1.0, 2.0, 3.0, 4.0));
        assert_eq!(pt2.x(), 1.0);
        assert_eq!(pt2.y(), 2.0);
        assert_eq!(pt2.z(), 3.0);
        assert_eq!(pt2.w(), 4.0);

        let tuple_4: (f64, f64, f64, f64) = pt2.into();
        assert_eq!(tuple_4, (1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn test_t0985_serde_serialization_roundtrip() {
        let rect = DomRect::new(10.0, 20.0, -100.0, -50.0);
        let serialized_value = serde_json::to_value(rect).unwrap();
        assert_eq!(serialized_value["x"], 10.0);
        assert_eq!(serialized_value["y"], 20.0);
        assert_eq!(serialized_value["width"], -100.0);
        assert_eq!(serialized_value["height"], -50.0);
        assert_eq!(serialized_value["left"], -90.0);
        assert_eq!(serialized_value["top"], -30.0);
        assert_eq!(serialized_value["right"], 10.0);
        assert_eq!(serialized_value["bottom"], 20.0);

        // Deserialization of DomRect
        let deserialized_rect: DomRect = serde_json::from_value(serialized_value.clone()).unwrap();
        assert_eq!(deserialized_rect.x(), 10.0);
        assert_eq!(deserialized_rect.y(), 20.0);
        assert_eq!(deserialized_rect.width(), -100.0);
        assert_eq!(deserialized_rect.height(), -50.0);

        // Serialization of DomRectReadOnly
        let readonly = DomRectReadOnly::new(5.0, 15.0, 30.0, 40.0);
        let ro_serialized = serde_json::to_value(readonly).unwrap();
        assert_eq!(ro_serialized["x"], 5.0);
        assert_eq!(ro_serialized["y"], 15.0);
        assert_eq!(ro_serialized["width"], 30.0);
        assert_eq!(ro_serialized["height"], 40.0);
        assert_eq!(ro_serialized["left"], 5.0);
        assert_eq!(ro_serialized["top"], 15.0);
        assert_eq!(ro_serialized["right"], 35.0);
        assert_eq!(ro_serialized["bottom"], 55.0);

        // Deserialization of DomRectReadOnly
        let deserialized_ro: DomRectReadOnly = serde_json::from_value(ro_serialized).unwrap();
        assert_eq!(deserialized_ro.x(), 5.0);
        assert_eq!(deserialized_ro.y(), 15.0);
        assert_eq!(deserialized_ro.width(), 30.0);
        assert_eq!(deserialized_ro.height(), 40.0);
    }

    #[test]
    fn test_t0985_negative_dimensions_correctness() {
        // Detailed correctness tests of left, top, right, bottom for positive and negative combinations
        let cases = vec![
            // (x, y, width, height, expected_left, expected_top, expected_right, expected_bottom)
            (10.0, 20.0, 100.0, 50.0, 10.0, 20.0, 110.0, 70.0),
            (10.0, 20.0, -100.0, 50.0, -90.0, 20.0, 10.0, 70.0),
            (10.0, 20.0, 100.0, -50.0, 10.0, -30.0, 110.0, 20.0),
            (10.0, 20.0, -100.0, -50.0, -90.0, -30.0, 10.0, 20.0),
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            (-5.0, -15.0, -25.0, -35.0, -30.0, -50.0, -5.0, -15.0),
        ];

        for (x, y, w, h, el, et, er, eb) in cases {
            let rect_ro = DomRectReadOnly::new(x, y, w, h);
            assert_eq!(rect_ro.left(), el);
            assert_eq!(rect_ro.top(), et);
            assert_eq!(rect_ro.right(), er);
            assert_eq!(rect_ro.bottom(), eb);

            let rect_mut = DomRect::new(x, y, w, h);
            assert_eq!(rect_mut.left(), el);
            assert_eq!(rect_mut.top(), et);
            assert_eq!(rect_mut.right(), er);
            assert_eq!(rect_mut.bottom(), eb);
        }
    }

    #[test]
    fn test_get_bounding_client_rect_nested_and_transformed() {
        use crate::css::values::TransformFn;
        use crate::geom::Rect;
        use crate::layout::LayoutBox;
        use crate::style::CategorizedComputedStyle;
        use std::collections::HashMap;

        // 1. Set up DOM
        let mut dom = Dom::new();
        let document = dom.document();

        let parent_node = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(document, parent_node);

        let child_node = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(parent_node, child_node);

        // 2. Set up Styles
        let mut styles = HashMap::new();

        // Parent transform: scale(2.0, 3.0) at origin (50, 50) of parent_box (0,0, 100x100)
        let mut parent_style = CategorizedComputedStyle::default();
        std::sync::Arc::make_mut(&mut parent_style.reset_effects).transform =
            vec![TransformFn::Scale { x: 2.0, y: 3.0 }];
        styles.insert(parent_node, parent_style);

        // Child transform: translate(10.0, 20.0) -> then scale(1.5, 1.5) at origin (15, 25) of child_box (10,10, 10x30)
        let mut child_style = CategorizedComputedStyle::default();
        std::sync::Arc::make_mut(&mut child_style.reset_effects).transform = vec![
            TransformFn::Translate {
                x: crate::css::values::LengthOrPercent {
                    value: 10.0,
                    unit: crate::css::values::LengthUnit::Px,
                },
                y: crate::css::values::LengthOrPercent {
                    value: 20.0,
                    unit: crate::css::values::LengthUnit::Px,
                },
            },
            TransformFn::Scale { x: 1.5, y: 1.5 },
        ];
        styles.insert(child_node, child_style);

        // 3. Set up LayoutBox tree
        // Parent box at (0.0, 0.0, 100.0, 100.0)
        // Child box at (10.0, 10.0, 10.0, 30.0)
        let child_box = LayoutBox {
            node: Some(child_node),
            rect: Rect::new(10.0, 10.0, 10.0, 30.0),
            children: Vec::new(),
            text: None,
        };

        let parent_box = LayoutBox {
            node: Some(parent_node),
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            children: vec![child_box],
            text: None,
        };

        // 4. Set thread locals
        CURRENT_LAYOUT.with(|cell| {
            *cell.borrow_mut() = Some(parent_box);
        });
        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = Some(styles);
        });

        // 5. Query and Assert
        let parent_rect = dom.get_bounding_client_rect(parent_node);
        assert_eq!(parent_rect.x(), -50.0);
        assert_eq!(parent_rect.y(), -100.0);
        assert_eq!(parent_rect.width(), 200.0);
        assert_eq!(parent_rect.height(), 300.0);

        let child_rect = dom.get_bounding_client_rect(child_node);
        assert_eq!(child_rect.x(), -15.0);
        assert_eq!(child_rect.y(), -32.5);
        assert_eq!(child_rect.width(), 30.0);
        assert_eq!(child_rect.height(), 135.0);

        // Clean up thread locals
        CURRENT_LAYOUT.with(|cell| {
            *cell.borrow_mut() = None;
        });
        CURRENT_STYLES.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }

    #[test]
    fn test_get_client_rects_multiple_fragments() {
        use crate::geom::Rect;
        use crate::layout::LayoutBox;

        let mut dom = Dom::new();
        let document = dom.document();

        let elem_node = dom.create_node(crate::dom::NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(document, elem_node);

        // Multiple layout fragments for the span
        let frag1 = LayoutBox {
            node: Some(elem_node),
            rect: Rect::new(10.0, 10.0, 50.0, 20.0),
            children: Vec::new(),
            text: None,
        };
        let frag2 = LayoutBox {
            node: Some(elem_node),
            rect: Rect::new(10.0, 40.0, 30.0, 20.0),
            children: Vec::new(),
            text: None,
        };

        let root_box = LayoutBox {
            node: Some(document),
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            children: vec![frag1, frag2],
            text: None,
        };

        CURRENT_LAYOUT.with(|cell| {
            *cell.borrow_mut() = Some(root_box);
        });

        let rects = dom.get_client_rects(elem_node);
        assert_eq!(rects.length(), 2);
        assert_eq!(rects.item(0).unwrap().x(), 10.0);
        assert_eq!(rects.item(0).unwrap().width(), 50.0);
        assert_eq!(rects.item(1).unwrap().x(), 10.0);
        assert_eq!(rects.item(1).unwrap().y(), 40.0);

        let bounding = dom.get_bounding_client_rect(elem_node);
        // Union of (10, 10, 50, 20) and (10, 40, 30, 20)
        // x range: [10, 60]
        // y range: [10, 60]
        assert_eq!(bounding.x(), 10.0);
        assert_eq!(bounding.y(), 10.0);
        assert_eq!(bounding.width(), 50.0);
        assert_eq!(bounding.height(), 50.0);

        CURRENT_LAYOUT.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }
}
