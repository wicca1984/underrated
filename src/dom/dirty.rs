use crate::dom::Dom;
use crate::infra::NodeId;

impl Dom {
    /// Marks the specified node as layout-dirty.
    ///
    /// This method is idempotent: marking the same node multiple times
    /// will not result in duplicates in the dirty tracking set.
    pub fn mark_dirty(&mut self, node: NodeId) {
        if !self.dirty_nodes.contains(&node) {
            self.dirty_nodes.push(node);
        }
    }

    /// Checks if the specified node is currently marked as layout-dirty.
    pub fn is_dirty(&self, node: NodeId) -> bool {
        self.dirty_nodes.contains(&node)
    }

    /// Drains all currently tracked layout-dirty nodes, returning them
    /// in the order they were marked (insertion order), and clearing
    /// the internal set.
    ///
    /// // TODO(spec): The engine (S-100) will consume this in a batched
    /// // post-script-eval flush to drive relayout.
    pub fn take_dirty(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.dirty_nodes)
    }

    /// Returns whether any nodes are currently marked as layout-dirty.
    pub fn has_dirty(&self) -> bool {
        !self.dirty_nodes.is_empty()
    }

    /// Clears all currently tracked layout-dirty nodes without returning them.
    pub fn clear_dirty(&mut self) {
        self.dirty_nodes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeData;

    #[test]
    fn test_mark_dirty_is_idempotent() {
        let mut dom = Dom::new();
        let node1 = dom.create_node(NodeData::Text("node1".into()));

        assert!(!dom.is_dirty(node1));
        assert!(!dom.has_dirty());

        dom.mark_dirty(node1);
        assert!(dom.is_dirty(node1));
        assert!(dom.has_dirty());

        // Mark again to test idempotency
        dom.mark_dirty(node1);
        assert!(dom.is_dirty(node1));

        let dirties = dom.take_dirty();
        assert_eq!(dirties.len(), 1);
        assert_eq!(dirties[0], node1);
    }

    #[test]
    fn test_take_dirty_clears_set() {
        let mut dom = Dom::new();
        let node1 = dom.create_node(NodeData::Text("node1".into()));

        dom.mark_dirty(node1);
        assert!(dom.has_dirty());

        let dirties = dom.take_dirty();
        assert_eq!(dirties.len(), 1);

        assert!(!dom.has_dirty());
        assert!(!dom.is_dirty(node1));

        let dirties_after = dom.take_dirty();
        assert!(dirties_after.is_empty());
    }

    #[test]
    fn test_take_dirty_order_deterministic() {
        let mut dom = Dom::new();
        let node1 = dom.create_node(NodeData::Text("node1".into()));
        let node2 = dom.create_node(NodeData::Text("node2".into()));
        let node3 = dom.create_node(NodeData::Text("node3".into()));

        // Mark in a specific order: node2, then node3, then node1
        dom.mark_dirty(node2);
        dom.mark_dirty(node3);
        dom.mark_dirty(node1);

        // Mark node2 again (should be a no-op, preserving its first position)
        dom.mark_dirty(node2);

        let dirties = dom.take_dirty();
        assert_eq!(dirties, vec![node2, node3, node1]);
    }

    #[test]
    fn test_clear_dirty_empties_set() {
        let mut dom = Dom::new();
        let node1 = dom.create_node(NodeData::Text("node1".into()));
        let node2 = dom.create_node(NodeData::Text("node2".into()));

        dom.mark_dirty(node1);
        dom.mark_dirty(node2);
        assert!(dom.has_dirty());
        assert!(dom.is_dirty(node1));
        assert!(dom.is_dirty(node2));

        dom.clear_dirty();
        assert!(!dom.has_dirty());
        assert!(!dom.is_dirty(node1));
        assert!(!dom.is_dirty(node2));

        let dirties = dom.take_dirty();
        assert!(dirties.is_empty());
    }
}
