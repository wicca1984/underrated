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
}
