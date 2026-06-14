use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;

/// Errors that can occur during DOM CharacterData operations.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DomError {
    /// The offset is greater than the node's length.
    IndexSize,
    /// The operation is not supported on this type of node.
    NotSupported,
}

impl Dom {
    /// Returns the text content of the given node and its descendants.
    ///
    /// For a Text node, this returns its data.
    /// For other nodes, it returns the concatenation of the text content of all its descendant Text nodes, in tree order.
    // spec: https://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn text_content(&self, node: NodeId) -> String {
        let Some(data) = self.data(node) else {
            return String::new();
        };

        // If the node is a Text node, return its text.
        if let NodeData::Text(s) = data {
            return s.clone();
        }

        // For other nodes (Element, Document, etc.), concatenate descendant Text nodes in pre-order.
        let mut result = String::new();
        let mut stack: Vec<NodeId> = self.children(node).iter().rev().copied().collect();

        // Iterative pre-order traversal (I-6: no unbounded recursion).
        while let Some(n) = stack.pop() {
            if let Some(NodeData::Text(s)) = self.data(n) {
                result.push_str(s);
            }
            // Add children in reverse order to the stack so they are processed in correct order.
            stack.extend(self.children(n).iter().rev().copied());
        }

        result
    }

    /// Sets the text content of the given node and its descendants.
    ///
    /// For a Text node, this replaces its data.
    /// For other nodes, it detaches all descendants and inserts a single new Text node with the given value.
    // spec: https://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn set_text_content(&mut self, node: NodeId, text: &str) {
        let is_text = if let Some(n) = self.arena.get(node) {
            matches!(n.data, NodeData::Text(_))
        } else {
            return;
        };

        if is_text {
            let mut changed = false;
            if let Some(n) = self.arena.get_mut(node)
                && let NodeData::Text(ref mut s) = n.data
                && s != text
            {
                *s = text.to_string();
                changed = true;
            }
            if changed {
                self.mark_dirty(node);
            }
        } else {
            let old_children = if let Some(n) = self.arena.get_mut(node) {
                std::mem::take(&mut n.children)
            } else {
                return;
            };

            for child_id in &old_children {
                if let Some(child_node) = self.arena.get_mut(*child_id) {
                    child_node.parent = None;
                }
            }

            let mutated = !old_children.is_empty() || !text.is_empty();

            if !text.is_empty() {
                let child_id = self.create_node(NodeData::Text(text.to_string()));
                if let Some(child_node) = self.arena.get_mut(child_id) {
                    child_node.parent = Some(node);
                }
                if let Some(n) = self.arena.get_mut(node) {
                    n.children.push(child_id);
                }
            }

            if mutated {
                self.mark_dirty(node);
            }
        }
    }

    /// Returns the character data of a Text or Comment node.
    ///
    /// Returns `None` if the node is not a Text or Comment node, or if the node ID is invalid.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-data
    pub fn character_data(&self, node: NodeId) -> Option<String> {
        let data = self.data(node)?;
        match data {
            NodeData::Text(s) | NodeData::Comment(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Replaces the character data of a Text or Comment node.
    ///
    /// Returns `Err(DomError::NotSupported)` if the node is not a Text or Comment node, or if the node ID is invalid.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-data
    pub fn set_character_data(&mut self, node: NodeId, value: &str) -> Result<(), DomError> {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node) {
            match &mut n.data {
                NodeData::Text(s) | NodeData::Comment(s) => {
                    if *s != value {
                        *s = value.to_string();
                        changed = true;
                    }
                }
                _ => return Err(DomError::NotSupported),
            }
        } else {
            return Err(DomError::NotSupported);
        }

        if changed {
            self.mark_dirty(node);
        }
        Ok(())
    }

    /// Returns the length of the character data in UTF-16 code units.
    ///
    /// Returns `None` if the node is not a Text or Comment node, or if the node ID is invalid.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-length
    pub fn character_data_len(&self, node: NodeId) -> Option<usize> {
        let data = self.data(node)?;
        match data {
            NodeData::Text(s) | NodeData::Comment(s) => Some(s.encode_utf16().count()),
            _ => None,
        }
    }

    /// Returns a substring of the character data from the given UTF-16 code unit offset and count.
    ///
    /// Clamps the count if it goes beyond the end of the data.
    /// Returns `Err(DomError::IndexSize)` if the offset is greater than the data's length.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-substringdata
    pub fn substring_data(
        &self,
        node: NodeId,
        offset: usize,
        count: usize,
    ) -> Result<String, DomError> {
        let data = self.data(node).ok_or(DomError::NotSupported)?;
        let s = match data {
            NodeData::Text(s) | NodeData::Comment(s) => s,
            _ => return Err(DomError::NotSupported),
        };

        let utf16: Vec<u16> = s.encode_utf16().collect();
        let len = utf16.len();

        if offset > len {
            return Err(DomError::IndexSize);
        }

        let end = offset.checked_add(count).unwrap_or(len).min(len);
        let sub = &utf16[offset..end];
        Ok(String::from_utf16_lossy(sub))
    }

    /// Appends the given string data to the end of the character data.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-appenddata
    pub fn append_data(&mut self, node: NodeId, data: &str) -> Result<(), DomError> {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node) {
            match &mut n.data {
                NodeData::Text(s) | NodeData::Comment(s) => {
                    if !data.is_empty() {
                        s.push_str(data);
                        changed = true;
                    }
                }
                _ => return Err(DomError::NotSupported),
            }
        } else {
            return Err(DomError::NotSupported);
        }

        if changed {
            self.mark_dirty(node);
        }
        Ok(())
    }

    /// Inserts the given string data at the specified UTF-16 code unit offset.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-insertdata
    pub fn insert_data(&mut self, node: NodeId, offset: usize, data: &str) -> Result<(), DomError> {
        self.replace_data(node, offset, 0, data)
    }

    /// Deletes a range of character data starting from the specified UTF-16 code unit offset and count.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-deletedata
    pub fn delete_data(
        &mut self,
        node: NodeId,
        offset: usize,
        count: usize,
    ) -> Result<(), DomError> {
        self.replace_data(node, offset, count, "")
    }

    /// Replaces a range of character data starting from the specified UTF-16 code unit offset and count with new string data.
    // spec: https://dom.spec.whatwg.org/#dom-characterdata-replacedata
    pub fn replace_data(
        &mut self,
        node: NodeId,
        offset: usize,
        mut count: usize,
        data: &str,
    ) -> Result<(), DomError> {
        let mut changed = false;
        if let Some(n) = self.arena.get_mut(node) {
            match &mut n.data {
                NodeData::Text(s) | NodeData::Comment(s) => {
                    let mut utf16: Vec<u16> = s.encode_utf16().collect();
                    let len = utf16.len();

                    if offset > len {
                        return Err(DomError::IndexSize);
                    }

                    let overflow_or_greater = offset.checked_add(count).is_none_or(|end| end > len);
                    if overflow_or_greater {
                        count = len - offset;
                    }

                    let insert_utf16: Vec<u16> = data.encode_utf16().collect();
                    utf16.splice(offset..(offset + count), insert_utf16);

                    let new_s = String::from_utf16_lossy(&utf16);
                    if *s != new_s {
                        *s = new_s;
                        changed = true;
                    }
                }
                _ => return Err(DomError::NotSupported),
            }
        } else {
            return Err(DomError::NotSupported);
        }

        if changed {
            self.mark_dirty(node);
        }
        Ok(())
    }

    /// Splits a Text node into two Text nodes at the specified UTF-16 code unit offset.
    ///
    /// Both nodes remain in the tree as siblings, and the new Text node is returned.
    /// Returns `Err(DomError::NotSupported)` if the node is not a Text node.
    /// Returns `Err(DomError::IndexSize)` if the offset is greater than the data's length.
    // spec: https://dom.spec.whatwg.org/#dom-text-splittext
    pub fn split_text(&mut self, node: NodeId, offset: usize) -> Result<NodeId, DomError> {
        let data = self.data(node).ok_or(DomError::NotSupported)?;
        let length = match data {
            NodeData::Text(s) => s.encode_utf16().count(),
            _ => return Err(DomError::NotSupported),
        };

        if offset > length {
            return Err(DomError::IndexSize);
        }

        let count = length - offset;
        let new_data = self.substring_data(node, offset, count)?;
        let new_node = self.create_node(NodeData::Text(new_data));

        if let Some(parent) = self.parent(node) {
            let reference = if let Some(p_node) = self.arena.get(parent) {
                let idx = p_node.children.iter().position(|&c| c == node);
                idx.and_then(|i| p_node.children.get(i + 1).copied())
            } else {
                None
            };
            self.insert_before(parent, new_node, reference);
        }

        self.replace_data(node, offset, count, "")?;

        Ok(new_node)
    }

    /// Returns the contiguous text of all sibling Text nodes of the given Text node.
    ///
    /// Returns `None` if the node is not a Text node.
    // spec: https://dom.spec.whatwg.org/#dom-text-wholetext
    pub fn whole_text(&self, node: NodeId) -> Option<String> {
        let data = self.data(node)?;
        if !matches!(data, NodeData::Text(_)) {
            return None;
        }

        let parent = self.parent(node);
        match parent {
            None => {
                if let NodeData::Text(s) = data {
                    Some(s.clone())
                } else {
                    None
                }
            }
            Some(parent_id) => {
                let children = self.children(parent_id);
                let idx = children.iter().position(|&c| c == node)?;

                let mut start_idx = idx;
                while start_idx > 0 {
                    let prev_sibling = children[start_idx - 1];
                    if let Some(NodeData::Text(_)) = self.data(prev_sibling) {
                        start_idx -= 1;
                    } else {
                        break;
                    }
                }

                let mut end_idx = idx;
                while end_idx + 1 < children.len() {
                    let next_sibling = children[end_idx + 1];
                    if let Some(NodeData::Text(_)) = self.data(next_sibling) {
                        end_idx += 1;
                    } else {
                        break;
                    }
                }

                let mut result = String::new();
                for &sibling in &children[start_idx..=end_idx] {
                    if let Some(NodeData::Text(s)) = self.data(sibling) {
                        result.push_str(s);
                    }
                }
                Some(result)
            }
        }
    }

    /// Replaces the text of the given Text node and all of its contiguous Text siblings with the given data.
    ///
    /// If data is empty, the node is removed from its parent and `None` is returned.
    /// Returns `Err(DomError::NotSupported)` if the node is not a Text node.
    // spec: https://dom.spec.whatwg.org/#dom-text-replacewholetext
    pub fn replace_whole_text(
        &mut self,
        node: NodeId,
        data: &str,
    ) -> Result<Option<NodeId>, DomError> {
        let is_text = if let Some(n) = self.arena.get(node) {
            matches!(n.data, NodeData::Text(_))
        } else {
            return Err(DomError::NotSupported);
        };

        if !is_text {
            return Err(DomError::NotSupported);
        }

        let parent = self.parent(node);
        match parent {
            None => {
                if data.is_empty() {
                    self.set_character_data(node, "")?;
                    Ok(None)
                } else {
                    self.set_character_data(node, data)?;
                    Ok(Some(node))
                }
            }
            Some(parent_id) => {
                let children = self.children(parent_id);
                let idx = children
                    .iter()
                    .position(|&c| c == node)
                    .ok_or(DomError::NotSupported)?;

                // Find contiguous Text nodes
                let mut start_idx = idx;
                while start_idx > 0 {
                    let prev_sibling = children[start_idx - 1];
                    if let Some(NodeData::Text(_)) = self.data(prev_sibling) {
                        start_idx -= 1;
                    } else {
                        break;
                    }
                }

                let mut end_idx = idx;
                while end_idx + 1 < children.len() {
                    let next_sibling = children[end_idx + 1];
                    if let Some(NodeData::Text(_)) = self.data(next_sibling) {
                        end_idx += 1;
                    } else {
                        break;
                    }
                }

                let contiguous: Vec<NodeId> = children[start_idx..=end_idx].to_vec();

                if data.is_empty() {
                    // Remove all contiguous text nodes including the target node
                    for &sibling in &contiguous {
                        self.remove_child(parent_id, sibling);
                    }
                    Ok(None)
                } else {
                    // Set data of the target node
                    self.set_character_data(node, data)?;

                    // Remove all other contiguous text nodes
                    for &sibling in &contiguous {
                        if sibling != node {
                            self.remove_child(parent_id, sibling);
                        }
                    }
                    Ok(Some(node))
                }
            }
        }
    }

    /// Removes the given node from its parent.
    ///
    /// If the node has no parent, this does nothing.
    // spec: https://dom.spec.whatwg.org/#dom-childnode-remove
    pub fn remove(&mut self, node: NodeId) {
        if let Some(parent) = self.parent(node) {
            self.remove_child(parent, node);
        }
    }

    /// Inserts nodes before the given node in its parent's children.
    ///
    /// If the node has no parent, this does nothing.
    // spec: https://dom.spec.whatwg.org/#dom-childnode-before
    pub fn before(&mut self, node: NodeId, new_node: NodeId) {
        if let Some(parent) = self.parent(node) {
            self.insert_before(parent, new_node, Some(node));
        }
    }

    /// Inserts nodes after the given node in its parent's children.
    ///
    /// If the node has no parent, this does nothing.
    // spec: https://dom.spec.whatwg.org/#dom-childnode-after
    pub fn after(&mut self, node: NodeId, new_node: NodeId) {
        if let Some(parent) = self.parent(node) {
            // Find the sibling immediately following `node`.
            let next = if let Some(p_node) = self.arena.get(parent) {
                let idx = p_node.children.iter().position(|&c| c == node);
                idx.and_then(|i| p_node.children.get(i + 1).copied())
            } else {
                None
            };
            self.insert_before(parent, new_node, next);
        }
    }

    /// Replaces the given node with another node in its parent's children.
    ///
    /// If the node has no parent, this does nothing.
    // spec: https://dom.spec.whatwg.org/#dom-childnode-replacewith
    pub fn replace_with(&mut self, node: NodeId, new_node: NodeId) {
        if let Some(parent) = self.parent(node) {
            // Find the position of `node`.
            let next = if let Some(p_node) = self.arena.get(parent) {
                let idx = p_node.children.iter().position(|&c| c == node);
                idx.and_then(|i| p_node.children.get(i + 1).copied())
            } else {
                None
            };
            self.insert_before(parent, new_node, next);
            self.remove_child(parent, node);
        }
    }

    /// Normalizes the subtree rooted at the given node.
    ///
    /// This removes empty Text nodes and merges contiguous Text nodes.
    // spec: https://dom.spec.whatwg.org/#dom-node-normalize
    pub fn normalize(&mut self, node: NodeId) {
        // Post-order iterative traversal using a stack to avoid unbounded call stack recursion (I-6).
        let mut stack = vec![(node, false)];
        let mut post_order = Vec::new();

        while let Some((n, visited)) = stack.pop() {
            if visited {
                post_order.push(n);
            } else {
                stack.push((n, true));
                // Push children in reverse order to stack so they are popped in correct pre/post order
                let children = self.children(n).to_vec();
                for &child in children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }

        // Now process nodes in post-order.
        for n in post_order {
            let mut current_children = self.children(n).to_vec();
            let mut i = 0;
            while i < current_children.len() {
                let child = current_children[i];
                let is_text_opt = if let Some(NodeData::Text(text)) = self.data(child) {
                    Some(text.clone())
                } else {
                    None
                };

                if let Some(text) = is_text_opt {
                    if text.is_empty() {
                        self.remove_child(n, child);
                        current_children.remove(i);
                        continue;
                    }

                    // Look ahead to collect contiguous adjacent Text siblings
                    let mut next_idx = i + 1;
                    let mut merged_text = text;
                    let mut to_remove = Vec::new();
                    while next_idx < current_children.len() {
                        let next_child = current_children[next_idx];
                        if let Some(NodeData::Text(next_text)) = self.data(next_child) {
                            merged_text.push_str(next_text);
                            to_remove.push(next_child);
                            next_idx += 1;
                        } else {
                            break;
                        }
                    }

                    if !to_remove.is_empty() {
                        if let Some(child_node) = self.arena.get_mut(child)
                            && let NodeData::Text(ref mut s) = child_node.data
                        {
                            *s = merged_text;
                        }
                        self.mark_dirty(child);

                        for rem_child in to_remove {
                            self.remove_child(n, rem_child);
                        }
                        current_children.drain(i + 1..next_idx);
                    }
                }
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_text_content_element_with_children() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let child1 = dom.create_node(NodeData::Text("hello ".into()));
        let child2 = dom.create_node(NodeData::Text("world".into()));
        dom.append_child(parent, child1);
        dom.append_child(parent, child2);

        assert_eq!(dom.text_content(parent), "hello world");
        assert_eq!(dom.children(parent).len(), 2);

        dom.clear_dirty();
        dom.set_text_content(parent, "new text");

        assert_eq!(dom.text_content(parent), "new text");
        let children = dom.children(parent);
        assert_eq!(children.len(), 1);
        let first_child = children[0];
        assert_eq!(dom.text_content(first_child), "new text");
        assert!(dom.is_dirty(parent));
    }

    #[test]
    fn test_set_text_content_empty_on_element() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let child = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(parent, child);

        assert_eq!(dom.text_content(parent), "hello");
        assert_eq!(dom.children(parent).len(), 1);

        dom.clear_dirty();
        dom.set_text_content(parent, "");

        assert_eq!(dom.text_content(parent), "");
        assert_eq!(dom.children(parent).len(), 0);
        assert!(dom.is_dirty(parent));
    }

    #[test]
    fn test_set_text_content_text_node() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("old text".into()));

        assert_eq!(dom.text_content(text_node), "old text");

        dom.clear_dirty();
        dom.set_text_content(text_node, "new text");

        assert_eq!(dom.text_content(text_node), "new text");
        assert!(dom.is_dirty(text_node));
    }

    #[test]
    fn test_set_text_content_dirty_marking() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        assert!(!dom.is_dirty(parent));
        assert!(!dom.has_dirty());

        dom.set_text_content(parent, "dirty me");
        assert!(dom.is_dirty(parent));
        assert!(dom.has_dirty());
    }

    #[test]
    fn test_set_text_content_invalid_node_id() {
        let mut dom1 = Dom::new();
        let mut dom2 = Dom::new();
        let foreign = dom1.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        dom2.set_text_content(foreign, "ignored");
        assert_eq!(dom2.text_content(foreign), "");
        assert!(!dom2.is_dirty(foreign));
        assert!(!dom2.has_dirty());
    }

    #[test]
    fn test_character_data_basic() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("foo".into()));
        let comment_node = dom.create_node(NodeData::Comment("secret".into()));

        // Text Node Getter
        assert_eq!(dom.character_data(text_node), Some("foo".into()));
        assert_eq!(dom.character_data_len(text_node), Some(3));

        // Comment Node Getter
        assert_eq!(dom.character_data(comment_node), Some("secret".into()));
        assert_eq!(dom.character_data_len(comment_node), Some(6));

        // Text Node Setter
        dom.clear_dirty();
        assert_eq!(dom.set_character_data(text_node, "bar"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("bar".into()));
        assert!(dom.is_dirty(text_node));

        // Comment Node Setter
        dom.clear_dirty();
        assert_eq!(dom.set_character_data(comment_node, "public"), Ok(()));
        assert_eq!(dom.character_data(comment_node), Some("public".into()));
        assert!(dom.is_dirty(comment_node));
    }

    #[test]
    fn test_character_data_unsupported() {
        let mut dom = Dom::new();
        let element_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        assert_eq!(dom.character_data(element_node), None);
        assert_eq!(dom.character_data_len(element_node), None);
        assert_eq!(
            dom.set_character_data(element_node, "test"),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.substring_data(element_node, 0, 1),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.append_data(element_node, "test"),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.insert_data(element_node, 0, "test"),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.delete_data(element_node, 0, 1),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.replace_data(element_node, 0, 1, "test"),
            Err(DomError::NotSupported)
        );
    }

    #[test]
    fn test_character_data_substring() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hello world".into()));

        assert_eq!(dom.substring_data(text_node, 0, 5), Ok("hello".into()));
        assert_eq!(dom.substring_data(text_node, 6, 5), Ok("world".into()));
        assert_eq!(dom.substring_data(text_node, 6, 100), Ok("world".into()));
        assert_eq!(dom.substring_data(text_node, 11, 0), Ok("".into()));
        assert_eq!(
            dom.substring_data(text_node, 12, 0),
            Err(DomError::IndexSize)
        );
    }

    #[test]
    fn test_character_data_append() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hello".into()));

        dom.clear_dirty();
        assert_eq!(dom.append_data(text_node, " world"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("hello world".into()));
        assert!(dom.is_dirty(text_node));
    }

    #[test]
    fn test_character_data_insert() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hllo".into()));

        dom.clear_dirty();
        assert_eq!(dom.insert_data(text_node, 1, "e"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("hello".into()));
        assert!(dom.is_dirty(text_node));

        assert_eq!(dom.insert_data(text_node, 5, "!"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("hello!".into()));

        assert_eq!(dom.insert_data(text_node, 7, "?"), Err(DomError::IndexSize));
    }

    #[test]
    fn test_character_data_delete() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hello".into()));

        dom.clear_dirty();
        assert_eq!(dom.delete_data(text_node, 1, 3), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("ho".into()));
        assert!(dom.is_dirty(text_node));

        assert_eq!(dom.delete_data(text_node, 1, 10), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("h".into()));

        assert_eq!(dom.delete_data(text_node, 2, 1), Err(DomError::IndexSize));
    }

    #[test]
    fn test_character_data_replace() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hello".into()));

        dom.clear_dirty();
        assert_eq!(dom.replace_data(text_node, 1, 3, "i"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("hio".into()));
        assert!(dom.is_dirty(text_node));

        assert_eq!(dom.replace_data(text_node, 2, 10, "!"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("hi!".into()));

        assert_eq!(
            dom.replace_data(text_node, 4, 1, "test"),
            Err(DomError::IndexSize)
        );
    }

    #[test]
    fn test_character_data_utf16() {
        let mut dom = Dom::new();
        // Fox emoji "🦊" is surrogate pair in UTF-16, length 2
        let text_node = dom.create_node(NodeData::Text("a🦊b".into()));

        assert_eq!(dom.character_data_len(text_node), Some(4));

        assert_eq!(dom.substring_data(text_node, 0, 1), Ok("a".into()));
        assert_eq!(dom.substring_data(text_node, 1, 2), Ok("🦊".into()));
        assert_eq!(dom.substring_data(text_node, 3, 1), Ok("b".into()));

        // Split surrogate pair test
        let half = dom.substring_data(text_node, 1, 1).unwrap();
        assert_eq!(half, "\u{FFFD}"); // Standard replacement character for isolated surrogate in lossy UTF-8

        // Delete UTF-16 surrogate pair
        assert_eq!(dom.delete_data(text_node, 1, 2), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("ab".into()));

        // Replace with new text
        let foxy_node = dom.create_node(NodeData::Text("a🦊b".into()));
        assert_eq!(dom.replace_data(foxy_node, 1, 2, "cat"), Ok(()));
        assert_eq!(dom.character_data(foxy_node), Some("acatb".into()));
    }

    #[test]
    fn test_split_text_basic() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let text_node = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(parent, text_node);

        assert_eq!(dom.character_data(text_node), Some("hello world".into()));
        assert_eq!(dom.children(parent).len(), 1);

        dom.clear_dirty();
        let new_node = dom.split_text(text_node, 5).unwrap();

        assert_eq!(dom.character_data(text_node), Some("hello".into()));
        assert_eq!(dom.character_data(new_node), Some(" world".into()));

        let children = dom.children(parent);
        assert_eq!(children.len(), 2);
        assert_eq!(children[0], text_node);
        assert_eq!(children[1], new_node);
        assert_eq!(dom.parent(new_node), Some(parent));
        assert!(dom.is_dirty(parent));
    }

    #[test]
    fn test_split_text_no_parent() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("independent text".into()));

        let new_node = dom.split_text(text_node, 11).unwrap();
        assert_eq!(dom.character_data(text_node), Some("independent".into()));
        assert_eq!(dom.character_data(new_node), Some(" text".into()));
        assert_eq!(dom.parent(text_node), None);
        assert_eq!(dom.parent(new_node), None);
    }

    #[test]
    fn test_split_text_errors() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("abc".into()));
        let comment_node = dom.create_node(NodeData::Comment("comment".into()));
        let element_node = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });

        assert_eq!(dom.split_text(text_node, 4), Err(DomError::IndexSize));
        assert_eq!(dom.split_text(comment_node, 3), Err(DomError::NotSupported));
        assert_eq!(dom.split_text(element_node, 0), Err(DomError::NotSupported));
    }

    #[test]
    fn test_whole_text_basic() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let t1 = dom.create_node(NodeData::Text("a".into()));
        let c1 = dom.create_node(NodeData::Comment("comment".into()));
        let t2 = dom.create_node(NodeData::Text("b".into()));
        let t3 = dom.create_node(NodeData::Text("c".into()));
        let el = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        let t4 = dom.create_node(NodeData::Text("d".into()));

        dom.append_child(parent, t1);
        dom.append_child(parent, c1);
        dom.append_child(parent, t2);
        dom.append_child(parent, t3);
        dom.append_child(parent, el);
        dom.append_child(parent, t4);

        assert_eq!(dom.whole_text(t1), Some("a".into()));
        assert_eq!(dom.whole_text(t2), Some("bc".into()));
        assert_eq!(dom.whole_text(t3), Some("bc".into()));
        assert_eq!(dom.whole_text(t4), Some("d".into()));
    }

    #[test]
    fn test_whole_text_no_parent() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("independent".into()));
        assert_eq!(dom.whole_text(text_node), Some("independent".into()));
    }

    #[test]
    fn test_whole_text_errors() {
        let mut dom = Dom::new();
        let comment_node = dom.create_node(NodeData::Comment("comment".into()));
        let element_node = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });

        assert_eq!(dom.whole_text(comment_node), None);
        assert_eq!(dom.whole_text(element_node), None);
    }

    #[test]
    fn test_t0871_character_data_mutation_api() {
        let mut dom = Dom::new();
        let text_node = dom.create_node(NodeData::Text("hello".into()));

        // 1. data getter/setter
        assert_eq!(dom.character_data(text_node), Some("hello".into()));
        assert_eq!(dom.set_character_data(text_node, "world"), Ok(()));
        assert_eq!(dom.character_data(text_node), Some("world".into()));

        // 2. length on ASCII and non-BMP text (e.g. an emoji counts as 2 UTF-16 units)
        let ascii_node = dom.create_node(NodeData::Text("abc".into()));
        assert_eq!(dom.character_data_len(ascii_node), Some(3));
        let emoji_node = dom.create_node(NodeData::Text("🚀".into())); // Rocket emoji is U+1F680 (surrogate pair, length 2)
        assert_eq!(dom.character_data_len(emoji_node), Some(2));
        let mixed_node = dom.create_node(NodeData::Text("a🚀b".into()));
        assert_eq!(dom.character_data_len(mixed_node), Some(4));

        // 3. substringData normal + clamped + out-of-range error
        assert_eq!(dom.substring_data(mixed_node, 0, 1), Ok("a".into()));
        assert_eq!(dom.substring_data(mixed_node, 1, 2), Ok("🚀".into()));
        assert_eq!(dom.substring_data(mixed_node, 3, 1), Ok("b".into()));
        // clamped count
        assert_eq!(dom.substring_data(mixed_node, 1, 100), Ok("🚀b".into()));
        // out-of-range error (IndexSize)
        assert_eq!(
            dom.substring_data(mixed_node, 5, 1),
            Err(DomError::IndexSize)
        );

        // 4. appendData
        let append_node = dom.create_node(NodeData::Text("foo".into()));
        assert_eq!(dom.append_data(append_node, "bar"), Ok(()));
        assert_eq!(dom.character_data(append_node), Some("foobar".into()));

        // 5. insertData at start, middle, and end
        let insert_node = dom.create_node(NodeData::Text("ace".into()));
        // insert at start
        assert_eq!(dom.insert_data(insert_node, 0, "1"), Ok(()));
        assert_eq!(dom.character_data(insert_node), Some("1ace".into()));
        // insert in middle
        assert_eq!(dom.insert_data(insert_node, 2, "b"), Ok(()));
        assert_eq!(dom.character_data(insert_node), Some("1abce".into()));
        // insert at end
        assert_eq!(dom.insert_data(insert_node, 5, "2"), Ok(()));
        assert_eq!(dom.character_data(insert_node), Some("1abce2".into()));

        // 6. deleteData with clamping
        let delete_node = dom.create_node(NodeData::Text("abcdef".into()));
        assert_eq!(dom.delete_data(delete_node, 2, 2), Ok(())); // delete "cd"
        assert_eq!(dom.character_data(delete_node), Some("abef".into()));
        // clamping count past end
        assert_eq!(dom.delete_data(delete_node, 2, 100), Ok(())); // delete "ef" (clamped count)
        assert_eq!(dom.character_data(delete_node), Some("ab".into()));

        // 7. replaceData
        let replace_node = dom.create_node(NodeData::Text("abcde".into()));
        assert_eq!(dom.replace_data(replace_node, 1, 3, "xyz"), Ok(())); // replace "bcd" with "xyz"
        assert_eq!(dom.character_data(replace_node), Some("axyze".into()));
    }

    #[test]
    fn test_t0897_replace_whole_text() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let t1 = dom.create_node(NodeData::Text("a".into()));
        let t2 = dom.create_node(NodeData::Text("b".into()));
        let t3 = dom.create_node(NodeData::Text("c".into()));

        dom.append_child(parent, t1);
        dom.append_child(parent, t2);
        dom.append_child(parent, t3);

        // Replace whole text of t2 with "xyz"
        let res = dom.replace_whole_text(t2, "xyz");
        assert_eq!(res, Ok(Some(t2)));
        assert_eq!(dom.character_data(t2), Some("xyz".into()));

        // Contiguous nodes t1 and t3 should be removed
        let children = dom.children(parent);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], t2);

        // Try replacing with empty string
        let res_empty = dom.replace_whole_text(t2, "");
        assert_eq!(res_empty, Ok(None));
        assert_eq!(dom.children(parent).len(), 0);
    }

    #[test]
    fn test_t0897_replace_whole_text_errors_and_independent() {
        let mut dom = Dom::new();
        let comment_node = dom.create_node(NodeData::Comment("comment".into()));
        let element_node = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });

        assert_eq!(
            dom.replace_whole_text(comment_node, "test"),
            Err(DomError::NotSupported)
        );
        assert_eq!(
            dom.replace_whole_text(element_node, "test"),
            Err(DomError::NotSupported)
        );

        let independent_node = dom.create_node(NodeData::Text("independent".into()));
        let res = dom.replace_whole_text(independent_node, "new");
        assert_eq!(res, Ok(Some(independent_node)));
        assert_eq!(dom.character_data(independent_node), Some("new".into()));

        let res_empty = dom.replace_whole_text(independent_node, "");
        assert_eq!(res_empty, Ok(None));
        assert_eq!(dom.character_data(independent_node), Some("".into()));
    }

    #[test]
    fn test_t0897_child_node_methods() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let a = dom.create_node(NodeData::Text("a".into()));
        let b = dom.create_node(NodeData::Text("b".into()));
        let c = dom.create_node(NodeData::Text("c".into()));

        dom.append_child(parent, a);
        dom.append_child(parent, b);
        dom.append_child(parent, c);

        // 1. remove
        dom.remove(b);
        assert_eq!(dom.children(parent), vec![a, c]);

        // 2. before
        let before_node = dom.create_node(NodeData::Text("before".into()));
        dom.before(c, before_node);
        assert_eq!(dom.children(parent), vec![a, before_node, c]);

        // 3. after
        let after_node = dom.create_node(NodeData::Text("after".into()));
        dom.after(c, after_node);
        assert_eq!(dom.children(parent), vec![a, before_node, c, after_node]);

        // 4. replace_with
        let replaced_node = dom.create_node(NodeData::Text("replaced".into()));
        dom.replace_with(c, replaced_node);
        assert_eq!(
            dom.children(parent),
            vec![a, before_node, replaced_node, after_node]
        );
    }

    #[test]
    fn test_dom_normalize_method() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let t1 = dom.create_node(NodeData::Text("hello ".into()));
        let t2 = dom.create_node(NodeData::Text("".into()));
        let t3 = dom.create_node(NodeData::Text("world".into()));

        dom.append_child(parent, t1);
        dom.append_child(parent, t2);
        dom.append_child(parent, t3);

        assert_eq!(dom.children(parent).len(), 3);
        dom.normalize(parent);

        assert_eq!(dom.children(parent).len(), 1);
        assert_eq!(
            dom.character_data(dom.children(parent)[0]),
            Some("hello world".into())
        );
    }
}
