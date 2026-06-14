//! classList helpers on Dom.
//!
//! These methods allow querying and modifying the classes of an Element node
//! by manipulating its `class` attribute.
//! All operations are safe and gracefully handle non-element nodes or invalid
//! / stale NodeId values as safe no-ops (no panic — I-6).

use super::{Dom, NodeData};
use crate::infra::NodeId;

impl Dom {
    /// Returns the tokens of the element's `class` attribute split on ASCII whitespace.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty vector.
    // spec: https://dom.spec.whatwg.org/#dom-element-classlist
    // spec: https://dom.spec.whatwg.org/#domtokenlist
    pub fn class_list(&self, node: NodeId) -> Vec<String> {
        if let Some(class_attr) = self.get_attribute(node, "class") {
            class_attr
                .split(crate::ascii::is_html_whitespace)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns `true` if the element has the given class `name`.
    ///
    /// If the node is not an Element, returns `false`.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-contains
    pub fn has_class(&self, node: NodeId, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        self.class_list(node).iter().any(|c| c == name)
    }

    /// Adds the class `name` to the element's `class` attribute.
    ///
    /// This operation is idempotent. If the class is already present,
    /// or if the node is not an Element, or if `name` is empty, this is a no-op.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-add
    pub fn add_class(&mut self, node: NodeId, name: &str) {
        if name.is_empty() {
            return;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let mut classes = self.class_list(node);
            if !classes.contains(&name.to_string()) {
                classes.push(name.to_string());
            }
            let new_value = classes.join(" ");
            self.set_attribute(node, "class", &new_value);
        }
    }

    /// Removes the class `name` from the element's `class` attribute.
    ///
    /// This drops all occurrences of the class `name`.
    /// If the node is not an Element, or if `name` is empty, this is a no-op.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    pub fn remove_class(&mut self, node: NodeId, name: &str) {
        if name.is_empty() {
            return;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            // TODO(spec): WHATWG DOM Standard specifies that if name is empty,
            // it should throw a SyntaxError DOMException, and if name contains
            // ASCII whitespace, it should throw an InvalidCharacterError DOMException.
            // Since we return no-op / safe failure here, we should update this if
            // DOMException-throwing bindings are introduced.
            let classes = self.class_list(node);
            let new_classes: Vec<String> = classes.into_iter().filter(|c| c != name).collect();
            let new_value = new_classes.join(" ");
            self.set_attribute(node, "class", &new_value);
        }
    }

    /// Toggles the presence of class `name` in the element's `class` attribute.
    ///
    /// If the class is present, removes all its occurrences and returns `false`.
    /// If the class is absent, adds it (appends) and returns `true`.
    /// If the node is not an Element, or if `name` is empty, this is a no-op returning `false`.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
    pub fn toggle_class(&mut self, node: NodeId, name: &str) -> bool {
        self.toggle_class_force(node, name, None)
    }

    /// Toggles the presence of class `name` in the element's `class` attribute with an optional force behavior.
    ///
    /// When force == None: behaves like the plain toggle (adds if absent and returns true; removes if present and returns false).
    /// When force == Some(true): ensures the token is present (adds if absent, leaves if present) and returns true.
    /// When force == Some(false): ensures the token is absent (removes if present, no-op if absent) and returns false.
    ///
    /// If the node is not an Element, or if `name` is empty, this is a no-op returning `false`.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
    pub fn toggle_class_force(&mut self, node: NodeId, name: &str, force: Option<bool>) -> bool {
        if name.is_empty() {
            return false;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let classes = self.class_list(node);
            let is_present = classes.contains(&name.to_string());

            let should_be_present = match force {
                Some(f) => f,
                None => !is_present,
            };

            if should_be_present {
                let mut new_classes = classes;
                if !is_present {
                    new_classes.push(name.to_string());
                }
                let new_value = new_classes.join(" ");
                self.set_attribute(node, "class", &new_value);
                true
            } else {
                let new_classes: Vec<String> = classes.into_iter().filter(|c| c != name).collect();
                let new_value = new_classes.join(" ");
                self.set_attribute(node, "class", &new_value);
                false
            }
        } else {
            false
        }
    }

    /// Replaces class `old` with class `new` in the element's `class` attribute.
    ///
    /// If `old` is present, it is replaced with `new` and returns `true`.
    /// If `new` is already present, `old` is just dropped.
    /// If `old` is not present, or if `old` or `new` is empty, or if the node is
    /// not an Element, this is a no-op returning `false`.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-replace
    pub fn replace_class(&mut self, node: NodeId, old: &str, new: &str) -> bool {
        if old.is_empty() || new.is_empty() {
            return false;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let classes = self.class_list(node);
            if !classes.contains(&old.to_string()) {
                return false;
            }
            let has_new = classes.contains(&new.to_string());
            let new_classes: Vec<String> = if has_new {
                classes.into_iter().filter(|c| c != old).collect()
            } else {
                classes
                    .into_iter()
                    .map(|c| if c == old { new.to_string() } else { c })
                    .collect()
            };
            let new_value = new_classes.join(" ");
            self.set_attribute(node, "class", &new_value);
            true
        } else {
            false
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
    fn test_class_list_empty_and_normal() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Initially empty
        assert!(dom.class_list(el).is_empty());
        assert!(!dom.has_class(el, "foo"));

        // Setting a normal class
        dom.set_attribute(el, "class", "foo bar");
        assert_eq!(dom.class_list(el), vec!["foo", "bar"]);
        assert!(dom.has_class(el, "foo"));
        assert!(dom.has_class(el, "bar"));
        assert!(!dom.has_class(el, "baz"));
    }

    #[test]
    fn test_class_list_whitespace_splitting() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Tabs, newlines, multiple spaces
        dom.set_attribute(el, "class", "  foo\tbar\n\r baz  ");
        assert_eq!(dom.class_list(el), vec!["foo", "bar", "baz"]);
        assert!(dom.has_class(el, "foo"));
        assert!(dom.has_class(el, "bar"));
        assert!(dom.has_class(el, "baz"));
    }

    #[test]
    fn test_add_class_idempotent_and_normalization() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Add once
        dom.add_class(el, "foo");
        assert_eq!(dom.class_list(el), vec!["foo"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // Add twice (idempotence check)
        dom.add_class(el, "foo");
        assert_eq!(dom.class_list(el), vec!["foo"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // Add another
        dom.add_class(el, "bar");
        assert_eq!(dom.class_list(el), vec!["foo", "bar"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar"));

        // Add existing class to a non-normalized list of classes
        dom.set_attribute(el, "class", "  foo\tbar  ");
        dom.add_class(el, "foo"); // foo already exists, but should normalize to space-separated
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar"));
    }

    #[test]
    fn test_remove_class_drops_all() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        dom.set_attribute(el, "class", "foo bar foo baz foo");
        assert_eq!(dom.class_list(el), vec!["foo", "bar", "foo", "baz", "foo"]);

        // Remove foo
        dom.remove_class(el, "foo");
        assert_eq!(dom.class_list(el), vec!["bar", "baz"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("bar baz"));

        // Remove non-existent
        dom.remove_class(el, "non-existent");
        assert_eq!(dom.get_attribute(el, "class"), Some("bar baz"));

        // Remove remaining
        dom.remove_class(el, "bar");
        dom.remove_class(el, "baz");
        assert_eq!(dom.class_list(el), Vec::<String>::new());
        assert_eq!(dom.get_attribute(el, "class"), Some(""));
    }

    #[test]
    fn test_invalid_nodes_and_non_elements() {
        let mut dom = Dom::new();
        let text = dom.create_node(NodeData::Text("hello".into()));

        // Non-elements should be safe no-ops
        assert!(dom.class_list(text).is_empty());
        assert!(!dom.has_class(text, "foo"));
        dom.add_class(text, "foo");
        dom.remove_class(text, "foo");
        assert_eq!(dom.get_attribute(text, "class"), None);

        // Invalid NodeId from another Dom instance should be safe no-ops
        let mut foreign_dom = Dom::new();
        let invalid = elem(&mut foreign_dom, "foreign");
        assert!(dom.class_list(invalid).is_empty());
        assert!(!dom.has_class(invalid, "foo"));
        dom.add_class(invalid, "foo");
        dom.remove_class(invalid, "foo");
    }

    #[test]
    fn test_toggle_class() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Toggle on empty -> adds class, returns true
        assert!(dom.toggle_class(el, "foo"));
        assert_eq!(dom.class_list(el), vec!["foo"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // Toggle existing -> removes class, returns false
        assert!(!dom.toggle_class(el, "foo"));
        assert!(dom.class_list(el).is_empty());
        assert_eq!(dom.get_attribute(el, "class"), Some(""));

        // Toggle empty string -> returns false (no-op)
        assert!(!dom.toggle_class(el, ""));
        assert!(dom.class_list(el).is_empty());

        // Toggle non-element -> returns false (no-op)
        let text = dom.create_node(NodeData::Text("hello".into()));
        assert!(!dom.toggle_class(text, "foo"));
    }

    #[test]
    fn test_toggle_class_force() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. force = Some(true) on an absent token: adds it and returns true
        assert!(dom.toggle_class_force(el, "foo", Some(true)));
        assert_eq!(dom.class_list(el), vec!["foo"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // 2. force = Some(true) on a present token: no-op (no duplicate) and returns true
        assert!(dom.toggle_class_force(el, "foo", Some(true)));
        assert_eq!(dom.class_list(el), vec!["foo"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // Add a second class for further testing
        dom.add_class(el, "bar");
        assert_eq!(dom.class_list(el), vec!["foo", "bar"]);

        // 3. force = Some(false) on a present token: removes it and returns false
        assert!(!dom.toggle_class_force(el, "foo", Some(false)));
        assert_eq!(dom.class_list(el), vec!["bar"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("bar"));

        // 4. force = Some(false) on an absent token: no-op and returns false
        assert!(!dom.toggle_class_force(el, "foo", Some(false)));
        assert_eq!(dom.class_list(el), vec!["bar"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("bar"));

        // 5. force = None on an absent token (matches existing toggle): adds it and returns true
        assert!(dom.toggle_class_force(el, "foo", None));
        assert_eq!(dom.class_list(el), vec!["bar", "foo"]);

        // 6. force = None on a present token (matches existing toggle): removes it and returns false
        assert!(!dom.toggle_class_force(el, "foo", None));
        assert_eq!(dom.class_list(el), vec!["bar"]);

        // 7. empty name: returns false (no-op)
        assert!(!dom.toggle_class_force(el, "", Some(true)));
        assert!(!dom.toggle_class_force(el, "", Some(false)));
        assert!(!dom.toggle_class_force(el, "", None));
        assert_eq!(dom.class_list(el), vec!["bar"]);

        // 8. non-Element node: returns false (no-op)
        let text = dom.create_node(NodeData::Text("hello".into()));
        assert!(!dom.toggle_class_force(text, "baz", Some(true)));
        assert!(!dom.toggle_class_force(text, "baz", Some(false)));
        assert!(!dom.toggle_class_force(text, "baz", None));
    }

    #[test]
    fn test_replace_class() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Setting initial classes
        dom.set_attribute(el, "class", "foo baz");

        // Replace present class -> returns true, class replaced
        assert!(dom.replace_class(el, "foo", "bar"));
        assert_eq!(dom.class_list(el), vec!["bar", "baz"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("bar baz"));

        // Replace absent class -> returns false (no-op)
        assert!(!dom.replace_class(el, "absent", "qux"));
        assert_eq!(dom.class_list(el), vec!["bar", "baz"]);

        // Replace with existing 'new' -> old is dropped
        dom.set_attribute(el, "class", "foo bar baz");
        assert!(dom.replace_class(el, "foo", "bar"));
        assert_eq!(dom.class_list(el), vec!["bar", "baz"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("bar baz"));

        // Replace with empty old or new -> returns false (no-op)
        assert!(!dom.replace_class(el, "", "bar"));
        assert!(!dom.replace_class(el, "bar", ""));
        assert_eq!(dom.class_list(el), vec!["bar", "baz"]);

        // Replace on non-element -> returns false (no-op)
        let text = dom.create_node(NodeData::Text("hello".into()));
        assert!(!dom.replace_class(text, "foo", "bar"));
    }
}
