//! classList helpers on Dom.
//!
//! These methods allow querying and modifying the classes of an Element node
//! by manipulating its `class` attribute.
//! All operations are safe and gracefully handle non-element nodes or invalid
//! / stale NodeId values as safe no-ops (no panic — I-6).

use super::{Dom, NodeData};
use crate::infra::NodeId;

/// Error type for DOMTokenList class token validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassTokenError {
    /// Token is empty.
    Empty,
    /// Token contains ASCII whitespace.
    ContainsWhitespace,
    /// Supported tokens are not defined for this attribute.
    SupportedTokensNotDefined,
}

impl Dom {
    /// Returns the tokens of the element's `class` attribute split on ASCII whitespace.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty vector.
    // spec: https://dom.spec.whatwg.org/#dom-element-classlist
    // spec: https://dom.spec.whatwg.org/#domtokenlist
    pub fn class_list(&self, node: NodeId) -> Vec<String> {
        if let Some(class_attr) = self.get_attribute(node, "class") {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for token in class_attr.split(crate::ascii::is_html_whitespace) {
                if !token.is_empty() {
                    let s = token.to_string();
                    if seen.insert(s.clone()) {
                        result.push(s);
                    }
                }
            }
            result
        } else {
            Vec::new()
        }
    }

    /// Validates a class token according to the DOM Standard.
    ///
    /// Returns `Ok(())` if the token is valid.
    /// Returns `Err(ClassTokenError::Empty)` if the token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if the token contains ASCII whitespace.
    pub fn validate_class_token(&self, token: &str) -> Result<(), ClassTokenError> {
        if token.is_empty() {
            return Err(ClassTokenError::Empty);
        }
        if token.chars().any(crate::ascii::is_html_whitespace) {
            return Err(ClassTokenError::ContainsWhitespace);
        }
        Ok(())
    }

    /// Checks if a class token is valid according to the DOM Standard.
    ///
    /// A token is valid if it is not empty and does not contain any ASCII whitespace.
    pub fn is_valid_class_token(&self, token: &str) -> bool {
        self.validate_class_token(token).is_ok()
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
            let has_class_attr = self.get_attribute(node, "class").is_some();
            let mut classes = self.class_list(node);
            if !classes.contains(&name.to_string()) {
                classes.push(name.to_string());
            }
            if !classes.is_empty() || has_class_attr {
                let new_value = classes.join(" ");
                self.set_attribute(node, "class", &new_value);
            }
        }
    }

    /// Adds multiple class tokens to the element's `class` attribute.
    ///
    /// This operation is idempotent. Any invalid tokens are skipped as safe no-ops.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-add
    pub fn add_classes(&mut self, node: NodeId, tokens: &[&str]) {
        if tokens.is_empty() {
            return;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let has_class_attr = self.get_attribute(node, "class").is_some();
            let mut classes = self.class_list(node);
            for token in tokens {
                if self.is_valid_class_token(token) && !classes.contains(&token.to_string()) {
                    classes.push(token.to_string());
                }
            }
            if !classes.is_empty() || has_class_attr {
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
            let has_class_attr = self.get_attribute(node, "class").is_some();
            let classes = self.class_list(node);
            let new_classes: Vec<String> = classes.into_iter().filter(|c| c != name).collect();
            if !new_classes.is_empty() || has_class_attr {
                let new_value = new_classes.join(" ");
                self.set_attribute(node, "class", &new_value);
            }
        }
    }

    /// Removes multiple class tokens from the element's `class` attribute.
    ///
    /// Any invalid tokens are skipped as safe no-ops.
    /// The resulting `class` attribute is normalized to be space-separated.
    // spec: https://dom.spec.whatwg.org/#dom-domtokenlist-remove
    pub fn remove_classes(&mut self, node: NodeId, tokens: &[&str]) {
        if tokens.is_empty() {
            return;
        }
        if let Some(NodeData::Element { .. }) = self.data(node) {
            let has_class_attr = self.get_attribute(node, "class").is_some();
            let mut classes = self.class_list(node);
            for token in tokens {
                if self.is_valid_class_token(token) {
                    classes.retain(|c| c != token);
                }
            }
            if !classes.is_empty() || has_class_attr {
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
            let has_class_attr = self.get_attribute(node, "class").is_some();
            let classes = self.class_list(node);
            let is_present = classes.contains(&name.to_string());

            match force {
                Some(true) => {
                    if is_present {
                        true
                    } else {
                        let mut new_classes = classes;
                        new_classes.push(name.to_string());
                        let new_value = new_classes.join(" ");
                        self.set_attribute(node, "class", &new_value);
                        true
                    }
                }
                Some(false) => {
                    if is_present {
                        let new_classes: Vec<String> =
                            classes.into_iter().filter(|c| c != name).collect();
                        if !new_classes.is_empty() || has_class_attr {
                            let new_value = new_classes.join(" ");
                            self.set_attribute(node, "class", &new_value);
                        }
                        false
                    } else {
                        false
                    }
                }
                None => {
                    if is_present {
                        let new_classes: Vec<String> =
                            classes.into_iter().filter(|c| c != name).collect();
                        if !new_classes.is_empty() || has_class_attr {
                            let new_value = new_classes.join(" ");
                            self.set_attribute(node, "class", &new_value);
                        }
                        false
                    } else {
                        let mut new_classes = classes;
                        new_classes.push(name.to_string());
                        let new_value = new_classes.join(" ");
                        self.set_attribute(node, "class", &new_value);
                        true
                    }
                }
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
            let has_class_attr = self.get_attribute(node, "class").is_some();
            if old == new {
                if !classes.is_empty() || has_class_attr {
                    let new_value = classes.join(" ");
                    self.set_attribute(node, "class", &new_value);
                }
                return true;
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
            if !new_classes.is_empty() || has_class_attr {
                let new_value = new_classes.join(" ");
                self.set_attribute(node, "class", &new_value);
            }
            true
        } else {
            false
        }
    }

    /// Try adding the class `name` to the element's `class` attribute.
    ///
    /// Returns `Ok(())` if successful or if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if the token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if the token contains ASCII whitespace.
    pub fn try_add_class(&mut self, node: NodeId, name: &str) -> Result<(), ClassTokenError> {
        self.validate_class_token(name)?;
        self.add_class(node, name);
        Ok(())
    }

    /// Try adding multiple class tokens to the element's `class` attribute.
    ///
    /// Returns `Ok(())` if successful or if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if any token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if any token contains ASCII whitespace.
    pub fn try_add_classes(
        &mut self,
        node: NodeId,
        tokens: &[&str],
    ) -> Result<(), ClassTokenError> {
        if tokens.is_empty() {
            return Ok(());
        }
        for token in tokens {
            self.validate_class_token(token)?;
        }
        self.add_classes(node, tokens);
        Ok(())
    }

    /// Try removing the class `name` from the element's `class` attribute.
    ///
    /// Returns `Ok(())` if successful or if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if the token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if the token contains ASCII whitespace.
    pub fn try_remove_class(&mut self, node: NodeId, name: &str) -> Result<(), ClassTokenError> {
        self.validate_class_token(name)?;
        self.remove_class(node, name);
        Ok(())
    }

    /// Try removing multiple class tokens from the element's `class` attribute.
    ///
    /// Returns `Ok(())` if successful or if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if any token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if any token contains ASCII whitespace.
    pub fn try_remove_classes(
        &mut self,
        node: NodeId,
        tokens: &[&str],
    ) -> Result<(), ClassTokenError> {
        if tokens.is_empty() {
            return Ok(());
        }
        for token in tokens {
            self.validate_class_token(token)?;
        }
        self.remove_classes(node, tokens);
        Ok(())
    }

    /// Try toggling the presence of class `name` with optional force behavior.
    ///
    /// Returns `Ok(bool)` representing the presence state if successful, or `Ok(false)` if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if the token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if the token contains ASCII whitespace.
    pub fn try_toggle_class_force(
        &mut self,
        node: NodeId,
        name: &str,
        force: Option<bool>,
    ) -> Result<bool, ClassTokenError> {
        self.validate_class_token(name)?;
        Ok(self.toggle_class_force(node, name, force))
    }

    /// Try replacing class `old` with class `new`.
    ///
    /// Returns `Ok(true)` if old was replaced, `Ok(false)` if old was not found or if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if either token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if either token contains ASCII whitespace.
    pub fn try_replace_class(
        &mut self,
        node: NodeId,
        old: &str,
        new: &str,
    ) -> Result<bool, ClassTokenError> {
        self.validate_class_token(old)?;
        self.validate_class_token(new)?;
        Ok(self.replace_class(node, old, new))
    }

    /// Try checking if the element has the given class `name`.
    ///
    /// Returns `Ok(bool)` if successful, or `Ok(false)` if the node is not an Element (no-op).
    /// Returns `Err(ClassTokenError::Empty)` if the token is empty.
    /// Returns `Err(ClassTokenError::ContainsWhitespace)` if the token contains ASCII whitespace.
    pub fn try_contains_class(&self, node: NodeId, name: &str) -> Result<bool, ClassTokenError> {
        self.validate_class_token(name)?;
        Ok(self.contains_class(node, name))
    }

    /// Standard alias of `add_class`.
    pub fn class_list_add(&mut self, node: NodeId, token: &str) {
        self.add_class(node, token);
    }

    /// Standard alias of `add_classes`.
    pub fn class_list_add_multiple(&mut self, node: NodeId, tokens: &[&str]) {
        self.add_classes(node, tokens);
    }

    /// Standard alias of `remove_class`.
    pub fn class_list_remove(&mut self, node: NodeId, token: &str) {
        self.remove_class(node, token);
    }

    /// Standard alias of `remove_classes`.
    pub fn class_list_remove_multiple(&mut self, node: NodeId, tokens: &[&str]) {
        self.remove_classes(node, tokens);
    }

    /// Standard alias of `toggle_class_force`.
    pub fn class_list_toggle(&mut self, node: NodeId, token: &str, force: Option<bool>) -> bool {
        self.toggle_class_force(node, token, force)
    }

    /// Standard alias of `replace_class`.
    pub fn class_list_replace(&mut self, node: NodeId, old: &str, new: &str) -> bool {
        self.replace_class(node, old, new)
    }

    /// Standard alias of `contains_class`.
    pub fn class_list_contains(&self, node: NodeId, name: &str) -> bool {
        self.contains_class(node, name)
    }

    /// Standard alias of `try_add_class`.
    pub fn class_list_try_add(&mut self, node: NodeId, token: &str) -> Result<(), ClassTokenError> {
        self.try_add_class(node, token)
    }

    /// Standard alias of `try_add_classes`.
    pub fn class_list_try_add_multiple(
        &mut self,
        node: NodeId,
        tokens: &[&str],
    ) -> Result<(), ClassTokenError> {
        self.try_add_classes(node, tokens)
    }

    /// Standard alias of `try_remove_class`.
    pub fn class_list_try_remove(
        &mut self,
        node: NodeId,
        token: &str,
    ) -> Result<(), ClassTokenError> {
        self.try_remove_class(node, token)
    }

    /// Standard alias of `try_remove_classes`.
    pub fn class_list_try_remove_multiple(
        &mut self,
        node: NodeId,
        tokens: &[&str],
    ) -> Result<(), ClassTokenError> {
        self.try_remove_classes(node, tokens)
    }

    /// Standard alias of `try_toggle_class_force`.
    pub fn class_list_try_toggle(
        &mut self,
        node: NodeId,
        token: &str,
        force: Option<bool>,
    ) -> Result<bool, ClassTokenError> {
        self.try_toggle_class_force(node, token, force)
    }

    /// Standard alias of `try_replace_class`.
    pub fn class_list_try_replace(
        &mut self,
        node: NodeId,
        old: &str,
        new: &str,
    ) -> Result<bool, ClassTokenError> {
        self.try_replace_class(node, old, new)
    }

    /// Standard alias of `try_contains_class`.
    pub fn class_list_try_contains(
        &self,
        node: NodeId,
        name: &str,
    ) -> Result<bool, ClassTokenError> {
        self.try_contains_class(node, name)
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

    /// Try checking if the token is supported. For `classList`, this always returns
    /// `Err(ClassTokenError::SupportedTokensNotDefined)` per standard DOM specification.
    pub fn try_class_list_supports(
        &self,
        _node: NodeId,
        _token: &str,
    ) -> Result<bool, ClassTokenError> {
        Err(ClassTokenError::SupportedTokensNotDefined)
    }

    /// Standard alias of `try_class_list_supports`.
    pub fn class_list_try_supports(
        &self,
        node: NodeId,
        token: &str,
    ) -> Result<bool, ClassTokenError> {
        self.try_class_list_supports(node, token)
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

    /// Normalizes the element's `class` attribute by collapsing consecutive whitespaces,
    /// stripping leading/trailing whitespaces, and deduplicating tokens (preserving the first occurrence of each).
    ///
    /// If the node is not an Element or has no `class` attribute, this is a safe no-op.
    pub fn class_list_normalize(&mut self, node: NodeId) {
        if let Some(NodeData::Element { .. }) = self.data(node)
            && let Some(class_attr) = self.get_attribute(node, "class")
        {
            let classes = self.class_list(node);
            let new_value = classes.join(" ");
            if class_attr != new_value {
                self.set_attribute(node, "class", &new_value);
            }
        }
    }

    /// Returns an iterator over the unique class tokens of the element's `class` attribute.
    ///
    /// If the node is not an Element or has no `class` attribute, returns an empty iterator.
    pub fn class_list_iter(&self, node: NodeId) -> std::vec::IntoIter<String> {
        self.class_list(node).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::ClassTokenError;
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
        assert_eq!(dom.class_list(el), vec!["foo", "bar", "baz"]);

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

    #[test]
    fn test_domtokenlist_standard_compliance_and_edge_cases() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. Verification of normalization on add_class
        dom.set_attribute(el, "class", "  a   b  ");
        dom.add_class(el, "a"); // "a" is already present, but normalizes attribute spacing per engine design
        assert_eq!(dom.get_attribute(el, "class"), Some("a b"));

        dom.set_attribute(el, "class", "  a   b  ");
        dom.add_class(el, "c"); // "c" is not present, adds "c" and normalizes spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("a b c"));

        // 2. Verification of normalization on remove_class
        dom.set_attribute(el, "class", "  a   b  ");
        dom.remove_class(el, "c"); // "c" is not present, but normalizes attribute spacing per engine design
        assert_eq!(dom.get_attribute(el, "class"), Some("a b"));

        dom.set_attribute(el, "class", "  a   b  ");
        dom.remove_class(el, "a"); // "a" is present, removes "a" and normalizes spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("b"));

        // 3. Verification of standard-compliant toggle_class_force (does not normalize on no-op force toggle)
        dom.set_attribute(el, "class", "  a   b  ");
        assert!(dom.toggle_class_force(el, "a", Some(true))); // "a" already present, returns true, does NOT normalize spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("  a   b  "));

        dom.set_attribute(el, "class", "  a   b  ");
        assert!(!dom.toggle_class_force(el, "c", Some(false))); // "c" already absent, returns false, does NOT normalize spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("  a   b  "));

        // 3b. Verify normalization happens when there IS a change with force toggle
        dom.set_attribute(el, "class", "  a   b  ");
        assert!(dom.toggle_class_force(el, "c", Some(true))); // "c" absent, adds c, returns true and normalizes spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("a b c"));

        dom.set_attribute(el, "class", "  a   b  ");
        assert!(!dom.toggle_class_force(el, "a", Some(false))); // "a" present, removes a, returns false and normalizes spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("b"));

        // 4. Verification of replace_class old == new (now normalizes when present per spec update steps)
        dom.set_attribute(el, "class", "  a   b  ");
        assert!(dom.replace_class(el, "a", "a")); // "a" is present, returns true and normalizes spacing
        assert_eq!(dom.get_attribute(el, "class"), Some("a b"));

        dom.set_attribute(el, "class", "  a   b  ");
        assert!(!dom.replace_class(el, "c", "c")); // "c" is absent, returns false without altering attribute
        assert_eq!(dom.get_attribute(el, "class"), Some("  a   b  "));

        // 5. Verification of replace_class old != new where new is already present (old is dropped)
        dom.set_attribute(el, "class", "a b");
        assert!(dom.replace_class(el, "a", "b")); // "b" is already present, so "a" is dropped and returns true
        assert_eq!(dom.get_attribute(el, "class"), Some("b"));
    }

    #[test]
    fn test_domtokenlist_new_features_and_aliases() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. ClassTokenError validation
        assert_eq!(dom.validate_class_token(""), Err(ClassTokenError::Empty));
        assert_eq!(
            dom.validate_class_token("foo bar"),
            Err(ClassTokenError::ContainsWhitespace)
        );
        assert_eq!(dom.validate_class_token("foo"), Ok(()));

        // 2. class_list_add and class_list_add_multiple aliases
        dom.class_list_add(el, "x");
        assert_eq!(dom.get_attribute(el, "class"), Some("x"));
        dom.class_list_add_multiple(el, &["y", "z"]);
        assert_eq!(dom.get_attribute(el, "class"), Some("x y z"));

        // 3. class_list_contains alias
        assert!(dom.class_list_contains(el, "y"));
        assert!(!dom.class_list_contains(el, "w"));

        // 4. class_list_toggle alias
        assert!(!dom.class_list_toggle(el, "y", None)); // removes "y"
        assert_eq!(dom.get_attribute(el, "class"), Some("x z"));
        assert!(dom.class_list_toggle(el, "y", Some(true))); // adds "y"
        assert_eq!(dom.get_attribute(el, "class"), Some("x z y"));

        // 5. class_list_replace alias
        assert!(dom.class_list_replace(el, "z", "w"));
        assert_eq!(dom.get_attribute(el, "class"), Some("x w y"));

        // 6. class_list_remove and class_list_remove_multiple aliases
        dom.class_list_remove(el, "x");
        assert_eq!(dom.get_attribute(el, "class"), Some("w y"));
        dom.class_list_remove_multiple(el, &["w", "y"]);
        assert_eq!(dom.get_attribute(el, "class"), Some(""));
    }

    #[test]
    fn test_classlist_normalize_and_iter() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // Test normalizer on empty/absent class attribute
        dom.class_list_normalize(el);
        assert_eq!(dom.get_attribute(el, "class"), None);

        // Test normalizer on non-element node
        let text = dom.create_node(NodeData::Text("hello".into()));
        dom.class_list_normalize(text);
        assert_eq!(dom.get_attribute(text, "class"), None);

        // Test normalizer with duplicates and multiple whitespaces
        dom.set_attribute(el, "class", "  foo   bar   foo  baz\tbar  ");
        dom.class_list_normalize(el);
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar baz"));

        // Test iterator
        dom.set_attribute(el, "class", "apple banana orange");
        let tokens: Vec<String> = dom.class_list_iter(el).collect();
        assert_eq!(
            tokens,
            vec![
                "apple".to_string(),
                "banana".to_string(),
                "orange".to_string()
            ]
        );

        // Test iterator on absent/non-element node
        let text2 = dom.create_node(NodeData::Text("hello".into()));
        let tokens_empty: Vec<String> = dom.class_list_iter(text2).collect();
        assert!(tokens_empty.is_empty());
    }

    #[test]
    fn test_domtokenlist_extended_try_semantics() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. Valid try_add_class and try_contains_class
        assert_eq!(dom.try_add_class(el, "foo"), Ok(()));
        assert_eq!(dom.try_contains_class(el, "foo"), Ok(true));
        assert_eq!(dom.get_attribute(el, "class"), Some("foo"));

        // 2. Invalid validation cases
        assert_eq!(dom.try_add_class(el, ""), Err(ClassTokenError::Empty));
        assert_eq!(
            dom.try_add_class(el, "foo bar"),
            Err(ClassTokenError::ContainsWhitespace)
        );
        assert_eq!(dom.try_contains_class(el, ""), Err(ClassTokenError::Empty));
        assert_eq!(
            dom.try_contains_class(el, "foo bar"),
            Err(ClassTokenError::ContainsWhitespace)
        );

        // 3. Valid try_add_classes and validation on invalid tokens
        assert_eq!(dom.try_add_classes(el, &["bar", "baz"]), Ok(()));
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar baz"));
        assert_eq!(
            dom.try_add_classes(el, &["qux", ""]),
            Err(ClassTokenError::Empty)
        );

        // 4. try_remove_class and validation
        assert_eq!(dom.try_remove_class(el, "bar"), Ok(()));
        assert_eq!(dom.get_attribute(el, "class"), Some("foo baz"));
        assert_eq!(
            dom.try_remove_class(el, " "),
            Err(ClassTokenError::ContainsWhitespace)
        );

        // 5. try_remove_classes and validation
        assert_eq!(dom.try_remove_classes(el, &["foo"]), Ok(()));
        assert_eq!(dom.get_attribute(el, "class"), Some("baz"));
        assert_eq!(
            dom.try_remove_classes(el, &["baz", "invalid whitespace"]),
            Err(ClassTokenError::ContainsWhitespace)
        );

        // 6. try_toggle_class_force and validation
        assert_eq!(
            dom.try_toggle_class_force(el, "newclass", Some(true)),
            Ok(true)
        );
        assert_eq!(dom.get_attribute(el, "class"), Some("baz newclass"));
        assert_eq!(
            dom.try_toggle_class_force(el, "newclass", Some(false)),
            Ok(false)
        );
        assert_eq!(dom.get_attribute(el, "class"), Some("baz"));
        assert_eq!(
            dom.try_toggle_class_force(el, "with space", None),
            Err(ClassTokenError::ContainsWhitespace)
        );

        // 7. try_replace_class and validation
        assert_eq!(dom.try_replace_class(el, "baz", "replaced"), Ok(true));
        assert_eq!(dom.get_attribute(el, "class"), Some("replaced"));
        assert_eq!(
            dom.try_replace_class(el, "replaced", ""),
            Err(ClassTokenError::Empty)
        );
        assert_eq!(
            dom.try_replace_class(el, "", "replaced"),
            Err(ClassTokenError::Empty)
        );

        // 8. try_class_list_supports and class_list_try_supports
        assert_eq!(
            dom.try_class_list_supports(el, "foo"),
            Err(ClassTokenError::SupportedTokensNotDefined)
        );
        assert_eq!(
            dom.class_list_try_supports(el, "bar"),
            Err(ClassTokenError::SupportedTokensNotDefined)
        );

        // 9. Safe no-ops on non-element or invalid
        let text = dom.create_node(NodeData::Text("hello".into()));
        assert_eq!(dom.try_add_class(text, "foo"), Ok(()));
        assert_eq!(dom.try_contains_class(text, "foo"), Ok(false));
        assert_eq!(dom.try_remove_class(text, "foo"), Ok(()));
        assert_eq!(dom.try_toggle_class_force(text, "foo", None), Ok(false));
        assert_eq!(dom.try_replace_class(text, "foo", "bar"), Ok(false));

        // 10. Verification that empty token slices result in no-op early returns
        dom.set_attribute(el, "class", "  original   spacing  ");
        assert_eq!(dom.try_add_classes(el, &[]), Ok(()));
        assert_eq!(
            dom.get_attribute(el, "class"),
            Some("  original   spacing  ")
        );
        assert_eq!(dom.try_remove_classes(el, &[]), Ok(()));
        assert_eq!(
            dom.get_attribute(el, "class"),
            Some("  original   spacing  ")
        );
    }

    #[test]
    fn test_domtokenlist_advanced_compliance_additional() {
        let mut dom = Dom::new();
        let el = elem(&mut dom, "div");

        // 1. Exact check on all 5 HTML ASCII whitespace characters: \t, \n, \r, \x0c, and ' '
        for ws_char in &['\t', '\n', '\r', '\x0C', ' '] {
            let token = format!("foo{}bar", ws_char);
            assert_eq!(
                dom.validate_class_token(&token),
                Err(ClassTokenError::ContainsWhitespace),
                "Failed to reject whitespace character: {:?}",
                ws_char
            );
        }

        // 2. Transactional safety for try_add_classes:
        // If there's an error in any token, NO mutation is applied to the class attribute at all.
        dom.set_attribute(el, "class", "initial");
        let result = dom.try_add_classes(el, &["new1", "invalid token", "new2"]);
        assert_eq!(result, Err(ClassTokenError::ContainsWhitespace));
        // Verify it was transactional (not even 'new1' was added)
        assert_eq!(dom.get_attribute(el, "class"), Some("initial"));

        // 3. Transactional safety for try_remove_classes:
        // If there's an error in any token, NO mutation is applied.
        dom.set_attribute(el, "class", "foo bar baz");
        let result = dom.try_remove_classes(el, &["foo", "invalid token", "baz"]);
        assert_eq!(result, Err(ClassTokenError::ContainsWhitespace));
        // Verify it was transactional (neither 'foo' nor 'baz' was removed)
        assert_eq!(dom.get_attribute(el, "class"), Some("foo bar baz"));

        // 4. Checking replace_class with old == new where old is absent vs present
        dom.set_attribute(el, "class", "  a   b  ");
        assert!(!dom.replace_class(el, "c", "c")); // c absent, returns false and doesn't normalize
        assert_eq!(dom.get_attribute(el, "class"), Some("  a   b  "));

        assert!(dom.replace_class(el, "a", "a")); // a present, returns true and normalizes
        assert_eq!(dom.get_attribute(el, "class"), Some("a b"));

        // 5. Verification of all-or-nothing check in try_replace_class
        assert_eq!(
            dom.try_replace_class(el, "a", "with space"),
            Err(ClassTokenError::ContainsWhitespace)
        );
        // Spacing must not be normalized or altered since it errored early
        dom.set_attribute(el, "class", "  a   b  ");
        assert_eq!(
            dom.try_replace_class(el, "a", "with space"),
            Err(ClassTokenError::ContainsWhitespace)
        );
        assert_eq!(dom.get_attribute(el, "class"), Some("  a   b  "));

        // 6. class_list_length & class_list_item correctness
        dom.set_attribute(el, "class", "  p   q   p  ");
        assert_eq!(dom.class_list_length(el), 2);
        assert_eq!(dom.class_list_item(el, 0), Some("p".to_string()));
        assert_eq!(dom.class_list_item(el, 1), Some("q".to_string()));
        assert_eq!(dom.class_list_item(el, 2), None);

        // 7. Verify class_list_keys, class_list_values, class_list_entries
        assert_eq!(dom.class_list_keys(el), vec![0, 1]);
        assert_eq!(
            dom.class_list_values(el),
            vec!["p".to_string(), "q".to_string()]
        );
        assert_eq!(
            dom.class_list_entries(el),
            vec![(0, "p".to_string()), (1, "q".to_string())]
        );
    }
}
