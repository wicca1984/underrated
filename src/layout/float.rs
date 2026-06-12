use super::LayoutBox;
use crate::infra::NodeId;
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

/// Helper to get the computed float value of a style.
/// Returns Some("left"), Some("right"), or None.
///
/// // TODO(spec): DO NOT implement text/line-box shortening or wrapping content around the float.
/// // TODO(spec): DO NOT implement float stacking of multiple floats side-by-side beyond the basic left/right edge placement.
pub(crate) fn get_float_value(style: &CategorizedComputedStyle) -> Option<&str> {
    let fl = style.reset_box.float.as_str();
    if fl == "left" || fl == "right" {
        Some(fl)
    } else {
        None
    }
}

/// Helper to get the computed clear value of a style.
/// Returns Some("left"), Some("right"), Some("both"), or None.
pub(crate) fn get_clear_value(style: &CategorizedComputedStyle) -> Option<&str> {
    let cl = style.reset_box.clear.as_str();
    if cl == "left" || cl == "right" || cl == "both" {
        Some(cl)
    } else {
        None
    }
}

/// Computes the maximum bottom edge of the relevant active floats based on `clear_val`.
pub(crate) fn find_clearance_y(
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    clear_val: &str,
) -> Option<f32> {
    let mut max_float_y = None;
    let mut stack = Vec::new();
    for child in children {
        stack.push(child);
    }

    while let Some(current) = stack.pop() {
        if let Some(fv) = current
            .node
            .and_then(|node_id| styles.get(&node_id))
            .and_then(get_float_value)
        {
            let matches_side = match clear_val {
                "left" => fv == "left",
                "right" => fv == "right",
                "both" => fv == "left" || fv == "right",
                _ => false,
            };
            if matches_side {
                let bottom_edge = current.rect.max_y();
                max_float_y = Some(match max_float_y {
                    Some(y) => f32::max(y, bottom_edge),
                    None => bottom_edge,
                });
            }
        }
        for child in &current.children {
            stack.push(child);
        }
    }

    max_float_y
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

    #[test]
    fn test_clear_left_positions_below_left_float() {
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
                clear: left;
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

        // The left float is at y = 0, height = 50. So bottom is 50.
        // The normal_box has clear: left, so its top margin edge (offset_y)
        // should be pushed to at least 50.
        // Margin top of normal_box is 10.0, so normal_layout.rect.origin.y is 50.0 + 10.0 = 60.0.
        assert!(approx_eq(float_layout.rect.origin.y, 0.0));
        assert!(approx_eq(normal_layout.rect.origin.y, 60.0));
    }

    #[test]
    fn test_clear_none_leaves_sibling_at_normal_flow() {
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
                clear: none;
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

        // Sibling p should start at y = 10.0 (margin-top), because clear is none.
        assert!(approx_eq(float_layout.rect.origin.y, 0.0));
        assert!(approx_eq(normal_layout.rect.origin.y, 10.0));
    }

    #[test]
    fn test_clear_both_clears_left_and_right_floats() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "left".into())],
        });
        dom.append_child(body, left_float);

        let right_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "right".into())],
        });
        dom.append_child(body, right_float);

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
            .left {
                float: left;
                width: 100px;
                height: 50px;
            }
            .right {
                float: right;
                width: 100px;
                height: 80px;
            }
            p {
                display: block;
                clear: both;
                margin-top: 10px;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // Children of body: left_float (0), right_float (1), p (2)
        assert_eq!(body_box.children.len(), 3);
        let p_layout = &body_box.children[2];

        // The right float is at y = 0, height = 80, so max_y is 80.
        // The left float is at y = 0, height = 50, so max_y is 50.
        // clearance_y should be max(50, 80) = 80.
        // p has clear: both, margin-top: 10.
        // p_layout.rect.origin.y should be 80 + 10 = 90.
        assert!(approx_eq(p_layout.rect.origin.y, 90.0));
    }
}
