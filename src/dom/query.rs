use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;
use crate::selector;

impl Dom {
    /// Returns true if `other` is an inclusive descendant of `node`
    /// (i.e. `other` is `node` itself, or a descendant of `node`).
    // spec: https://dom.spec.whatwg.org/#dom-node-contains
    pub fn contains(&self, node: NodeId, other: NodeId) -> bool {
        let mut curr = Some(other);
        while let Some(curr_node) = curr {
            if curr_node == node {
                return true;
            }
            curr = self.parent(curr_node);
        }
        false
    }

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

    /// Returns the first following sibling of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-nextelementsibling
    pub fn next_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;

        children
            .get(pos + 1..)?
            .iter()
            .copied()
            .find(|&sibling_id| matches!(self.data(sibling_id), Some(NodeData::Element { .. })))
    }

    /// Returns the nearest preceding sibling of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-nondocumenttypechildnode-previouselementsibling
    pub fn previous_element_sibling(&self, node: NodeId) -> Option<NodeId> {
        let parent = self.parent(node)?;
        let children = self.children(parent);
        let pos = children.iter().position(|&id| id == node)?;

        children
            .get(..pos)?
            .iter()
            .copied()
            .rev()
            .find(|&sibling_id| matches!(self.data(sibling_id), Some(NodeData::Element { .. })))
    }

    /// Returns the first child of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-firstelementchild
    pub fn first_element_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .find(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
    }

    /// Returns the last child of the given `node` that is an element.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-lastelementchild
    pub fn last_element_child(&self, node: NodeId) -> Option<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .rev()
            .find(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
    }

    /// Returns the number of child nodes of the given `node` that are elements.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-childelementcount
    pub fn child_element_count(&self, node: NodeId) -> usize {
        self.children(node)
            .iter()
            .filter(|&&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
            .count()
    }

    /// Returns a list of child nodes of the given `node` that are elements.
    // spec: https://dom.spec.whatwg.org/#dom-parentnode-children
    pub fn child_elements(&self, node: NodeId) -> Vec<NodeId> {
        self.children(node)
            .iter()
            .copied()
            .filter(|&child_id| matches!(self.data(child_id), Some(NodeData::Element { .. })))
            .collect()
    }

    /// Returns true if the element matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-element-matches
    pub fn matches(&self, node: NodeId, selector: &str) -> bool {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return false,
        };
        selector::matches(&selector_list, self, node)
    }

    /// Returns the closest ancestor of the given `node` (including `node` itself)
    /// that matches the given `selector`.
    // spec: https://dom.spec.whatwg.org/#dom-element-closest
    pub fn closest(&self, node: NodeId, selector: &str) -> Option<NodeId> {
        let selector_list = match selector::parse_selector_list(selector) {
            Ok(list) => list,
            Err(_) => return None,
        };
        let mut curr = Some(node);
        while let Some(curr_node) = curr {
            if selector::matches(&selector_list, self, curr_node) {
                return Some(curr_node);
            }
            curr = self.parent(curr_node);
        }
        None
    }

    /// Returns descendants of the document root that have the given HTML local `tag_name`.
    /// If `tag_name` is `*`, returns all descendant element nodes.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbytagname
    pub fn get_elements_by_tag_name(&self, tag_name: &str) -> Vec<NodeId> {
        self.get_elements_by_tag_name_from(self.document(), tag_name)
    }

    /// Returns descendants of the given `root` node that have the given HTML local `tag_name`.
    /// If `tag_name` is `*`, returns all descendant element nodes.
    // spec: https://dom.spec.whatwg.org/#dom-element-getelementsbytagname
    pub fn get_elements_by_tag_name_from(&self, root: NodeId, tag_name: &str) -> Vec<NodeId> {
        self.descendants_iter(root)
            .filter(|&node_id| {
                if let Some(NodeData::Element { name, .. }) = self.data(node_id) {
                    if tag_name == "*" {
                        true
                    } else {
                        name.eq_ignore_ascii_case(tag_name)
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns descendants of the document root that have all the given space-separated class names.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbyclassname
    pub fn get_elements_by_class_name(&self, class_name: &str) -> Vec<NodeId> {
        self.get_elements_by_class_name_from(self.document(), class_name)
    }

    /// Returns descendants of the given `root` node that have all the given space-separated class names.
    // spec: https://dom.spec.whatwg.org/#dom-element-getelementsbyclassname
    pub fn get_elements_by_class_name_from(&self, root: NodeId, class_name: &str) -> Vec<NodeId> {
        let targets: Vec<&str> = class_name.split_ascii_whitespace().collect();
        if targets.is_empty() {
            return Vec::new();
        }

        self.descendants_iter(root)
            .filter(|&node_id| {
                if matches!(self.data(node_id), Some(NodeData::Element { .. })) {
                    targets
                        .iter()
                        .all(|&target| self.has_class(node_id, target))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Returns descendants of the document root that have a `name` attribute equal to the given value.
    // spec: https://dom.spec.whatwg.org/#dom-document-getelementsbyname
    pub fn get_elements_by_name(&self, name: &str) -> Vec<NodeId> {
        self.get_elements_by_name_from(self.document(), name)
    }

    /// Returns descendants of the given `root` node that have a `name` attribute equal to the given value.
    pub fn get_elements_by_name_from(&self, root: NodeId, name: &str) -> Vec<NodeId> {
        self.descendants_iter(root)
            .filter(|&node_id| {
                if let Some(NodeData::Element { attrs, .. }) = self.data(node_id) {
                    attrs.iter().any(|(n, v)| n == "name" && v == name)
                } else {
                    false
                }
            })
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

    #[test]
    fn test_element_sibling_navigation() {
        let mut dom = Dom::new();
        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });

        let child_a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        let child_text = dom.create_node(NodeData::Text("some text".into()));
        let child_b = dom.create_node(NodeData::Element {
            name: "b".into(),
            attrs: vec![],
        });
        let child_c = dom.create_node(NodeData::Element {
            name: "c".into(),
            attrs: vec![],
        });

        dom.append_child(parent, child_a);
        dom.append_child(parent, child_text);
        dom.append_child(parent, child_b);
        dom.append_child(parent, child_c);

        // Next element sibling
        assert_eq!(dom.next_element_sibling(child_a), Some(child_b));
        assert_eq!(dom.next_element_sibling(child_b), Some(child_c));
        assert_eq!(dom.next_element_sibling(child_c), None);

        // Previous element sibling
        assert_eq!(dom.previous_element_sibling(child_c), Some(child_b));
        assert_eq!(dom.previous_element_sibling(child_b), Some(child_a));
        assert_eq!(dom.previous_element_sibling(child_a), None);

        // Unattached/no parent node
        let unattached = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![],
        });
        assert_eq!(dom.next_element_sibling(unattached), None);
        assert_eq!(dom.previous_element_sibling(unattached), None);
    }

    #[test]
    fn test_node_contains_query() {
        let mut dom = setup_test_dom();
        let doc = dom.document();
        let html = dom.query_selector("html").unwrap();
        let body = dom.query_selector("body").unwrap();
        let div = dom.query_selector("#container").unwrap();
        let p1 = dom.query_selector("#p1").unwrap();
        let span = dom.query_selector("span").unwrap();

        // 1. A node contains itself (contains(n, n) == true).
        assert!(dom.contains(doc, doc));
        assert!(dom.contains(html, html));
        assert!(dom.contains(p1, p1));

        // 2. A parent contains its direct child.
        assert!(dom.contains(html, body));
        assert!(dom.contains(div, p1));
        assert!(dom.contains(div, span));

        // 3. An ancestor contains a deep (grand+) descendant.
        assert!(dom.contains(html, p1));
        assert!(dom.contains(doc, span));

        // 4. A node does NOT contain its own ancestor (contains(child, parent) == false).
        assert!(!dom.contains(body, html));
        assert!(!dom.contains(p1, div));
        assert!(!dom.contains(span, doc));

        // 5. Two sibling subtrees do not contain each other.
        assert!(!dom.contains(p1, span));
        assert!(!dom.contains(span, p1));

        // 6. The document root contains every node in the tree.
        assert!(dom.contains(doc, doc));
        assert!(dom.contains(doc, html));
        assert!(dom.contains(doc, body));
        assert!(dom.contains(doc, div));
        assert!(dom.contains(doc, p1));
        assert!(dom.contains(doc, span));

        // Extra: Unattached node in the same DOM
        let unattached = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![],
        });
        assert!(dom.contains(unattached, unattached));
        assert!(!dom.contains(doc, unattached));
        assert!(!dom.contains(unattached, doc));
    }

    #[test]
    fn test_expanded_query_and_traversal() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Let's build:
        // <html>
        //   <body>
        //     <div id="main" class="container content" name="app-root">
        //       text node child
        //       <p class="text item" id="p1" name="paragraph">First paragraph</p>
        //       comment node child
        //       <span class="text label" id="s1">Some span</span>
        //       <div class="footer item" name="paragraph">Footer div</div>
        //     </div>
        //   </body>
        // </html>

        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        dom.append_child(doc, html);

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(html, body);

        let main_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("id".into(), "main".into()),
                ("class".into(), "container content".into()),
                ("name".into(), "app-root".into()),
            ],
        });
        dom.append_child(body, main_div);

        let text_node = dom.create_node(NodeData::Text("   some text   ".into()));
        dom.append_child(main_div, text_node);

        let p1 = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![
                ("id".into(), "p1".into()),
                ("class".into(), "text item".into()),
                ("name".into(), "paragraph".into()),
            ],
        });
        dom.append_child(main_div, p1);

        let comment_node = dom.create_node(NodeData::Comment("this is a comment".into()));
        dom.append_child(main_div, comment_node);

        let span1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![
                ("id".into(), "s1".into()),
                ("class".into(), "text label".into()),
            ],
        });
        dom.append_child(main_div, span1);

        let footer_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![
                ("class".into(), "footer item".into()),
                ("name".into(), "paragraph".into()),
            ],
        });
        dom.append_child(main_div, footer_div);

        // --- 1. first_element_child, last_element_child, child_element_count, child_elements ---
        assert_eq!(dom.first_element_child(main_div), Some(p1));
        assert_eq!(dom.last_element_child(main_div), Some(footer_div));
        assert_eq!(dom.child_element_count(main_div), 3);
        assert_eq!(dom.child_elements(main_div), vec![p1, span1, footer_div]);

        // HTML/body check
        assert_eq!(dom.first_element_child(html), Some(body));
        assert_eq!(dom.last_element_child(html), Some(body));
        assert_eq!(dom.child_element_count(html), 1);
        assert_eq!(dom.child_elements(html), vec![body]);

        // No child element checks
        assert_eq!(dom.first_element_child(p1), None);
        assert_eq!(dom.last_element_child(p1), None);
        assert_eq!(dom.child_element_count(p1), 0);
        assert!(dom.child_elements(p1).is_empty());

        // --- 2. matches ---
        assert!(dom.matches(p1, "p"));
        assert!(dom.matches(p1, ".text"));
        assert!(dom.matches(p1, ".item"));
        assert!(dom.matches(p1, "p.text.item"));
        assert!(dom.matches(p1, "div p"));
        assert!(dom.matches(p1, "#p1"));
        assert!(!dom.matches(p1, "span"));
        assert!(!dom.matches(p1, "div"));
        assert!(!dom.matches(text_node, "p")); // Non-element node

        // --- 3. closest ---
        assert_eq!(dom.closest(p1, "p"), Some(p1));
        assert_eq!(dom.closest(p1, "div"), Some(main_div));
        assert_eq!(dom.closest(p1, "body"), Some(body));
        assert_eq!(dom.closest(p1, "html"), Some(html));
        assert_eq!(dom.closest(p1, "span"), None);
        assert_eq!(dom.closest(text_node, "div"), Some(main_div)); // Text node matches parent

        // --- 4. get_elements_by_tag_name ---
        // Document-wide tag matching
        let divs = dom.get_elements_by_tag_name("div");
        assert_eq!(divs, vec![main_div, footer_div]);

        let ps = dom.get_elements_by_tag_name("p");
        assert_eq!(ps, vec![p1]);

        let stars = dom.get_elements_by_tag_name("*");
        assert_eq!(stars, vec![html, body, main_div, p1, span1, footer_div]);

        // Case-insensitivity
        let upper_divs = dom.get_elements_by_tag_name("DIV");
        assert_eq!(upper_divs, vec![main_div, footer_div]);

        // Root-specific tag matching
        let sub_divs = dom.get_elements_by_tag_name_from(main_div, "div");
        assert_eq!(sub_divs, vec![footer_div]);

        let sub_stars = dom.get_elements_by_tag_name_from(main_div, "*");
        assert_eq!(sub_stars, vec![p1, span1, footer_div]);

        // --- 5. get_elements_by_class_name ---
        // Document-wide class matching
        let texts = dom.get_elements_by_class_name("text");
        assert_eq!(texts, vec![p1, span1]);

        let items = dom.get_elements_by_class_name("item");
        assert_eq!(items, vec![p1, footer_div]);

        // Multiple class matching (any order)
        let multiple1 = dom.get_elements_by_class_name("text item");
        assert_eq!(multiple1, vec![p1]);

        let multiple2 = dom.get_elements_by_class_name("item text");
        assert_eq!(multiple2, vec![p1]);

        // Empty class names or non-existent
        assert!(dom.get_elements_by_class_name("").is_empty());
        assert!(dom.get_elements_by_class_name("nonexistent").is_empty());

        // Root-specific class matching
        let sub_items = dom.get_elements_by_class_name_from(main_div, "item");
        assert_eq!(sub_items, vec![p1, footer_div]);

        // --- 6. get_elements_by_name ---
        let app_roots = dom.get_elements_by_name("app-root");
        assert_eq!(app_roots, vec![main_div]);

        let paragraphs = dom.get_elements_by_name("paragraph");
        assert_eq!(paragraphs, vec![p1, footer_div]);

        let sub_paragraphs = dom.get_elements_by_name_from(main_div, "paragraph");
        assert_eq!(sub_paragraphs, vec![p1, footer_div]);

        let nonexistent_name = dom.get_elements_by_name("nonexistent");
        assert!(nonexistent_name.is_empty());
    }
}
