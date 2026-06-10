use crate::css::values::{AlignItemsValue, CssValue, JustifyContentValue};
use crate::dom::Dom;
use crate::geom::{Point, Rect};
use crate::infra::NodeId;
use crate::layout::{LayoutBox, get_px, is_absolute_or_fixed, layout_node};
use crate::style::ComputedStyle;
use std::collections::HashMap;

pub fn layout_flex_container(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> Option<LayoutBox> {
    let style = styles.get(&node)?;

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

    // Flex properties
    let flex_direction = match style.get("flex-direction") {
        Some(CssValue::Keyword(kw)) if kw == "column" => FlexDirection::Column,
        _ => FlexDirection::Row,
    };

    let justify_content = match style.get("justify-content") {
        Some(CssValue::JustifyContent(val)) => match val {
            JustifyContentValue::FlexStart => JustifyContent::FlexStart,
            JustifyContentValue::FlexEnd => JustifyContent::FlexEnd,
            JustifyContentValue::Center => JustifyContent::Center,
            JustifyContentValue::SpaceBetween => JustifyContent::SpaceBetween,
            JustifyContentValue::SpaceAround => JustifyContent::SpaceAround,
            JustifyContentValue::SpaceEvenly => JustifyContent::SpaceEvenly,
        },
        Some(CssValue::Keyword(kw)) => match kw.as_str() {
            "flex-start" => JustifyContent::FlexStart,
            "flex-end" => JustifyContent::FlexEnd,
            "center" => JustifyContent::Center,
            "space-between" => JustifyContent::SpaceBetween,
            "space-around" => JustifyContent::SpaceAround,
            "space-evenly" => JustifyContent::SpaceEvenly,
            _ => JustifyContent::FlexStart,
        },
        _ => JustifyContent::FlexStart,
    };

    let align_items = match style.get("align-items") {
        Some(CssValue::AlignItems(val)) => match val {
            AlignItemsValue::Stretch => AlignItems::Stretch,
            AlignItemsValue::FlexStart => AlignItems::FlexStart,
            AlignItemsValue::FlexEnd => AlignItems::FlexEnd,
            AlignItemsValue::Center => AlignItems::Center,
            AlignItemsValue::Baseline => AlignItems::Baseline,
        },
        Some(CssValue::Keyword(kw)) => match kw.as_str() {
            "stretch" => AlignItems::Stretch,
            "flex-start" => AlignItems::FlexStart,
            "flex-end" => AlignItems::FlexEnd,
            "center" => AlignItems::Center,
            "baseline" => AlignItems::Baseline,
            _ => AlignItems::Stretch,
        },
        _ => AlignItems::Stretch,
    };

    let flex_wrap = match style.get("flex-wrap") {
        Some(CssValue::Keyword(kw)) if kw == "wrap" => FlexWrap::Wrap,
        Some(CssValue::Keyword(kw)) if kw == "wrap-reverse" => {
            // TODO(spec): wrap-reverse is OUT of scope
            FlexWrap::Nowrap
        }
        _ => FlexWrap::Nowrap,
    };

    // 1. Layout children to determine their base sizes.
    // For now, we layout them as blocks to get their natural height/width.
    let mut children = Vec::new();
    let inner_x = border_box_x + border_left + padding_left;
    let inner_y = border_box_y + border_top + padding_top;

    for &child in dom.children(node) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }
        if let Some(child_box) = layout_node(
            dom,
            styles,
            child,
            content_width,
            inner_x,
            inner_y,
            depth + 1,
        ) {
            children.push(child_box);
        }
    }

    // 2. Distribute free space along the main axis.
    let (main_size, _cross_size) = match flex_direction {
        FlexDirection::Row => (content_width, get_px(style, "height", 0.0)),
        FlexDirection::Column => (get_px(style, "height", 0.0), content_width),
    };

    // Group children into lines based on flex_wrap
    struct FlexLine {
        children: Vec<LayoutBox>,
    }

    let mut lines = Vec::new();
    if flex_wrap == FlexWrap::Nowrap || children.is_empty() {
        lines.push(FlexLine { children });
    } else {
        let mut current_line = FlexLine {
            children: Vec::new(),
        };
        let mut current_line_main_size = 0.0;

        for child in children {
            let child_main_size = match flex_direction {
                FlexDirection::Row => child.rect.size.width,
                FlexDirection::Column => child.rect.size.height,
            };

            if !current_line.children.is_empty()
                && current_line_main_size + child_main_size > main_size
            {
                lines.push(current_line);
                current_line = FlexLine {
                    children: Vec::new(),
                };
                current_line_main_size = 0.0;
            }

            current_line_main_size += child_main_size;
            current_line.children.push(child);
        }

        if !current_line.children.is_empty() {
            lines.push(current_line);
        }
    }

    // Distribute free space along the main axis for each line separately (flex-grow)
    for line in &mut lines {
        let mut total_line_main_size = 0.0;
        let mut total_line_flex_grow = 0.0;

        for child_box in &line.children {
            if let Some(child_style) = child_box.node.and_then(|id| styles.get(&id)) {
                total_line_main_size += match flex_direction {
                    FlexDirection::Row => child_box.rect.size.width,
                    FlexDirection::Column => child_box.rect.size.height,
                };
                total_line_flex_grow += get_number(child_style, "flex-grow", 0.0);
            }
        }

        let line_free_space = (main_size - total_line_main_size).max(0.0);

        if line_free_space > 0.0 && total_line_flex_grow > 0.0 {
            for child_box in &mut line.children {
                if let Some(child_style) = child_box.node.and_then(|id| styles.get(&id)) {
                    let grow = get_number(child_style, "flex-grow", 0.0);
                    let extra = (grow / total_line_flex_grow) * line_free_space;
                    match flex_direction {
                        FlexDirection::Row => child_box.rect.size.width += extra,
                        FlexDirection::Column => child_box.rect.size.height += extra,
                    }
                }
            }
        }
    }

    // Calculate cross size and total main size for each line after flex-grow
    let mut line_max_cross_sizes = Vec::new();
    let mut line_total_main_sizes = Vec::new();

    for line in &lines {
        let mut total_main_size = 0.0;
        let mut max_child_cross_size: f32 = 0.0;
        for child_box in &line.children {
            match flex_direction {
                FlexDirection::Row => {
                    total_main_size += child_box.rect.size.width;
                    max_child_cross_size = max_child_cross_size.max(child_box.rect.size.height);
                }
                FlexDirection::Column => {
                    total_main_size += child_box.rect.size.height;
                    max_child_cross_size = max_child_cross_size.max(child_box.rect.size.width);
                }
            }
        }
        line_max_cross_sizes.push(max_child_cross_size);
        line_total_main_sizes.push(total_main_size);
    }

    let sum_of_each_line_max_cross_size: f32 = line_max_cross_sizes.iter().sum();
    let container_cross_size = match flex_direction {
        FlexDirection::Row => get_px(style, "height", sum_of_each_line_max_cross_size)
            .max(sum_of_each_line_max_cross_size),
        FlexDirection::Column => content_width.max(sum_of_each_line_max_cross_size),
    };

    // Calculate cross offsets for each line
    let mut line_cross_offsets = Vec::new();
    let mut current_offset = 0.0;
    for &size in &line_max_cross_sizes {
        line_cross_offsets.push(current_offset);
        current_offset += size;
    }

    // TODO(spec): align-content is OUT of scope
    let num_lines = lines.len();
    let mut positioned_children = Vec::new();

    for (line_idx, mut line) in lines.into_iter().enumerate() {
        let line_max_cross_size = line_max_cross_sizes[line_idx];
        let line_total_main_size = line_total_main_sizes[line_idx];
        let line_cross_offset_base = line_cross_offsets[line_idx];

        let line_cross_size = if num_lines == 1 {
            container_cross_size
        } else {
            line_max_cross_size
        };

        // 3. Position children inside this line
        let (mut main_cursor, spacing) = match justify_content {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (main_size - line_total_main_size, 0.0),
            JustifyContent::Center => ((main_size - line_total_main_size) / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                let spacing = if line.children.len() > 1 {
                    ((main_size - line_total_main_size) / (line.children.len() - 1) as f32).max(0.0)
                } else {
                    0.0
                };
                (0.0, spacing)
            }
            JustifyContent::SpaceAround => {
                if line.children.is_empty() {
                    (0.0, 0.0)
                } else if line.children.len() == 1 || main_size < line_total_main_size {
                    ((main_size - line_total_main_size) / 2.0, 0.0)
                } else {
                    let free_space = main_size - line_total_main_size;
                    let spacing = free_space / line.children.len() as f32;
                    (spacing / 2.0, spacing)
                }
            }
            JustifyContent::SpaceEvenly => {
                if line.children.is_empty() {
                    (0.0, 0.0)
                } else if line.children.len() == 1 || main_size < line_total_main_size {
                    ((main_size - line_total_main_size) / 2.0, 0.0)
                } else {
                    let free_space = main_size - line_total_main_size;
                    let spacing = free_space / (line.children.len() + 1) as f32;
                    (spacing, spacing)
                }
            }
        };

        for child_box in &mut line.children {
            let child_style = child_box.node.and_then(|id| styles.get(&id));

            let child_cross_size = match flex_direction {
                FlexDirection::Row => child_box.rect.size.height,
                FlexDirection::Column => child_box.rect.size.width,
            };

            let cross_offset = match align_items {
                AlignItems::FlexStart => 0.0,
                AlignItems::FlexEnd => line_cross_size - child_cross_size,
                AlignItems::Center => (line_cross_size - child_cross_size) / 2.0,
                AlignItems::Stretch => {
                    let has_explicit = match flex_direction {
                        FlexDirection::Row => has_explicit_size(child_style, "height"),
                        FlexDirection::Column => has_explicit_size(child_style, "width"),
                    };
                    if !has_explicit {
                        match flex_direction {
                            FlexDirection::Row => child_box.rect.size.height = line_cross_size,
                            FlexDirection::Column => child_box.rect.size.width = line_cross_size,
                        }
                    }
                    0.0
                }
                AlignItems::Baseline => {
                    // TODO(spec): True baseline alignment is not yet available.
                    // Map baseline to flex-start for now.
                    0.0
                }
            };

            let target_origin = match flex_direction {
                FlexDirection::Row => Point {
                    x: inner_x + main_cursor,
                    y: inner_y + line_cross_offset_base + cross_offset,
                },
                FlexDirection::Column => Point {
                    x: inner_x + line_cross_offset_base + cross_offset,
                    y: inner_y + main_cursor,
                },
            };

            let dx = target_origin.x - child_box.rect.origin.x;
            let dy = target_origin.y - child_box.rect.origin.y;

            crate::layout::position::shift_layout_box(child_box, styles, dx, dy, depth);

            match flex_direction {
                FlexDirection::Row => {
                    main_cursor += child_box.rect.size.width + spacing;
                }
                FlexDirection::Column => {
                    main_cursor += child_box.rect.size.height + spacing;
                }
            }
        }

        positioned_children.extend(line.children);
    }

    let max_line_total_main_size = line_total_main_sizes.iter().cloned().fold(0.0f32, f32::max);
    let border_box_height = match flex_direction {
        FlexDirection::Row => container_cross_size,
        FlexDirection::Column => get_px(style, "height", max_line_total_main_size),
    } + padding_top
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
        children: positioned_children,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlexWrap {
    Nowrap,
    Wrap,
}

fn get_number(style: &ComputedStyle, prop: &str, default: f32) -> f32 {
    match style.get(prop) {
        Some(CssValue::Number(v)) => *v,
        _ => default,
    }
}

fn has_explicit_size(style: Option<&ComputedStyle>, prop: &str) -> bool {
    let Some(style) = style else {
        return false;
    };
    match style.get(prop) {
        Some(CssValue::Length(_, _)) => true,
        Some(CssValue::Keyword(kw)) if kw != "auto" => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::style::compute_styles;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_flex_row_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        for i in 0..3 {
            let child = dom.create_node(NodeData::Element {
                name: "div".into(),
                attrs: vec![("id".into(), format!("child{}", i))],
            });
            dom.append_child(container, child);
        }

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                width: 300px;
            }
            div {
                width: 50px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 3);
        // child0: x=0, y=0, w=50, h=50
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.size.width, 50.0));

        // child1: x=50, y=0, w=50, h=50
        assert!(approx_eq(container_box.children[1].rect.origin.x, 50.0));
        assert!(approx_eq(container_box.children[1].rect.size.width, 50.0));

        // child2: x=100, y=0, w=50, h=50
        assert!(approx_eq(container_box.children[2].rect.origin.x, 100.0));
        assert!(approx_eq(container_box.children[2].rect.size.width, 50.0));
    }

    #[test]
    fn test_flex_grow() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child1".into())],
        });
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        dom.append_child(container, child1);
        dom.append_child(container, child2);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
            }
            #child1 {
                width: 50px;
                flex-grow: 1;
            }
            #child2 {
                width: 50px;
                flex-grow: 2;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Total width: 300. Base widths: 50+50=100. Free space: 200.
        // Total grow: 1+2=3.
        // Child 1: 50 + (1/3)*200 = 50 + 66.666 = 116.666
        // Child 2: 50 + (2/3)*200 = 50 + 133.333 = 183.333
        assert!(approx_eq(
            container_box.children[0].rect.size.width,
            116.66667
        ));
        assert!(approx_eq(
            container_box.children[1].rect.size.width,
            183.33334
        ));
    }

    #[test]
    fn test_justify_content_center() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
                justify-content: center;
            }
            div {
                width: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // (300 - 100) / 2 = 100 offset
        assert!(approx_eq(container_box.children[0].rect.origin.x, 100.0));
    }

    #[test]
    fn test_align_items_center() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: center;
            }
            div {
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // (200 - 50) / 2 = 75 offset
        assert!(approx_eq(container_box.children[0].rect.origin.y, 75.0));
    }

    #[test]
    fn test_flex_direction_column() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        for _ in 0..2 {
            let child = dom.create_node(NodeData::Element {
                name: "div".into(),
                attrs: vec![],
            });
            dom.append_child(container, child);
        }

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: column;
            }
            div {
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 50.0));
    }

    #[test]
    fn test_descendant_position_shifting() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child".into())],
        });
        dom.append_child(container, child);

        let grandchild = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "grandchild".into())],
        });
        dom.append_child(child, grandchild);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
                justify-content: center; /* moves child to x = 100 */
            }
            #child {
                width: 100px;
                height: 50px;
            }
            #grandchild {
                width: 50px;
                height: 20px;
                margin-left: 10px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Check container
        assert_eq!(container_box.children.len(), 1);
        let child_box = &container_box.children[0];
        // Child should be shifted to x = 100 (due to centering)
        assert!(approx_eq(child_box.rect.origin.x, 100.0));

        // Grandchild should be shifted recursively as well
        assert_eq!(child_box.children.len(), 1);
        let grandchild_box = &child_box.children[0];
        // Grandchild original offset was inner_x + child_margin_left = 10px.
        // It should now be shifted to 100 + 10 = 110px.
        assert!(approx_eq(grandchild_box.rect.origin.x, 110.0));
    }

    #[test]
    fn test_align_items_stretch_with_explicit_size() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child1".into())],
        });
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        dom.append_child(container, child1);
        dom.append_child(container, child2);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: stretch;
            }
            #child1 {
                height: 50px; /* has explicit height, shouldn't stretch */
            }
            #child2 {
                /* has auto height, should stretch to 200 */
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 height should remain 50
        assert!(approx_eq(container_box.children[0].rect.size.height, 50.0));
        // child2 height should stretch to 200
        assert!(approx_eq(container_box.children[1].rect.size.height, 200.0));
    }

    #[test]
    fn test_column_container_explicit_height() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: column;
                height: 300px;
                justify-content: center; /* centers items inside 300px height */
            }
            div {
                height: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Container height should be 300px
        assert!(approx_eq(container_box.rect.size.height, 300.0));

        // Child should be vertically centered: (300 - 100) / 2 = 100px offset
        assert_eq!(container_box.children.len(), 1);
        assert!(approx_eq(container_box.children[0].rect.origin.y, 100.0));
    }

    #[test]
    fn test_flex_wrap_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        for i in 0..4 {
            let child = dom.create_node(NodeData::Element {
                name: "div".into(),
                attrs: vec![("id".into(), format!("child{}", i))],
            });
            dom.append_child(container, child);
        }

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                flex-wrap: wrap;
                width: 300px;
            }
            div {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 4);

        // Line 1: child0, child1, child2
        // child0: x=0, y=0, w=100, h=50
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[0].rect.size.width, 100.0));

        // child1: x=100, y=0, w=100, h=50
        assert!(approx_eq(container_box.children[1].rect.origin.x, 100.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 0.0));

        // child2: x=200, y=0, w=100, h=50
        assert!(approx_eq(container_box.children[2].rect.origin.x, 200.0));
        assert!(approx_eq(container_box.children[2].rect.origin.y, 0.0));

        // Line 2: child3
        // child3: x=0, y=50, w=100, h=50
        assert!(approx_eq(container_box.children[3].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[3].rect.origin.y, 50.0));
        assert!(approx_eq(container_box.children[3].rect.size.width, 100.0));

        // Container height: 50 + 50 = 100px
        assert!(approx_eq(container_box.rect.size.height, 100.0));
    }

    #[test]
    fn test_flex_wrap_grow_per_line() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child1".into())],
        });
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        let child3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child3".into())],
        });
        let child4 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child4".into())],
        });
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);
        dom.append_child(container, child4);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-wrap: wrap;
                width: 300px;
            }
            #child1 {
                width: 100px;
                flex-grow: 1;
            }
            #child2 {
                width: 100px;
                flex-grow: 2;
            }
            #child3 {
                width: 200px;
                flex-grow: 1;
            }
            #child4 {
                width: 50px;
                flex-grow: 1;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 4);

        // Line 1: child1, child2
        // Free space = 300 - 200 = 100
        // child1: 100 + 1/3 * 100 = 133.33333
        // child2: 100 + 2/3 * 100 = 166.66667
        assert!(approx_eq(
            container_box.children[0].rect.size.width,
            133.33333
        ));
        assert!(approx_eq(
            container_box.children[1].rect.size.width,
            166.66667
        ));

        // Line 2: child3, child4
        // Free space = 300 - 250 = 50
        // child3: 200 + 1/2 * 50 = 225
        // child4: 50 + 1/2 * 50 = 75
        assert!(approx_eq(container_box.children[2].rect.size.width, 225.0));
        assert!(approx_eq(container_box.children[3].rect.size.width, 75.0));
    }

    #[test]
    fn test_justify_content_flex_end() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
                justify-content: flex-end;
            }
            div {
                width: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // (300 - 100) = 200 offset
        assert!(approx_eq(container_box.children[0].rect.origin.x, 200.0));
    }

    #[test]
    fn test_justify_content_space_around() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child1);
        dom.append_child(container, child2);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
                justify-content: space-around;
            }
            div {
                width: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 starts at 50px
        assert!(approx_eq(container_box.children[0].rect.origin.x, 50.0));
        // child2 starts at 200px
        assert!(approx_eq(container_box.children[1].rect.origin.x, 200.0));
    }

    #[test]
    fn test_justify_content_space_evenly() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child1);
        dom.append_child(container, child2);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                width: 300px;
                justify-content: space-evenly;
            }
            div {
                width: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 starts at 66.66667
        assert!(approx_eq(container_box.children[0].rect.origin.x, 66.66667));
        // child2 starts at 183.33333
        assert!(approx_eq(
            container_box.children[1].rect.origin.x,
            183.33333
        ));
    }

    #[test]
    fn test_align_items_flex_end() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: flex-end;
            }
            div {
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // (200 - 50) = 150 offset
        assert!(approx_eq(container_box.children[0].rect.origin.y, 150.0));
    }

    #[test]
    fn test_align_items_baseline_fallback() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, child);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: baseline;
            }
            div {
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Mapped to flex-start: 0.0 offset
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
    }
}
