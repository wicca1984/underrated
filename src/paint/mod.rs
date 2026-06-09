use crate::css::values::{Color, CssValue};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A single item to be displayed on the screen.
/// spec: S-12
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
}

/// A list of display items, representing the final visual output.
/// spec: S-12
pub struct DisplayList(pub Vec<DisplayItem>);

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

            // spec: if node is a Text node -> Text item
            if let Some(NodeData::Text(text)) = dom.data(node_id) {
                let color = match style.get("color") {
                    Some(CssValue::Color(c)) => c.clone(),
                    _ => Color::Rgba(0, 0, 0, 255), // default black
                };
                items.push(DisplayItem::Text {
                    rect: layout_box.rect,
                    text: text.clone(),
                    color,
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
                DisplayItem::Text { text, color, .. } => {
                    if text == "paint me" && *color == Color::Rgba(0, 0, 255, 255) {
                        found_text = true;
                    }
                }
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
}
