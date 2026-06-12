#![allow(dead_code)]

mod classlist;
mod dirty;
mod focus;
mod mutate;
mod query;
mod rect;
mod serialize;
mod text;

pub use rect::DomRect;

use crate::infra::{Arena, NodeId};

// TODO(spec): loading=lazy currently behaves as eager (no viewport-proximity deferral); see src/loader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLoading {
    Eager,
    Lazy,
}

fn parse_loading(value: Option<&str>) -> ImageLoading {
    match value {
        Some(v) => {
            if v.trim().eq_ignore_ascii_case("lazy") {
                ImageLoading::Lazy
            } else {
                ImageLoading::Eager
            }
        }
        None => ImageLoading::Eager,
    }
}

/// Node data for different types of DOM nodes.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeData {
    Document,
    Doctype {
        name: String,
        public_id: String,
        system_id: String,
    },
    Element {
        name: String,                 // TODO(spec): use infra::Symbol
        attrs: Vec<(String, String)>, // TODO(spec): use infra::Symbol for names
    },
    Text(String),
    Comment(String),
}

impl NodeData {
    /// Returns the parsed value of the `loading` attribute.
    pub fn loading(&self) -> ImageLoading {
        let value = match self {
            NodeData::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == "loading")
                .map(|(_, v)| v.as_str()),
            _ => None,
        };
        parse_loading(value)
    }

    /// Returns the value of the `role` attribute if present.
    pub fn role(&self) -> Option<&str> {
        match self {
            NodeData::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == "role")
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Returns the value of attribute `aria-{name}` (e.g. `el.aria("label")` reads `aria-label`).
    pub fn aria(&self, name: &str) -> Option<&str> {
        let key = format!("aria-{name}");
        match self {
            NodeData::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == &key)
                .map(|(_, v)| v.as_str()),
            _ => None,
        }
    }

    /// Returns the parsed value of the `tabindex` attribute if present.
    pub fn tabindex(&self) -> Option<i32> {
        match self {
            NodeData::Element { attrs, .. } => attrs
                .iter()
                .find(|(k, _)| k == "tabindex")
                .and_then(|(_, v)| v.trim().parse::<i32>().ok()),
            _ => None,
        }
    }

    /// Returns true if the `hidden` attribute is present.
    pub fn hidden(&self) -> bool {
        match self {
            NodeData::Element { attrs, .. } => attrs.iter().any(|(k, _)| k == "hidden"),
            _ => false,
        }
    }
}

/// Internal node structure to be stored in the arena.
struct Node {
    data: NodeData,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    input_value: Option<String>,
    input_value_dirty: bool,
}

/// The DOM tree, managing nodes in an arena.
pub struct Dom {
    arena: Arena<Node>,
    document: NodeId,
    focused_node: std::cell::Cell<Option<NodeId>>,
    images: std::cell::RefCell<std::collections::HashMap<String, crate::image::DecodedImage>>,
    dirty_nodes: Vec<NodeId>,
}

impl Default for Dom {
    fn default() -> Self {
        Self::new()
    }
}

impl Dom {
    /// Creates a new DOM with a Document root.
    // spec: https://dom.spec.whatwg.org/#dom-document
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let document = arena.insert(Node {
            data: NodeData::Document,
            parent: None,
            children: Vec::new(),
            input_value: None,
            input_value_dirty: false,
        });
        Self {
            arena,
            document,
            focused_node: std::cell::Cell::new(None),
            images: std::cell::RefCell::new(std::collections::HashMap::new()),
            dirty_nodes: Vec::new(),
        }
    }

    /// Adds a decoded image to the DOM's image cache.
    pub fn add_image(&self, src: String, img: crate::image::DecodedImage) {
        self.images.borrow_mut().insert(src, img);
    }

    /// Retrieves a decoded image from the DOM's image cache.
    pub fn get_image(&self, src: &str) -> Option<crate::image::DecodedImage> {
        self.images.borrow().get(src).cloned()
    }

    /// Returns the NodeId of the Document root.
    pub fn document(&self) -> NodeId {
        self.document
    }

    /// Creates a new node with the given data.
    pub fn create_node(&mut self, data: NodeData) -> NodeId {
        self.arena.insert(Node {
            data,
            parent: None,
            children: Vec::new(),
            input_value: None,
            input_value_dirty: false,
        })
    }

    /// Appends a child node to a parent node.
    // spec: https://dom.spec.whatwg.org/#dom-node-appendchild
    pub fn append_child(&mut self, parent: NodeId, child: NodeId) {
        // Remove from old parent if exists to maintain consistency.
        if let Some(old_parent_node) = self
            .parent(child)
            .and_then(|parent_id| self.arena.get_mut(parent_id))
        {
            old_parent_node.children.retain(|&id| id != child);
        }

        if let Some(parent_node) = self.arena.get_mut(parent) {
            parent_node.children.push(child);
        }
        if let Some(child_node) = self.arena.get_mut(child) {
            child_node.parent = Some(parent);
        }
    }

    /// Returns the parent of the given node, if any.
    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.arena.get(node).and_then(|n| n.parent)
    }

    /// Returns the children of the given node.
    pub fn children(&self, node: NodeId) -> &[NodeId] {
        self.arena
            .get(node)
            .map(|n| n.children.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the data of the given node, or `None` if the `NodeId` is
    /// invalid or stale (mirrors the generational arena's `get`).
    pub fn data(&self, node: NodeId) -> Option<&NodeData> {
        self.arena.get(node).map(|n| &n.data)
    }

    /// Returns the parsed value of the `loading` attribute on the given node.
    pub fn loading(&self, node: NodeId) -> ImageLoading {
        parse_loading(self.get_attribute(node, "loading"))
    }

    /// Returns the value of the `role` attribute if present on the given node.
    pub fn role(&self, node: NodeId) -> Option<&str> {
        self.get_attribute(node, "role")
    }

    /// Returns the value of the `aria-{name}` attribute if present on the given node.
    pub fn aria(&self, node: NodeId, name: &str) -> Option<&str> {
        self.get_attribute(node, &format!("aria-{name}"))
    }

    /// Returns the parsed value of the `tabindex` attribute if present on the given node.
    pub fn tabindex(&self, node: NodeId) -> Option<i32> {
        self.get_attribute(node, "tabindex")
            .and_then(|v| v.trim().parse::<i32>().ok())
    }

    /// Returns true if the `hidden` attribute is present on the given node.
    pub fn hidden(&self, node: NodeId) -> bool {
        self.get_attribute(node, "hidden").is_some()
    }

    /// Returns an iterator over all descendants of the given node in pre-order.
    /// The node itself is excluded.
    // spec: https://dom.spec.whatwg.org/#concept-tree-descendant
    pub fn descendants_iter(&self, node: NodeId) -> DescendantsIter<'_> {
        DescendantsIter {
            dom: self,
            stack: self.children(node).iter().rev().copied().collect(),
        }
    }

    /// Returns all descendants of the given node in pre-order.
    /// The node itself is excluded.
    // spec: https://dom.spec.whatwg.org/#concept-tree-descendant
    pub fn descendants(&self, node: NodeId) -> Vec<NodeId> {
        self.descendants_iter(node).collect()
    }

    /// Serializes the given node and its descendants into an HTML string.
    pub fn serialize(&self, node: NodeId) -> String {
        serialize::serialize(self, node)
    }
}

/// An iterator over the descendants of a DOM node in pre-order (document order).
pub struct DescendantsIter<'a> {
    dom: &'a Dom,
    stack: Vec<NodeId>,
}

impl<'a> Iterator for DescendantsIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = self.stack.pop() {
            self.stack
                .extend(self.dom.children(n).iter().rev().copied());
            Some(n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_new() {
        let dom = Dom::new();
        let doc_id = dom.document();
        match dom.data(doc_id) {
            Some(NodeData::Document) => {}
            _ => panic!("Root should be a Document"),
        }
    }

    #[test]
    fn test_create_node() {
        let mut dom = Dom::new();
        let data = NodeData::Element {
            name: "div".to_string(),
            attrs: vec![],
        };
        let node_id = dom.create_node(data.clone());
        assert_eq!(dom.data(node_id), Some(&data));
    }

    #[test]
    fn test_doctype_node() {
        let mut dom = Dom::new();
        let data = NodeData::Doctype {
            name: "html".to_string(),
            public_id: "-//W3C//DTD XHTML 1.0 Strict//EN".to_string(),
            system_id: "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd".to_string(),
        };
        let node_id = dom.create_node(data.clone());
        assert_eq!(dom.data(node_id), Some(&data));
    }

    #[test]
    fn test_append_child() {
        let mut dom = Dom::new();
        let doc_id = dom.document();
        let child_id = dom.create_node(NodeData::Element {
            name: "html".to_string(),
            attrs: vec![],
        });

        dom.append_child(doc_id, child_id);

        assert_eq!(dom.parent(child_id), Some(doc_id));
        assert_eq!(dom.children(doc_id), &[child_id]);
    }

    #[test]
    fn test_reparenting() {
        let mut dom = Dom::new();
        let p1 = dom.create_node(NodeData::Element {
            name: "p1".to_string(),
            attrs: vec![],
        });
        let p2 = dom.create_node(NodeData::Element {
            name: "p2".to_string(),
            attrs: vec![],
        });
        let child = dom.create_node(NodeData::Text("child".to_string()));

        dom.append_child(p1, child);
        assert_eq!(dom.parent(child), Some(p1));
        assert_eq!(dom.children(p1), &[child]);

        dom.append_child(p2, child);
        assert_eq!(dom.parent(child), Some(p2));
        assert_eq!(dom.children(p2), &[child]);
        assert_eq!(dom.children(p1), &[] as &[NodeId]);
    }

    #[test]
    fn test_sibling_order() {
        let mut dom = Dom::new();
        let doc_id = dom.document();
        let c1 = dom.create_node(NodeData::Comment("1".to_string()));
        let c2 = dom.create_node(NodeData::Comment("2".to_string()));

        dom.append_child(doc_id, c1);
        dom.append_child(doc_id, c2);

        assert_eq!(dom.children(doc_id), &[c1, c2]);
    }

    #[test]
    fn test_foreign_id_returns_none() {
        let mut dom1 = Dom::new();
        let dom2 = Dom::new();

        let mut last_id = dom1.document();
        for i in 0..100 {
            last_id = dom1.create_node(NodeData::Text(i.to_string()));
        }

        // last_id is from dom1; querying dom2 must not panic and must return None.
        assert_eq!(dom2.data(last_id), None);
    }

    #[test]
    fn test_descendants() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });

        dom.append_child(doc, html);
        dom.append_child(html, body);
        dom.append_child(body, p);

        let desc = dom.descendants(html);
        assert_eq!(desc, vec![body, p]);

        let doc_desc = dom.descendants(doc);
        assert_eq!(doc_desc, vec![html, body, p]);
    }

    #[test]
    fn test_serialize_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
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

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("class".into(), "test".into())],
        });
        dom.append_child(body, p);

        let text = dom.create_node(NodeData::Text("hi".into()));
        dom.append_child(p, text);

        assert_eq!(
            dom.serialize(doc),
            "<html><body><p class=\"test\">hi</p></body></html>"
        );
    }

    #[test]
    fn test_serialize_void_element() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let br = dom.create_node(NodeData::Element {
            name: "br".into(),
            attrs: vec![],
        });
        dom.append_child(doc, br);
        assert_eq!(dom.serialize(doc), "<br>");

        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), "a.png".into())],
        });
        dom.append_child(doc, img);
        assert_eq!(dom.serialize(doc), "<br><img src=\"a.png\">");
    }

    #[test]
    fn test_serialize_escaping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("title".into(), "a \"quoted\" & b".into())],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("1 < 2 & 3 > 0".into()));
        dom.append_child(p, text);

        assert_eq!(
            dom.serialize(doc),
            "<p title=\"a &quot;quoted&quot; &amp; b\">1 &lt; 2 &amp; 3 &gt; 0</p>"
        );
    }

    #[test]
    fn test_serialize_comment_and_doctype() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let doctype = dom.create_node(NodeData::Doctype {
            name: "html".into(),
            public_id: "".into(),
            system_id: "".into(),
        });
        dom.append_child(doc, doctype);

        let comment = dom.create_node(NodeData::Comment("secret".into()));
        dom.append_child(doc, comment);

        let html = dom.create_node(NodeData::Element {
            name: "html".into(),
            attrs: vec![],
        });
        dom.append_child(doc, html);

        assert_eq!(
            dom.serialize(doc),
            "<!DOCTYPE html><!--secret--><html></html>"
        );
    }

    #[test]
    fn test_role_attribute_retained() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<div role="button">x</div>"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        // Find the div node
        let div_id = dom.query_selector("div").expect("Should find div");

        // Verify role via Dom accessor
        assert_eq!(dom.role(div_id), Some("button"));

        // Verify role via NodeData accessor
        let node_data = dom.data(div_id).expect("Should have data");
        assert_eq!(node_data.role(), Some("button"));
    }

    #[test]
    fn test_aria_attribute_retained() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<div aria-label="Close" aria-hidden="true">x</div>"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        // Find the div node
        let div_id = dom.query_selector("div").expect("Should find div");

        // Verify aria via Dom accessor
        assert_eq!(dom.aria(div_id, "label"), Some("Close"));
        assert_eq!(dom.aria(div_id, "hidden"), Some("true"));

        // Verify aria via NodeData accessor
        let node_data = dom.data(div_id).expect("Should have data");
        assert_eq!(node_data.aria("label"), Some("Close"));
        assert_eq!(node_data.aria("hidden"), Some("true"));
    }

    #[test]
    fn test_role_absent() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = "<div>x</div>";
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        // Find the div node
        let div_id = dom.query_selector("div").expect("Should find div");

        // Verify role and aria are None via Dom accessor
        assert_eq!(dom.role(div_id), None);
        assert_eq!(dom.aria(div_id, "label"), None);

        // Verify role and aria are None via NodeData accessor
        let node_data = dom.data(div_id).expect("Should have data");
        assert_eq!(node_data.role(), None);
        assert_eq!(node_data.aria("label"), None);
    }

    #[test]
    fn test_loading_lazy_retained() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<img loading="lazy">"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        let img_id = dom.query_selector("img").expect("Should find img");

        assert_eq!(dom.loading(img_id), ImageLoading::Lazy);

        let node_data = dom.data(img_id).expect("Should have data");
        assert_eq!(node_data.loading(), ImageLoading::Lazy);

        // Test with whitespace to verify trim
        let html_trimmed = r#"<img loading=" lazy ">"#;
        let stream_trimmed = InputStream::from_utf8(html_trimmed.as_bytes());
        let dom_trimmed = parse_document(stream_trimmed);
        let img_id_trimmed = dom_trimmed.query_selector("img").expect("Should find img");
        assert_eq!(dom_trimmed.loading(img_id_trimmed), ImageLoading::Lazy);
    }

    #[test]
    fn test_loading_eager_explicit() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<img loading="eager">"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        let img_id = dom.query_selector("img").expect("Should find img");

        assert_eq!(dom.loading(img_id), ImageLoading::Eager);

        let node_data = dom.data(img_id).expect("Should have data");
        assert_eq!(node_data.loading(), ImageLoading::Eager);
    }

    #[test]
    fn test_loading_default_eager() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<img>"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        let img_id = dom.query_selector("img").expect("Should find img");

        assert_eq!(dom.loading(img_id), ImageLoading::Eager);

        let node_data = dom.data(img_id).expect("Should have data");
        assert_eq!(node_data.loading(), ImageLoading::Eager);
    }

    #[test]
    fn test_loading_invalid_is_eager() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<img loading="garbage">"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        let img_id = dom.query_selector("img").expect("Should find img");

        assert_eq!(dom.loading(img_id), ImageLoading::Eager);

        let node_data = dom.data(img_id).expect("Should have data");
        assert_eq!(node_data.loading(), ImageLoading::Eager);
    }

    #[test]
    fn test_loading_case_insensitive() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        let html = r#"<img loading="LAZY">"#;
        let stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(stream);

        let img_id = dom.query_selector("img").expect("Should find img");

        assert_eq!(dom.loading(img_id), ImageLoading::Lazy);

        let node_data = dom.data(img_id).expect("Should have data");
        assert_eq!(node_data.loading(), ImageLoading::Lazy);
    }

    #[test]
    fn test_tabindex_attribute_accessor() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        // 1. Positive value
        {
            let html = r#"<a tabindex="3">x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("a").expect("Should find a");
            assert_eq!(dom.tabindex(id), Some(3));
            assert_eq!(dom.data(id).and_then(|n| n.tabindex()), Some(3));
        }

        // 2. Negative value
        {
            let html = r#"<a tabindex="-1">x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("a").expect("Should find a");
            assert_eq!(dom.tabindex(id), Some(-1));
            assert_eq!(dom.data(id).and_then(|n| n.tabindex()), Some(-1));
        }

        // 3. Surrounding whitespace
        {
            let html = r#"<a tabindex="  5 ">x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("a").expect("Should find a");
            assert_eq!(dom.tabindex(id), Some(5));
            assert_eq!(dom.data(id).and_then(|n| n.tabindex()), Some(5));
        }

        // 4. Invalid integer
        {
            let html = r#"<a tabindex="abc">x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("a").expect("Should find a");
            assert_eq!(dom.tabindex(id), None);
            assert_eq!(dom.data(id).and_then(|n| n.tabindex()), None);
        }

        // 5. No attribute
        {
            let html = r#"<a>x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("a").expect("Should find a");
            assert_eq!(dom.tabindex(id), None);
            assert_eq!(dom.data(id).and_then(|n| n.tabindex()), None);
        }

        // 6. Non-element node (Document, Text)
        {
            let html = r#"<a tabindex="3">x</a>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);

            // Document node
            let doc_id = dom.document();
            assert_eq!(dom.tabindex(doc_id), None);
            assert_eq!(dom.data(doc_id).and_then(|n| n.tabindex()), None);

            // Text node
            let children = dom.children(dom.query_selector("a").unwrap());
            let text_id = children[0];
            assert_eq!(dom.tabindex(text_id), None);
            assert_eq!(dom.data(text_id).and_then(|n| n.tabindex()), None);
        }
    }

    #[test]
    fn test_hidden_attribute_accessor() {
        use crate::encoding::InputStream;
        use crate::html::parse_document;

        // 1. Plain hidden attribute
        {
            let html = r#"<div hidden>x</div>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("div").expect("Should find div");
            assert!(dom.hidden(id));
            assert!(dom.data(id).is_some_and(|n| n.hidden()));
        }

        // 2. Empty string value
        {
            let html = r#"<div hidden="">x</div>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("div").expect("Should find div");
            assert!(dom.hidden(id));
            assert!(dom.data(id).is_some_and(|n| n.hidden()));
        }

        // 3. String value like "false" (still true because attribute is present)
        {
            let html = r#"<div hidden="false">x</div>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("div").expect("Should find div");
            assert!(dom.hidden(id));
            assert!(dom.data(id).is_some_and(|n| n.hidden()));
        }

        // 4. No attribute
        {
            let html = r#"<div>x</div>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);
            let id = dom.query_selector("div").expect("Should find div");
            assert!(!dom.hidden(id));
            assert!(!dom.data(id).is_some_and(|n| n.hidden()));
        }

        // 5. Non-element node (Document, Text)
        {
            let html = r#"<div hidden>x</div>"#;
            let stream = InputStream::from_utf8(html.as_bytes());
            let dom = parse_document(stream);

            // Document node
            let doc_id = dom.document();
            assert!(!dom.hidden(doc_id));
            assert!(!dom.data(doc_id).is_some_and(|n| n.hidden()));

            // Text node
            let children = dom.children(dom.query_selector("div").unwrap());
            let text_id = children[0];
            assert!(!dom.hidden(text_id));
            assert!(!dom.data(text_id).is_some_and(|n| n.hidden()));
        }
    }
}
