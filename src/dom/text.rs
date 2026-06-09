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
}
