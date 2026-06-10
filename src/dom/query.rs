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
            .chain(self.descendants_iter(self.document()))
            .find(|&node_id| {
                if let Some(NodeData::Element { attrs, .. }) = self.data(node_id) {
                    attrs.iter().any(|(n, v)| n == "id" && v == id)
                } else {
                    false
                }
            })
    }

    /// Returns the first descendant of the document root that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, selector: &str) -> Option<NodeId> {
        self.query_selector_from(self.document(), selector)
    }

    /// Returns all descendants of the document root that match the given `selector` in document order.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    pub fn query_selector_all(&self, selector: &str) -> Vec<NodeId> {
        self.query_selector_all_from(self.document(), selector)
    }

    /// Returns the first descendant of the given `root` node that matches the given `selector`.
    pub fn query_selector_from(&self, root: NodeId, selector: &str) -> Option<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };

        self.descendants_iter(root)
            .find(|&node_id| selector::matches(&selector_list, self, node_id))
    }

    /// Returns all descendants of the given `root` node that match the given `selector` in document order.
    pub fn query_selector_all_from(&self, root: NodeId, selector: &str) -> Vec<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return Vec::new(),
        };

        self.descendants_iter(root)
            .filter(|&node_id| selector::matches(&selector_list, self, node_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_dom() -> Dom {
        let mut dom = Dom::new();
        let doc = dom.document();

        // <html>
        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        dom.append_child(doc, html);

        //   <body>
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(html, body);

        //     <div id="container" class="main box">
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "container".into()),
                ("class".into(), "main box".into()),
            ],
        });
        dom.append_child(body, div);

        //       <p class="text" id="p1">Hello</p>
        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("id".into(), "p1".into()), ("class".into(), "text".into())],
        });
        dom.append_child(div, p1);

        //       <span class="text">World</span>
        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "text".into())],
        });
        dom.append_child(div, span);

        dom
    }

    #[test]
    fn test_t0024_query_selector_types() {
        let dom = setup_test_dom();

        // 1. Type selector
        let html_node = dom.query_selector("html");
        assert!(html_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(html_node.unwrap()) {
            assert_eq!(name, "html");
        } else {
            panic!("Expected html element");
        }

        // 2. ID selector
        let p1_node = dom.query_selector("#p1");
        assert!(p1_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(p1_node.unwrap()) {
            assert_eq!(name, "p");
        } else {
            panic!("Expected p element");
        }

        // 3. Class selector
        let main_node = dom.query_selector(".main");
        assert!(main_node.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(main_node.unwrap()) {
            assert_eq!(name, "div");
        } else {
            panic!("Expected div element");
        }

        // 4. Descendant selector
        let desc_p = dom.query_selector("div p");
        assert!(desc_p.is_some());
        assert_eq!(desc_p, p1_node);

        // 5. Child selector
        let child_span = dom.query_selector("div > span");
        assert!(child_span.is_some());
        if let Some(NodeData::Element { name, .. }) = dom.data(child_span.unwrap()) {
            assert_eq!(name, "span");
        } else {
            panic!("Expected span element");
        }

        // 6. Invalid selector returns None
        assert_eq!(dom.query_selector("div > > p"), None);
        assert_eq!(dom.query_selector(""), None);
    }

    #[test]
    fn test_t0024_query_selector_all_order() {
        let dom = setup_test_dom();

        // All .text elements (p then span in pre-order/document order)
        let matched = dom.query_selector_all(".text");
        assert_eq!(matched.len(), 2);

        if let Some(NodeData::Element { name, .. }) = dom.data(matched[0]) {
            assert_eq!(name, "p");
        } else {
            panic!("Expected first element to be p");
        }

        if let Some(NodeData::Element { name, .. }) = dom.data(matched[1]) {
            assert_eq!(name, "span");
        } else {
            panic!("Expected second element to be span");
        }

        // Invalid selector returns empty Vec
        assert!(dom.query_selector_all("div > > p").is_empty());
        assert!(dom.query_selector_all("").is_empty());
    }
}
