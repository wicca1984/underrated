mod inline;

use crate::css::values::{CssValue, LengthUnit};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::inline::layout_inline;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A box in the layout tree.
/// spec: S-11
pub struct LayoutBox {
    pub node: Option<NodeId>,
    pub rect: Rect,
    pub children: Vec<LayoutBox>,
}

const MAX_DEPTH: usize = 1000;

/// Performs block layout on the document.
/// spec: S-11
pub fn layout_document(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
) -> LayoutBox {
    let mut root_box = LayoutBox {
        node: Some(dom.document()),
        rect: Rect::new(0.0, 0.0, viewport_width, 0.0),
        children: Vec::new(),
    };

    // The document's children (usually just <html>)
    let mut cursor_y = 0.0;
    for &child in dom.children(dom.document()) {
        if let Some(child_box) = layout_node(dom, styles, child, viewport_width, 0.0, cursor_y, 0) {
            if let Some(child_style) = styles.get(&child) {
                let margin_bottom = get_px(child_style, "margin-bottom", 0.0);
                cursor_y = child_box.rect.max_y() + margin_bottom;
            }
            root_box.children.push(child_box);
        }
    }

    root_box.rect.size.height = cursor_y;
    root_box
}

fn layout_node(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> Option<LayoutBox> {
    if depth > MAX_DEPTH {
        // TODO(spec): Report stack depth limit exceeded
        return None;
    }

    let style = styles.get(&node)?;

    // display: none -> no box
    if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "none") {
        return None;
    }

    // TODO(spec): Support display: inline and others. For now, assume block.

    // Get box model values
    let margin_left = get_px(style, "margin-left", 0.0);
    let margin_right = get_px(style, "margin-right", 0.0);
    let margin_top = get_px(style, "margin-top", 0.0);

    let padding_left = get_px(style, "padding-left", 0.0);
    let padding_right = get_px(style, "padding-right", 0.0);
    let padding_top = get_px(style, "padding-top", 0.0);
    let padding_bottom = get_px(style, "padding-bottom", 0.0);

    let border_left = get_px(style, "border-left-width", 0.0);
    let border_right = get_px(style, "border-right-width", 0.0);
    let border_top = get_px(style, "border-top-width", 0.0);
    let border_bottom = get_px(style, "border-bottom-width", 0.0);

    // Calculate content width
    // width = containing-block width minus its own horizontal margin/padding/border
    let auto_width = containing_width
        - margin_left
        - margin_right
        - border_left
        - border_right
        - padding_left
        - padding_right;
    let content_width = get_px(style, "width", auto_width.max(0.0));

    // Position of the border box
    let border_box_x = offset_x + margin_left;
    let border_box_y = offset_y + margin_top;

    let mut children = Vec::new();
    let mut child_cursor_y = border_box_y + border_top + padding_top;

    // Layout children
    if has_inline_content(dom, styles, node) {
        let (line_boxes, total_height) = layout_inline(
            dom,
            styles,
            node,
            content_width,
            border_box_x + border_left + padding_left,
            child_cursor_y,
        );
        children.extend(line_boxes);
        child_cursor_y += total_height;
    } else {
        for &child in dom.children(node) {
            if let Some(child_box) = layout_node(
                dom,
                styles,
                child,
                content_width,
                border_box_x + border_left + padding_left,
                child_cursor_y,
                depth + 1,
            ) {
                if let Some(child_style) = styles.get(&child) {
                    let child_margin_bottom = get_px(child_style, "margin-bottom", 0.0);
                    child_cursor_y = child_box.rect.max_y() + child_margin_bottom;
                }
                children.push(child_box);
            }
        }
    }

    // Calculate height
    let content_height = child_cursor_y - (border_box_y + border_top + padding_top);
    let border_box_height = get_px(style, "height", content_height)
        + padding_top
        + padding_bottom
        + border_top
        + border_bottom;

    Some(LayoutBox {
        node: Some(node),
        rect: Rect::new(
            border_box_x,
            border_box_y,
            content_width + padding_left + padding_right + border_left + border_right,
            border_box_height,
        ),
        children,
    })
}

fn has_inline_content(dom: &Dom, styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> bool {
    for &child in dom.children(node) {
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Text(_) => return true,
                NodeData::Element { .. } => {
                    if let Some(style) = styles.get(&child) {
                        // If it's display: inline, it's inline content.
                        // Default for unknown or block-level is NOT inline for now.
                        if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "inline")
                        {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn get_px(style: &ComputedStyle, prop: &str, default: f32) -> f32 {
    match style.get(prop) {
        Some(CssValue::Length(v, LengthUnit::Px)) => *v,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::NodeData;
    use crate::style::compute_styles;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_nested_block_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(div1, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                margin-left: 10px;
                margin-right: 10px;
                margin-top: 10px;
                margin-bottom: 10px;
                padding-left: 20px;
                padding-right: 20px;
                padding-top: 20px;
                padding-bottom: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);

        // root_box (Document)
        assert_eq!(layout_tree.node, Some(doc));
        assert_eq!(layout_tree.children.len(), 1);

        // body_box
        let body_box = &layout_tree.children[0];
        assert_eq!(body_box.node, Some(body));
        assert!(approx_eq(body_box.rect.origin.x, 0.0));
        assert!(approx_eq(body_box.rect.origin.y, 0.0));
        assert!(approx_eq(body_box.rect.size.width, 500.0));

        // div1_box
        assert_eq!(body_box.children.len(), 1);
        let div1_box = &body_box.children[0];
        assert_eq!(div1_box.node, Some(div1));
        // body width is 500. div1 has margin 10.
        // content_width = 500 - 10 - 10 = 480.
        // border_box width = 480 + 20 + 20 = 520?
        // Wait, width property in CSS is content width.
        // "width = containing-block width minus its own horizontal margin/padding/border"
        // containing_width = 500.
        // margin_left=10, margin_right=10, padding_left=20, padding_right=20.
        // auto_width = 500 - 10 - 10 - 20 - 20 = 440.
        // content_width = 440.
        // border_box_width = 440 + 20 + 20 = 480.
        assert!(approx_eq(div1_box.rect.size.width, 480.0));
        assert!(approx_eq(div1_box.rect.origin.x, 10.0));
        assert!(approx_eq(div1_box.rect.origin.y, 10.0));

        // div2_box
        assert_eq!(div1_box.children.len(), 1);
        let div2_box = &div1_box.children[0];
        assert_eq!(div2_box.node, Some(div2));
        // containing_width for div2 is div1's content width = 440.
        // margin=10, padding=20.
        // auto_width = 440 - 10 - 10 - 20 - 20 = 380.
        // border_box_width = 380 + 20 + 20 = 420.
        assert!(approx_eq(div2_box.rect.size.width, 420.0));
        // div2 x = div1 border box x (10) + div1 padding (20) + div2 margin (10) = 40.
        assert!(approx_eq(div2_box.rect.origin.x, 40.0));
        // div2 y = div1 border box y (10) + div1 padding (20) + div2 margin (10) = 40.
        assert!(approx_eq(div2_box.rect.origin.y, 40.0));
    }

    #[test]
    fn test_display_none() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet("div { display: none; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        assert_eq!(body_box.children.len(), 0);
    }

    #[test]
    fn test_explicit_height() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet("div { display: block; height: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];
        assert!(approx_eq(div_box.rect.size.height, 100.0));
    }

    #[test]
    fn test_text_wrapping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        // CHAR_WIDTH is 8.0. LINE_HEIGHT is 16.0.
        // Words: "Hello " (48px), "world " (48px), "this " (40px), "is " (24px), "a " (16px), "test" (32px)
        // Limit: 150px
        // Line 1: "Hello ", "world ", "this " (Total 136px)
        // Line 2: "is ", "a ", "test" (Total 72px)
        let text = dom.create_node(NodeData::Text("Hello world this is a test".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 150px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        // Should have 2 line boxes
        assert_eq!(div_box.children.len(), 2);

        // div_box height should be 2 * LINE_HEIGHT (16.0) = 32.0
        assert!(approx_eq(div_box.rect.size.height, 32.0));

        // Check first line children
        let line1 = &div_box.children[0];
        assert_eq!(line1.children.len(), 3);
        assert!(approx_eq(line1.rect.size.width, 136.0));

        let line2 = &div_box.children[1];
        assert_eq!(line2.children.len(), 3);
        assert!(approx_eq(line2.rect.size.width, 72.0));
    }
}
