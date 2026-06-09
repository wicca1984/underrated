use crate::dom::{Dom, NodeData};
use crate::infra::NodeId;

/// Serializes the given node and its descendants into an HTML string.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#serializing-html-fragments
pub fn serialize(dom: &Dom, node: NodeId) -> String {
    let mut result = String::new();
    serialize_recursive(dom, node, &mut result);
    result
}

fn serialize_recursive(dom: &Dom, node: NodeId, result: &mut String) {
    let Some(data) = dom.data(node) else {
        return;
    };

    match data {
        NodeData::Document => {
            for &child in dom.children(node) {
                serialize_recursive(dom, child, result);
            }
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
                for &child in dom.children(node) {
                    serialize_recursive(dom, child, result);
                }
                result.push_str("</");
                result.push_str(name);
                result.push('>');
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
