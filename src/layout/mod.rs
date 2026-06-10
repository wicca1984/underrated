mod flex;
mod inline;
mod position;

pub(crate) use position::is_absolute_or_fixed;

use crate::css::values::{CssValue, LengthUnit};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::inline::{layout_inline, layout_inline_run};
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A box in the layout tree.
/// spec: S-11
pub struct LayoutBox {
    pub node: Option<NodeId>,
    pub rect: Rect,
    pub children: Vec<LayoutBox>,
}

pub(crate) const MAX_DEPTH: usize = 1000;

fn collapse_margins(m1: f32, m2: f32) -> f32 {
    if m1 >= 0.0 && m2 >= 0.0 {
        m1.max(m2)
    } else if m1 < 0.0 && m2 < 0.0 {
        m1.min(m2) // most-negative
    } else {
        // mixed sign (one positive or zero, one negative)
        let pos = m1.max(0.0).max(m2.max(0.0));
        let neg = m1.min(0.0).min(m2.min(0.0));
        pos + neg
    }
}

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
    let mut prev_margin_bottom: Option<f32> = None;
    let mut last_child_box_max_y: Option<f32> = None;

    for &child in dom.children(dom.document()) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }

        let offset_y =
            if let (Some(prev_mb), Some(last_max_y)) = (prev_margin_bottom, last_child_box_max_y) {
                let child_style = styles.get(&child);
                let margin_top = child_style
                    .map(|s| get_px(s, "margin-top", 0.0))
                    .unwrap_or(0.0);
                let collapsed = collapse_margins(prev_mb, margin_top);
                last_max_y + collapsed - margin_top
            } else {
                cursor_y
            };

        if let Some(child_box) = layout_node(dom, styles, child, viewport_width, 0.0, offset_y, 0) {
            let margin_bottom = styles
                .get(&child)
                .map(|s| get_px(s, "margin-bottom", 0.0))
                .unwrap_or(0.0);

            last_child_box_max_y = Some(child_box.rect.max_y());
            prev_margin_bottom = Some(margin_bottom);

            cursor_y = child_box.rect.max_y() + margin_bottom;
            root_box.children.push(child_box);
        }
    }

    root_box.rect.size.height = cursor_y;

    // Apply absolute and fixed positioning
    // spec: S-31
    position::layout_absolute_and_fixed_elements(dom, styles, viewport_width, &mut root_box);

    // Apply relative positioning offsets
    // spec: S-31
    position::resolve_relative_positions(&mut root_box, styles, 0);

    root_box
}

pub(crate) fn layout_node(
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

    // display: flex
    if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "flex") {
        return crate::layout::flex::layout_flex_container(
            dom,
            styles,
            node,
            containing_width,
            offset_x,
            offset_y,
            depth,
        );
    }

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
    // TODO(spec): Parent-child margin collapse (collapse-through) is out of scope.
    // Also, collapse suppression by intervening padding/border, collapse-through empty blocks,
    // clear/clearance, floats, and BFC establishment via overflow are out of scope.
    let mut child_cursor_y = border_box_y + border_top + padding_top;

    let layoutable_children = get_layoutable_children(dom, styles, node);
    let has_inline = layoutable_children
        .iter()
        .any(|&c| is_inline_level(styles, dom, c));
    let has_block = layoutable_children
        .iter()
        .any(|&c| !is_inline_level(styles, dom, c));

    // Layout children
    if has_inline && !has_block {
        // If ALL children are inline, keep current behavior (single inline pass)
        let (line_boxes, total_height) = layout_inline(
            dom,
            styles,
            node,
            content_width,
            border_box_x + border_left + padding_left,
            child_cursor_y,
            depth,
        );
        children.extend(line_boxes);
        child_cursor_y += total_height;
    } else if !has_inline {
        // If ALL children are block (or empty), keep current block behavior
        let mut prev_margin_bottom: Option<f32> = None;
        let mut last_child_box_max_y: Option<f32> = None;

        for &child in dom.children(node) {
            if is_absolute_or_fixed(styles, child) {
                continue;
            }

            let offset_y = if let (Some(prev_mb), Some(last_max_y)) =
                (prev_margin_bottom, last_child_box_max_y)
            {
                let child_style = styles.get(&child);
                let margin_top = child_style
                    .map(|s| get_px(s, "margin-top", 0.0))
                    .unwrap_or(0.0);
                let collapsed = collapse_margins(prev_mb, margin_top);
                last_max_y + collapsed - margin_top
            } else {
                child_cursor_y
            };

            if let Some(child_box) = layout_node(
                dom,
                styles,
                child,
                content_width,
                border_box_x + border_left + padding_left,
                offset_y,
                depth + 1,
            ) {
                let margin_bottom = styles
                    .get(&child)
                    .map(|s| get_px(s, "margin-bottom", 0.0))
                    .unwrap_or(0.0);

                last_child_box_max_y = Some(child_box.rect.max_y());
                prev_margin_bottom = Some(margin_bottom);

                child_cursor_y = child_box.rect.max_y() + margin_bottom;
                children.push(child_box);
            }
        }
    } else {
        // MIXED: wrap each maximal run of consecutive inline-level children in an anonymous block box
        // spec: S-anonymous-block-boxes
        let mut prev_margin_bottom: Option<f32> = None;
        let mut last_child_box_max_y: Option<f32> = None;

        let mut i = 0;
        while i < layoutable_children.len() {
            let child = layoutable_children[i];
            if is_inline_level(styles, dom, child) {
                let mut inline_run = Vec::new();
                while i < layoutable_children.len()
                    && is_inline_level(styles, dom, layoutable_children[i])
                {
                    inline_run.push(layoutable_children[i]);
                    i += 1;
                }

                // Treat anonymous block as having margin_top = 0.0
                let start_y = if let (Some(prev_mb), Some(last_max_y)) =
                    (prev_margin_bottom, last_child_box_max_y)
                {
                    let collapsed = collapse_margins(prev_mb, 0.0);
                    last_max_y + collapsed
                } else {
                    child_cursor_y
                };

                let (line_boxes, total_height) = layout_inline_run(
                    dom,
                    styles,
                    &inline_run,
                    content_width,
                    border_box_x + border_left + padding_left,
                    start_y,
                    depth,
                );
                if !line_boxes.is_empty() {
                    let anon_box = LayoutBox {
                        node: None,
                        rect: Rect::new(
                            border_box_x + border_left + padding_left,
                            start_y,
                            content_width,
                            total_height,
                        ),
                        children: line_boxes,
                    };

                    last_child_box_max_y = Some(anon_box.rect.max_y());
                    // Treat anonymous block as having margin_bottom = 0.0
                    prev_margin_bottom = Some(0.0);
                    child_cursor_y = anon_box.rect.max_y();

                    children.push(anon_box);
                }
            } else {
                let offset_y = if let (Some(prev_mb), Some(last_max_y)) =
                    (prev_margin_bottom, last_child_box_max_y)
                {
                    let child_style = styles.get(&child);
                    let margin_top = child_style
                        .map(|s| get_px(s, "margin-top", 0.0))
                        .unwrap_or(0.0);
                    let collapsed = collapse_margins(prev_mb, margin_top);
                    last_max_y + collapsed - margin_top
                } else {
                    child_cursor_y
                };

                if let Some(child_box) = layout_node(
                    dom,
                    styles,
                    child,
                    content_width,
                    border_box_x + border_left + padding_left,
                    offset_y,
                    depth + 1,
                ) {
                    let margin_bottom = styles
                        .get(&child)
                        .map(|s| get_px(s, "margin-bottom", 0.0))
                        .unwrap_or(0.0);

                    last_child_box_max_y = Some(child_box.rect.max_y());
                    prev_margin_bottom = Some(margin_bottom);

                    child_cursor_y = child_box.rect.max_y() + margin_bottom;
                    children.push(child_box);
                }
                i += 1;
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

fn get_layoutable_children(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
) -> Vec<NodeId> {
    let mut result = Vec::new();
    for &child in dom.children(node) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Text(_) => {
                    result.push(child);
                }
                NodeData::Element { .. } => {
                    if let Some(style) = styles.get(&child) {
                        if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "none")
                        {
                            continue;
                        }
                        result.push(child);
                    }
                }
                _ => {}
            }
        }
    }
    result
}

fn is_inline_level(styles: &HashMap<NodeId, ComputedStyle>, dom: &Dom, child: NodeId) -> bool {
    if let Some(data) = dom.data(child) {
        match data {
            NodeData::Text(_) => true,
            NodeData::Element { .. } => {
                if let Some(style) = styles.get(&child) {
                    matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "inline")
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

pub(crate) fn get_px(style: &ComputedStyle, prop: &str, default: f32) -> f32 {
    match style.get(prop) {
        Some(CssValue::Length(v, LengthUnit::Px)) => *v,
        _ => default,
    }
}

/// Performs hit-testing on the layout tree.
///
/// Pre-order traversal; returns the NodeId of the deepest box whose rect contains (x, y);
/// among siblings, later (painted-on-top) wins.
/// Boxes with no `node` (None) are skipped for the result but their children are still tested.
/// Bounded to prevent stack overflow.
///
/// spec: S-36
pub fn hit_test(root: &LayoutBox, x: f32, y: f32) -> Option<NodeId> {
    let mut best_node = None;
    hit_test_impl(root, x, y, 0, &mut best_node);
    best_node
}

fn hit_test_impl(box_: &LayoutBox, x: f32, y: f32, depth: usize, best_node: &mut Option<NodeId>) {
    if depth > MAX_DEPTH {
        // TODO(spec): Report stack depth limit exceeded in hit_test
        return;
    }

    if box_.rect.contains(crate::geom::Point { x, y }) && box_.node.is_some() {
        *best_node = box_.node;
    }

    for child in &box_.children {
        hit_test_impl(child, x, y, depth + 1, best_node);
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

        // CHAR_WIDTH is 8.0. LINE_HEIGHT is 8.0 (font line_height).
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

        // div_box height should be 2 * LINE_HEIGHT (8.0) = 16.0
        assert!(approx_eq(div_box.rect.size.height, 16.0));

        // Check first line children
        let line1 = &div_box.children[0];
        assert_eq!(line1.children.len(), 3);
        assert!(approx_eq(line1.rect.size.width, 136.0));

        let line2 = &div_box.children[1];
        assert_eq!(line2.children.len(), 3);
        assert!(approx_eq(line2.rect.size.width, 72.0));
    }

    #[test]
    fn test_inline_line_wrapping_acceptance() {
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

        // Word "aaaa " is 5 chars * 8px = 40px
        // Word "bbbb " is 5 chars * 8px = 40px
        // Word "cccc" is 4 chars * 8px = 32px
        // Total of first two is 80px <= 100px width.
        // Adding third makes 112px > 100px width.
        // It must wrap to at least 2 lines.
        let text = dom.create_node(NodeData::Text("aaaa bbbb cccc".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        assert!(div_box.children.len() >= 2);

        let line1 = &div_box.children[0];
        assert_eq!(line1.children.len(), 2); // "aaaa ", "bbbb "
        assert!(approx_eq(line1.rect.size.width, 80.0));

        let line2 = &div_box.children[1];
        assert_eq!(line2.children.len(), 1); // "cccc"
        assert!(approx_eq(line2.rect.size.width, 32.0));
    }

    #[test]
    fn test_s57_text_line_flow_wiring() {
        // spec: S-57
        // Verify that long text in a narrow container occupies multiple line-heights of block height,
        // using the real font glyph widths and inline wrapping, and advances block height accordingly.
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

        // Built-in font has character width 8.0, and height 8.0.
        // "word1 word2 word3 word4" has:
        // "word1 " (40px with space)
        // "word2 " (40px with space)
        // "word3 " (40px with space)
        // "word4"  (40px without space)
        // In a container of width 50px, each word has to be wrapped on its own line!
        // This will result in 4 lines, total height 4 * 8.0 = 32.0.
        let text = dom.create_node(NodeData::Text("word1 word2 word3 word4".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 50px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        // Should have exactly 4 line boxes
        assert_eq!(div_box.children.len(), 4);

        // Block height must be exactly 4 * line-height (8.0) = 32.0
        assert!(approx_eq(div_box.rect.size.height, 32.0));

        // Let's check each line width:
        // "word1 " is 6 characters (5 + space), so 48px.
        // "word2 " is 48px.
        // "word3 " is 48px.
        // "word4" is 5 characters (no space), so 40px.
        assert!(approx_eq(div_box.children[0].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[1].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[2].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[3].rect.size.width, 40.0));
    }

    #[test]
    fn test_relative_position_offset() {
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

        let sibling = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, sibling);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                height: 50px;
            }
            body > div:first-child {
                position: relative;
                top: 10px;
                left: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // First div (relative)
        let div_box = &body_box.children[0];
        assert_eq!(div_box.node, Some(div));
        // Static layout position would be (0, 0)
        // With relative top:10px; left:20px, it should be offset to (20, 10)
        assert!(approx_eq(div_box.rect.origin.x, 20.0));
        assert!(approx_eq(div_box.rect.origin.y, 10.0));

        // Second div (sibling)
        let sibling_box = &body_box.children[1];
        assert_eq!(sibling_box.node, Some(sibling));
        // Sibling position should not be affected by first div's relative offset
        // Static height of first div is 50px. Sibling should start at (0, 50).
        assert!(approx_eq(sibling_box.rect.origin.x, 0.0));
        assert!(approx_eq(sibling_box.rect.origin.y, 50.0));
    }

    #[test]
    fn test_absolute_position_and_out_of_flow() {
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

        let div_abs = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div_abs);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                height: 50px;
            }
            body > div:nth-child(2) {
                position: absolute;
                top: 5px;
                left: 5px;
                width: 100px;
                height: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // div1 (static)
        let div1_box = &body_box.children[0];
        assert_eq!(div1_box.node, Some(div1));
        assert!(approx_eq(div1_box.rect.origin.x, 0.0));
        assert!(approx_eq(div1_box.rect.origin.y, 0.0));

        // div2 (static, sibling after the absolute div)
        let div2_box = &body_box.children[1];
        assert_eq!(div2_box.node, Some(div2));
        // Since absolute div is out of flow, div2 should immediately follow div1.
        // div1 has height 50, so div2 starts at (0, 50).
        assert!(approx_eq(div2_box.rect.origin.x, 0.0));
        assert!(approx_eq(div2_box.rect.origin.y, 50.0));

        // div_abs (absolute)
        // It is integrated into parent's children (or root_box/nearest ancestor layout box)
        // So let's find it in the layout tree.
        let div_abs_box = body_box
            .children
            .iter()
            .find(|b| b.node == Some(div_abs))
            .expect("abs box should exist in layout tree");
        // It should be placed at (5, 5) relative to viewport/containing block
        assert!(approx_eq(div_abs_box.rect.origin.x, 5.0));
        assert!(approx_eq(div_abs_box.rect.origin.y, 5.0));
        assert!(approx_eq(div_abs_box.rect.size.width, 100.0));
        assert!(approx_eq(div_abs_box.rect.size.height, 100.0));
    }

    #[test]
    fn test_relative_nested_absolute_no_shifting() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let rel_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, rel_div);

        let abs_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(rel_div, abs_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            body > div {
                position: relative;
                top: 20px;
                left: 30px;
                height: 50px;
            }
            div > div {
                position: absolute;
                top: 5px;
                left: 5px;
                width: 40px;
                height: 40px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // Relative div
        let rel_box = &body_box.children[0];
        assert_eq!(rel_box.node, Some(rel_div));
        assert!(approx_eq(rel_box.rect.origin.x, 30.0));
        assert!(approx_eq(rel_box.rect.origin.y, 20.0));

        // Absolute child (it is positioned relative to viewport for v1)
        // In the layout tree, it is added under its nearest ancestor layout box, which is rel_box.
        let abs_box = rel_box
            .children
            .iter()
            .find(|b| b.node == Some(abs_child))
            .expect("abs child box should exist in layout tree");
        // Since it is positioned relative to the viewport/containing block (use viewport/root for v1),
        // its final coordinates should be (5, 5) and NOT shifted by the relative parent's offset of (30, 20)!
        assert!(approx_eq(abs_box.rect.origin.x, 5.0));
        assert!(approx_eq(abs_box.rect.origin.y, 5.0));
    }

    #[test]
    fn test_hit_test() {
        let mut dom = Dom::new();
        let node_root = dom.create_node(NodeData::Comment("root".into()));
        let node_child1 = dom.create_node(NodeData::Comment("child1".into()));
        let node_nested = dom.create_node(NodeData::Comment("nested".into()));
        let node_child2 = dom.create_node(NodeData::Comment("child2".into()));
        let node_nested_under_none = dom.create_node(NodeData::Comment("nested_under_none".into()));

        let root_box = LayoutBox {
            node: Some(node_root),
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            children: vec![
                LayoutBox {
                    node: Some(node_child1),
                    rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                    children: vec![LayoutBox {
                        node: Some(node_nested),
                        rect: Rect::new(15.0, 15.0, 20.0, 20.0),
                        children: vec![],
                    }],
                },
                LayoutBox {
                    node: Some(node_child2),
                    rect: Rect::new(40.0, 10.0, 40.0, 40.0),
                    children: vec![],
                },
                LayoutBox {
                    node: None,
                    rect: Rect::new(0.0, 80.0, 100.0, 20.0),
                    children: vec![LayoutBox {
                        node: Some(node_nested_under_none),
                        rect: Rect::new(10.0, 85.0, 20.0, 10.0),
                        children: vec![],
                    }],
                },
            ],
        };

        // Point outside everything returns None
        assert_eq!(hit_test(&root_box, 150.0, 150.0), None);
        assert_eq!(hit_test(&root_box, -5.0, 20.0), None);

        // Point only in root box returns root
        assert_eq!(hit_test(&root_box, 5.0, 5.0), Some(node_root));

        // Point in child1 but not nested returns child1
        assert_eq!(hit_test(&root_box, 12.0, 12.0), Some(node_child1));

        // Point inside nested returns nested (deepest wins)
        assert_eq!(hit_test(&root_box, 20.0, 20.0), Some(node_nested));

        // Point inside both child1 and child2 (overlapping region: child2 is later sibling, so child2 wins)
        assert_eq!(hit_test(&root_box, 45.0, 20.0), Some(node_child2));

        // Point inside the no_node box, but not nested_under_none:
        // Returns root (since no_node is skipped, and root contains the point)
        assert_eq!(hit_test(&root_box, 5.0, 95.0), Some(node_root));

        // Point inside nested_under_none (inside no_node and root):
        // Returns nested_under_none
        assert_eq!(
            hit_test(&root_box, 15.0, 90.0),
            Some(node_nested_under_none)
        );
    }

    #[test]
    fn test_display_none_absolute_subtree_pruning() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, container);

        let abs_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, abs_div);

        let stylesheet = parse_stylesheet(
            "
            div { display: none; }
            div > div { position: absolute; top: 10px; left: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // Neither container nor abs_div should have any box, so body should have 0 children
        assert_eq!(body_box.children.len(), 0);
    }

    #[test]
    fn test_deep_tree_recursion_cap() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let mut current = doc;
        // Nest 1050 levels deep
        for _ in 0..1050 {
            let div = dom.create_node(NodeData::Element {
                name: "div".into(),
                attrs: vec![],
            });
            dom.append_child(current, div);
            current = div;
        }

        let stylesheet = parse_stylesheet("div { display: block; }");
        let styles = compute_styles(&dom, &stylesheet);

        // This must not stack overflow!
        let _layout_tree = layout_document(&dom, &styles, 800.0);
    }

    #[test]
    fn test_mixed_layout_body_with_div_and_whitespace() {
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

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        let text_hi = dom.create_node(NodeData::Text("hi".into()));
        dom.append_child(p, text_hi);

        let text_ws = dom.create_node(NodeData::Text("\n".into()));
        dom.append_child(body, text_ws);

        let stylesheet = parse_stylesheet(
            "body { display: block; } div { display: block; } p { display: block; }",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // The body has a mix of div (block) and "\n" (whitespace inline).
        // The div should be laid out as a normal block box under body.
        // The whitespace text collapses and yields no line box (so no anonymous block box is created for it).
        // Therefore, body_box has exactly 1 child: the div_box.
        assert_eq!(body_box.children.len(), 1);

        let div_box = &body_box.children[0];
        assert_eq!(div_box.node, Some(div));

        // Under div, there is p (block). So div has 1 child: p_box.
        assert_eq!(div_box.children.len(), 1);

        let p_box = &div_box.children[0];
        assert_eq!(p_box.node, Some(p));

        // Under p, there is "hi" (inline). So p has 1 child: line box.
        assert_eq!(p_box.children.len(), 1);
        let line_box = &p_box.children[0];
        assert_eq!(line_box.node, None); // Anonymous line box
        assert!(line_box.rect.size.width > 0.0);
        assert!(line_box.rect.size.height > 0.0);

        let text_box = &line_box.children[0];
        assert_eq!(text_box.node, Some(text_hi));
        assert!(text_box.rect.size.width > 0.0);
        assert!(text_box.rect.size.height > 0.0);
    }

    #[test]
    fn test_mixed_layout_block_with_mixed_text_div_text() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let parent_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, parent_div);

        let text1 = dom.create_node(NodeData::Text("first ".into()));
        dom.append_child(parent_div, text1);

        let child_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(parent_div, child_div);

        let text_child = dom.create_node(NodeData::Text("second ".into()));
        dom.append_child(child_div, text_child);

        let text3 = dom.create_node(NodeData::Text(" third".into()));
        dom.append_child(parent_div, text3);

        let stylesheet = parse_stylesheet("body { display: block; } div { display: block; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let parent_div_box = &body_box.children[0];

        // Mixed layout: [text1, child_div, text3]
        // Should produce:
        // 1. Anonymous block box wrapping text1
        // 2. Normal block box for child_div
        // 3. Anonymous block box wrapping text3
        assert_eq!(parent_div_box.children.len(), 3);

        let anon1 = &parent_div_box.children[0];
        assert_eq!(anon1.node, None); // Anonymous block box
        assert!(anon1.rect.size.width > 0.0);
        assert!(anon1.rect.size.height > 0.0);

        let child_div_box = &parent_div_box.children[1];
        assert_eq!(child_div_box.node, Some(child_div));
        assert!(child_div_box.rect.size.width > 0.0);
        assert!(child_div_box.rect.size.height > 0.0);

        let anon3 = &parent_div_box.children[2];
        assert_eq!(anon3.node, None); // Anonymous block box
        assert!(anon3.rect.size.width > 0.0);
        assert!(anon3.rect.size.height > 0.0);

        // Verify correct vertical ordering and positions
        assert!(anon1.rect.origin.y < child_div_box.rect.origin.y);
        assert!(child_div_box.rect.origin.y < anon3.rect.origin.y);

        // Check height matches sum of elements
        let expected_min_height =
            anon1.rect.size.height + child_div_box.rect.size.height + anon3.rect.size.height;
        assert!(parent_div_box.rect.size.height >= expected_min_height);
    }

    #[test]
    fn test_margin_collapse_positive() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: 30px; }
            .next { margin-top: 20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = max(30, 20) = 30.
        // So box2 border box y should be box1.rect.max_y() + 30 = 50.0 + 30.0 = 80.0.
        assert!(
            approx_eq(box2.rect.origin.y, 80.0),
            "Expected second sibling border box y to be 80.0, got {}",
            box2.rect.origin.y
        );
    }

    #[test]
    fn test_margin_collapse_mixed() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: 30px; }
            .next { margin-top: -10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = max(30, 0) + min(-10, 0) = 30 + (-10) = 20.
        // So box2 border box y should be box1.rect.max_y() + 20 = 50.0 + 20.0 = 70.0.
        assert!(
            approx_eq(box2.rect.origin.y, 70.0),
            "Expected second sibling border box y to be 70.0, got {}",
            box2.rect.origin.y
        );
    }

    #[test]
    fn test_margin_collapse_both_negative() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: -10px; }
            .next { margin-top: -20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = min(-10, -20) = -20.
        // So box2 border box y should be box1.rect.max_y() + (-20) = 50.0 - 20.0 = 30.0.
        assert!(
            approx_eq(box2.rect.origin.y, 30.0),
            "Expected second sibling border box y to be 30.0, got {}",
            box2.rect.origin.y
        );
    }
}
