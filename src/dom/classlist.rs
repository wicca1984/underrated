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

    /// Checks if a class token is valid according to the DOM Standard.
    ///
    /// A token is valid if it is not empty and does not contain any ASCII whitespace.
    pub fn is_valid_class_token(&self, token: &str) -> bool {
        !token.is_empty() && !token.chars().any(crate::ascii::is_html_whitespace)
    }

    /// Returns `true` if the element has the given class `name`.
    ///
    /// If the node is not an Element, or if `name` is empty/invalid, returns `false`.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-contains
    pub fn has_class(&self, node: NodeId, name: &str) -> bool {
        if !self.is_valid_class_token(name) {
            return false;
        }
        self.class_list(node).iter().any(|c| c == name)
    }

    /// Adds the class `name` to the element's `class` attribute.
    ///
    /// This operation is idempotent. If the class is already present,
    /// or if the node is not an Element, or if `name` is invalid, this is a no-op.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-add
    pub fn add_class(&mut self, node: NodeId, name: &str) {
        if !self.is_valid_class_token(name) {
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

    /// Adds multiple class tokens to the element's `class` attribute.
    ///
    /// This operation is idempotent. Any invalid tokens are skipped as safe no-ops.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-add
    pub fn add_classes(&mut self, node: NodeId, tokens: &[&str]) {
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let mut classes = self.class_list(node);
            let mut changed = false;
            for token in tokens {
                if self.is_valid_class_token(token) && !classes.contains(&token.to_string()) {
                    classes.push(token.to_string());
                    changed = true;
                }
            }
            if changed {
                let new_value = classes.join(" ");
                self.set_attribute(node, "class", &new_value);
            }
        }
    }

    /// Removes the class `name` from the element's `class` attribute.
    ///
    /// This drops all occurrences of the class `name`.
    /// If the node is not an Element, or if `name` is invalid, this is a no-op.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    pub fn remove_class(&mut self, node: NodeId, name: &str) {
        if !self.is_valid_class_token(name) {
            return;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let classes = self.class_list(node);
            let new_classes: Vec<String> = classes.into_iter().filter(|c| c != name).collect();
            let new_value = new_classes.join(" ");
            self.set_attribute(node, "class", &new_value);
        }
    }

    /// Removes multiple class tokens from the element's `class` attribute.
    ///
    /// Any invalid tokens are skipped as safe no-ops.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    pub fn remove_classes(&mut self, node: NodeId, tokens: &[&str]) {
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let mut classes = self.class_list(node);
            let len_before = classes.len();
            for token in tokens {
                if self.is_valid_class_token(token) {
                    classes.retain(|c| c != token);
                }
            }
            if classes.len() != len_before {
                let new_value = classes.join(" ");
                self.set_attribute(node, "class", &new_value);
            }
        }
    }

    /// Toggles the presence of class `name` in the element's `class` attribute.
    ///
    /// If the class is present, removes all its occurrences and returns `false`.
    /// If the class is absent, adds it (appends) and returns `true`.
    /// If the node is not an Element, or if `name` is invalid, this is a no-op returning `false`.
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
    /// If the node is not an Element, or if `name` is invalid, this is a no-op returning `false`.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-toggle
    pub fn toggle_class_force(&mut self, node: NodeId, name: &str, force: Option<bool>) -> bool {
        if !self.is_valid_class_token(name) {
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
    /// If `old` is not present, or if `old` or `new` is invalid, or if the node is
    /// not an Element, this is a no-op returning `false`.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-replace
    pub fn replace_class(&mut self, node: NodeId, old: &str, new: &str) -> bool {
        if !self.is_valid_class_token(old) || !self.is_valid_class_token(new) {
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

    /// Executes a callback for each class token in the element's class list.
    ///
    /// The callback receives the zero-based index and the class token.
    /// If the node is not an Element, this is a no-op.
    pub fn class_list_for_each<F>(&self, node: NodeId, mut f: F)
    where
        F: FnMut(usize, &str),
    {
        for (index, token) in self.class_list(node).iter().enumerate() {
            f(index, token);
        }
    }

    /// Returns the number of tokens in the class list of the given element.
    ///
    /// If the node is not an Element or has no `class` attribute, returns `0`.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-length
    pub fn class_list_length(&self, node: NodeId) -> usize {
        self.class_list(node).len()
    }

    /// Returns the class token at the specified zero-based index.
    ///
    /// If the index is out of bounds, or if the node is not an Element, returns `None`.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-item
    pub fn class_list_item(&self, node: NodeId, index: usize) -> Option<String> {
        let list = self.class_list(node);
        if index < list.len() {
            Some(list[index].clone())
        } else {
            None
        }
    }

    /// Returns the serialized value of the element's `class` attribute.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty string.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-value
    pub fn class_list_value(&self, node: NodeId) -> String {
        self.get_attribute(node, "class")
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    /// Sets the serialized value of the element's `class` attribute.
    ///
    /// If the node is not an Element, this is a no-op.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-value
    pub fn set_class_list_value(&mut self, node: NodeId, value: &str) {
        if let Some(NodeData::Element { .. }) = self.data(node) {
            self.set_attribute(node, "class", value);
        }
    }

    /// Returns `true` if the element has the given class `name`.
    ///
    /// If the node is not an Element, returns `false`. This is equivalent to `has_class`.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-contains
    pub fn contains_class(&self, node: NodeId, name: &str) -> bool {
        self.has_class(node, name)
    }

    /// Returns whether the token is supported. For `classList`, this always returns `false`
    /// since there are no supported tokens defined for the `class` attribute.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-supports
    pub fn class_list_supports(&self, _node: NodeId, _token: &str) -> bool {
        false
    }

    /// Returns a vector of zero-based index and class token pairs.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty vector.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-entries
    pub fn class_list_entries(&self, node: NodeId) -> Vec<(usize, String)> {
        self.class_list(node).into_iter().enumerate().collect()
    }

    /// Returns a vector of zero-based indices of the classes in the list.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty vector.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-keys
    pub fn class_list_keys(&self, node: NodeId) -> Vec<usize> {
        (0..self.class_list(node).len()).collect()
    }

    /// Returns a vector of class tokens in the list.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty vector.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-values
    pub fn class_list_values(&self, node: NodeId) -> Vec<String> {
        self.class_list(node)
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

    #[test]
    fn test_new_classlist_methods() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. Initial empty state
        assert_eq!(dom.class_list_length(el), 0);
        assert!(dom.class_list_item(el, 0).is_none());
        assert_eq!(dom.class_list_value(el), "");
        assert!(!dom.contains_class(el, "foo"));
        assert!(!dom.class_list_supports(el, "foo"));
        assert!(dom.class_list_entries(el).is_empty());
        assert!(dom.class_list_keys(el).is_empty());
        assert!(dom.class_list_values(el).is_empty());

        // 2. Setting value directly
        dom.set_class_list_value(el, "foo bar baz");
        assert_eq!(dom.class_list_value(el), "foo bar baz");
        assert_eq!(dom.class_list_length(el), 3);

        // 3. Testing contains and item
        assert!(dom.contains_class(el, "foo"));
        assert!(dom.contains_class(el, "bar"));
        assert!(!dom.contains_class(el, "qux"));
        assert_eq!(dom.class_list_item(el, 0), Some("foo".to_string()));
        assert_eq!(dom.class_list_item(el, 1), Some("bar".to_string()));
        assert_eq!(dom.class_list_item(el, 2), Some("baz".to_string()));
        assert_eq!(dom.class_list_item(el, 3), None);

        // 4. Testing supports (always false for classList)
        assert!(!dom.class_list_supports(el, "foo"));

        // 5. Testing entries, keys, values
        assert_eq!(
            dom.class_list_entries(el),
            vec![
                (0, "foo".to_string()),
                (1, "bar".to_string()),
                (2, "baz".to_string())
            ]
        );
        assert_eq!(dom.class_list_keys(el), vec![0, 1, 2]);
        assert_eq!(
            dom.class_list_values(el),
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()]
        );

        // 6. Non-element safety
        let text = dom.create_node(NodeData::Text("hello".into()));
        assert_eq!(dom.class_list_length(text), 0);
        assert!(dom.class_list_item(text, 0).is_none());
        assert_eq!(dom.class_list_value(text), "");
        assert!(!dom.contains_class(text, "foo"));
        dom.set_class_list_value(text, "foo");
        assert_eq!(dom.class_list_value(text), "");
        assert!(dom.class_list_entries(text).is_empty());
        assert!(dom.class_list_keys(text).is_empty());
        assert!(dom.class_list_values(text).is_empty());
    }

    #[test]
    fn test_classlist_completeness() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. Token validation checks
        assert!(dom.is_valid_class_token("valid-token"));
        assert!(!dom.is_valid_class_token(""));
        assert!(!dom.is_valid_class_token("has space"));
        assert!(!dom.is_valid_class_token("has\twhitespace"));
        assert!(!dom.is_valid_class_token("has\nnewline"));

        // 2. add_classes multi-token behavior
        dom.add_classes(el, &["foo", "bar", "invalid space", "baz", ""]);
        assert_eq!(dom.class_list(el), vec!["foo", "bar", "baz"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar baz"));

        // 3. remove_classes multi-token behavior
        dom.remove_classes(el, &["bar", "", "invalid space", "foo"]);
        assert_eq!(dom.class_list(el), vec!["baz"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("baz"));

        // 4. class_list_for_each behavior
        dom.add_classes(el, &["apple", "banana"]);
        let mut visited = Vec::new();
        dom.class_list_for_each(el, |idx, token| {
            visited.push((idx, token.to_string()));
        });
        assert_eq!(
            visited,
            vec![
                (0, "baz".to_string()),
                (1, "apple".to_string()),
                (2, "banana".to_string())
            ]
        );

        // 5. Safe no-ops on non-element or invalid
        let text = dom.create_node(NodeData::Text("text".into()));
        dom.add_classes(text, &["foo", "bar"]);
        assert_eq!(dom.get_attribute(text, "class"), None);
        let mut visited_text = Vec::new();
        dom.class_list_for_each(text, |idx, token| {
            visited_text.push((idx, token.to_string()));
        });
        assert!(visited_text.is_empty());
    }
}
