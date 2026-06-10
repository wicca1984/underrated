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
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { attrs, .. } = &mut n.data
        {
            if let Some(pair) = attrs.iter_mut().find(|(k, _)| k == name) {
                pair.1 = value.to_string();
            } else {
                attrs.push((name.to_string(), value.to_string()));
            }
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
    }

    /// Replaces the text of a `Text` node. No-op for other node kinds.
    pub fn set_text(&mut self, node: NodeId, text: &str) {
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Text(t) = &mut n.data
        {
            *t = text.to_string();
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

    /// Sets the current value of an `<input>` element, marking it as dirty.
    /// No-op if the node is not an `<input>` element, or if the `NodeId` is invalid.
    pub fn set_input_value(&mut self, node: NodeId, value: &str) {
        if let Some(n) = self.arena.get_mut(node)
            && let NodeData::Element { name, .. } = &n.data
            && name.eq_ignore_ascii_case("input")
        {
            n.input_value = Some(value.to_string());
            n.input_value_dirty = true;
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
}
