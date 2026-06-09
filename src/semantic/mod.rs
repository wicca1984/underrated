use crate::dom::{Dom, NodeData};

/// A semantic node representing a simplified structure of a DOM node.
///
/// This is a pure derivation from the DOM, used for high-level analysis or
/// serialization to formats like Markdown.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SemanticNode {
    /// A heading with a level (1-6) and plain text content.
    Heading { level: u8, text: String },
    /// A simple paragraph of plain text.
    Paragraph(String),
    /// A hyperlink with text and a URL.
    Link { text: String, href: String },
    /// A list (ordered or unordered) containing other semantic nodes.
    List(Vec<SemanticNode>),
    /// A single item in a list, containing plain text.
    ListItem(String),
    /// A piece of plain text.
    Text(String),
    /// A generic section grouping related semantic nodes.
    Section(Vec<SemanticNode>),
}

/// A simplified, semantic view of a DOM document.
pub struct SemanticView {
    /// The top-level semantic nodes of the document.
    pub roots: Vec<SemanticNode>,
}

/// Builds a `SemanticView` from the given `Dom`.
///
/// This traverses the DOM tree and maps relevant elements to `SemanticNode`s.
/// Elements that are not explicitly recognized are recursed into, and their
/// children are processed.
pub fn build_semantic_view(dom: &Dom) -> SemanticView {
    let mut roots = Vec::new();
    let doc = dom.document();

    // Traverse children of the document root.
    for &child in dom.children(doc) {
        roots.extend(build_nodes(dom, child));
    }

    SemanticView { roots }
}

/// Recursively builds a list of semantic nodes from a DOM node.
fn build_nodes(dom: &Dom, node_id: crate::infra::NodeId) -> Vec<SemanticNode> {
    let Some(data) = dom.data(node_id) else {
        return Vec::new();
    };

    match data {
        NodeData::Element { name, attrs } => {
            match name.as_str() {
                // Headings
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let level = name[1..].parse().unwrap_or(1);
                    vec![SemanticNode::Heading {
                        level,
                        text: dom.text_content(node_id),
                    }]
                }
                // Paragraphs
                "p" => {
                    // Check if it has links. If it does, we use Section to keep them as Link nodes.
                    // Otherwise, we use Paragraph for simplicity.
                    let children_nodes = build_children(dom, node_id);
                    if has_links(&children_nodes) {
                        vec![SemanticNode::Section(children_nodes)]
                    } else {
                        vec![SemanticNode::Paragraph(dom.text_content(node_id))]
                    }
                }
                // Links
                "a" => {
                    let href = attrs
                        .iter()
                        .find(|(n, _)| n == "href")
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default();
                    vec![SemanticNode::Link {
                        text: dom.text_content(node_id),
                        href,
                    }]
                }
                // Lists
                "ul" | "ol" => {
                    let items = dom
                        .children(node_id)
                        .iter()
                        .flat_map(|&c| build_nodes(dom, c))
                        .collect();
                    vec![SemanticNode::List(items)]
                }
                // List Items
                "li" => {
                    // Similar to p, use Section if complex, otherwise ListItem.
                    let children_nodes = build_children(dom, node_id);
                    if has_links(&children_nodes) {
                        vec![SemanticNode::Section(children_nodes)]
                    } else {
                        vec![SemanticNode::ListItem(dom.text_content(node_id))]
                    }
                }
                // Sections and other block containers
                "div" | "section" | "article" | "main" | "nav" | "aside" | "header" | "footer" => {
                    vec![SemanticNode::Section(build_children(dom, node_id))]
                }
                // Inline text formatting (flatten to Text)
                "strong" | "b" | "em" | "i" | "span" => {
                    // Recurse to children to handle nested links or just text.
                    build_children(dom, node_id)
                }
                // Unknown elements: recurse into children
                _ => build_children(dom, node_id),
            }
        }
        NodeData::Text(s) => {
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![SemanticNode::Text(s.clone())]
            }
        }
        _ => Vec::new(),
    }
}

/// Helper to build semantic nodes for all children of a DOM node.
fn build_children(dom: &Dom, node_id: crate::infra::NodeId) -> Vec<SemanticNode> {
    dom.children(node_id)
        .iter()
        .flat_map(|&c| build_nodes(dom, c))
        .collect()
}

/// Helper to check if a list of semantic nodes contains any Link.
fn has_links(nodes: &[SemanticNode]) -> bool {
    nodes.iter().any(|n| match n {
        SemanticNode::Link { .. } => true,
        SemanticNode::Section(children) | SemanticNode::List(children) => has_links(children),
        _ => false,
    })
}

/// Serializes a `SemanticView` to a Markdown string.
pub fn to_markdown(view: &SemanticView) -> String {
    let mut result = String::new();
    for node in &view.roots {
        append_markdown(node, &mut result, true);
    }
    result.trim().to_string()
}

/// Recursively appends Markdown representation of a node to the result string.
fn append_markdown(node: &SemanticNode, result: &mut String, is_block: bool) {
    match node {
        SemanticNode::Heading { level, text } => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            for _ in 0..*level {
                result.push('#');
            }
            result.push(' ');
            result.push_str(text);
            result.push_str("\n\n");
        }
        SemanticNode::Paragraph(text) => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            result.push_str(text);
            result.push_str("\n\n");
        }
        SemanticNode::Link { text, href } => {
            result.push('[');
            result.push_str(text);
            result.push(']');
            result.push('(');
            result.push_str(href);
            result.push(')');
            if is_block {
                result.push_str("\n\n");
            }
        }
        SemanticNode::List(items) => {
            if !result.is_empty() && !result.ends_with("\n\n") {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
                result.push('\n');
            }
            for item in items {
                result.push_str("- ");
                append_markdown(item, result, false);
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            result.push('\n');
        }
        SemanticNode::ListItem(text) => {
            result.push_str(text);
        }
        SemanticNode::Text(text) => {
            result.push_str(text);
        }
        SemanticNode::Section(children) => {
            let mut sub = String::new();
            for child in children {
                append_markdown(child, &mut sub, false);
            }
            if !sub.is_empty() {
                if is_block && !result.is_empty() && !result.ends_with("\n\n") {
                    if !result.ends_with('\n') {
                        result.push('\n');
                    }
                    result.push('\n');
                }
                result.push_str(&sub);
                if is_block {
                    result.push_str("\n\n");
                }
            }
        }
    }
}
