use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::selector;

impl Dom {
    /// Returns the first element in the document with the given `id`.
    // spec: https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
    pub fn get_element_by_id(&self, id: &str) -> Option<NodeId> {
        // Document order (pre-order) traversal.
        // We include the document root itself, although it won't match an ID attribute.
        std::iter::once(self.document())
            .chain(self.descendants(self.document()))
            .find(|&node_id| {
                if let Some(NodeData::Element { attrs, .. }) = self.data(node_id) {
                    attrs.iter().any(|(n, v)| n == "id" && v == id)
                } else {
                    false
                }
            })
    }

    /// Returns the first descendant of the given `root` node that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };

        self.descendants(root)
            .into_iter()
            .find(|&node_id| selector::matches(&selector_list, self, node_id))
    }

    /// Returns all descendants of the given `root` node that match the given `selector` in document order.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    pub fn query_selector_all(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return Vec::new(),
        };

        self.descendants(root)
            .into_iter()
            .filter(|&node_id| selector::matches(&selector_list, self, node_id))
            .collect()
    }
}
