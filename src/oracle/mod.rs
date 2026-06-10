//! Oracle differential-testing snapshot exporter module.
//!
//! This module implements the engine-side exporter matching the Playwright oracle schema
//! defined in `/tools/oracle/extract.mjs`.

use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use std::collections::HashMap;

/// Helper function to recursively collect all layout rects associated with each NodeId.
///
/// // TODO(spec): Anonymous boxes created by the layout engine (e.g., line boxes)
/// do not correspond to any DOM node and are naturally bypassed by traversing the DOM tree.
fn collect_layout_rects(layout_box: &LayoutBox, map: &mut HashMap<NodeId, Vec<Rect>>) {
    if let Some(node_id) = layout_box.node {
        map.entry(node_id).or_default().push(layout_box.rect);
    }
    for child in &layout_box.children {
        collect_layout_rects(child, map);
    }
}

/// Computes the union of a slice of rectangles.
/// If empty, returns a zero-sized rectangle at the origin.
fn union_rects(rects: &[Rect]) -> Rect {
    if rects.is_empty() {
        return Rect::new(0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for r in rects {
        min_x = min_x.min(r.origin.x);
        min_y = min_y.min(r.origin.y);
        max_x = max_x.max(r.origin.x + r.size.width);
        max_y = max_y.max(r.origin.y + r.size.height);
    }
    Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

/// Recursively computes the border-box bounding rect of a DOM node.
/// If the node has layout boxes directly associated with it (e.g. block containers, text fragments),
/// we return the union of those boxes.
/// If it does not (e.g. inline elements like `<a>` which are flattened during line box layout),
/// we compute its rect as the union of its DOM children's rects.
///
/// // TODO(spec): Inline level elements like `display: inline` do not have direct layout boxes.
/// Bounding client rect calculations for inline elements fall back to the bounding union of their children.
fn get_node_rect(dom: &Dom, node_id: NodeId, layout_map: &HashMap<NodeId, Vec<Rect>>) -> Rect {
    if let Some(rects) = layout_map.get(&node_id).filter(|r| !r.is_empty()) {
        return union_rects(rects);
    }

    let mut child_rects = Vec::new();
    for &child_id in dom.children(node_id) {
        let r = get_node_rect(dom, child_id, layout_map);
        if r.size.width > 0.0 || r.size.height > 0.0 {
            child_rects.push(r);
        }
    }

    if child_rects.is_empty() {
        Rect::new(0.0, 0.0, 0.0, 0.0)
    } else {
        union_rects(&child_rects)
    }
}

/// Finds the root element (`documentElement`) of the DOM, usually `<html>`.
/// This is the first Element child of the Document node.
fn find_document_element(dom: &Dom) -> Option<NodeId> {
    dom.children(dom.document())
        .iter()
        .copied()
        .find(|&node_id| matches!(dom.data(node_id), Some(NodeData::Element { .. })))
}

/// Recursively serializes a DOM node and its descendants into the Playwright oracle snapshot format.
/// Returns `None` if the node should be skipped (e.g. whitespace-only text, comments, doctypes).
///
/// // TODO(spec): Text nodes in the layout tree have collapsed whitespaces, but the DOM tree
/// contains the original raw text content. We serialize the raw DOM text content to match
/// the behavior of Playwright's `textContent`.
fn serialize_node(
    dom: &Dom,
    node_id: NodeId,
    layout_map: &HashMap<NodeId, Vec<Rect>>,
) -> Option<serde_json::Value> {
    match dom.data(node_id)? {
        NodeData::Document => None,
        NodeData::Doctype { .. } => None,
        NodeData::Comment(_) => None,
        NodeData::Text(text) => {
            if text.trim().is_empty() {
                None
            } else {
                Some(serde_json::json!({
                    "type": "text",
                    "text": text
                }))
            }
        }
        NodeData::Element { name, attrs } => {
            let rect = get_node_rect(dom, node_id, layout_map);

            let mut children_json = Vec::new();
            for &child_id in dom.children(node_id) {
                if let Some(child_val) = serialize_node(dom, child_id, layout_map) {
                    children_json.push(child_val);
                }
            }

            let mut attrs_map = serde_json::Map::new();
            for (attr_name, attr_value) in attrs {
                attrs_map.insert(
                    attr_name.clone(),
                    serde_json::Value::String(attr_value.clone()),
                );
            }

            Some(serde_json::json!({
                "type": "element",
                "tag": name.to_lowercase(),
                "attrs": attrs_map,
                "rect": {
                    "x": rect.origin.x as f64,
                    "y": rect.origin.y as f64,
                    "width": rect.size.width as f64,
                    "height": rect.size.height as f64
                },
                "children": children_json
            }))
        }
    }
}

/// Renders the HTML and CSS with the given viewport width and height, then walks the layout
/// tree and DOM tree to export a normalized oracle snapshot as a JSON Value.
pub fn export_snapshot(html: &str, css: &str, width: u32, height: u32) -> serde_json::Value {
    let _ = height; // height is kept for interface matching with the schema if needed.
    let page = crate::engine::render(html, css, width as f32);

    let mut layout_map = HashMap::new();
    collect_layout_rects(&page.layout, &mut layout_map);

    if let Some(doc_element) = find_document_element(&page.dom) {
        match serialize_node(&page.dom, doc_element, &layout_map) {
            Some(val) => val,
            None => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_hello_snapshot() {
        let html = r#"<!DOCTYPE html>
<html><head><title>hello</title><style>
  body { margin: 0; }
  .box { width: 200px; height: 100px; background-color: red; }
  p { color: green; }
</style></head>
<body><div class="box"></div><p>Hello <a href="https://example.com">world</a></p></body></html>"#;

        // Note: we can pass empty CSS because style is inside <style> and get hoisted
        let css = "";
        let width = 800;
        let height = 600;

        let snapshot = export_snapshot(html, css, width, height);

        // Let's assert shape and structure of the snapshot JSON.
        assert_eq!(snapshot["type"], "element");
        assert_eq!(snapshot["tag"], "html");

        // Verify rect exists on root and has numeric values
        let root_rect = &snapshot["rect"];
        assert!(root_rect["x"].is_number());
        assert!(root_rect["y"].is_number());
        assert!(root_rect["width"].is_number());
        assert!(root_rect["height"].is_number());

        // Locate head and body in html's children
        let html_children = snapshot["children"]
            .as_array()
            .expect("html children is array");
        let head = html_children
            .iter()
            .find(|&c| c["tag"] == "head")
            .expect("head element exists");
        let body = html_children
            .iter()
            .find(|&c| c["tag"] == "body")
            .expect("body element exists");

        // Assert rect exists on head and body
        assert!(head["rect"]["x"].is_number());
        assert!(body["rect"]["y"].is_number());

        // head > title("hello")
        let head_children = head["children"].as_array().expect("head children is array");
        let title = head_children
            .iter()
            .find(|&c| c["tag"] == "title")
            .expect("title element exists");
        let title_children = title["children"]
            .as_array()
            .expect("title children is array");
        let title_text = title_children.first().expect("title has child");
        assert_eq!(title_text["type"], "text");
        assert_eq!(title_text["text"], "hello");

        // body > div.box and p
        let body_children = body["children"].as_array().expect("body children is array");
        let div_box = body_children
            .iter()
            .find(|&c| c["tag"] == "div")
            .expect("div element exists");
        assert_eq!(div_box["attrs"]["class"], "box");

        // div.box width is ~200 and height ~100 (allow a tolerance of +/- 2px)
        let div_width = div_box["rect"]["width"].as_f64().expect("width is float");
        let div_height = div_box["rect"]["height"].as_f64().expect("height is float");
        assert!(
            (div_width - 200.0).abs() < 2.0,
            "div width {} not close to 200",
            div_width
        );
        assert!(
            (div_height - 100.0).abs() < 2.0,
            "div height {} not close to 100",
            div_height
        );

        let p = body_children
            .iter()
            .find(|&c| c["tag"] == "p")
            .expect("p element exists");
        let p_children = p["children"].as_array().expect("p children is array");

        // Assert children of p: text "Hello " and `a[href]` with text "world"
        let p_text = p_children
            .iter()
            .find(|&c| c["type"] == "text")
            .expect("p text child exists");
        assert_eq!(p_text["text"], "Hello ");

        let a_tag = p_children
            .iter()
            .find(|&c| c["tag"] == "a")
            .expect("a element exists");
        assert_eq!(a_tag["attrs"]["href"], "https://example.com");

        let a_children = a_tag["children"].as_array().expect("a children is array");
        let a_text = a_children
            .iter()
            .find(|&c| c["type"] == "text")
            .expect("a text child exists");
        assert_eq!(a_text["text"], "world");

        // Assert rect properties for all elements in the snapshot recursively
        fn assert_all_elements_have_rects(val: &serde_json::Value) {
            if val["type"] == "element" {
                assert!(
                    val["rect"]["x"].is_number(),
                    "element {} missing rect.x",
                    val["tag"]
                );
                assert!(
                    val["rect"]["y"].is_number(),
                    "element {} missing rect.y",
                    val["tag"]
                );
                assert!(
                    val["rect"]["width"].is_number(),
                    "element {} missing rect.width",
                    val["tag"]
                );
                assert!(
                    val["rect"]["height"].is_number(),
                    "element {} missing rect.height",
                    val["tag"]
                );

                if let Some(arr) = val["children"].as_array() {
                    for child in arr {
                        assert_all_elements_have_rects(child);
                    }
                }
            }
        }
        assert_all_elements_have_rects(&snapshot);
    }
}
