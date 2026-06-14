//! DOM mutation API (WHATWG DOM): attribute and tree edits. These are the
//! operations scripting, forms and event handlers use to update the tree.
//!
//! All methods tolerate invalid / stale `NodeId`s and non-matching node kinds
//! gracefully (no panic — I-6); they simply do nothing or return `None`.

use super::{Dom, NodeData};
use crate::infra::NodeId;

impl Dom {
    /// Sets (adds or overwrites) an attribute on an element node.
    /// No-op for non-element nodes or invalid ids.
    // spec: https://dom.spec.whatwg.org/#dom-element-setattribute
    pub fn set_attribute(&mut self, node: NodeId, name: &str, value: &str) {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { attrs, .. } = &mut n.data
        {
            if let Some(pair) = attrs.iter_mut().find(|(k, _)| k == name) {
                if pair.1 != value {
                    pair.1 = value.to_string();
                    changed = true;
                }
            } else {
                attrs.push((name.to_string(), value.to_string()));
                changed = true;
            }
        }
        if changed {
            self.mark_dirty(node);
        }
    }

    /// Removes an attribute from an element node.
    /// No-op for non-element nodes or invalid ids.
    // spec: https://dom.spec.whatwg.org/#dom-element-removeattribute
    pub fn remove_attribute(&mut self, node: NodeId, name: &str) {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { attrs, .. } = &mut n.data
        {
            let before = attrs.len();
            attrs.retain(|(k, _)| k != name);
            if attrs.len() != before {
                changed = true;
            }
        }
        if changed {
            self.mark_dirty(node);
        }
    }

    /// Returns the value of an element's attribute, or `None`.
    // spec: https://dom.spec.whatwg.org/#dom-element-getattribute
    pub fn get_attribute(&self, node: NodeId, name: &str) -> Option<&str> {
        match &self.arena.get(node)?.data {
            NodeData::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Removes `child` from `parent`'s child list and clears its parent link.
    /// No-op if `child` is not a child of `parent`.
    // spec: https://dom.spec.whatwg.org/#dom-node-removechild
    pub fn remove_child(&mut self, parent: NodeId, child: NodeId) {
        let removed = if let Some(p) = self.arena.get_mut(parent) {
            let before = p.children.len();
            p.children.retain(|&c| c != child);
            p.children.len() != before
        } else {
            false
        };
        if removed && let Some(c) = self.arena.get_mut(child) {
            c.parent = None;
        }
        if removed {
            self.mark_dirty(parent);
        }
    }

    /// Inserts `child` into `parent` before `reference` (or appends when
    /// `reference` is `None`). `child` is first detached from any old parent.
    // spec: https://dom.spec.whatwg.org/#dom-node-insertbefore
    pub fn insert_before(&mut self, parent: NodeId, child: NodeId, reference: Option<NodeId>) {
        // Detach from the previous parent, if any.
        if let Some(old) = self.parent(child)
            && let Some(op) = self.arena.get_mut(old)
        {
            op.children.retain(|&c| c != child);
        }

        let inserted = if let Some(p) = self.arena.get_mut(parent) {
            let idx = match reference {
                Some(r) => p
                    .children
                    .iter()
                    .position(|&c| c == r)
                    .unwrap_or(p.children.len()),
                None => p.children.len(),
            };
            p.children.insert(idx, child);
            true
        } else {
            false
        };
        if inserted && let Some(c) = self.arena.get_mut(child) {
            c.parent = Some(parent);
        }
        if inserted {
            self.mark_dirty(parent);
        }
    }

    /// Replaces the text of a `Text` or `Comment` node. No-op for other node kinds.
    pub fn set_text(&mut self, node: NodeId, text: &str) {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node) {
            match &mut n.data {
                NodeData::Text(t) | NodeData::Comment(t) if t != text => {
                    *t = text.to_string();
                    changed = true;
                }
                _ => {}
            }
        }
        if changed {
            self.mark_dirty(node);
        }
    }

    /// Returns the current value of an `<input>` element.
    /// Returns `None` if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn get_input_value(&self, node: NodeId) -> Option<String> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            if n.input_value_dirty {
                // If dirty, return the current input_value or fallback to empty string
                let val = match &n.input_value {
                    Some(v) => v.clone(),
                    None => String::new(),
                };
                return Some(val);
            } else {
                // If not dirty, return the value of the "value" content attribute, or empty string
                let attr_val = attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                    .map(|(_, v)| v.clone())
                    .unwrap_or_default();
                return Some(attr_val);
            }
        }
        None
    }

    /// Returns the value of the `href` content attribute of a valid element node,
    /// but only if `href` is a defined attribute for its element tag (a, area, link, base).
    /// Returns `None` if the node is not one of those element tags, has no `href` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_href(&self, node: NodeId) -> Option<&str> {
        // TODO(spec): Resolving href against the document base URL is out of scope and belongs to a higher layer.
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("a")
                || name.eq_ignore_ascii_case("area")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("base");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("href"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `src` content attribute of a valid element node,
    /// but only if `src` is a defined attribute for its element tag (img, script, iframe, source, audio, video, embed, track, input).
    /// Returns `None` if the node is not one of those element tags, has no `src` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_src(&self, node: NodeId) -> Option<&str> {
        // TODO(spec): Resolving src against the document base URL is out of scope and belongs to a higher layer.
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("iframe")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("audio")
                || name.eq_ignore_ascii_case("video")
                || name.eq_ignore_ascii_case("embed")
                || name.eq_ignore_ascii_case("track")
                || name.eq_ignore_ascii_case("input");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("src"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `id` content attribute of a valid element node.
    /// Returns `None` if the node has no `id` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_id(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("id"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `class` content attribute of a valid element node.
    /// Returns `None` if the node has no `class` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_class_name(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("class"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `alt` content attribute of a valid element node.
    /// Returns `None` if the node has no `alt` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_alt(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("alt"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `srcset` content attribute of a valid element node,
    /// but only if `srcset` is a defined attribute for its element tag (img, source).
    /// Returns `None` if the node is not one of those element tags, has no `srcset` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_srcset(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined =
                name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("source");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("srcset"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `sizes` content attribute of a valid element node,
    /// but only if `sizes` is a defined attribute for its element tag (img, source, link).
    /// Returns `None` if the node is not one of those element tags, has no `sizes` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_sizes(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("link");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("sizes"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `decoding` content attribute of a valid element node,
    /// but only if `decoding` is a defined attribute for its element tag (img).
    /// Returns `None` if the node is not one of those element tags, has no `decoding` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_decoding(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("img");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("decoding"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `crossorigin` content attribute of a valid element node,
    /// but only if `crossorigin` is a defined attribute for its element tag (img, script, link, audio, video).
    /// Returns `None` if the node is not one of those element tags, has no `crossorigin` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_cross_origin(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("audio")
                || name.eq_ignore_ascii_case("video");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("crossorigin"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `placeholder` content attribute of a valid element node.
    /// Returns `None` if the node has no `placeholder` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_placeholder(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("placeholder"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `name` content attribute of a valid element node.
    /// Returns `None` if the node has no `name` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_name(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("name"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `title` content attribute of a valid element node.
    /// Returns `None` if the node has no `title` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_title(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("title"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `lang` content attribute of a valid element node.
    /// Returns `None` if the node has no `lang` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_lang(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("lang"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `rel` content attribute of a valid element node.
    /// Returns `None` if the node has no `rel` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_rel(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("rel"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `hreflang` content attribute of a valid element node.
    /// Returns `None` if the node has no `hreflang` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_hreflang(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("hreflang"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `download` content attribute of a valid element node.
    /// Returns `None` if the node has no `download` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_download(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("download"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `referrerpolicy` content attribute of a valid element node.
    /// Returns `None` if the node has no `referrerpolicy` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_referrer_policy(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("referrerpolicy"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `for` content attribute of a valid element node.
    /// Returns `None` if the node has no `for` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_for(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("for"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `dir` content attribute of a valid element node.
    /// Returns `None` if the node has no `dir` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_dir(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("dir"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `target` content attribute of a valid element node,
    /// but only if `target` is a defined attribute for its element tag (a, area, base, form).
    /// Returns `None` if the node is not one of those element tags, has no `target` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_target(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("a")
                || name.eq_ignore_ascii_case("area")
                || name.eq_ignore_ascii_case("base")
                || name.eq_ignore_ascii_case("form");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("target"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `type` content attribute of a valid element node,
    /// but only if `type` is a defined attribute for its element tag (button, input, embed, object, ol, script, source, style, link, menu, command).
    /// Returns `None` if the node is not one of those element tags, has no `type` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_type(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data {
            let is_defined = name.eq_ignore_ascii_case("button")
                || name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("embed")
                || name.eq_ignore_ascii_case("object")
                || name.eq_ignore_ascii_case("ol")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("style")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("menu")
                || name.eq_ignore_ascii_case("command");
            if is_defined {
                return attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("type"))
                    .map(|(_, v)| v.as_str());
            }
        }
        None
    }

    /// Returns the value of the `action` content attribute of a valid element node,
    /// but only if the tag is `form`.
    /// Returns `None` if the node is not a `form` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_action(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("action"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `method` content attribute of a valid element node,
    /// but only if the tag is `form`.
    /// Returns `None` if the node is not a `form` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_method(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("method"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `enctype` content attribute of a valid element node,
    /// but only if the tag is `form`.
    /// Returns `None` if the node is not a `form` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_enctype(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("enctype"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `width` content attribute of a valid element node,
    /// but only if the tag is `img` or `canvas`.
    /// Returns `None` if the node is not an `img` or `canvas` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_width(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("canvas"))
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("width"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `height` content attribute of a valid element node,
    /// but only if the tag is `img` or `canvas`.
    /// Returns `None` if the node is not an `img` or `canvas` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_height(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("canvas"))
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("height"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `max` content attribute of a valid element node,
    /// but only if the tag is `input`.
    /// Returns `None` if the node is not an `input` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_max(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("max"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `min` content attribute of a valid element node,
    /// but only if the tag is `input`.
    /// Returns `None` if the node is not an `input` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_min(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("min"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `step` content attribute of a valid element node,
    /// but only if the tag is `input`.
    /// Returns `None` if the node is not an `input` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_step(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("step"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `pattern` content attribute of a valid element node,
    /// but only if the tag is `input`.
    /// Returns `None` if the node is not an `input` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_pattern(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("pattern"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `accept` content attribute of a valid element node,
    /// but only if the tag is `input`.
    /// Returns `None` if the node is not an `input` element, or if the `NodeId` is invalid.
    /// If the attribute is absent, returns `Some("")`.
    pub fn get_accept(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(
                attrs
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case("accept"))
                    .map(|(_, v)| v.as_str())
                    .unwrap_or(""),
            );
        }
        None
    }

    /// Returns the value of the `colspan` content attribute of a valid table cell element (`<td>` or `<th>`).
    /// Returns `None` if the node is not a table cell element, has no `colspan` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_colspan(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("colspan"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `rowspan` content attribute of a valid table cell element (`<td>` or `<th>`).
    /// Returns `None` if the node is not a table cell element, has no `rowspan` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_rowspan(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("rowspan"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `headers` content attribute of a valid table cell element (`<td>` or `<th>`).
    /// Returns `None` if the node is not a table cell element, has no `headers` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_headers(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("headers"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `scope` content attribute of a valid `<th>` element.
    /// Returns `None` if the node is not a `<th>` element, has no `scope` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_scope(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("th")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("scope"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `abbr` content attribute of a valid `<th>` element.
    /// Returns `None` if the node is not a `<th>` element, has no `abbr` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_abbr(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("th")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("abbr"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `start` content attribute of a valid `<ol>` element.
    /// Returns `None` if the node is not an `<ol>` element, has no `start` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_start(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("ol")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("start"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `reversed` content attribute of a valid `<ol>` element.
    /// Returns `None` if the node is not an `<ol>` element, has no `reversed` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_reversed(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("ol")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("reversed"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `value` content attribute of a valid `<li>` element.
    /// Returns `None` if the node is not an `<li>` element, has no `value` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_value(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("li")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `content` content attribute of a valid `<meta>` element.
    /// Returns `None` if the node is not a `<meta>` element, has no `content` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_content(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("content"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `http-equiv` content attribute of a valid `<meta>` element.
    /// Returns `None` if the node is not a `<meta>` element, has no `http-equiv` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_http_equiv(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("http-equiv"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `charset` content attribute of a valid `<meta>` element.
    /// Returns `None` if the node is not a `<meta>` element, has no `charset` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_charset(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("charset"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `cite` content attribute of a valid `<blockquote>`, `<q>`, `<ins>`, or `<del>` element.
    /// Returns `None` if the node is not a matching element, has no `cite` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_cite(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("blockquote")
                || name.eq_ignore_ascii_case("q")
                || name.eq_ignore_ascii_case("ins")
                || name.eq_ignore_ascii_case("del"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("cite"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `datetime` content attribute of a valid `<time>`, `<ins>`, or `<del>` element.
    /// Returns `None` if the node is not a matching element, has no `datetime` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_datetime(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("time")
                || name.eq_ignore_ascii_case("ins")
                || name.eq_ignore_ascii_case("del"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("datetime"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `media` content attribute of a valid `<link>`, `<style>`, or `<source>` element.
    /// Returns `None` if the node is not a matching element, has no `media` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_media(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("style")
                || name.eq_ignore_ascii_case("source"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("media"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `wrap` content attribute of a valid `<textarea>` element.
    /// Returns `None` if the node is not a `<textarea>` element, has no `wrap` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_wrap(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("textarea")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("wrap"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the current value of an `<input>` element, marking it as dirty.
    /// No-op if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn set_input_value(&mut self, node: NodeId, value: &str) {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            n.input_value = Some(value.to_string());
            n.input_value_dirty = true;
            changed = true;
        }
        if changed {
            self.mark_dirty(node);
        }
    }

    /// Returns whether the `<input>` element's value has been modified (dirty flag).
    /// Returns `None` if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn is_input_value_dirty(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(n.input_value_dirty);
        }
        None
    }

    /// Sets the dirty flag of an `<input>` element.
    /// No-op if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn set_input_value_dirty(&mut self, node: NodeId, dirty: bool) {
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            n.input_value_dirty = dirty;
        }
    }

    /// Returns whether the `disabled` content attribute is present on a valid form control element
    /// (input, button, select, textarea, option, optgroup, fieldset).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_disabled(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("button")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("option")
                || name.eq_ignore_ascii_case("optgroup")
                || name.eq_ignore_ascii_case("fieldset"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("disabled")),
            );
        }
        None
    }

    /// Returns whether the `required` content attribute is present on a valid element node
    /// (input, select, textarea).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_required(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("textarea"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("required")),
            );
        }
        None
    }

    /// Returns whether the `readonly` content attribute is present on a valid element node
    /// (input, textarea).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_readonly(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("readonly")),
            );
        }
        None
    }

    /// Returns whether the `autofocus` content attribute is present on any valid element node.
    /// Returns `None` if the node is not an element node, or if the `NodeId` is invalid.
    pub fn get_autofocus(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("autofocus")),
            );
        }
        None
    }

    /// Returns whether the `multiple` content attribute is present on a valid element node
    /// (input, select).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_multiple(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("select"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("multiple")),
            );
        }
        None
    }

    /// Returns whether the `checked` content attribute is present on a valid `<input>` element.
    /// Returns `None` if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn get_checked(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            return Some(attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("checked")));
        }
        None
    }

    /// Returns whether the `selected` content attribute is present on a valid `<option>` element.
    /// Returns `None` if the node is not an `<option>` element, or if the `NodeId` is invalid.
    pub fn get_selected(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("option")
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("selected")),
            );
        }
        None
    }

    /// Returns the value of the `value` content attribute of a valid `<textarea>` element.
    /// Returns `None` if the node is not a `<textarea>` element, has no `value` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_textarea_value(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("textarea")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `value` content attribute of a valid `<button>` element.
    /// Returns `None` if the node is not a `<button>` element, has no `value` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_button_value(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("button")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `value` content attribute of a valid `<option>` element.
    /// Returns `None` if the node is not an `<option>` element, has no `value` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_option_value(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("option")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("value"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns whether the `controls` content attribute is present on a valid media element
    /// (audio, video).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_controls(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("controls")),
            );
        }
        None
    }

    /// Sets or removes the `controls` content attribute on a valid media element (audio, video).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_controls(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            if value {
                self.set_attribute(node, "controls", "");
            } else {
                self.remove_attribute(node, "controls");
            }
        }
    }

    /// Returns whether the `loop` content attribute is present on a valid media element
    /// (audio, video).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_loop(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            return Some(attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("loop")));
        }
        None
    }

    /// Sets or removes the `loop` content attribute on a valid media element (audio, video).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_loop(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            if value {
                self.set_attribute(node, "loop", "");
            } else {
                self.remove_attribute(node, "loop");
            }
        }
    }

    /// Returns the value of the `preload` content attribute of a valid media element
    /// (audio, video).
    /// Returns `None` if the node has no `preload` attribute, is not a media element,
    /// or if the `NodeId` is invalid.
    pub fn get_preload(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("preload"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `preload` content attribute on a valid media element (audio, video).
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_preload(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("audio") || name.eq_ignore_ascii_case("video"))
        {
            self.set_attribute(node, "preload", value);
        }
    }

    /// Returns the value of the `poster` content attribute of a valid video element.
    /// Returns `None` if the node has no `poster` attribute, is not a `<video>` element,
    /// or if the `NodeId` is invalid.
    pub fn get_poster(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("video")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("poster"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `poster` content attribute on a valid video element.
    /// No-op if the node is not a `<video>` element, or if the `NodeId` is invalid.
    pub fn set_poster(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("video")
        {
            self.set_attribute(node, "poster", value);
        }
    }

    /// Returns the value of the `kind` content attribute of a valid `<track>` element.
    /// Returns `None` if the node is not a `<track>` element, has no `kind` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_kind(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("kind"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `srclang` content attribute of a valid `<track>` element.
    /// Returns `None` if the node is not a `<track>` element, has no `srclang` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_srclang(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("srclang"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `label` content attribute of a valid `<track>` element.
    /// Returns `None` if the node is not a `<track>` element, has no `label` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_label(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("label"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `sandbox` content attribute of a valid `<iframe>` element.
    /// Returns `None` if the node is not an `<iframe>` element, has no `sandbox` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_sandbox(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("iframe")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("sandbox"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `allow` content attribute of a valid `<iframe>` element.
    /// Returns `None` if the node is not an `<iframe>` element, has no `allow` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_allow(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("iframe")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("allow"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `formaction` content attribute of a valid submit button element
    /// (input, button).
    /// Returns `None` if the node is not one of these elements, has no `formaction` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_formaction(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("formaction"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `formmethod` content attribute of a valid submit button element
    /// (input, button).
    /// Returns `None` if the node is not one of these elements, has no `formmethod` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_formmethod(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("formmethod"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `formtarget` content attribute of a valid submit button element
    /// (input, button).
    /// Returns `None` if the node is not one of these elements, has no `formtarget` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_formtarget(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("formtarget"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns whether the `formnovalidate` content attribute is present on a valid submit button element
    /// (input, button).
    /// Returns `None` if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn get_formnovalidate(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            return Some(
                attrs
                    .iter()
                    .any(|(k, _)| k.eq_ignore_ascii_case("formnovalidate")),
            );
        }
        None
    }

    /// Returns the value of the `inputmode` content attribute of a valid form control element
    /// (input, textarea, select, button).
    /// Returns `None` if the node is not one of these elements, has no `inputmode` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_inputmode(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("button"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("inputmode"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `enterkeyhint` content attribute of a valid form control element
    /// (input, textarea, select, button).
    /// Returns `None` if the node is not one of these elements, has no `enterkeyhint` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_enterkeyhint(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("button"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("enterkeyhint"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `dirname` content attribute of a valid form control element
    /// (input, textarea).
    /// Returns `None` if the node is not one of these elements, has no `dirname` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_dirname(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("dirname"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `maxlength` content attribute of a valid form control element
    /// (input, textarea).
    /// Returns `None` if the node is not one of these elements, has no `maxlength` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_maxlength(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("maxlength"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `minlength` content attribute of a valid form control element
    /// (input, textarea).
    /// Returns `None` if the node is not one of these elements, has no `minlength` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_minlength(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("minlength"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `size` content attribute of a valid form control element
    /// Returns the value of the `size` content attribute of a valid form control element
    /// (input, select).
    /// Returns `None` if the node is not one of these elements, has no `size` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_size(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("select"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("size"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Returns the value of the `loading` content attribute of a valid `img` or `iframe` element.
    /// Returns `None` if the node is not one of these elements, has no `loading` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_loading(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("iframe"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("loading"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `loading` content attribute on a valid `img` or `iframe` element.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_loading(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("iframe"))
        {
            self.set_attribute(node, "loading", value);
        }
    }

    /// Returns the value of the `ping` content attribute of a valid `a` or `area` element.
    /// Returns `None` if the node is not one of these elements, has no `ping` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_ping(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("a") || name.eq_ignore_ascii_case("area"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("ping"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `ping` content attribute on a valid `a` or `area` element.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_ping(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("a") || name.eq_ignore_ascii_case("area"))
        {
            self.set_attribute(node, "ping", value);
        }
    }

    /// Returns the value of the `coords` content attribute of a valid `area` element.
    /// Returns `None` if the node is not an `area` element, has no `coords` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_coords(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("area")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("coords"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `coords` content attribute on a valid `area` element.
    /// No-op if the node is not an `area` element, or if the `NodeId` is invalid.
    pub fn set_coords(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("area")
        {
            self.set_attribute(node, "coords", value);
        }
    }

    /// Returns the value of the `shape` content attribute of a valid `area` element.
    /// Returns `None` if the node is not an `area` element, has no `shape` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_shape(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("area")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("shape"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `shape` content attribute on a valid `area` element.
    /// No-op if the node is not an `area` element, or if the `NodeId` is invalid.
    pub fn set_shape(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("area")
        {
            self.set_attribute(node, "shape", value);
        }
    }

    /// Returns the value of the `as` content attribute of a valid `link` element.
    /// Returns `None` if the node is not a `link` element, has no `as` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_as(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("as"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `as` content attribute on a valid `link` element.
    /// No-op if the node is not a `link` element, or if the `NodeId` is invalid.
    pub fn set_as(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            self.set_attribute(node, "as", value);
        }
    }

    /// Returns the value of the `imagesrcset` content attribute of a valid `link` element.
    /// Returns `None` if the node is not a `link` element, has no `imagesrcset` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_image_srcset(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("imagesrcset"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `imagesrcset` content attribute on a valid `link` element.
    /// No-op if the node is not a `link` element, or if the `NodeId` is invalid.
    pub fn set_image_srcset(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            self.set_attribute(node, "imagesrcset", value);
        }
    }

    /// Returns the value of the `imagesizes` content attribute of a valid `link` element.
    /// Returns `None` if the node is not a `link` element, has no `imagesizes` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_image_sizes(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("imagesizes"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `imagesizes` content attribute on a valid `link` element.
    /// No-op if the node is not a `link` element, or if the `NodeId` is invalid.
    pub fn set_image_sizes(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("link")
        {
            self.set_attribute(node, "imagesizes", value);
        }
    }

    /// Sets the `media` content attribute on a valid `<link>`, `<style>`, or `<source>` element.
    /// No-op if the node is not a matching element, or if the `NodeId` is invalid.
    pub fn set_media(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("style")
                || name.eq_ignore_ascii_case("source"))
        {
            self.set_attribute(node, "media", value);
        }
    }

    /// Sets the `hreflang` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_hreflang(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "hreflang", value);
        }
    }

    /// Sets the `title` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_title(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "title", value);
        }
    }

    /// Sets the `lang` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_lang(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "lang", value);
        }
    }

    /// Sets the `dir` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_dir(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "dir", value);
        }
    }

    /// Sets the `target` content attribute on a valid element node (a, area, base, form).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_target(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined = name.eq_ignore_ascii_case("a")
                || name.eq_ignore_ascii_case("area")
                || name.eq_ignore_ascii_case("base")
                || name.eq_ignore_ascii_case("form");
            if is_defined {
                self.set_attribute(node, "target", value);
            }
        }
    }

    /// Sets the `rel` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_rel(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "rel", value);
        }
    }

    /// Sets the `formtarget` content attribute on a valid submit button element
    /// (input, button).
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_formtarget(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            self.set_attribute(node, "formtarget", value);
        }
    }

    /// Sets the `formaction` content attribute on a valid submit button element
    /// (input, button).
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_formaction(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            self.set_attribute(node, "formaction", value);
        }
    }

    /// Sets the `formmethod` content attribute on a valid submit button element
    /// (input, button).
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_formmethod(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            self.set_attribute(node, "formmethod", value);
        }
    }

    /// Sets the `type` content attribute on a valid element node (button, input, embed, object, ol, script, source, style, link, menu, command).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_type(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined = name.eq_ignore_ascii_case("button")
                || name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("embed")
                || name.eq_ignore_ascii_case("object")
                || name.eq_ignore_ascii_case("ol")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("style")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("menu")
                || name.eq_ignore_ascii_case("command");
            if is_defined {
                self.set_attribute(node, "type", value);
            }
        }
    }

    /// Sets the `referrerpolicy` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_referrer_policy(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "referrerpolicy", value);
        }
    }

    /// Sets the `for` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_for(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "for", value);
        }
    }

    /// Sets the `colspan` content attribute on a valid table cell element (`<td>` or `<th>`).
    /// No-op if the node is not a table cell element, or if the `NodeId` is invalid.
    pub fn set_colspan(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            self.set_attribute(node, "colspan", value);
        }
    }

    /// Sets the `rowspan` content attribute on a valid table cell element (`<td>` or `<th>`).
    /// No-op if the node is not a table cell element, or if the `NodeId` is invalid.
    pub fn set_rowspan(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            self.set_attribute(node, "rowspan", value);
        }
    }

    /// Sets the `headers` content attribute on a valid table cell element (`<td>` or `<th>`).
    /// No-op if the node is not a table cell element, or if the `NodeId` is invalid.
    pub fn set_headers(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("td") || name.eq_ignore_ascii_case("th"))
        {
            self.set_attribute(node, "headers", value);
        }
    }

    /// Sets the `scope` content attribute on a valid `<th>` element.
    /// No-op if the node is not a `<th>` element, or if the `NodeId` is invalid.
    pub fn set_scope(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("th")
        {
            self.set_attribute(node, "scope", value);
        }
    }

    /// Sets the `abbr` content attribute on a valid `<th>` element.
    /// No-op if the node is not a `<th>` element, or if the `NodeId` is invalid.
    pub fn set_abbr(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("th")
        {
            self.set_attribute(node, "abbr", value);
        }
    }

    /// Sets the `cite` content attribute on a valid `blockquote`, `q`, `ins`, or `del` element.
    /// No-op if the node is not a matching element, or if the `NodeId` is invalid.
    pub fn set_cite(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("blockquote")
                || name.eq_ignore_ascii_case("q")
                || name.eq_ignore_ascii_case("ins")
                || name.eq_ignore_ascii_case("del"))
        {
            self.set_attribute(node, "cite", value);
        }
    }

    /// Sets the `datetime` content attribute on a valid `<time>`, `<ins>`, or `<del>` element.
    /// No-op if the node is not a matching element, or if the `NodeId` is invalid.
    pub fn set_datetime(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("time")
                || name.eq_ignore_ascii_case("ins")
                || name.eq_ignore_ascii_case("del"))
        {
            self.set_attribute(node, "datetime", value);
        }
    }

    /// Sets the `crossorigin` content attribute on a valid element node (img, script, link, audio, video).
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_cross_origin(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("audio")
                || name.eq_ignore_ascii_case("video"))
        {
            self.set_attribute(node, "crossorigin", value);
        }
    }

    /// Returns the value of the `span` content attribute of a valid `<col>` or `<colgroup>` element.
    /// Returns `None` if the node is not a `<col>` or `<colgroup>` element, has no `span` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_span(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("col") || name.eq_ignore_ascii_case("colgroup"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("span"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `span` content attribute on a valid `<col>` or `<colgroup>` element.
    /// No-op if the node is not a `<col>` or `<colgroup>` element, or if the `NodeId` is invalid.
    pub fn set_span(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("col") || name.eq_ignore_ascii_case("colgroup"))
        {
            self.set_attribute(node, "span", value);
        }
    }

    /// Returns the value of the `integrity` content attribute of a valid `<link>` or `<script>` element.
    /// Returns `None` if the node is not one of these elements, has no `integrity` attribute,
    /// or if the `NodeId` is invalid.
    pub fn get_integrity(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name, attrs } = &n.data
            && (name.eq_ignore_ascii_case("link") || name.eq_ignore_ascii_case("script"))
        {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("integrity"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `integrity` content attribute on a valid `<link>` or `<script>` element.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_integrity(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("link") || name.eq_ignore_ascii_case("script"))
        {
            self.set_attribute(node, "integrity", value);
        }
    }

    /// Sets or removes the `disabled` content attribute on a valid element node
    /// (input, button, select, textarea, option, optgroup, fieldset).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_disabled(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("button")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("option")
                || name.eq_ignore_ascii_case("optgroup")
                || name.eq_ignore_ascii_case("fieldset"))
        {
            if value {
                self.set_attribute(node, "disabled", "");
            } else {
                self.remove_attribute(node, "disabled");
            }
        }
    }

    /// Sets or removes the `required` content attribute on a valid element node (input, select, textarea).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_required(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("textarea"))
        {
            if value {
                self.set_attribute(node, "required", "");
            } else {
                self.remove_attribute(node, "required");
            }
        }
    }

    /// Sets or removes the `readonly` content attribute on a valid element node (input, textarea).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_readonly(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            if value {
                self.set_attribute(node, "readonly", "");
            } else {
                self.remove_attribute(node, "readonly");
            }
        }
    }

    /// Sets or removes the `autofocus` content attribute on any valid element node.
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_autofocus(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            if value {
                self.set_attribute(node, "autofocus", "");
            } else {
                self.remove_attribute(node, "autofocus");
            }
        }
    }

    /// Sets or removes the `multiple` content attribute on a valid element node (input, select).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of these elements, or if the `NodeId` is invalid.
    pub fn set_multiple(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("select"))
        {
            if value {
                self.set_attribute(node, "multiple", "");
            } else {
                self.remove_attribute(node, "multiple");
            }
        }
    }

    /// Sets or removes the `checked` content attribute on a valid `<input>` element.
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn set_checked(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            if value {
                self.set_attribute(node, "checked", "");
            } else {
                self.remove_attribute(node, "checked");
            }
        }
    }

    /// Sets or removes the `selected` content attribute on a valid `<option>` element.
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not an `<option>` element, or if the `NodeId` is invalid.
    pub fn set_selected(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("option")
        {
            if value {
                self.set_attribute(node, "selected", "");
            } else {
                self.remove_attribute(node, "selected");
            }
        }
    }

    /// Sets the `href` content attribute on a valid element node (a, area, link, base).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_href(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined = name.eq_ignore_ascii_case("a")
                || name.eq_ignore_ascii_case("area")
                || name.eq_ignore_ascii_case("link")
                || name.eq_ignore_ascii_case("base");
            if is_defined {
                self.set_attribute(node, "href", value);
            }
        }
    }

    /// Sets the `src` content attribute on a valid element node (img, script, iframe, source, audio, video, embed, track, input).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_src(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined = name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("script")
                || name.eq_ignore_ascii_case("iframe")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("audio")
                || name.eq_ignore_ascii_case("video")
                || name.eq_ignore_ascii_case("embed")
                || name.eq_ignore_ascii_case("track")
                || name.eq_ignore_ascii_case("input");
            if is_defined {
                self.set_attribute(node, "src", value);
            }
        }
    }

    /// Sets the `id` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_id(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "id", value);
        }
    }

    /// Sets the `class` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_class_name(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "class", value);
        }
    }

    /// Sets the `alt` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_alt(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "alt", value);
        }
    }

    /// Sets the `srcset` content attribute on a valid element node (img, source).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_srcset(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined =
                name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("source");
            if is_defined {
                self.set_attribute(node, "srcset", value);
            }
        }
    }

    /// Sets the `sizes` content attribute on a valid element node (img, source, link).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_sizes(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
        {
            let is_defined = name.eq_ignore_ascii_case("img")
                || name.eq_ignore_ascii_case("source")
                || name.eq_ignore_ascii_case("link");
            if is_defined {
                self.set_attribute(node, "sizes", value);
            }
        }
    }

    /// Sets the `decoding` content attribute on a valid `img` element.
    /// No-op if the node is not an `img` element, or if the `NodeId` is invalid.
    pub fn set_decoding(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("img")
        {
            self.set_attribute(node, "decoding", value);
        }
    }

    /// Sets the `placeholder` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_placeholder(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "placeholder", value);
        }
    }

    /// Sets the `name` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_name(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "name", value);
        }
    }

    /// Sets the `download` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_download(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "download", value);
        }
    }

    /// Sets the `action` content attribute on a valid `form` element.
    /// No-op if the node is not a `form` element, or if the `NodeId` is invalid.
    pub fn set_action(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            self.set_attribute(node, "action", value);
        }
    }

    /// Sets the `method` content attribute on a valid `form` element.
    /// No-op if the node is not a `form` element, or if the `NodeId` is invalid.
    pub fn set_method(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            self.set_attribute(node, "method", value);
        }
    }

    /// Sets the `enctype` content attribute on a valid `form` element.
    /// No-op if the node is not a `form` element, or if the `NodeId` is invalid.
    pub fn set_enctype(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("form")
        {
            self.set_attribute(node, "enctype", value);
        }
    }

    /// Sets the `width` content attribute on a valid `img` or `canvas` element.
    /// No-op if the node is not an `img` or `canvas` element, or if the `NodeId` is invalid.
    pub fn set_width(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("canvas"))
        {
            self.set_attribute(node, "width", value);
        }
    }

    /// Sets the `height` content attribute on a valid `img` or `canvas` element.
    /// No-op if the node is not an `img` or `canvas` element, or if the `NodeId` is invalid.
    pub fn set_height(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("img") || name.eq_ignore_ascii_case("canvas"))
        {
            self.set_attribute(node, "height", value);
        }
    }

    /// Sets the `max` content attribute on a valid `input` element.
    /// No-op if the node is not an `input` element, or if the `NodeId` is invalid.
    pub fn set_max(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            self.set_attribute(node, "max", value);
        }
    }

    /// Sets the `min` content attribute on a valid `input` element.
    /// No-op if the node is not an `input` element, or if the `NodeId` is invalid.
    pub fn set_min(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            self.set_attribute(node, "min", value);
        }
    }

    /// Sets the `step` content attribute on a valid `input` element.
    /// No-op if the node is not an `input` element, or if the `NodeId` is invalid.
    pub fn set_step(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            self.set_attribute(node, "step", value);
        }
    }

    /// Sets the `pattern` content attribute on a valid `input` element.
    /// No-op if the node is not an `input` element, or if the `NodeId` is invalid.
    pub fn set_pattern(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            self.set_attribute(node, "pattern", value);
        }
    }

    /// Sets the `accept` content attribute on a valid `input` element.
    /// No-op if the node is not an `input` element, or if the `NodeId` is invalid.
    pub fn set_accept(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            self.set_attribute(node, "accept", value);
        }
    }

    /// Sets the `start` content attribute on a valid `ol` element.
    /// No-op if the node is not an `ol` element, or if the `NodeId` is invalid.
    pub fn set_start(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("ol")
        {
            self.set_attribute(node, "start", value);
        }
    }

    /// Sets the `reversed` content attribute on a valid `ol` element.
    /// No-op if the node is not an `ol` element, or if the `NodeId` is invalid.
    pub fn set_reversed(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("ol")
        {
            self.set_attribute(node, "reversed", value);
        }
    }

    /// Sets the `value` content attribute on a valid `li` element.
    /// No-op if the node is not an `li` element, or if the `NodeId` is invalid.
    pub fn set_value(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("li")
        {
            self.set_attribute(node, "value", value);
        }
    }

    /// Sets the `content` content attribute on a valid `meta` element.
    /// No-op if the node is not a `meta` element, or if the `NodeId` is invalid.
    pub fn set_content(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            self.set_attribute(node, "content", value);
        }
    }

    /// Sets the `http-equiv` content attribute on a valid `meta` element.
    /// No-op if the node is not a `meta` element, or if the `NodeId` is invalid.
    pub fn set_http_equiv(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            self.set_attribute(node, "http-equiv", value);
        }
    }

    /// Sets the `charset` content attribute on a valid `meta` element.
    /// No-op if the node is not a `meta` element, or if the `NodeId` is invalid.
    pub fn set_charset(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("meta")
        {
            self.set_attribute(node, "charset", value);
        }
    }

    /// Sets the `wrap` content attribute on a valid `textarea` element.
    /// No-op if the node is not a `textarea` element, or if the `NodeId` is invalid.
    pub fn set_wrap(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("textarea")
        {
            self.set_attribute(node, "wrap", value);
        }
    }

    /// Sets the `value` content attribute on a valid `<textarea>` element.
    /// No-op if the node is not a `<textarea>` element, or if the `NodeId` is invalid.
    pub fn set_textarea_value(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("textarea")
        {
            self.set_attribute(node, "value", value);
        }
    }

    /// Sets the `value` content attribute on a valid `<button>` element.
    /// No-op if the node is not a `<button>` element, or if the `NodeId` is invalid.
    pub fn set_button_value(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("button")
        {
            self.set_attribute(node, "value", value);
        }
    }

    /// Sets the `value` content attribute on a valid `<option>` element.
    /// No-op if the node is not an `<option>` element, or if the `NodeId` is invalid.
    pub fn set_option_value(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("option")
        {
            self.set_attribute(node, "value", value);
        }
    }

    /// Sets the `kind` content attribute on a valid `track` element.
    /// No-op if the node is not a `track` element, or if the `NodeId` is invalid.
    pub fn set_kind(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            self.set_attribute(node, "kind", value);
        }
    }

    /// Sets the `srclang` content attribute on a valid `track` element.
    /// No-op if the node is not a `track` element, or if the `NodeId` is invalid.
    pub fn set_srclang(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            self.set_attribute(node, "srclang", value);
        }
    }

    /// Sets the `label` content attribute on a valid `track` element.
    /// No-op if the node is not a `track` element, or if the `NodeId` is invalid.
    pub fn set_label(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("track")
        {
            self.set_attribute(node, "label", value);
        }
    }

    /// Sets the `sandbox` content attribute on a valid `iframe` element.
    /// No-op if the node is not an `iframe` element, or if the `NodeId` is invalid.
    pub fn set_sandbox(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("iframe")
        {
            self.set_attribute(node, "sandbox", value);
        }
    }

    /// Sets the `allow` content attribute on a valid `iframe` element.
    /// No-op if the node is not an `iframe` element, or if the `NodeId` is invalid.
    pub fn set_allow(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("iframe")
        {
            self.set_attribute(node, "allow", value);
        }
    }

    /// Sets the `formnovalidate` content attribute on a valid submit button element (input, button).
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_formnovalidate(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("button"))
        {
            if value {
                self.set_attribute(node, "formnovalidate", "");
            } else {
                self.remove_attribute(node, "formnovalidate");
            }
        }
    }

    /// Sets the `inputmode` content attribute on a valid form control element (input, textarea, select, button).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_inputmode(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("button"))
        {
            self.set_attribute(node, "inputmode", value);
        }
    }

    /// Sets the `enterkeyhint` content attribute on a valid form control element (input, textarea, select, button).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_enterkeyhint(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input")
                || name.eq_ignore_ascii_case("textarea")
                || name.eq_ignore_ascii_case("select")
                || name.eq_ignore_ascii_case("button"))
        {
            self.set_attribute(node, "enterkeyhint", value);
        }
    }

    /// Sets the `dirname` content attribute on a valid form control element (input, textarea).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_dirname(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            self.set_attribute(node, "dirname", value);
        }
    }

    /// Sets the `maxlength` content attribute on a valid form control element (input, textarea).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_maxlength(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            self.set_attribute(node, "maxlength", value);
        }
    }

    /// Sets the `minlength` content attribute on a valid form control element (input, textarea).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_minlength(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("textarea"))
        {
            self.set_attribute(node, "minlength", value);
        }
    }

    /// Sets the `size` content attribute on a valid form control element (input, select).
    /// No-op if the node is not one of those element tags, or if the `NodeId` is invalid.
    pub fn set_size(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { name, .. } = &n.data
            && (name.eq_ignore_ascii_case("input") || name.eq_ignore_ascii_case("select"))
        {
            self.set_attribute(node, "size", value);
        }
    }

    /// Returns the value of the `slot` content attribute of a valid element node.
    /// Returns `None` if the node has no `slot` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_slot(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("slot"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `slot` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_slot(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "slot", value);
        }
    }

    /// Returns the value of the `nonce` content attribute of a valid element node.
    /// Returns `None` if the node has no `nonce` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_nonce(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("nonce"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `nonce` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_nonce(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "nonce", value);
        }
    }

    /// Returns the value of the `popover` content attribute of a valid element node.
    /// Returns `None` if the node has no `popover` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_popover(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("popover"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `popover` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_popover(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "popover", value);
        }
    }

    /// Returns the value of the `tabindex` content attribute of a valid element node.
    /// Returns `None` if the node has no `tabindex` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_tabindex(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("tabindex"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `tabindex` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_tabindex(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "tabindex", value);
        }
    }

    /// Returns whether the `hidden` content attribute is present on any valid element node.
    /// Returns `None` if the node is not an element node, or if the `NodeId` is invalid.
    pub fn get_hidden(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return Some(attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("hidden")));
        }
        None
    }

    /// Sets or removes the `hidden` content attribute on any valid element node.
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_hidden(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            if value {
                self.set_attribute(node, "hidden", "");
            } else {
                self.remove_attribute(node, "hidden");
            }
        }
    }

    /// Returns whether the `inert` content attribute is present on any valid element node.
    /// Returns `None` if the node is not an element node, or if the `NodeId` is invalid.
    pub fn get_inert(&self, node: NodeId) -> Option<bool> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return Some(attrs.iter().any(|(k, _)| k.eq_ignore_ascii_case("inert")));
        }
        None
    }

    /// Sets or removes the `inert` content attribute on any valid element node.
    /// If `value` is true, sets the attribute to `""`. If `value` is false, removes the attribute.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_inert(&mut self, node: NodeId, value: bool) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            if value {
                self.set_attribute(node, "inert", "");
            } else {
                self.remove_attribute(node, "inert");
            }
        }
    }

    /// Returns the value of the `autocapitalize` content attribute of a valid element node.
    /// Returns `None` if the node has no `autocapitalize` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_autocapitalize(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("autocapitalize"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `autocapitalize` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_autocapitalize(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "autocapitalize", value);
        }
    }

    /// Returns the value of the `accesskey` content attribute of a valid element node.
    /// Returns `None` if the node has no `accesskey` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_accesskey(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("accesskey"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `accesskey` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_accesskey(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "accesskey", value);
        }
    }

    /// Returns the value of the `spellcheck` content attribute of a valid element node.
    /// Returns `None` if the node has no `spellcheck` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_spellcheck(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("spellcheck"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `spellcheck` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_spellcheck(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "spellcheck", value);
        }
    }

    /// Returns the value of the `translate` content attribute of a valid element node.
    /// Returns `None` if the node has no `translate` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_translate(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("translate"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `translate` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_translate(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "translate", value);
        }
    }

    /// Returns the value of the `draggable` content attribute of a valid element node.
    /// Returns `None` if the node has no `draggable` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_draggable(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("draggable"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `draggable` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_draggable(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "draggable", value);
        }
    }

    /// Returns the value of the `contenteditable` content attribute of a valid element node.
    /// Returns `None` if the node has no `contenteditable` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_content_editable(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("contenteditable"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `contenteditable` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_content_editable(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "contenteditable", value);
        }
    }

    /// Returns the value of the `autocorrect` content attribute of a valid element node.
    /// Returns `None` if the node has no `autocorrect` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_autocorrect(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("autocorrect"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `autocorrect` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_autocorrect(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "autocorrect", value);
        }
    }

    /// Returns the value of the `writingsuggestions` content attribute of a valid element node.
    /// Returns `None` if the node has no `writingsuggestions` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_writing_suggestions(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("writingsuggestions"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `writingsuggestions` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_writing_suggestions(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "writingsuggestions", value);
        }
    }

    /// Returns the value of the `itemid` content attribute of a valid element node.
    /// Returns `None` if the node has no `itemid` attribute, is not an element node,
    /// or if the `NodeId` is invalid.
    pub fn get_item_id(&self, node: NodeId) -> Option<&str> {
        let n = self.arena.get(node)?;
        if let NodeData::Element { name: _, attrs } = &n.data {
            return attrs
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case("itemid"))
                .map(|(_, v)| v.as_str());
        }
        None
    }

    /// Sets the `itemid` content attribute on a valid element node.
    /// No-op if the node is not an element node, or if the `NodeId` is invalid.
    pub fn set_item_id(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get(node)
            && let NodeData::Element { .. } = &n.data
        {
            self.set_attribute(node, "itemid", value);
        }
    }

    // TODO(spec): muted reflects defaultMuted
}

#[cfg(test)]
mod tests {
    use crate::dom::{Dom, NodeData};
    use crate::infra::NodeId;

    fn elem(dom: &mut Dom, name: &str) -> NodeId {
        dom.create_node(NodeData::Element {
            name: name.to_string(),
            attrs: vec![],
        })
    }

    #[test]
    fn set_get_attribute_roundtrip_and_overwrite() {
        let mut dom = Dom::new();
        let a = elem(&mut dom, "a");
        assert_eq!(dom.get_attribute(a, "href"), None);
        dom.set_attribute(a, "href", "/x");
        assert_eq!(dom.get_attribute(a, "href"), Some("/x"));
        dom.set_attribute(a, "href", "/y");
        assert_eq!(dom.get_attribute(a, "href"), Some("/y"));
    }

    #[test]
    fn attribute_on_non_element_is_none() {
        let mut dom = Dom::new();
        let t = dom.create_node(NodeData::Text("hi".into()));
        dom.set_attribute(t, "x", "1"); // no-op, no panic
        assert_eq!(dom.get_attribute(t, "x"), None);
    }

    #[test]
    fn remove_child_detaches() {
        let mut dom = Dom::new();
        let p = elem(&mut dom, "p");
        let c = elem(&mut dom, "c");
        dom.append_child(p, c);
        assert_eq!(dom.children(p), &[c]);
        dom.remove_child(p, c);
        assert_eq!(dom.children(p), &[] as &[NodeId]);
        assert_eq!(dom.parent(c), None);
    }

    #[test]
    fn insert_before_places_and_appends() {
        let mut dom = Dom::new();
        let p = elem(&mut dom, "p");
        let a = elem(&mut dom, "a");
        let b = elem(&mut dom, "b");
        let c = elem(&mut dom, "c");
        dom.append_child(p, a);
        dom.append_child(p, c);
        // insert b before c -> [a, b, c]
        dom.insert_before(p, b, Some(c));
        assert_eq!(dom.children(p), &[a, b, c]);
        assert_eq!(dom.parent(b), Some(p));
        // None reference appends
        let d = elem(&mut dom, "d");
        dom.insert_before(p, d, None);
        assert_eq!(dom.children(p), &[a, b, c, d]);
    }

    #[test]
    fn insert_before_reparents() {
        let mut dom = Dom::new();
        let p1 = elem(&mut dom, "p1");
        let p2 = elem(&mut dom, "p2");
        let c = elem(&mut dom, "c");
        dom.append_child(p1, c);
        dom.insert_before(p2, c, None);
        assert_eq!(dom.children(p1), &[] as &[NodeId]);
        assert_eq!(dom.children(p2), &[c]);
        assert_eq!(dom.parent(c), Some(p2));
    }

    #[test]
    fn set_text_updates_text_node() {
        let mut dom = Dom::new();
        let t = dom.create_node(NodeData::Text("old".into()));
        dom.set_text(t, "new");
        assert_eq!(dom.data(t), Some(&NodeData::Text("new".into())));
    }

    #[test]
    fn invalid_ids_are_noops() {
        let mut dom1 = Dom::new();
        let mut dom2 = Dom::new();
        let foreign = elem(&mut dom1, "x");
        // Using dom1's id on dom2 must not panic and must do nothing.
        dom2.set_attribute(foreign, "a", "1");
        dom2.set_text(foreign, "t");
        dom2.remove_child(dom2.document(), foreign);
        dom2.insert_before(dom2.document(), foreign, None);
        assert_eq!(dom2.get_attribute(foreign, "a"), None);
    }

    #[test]
    fn test_input_value_initial_and_attribute() {
        let mut dom = Dom::new();
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("value".to_string(), "initial-val".to_string())],
        });

        // Initially not dirty, should return the attribute value
        assert_eq!(dom.is_input_value_dirty(input_id), Some(false));
        assert_eq!(
            dom.get_input_value(input_id),
            Some("initial-val".to_string())
        );

        // Update value attribute on non-dirty input
        dom.set_attribute(input_id, "value", "updated-val");
        assert_eq!(
            dom.get_input_value(input_id),
            Some("updated-val".to_string())
        );
    }

    #[test]
    fn test_input_value_set_dirty_exclusivity() {
        let mut dom = Dom::new();
        let input_id = dom.create_node(NodeData::Element {
            name: "INPUT".to_string(), // case-insensitive tag test
            attrs: vec![("value".to_string(), "initial".to_string())],
        });

        // Explicitly setting input value makes it dirty
        dom.set_input_value(input_id, "user-typed");
        assert_eq!(dom.is_input_value_dirty(input_id), Some(true));
        assert_eq!(
            dom.get_input_value(input_id),
            Some("user-typed".to_string())
        );

        // Setting "value" attribute when dirty should NOT change the current value
        dom.set_attribute(input_id, "value", "attr-change-while-dirty");
        assert_eq!(
            dom.get_input_value(input_id),
            Some("user-typed".to_string())
        );

        // But the attribute itself is updated successfully
        assert_eq!(
            dom.get_attribute(input_id, "value"),
            Some("attr-change-while-dirty")
        );

        // If we reset the dirty flag, it should fall back to the attribute value
        dom.set_input_value_dirty(input_id, false);
        assert_eq!(dom.is_input_value_dirty(input_id), Some(false));
        assert_eq!(
            dom.get_input_value(input_id),
            Some("attr-change-while-dirty".to_string())
        );
    }

    #[test]
    fn test_input_value_non_input_and_invalid_nodes() {
        let mut dom1 = Dom::new();
        let mut dom2 = Dom::new();
        let div_id = elem(&mut dom1, "div");
        let text_id = dom1.create_node(NodeData::Text("hello".to_string()));
        let foreign_id = elem(&mut dom2, "input");

        // Division is not an input
        assert_eq!(dom1.get_input_value(div_id), None);
        assert_eq!(dom1.is_input_value_dirty(div_id), None);
        dom1.set_input_value(div_id, "test"); // no-op
        assert_eq!(dom1.get_input_value(div_id), None);

        // Text node is not an input
        assert_eq!(dom1.get_input_value(text_id), None);
        assert_eq!(dom1.is_input_value_dirty(text_id), None);

        // Stale / foreign NodeId on dom1
        assert_eq!(dom1.get_input_value(foreign_id), None);
        assert_eq!(dom1.is_input_value_dirty(foreign_id), None);
    }

    #[test]
    fn test_href_and_src_accessors() {
        let mut dom = Dom::new();

        // 1. <a href="/foo"> => get_href returns Some("/foo"); get_src returns None.
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("href".to_string(), "/foo".to_string())],
        });
        assert_eq!(dom.get_href(a_id), Some("/foo"));
        assert_eq!(dom.get_src(a_id), None);

        // 2. <img src="a.png"> => get_src returns Some("a.png"); get_href returns None.
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("src".to_string(), "a.png".to_string())],
        });
        assert_eq!(dom.get_src(img_id), Some("a.png"));
        assert_eq!(dom.get_href(img_id), None);

        // 3. <link href="style.css"> and <base href="https://x/"> => get_href returns the value.
        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![("href".to_string(), "style.css".to_string())],
        });
        assert_eq!(dom.get_href(link_id), Some("style.css"));

        let base_id = dom.create_node(NodeData::Element {
            name: "base".to_string(),
            attrs: vec![("href".to_string(), "https://x/".to_string())],
        });
        assert_eq!(dom.get_href(base_id), Some("https://x/"));

        // 4. <div href="x"> => get_href returns None (href not defined on div).
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("href".to_string(), "x".to_string())],
        });
        assert_eq!(dom.get_href(div_id), None);

        // 5. An element of the right tag but missing the attribute => None.
        let a_no_href = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_href(a_no_href), None);

        // 6. A non-element node (e.g. a Text node) => both return None.
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_href(text_id), None);
        assert_eq!(dom.get_src(text_id), None);

        // 7. Case-insensitive tag name (e.g. A / IMG) is honored.
        let a_caps = dom.create_node(NodeData::Element {
            name: "A".to_string(),
            attrs: vec![("href".to_string(), "/caps".to_string())],
        });
        assert_eq!(dom.get_href(a_caps), Some("/caps"));

        let img_caps = dom.create_node(NodeData::Element {
            name: "IMG".to_string(),
            attrs: vec![("src".to_string(), "caps.png".to_string())],
        });
        assert_eq!(dom.get_src(img_caps), Some("caps.png"));

        // Additional: Case-insensitive attributes.
        let a_attr_caps = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("HREF".to_string(), "/attr-caps".to_string())],
        });
        assert_eq!(dom.get_href(a_attr_caps), Some("/attr-caps"));

        // Let's check other tags for src: script, iframe, source, audio, video, embed, track, input
        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("src".to_string(), "main.js".to_string())],
        });
        assert_eq!(dom.get_src(script_id), Some("main.js"));

        let iframe_id = dom.create_node(NodeData::Element {
            name: "iframe".to_string(),
            attrs: vec![("src".to_string(), "frame.html".to_string())],
        });
        assert_eq!(dom.get_src(iframe_id), Some("frame.html"));

        let source_id = dom.create_node(NodeData::Element {
            name: "source".to_string(),
            attrs: vec![("src".to_string(), "media.mp3".to_string())],
        });
        assert_eq!(dom.get_src(source_id), Some("media.mp3"));

        let audio_id = dom.create_node(NodeData::Element {
            name: "audio".to_string(),
            attrs: vec![("src".to_string(), "song.mp3".to_string())],
        });
        assert_eq!(dom.get_src(audio_id), Some("song.mp3"));

        let video_id = dom.create_node(NodeData::Element {
            name: "video".to_string(),
            attrs: vec![("src".to_string(), "movie.mp4".to_string())],
        });
        assert_eq!(dom.get_src(video_id), Some("movie.mp4"));

        let embed_id = dom.create_node(NodeData::Element {
            name: "embed".to_string(),
            attrs: vec![("src".to_string(), "plugin".to_string())],
        });
        assert_eq!(dom.get_src(embed_id), Some("plugin"));

        let track_id = dom.create_node(NodeData::Element {
            name: "track".to_string(),
            attrs: vec![("src".to_string(), "subs.vtt".to_string())],
        });
        assert_eq!(dom.get_src(track_id), Some("subs.vtt"));

        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("src".to_string(), "button.png".to_string())],
        });
        assert_eq!(dom.get_src(input_id), Some("button.png"));
    }

    #[test]
    fn test_id_accessor() {
        let mut dom = Dom::new();

        // 1. Element with id attribute -> returns Some(value)
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("id".to_string(), "my-element".to_string())],
        });
        assert_eq!(dom.get_id(div_id), Some("my-element"));

        // 2. Element with case-insensitive id attribute (e.g., ID) -> returns Some(value)
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("ID".to_string(), "another-element".to_string())],
        });
        assert_eq!(dom.get_id(span_id), Some("another-element"));

        // 3. Element without id attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_id(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_id(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        assert_eq!(dom.get_id(foreign_node), None);
    }

    #[test]
    fn test_class_name_accessor() {
        let mut dom = Dom::new();

        // 1. Element with class attribute -> returns Some(value)
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("class".to_string(), "foo bar".to_string())],
        });
        assert_eq!(dom.get_class_name(div_id), Some("foo bar"));

        // 2. Element with case-insensitive class attribute (e.g., CLASS) -> returns Some(value)
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("CLASS".to_string(), "another-class".to_string())],
        });
        assert_eq!(dom.get_class_name(span_id), Some("another-class"));

        // 3. Element without class attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_class_name(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_class_name(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        assert_eq!(dom.get_class_name(foreign_node), None);
    }

    #[test]
    fn test_alt_accessor() {
        let mut dom = Dom::new();

        // 1. Element with alt attribute -> returns Some(value)
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("alt".to_string(), "A cute puppy".to_string())],
        });
        assert_eq!(dom.get_alt(img_id), Some("A cute puppy"));

        // 2. Element with case-insensitive alt attribute (e.g., ALT) -> returns Some(value)
        let area_id = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("ALT".to_string(), "Clickable region".to_string())],
        });
        assert_eq!(dom.get_alt(area_id), Some("Clickable region"));

        // 3. Element without alt attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_alt(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_alt(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "img");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "img");
        }
        assert_eq!(dom.get_alt(foreign_node), None);
    }

    #[test]
    fn test_srcset_accessor() {
        let mut dom = Dom::new();

        // 1. Element of allowed tag with srcset -> returns Some
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("srcset".to_string(), "foo.png 1x, bar.png 2x".to_string())],
        });
        assert_eq!(dom.get_srcset(img_id), Some("foo.png 1x, bar.png 2x"));

        let source_id = dom.create_node(NodeData::Element {
            name: "source".to_string(),
            attrs: vec![("SRCSET".to_string(), "baz.png 1x".to_string())],
        });
        assert_eq!(dom.get_srcset(source_id), Some("baz.png 1x"));

        // 2. Element of not allowed tag with srcset -> returns None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("srcset".to_string(), "foo.png".to_string())],
        });
        assert_eq!(dom.get_srcset(div_id), None);

        // 3. Allowed tag without srcset -> returns None
        let img_no_attr_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_srcset(img_no_attr_id), None);

        // 4. Non-element node -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_srcset(text_id), None);
    }

    #[test]
    fn test_sizes_accessor() {
        let mut dom = Dom::new();

        // 1. Element of allowed tag with sizes -> returns Some
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("sizes".to_string(), "100vw".to_string())],
        });
        assert_eq!(dom.get_sizes(img_id), Some("100vw"));

        let source_id = dom.create_node(NodeData::Element {
            name: "source".to_string(),
            attrs: vec![("SIZES".to_string(), "50vw".to_string())],
        });
        assert_eq!(dom.get_sizes(source_id), Some("50vw"));

        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![("sizes".to_string(), "16x16".to_string())],
        });
        assert_eq!(dom.get_sizes(link_id), Some("16x16"));

        // 2. Element of not allowed tag with sizes -> returns None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("sizes".to_string(), "100vw".to_string())],
        });
        assert_eq!(dom.get_sizes(div_id), None);

        // 3. Allowed tag without sizes -> returns None
        let img_no_attr_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_sizes(img_no_attr_id), None);

        // 4. Non-element node -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_sizes(text_id), None);
    }

    #[test]
    fn test_decoding_accessor() {
        let mut dom = Dom::new();

        // 1. Element of allowed tag with decoding -> returns Some
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("decoding".to_string(), "async".to_string())],
        });
        assert_eq!(dom.get_decoding(img_id), Some("async"));

        let img_upper_id = dom.create_node(NodeData::Element {
            name: "IMG".to_string(),
            attrs: vec![("DECODING".to_string(), "sync".to_string())],
        });
        assert_eq!(dom.get_decoding(img_upper_id), Some("sync"));

        // 2. Element of not allowed tag with decoding -> returns None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("decoding".to_string(), "async".to_string())],
        });
        assert_eq!(dom.get_decoding(div_id), None);

        // 3. Allowed tag without decoding -> returns None
        let img_no_attr_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_decoding(img_no_attr_id), None);

        // 4. Non-element node -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_decoding(text_id), None);
    }

    #[test]
    fn test_cross_origin_accessor() {
        let mut dom = Dom::new();

        // 1. Element of allowed tag with crossorigin -> returns Some
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("crossorigin".to_string(), "anonymous".to_string())],
        });
        assert_eq!(dom.get_cross_origin(img_id), Some("anonymous"));

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("CROSSORIGIN".to_string(), "use-credentials".to_string())],
        });
        assert_eq!(dom.get_cross_origin(script_id), Some("use-credentials"));

        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![("crossorigin".to_string(), "anonymous".to_string())],
        });
        assert_eq!(dom.get_cross_origin(link_id), Some("anonymous"));

        let audio_id = dom.create_node(NodeData::Element {
            name: "audio".to_string(),
            attrs: vec![("crossorigin".to_string(), "anonymous".to_string())],
        });
        assert_eq!(dom.get_cross_origin(audio_id), Some("anonymous"));

        let video_id = dom.create_node(NodeData::Element {
            name: "video".to_string(),
            attrs: vec![("crossorigin".to_string(), "anonymous".to_string())],
        });
        assert_eq!(dom.get_cross_origin(video_id), Some("anonymous"));

        // 2. Element of not allowed tag with crossorigin -> returns None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("crossorigin".to_string(), "anonymous".to_string())],
        });
        assert_eq!(dom.get_cross_origin(div_id), None);

        // 3. Allowed tag without crossorigin -> returns None
        let img_no_attr_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_cross_origin(img_no_attr_id), None);

        // 4. Non-element node -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_cross_origin(text_id), None);
    }

    #[test]
    fn test_placeholder_accessor() {
        let mut dom = Dom::new();

        // 1. Element with placeholder attribute -> returns Some(value)
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("placeholder".to_string(), "Search".to_string())],
        });
        assert_eq!(dom.get_placeholder(input_id), Some("Search"));

        // 2. Element with case-insensitive placeholder attribute (e.g., PLACEHOLDER) -> returns Some(value)
        let input_id_upper = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("PLACEHOLDER".to_string(), "Search Upper".to_string())],
        });
        assert_eq!(dom.get_placeholder(input_id_upper), Some("Search Upper"));

        // 3. Element without placeholder attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_placeholder(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_placeholder(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "input");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "input");
        }
        assert_eq!(dom.get_placeholder(foreign_node), None);
    }

    #[test]
    fn test_name_accessor() {
        let mut dom = Dom::new();

        // 1. Element with name attribute -> returns Some(value)
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("name".to_string(), "username".to_string())],
        });
        assert_eq!(dom.get_name(input_id), Some("username"));

        // 2. Element with case-insensitive name attribute (e.g., NAME) -> returns Some(value)
        let button_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("NAME".to_string(), "submit-btn".to_string())],
        });
        assert_eq!(dom.get_name(button_id), Some("submit-btn"));

        // 3. Element without name attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_name(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_name(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "input");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "input");
        }
        assert_eq!(dom.get_name(foreign_node), None);
    }

    #[test]
    fn test_title_accessor() {
        let mut dom = Dom::new();

        // 1. Element with title attribute -> returns Some(value)
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("title".to_string(), "my-title".to_string())],
        });
        assert_eq!(dom.get_title(div_id), Some("my-title"));

        // 2. Element with case-insensitive title attribute (e.g., TITLE) -> returns Some(value)
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("TITLE".to_string(), "other-title".to_string())],
        });
        assert_eq!(dom.get_title(span_id), Some("other-title"));

        // 3. Element without title attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_title(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_title(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        assert_eq!(dom.get_title(foreign_node), None);
    }

    #[test]
    fn test_lang_accessor() {
        let mut dom = Dom::new();

        // 1. Element with lang attribute -> returns Some(value)
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("lang".to_string(), "en".to_string())],
        });
        assert_eq!(dom.get_lang(div_id), Some("en"));

        // 2. Element with case-insensitive lang attribute (e.g., LANG) -> returns Some(value)
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("LANG".to_string(), "fr".to_string())],
        });
        assert_eq!(dom.get_lang(span_id), Some("fr"));

        // 3. Element without lang attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_lang(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_lang(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        assert_eq!(dom.get_lang(foreign_node), None);
    }

    #[test]
    fn test_rel_and_for_accessors() {
        let mut dom = Dom::new();

        // 1. Element with rel / for attribute -> returns Some(value)
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("rel".to_string(), "stylesheet".to_string())],
        });
        assert_eq!(dom.get_rel(a_id), Some("stylesheet"));

        let label_id = dom.create_node(NodeData::Element {
            name: "label".to_string(),
            attrs: vec![("for".to_string(), "username".to_string())],
        });
        assert_eq!(dom.get_for(label_id), Some("username"));

        // 2. Element with case-insensitive rel / for attribute (e.g., REL / FOR) -> returns Some(value)
        let a_id_upper = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("REL".to_string(), "stylesheet-upper".to_string())],
        });
        assert_eq!(dom.get_rel(a_id_upper), Some("stylesheet-upper"));

        let label_id_upper = dom.create_node(NodeData::Element {
            name: "label".to_string(),
            attrs: vec![("FOR".to_string(), "username-upper".to_string())],
        });
        assert_eq!(dom.get_for(label_id_upper), Some("username-upper"));

        // 3. Element without rel / for attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_rel(p_id), None);
        assert_eq!(dom.get_for(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_rel(text_id), None);
        assert_eq!(dom.get_for(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "input");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "input");
        }
        assert_eq!(dom.get_rel(foreign_node), None);
        assert_eq!(dom.get_for(foreign_node), None);
    }

    #[test]
    fn test_anchor_attributes_accessors() {
        let mut dom = Dom::new();

        // 1. Element with the attributes set -> returns Some(value)
        let node_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![
                ("hreflang".to_string(), "en-US".to_string()),
                ("download".to_string(), "file.pdf".to_string()),
                ("referrerpolicy".to_string(), "no-referrer".to_string()),
            ],
        });
        assert_eq!(dom.get_hreflang(node_id), Some("en-US"));
        assert_eq!(dom.get_download(node_id), Some("file.pdf"));
        assert_eq!(dom.get_referrer_policy(node_id), Some("no-referrer"));

        // 2. Element with case-insensitive attributes (e.g. HREFLANG / DOWNLOAD / REFERRERPOLICY) -> returns Some(value)
        let upper_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![
                ("HREFLANG".to_string(), "fr-FR".to_string()),
                ("DOWNLOAD".to_string(), "doc.txt".to_string()),
                ("REFERRERPOLICY".to_string(), "origin".to_string()),
            ],
        });
        assert_eq!(dom.get_hreflang(upper_id), Some("fr-FR"));
        assert_eq!(dom.get_download(upper_id), Some("doc.txt"));
        assert_eq!(dom.get_referrer_policy(upper_id), Some("origin"));

        // 3. Element without attributes -> returns None
        let empty_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_hreflang(empty_id), None);
        assert_eq!(dom.get_download(empty_id), None);
        assert_eq!(dom.get_referrer_policy(empty_id), None);

        // 4. Non-element node (e.g. Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_hreflang(text_id), None);
        assert_eq!(dom.get_download(text_id), None);
        assert_eq!(dom.get_referrer_policy(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "a");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "a");
        }
        assert_eq!(dom.get_hreflang(foreign_node), None);
        assert_eq!(dom.get_download(foreign_node), None);
        assert_eq!(dom.get_referrer_policy(foreign_node), None);
    }

    #[test]
    fn test_dir_accessor() {
        let mut dom = Dom::new();

        // 1. Element with dir attribute -> returns Some(value)
        let div_rtl = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("dir".to_string(), "rtl".to_string())],
        });
        assert_eq!(dom.get_dir(div_rtl), Some("rtl"));

        let div_ltr = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("dir".to_string(), "ltr".to_string())],
        });
        assert_eq!(dom.get_dir(div_ltr), Some("ltr"));

        // 2. Element with case-insensitive dir attribute (e.g., DIR) -> returns Some(value)
        let span_id = dom.create_node(NodeData::Element {
            name: "span".to_string(),
            attrs: vec![("DIR".to_string(), "rtl".to_string())],
        });
        assert_eq!(dom.get_dir(span_id), Some("rtl"));

        // 3. Element without dir attribute -> returns None
        let p_id = dom.create_node(NodeData::Element {
            name: "p".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_dir(p_id), None);

        // 4. Non-element node (e.g., Text node) -> returns None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_dir(text_id), None);

        // 5. Invalid / foreign NodeId -> returns None
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        assert_eq!(dom.get_dir(foreign_node), None);
    }

    #[test]
    fn test_set_attribute_marks_dirty_only_on_success() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");
        assert!(!dom.is_dirty(el));
        assert!(!dom.has_dirty());

        // 1. set_attribute on a valid element marks it dirty (is_dirty true).
        dom.set_attribute(el, "class", "active");
        assert!(dom.is_dirty(el));
        assert!(dom.has_dirty());

        dom.clear_dirty();
        assert!(!dom.has_dirty());

        // 2. set_attribute on an INVALID node id does NOT mark anything dirty (has_dirty() false).
        let mut foreign_dom = Dom::new();
        let mut foreign_node = elem(&mut foreign_dom, "div");
        for _ in 0..100 {
            foreign_node = elem(&mut foreign_dom, "div");
        }
        dom.set_attribute(foreign_node, "class", "inactive");
        assert!(!dom.has_dirty());
    }

    #[test]
    fn test_remove_attribute_marks_dirty_only_on_actual_removal() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");
        dom.set_attribute(el, "class", "active");
        dom.clear_dirty();

        // Removing a non-existent attribute does NOT mark dirty.
        dom.remove_attribute(el, "id");
        assert!(!dom.has_dirty());

        // Removing an existing attribute marks the element dirty.
        dom.remove_attribute(el, "class");
        assert!(dom.is_dirty(el));
        assert!(dom.has_dirty());
    }

    #[test]
    fn test_remove_child_marks_parent_dirty() {
        let mut dom = Dom::new();
        let p = elem(&mut dom, "div");
        let c = elem(&mut dom, "span");
        dom.append_child(p, c);
        dom.clear_dirty();

        // removing a non-child is a no-op (no dirty).
        let non_child = elem(&mut dom, "p");
        dom.clear_dirty();
        dom.remove_child(p, non_child);
        assert!(!dom.has_dirty());

        // removing an actual child marks the PARENT dirty.
        dom.remove_child(p, c);
        assert!(dom.is_dirty(p));
        assert!(!dom.is_dirty(c));
    }

    #[test]
    fn test_insert_before_marks_parent_dirty() {
        let mut dom = Dom::new();
        let p = elem(&mut dom, "div");
        let c1 = elem(&mut dom, "span");
        let c2 = elem(&mut dom, "p");
        dom.clear_dirty();

        // insert_before of a child marks the PARENT dirty.
        dom.insert_before(p, c1, None);
        assert!(dom.is_dirty(p));
        assert!(!dom.is_dirty(c1));

        dom.clear_dirty();
        dom.insert_before(p, c2, Some(c1));
        assert!(dom.is_dirty(p));
        assert!(!dom.is_dirty(c2));
    }

    #[test]
    fn test_set_text_marks_dirty_only_on_success() {
        let mut dom = Dom::new();
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        dom.clear_dirty();

        // set_text on text node marks it dirty.
        dom.set_text(text_id, "world");
        assert!(dom.is_dirty(text_id));

        dom.clear_dirty();

        // set_text on element node is a no-op (no dirty).
        let el = elem(&mut dom, "div");
        dom.clear_dirty();
        dom.set_text(el, "world");
        assert!(!dom.has_dirty());
    }

    #[test]
    fn test_set_input_value_marks_dirty_only_on_success() {
        let mut dom = Dom::new();
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        dom.clear_dirty();

        // set_input_value on input element marks it dirty.
        dom.set_input_value(input_id, "new-val");
        assert!(dom.is_dirty(input_id));

        dom.clear_dirty();

        // set_input_value on non-input element is a no-op (no dirty).
        let div_id = elem(&mut dom, "div");
        dom.clear_dirty();
        dom.set_input_value(div_id, "new-val");
        assert!(!dom.has_dirty());
    }

    #[test]
    fn test_target_accessor() {
        let mut dom = Dom::new();

        // 1. <a target="_blank"> => get_target returns Some("_blank").
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("target".to_string(), "_blank".to_string())],
        });
        assert_eq!(dom.get_target(a_id), Some("_blank"));

        // 2. <form target="_parent"> => get_target returns Some("_parent").
        let form_id = dom.create_node(NodeData::Element {
            name: "form".to_string(),
            attrs: vec![("target".to_string(), "_parent".to_string())],
        });
        assert_eq!(dom.get_target(form_id), Some("_parent"));

        // 3. <base target="_top"> => get_target returns Some("_top").
        let base_id = dom.create_node(NodeData::Element {
            name: "base".to_string(),
            attrs: vec![("target".to_string(), "_top".to_string())],
        });
        assert_eq!(dom.get_target(base_id), Some("_top"));

        // 4. <area target="_self"> => get_target returns Some("_self").
        let area_id = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("target".to_string(), "_self".to_string())],
        });
        assert_eq!(dom.get_target(area_id), Some("_self"));

        // 5. <div target="_blank"> => get_target returns None (target not defined on div).
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("target".to_string(), "_blank".to_string())],
        });
        assert_eq!(dom.get_target(div_id), None);

        // 6. Element of right tag but missing the target attribute => None.
        let a_no_target = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_target(a_no_target), None);

        // 7. Non-element node (e.g. Text node) => None.
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_target(text_id), None);

        // 8. Case-insensitive tag name is honored.
        let a_caps = dom.create_node(NodeData::Element {
            name: "A".to_string(),
            attrs: vec![("target".to_string(), "_blank".to_string())],
        });
        assert_eq!(dom.get_target(a_caps), Some("_blank"));

        // 9. Case-insensitive attribute name is honored.
        let a_attr_caps = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("TARGET".to_string(), "_blank".to_string())],
        });
        assert_eq!(dom.get_target(a_attr_caps), Some("_blank"));
    }

    #[test]
    fn test_type_accessor() {
        let mut dom = Dom::new();

        // 1. Valid tags with 'type' attribute should return Some
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("type".to_string(), "text".to_string())],
        });
        assert_eq!(dom.get_type(input_id), Some("text"));

        let button_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("type".to_string(), "submit".to_string())],
        });
        assert_eq!(dom.get_type(button_id), Some("submit"));

        let ol_id = dom.create_node(NodeData::Element {
            name: "ol".to_string(),
            attrs: vec![("type".to_string(), "1".to_string())],
        });
        assert_eq!(dom.get_type(ol_id), Some("1"));

        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![("type".to_string(), "text/css".to_string())],
        });
        assert_eq!(dom.get_type(link_id), Some("text/css"));

        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![("type".to_string(), "module".to_string())],
        });
        assert_eq!(dom.get_type(script_id), Some("module"));

        let embed_id = dom.create_node(NodeData::Element {
            name: "embed".to_string(),
            attrs: vec![("type".to_string(), "video/mp4".to_string())],
        });
        assert_eq!(dom.get_type(embed_id), Some("video/mp4"));

        let object_id = dom.create_node(NodeData::Element {
            name: "object".to_string(),
            attrs: vec![("type".to_string(), "application/pdf".to_string())],
        });
        assert_eq!(dom.get_type(object_id), Some("application/pdf"));

        let source_id = dom.create_node(NodeData::Element {
            name: "source".to_string(),
            attrs: vec![("type".to_string(), "audio/ogg".to_string())],
        });
        assert_eq!(dom.get_type(source_id), Some("audio/ogg"));

        let style_id = dom.create_node(NodeData::Element {
            name: "style".to_string(),
            attrs: vec![("type".to_string(), "text/css".to_string())],
        });
        assert_eq!(dom.get_type(style_id), Some("text/css"));

        let menu_id = dom.create_node(NodeData::Element {
            name: "menu".to_string(),
            attrs: vec![("type".to_string(), "context".to_string())],
        });
        assert_eq!(dom.get_type(menu_id), Some("context"));

        let command_id = dom.create_node(NodeData::Element {
            name: "command".to_string(),
            attrs: vec![("type".to_string(), "checkbox".to_string())],
        });
        assert_eq!(dom.get_type(command_id), Some("checkbox"));

        // 2. Undefined tag with 'type' attribute should return None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("type".to_string(), "text".to_string())],
        });
        assert_eq!(dom.get_type(div_id), None);

        // 3. Defined tag missing 'type' attribute should return None
        let input_no_type_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_type(input_no_type_id), None);

        // 4. Non-element node should return None
        let text_id = dom.create_node(NodeData::Text("hello".to_string()));
        assert_eq!(dom.get_type(text_id), None);

        // 5. Case-insensitivity should be honored for tag name
        let input_caps_id = dom.create_node(NodeData::Element {
            name: "InPuT".to_string(),
            attrs: vec![("type".to_string(), "password".to_string())],
        });
        assert_eq!(dom.get_type(input_caps_id), Some("password"));

        // 6. Case-insensitivity should be honored for attribute name
        let input_attr_caps_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("TyPe".to_string(), "email".to_string())],
        });
        assert_eq!(dom.get_type(input_attr_caps_id), Some("email"));

        // 7. Invalid NodeId should return None
        let dom2 = Dom::new();
        assert_eq!(dom2.get_type(input_id), None);
    }

    #[test]
    fn test_new_reflected_getters() {
        let mut dom = Dom::new();

        // --- get_action, get_method, get_enctype (form) ---
        let form_id = dom.create_node(NodeData::Element {
            name: "form".to_string(),
            attrs: vec![
                ("action".to_string(), "/submit".to_string()),
                ("method".to_string(), "post".to_string()),
                ("enctype".to_string(), "multipart/form-data".to_string()),
            ],
        });
        assert_eq!(dom.get_action(form_id), Some("/submit"));
        assert_eq!(dom.get_method(form_id), Some("post"));
        assert_eq!(dom.get_enctype(form_id), Some("multipart/form-data"));

        // Case-insensitivity tests for form elements
        let form_caps_id = dom.create_node(NodeData::Element {
            name: "FORM".to_string(),
            attrs: vec![
                ("ACTION".to_string(), "/SUBMIT".to_string()),
                ("METHOD".to_string(), "POST".to_string()),
                ("ENCTYPE".to_string(), "MULTIPART".to_string()),
            ],
        });
        assert_eq!(dom.get_action(form_caps_id), Some("/SUBMIT"));
        assert_eq!(dom.get_method(form_caps_id), Some("POST"));
        assert_eq!(dom.get_enctype(form_caps_id), Some("MULTIPART"));

        // When absent, they should return Some("")
        let form_absent_id = dom.create_node(NodeData::Element {
            name: "form".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_action(form_absent_id), Some(""));
        assert_eq!(dom.get_method(form_absent_id), Some(""));
        assert_eq!(dom.get_enctype(form_absent_id), Some(""));

        // When node is invalid/non-form, they should return None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("action".to_string(), "x".to_string()),
                ("method".to_string(), "x".to_string()),
                ("enctype".to_string(), "x".to_string()),
            ],
        });
        assert_eq!(dom.get_action(div_id), None);
        assert_eq!(dom.get_method(div_id), None);
        assert_eq!(dom.get_enctype(div_id), None);

        // --- get_width, get_height (img, canvas) ---
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![
                ("width".to_string(), "400".to_string()),
                ("height".to_string(), "300".to_string()),
            ],
        });
        assert_eq!(dom.get_width(img_id), Some("400"));
        assert_eq!(dom.get_height(img_id), Some("300"));

        let canvas_id = dom.create_node(NodeData::Element {
            name: "canvas".to_string(),
            attrs: vec![
                ("width".to_string(), "800".to_string()),
                ("height".to_string(), "600".to_string()),
            ],
        });
        assert_eq!(dom.get_width(canvas_id), Some("800"));
        assert_eq!(dom.get_height(canvas_id), Some("600"));

        // When absent, they should return Some("")
        let img_absent_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_width(img_absent_id), Some(""));
        assert_eq!(dom.get_height(img_absent_id), Some(""));

        // When invalid tag, they should return None
        assert_eq!(dom.get_width(div_id), None);
        assert_eq!(dom.get_height(div_id), None);

        // --- get_max, get_min, get_step, get_pattern, get_accept (input) ---
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("max".to_string(), "10".to_string()),
                ("min".to_string(), "1".to_string()),
                ("step".to_string(), "2".to_string()),
                ("pattern".to_string(), "[0-9]+".to_string()),
                ("accept".to_string(), "image/*".to_string()),
            ],
        });
        assert_eq!(dom.get_max(input_id), Some("10"));
        assert_eq!(dom.get_min(input_id), Some("1"));
        assert_eq!(dom.get_step(input_id), Some("2"));
        assert_eq!(dom.get_pattern(input_id), Some("[0-9]+"));
        assert_eq!(dom.get_accept(input_id), Some("image/*"));

        // Case-insensitivity for input attributes
        let input_caps_id = dom.create_node(NodeData::Element {
            name: "INPUT".to_string(),
            attrs: vec![
                ("MAX".to_string(), "20".to_string()),
                ("MIN".to_string(), "5".to_string()),
                ("STEP".to_string(), "3".to_string()),
                ("PATTERN".to_string(), "a".to_string()),
                ("ACCEPT".to_string(), "b".to_string()),
            ],
        });
        assert_eq!(dom.get_max(input_caps_id), Some("20"));
        assert_eq!(dom.get_min(input_caps_id), Some("5"));
        assert_eq!(dom.get_step(input_caps_id), Some("3"));
        assert_eq!(dom.get_pattern(input_caps_id), Some("a"));
        assert_eq!(dom.get_accept(input_caps_id), Some("b"));

        // When absent, they should return Some("")
        let input_absent_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_max(input_absent_id), Some(""));
        assert_eq!(dom.get_min(input_absent_id), Some(""));
        assert_eq!(dom.get_step(input_absent_id), Some(""));
        assert_eq!(dom.get_pattern(input_absent_id), Some(""));
        assert_eq!(dom.get_accept(input_absent_id), Some(""));

        // When invalid tag, they should return None
        assert_eq!(dom.get_max(div_id), None);
        assert_eq!(dom.get_min(div_id), None);
        assert_eq!(dom.get_step(div_id), None);
        assert_eq!(dom.get_pattern(div_id), None);
        assert_eq!(dom.get_accept(div_id), None);

        // --- Invalid NodeId should return None for all ---
        let foreign_dom = Dom::new();
        let foreign_node = dom.create_node(NodeData::Text("hi".to_string()));
        assert_eq!(foreign_dom.get_action(foreign_node), None);
        assert_eq!(foreign_dom.get_method(foreign_node), None);
        assert_eq!(foreign_dom.get_enctype(foreign_node), None);
        assert_eq!(foreign_dom.get_width(foreign_node), None);
        assert_eq!(foreign_dom.get_height(foreign_node), None);
        assert_eq!(foreign_dom.get_max(foreign_node), None);
        assert_eq!(foreign_dom.get_min(foreign_node), None);
        assert_eq!(foreign_dom.get_step(foreign_node), None);
        assert_eq!(foreign_dom.get_pattern(foreign_node), None);
        assert_eq!(foreign_dom.get_accept(foreign_node), None);
    }

    #[test]
    fn test_table_cell_list_and_meta_accessors() {
        let mut dom = Dom::new();

        // --- Table Cell: colspan, rowspan, headers (td, th) ---
        let td_id = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![
                ("colspan".to_string(), "2".to_string()),
                ("rowspan".to_string(), "3".to_string()),
                ("headers".to_string(), "header-id".to_string()),
            ],
        });
        assert_eq!(dom.get_colspan(td_id), Some("2"));
        assert_eq!(dom.get_rowspan(td_id), Some("3"));
        assert_eq!(dom.get_headers(td_id), Some("header-id"));

        let th_id = dom.create_node(NodeData::Element {
            name: "th".to_string(),
            attrs: vec![
                ("colspan".to_string(), "4".to_string()),
                ("rowspan".to_string(), "5".to_string()),
                ("headers".to_string(), "h1 h2".to_string()),
                ("scope".to_string(), "row".to_string()),
                ("abbr".to_string(), "Abbreviation".to_string()),
            ],
        });
        assert_eq!(dom.get_colspan(th_id), Some("4"));
        assert_eq!(dom.get_rowspan(th_id), Some("5"));
        assert_eq!(dom.get_headers(th_id), Some("h1 h2"));
        assert_eq!(dom.get_scope(th_id), Some("row"));
        assert_eq!(dom.get_abbr(th_id), Some("Abbreviation"));

        // Case insensitivity for td/th
        let td_caps_id = dom.create_node(NodeData::Element {
            name: "TD".to_string(),
            attrs: vec![
                ("COLSPAN".to_string(), "10".to_string()),
                ("ROWSPAN".to_string(), "20".to_string()),
                ("HEADERS".to_string(), "cap-header".to_string()),
            ],
        });
        assert_eq!(dom.get_colspan(td_caps_id), Some("10"));
        assert_eq!(dom.get_rowspan(td_caps_id), Some("20"));
        assert_eq!(dom.get_headers(td_caps_id), Some("cap-header"));

        let th_caps_id = dom.create_node(NodeData::Element {
            name: "TH".to_string(),
            attrs: vec![
                ("SCOPE".to_string(), "col".to_string()),
                ("ABBR".to_string(), "Short".to_string()),
            ],
        });
        assert_eq!(dom.get_scope(th_caps_id), Some("col"));
        assert_eq!(dom.get_abbr(th_caps_id), Some("Short"));

        // Negative check: wrong tags return None
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("colspan".to_string(), "2".to_string()),
                ("rowspan".to_string(), "3".to_string()),
                ("headers".to_string(), "header-id".to_string()),
                ("scope".to_string(), "col".to_string()),
                ("abbr".to_string(), "Abbreviation".to_string()),
            ],
        });
        assert_eq!(dom.get_colspan(div_id), None);
        assert_eq!(dom.get_rowspan(div_id), None);
        assert_eq!(dom.get_headers(div_id), None);
        assert_eq!(dom.get_scope(div_id), None);
        assert_eq!(dom.get_abbr(div_id), None);

        // td has no scope/abbr defined by spec
        assert_eq!(dom.get_scope(td_id), None);
        assert_eq!(dom.get_abbr(td_id), None);

        // Absent attributes on th return None
        let th_empty_id = dom.create_node(NodeData::Element {
            name: "th".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_colspan(th_empty_id), None);
        assert_eq!(dom.get_rowspan(th_empty_id), None);
        assert_eq!(dom.get_headers(th_empty_id), None);
        assert_eq!(dom.get_scope(th_empty_id), None);
        assert_eq!(dom.get_abbr(th_empty_id), None);

        // --- Ordered List: start, reversed (ol); List Item: value (li) ---
        let ol_id = dom.create_node(NodeData::Element {
            name: "ol".to_string(),
            attrs: vec![
                ("start".to_string(), "5".to_string()),
                ("reversed".to_string(), "reversed".to_string()),
            ],
        });
        assert_eq!(dom.get_start(ol_id), Some("5"));
        assert_eq!(dom.get_reversed(ol_id), Some("reversed"));

        let li_id = dom.create_node(NodeData::Element {
            name: "li".to_string(),
            attrs: vec![("value".to_string(), "10".to_string())],
        });
        assert_eq!(dom.get_value(li_id), Some("10"));

        // Case insensitivity
        let ol_caps_id = dom.create_node(NodeData::Element {
            name: "OL".to_string(),
            attrs: vec![
                ("START".to_string(), "1".to_string()),
                ("REVERSED".to_string(), "".to_string()),
            ],
        });
        assert_eq!(dom.get_start(ol_caps_id), Some("1"));
        assert_eq!(dom.get_reversed(ol_caps_id), Some(""));

        let li_caps_id = dom.create_node(NodeData::Element {
            name: "LI".to_string(),
            attrs: vec![("VALUE".to_string(), "42".to_string())],
        });
        assert_eq!(dom.get_value(li_caps_id), Some("42"));

        // Negative check: wrong tags return None
        assert_eq!(dom.get_start(div_id), None);
        assert_eq!(dom.get_reversed(div_id), None);
        assert_eq!(dom.get_value(div_id), None);

        // ol has no value, li has no start/reversed
        assert_eq!(dom.get_value(ol_id), None);
        assert_eq!(dom.get_start(li_id), None);
        assert_eq!(dom.get_reversed(li_id), None);

        // Absent attributes
        let ol_empty_id = dom.create_node(NodeData::Element {
            name: "ol".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_start(ol_empty_id), None);
        assert_eq!(dom.get_reversed(ol_empty_id), None);

        let li_empty_id = dom.create_node(NodeData::Element {
            name: "li".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_value(li_empty_id), None);

        // --- Meta: name, content, http-equiv, charset (meta) ---
        let meta_id = dom.create_node(NodeData::Element {
            name: "meta".to_string(),
            attrs: vec![
                ("name".to_string(), "description".to_string()),
                ("content".to_string(), "hello world".to_string()),
                ("http-equiv".to_string(), "content-type".to_string()),
                ("charset".to_string(), "utf-8".to_string()),
            ],
        });
        assert_eq!(dom.get_name(meta_id), Some("description"));
        assert_eq!(dom.get_content(meta_id), Some("hello world"));
        assert_eq!(dom.get_http_equiv(meta_id), Some("content-type"));
        assert_eq!(dom.get_charset(meta_id), Some("utf-8"));

        // Case insensitivity
        let meta_caps_id = dom.create_node(NodeData::Element {
            name: "META".to_string(),
            attrs: vec![
                ("NAME".to_string(), "viewport".to_string()),
                ("CONTENT".to_string(), "width=device-width".to_string()),
                ("HTTP-EQUIV".to_string(), "refresh".to_string()),
                ("CHARSET".to_string(), "iso-8859-1".to_string()),
            ],
        });
        assert_eq!(dom.get_name(meta_caps_id), Some("viewport"));
        assert_eq!(dom.get_content(meta_caps_id), Some("width=device-width"));
        assert_eq!(dom.get_http_equiv(meta_caps_id), Some("refresh"));
        assert_eq!(dom.get_charset(meta_caps_id), Some("iso-8859-1"));

        // Negative check: wrong tag returns None
        assert_eq!(dom.get_content(div_id), None);
        assert_eq!(dom.get_http_equiv(div_id), None);
        assert_eq!(dom.get_charset(div_id), None);

        // Absent attributes
        let meta_empty_id = dom.create_node(NodeData::Element {
            name: "meta".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_name(meta_empty_id), None);
        assert_eq!(dom.get_content(meta_empty_id), None);
        assert_eq!(dom.get_http_equiv(meta_empty_id), None);
        assert_eq!(dom.get_charset(meta_empty_id), None);

        // --- Invalid NodeId returns None for all new getters ---
        let foreign_dom = Dom::new();
        let foreign_node = dom.create_node(NodeData::Text("hi".to_string()));
        assert_eq!(foreign_dom.get_colspan(foreign_node), None);
        assert_eq!(foreign_dom.get_rowspan(foreign_node), None);
        assert_eq!(foreign_dom.get_headers(foreign_node), None);
        assert_eq!(foreign_dom.get_scope(foreign_node), None);
        assert_eq!(foreign_dom.get_abbr(foreign_node), None);
        assert_eq!(foreign_dom.get_start(foreign_node), None);
        assert_eq!(foreign_dom.get_reversed(foreign_node), None);
        assert_eq!(foreign_dom.get_value(foreign_node), None);
        assert_eq!(foreign_dom.get_content(foreign_node), None);
        assert_eq!(foreign_dom.get_http_equiv(foreign_node), None);
        assert_eq!(foreign_dom.get_charset(foreign_node), None);
    }

    #[test]
    fn test_form_control_boolean_and_value_accessors() {
        let mut dom = Dom::new();
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("disabled".to_string(), "".to_string()),
                ("required".to_string(), "".to_string()),
                ("readonly".to_string(), "".to_string()),
                ("autofocus".to_string(), "".to_string()),
                ("multiple".to_string(), "".to_string()),
                ("checked".to_string(), "".to_string()),
                ("selected".to_string(), "".to_string()),
                ("value".to_string(), "div-val".to_string()),
            ],
        });

        // 1. Test get_disabled
        // Guarded tags: input, button, select, textarea, option, optgroup, fieldset
        for tag in &[
            "input", "button", "select", "textarea", "option", "optgroup", "fieldset",
        ] {
            let node_present = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![("disabled".to_string(), "".to_string())],
            });
            let node_caps = dom.create_node(NodeData::Element {
                name: tag.to_uppercase(),
                attrs: vec![("DISABLED".to_string(), "true".to_string())],
            });
            let node_absent = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![],
            });
            assert_eq!(dom.get_disabled(node_present), Some(true));
            assert_eq!(dom.get_disabled(node_caps), Some(true));
            assert_eq!(dom.get_disabled(node_absent), Some(false));
        }
        // Negative test: div is not a guarded tag for disabled
        assert_eq!(dom.get_disabled(div_id), None);

        // 2. Test get_required
        // Guarded tags: input, select, textarea
        for tag in &["input", "select", "textarea"] {
            let node_present = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![("required".to_string(), "".to_string())],
            });
            let node_caps = dom.create_node(NodeData::Element {
                name: tag.to_uppercase(),
                attrs: vec![("REQUIRED".to_string(), "true".to_string())],
            });
            let node_absent = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![],
            });
            assert_eq!(dom.get_required(node_present), Some(true));
            assert_eq!(dom.get_required(node_caps), Some(true));
            assert_eq!(dom.get_required(node_absent), Some(false));
        }
        assert_eq!(dom.get_required(div_id), None);

        // 3. Test get_readonly
        // Guarded tags: input, textarea
        for tag in &["input", "textarea"] {
            let node_present = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![("readonly".to_string(), "".to_string())],
            });
            let node_caps = dom.create_node(NodeData::Element {
                name: tag.to_uppercase(),
                attrs: vec![("READONLY".to_string(), "true".to_string())],
            });
            let node_absent = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![],
            });
            assert_eq!(dom.get_readonly(node_present), Some(true));
            assert_eq!(dom.get_readonly(node_caps), Some(true));
            assert_eq!(dom.get_readonly(node_absent), Some(false));
        }
        assert_eq!(dom.get_readonly(div_id), None);

        // 4. Test get_autofocus
        // Any element node is valid
        let any_el_present = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("autofocus".to_string(), "".to_string())],
        });
        let any_el_caps = dom.create_node(NodeData::Element {
            name: "SPAN".to_string(),
            attrs: vec![("AUTOFOCUS".to_string(), "true".to_string())],
        });
        let any_el_absent = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_autofocus(any_el_present), Some(true));
        assert_eq!(dom.get_autofocus(any_el_caps), Some(true));
        assert_eq!(dom.get_autofocus(any_el_absent), Some(false));

        // 5. Test get_multiple
        // Guarded tags: input, select
        for tag in &["input", "select"] {
            let node_present = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![("multiple".to_string(), "".to_string())],
            });
            let node_caps = dom.create_node(NodeData::Element {
                name: tag.to_uppercase(),
                attrs: vec![("MULTIPLE".to_string(), "true".to_string())],
            });
            let node_absent = dom.create_node(NodeData::Element {
                name: tag.to_string(),
                attrs: vec![],
            });
            assert_eq!(dom.get_multiple(node_present), Some(true));
            assert_eq!(dom.get_multiple(node_caps), Some(true));
            assert_eq!(dom.get_multiple(node_absent), Some(false));
        }
        assert_eq!(dom.get_multiple(div_id), None);

        // 6. Test get_checked
        // Guarded tags: input
        let checked_present = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("checked".to_string(), "".to_string())],
        });
        let checked_caps = dom.create_node(NodeData::Element {
            name: "INPUT".to_string(),
            attrs: vec![("CHECKED".to_string(), "true".to_string())],
        });
        let checked_absent = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_checked(checked_present), Some(true));
        assert_eq!(dom.get_checked(checked_caps), Some(true));
        assert_eq!(dom.get_checked(checked_absent), Some(false));
        assert_eq!(dom.get_checked(div_id), None);

        // 7. Test get_selected
        // Guarded tags: option
        let selected_present = dom.create_node(NodeData::Element {
            name: "option".to_string(),
            attrs: vec![("selected".to_string(), "".to_string())],
        });
        let selected_caps = dom.create_node(NodeData::Element {
            name: "OPTION".to_string(),
            attrs: vec![("SELECTED".to_string(), "true".to_string())],
        });
        let selected_absent = dom.create_node(NodeData::Element {
            name: "option".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_selected(selected_present), Some(true));
        assert_eq!(dom.get_selected(selected_caps), Some(true));
        assert_eq!(dom.get_selected(selected_absent), Some(false));
        assert_eq!(dom.get_selected(div_id), None);

        // 8. Test get_textarea_value
        let textarea_present = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![("value".to_string(), "textarea-val".to_string())],
        });
        let textarea_caps = dom.create_node(NodeData::Element {
            name: "TEXTAREA".to_string(),
            attrs: vec![("VALUE".to_string(), "textarea-caps-val".to_string())],
        });
        let textarea_absent = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![],
        });
        assert_eq!(
            dom.get_textarea_value(textarea_present),
            Some("textarea-val")
        );
        assert_eq!(
            dom.get_textarea_value(textarea_caps),
            Some("textarea-caps-val")
        );
        assert_eq!(dom.get_textarea_value(textarea_absent), None);
        assert_eq!(dom.get_textarea_value(div_id), None);

        // 9. Test get_button_value
        let button_present = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![("value".to_string(), "button-val".to_string())],
        });
        let button_caps = dom.create_node(NodeData::Element {
            name: "BUTTON".to_string(),
            attrs: vec![("VALUE".to_string(), "button-caps-val".to_string())],
        });
        let button_absent = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_button_value(button_present), Some("button-val"));
        assert_eq!(dom.get_button_value(button_caps), Some("button-caps-val"));
        assert_eq!(dom.get_button_value(button_absent), None);
        assert_eq!(dom.get_button_value(div_id), None);

        // 10. Test get_option_value
        let option_present = dom.create_node(NodeData::Element {
            name: "option".to_string(),
            attrs: vec![("value".to_string(), "option-val".to_string())],
        });
        let option_caps = dom.create_node(NodeData::Element {
            name: "OPTION".to_string(),
            attrs: vec![("VALUE".to_string(), "option-caps-val".to_string())],
        });
        let option_absent = dom.create_node(NodeData::Element {
            name: "option".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_option_value(option_present), Some("option-val"));
        assert_eq!(dom.get_option_value(option_caps), Some("option-caps-val"));
        assert_eq!(dom.get_option_value(option_absent), None);
        assert_eq!(dom.get_option_value(div_id), None);

        // 11. Invalid NodeId returns None for all new getters
        let foreign_dom = Dom::new();
        let foreign_node = dom.create_node(NodeData::Text("hi".to_string()));
        assert_eq!(foreign_dom.get_disabled(foreign_node), None);
        assert_eq!(foreign_dom.get_required(foreign_node), None);
        assert_eq!(foreign_dom.get_readonly(foreign_node), None);
        assert_eq!(foreign_dom.get_autofocus(foreign_node), None);
        assert_eq!(foreign_dom.get_multiple(foreign_node), None);
        assert_eq!(foreign_dom.get_checked(foreign_node), None);
        assert_eq!(foreign_dom.get_selected(foreign_node), None);
        assert_eq!(foreign_dom.get_textarea_value(foreign_node), None);
        assert_eq!(foreign_dom.get_button_value(foreign_node), None);
        assert_eq!(foreign_dom.get_option_value(foreign_node), None);
    }

    #[test]
    fn test_media_element_accessors() {
        let mut dom = Dom::new();

        let audio = elem(&mut dom, "audio");
        let video = elem(&mut dom, "video");
        let div = elem(&mut dom, "div");

        // 1. controls
        assert_eq!(dom.get_controls(audio), Some(false));
        assert_eq!(dom.get_controls(video), Some(false));
        assert_eq!(dom.get_controls(div), None);

        dom.set_controls(audio, true);
        assert_eq!(dom.get_controls(audio), Some(true));
        assert_eq!(dom.get_attribute(audio, "controls"), Some(""));

        dom.set_controls(audio, false);
        assert_eq!(dom.get_controls(audio), Some(false));
        assert_eq!(dom.get_attribute(audio, "controls"), None);

        dom.set_controls(div, true);
        assert_eq!(dom.get_controls(div), None);
        assert_eq!(dom.get_attribute(div, "controls"), None);

        // 2. loop
        assert_eq!(dom.get_loop(audio), Some(false));
        assert_eq!(dom.get_loop(video), Some(false));
        assert_eq!(dom.get_loop(div), None);

        dom.set_loop(video, true);
        assert_eq!(dom.get_loop(video), Some(true));
        assert_eq!(dom.get_attribute(video, "loop"), Some(""));

        dom.set_loop(video, false);
        assert_eq!(dom.get_loop(video), Some(false));
        assert_eq!(dom.get_attribute(video, "loop"), None);

        dom.set_loop(div, true);
        assert_eq!(dom.get_loop(div), None);
        assert_eq!(dom.get_attribute(div, "loop"), None);

        // 3. preload
        assert_eq!(dom.get_preload(audio), None);
        assert_eq!(dom.get_preload(video), None);
        assert_eq!(dom.get_preload(div), None);

        dom.set_preload(audio, "metadata");
        assert_eq!(dom.get_preload(audio), Some("metadata"));
        assert_eq!(dom.get_attribute(audio, "preload"), Some("metadata"));

        dom.set_preload(div, "metadata");
        assert_eq!(dom.get_preload(div), None);
        assert_eq!(dom.get_attribute(div, "preload"), None);

        // 4. poster
        assert_eq!(dom.get_poster(video), None);
        assert_eq!(dom.get_poster(audio), None);
        assert_eq!(dom.get_poster(div), None);

        dom.set_poster(video, "https://example.com/poster.png");
        assert_eq!(
            dom.get_poster(video),
            Some("https://example.com/poster.png")
        );
        assert_eq!(
            dom.get_attribute(video, "poster"),
            Some("https://example.com/poster.png")
        );

        dom.set_poster(audio, "https://example.com/poster.png");
        assert_eq!(dom.get_poster(audio), None);
        assert_eq!(dom.get_attribute(audio, "poster"), None);

        dom.set_poster(div, "https://example.com/poster.png");
        assert_eq!(dom.get_poster(div), None);
        assert_eq!(dom.get_attribute(div, "poster"), None);

        let foreign_dom = Dom::new();
        let foreign_node = dom.create_node(NodeData::Text("hi".to_string()));
        assert_eq!(foreign_dom.get_controls(foreign_node), None);
        assert_eq!(foreign_dom.get_loop(foreign_node), None);
        assert_eq!(foreign_dom.get_preload(foreign_node), None);
        assert_eq!(foreign_dom.get_poster(foreign_node), None);
    }

    #[test]
    fn test_reflected_content_attribute_accessors_t0766() {
        let mut dom = Dom::new();

        // 1. Test get_cite (blockquote, q, ins, del)
        let blockquote_id = dom.create_node(NodeData::Element {
            name: "blockquote".to_string(),
            attrs: vec![("cite".to_string(), "http://cite1.com".to_string())],
        });
        let q_id = dom.create_node(NodeData::Element {
            name: "Q".to_string(),
            attrs: vec![("CITE".to_string(), "http://cite2.com".to_string())],
        });
        let ins_id = dom.create_node(NodeData::Element {
            name: "ins".to_string(),
            attrs: vec![("cite".to_string(), "http://cite3.com".to_string())],
        });
        let del_id = dom.create_node(NodeData::Element {
            name: "del".to_string(),
            attrs: vec![("cite".to_string(), "http://cite4.com".to_string())],
        });
        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![("cite".to_string(), "http://cite5.com".to_string())],
        });
        let blockquote_absent = dom.create_node(NodeData::Element {
            name: "blockquote".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_cite(blockquote_id), Some("http://cite1.com"));
        assert_eq!(dom.get_cite(q_id), Some("http://cite2.com"));
        assert_eq!(dom.get_cite(ins_id), Some("http://cite3.com"));
        assert_eq!(dom.get_cite(del_id), Some("http://cite4.com"));
        assert_eq!(dom.get_cite(div_id), None);
        assert_eq!(dom.get_cite(blockquote_absent), None);

        // 2. Test get_datetime (time, ins, del)
        let time_id = dom.create_node(NodeData::Element {
            name: "time".to_string(),
            attrs: vec![("datetime".to_string(), "2026-06-14".to_string())],
        });
        let ins_dt_id = dom.create_node(NodeData::Element {
            name: "INS".to_string(),
            attrs: vec![("DATETIME".to_string(), "2026-06-15".to_string())],
        });
        let del_dt_id = dom.create_node(NodeData::Element {
            name: "del".to_string(),
            attrs: vec![("datetime".to_string(), "2026-06-16".to_string())],
        });
        let time_absent = dom.create_node(NodeData::Element {
            name: "time".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_datetime(time_id), Some("2026-06-14"));
        assert_eq!(dom.get_datetime(ins_dt_id), Some("2026-06-15"));
        assert_eq!(dom.get_datetime(del_dt_id), Some("2026-06-16"));
        assert_eq!(dom.get_datetime(div_id), None);
        assert_eq!(dom.get_datetime(time_absent), None);

        // 3. Test get_media (link, style, source)
        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![("media".to_string(), "screen".to_string())],
        });
        let style_id = dom.create_node(NodeData::Element {
            name: "STYLE".to_string(),
            attrs: vec![("MEDIA".to_string(), "print".to_string())],
        });
        let source_id = dom.create_node(NodeData::Element {
            name: "source".to_string(),
            attrs: vec![("media".to_string(), "all".to_string())],
        });
        let link_absent = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_media(link_id), Some("screen"));
        assert_eq!(dom.get_media(style_id), Some("print"));
        assert_eq!(dom.get_media(source_id), Some("all"));
        assert_eq!(dom.get_media(div_id), None);
        assert_eq!(dom.get_media(link_absent), None);

        // 4. Test get_wrap (textarea)
        let textarea_id = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![("wrap".to_string(), "hard".to_string())],
        });
        let textarea_caps_id = dom.create_node(NodeData::Element {
            name: "TEXTAREA".to_string(),
            attrs: vec![("WRAP".to_string(), "soft".to_string())],
        });
        let textarea_absent = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_wrap(textarea_id), Some("hard"));
        assert_eq!(dom.get_wrap(textarea_caps_id), Some("soft"));
        assert_eq!(dom.get_wrap(div_id), None);
        assert_eq!(dom.get_wrap(textarea_absent), None);

        // 5. Test invalid NodeId / non-element returns None for all new getters
        let foreign_dom = Dom::new();
        let foreign_node = dom.create_node(NodeData::Text("text node".to_string()));
        assert_eq!(foreign_dom.get_cite(foreign_node), None);
        assert_eq!(foreign_dom.get_datetime(foreign_node), None);
        assert_eq!(foreign_dom.get_media(foreign_node), None);
        assert_eq!(foreign_dom.get_wrap(foreign_node), None);
    }

    #[test]
    fn test_reflected_content_attribute_accessors_t0772() {
        let mut dom = Dom::new();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![
                ("kind".to_string(), "subtitles".to_string()),
                ("srclang".to_string(), "en".to_string()),
                ("label".to_string(), "English".to_string()),
                ("sandbox".to_string(), "allow-scripts".to_string()),
                ("allow".to_string(), "geolocation".to_string()),
                ("formaction".to_string(), "/submit".to_string()),
                ("formmethod".to_string(), "post".to_string()),
                ("formtarget".to_string(), "_blank".to_string()),
                ("formnovalidate".to_string(), "".to_string()),
                ("inputmode".to_string(), "numeric".to_string()),
                ("enterkeyhint".to_string(), "search".to_string()),
                ("dirname".to_string(), "user.dir".to_string()),
                ("maxlength".to_string(), "100".to_string()),
                ("minlength".to_string(), "10".to_string()),
                ("size".to_string(), "20".to_string()),
            ],
        });

        // 1. kind, srclang, label (the <track> element trio)
        let track_id = dom.create_node(NodeData::Element {
            name: "track".to_string(),
            attrs: vec![
                ("KIND".to_string(), "subtitles".to_string()),
                ("srclang".to_string(), "en".to_string()),
                ("LABEL".to_string(), "English".to_string()),
            ],
        });
        let track_absent = dom.create_node(NodeData::Element {
            name: "track".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_kind(track_id), Some("subtitles"));
        assert_eq!(dom.get_srclang(track_id), Some("en"));
        assert_eq!(dom.get_label(track_id), Some("English"));
        assert_eq!(dom.get_kind(track_absent), None);
        assert_eq!(dom.get_srclang(track_absent), None);
        assert_eq!(dom.get_label(track_absent), None);
        assert_eq!(dom.get_kind(div_id), None);
        assert_eq!(dom.get_srclang(div_id), None);
        assert_eq!(dom.get_label(div_id), None);

        // 2. sandbox, allow (the <iframe> pair)
        let iframe_id = dom.create_node(NodeData::Element {
            name: "iframe".to_string(),
            attrs: vec![
                ("SANDBOX".to_string(), "allow-scripts".to_string()),
                ("allow".to_string(), "geolocation".to_string()),
            ],
        });
        let iframe_absent = dom.create_node(NodeData::Element {
            name: "iframe".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_sandbox(iframe_id), Some("allow-scripts"));
        assert_eq!(dom.get_allow(iframe_id), Some("geolocation"));
        assert_eq!(dom.get_sandbox(iframe_absent), None);
        assert_eq!(dom.get_allow(iframe_absent), None);
        assert_eq!(dom.get_sandbox(div_id), None);
        assert_eq!(dom.get_allow(div_id), None);

        // 3. formaction, formmethod, formtarget, formnovalidate (form-associated submit buttons)
        let input_submit_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![
                ("formaction".to_string(), "/submit".to_string()),
                ("formmethod".to_string(), "post".to_string()),
                ("FORMTARGET".to_string(), "_blank".to_string()),
                ("formnovalidate".to_string(), "".to_string()),
            ],
        });
        let button_submit_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![
                ("formaction".to_string(), "/action".to_string()),
                ("FORMMETHOD".to_string(), "get".to_string()),
                ("formtarget".to_string(), "_self".to_string()),
            ],
        });
        let input_submit_absent = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_formaction(input_submit_id), Some("/submit"));
        assert_eq!(dom.get_formmethod(input_submit_id), Some("post"));
        assert_eq!(dom.get_formtarget(input_submit_id), Some("_blank"));
        assert_eq!(dom.get_formnovalidate(input_submit_id), Some(true));

        assert_eq!(dom.get_formaction(button_submit_id), Some("/action"));
        assert_eq!(dom.get_formmethod(button_submit_id), Some("get"));
        assert_eq!(dom.get_formtarget(button_submit_id), Some("_self"));
        assert_eq!(dom.get_formnovalidate(button_submit_id), Some(false));

        assert_eq!(dom.get_formaction(input_submit_absent), None);
        assert_eq!(dom.get_formmethod(input_submit_absent), None);
        assert_eq!(dom.get_formtarget(input_submit_absent), None);
        assert_eq!(dom.get_formnovalidate(input_submit_absent), Some(false));

        assert_eq!(dom.get_formaction(div_id), None);
        assert_eq!(dom.get_formmethod(div_id), None);
        assert_eq!(dom.get_formtarget(div_id), None);
        assert_eq!(dom.get_formnovalidate(div_id), None);

        // 4. inputmode, enterkeyhint (form controls)
        let select_id = dom.create_node(NodeData::Element {
            name: "select".to_string(),
            attrs: vec![
                ("inputmode".to_string(), "numeric".to_string()),
                ("ENTERKEYHINT".to_string(), "done".to_string()),
            ],
        });
        let select_absent = dom.create_node(NodeData::Element {
            name: "select".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_inputmode(select_id), Some("numeric"));
        assert_eq!(dom.get_enterkeyhint(select_id), Some("done"));
        assert_eq!(dom.get_inputmode(select_absent), None);
        assert_eq!(dom.get_enterkeyhint(select_absent), None);
        assert_eq!(dom.get_inputmode(div_id), None);
        assert_eq!(dom.get_enterkeyhint(div_id), None);

        // 5. dirname, maxlength, minlength (input, textarea)
        let textarea_fc_id = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![
                ("DIRNAME".to_string(), "text.dir".to_string()),
                ("maxlength".to_string(), "150".to_string()),
                ("MINLENGTH".to_string(), "5".to_string()),
            ],
        });
        let textarea_fc_absent = dom.create_node(NodeData::Element {
            name: "textarea".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_dirname(textarea_fc_id), Some("text.dir"));
        assert_eq!(dom.get_maxlength(textarea_fc_id), Some("150"));
        assert_eq!(dom.get_minlength(textarea_fc_id), Some("5"));
        assert_eq!(dom.get_dirname(textarea_fc_absent), None);
        assert_eq!(dom.get_maxlength(textarea_fc_absent), None);
        assert_eq!(dom.get_minlength(textarea_fc_absent), None);
        assert_eq!(dom.get_dirname(div_id), None);
        assert_eq!(dom.get_maxlength(div_id), None);
        assert_eq!(dom.get_minlength(div_id), None);

        // 6. size (input, select)
        let input_size_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![("SIZE".to_string(), "40".to_string())],
        });
        let input_size_absent = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });

        assert_eq!(dom.get_size(input_size_id), Some("40"));
        assert_eq!(dom.get_size(input_size_absent), None);
        assert_eq!(dom.get_size(div_id), None);

        // 7. Non-element node / foreign NodeId
        let foreign_node = dom.create_node(NodeData::Text("some text".to_string()));
        assert_eq!(dom.get_kind(foreign_node), None);
        assert_eq!(dom.get_srclang(foreign_node), None);
        assert_eq!(dom.get_label(foreign_node), None);
        assert_eq!(dom.get_sandbox(foreign_node), None);
        assert_eq!(dom.get_allow(foreign_node), None);
        assert_eq!(dom.get_formaction(foreign_node), None);
        assert_eq!(dom.get_formmethod(foreign_node), None);
        assert_eq!(dom.get_formtarget(foreign_node), None);
        assert_eq!(dom.get_formnovalidate(foreign_node), None);
        assert_eq!(dom.get_inputmode(foreign_node), None);
        assert_eq!(dom.get_enterkeyhint(foreign_node), None);
        assert_eq!(dom.get_dirname(foreign_node), None);
        assert_eq!(dom.get_maxlength(foreign_node), None);
        assert_eq!(dom.get_minlength(foreign_node), None);
        assert_eq!(dom.get_size(foreign_node), None);
    }

    #[test]
    fn test_reflected_content_attribute_accessors_t0780() {
        let mut dom = Dom::new();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });

        // 1. loading (img, iframe)
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![("loading".to_string(), "lazy".to_string())],
        });
        let iframe_id = dom.create_node(NodeData::Element {
            name: "iframe".to_string(),
            attrs: vec![("loading".to_string(), "eager".to_string())],
        });
        assert_eq!(dom.get_loading(img_id), Some("lazy"));
        assert_eq!(dom.get_loading(iframe_id), Some("eager"));
        assert_eq!(dom.get_loading(div_id), None);

        dom.set_loading(img_id, "eager");
        assert_eq!(dom.get_loading(img_id), Some("eager"));
        dom.set_loading(div_id, "lazy");
        assert_eq!(dom.get_loading(div_id), None);

        // 2. ping (a, area)
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![("ping".to_string(), "https://tracker.com/ping".to_string())],
        });
        let area_id = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("ping".to_string(), "https://tracker2.com/ping".to_string())],
        });
        assert_eq!(dom.get_ping(a_id), Some("https://tracker.com/ping"));
        assert_eq!(dom.get_ping(area_id), Some("https://tracker2.com/ping"));
        assert_eq!(dom.get_ping(div_id), None);

        dom.set_ping(a_id, "https://other.com");
        assert_eq!(dom.get_ping(a_id), Some("https://other.com"));
        dom.set_ping(div_id, "https://other.com");
        assert_eq!(dom.get_ping(div_id), None);

        // 3. coords (area)
        let area_coords_id = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("coords".to_string(), "0,0,82,126".to_string())],
        });
        assert_eq!(dom.get_coords(area_coords_id), Some("0,0,82,126"));
        assert_eq!(dom.get_coords(div_id), None);

        dom.set_coords(area_coords_id, "1,2,3,4");
        assert_eq!(dom.get_coords(area_coords_id), Some("1,2,3,4"));
        dom.set_coords(div_id, "1,2,3,4");
        assert_eq!(dom.get_coords(div_id), None);

        // 4. shape (area)
        let area_shape_id = dom.create_node(NodeData::Element {
            name: "area".to_string(),
            attrs: vec![("shape".to_string(), "circle".to_string())],
        });
        assert_eq!(dom.get_shape(area_shape_id), Some("circle"));
        assert_eq!(dom.get_shape(div_id), None);

        dom.set_shape(area_shape_id, "rect");
        assert_eq!(dom.get_shape(area_shape_id), Some("rect"));
        dom.set_shape(div_id, "rect");
        assert_eq!(dom.get_shape(div_id), None);

        // 5. as, imagesrcset, imagesizes (link)
        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![
                ("as".to_string(), "image".to_string()),
                ("imagesrcset".to_string(), "logo-2x.png 2x".to_string()),
                ("imagesizes".to_string(), "50vw".to_string()),
            ],
        });
        assert_eq!(dom.get_as(link_id), Some("image"));
        assert_eq!(dom.get_image_srcset(link_id), Some("logo-2x.png 2x"));
        assert_eq!(dom.get_image_sizes(link_id), Some("50vw"));
        assert_eq!(dom.get_as(div_id), None);
        assert_eq!(dom.get_image_srcset(div_id), None);
        assert_eq!(dom.get_image_sizes(div_id), None);

        dom.set_as(link_id, "script");
        dom.set_image_srcset(link_id, "logo-3x.png 3x");
        dom.set_image_sizes(link_id, "100vw");
        assert_eq!(dom.get_as(link_id), Some("script"));
        assert_eq!(dom.get_image_srcset(link_id), Some("logo-3x.png 3x"));
        assert_eq!(dom.get_image_sizes(link_id), Some("100vw"));

        dom.set_as(div_id, "script");
        dom.set_image_srcset(div_id, "logo-3x.png 3x");
        dom.set_image_sizes(div_id, "100vw");
        assert_eq!(dom.get_as(div_id), None);
        assert_eq!(dom.get_image_srcset(div_id), None);
        assert_eq!(dom.get_image_sizes(div_id), None);

        // 6. Non-element node / foreign NodeId
        let foreign_node = dom.create_node(NodeData::Text("some text".to_string()));
        assert_eq!(dom.get_loading(foreign_node), None);
        assert_eq!(dom.get_ping(foreign_node), None);
        assert_eq!(dom.get_coords(foreign_node), None);
        assert_eq!(dom.get_shape(foreign_node), None);
        assert_eq!(dom.get_as(foreign_node), None);
        assert_eq!(dom.get_image_srcset(foreign_node), None);
        assert_eq!(dom.get_image_sizes(foreign_node), None);

        dom.set_loading(foreign_node, "lazy");
        dom.set_ping(foreign_node, "url");
        dom.set_coords(foreign_node, "0");
        dom.set_shape(foreign_node, "default");
        dom.set_as(foreign_node, "style");
        dom.set_image_srcset(foreign_node, "srcset");
        dom.set_image_sizes(foreign_node, "sizes");

        assert_eq!(dom.get_loading(foreign_node), None);
        assert_eq!(dom.get_ping(foreign_node), None);
        assert_eq!(dom.get_coords(foreign_node), None);
        assert_eq!(dom.get_shape(foreign_node), None);
        assert_eq!(dom.get_as(foreign_node), None);
        assert_eq!(dom.get_image_srcset(foreign_node), None);
        assert_eq!(dom.get_image_sizes(foreign_node), None);
    }

    #[test]
    fn test_reflected_content_attribute_accessors_t0790() {
        let mut dom = Dom::new();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });

        // 1. span (col, colgroup)
        let col_id = dom.create_node(NodeData::Element {
            name: "col".to_string(),
            attrs: vec![],
        });
        let colgroup_id = dom.create_node(NodeData::Element {
            name: "colgroup".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_span(col_id), None);
        dom.set_span(col_id, "2");
        assert_eq!(dom.get_span(col_id), Some("2"));
        dom.set_span(colgroup_id, "3");
        assert_eq!(dom.get_span(colgroup_id), Some("3"));
        dom.set_span(div_id, "4");
        assert_eq!(dom.get_span(div_id), None);

        // 2. integrity (link, script)
        let link_id = dom.create_node(NodeData::Element {
            name: "link".to_string(),
            attrs: vec![],
        });
        let script_id = dom.create_node(NodeData::Element {
            name: "script".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_integrity(link_id), None);
        dom.set_integrity(link_id, "sha256-abc");
        assert_eq!(dom.get_integrity(link_id), Some("sha256-abc"));
        dom.set_integrity(script_id, "sha256-def");
        assert_eq!(dom.get_integrity(script_id), Some("sha256-def"));
        dom.set_integrity(div_id, "sha256-xyz");
        assert_eq!(dom.get_integrity(div_id), None);

        // 3. media (link, style, source)
        dom.set_media(link_id, "screen");
        assert_eq!(dom.get_media(link_id), Some("screen"));
        let style_id = dom.create_node(NodeData::Element {
            name: "style".to_string(),
            attrs: vec![],
        });
        dom.set_media(style_id, "print");
        assert_eq!(dom.get_media(style_id), Some("print"));
        dom.set_media(div_id, "all");
        assert_eq!(dom.get_media(div_id), None);

        // 4. hreflang (generic)
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        dom.set_hreflang(a_id, "en");
        assert_eq!(dom.get_hreflang(a_id), Some("en"));

        // 5. type (button, input, etc.)
        let button_id = dom.create_node(NodeData::Element {
            name: "button".to_string(),
            attrs: vec![],
        });
        dom.set_type(button_id, "submit");
        assert_eq!(dom.get_type(button_id), Some("submit"));
        dom.set_type(div_id, "submit");
        assert_eq!(dom.get_type(div_id), None);

        // 6. referrerpolicy (generic)
        dom.set_referrer_policy(a_id, "no-referrer");
        assert_eq!(dom.get_referrer_policy(a_id), Some("no-referrer"));

        // 7. for (generic)
        let label_id = dom.create_node(NodeData::Element {
            name: "label".to_string(),
            attrs: vec![],
        });
        dom.set_for(label_id, "my-input");
        assert_eq!(dom.get_for(label_id), Some("my-input"));

        // 8. colspan, rowspan, headers (td, th)
        let td_id = dom.create_node(NodeData::Element {
            name: "td".to_string(),
            attrs: vec![],
        });
        dom.set_colspan(td_id, "4");
        dom.set_rowspan(td_id, "2");
        dom.set_headers(td_id, "h1");
        assert_eq!(dom.get_colspan(td_id), Some("4"));
        assert_eq!(dom.get_rowspan(td_id), Some("2"));
        assert_eq!(dom.get_headers(td_id), Some("h1"));

        dom.set_colspan(div_id, "4");
        assert_eq!(dom.get_colspan(div_id), None);

        // 9. scope, abbr (th)
        let th_id = dom.create_node(NodeData::Element {
            name: "th".to_string(),
            attrs: vec![],
        });
        dom.set_scope(th_id, "col");
        dom.set_abbr(th_id, "Abbreviation");
        assert_eq!(dom.get_scope(th_id), Some("col"));
        assert_eq!(dom.get_abbr(th_id), Some("Abbreviation"));

        dom.set_scope(div_id, "row");
        assert_eq!(dom.get_scope(div_id), None);

        // 10. cite (blockquote, q, ins, del)
        let bq_id = dom.create_node(NodeData::Element {
            name: "blockquote".to_string(),
            attrs: vec![],
        });
        dom.set_cite(bq_id, "http://example.com");
        assert_eq!(dom.get_cite(bq_id), Some("http://example.com"));
        dom.set_cite(div_id, "http://example.com");
        assert_eq!(dom.get_cite(div_id), None);

        // 11. datetime (time, ins, del)
        let time_id = dom.create_node(NodeData::Element {
            name: "time".to_string(),
            attrs: vec![],
        });
        dom.set_datetime(time_id, "2026-06-14");
        assert_eq!(dom.get_datetime(time_id), Some("2026-06-14"));
        dom.set_datetime(div_id, "2026-06-14");
        assert_eq!(dom.get_datetime(div_id), None);

        // 12. crossorigin (img, script, link, audio, video)
        let img_id = dom.create_node(NodeData::Element {
            name: "img".to_string(),
            attrs: vec![],
        });
        dom.set_cross_origin(img_id, "anonymous");
        assert_eq!(dom.get_cross_origin(img_id), Some("anonymous"));
        dom.set_cross_origin(div_id, "use-credentials");
        assert_eq!(dom.get_cross_origin(div_id), None);

        // 13. Boolean setters (disabled, required, readonly, autofocus, multiple, checked, selected)
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_disabled(input_id), Some(false));
        dom.set_disabled(input_id, true);
        assert_eq!(dom.get_disabled(input_id), Some(true));
        dom.set_disabled(input_id, false);
        assert_eq!(dom.get_disabled(input_id), Some(false));

        assert_eq!(dom.get_required(input_id), Some(false));
        dom.set_required(input_id, true);
        assert_eq!(dom.get_required(input_id), Some(true));
        dom.set_required(input_id, false);
        assert_eq!(dom.get_required(input_id), Some(false));

        assert_eq!(dom.get_readonly(input_id), Some(false));
        dom.set_readonly(input_id, true);
        assert_eq!(dom.get_readonly(input_id), Some(true));
        dom.set_readonly(input_id, false);
        assert_eq!(dom.get_readonly(input_id), Some(false));

        assert_eq!(dom.get_autofocus(input_id), Some(false));
        dom.set_autofocus(input_id, true);
        assert_eq!(dom.get_autofocus(input_id), Some(true));
        dom.set_autofocus(input_id, false);
        assert_eq!(dom.get_autofocus(input_id), Some(false));

        assert_eq!(dom.get_multiple(input_id), Some(false));
        dom.set_multiple(input_id, true);
        assert_eq!(dom.get_multiple(input_id), Some(true));
        dom.set_multiple(input_id, false);
        assert_eq!(dom.get_multiple(input_id), Some(false));

        assert_eq!(dom.get_checked(input_id), Some(false));
        dom.set_checked(input_id, true);
        assert_eq!(dom.get_checked(input_id), Some(true));
        dom.set_checked(input_id, false);
        assert_eq!(dom.get_checked(input_id), Some(false));

        let option_id = dom.create_node(NodeData::Element {
            name: "option".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_selected(option_id), Some(false));
        dom.set_selected(option_id, true);
        assert_eq!(dom.get_selected(option_id), Some(true));
        dom.set_selected(option_id, false);
        assert_eq!(dom.get_selected(option_id), Some(false));
    }

    #[test]
    fn test_reflected_content_attribute_accessors_t0813() {
        let mut dom = Dom::new();

        let div_id = dom.create_node(NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        });

        // 1. title, lang, dir (any element node)
        assert_eq!(dom.get_title(div_id), None);
        dom.set_title(div_id, "Hello Title");
        assert_eq!(dom.get_title(div_id), Some("Hello Title"));

        assert_eq!(dom.get_lang(div_id), None);
        dom.set_lang(div_id, "ja");
        assert_eq!(dom.get_lang(div_id), Some("ja"));

        assert_eq!(dom.get_dir(div_id), None);
        dom.set_dir(div_id, "rtl");
        assert_eq!(dom.get_dir(div_id), Some("rtl"));

        // 2. target (a, area, base, form)
        let a_id = dom.create_node(NodeData::Element {
            name: "a".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_target(a_id), None);
        dom.set_target(a_id, "_blank");
        assert_eq!(dom.get_target(a_id), Some("_blank"));
        dom.set_target(div_id, "_self");
        assert_eq!(dom.get_target(div_id), None);

        // 3. rel (any element node)
        assert_eq!(dom.get_rel(div_id), None);
        dom.set_rel(div_id, "stylesheet");
        assert_eq!(dom.get_rel(div_id), Some("stylesheet"));

        // 4. formtarget, formaction, formmethod (input, button)
        let input_id = dom.create_node(NodeData::Element {
            name: "input".to_string(),
            attrs: vec![],
        });
        assert_eq!(dom.get_formtarget(input_id), None);
        dom.set_formtarget(input_id, "_parent");
        assert_eq!(dom.get_formtarget(input_id), Some("_parent"));

        assert_eq!(dom.get_formaction(input_id), None);
        dom.set_formaction(input_id, "/submit-form");
        assert_eq!(dom.get_formaction(input_id), Some("/submit-form"));

        assert_eq!(dom.get_formmethod(input_id), None);
        dom.set_formmethod(input_id, "get");
        assert_eq!(dom.get_formmethod(input_id), Some("get"));

        // check non-submit elements are no-ops
        dom.set_formtarget(div_id, "_blank");
        assert_eq!(dom.get_formtarget(div_id), None);
        dom.set_formaction(div_id, "/post");
        assert_eq!(dom.get_formaction(div_id), None);
        dom.set_formmethod(div_id, "post");
        assert_eq!(dom.get_formmethod(div_id), None);
    }

    #[test]
    fn test_extended_reflected_attribute_accessors_t0828() {
        let mut dom = Dom::new();

        // 1. href (a, area, link, base)
        let a_id = elem(&mut dom, "a");
        let div_id = elem(&mut dom, "div");
        assert_eq!(dom.get_href(a_id), None);
        dom.set_href(a_id, "https://example.com");
        assert_eq!(dom.get_href(a_id), Some("https://example.com"));
        dom.set_href(div_id, "https://example.com"); // no-op on non-supported element
        assert_eq!(dom.get_href(div_id), None);

        // 2. src (img, script, iframe, source, audio, video, embed, track, input)
        let img_id = elem(&mut dom, "img");
        assert_eq!(dom.get_src(img_id), None);
        dom.set_src(img_id, "image.png");
        assert_eq!(dom.get_src(img_id), Some("image.png"));
        dom.set_src(div_id, "image.png"); // no-op
        assert_eq!(dom.get_src(div_id), None);

        // 3. id (any element)
        assert_eq!(dom.get_id(div_id), None);
        dom.set_id(div_id, "my-id");
        assert_eq!(dom.get_id(div_id), Some("my-id"));

        // 4. className (any element)
        assert_eq!(dom.get_class_name(div_id), None);
        dom.set_class_name(div_id, "foo bar");
        assert_eq!(dom.get_class_name(div_id), Some("foo bar"));

        // 5. alt (any element)
        assert_eq!(dom.get_alt(div_id), None);
        dom.set_alt(div_id, "Alternative text");
        assert_eq!(dom.get_alt(div_id), Some("Alternative text"));

        // 6. srcset (img, source)
        let source_id = elem(&mut dom, "source");
        assert_eq!(dom.get_srcset(source_id), None);
        dom.set_srcset(source_id, "srcset1 2x");
        assert_eq!(dom.get_srcset(source_id), Some("srcset1 2x"));
        dom.set_srcset(div_id, "srcset1 2x"); // no-op
        assert_eq!(dom.get_srcset(div_id), None);

        // 7. sizes (img, source, link)
        let link_id = elem(&mut dom, "link");
        assert_eq!(dom.get_sizes(link_id), None);
        dom.set_sizes(link_id, "50vw");
        assert_eq!(dom.get_sizes(link_id), Some("50vw"));
        dom.set_sizes(div_id, "50vw"); // no-op
        assert_eq!(dom.get_sizes(div_id), None);

        // 8. decoding (img)
        assert_eq!(dom.get_decoding(img_id), None);
        dom.set_decoding(img_id, "async");
        assert_eq!(dom.get_decoding(img_id), Some("async"));
        dom.set_decoding(div_id, "async"); // no-op
        assert_eq!(dom.get_decoding(div_id), None);

        // 9. placeholder (any element)
        assert_eq!(dom.get_placeholder(div_id), None);
        dom.set_placeholder(div_id, "Enter text...");
        assert_eq!(dom.get_placeholder(div_id), Some("Enter text..."));

        // 10. name (any element)
        assert_eq!(dom.get_name(div_id), None);
        dom.set_name(div_id, "username");
        assert_eq!(dom.get_name(div_id), Some("username"));

        // 11. download (any element)
        assert_eq!(dom.get_download(div_id), None);
        dom.set_download(div_id, "file.txt");
        assert_eq!(dom.get_download(div_id), Some("file.txt"));

        // 12. action (form)
        let form_id = elem(&mut dom, "form");
        assert_eq!(dom.get_action(form_id), Some(""));
        dom.set_action(form_id, "/submit");
        assert_eq!(dom.get_action(form_id), Some("/submit"));
        dom.set_action(div_id, "/submit"); // no-op
        assert_eq!(dom.get_action(div_id), None);

        // 13. method (form)
        assert_eq!(dom.get_method(form_id), Some(""));
        dom.set_method(form_id, "post");
        assert_eq!(dom.get_method(form_id), Some("post"));
        dom.set_method(div_id, "post"); // no-op
        assert_eq!(dom.get_method(div_id), None);

        // 14. enctype (form)
        assert_eq!(dom.get_enctype(form_id), Some(""));
        dom.set_enctype(form_id, "multipart/form-data");
        assert_eq!(dom.get_enctype(form_id), Some("multipart/form-data"));
        dom.set_enctype(div_id, "multipart/form-data"); // no-op
        assert_eq!(dom.get_enctype(div_id), None);

        // 15. width (img, canvas)
        let canvas_id = elem(&mut dom, "canvas");
        assert_eq!(dom.get_width(canvas_id), Some(""));
        dom.set_width(canvas_id, "100");
        assert_eq!(dom.get_width(canvas_id), Some("100"));
        dom.set_width(div_id, "100"); // no-op
        assert_eq!(dom.get_width(div_id), None);

        // 16. height (img, canvas)
        assert_eq!(dom.get_height(canvas_id), Some(""));
        dom.set_height(canvas_id, "200");
        assert_eq!(dom.get_height(canvas_id), Some("200"));
        dom.set_height(div_id, "200"); // no-op
        assert_eq!(dom.get_height(div_id), None);

        // 17. max (input)
        let input_id = elem(&mut dom, "input");
        assert_eq!(dom.get_max(input_id), Some(""));
        dom.set_max(input_id, "10");
        assert_eq!(dom.get_max(input_id), Some("10"));
        dom.set_max(div_id, "10"); // no-op
        assert_eq!(dom.get_max(div_id), None);

        // 18. min (input)
        assert_eq!(dom.get_min(input_id), Some(""));
        dom.set_min(input_id, "1");
        assert_eq!(dom.get_min(input_id), Some("1"));
        dom.set_min(div_id, "1"); // no-op
        assert_eq!(dom.get_min(div_id), None);

        // 19. step (input)
        assert_eq!(dom.get_step(input_id), Some(""));
        dom.set_step(input_id, "2");
        assert_eq!(dom.get_step(input_id), Some("2"));
        dom.set_step(div_id, "2"); // no-op
        assert_eq!(dom.get_step(div_id), None);

        // 20. pattern (input)
        assert_eq!(dom.get_pattern(input_id), Some(""));
        dom.set_pattern(input_id, "[0-9]+");
        assert_eq!(dom.get_pattern(input_id), Some("[0-9]+"));
        dom.set_pattern(div_id, "[0-9]+"); // no-op
        assert_eq!(dom.get_pattern(div_id), None);

        // 21. accept (input)
        assert_eq!(dom.get_accept(input_id), Some(""));
        dom.set_accept(input_id, "image/*");
        assert_eq!(dom.get_accept(input_id), Some("image/*"));
        dom.set_accept(div_id, "image/*"); // no-op
        assert_eq!(dom.get_accept(div_id), None);

        // 22. start (ol)
        let ol_id = elem(&mut dom, "ol");
        assert_eq!(dom.get_start(ol_id), None);
        dom.set_start(ol_id, "5");
        assert_eq!(dom.get_start(ol_id), Some("5"));
        dom.set_start(div_id, "5"); // no-op
        assert_eq!(dom.get_start(div_id), None);

        // 23. reversed (ol)
        assert_eq!(dom.get_reversed(ol_id), None);
        dom.set_reversed(ol_id, "reversed");
        assert_eq!(dom.get_reversed(ol_id), Some("reversed"));
        dom.set_reversed(div_id, "reversed"); // no-op
        assert_eq!(dom.get_reversed(div_id), None);

        // 24. value (li)
        let li_id = elem(&mut dom, "li");
        assert_eq!(dom.get_value(li_id), None);
        dom.set_value(li_id, "3");
        assert_eq!(dom.get_value(li_id), Some("3"));
        dom.set_value(div_id, "3"); // no-op
        assert_eq!(dom.get_value(div_id), None);

        // 25. content (meta)
        let meta_id = elem(&mut dom, "meta");
        assert_eq!(dom.get_content(meta_id), None);
        dom.set_content(meta_id, "width=device-width");
        assert_eq!(dom.get_content(meta_id), Some("width=device-width"));
        dom.set_content(div_id, "width=device-width"); // no-op
        assert_eq!(dom.get_content(div_id), None);

        // 26. http-equiv (meta)
        assert_eq!(dom.get_http_equiv(meta_id), None);
        dom.set_http_equiv(meta_id, "refresh");
        assert_eq!(dom.get_http_equiv(meta_id), Some("refresh"));
        dom.set_http_equiv(div_id, "refresh"); // no-op
        assert_eq!(dom.get_http_equiv(div_id), None);

        // 27. charset (meta)
        assert_eq!(dom.get_charset(meta_id), None);
        dom.set_charset(meta_id, "utf-8");
        assert_eq!(dom.get_charset(meta_id), Some("utf-8"));
        dom.set_charset(div_id, "utf-8"); // no-op
        assert_eq!(dom.get_charset(div_id), None);

        // 28. wrap (textarea)
        let textarea_id = elem(&mut dom, "textarea");
        assert_eq!(dom.get_wrap(textarea_id), None);
        dom.set_wrap(textarea_id, "hard");
        assert_eq!(dom.get_wrap(textarea_id), Some("hard"));
        dom.set_wrap(div_id, "hard"); // no-op
        assert_eq!(dom.get_wrap(div_id), None);

        // 29. textarea_value (textarea)
        assert_eq!(dom.get_textarea_value(textarea_id), None);
        dom.set_textarea_value(textarea_id, "some text");
        assert_eq!(dom.get_textarea_value(textarea_id), Some("some text"));
        dom.set_textarea_value(div_id, "some text"); // no-op
        assert_eq!(dom.get_textarea_value(div_id), None);

        // 30. button_value (button)
        let button_id = elem(&mut dom, "button");
        assert_eq!(dom.get_button_value(button_id), None);
        dom.set_button_value(button_id, "clickme");
        assert_eq!(dom.get_button_value(button_id), Some("clickme"));
        dom.set_button_value(div_id, "clickme"); // no-op
        assert_eq!(dom.get_button_value(div_id), None);

        // 31. option_value (option)
        let option_id = elem(&mut dom, "option");
        assert_eq!(dom.get_option_value(option_id), None);
        dom.set_option_value(option_id, "opt1");
        assert_eq!(dom.get_option_value(option_id), Some("opt1"));
        dom.set_option_value(div_id, "opt1"); // no-op
        assert_eq!(dom.get_option_value(div_id), None);

        // 32. kind (track)
        let track_id = elem(&mut dom, "track");
        assert_eq!(dom.get_kind(track_id), None);
        dom.set_kind(track_id, "subtitles");
        assert_eq!(dom.get_kind(track_id), Some("subtitles"));
        dom.set_kind(div_id, "subtitles"); // no-op
        assert_eq!(dom.get_kind(div_id), None);

        // 33. srclang (track)
        assert_eq!(dom.get_srclang(track_id), None);
        dom.set_srclang(track_id, "en");
        assert_eq!(dom.get_srclang(track_id), Some("en"));
        dom.set_srclang(div_id, "en"); // no-op
        assert_eq!(dom.get_srclang(div_id), None);

        // 34. label (track)
        assert_eq!(dom.get_label(track_id), None);
        dom.set_label(track_id, "English");
        assert_eq!(dom.get_label(track_id), Some("English"));
        dom.set_label(div_id, "English"); // no-op
        assert_eq!(dom.get_label(div_id), None);

        // 35. sandbox (iframe)
        let iframe_id = elem(&mut dom, "iframe");
        assert_eq!(dom.get_sandbox(iframe_id), None);
        dom.set_sandbox(iframe_id, "allow-scripts");
        assert_eq!(dom.get_sandbox(iframe_id), Some("allow-scripts"));
        dom.set_sandbox(div_id, "allow-scripts"); // no-op
        assert_eq!(dom.get_sandbox(div_id), None);

        // 36. allow (iframe)
        assert_eq!(dom.get_allow(iframe_id), None);
        dom.set_allow(iframe_id, "geolocation");
        assert_eq!(dom.get_allow(iframe_id), Some("geolocation"));
        dom.set_allow(div_id, "geolocation"); // no-op
        assert_eq!(dom.get_allow(div_id), None);

        // 37. formnovalidate (input, button)
        assert_eq!(dom.get_formnovalidate(input_id), Some(false));
        dom.set_formnovalidate(input_id, true);
        assert_eq!(dom.get_formnovalidate(input_id), Some(true));
        dom.set_formnovalidate(input_id, false);
        assert_eq!(dom.get_formnovalidate(input_id), Some(false));
        dom.set_formnovalidate(div_id, true); // no-op
        assert_eq!(dom.get_formnovalidate(div_id), None);

        // 38. inputmode (input, textarea, select, button)
        let select_id = elem(&mut dom, "select");
        assert_eq!(dom.get_inputmode(select_id), None);
        dom.set_inputmode(select_id, "numeric");
        assert_eq!(dom.get_inputmode(select_id), Some("numeric"));
        dom.set_inputmode(div_id, "numeric"); // no-op
        assert_eq!(dom.get_inputmode(div_id), None);

        // 39. enterkeyhint (input, textarea, select, button)
        assert_eq!(dom.get_enterkeyhint(select_id), None);
        dom.set_enterkeyhint(select_id, "search");
        assert_eq!(dom.get_enterkeyhint(select_id), Some("search"));
        dom.set_enterkeyhint(div_id, "search"); // no-op
        assert_eq!(dom.get_enterkeyhint(div_id), None);

        // 40. dirname (input, textarea)
        assert_eq!(dom.get_dirname(textarea_id), None);
        dom.set_dirname(textarea_id, "test.dir");
        assert_eq!(dom.get_dirname(textarea_id), Some("test.dir"));
        dom.set_dirname(div_id, "test.dir"); // no-op
        assert_eq!(dom.get_dirname(div_id), None);

        // 41. maxlength (input, textarea)
        assert_eq!(dom.get_maxlength(textarea_id), None);
        dom.set_maxlength(textarea_id, "50");
        assert_eq!(dom.get_maxlength(textarea_id), Some("50"));
        dom.set_maxlength(div_id, "50"); // no-op
        assert_eq!(dom.get_maxlength(div_id), None);

        // 42. minlength (input, textarea)
        assert_eq!(dom.get_minlength(textarea_id), None);
        dom.set_minlength(textarea_id, "5");
        assert_eq!(dom.get_minlength(textarea_id), Some("5"));
        dom.set_minlength(div_id, "5"); // no-op
        assert_eq!(dom.get_minlength(div_id), None);

        // 43. size (input, select)
        assert_eq!(dom.get_size(select_id), None);
        dom.set_size(select_id, "4");
        assert_eq!(dom.get_size(select_id), Some("4"));
        dom.set_size(div_id, "4"); // no-op
        assert_eq!(dom.get_size(div_id), None);
    }

    #[test]
    fn test_reflected_attribute_accessors_t0848() {
        let mut dom = Dom::new();
        let div_id = elem(&mut dom, "div");

        // 1. slot
        assert_eq!(dom.get_slot(div_id), None);
        dom.set_slot(div_id, "main-slot");
        assert_eq!(dom.get_slot(div_id), Some("main-slot"));

        // 2. nonce
        assert_eq!(dom.get_nonce(div_id), None);
        dom.set_nonce(div_id, "secret-nonce");
        assert_eq!(dom.get_nonce(div_id), Some("secret-nonce"));

        // 3. popover
        assert_eq!(dom.get_popover(div_id), None);
        dom.set_popover(div_id, "manual");
        assert_eq!(dom.get_popover(div_id), Some("manual"));

        // 4. tabindex
        assert_eq!(dom.get_tabindex(div_id), None);
        dom.set_tabindex(div_id, "3");
        assert_eq!(dom.get_tabindex(div_id), Some("3"));

        // 5. hidden (boolean)
        assert_eq!(dom.get_hidden(div_id), Some(false));
        dom.set_hidden(div_id, true);
        assert_eq!(dom.get_hidden(div_id), Some(true));
        dom.set_hidden(div_id, false);
        assert_eq!(dom.get_hidden(div_id), Some(false));

        // 6. inert (boolean)
        assert_eq!(dom.get_inert(div_id), Some(false));
        dom.set_inert(div_id, true);
        assert_eq!(dom.get_inert(div_id), Some(true));
        dom.set_inert(div_id, false);
        assert_eq!(dom.get_inert(div_id), Some(false));

        // 7. autocapitalize
        assert_eq!(dom.get_autocapitalize(div_id), None);
        dom.set_autocapitalize(div_id, "words");
        assert_eq!(dom.get_autocapitalize(div_id), Some("words"));

        // 8. accesskey
        assert_eq!(dom.get_accesskey(div_id), None);
        dom.set_accesskey(div_id, "k");
        assert_eq!(dom.get_accesskey(div_id), Some("k"));

        // 9. spellcheck
        assert_eq!(dom.get_spellcheck(div_id), None);
        dom.set_spellcheck(div_id, "true");
        assert_eq!(dom.get_spellcheck(div_id), Some("true"));

        // 10. translate
        assert_eq!(dom.get_translate(div_id), None);
        dom.set_translate(div_id, "no");
        assert_eq!(dom.get_translate(div_id), Some("no"));

        // 11. draggable
        assert_eq!(dom.get_draggable(div_id), None);
        dom.set_draggable(div_id, "true");
        assert_eq!(dom.get_draggable(div_id), Some("true"));

        // 12. contenteditable
        assert_eq!(dom.get_content_editable(div_id), None);
        dom.set_content_editable(div_id, "plaintext-only");
        assert_eq!(dom.get_content_editable(div_id), Some("plaintext-only"));

        // 13. autocorrect
        assert_eq!(dom.get_autocorrect(div_id), None);
        dom.set_autocorrect(div_id, "off");
        assert_eq!(dom.get_autocorrect(div_id), Some("off"));

        // 14. writingsuggestions
        assert_eq!(dom.get_writing_suggestions(div_id), None);
        dom.set_writing_suggestions(div_id, "true");
        assert_eq!(dom.get_writing_suggestions(div_id), Some("true"));

        // 15. itemid
        assert_eq!(dom.get_item_id(div_id), None);
        dom.set_item_id(div_id, "urn:isbn:096139210x");
        assert_eq!(dom.get_item_id(div_id), Some("urn:isbn:096139210x"));
    }
}
