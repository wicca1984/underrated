#![allow(dead_code)]

mod serialize;

use crate::infra::{Arena, NodeId};

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

/// Internal node structure to be stored in the arena.
struct Node {
    data: NodeData,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
}

/// The DOM tree, managing nodes in an arena.
pub struct Dom {
    arena: Arena<Node>,
    document: NodeId,
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
        });
        Self { arena, document }
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

    /// Returns all descendants of the given node in pre-order.
    /// The node itself is excluded.
    // spec: https://dom.spec.whatwg.org/#concept-tree-descendant
    pub fn descendants(&self, node: NodeId) -> Vec<NodeId> {
        // Iterative pre-order traversal with an explicit stack so that deeply
        // nested (or maliciously crafted) trees cannot overflow the call stack (I-6).
        let mut result = Vec::new();
        let mut stack: Vec<NodeId> = self.children(node).iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            result.push(n);
            stack.extend(self.children(n).iter().rev().copied());
        }
        result
    }

    /// Serializes the given node and its descendants into an HTML string.
    pub fn serialize(&self, node: NodeId) -> String {
        serialize::serialize(self, node)
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
}
