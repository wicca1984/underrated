use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;

/// A unit of serialization work. `Close` emits an element's end tag after all
/// of its children have been serialized.
enum Work {
    Open(NodeId),
    Close(String),
}

/// Serializes the given node and its descendants into an HTML string.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#serializing-html-fragments
pub fn serialize(dom: &Dom, node: NodeId) -> String {
    let mut result = String::new();

    // Iterative DFS with an explicit work stack so deeply nested (or maliciously
    // crafted) trees cannot overflow the call stack (I-6).
    let mut stack = vec![Work::Open(node)];
    while let Some(work) = stack.pop() {
        let node = match work {
            Work::Close(name) => {
                result.push_str("</");
                result.push_str(&name);
                result.push('>');
                continue;
            }
            Work::Open(node) => node,
        };

        let Some(data) = dom.data(node) else {
            continue;
        };

        match data {
            NodeData::Document => {
                push_children(dom, node, &mut stack);
            }
            NodeData::Doctype { name, .. } => {
                result.push_str("<!DOCTYPE ");
                result.push_str(name);
                result.push('>');
            }
            NodeData::Element { name, attrs } => {
                result.push('<');
                result.push_str(name);
                for (attr_name, attr_value) in attrs {
                    result.push(' ');
                    result.push_str(attr_name);
                    result.push_str("=\"");
                    result.push_str(&escape_attribute(attr_value));
                    result.push('"');
                }
                result.push('>');

                if !is_void_element(name) {
                    // End tag runs after the children (LIFO: push Close first).
                    stack.push(Work::Close(name.clone()));
                    push_children(dom, node, &mut stack);
                }
            }
            NodeData::Text(text) => {
                result.push_str(&escape_text(text));
            }
            NodeData::Comment(comment) => {
                result.push_str("<!--");
                result.push_str(comment);
                result.push_str("-->");
            }
        }
    }

    result
}

/// Pushes a node's children onto the work stack in reverse so they are
/// serialized in document order (LIFO).
fn push_children(dom: &Dom, node: NodeId, stack: &mut Vec<Work>) {
    for &child in dom.children(node).iter().rev() {
        stack.push(Work::Open(child));
    }
}

/// Returns true if the element is a void element.
// spec: https://html.spec.whatwg.org/multipage/syntax.html#void-elements
fn is_void_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "source"
            | "track"
            | "wbr"
    )
}

fn escape_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(c),
        }
    }
    result
}

fn escape_attribute(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            _ => result.push(c),
        }
    }
    result
}
