use crate::css::values::CssValue;
use crate::style::ComputedStyle;

/// Helper to get the computed float value of a style.
/// Returns Some("left"), Some("right"), or None.
///
/// // TODO(spec): DO NOT implement text/line-box shortening or wrapping content around the float.
/// // TODO(spec): DO NOT implement `clear`.
/// // TODO(spec): DO NOT implement float stacking of multiple floats side-by-side beyond the basic left/right edge placement.
pub(crate) fn get_float_value(style: &ComputedStyle) -> Option<&str> {
    match style.get("float") {
        Some(CssValue::Keyword(kw)) => {
            if kw == "left" {
                Some("left")
            } else if kw == "right" {
                Some("right")
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::layout::layout_document;
    use crate::style::compute_styles;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_float_left_positions_at_left_edge() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, float_box);

        let text = dom.create_node(NodeData::Text("ab".into())); // "ab" is 16px wide
        dom.append_child(float_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; padding-left: 10px; }
            div {
                float: left;
                padding-left: 5px;
                padding-right: 5px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // Float box should be placed directly inside body_box.children
        let float_layout = &body_box.children[0];

        // Content x-origin of body is border_box_x (0.0) + margin_left (0.0) + border_left (0.0) + padding_left (10.0) = 10.0
        // float_layout should start at x = 10.0
        assert!(approx_eq(float_layout.rect.origin.x, 10.0));
        // Width of float should be shrink-to-fit: "ab" is 16px + 5px padding_left + 5px padding_right = 26px
        assert!(approx_eq(float_layout.rect.size.width, 26.0));
    }

    #[test]
    fn test_float_right_positions_at_right_edge() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, float_box);

        let text = dom.create_node(NodeData::Text("ab".into())); // 16px
        dom.append_child(float_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; padding-left: 10px; }
            div {
                float: right;
                padding-left: 5px;
                padding-right: 5px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];
        let float_layout = &body_box.children[0];

        // Content x-origin of body is 10.0. Width of body content box is 500.0 (since width is explicit content-width 500.0).
        // So right edge of body content box is 10.0 + 500.0 = 510.0.
        // Float layout width is 26.0.
        // So float layout origin.x should be 510.0 - 26.0 = 484.0.
        assert!(approx_eq(float_layout.rect.origin.x, 484.0));
        assert!(approx_eq(float_layout.rect.size.width, 26.0));
    }

    #[test]
    fn test_float_removed_from_vertical_flow() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, float_box);

        let normal_box = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(body, normal_box);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(normal_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                float: left;
                width: 100px;
                height: 50px;
            }
            p {
                display: block;
                margin-top: 10px;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // Children of body: float_box (index 0) and normal_box (index 1)
        assert_eq!(body_box.children.len(), 2);
        let float_layout = &body_box.children[0];
        let normal_layout = &body_box.children[1];

        // The float is positioned at y = 0.0, height = 50.0.
        // But since float is removed from flow, the normal_box (p) should be positioned
        // as if the float weren't there.
        // Inside body, the cursor starts at y = 0.0.
        // The normal_box has margin-top = 10.0, so its border box y should be 10.0.
        assert!(approx_eq(float_layout.rect.origin.y, 0.0));
        assert!(approx_eq(normal_layout.rect.origin.y, 10.0));
    }
}
