use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;

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
}
