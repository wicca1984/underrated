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

    /// Returns the first element in the document that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };

        std::iter::once(self.document())
            .chain(self.descendants(self.document()))
            .find(|&node_id| selector::matches(&selector_list, self, node_id))
    }

    /// Returns all elements in the document that match the given `selector` in document order.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return Vec::new(),
        };

        std::iter::once(self.document())
            .chain(self.descendants(self.document()))
            .filter(|&node_id| selector::matches(&selector_list, self, node_id))
            .collect()
    }
}
