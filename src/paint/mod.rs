use crate::css::values::{Color, CssValue};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A single item to be displayed on the screen.
/// spec: S-12
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayItem {
    SolidRect {
        rect: Rect,
        color: Color,
    },
    Text {
        rect: Rect,
        text: String,
        color: Color,
    },
    Image {
        rect: Rect,
        src: String,
    },
}

/// A list of display items, representing the final visual output.
/// spec: S-12
pub struct DisplayList(pub Vec<DisplayItem>);

/// Helper to extract border width from computed styles in px.
/// spec: S-39
fn get_border_width(style: &ComputedStyle, prop: &str) -> f32 {
    match style.get(prop) {
        Some(CssValue::Length(v, crate::css::values::LengthUnit::Px)) => *v,
        _ => 0.0,
    }
}

/// Helper to recursively find any `Color` value inside a CSS property.
/// spec: S-39
fn find_color(value: &CssValue) -> Option<Color> {
    match value {
        CssValue::Color(c) => Some(c.clone()),
        CssValue::Multiple(values) => {
            for v in values {
                if let Some(c) = find_color(v) {
                    return Some(c);
                }
            }
            None
        }
        _ => None,
    }
}

/// Helper to resolve the general border color of an element style,
/// with fallbacks to shorthand `border`, computed text `color`, and finally black.
/// spec: S-39
fn get_border_color(style: &ComputedStyle) -> Color {
    // 1. Try "border-color" property
    if let Some(val) = style.get("border-color")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 2. Try the shorthand "border" property
    if let Some(val) = style.get("border")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 3. Fall back to computed text "color" (as standard currentColor fallback)
    if let Some(val) = style.get("color")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 4. Default to black
    Color::Rgba(0, 0, 0, 255)
}

/// Helper to resolve a specific edge border color, falling back to the resolved border color.
/// spec: S-39
fn get_edge_color(style: &ComputedStyle, edge_prop: &str, border_color: &Color) -> Color {
    if let Some(val) = style.get(edge_prop)
        && let Some(c) = find_color(val)
    {
        return c;
    }
    border_color.clone()
}

/// Helper to check if a ComputedStyle contains underline text decoration.
/// spec: S-55
fn has_underline(style: &ComputedStyle) -> bool {
    // spec: the `text-decoration` shorthand set to `none` clears the line,
    // overriding any `underline` keyword (e.g. from `text-decoration-line`).
    if let Some(CssValue::Keyword(s)) = style.get("text-decoration")
        && s.eq_ignore_ascii_case("none")
    {
        return false;
    }
    for prop in &["text-decoration", "text-decoration-line"] {
        if let Some(val) = style.get(prop) {
            match val {
                CssValue::Keyword(s) => {
                    if s.eq_ignore_ascii_case("underline") {
                        return true;
                    }
                }
                CssValue::Multiple(values) => {
                    for v in values {
                        if let CssValue::Keyword(s) = v
                            && s.eq_ignore_ascii_case("underline")
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

/// Helper to recursively check if a node or any of its DOM ancestors has computed text-decoration underline.
/// spec: S-55
fn is_underlined(dom: &Dom, node_id: NodeId, styles: &HashMap<NodeId, ComputedStyle>) -> bool {
    let mut current = Some(node_id);
    let mut depth = 0;
    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(style) = styles.get(&curr_id)
            && has_underline(style)
        {
            return true;
        }
        current = dom.parent(curr_id);
        depth += 1;
    }
    false
}

/// Builds a display list from the layout tree.
/// spec: S-12
pub fn build_display_list(
    layout: &LayoutBox,
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> DisplayList {
    let mut items = Vec::new();
    let mut stack = vec![layout];

    // spec: iterative pre-order traversal (no unbounded recursion — I-6)
    while let Some(layout_box) = stack.pop() {
        if let Some((node_id, style)) = layout_box
            .node
            .and_then(|id| styles.get(&id).map(|s| (id, s)))
        {
            // spec: if node has background-color -> SolidRect
            if let Some(CssValue::Color(color)) = style.get("background-color") {
                // TODO(spec): border/images/gradients/rasterization
                items.push(DisplayItem::SolidRect {
                    rect: layout_box.rect,
                    color: color.clone(),
                });
            }

            // spec: S-39: read computed border widths
            let border_top = get_border_width(style, "border-top-width");
            let border_right = get_border_width(style, "border-right-width");
            let border_bottom = get_border_width(style, "border-bottom-width");
            let border_left = get_border_width(style, "border-left-width");

            // If any border has a non-zero width, emit SolidRects for the active edges
            if border_top > 0.0 || border_right > 0.0 || border_bottom > 0.0 || border_left > 0.0 {
                let rect = layout_box.rect;
                let x = rect.origin.x;
                let y = rect.origin.y;
                let w = rect.size.width.max(0.0);
                let h = rect.size.height.max(0.0);

                // Clip/no-panic on weird sizes (I-6) by clamping border widths to fit within box dimensions
                let t = border_top.max(0.0).min(h);
                let b = border_bottom.max(0.0).min(h - t);
                let l = border_left.max(0.0).min(w);
                let r = border_right.max(0.0).min(w - l);

                let border_color = get_border_color(style);
                let top_color = get_edge_color(style, "border-top-color", &border_color);
                let right_color = get_edge_color(style, "border-right-color", &border_color);
                let bottom_color = get_edge_color(style, "border-bottom-color", &border_color);
                let left_color = get_edge_color(style, "border-left-color", &border_color);

                // Emit 4 border edges as SolidRects, ensuring no empty or negative rects are painted
                // Top border strip
                if t > 0.0 && w > 0.0 {
                    items.push(DisplayItem::SolidRect {
                        rect: Rect::new(x, y, w, t),
                        color: top_color,
                    });
                }
                // Bottom border strip
                if b > 0.0 && w > 0.0 {
                    items.push(DisplayItem::SolidRect {
                        rect: Rect::new(x, y + h - b, w, b),
                        color: bottom_color,
                    });
                }
                // Left border strip
                if l > 0.0 && h - t - b > 0.0 {
                    items.push(DisplayItem::SolidRect {
                        rect: Rect::new(x, y + t, l, h - t - b),
                        color: left_color,
                    });
                }
                // Right border strip
                if r > 0.0 && h - t - b > 0.0 {
                    items.push(DisplayItem::SolidRect {
                        rect: Rect::new(x + w - r, y + t, r, h - t - b),
                        color: right_color,
                    });
                }
            }

            // spec: if node is a Text node -> Text item
            if let Some(NodeData::Text(text)) = dom.data(node_id) {
                // spec: S-55: reliably carry computed color
                let color = style
                    .get("color")
                    .and_then(find_color)
                    .unwrap_or(Color::Rgba(0, 0, 0, 255));

                items.push(DisplayItem::Text {
                    rect: layout_box.rect,
                    text: text.clone(),
                    color: color.clone(),
                });

                // spec: S-55: when computed 'text-decoration: underline' is present,
                // additionally emit an underline SolidRect beneath the text run.
                // Multi-line/decoration-color -> // TODO(spec).
                if is_underlined(dom, node_id, styles) {
                    let rect = layout_box.rect;
                    let x = rect.origin.x;
                    let y = rect.origin.y;
                    let width = rect.size.width;
                    let height = rect.size.height;

                    // Position underline beneath the text run (e.g. at 2/3 of line height or bottom-ish)
                    // For standard 16px line height with 8px font, y + 10.0 or 11.0 is beneath the text run.
                    let underline_y = if height >= 12.0 {
                        y + 11.0
                    } else {
                        y + height.max(2.0) - 1.0
                    };
                    let underline_rect = Rect::new(x, underline_y, width, 1.0);

                    items.push(DisplayItem::SolidRect {
                        rect: underline_rect,
                        color: color.clone(),
                    });
                }
            }

            // spec: if node is an img Element node -> Image item (S-58)
            if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
                && name == "img"
                && let Some(src) = dom.get_attribute(node_id, "src")
            {
                items.push(DisplayItem::Image {
                    rect: layout_box.rect,
                    src: src.to_string(),
                });
            }
        }

        // Pre-order traversal: process current, then children left-to-right.
        // Since we use a stack (LIFO), we push children in reverse order.
        for child in layout_box.children.iter().rev() {
            stack.push(child);
        }
    }

    DisplayList(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::layout::layout_document;
    use crate::style::compute_styles;

    #[test]
    fn test_paint_basic() {
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

        let text = dom.create_node(NodeData::Text("paint me".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div { background-color: #ff0000; color: #0000ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // items should contain:
        // 1. SolidRect for div
        // 2. Text for "paint me"

        let mut found_rect = false;
        let mut found_text = false;

        for item in &items {
            match item {
                DisplayItem::SolidRect { color, .. } => {
                    if *color == Color::Rgba(255, 0, 0, 255) {
                        found_rect = true;
                    }
                }
                DisplayItem::Text { text, color, .. }
                    if text == "paint me" && *color == Color::Rgba(0, 0, 255, 255) =>
                {
                    found_text = true;
                }
                _ => {}
            }
        }

        assert!(found_rect, "SolidRect for div not found");
        assert!(found_text, "Text item for 'paint me' not found");
    }

    #[test]
    fn test_paint_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div1);
        dom.append_child(doc, div2);

        let stylesheet = parse_stylesheet(
            "
            div { background-color: blue; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Should have 2 items in order
        assert_eq!(items.len(), 2);
        for item in &items {
            match item {
                DisplayItem::SolidRect { color, .. } => {
                    assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
                }
                _ => panic!("Expected SolidRect"),
            }
        }
    }

    #[test]
    fn test_paint_border() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
                border-right-width: 3px;
                border-bottom-width: 4px;
                border-left-width: 5px;
                border-color: #00ff00;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(rects.len(), 4);

        let mut top_found = false;
        let mut bottom_found = false;
        let mut left_found = false;
        let mut right_found = false;

        let green = Color::Rgba(0, 255, 0, 255);

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &green);
                if rect.origin.x == 0.0
                    && rect.origin.y == 0.0
                    && rect.size.width == 108.0
                    && rect.size.height == 2.0
                {
                    top_found = true;
                } else if rect.origin.x == 0.0
                    && rect.origin.y == 102.0
                    && rect.size.width == 108.0
                    && rect.size.height == 4.0
                {
                    bottom_found = true;
                } else if rect.origin.x == 0.0
                    && rect.origin.y == 2.0
                    && rect.size.width == 5.0
                    && rect.size.height == 100.0
                {
                    left_found = true;
                } else if rect.origin.x == 105.0
                    && rect.origin.y == 2.0
                    && rect.size.width == 3.0
                    && rect.size.height == 100.0
                {
                    right_found = true;
                }
            }
        }

        assert!(top_found, "Top border rect mismatch");
        assert!(bottom_found, "Bottom border rect mismatch");
        assert!(left_found, "Left border rect mismatch");
        assert!(right_found, "Right border rect mismatch");
    }

    #[test]
    fn test_paint_border_fallback() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Scenario A: border-color is absent, but text "color" is specified -> should use text color
        let stylesheet_a = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
                color: #ff00ff;
            }
        ",
        );
        let styles_a = compute_styles(&dom, &stylesheet_a);
        let layout_a = layout_document(&dom, &styles_a, 800.0);
        let display_list_a = build_display_list(&layout_a, &dom, &styles_a);
        let items_a = display_list_a.0;

        let rects_a: Vec<&DisplayItem> = items_a
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();
        assert_eq!(rects_a.len(), 1);
        if let DisplayItem::SolidRect { color, .. } = rects_a[0] {
            assert_eq!(color, &Color::Rgba(255, 0, 255, 255));
        } else {
            panic!("Expected SolidRect");
        }

        // Scenario B: border-color and color are both absent -> should default to black
        let stylesheet_b = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
            }
        ",
        );
        let styles_b = compute_styles(&dom, &stylesheet_b);
        let layout_b = layout_document(&dom, &styles_b, 800.0);
        let display_list_b = build_display_list(&layout_b, &dom, &styles_b);
        let items_b = display_list_b.0;

        let rects_b: Vec<&DisplayItem> = items_b
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();
        assert_eq!(rects_b.len(), 1);
        if let DisplayItem::SolidRect { color, .. } = rects_b[0] {
            assert_eq!(color, &Color::Rgba(0, 0, 0, 255));
        } else {
            panic!("Expected SolidRect");
        }
    }

    #[test]
    fn test_paint_border_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let text = dom.create_node(NodeData::Text("paint me".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div {
                background-color: #ff0000;
                border-top-width: 2px;
                color: #0000ff;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // The expected order of display list items:
        // 1. SolidRect (background-color: red)
        // 2. SolidRect (border-color: text color blue, because border-color is absent)
        // 3. Text ("paint me", color: blue) - fragment 1
        // 4. Text ("paint me", color: blue) - fragment 2

        assert_eq!(items.len(), 4);

        // 1. Background
        if let DisplayItem::SolidRect { color, .. } = &items[0] {
            assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected background SolidRect first");
        }

        // 2. Border
        if let DisplayItem::SolidRect { color, .. } = &items[1] {
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected border SolidRect second");
        }

        // 3. Text - Fragment 1
        if let DisplayItem::Text { text, color, .. } = &items[2] {
            assert_eq!(text, "paint me");
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected Text item third");
        }

        // 4. Text - Fragment 2
        if let DisplayItem::Text { text, color, .. } = &items[3] {
            assert_eq!(text, "paint me");
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected Text item fourth");
        }
    }

    #[test]
    fn test_paint_text_color() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("colored text".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { color: #00ff00; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert!(!text_items.is_empty(), "Expected at least one Text item");
        for item in text_items {
            if let DisplayItem::Text { color, .. } = item {
                assert_eq!(color, &Color::Rgba(0, 255, 0, 255));
            }
        }
    }

    #[test]
    fn test_paint_text_underline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("underlined".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline; color: #ff00ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        // Since "underlined" collapses to one word fragment, there should be 1 Text item and 1 underline SolidRect
        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            rect: text_rect,
            color: text_color,
            ..
        } = text_items[0]
        {
            assert_eq!(text_color, &Color::Rgba(255, 0, 255, 255));
            if let DisplayItem::SolidRect {
                rect: rect_rect,
                color: rect_color,
            } = solid_rects[0]
            {
                assert_eq!(rect_color, &Color::Rgba(255, 0, 255, 255));
                assert_eq!(rect_rect.origin.x, text_rect.origin.x);
                assert_eq!(rect_rect.size.width, text_rect.size.width);
                assert_eq!(rect_rect.size.height, 1.0);
                assert!(rect_rect.origin.y > text_rect.origin.y);
            } else {
                panic!("Expected SolidRect");
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_underline_ancestor() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        let text = dom.create_node(NodeData::Text("nested".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            div { text-decoration: underline; color: #ff0000; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            color: text_color, ..
        } = text_items[0]
        {
            assert_eq!(text_color, &Color::Rgba(255, 0, 0, 255));
        }
        if let DisplayItem::SolidRect {
            color: rect_color, ..
        } = solid_rects[0]
        {
            assert_eq!(rect_color, &Color::Rgba(255, 0, 0, 255));
        }
    }

    #[test]
    fn test_paint_image() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), "test.png".into())],
        });
        dom.append_child(doc, img);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(image_items.len(), 1);
        if let DisplayItem::Image { src, .. } = image_items[0] {
            assert_eq!(src, "test.png");
        } else {
            panic!("Expected DisplayItem::Image");
        }
    }
}
