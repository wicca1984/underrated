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
                    if should_prepend_newline(dom, node, name) {
                        result.push('\n');
                    }

                    // End tag runs after the children (LIFO: push Close first).
                    stack.push(Work::Close(name.clone()));
                    push_children(dom, node, &mut stack);
                }
            }
            NodeData::Text(text) => {
                let mut is_raw = false;
                if let Some(parent_id) = dom.parent(node)
                    && let Some(NodeData::Element { name, .. }) = dom.data(parent_id)
                {
                    is_raw = is_raw_text_element(name);
                }

                if is_raw {
                    result.push_str(text);
                } else {
                    result.push_str(&escape_text(text));
                }
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

/// Returns true if the element is pre, textarea, or listing, and its first child
/// is a Text node starting with a U+000A LINE FEED character.
fn should_prepend_newline(dom: &Dom, node: NodeId, name: &str) -> bool {
    let lower_name = name.to_ascii_lowercase();
    if matches!(lower_name.as_str(), "pre" | "textarea" | "listing")
        && let Some(&first_child) = dom.children(node).first()
        && let Some(NodeData::Text(text)) = dom.data(first_child)
    {
        text.starts_with('\n')
    } else {
        false
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

/// Returns true if the element is a raw text element.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#html-fragment-serialization-algorithm
fn is_raw_text_element(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "style" | "script" | "xmp" | "iframe" | "noembed" | "noframes" | "plaintext" | "noscript"
    )
}

fn escape_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '\u{00A0}' => result.push_str("&nbsp;"),
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
            '\u{00A0}' => result.push_str("&nbsp;"),
            _ => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_raw_text_elements() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let script = dom.create_node(NodeData::Element {
            name: "script".into(),
            attrs: vec![],
        });
        dom.append_child(doc, script);
        let text = dom.create_node(NodeData::Text(
            "if (a < b && c > d) console.log('hello');".into(),
        ));
        dom.append_child(script, text);

        assert_eq!(
            dom.serialize(doc),
            "<script>if (a < b && c > d) console.log('hello');</script>"
        );
    }

    #[test]
    fn test_serialize_nbsp_escaping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![("title".into(), "hello\u{00A0}world".into())],
        });
        dom.append_child(doc, p);
        let text = dom.create_node(NodeData::Text("hello\u{00A0}world".into()));
        dom.append_child(p, text);

        assert_eq!(
            dom.serialize(doc),
            "<p title=\"hello&nbsp;world\">hello&nbsp;world</p>"
        );
    }

    #[test]
    fn test_serialize_leading_newline_pre_textarea_listing() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // 1. <pre> with leading newline
        let pre_with_lf = dom.create_node(NodeData::Element {
            name: "pre".into(),
            attrs: vec![],
        });
        dom.append_child(doc, pre_with_lf);
        let text1 = dom.create_node(NodeData::Text("\nhello".into()));
        dom.append_child(pre_with_lf, text1);

        // 2. <pre> without leading newline
        let pre_no_lf = dom.create_node(NodeData::Element {
            name: "pre".into(),
            attrs: vec![],
        });
        dom.append_child(doc, pre_no_lf);
        let text2 = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(pre_no_lf, text2);

        // 3. <textarea> with leading newline
        let textarea_with_lf = dom.create_node(NodeData::Element {
            name: "textarea".into(),
            attrs: vec![],
        });
        dom.append_child(doc, textarea_with_lf);
        let text3 = dom.create_node(NodeData::Text("\nhello".into()));
        dom.append_child(textarea_with_lf, text3);

        // 4. <textarea> without leading newline
        let textarea_no_lf = dom.create_node(NodeData::Element {
            name: "textarea".into(),
            attrs: vec![],
        });
        dom.append_child(doc, textarea_no_lf);
        let text4 = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(textarea_no_lf, text4);

        // 5. <listing> with leading newline
        let listing_with_lf = dom.create_node(NodeData::Element {
            name: "listing".into(),
            attrs: vec![],
        });
        dom.append_child(doc, listing_with_lf);
        let text5 = dom.create_node(NodeData::Text("\nhello".into()));
        dom.append_child(listing_with_lf, text5);

        // 6. <div> with leading newline (should NOT get extra LF prepended)
        let div_with_lf = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div_with_lf);
        let text6 = dom.create_node(NodeData::Text("\nhello".into()));
        dom.append_child(div_with_lf, text6);

        let serialized = dom.serialize(doc);
        assert_eq!(
            serialized,
            "<pre>\n\nhello</pre><pre>hello</pre><textarea>\n\nhello</textarea><textarea>hello</textarea><listing>\n\nhello</listing><div>\nhello</div>"
        );
    }
}
