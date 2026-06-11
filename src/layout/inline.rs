use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::geom::{Point, Rect, Size};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

fn is_inline_block(styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> bool {
    if let Some(style) = styles.get(&node) {
        matches!(
            style.get("display"),
            Some(crate::css::values::CssValue::Display(
                crate::css::values::DisplayValue::InlineBlock
            ))
        ) || matches!(style.get("display"), Some(crate::css::values::CssValue::Keyword(kw)) if kw == "inline-block")
    } else {
        false
    }
}

fn shift_y(layout_box: &mut LayoutBox, delta: f32) {
    layout_box.rect.origin.y += delta;
    for child in &mut layout_box.children {
        shift_y(child, delta);
    }
}

fn shift_x(layout_box: &mut LayoutBox, delta: f32) {
    layout_box.rect.origin.x += delta;
    for child in &mut layout_box.children {
        shift_x(child, delta);
    }
}

#[allow(clippy::too_many_arguments)]
fn create_line_box_adjusted(
    mut children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    line_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
    text_align: &str,
    containing_width: f32,
) -> LayoutBox {
    // For each child, adjust its Y position to align its bottom edge with the bottom of the line box.
    let line_box_bottom_y = offset_y + line_height;

    for child in &mut children {
        if let Some(style) = child
            .node
            .filter(|&id| is_inline_block(styles, id))
            .and_then(|id| styles.get(&id))
        {
            let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);
            let border_box_height = child.rect.size.height;
            let target_y = line_box_bottom_y - margin_bottom - border_box_height;
            let delta = target_y - child.rect.origin.y;
            if delta != 0.0 {
                shift_y(child, delta);
            }
            continue;
        }
        let border_box_height = child.rect.size.height;
        let target_y = line_box_bottom_y - border_box_height;
        let delta = target_y - child.rect.origin.y;
        if delta != 0.0 {
            shift_y(child, delta);
        }
    }

    // Adjust X positions based on text-align centering/right alignment
    let delta_x = match text_align {
        "center" => ((containing_width - width) / 2.0).max(0.0),
        "right" => (containing_width - width).max(0.0),
        _ => 0.0,
    };

    if delta_x != 0.0 {
        for child in &mut children {
            shift_x(child, delta_x);
        }
    }

    LayoutBox {
        node: None,
        rect: Rect {
            origin: Point {
                x: offset_x,
                y: offset_y,
            },
            size: Size {
                width,
                height: line_height,
            },
        },
        children,
        text: None,
    }
}

/// Layout inline content from a slice of children, wrapping text and display: inline boxes.
///
/// spec: S-45
#[allow(clippy::too_many_arguments)]
pub fn layout_inline_run(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    children: &[NodeId],
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
    text_align: &str,
) -> (Vec<LayoutBox>, f32) {
    let font = crate::font::BitmapFont::builtin();
    let line_height = font.line_height() as f32;

    let mut line_boxes = Vec::new();
    let mut current_line_children = Vec::new();
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;
    let mut current_line_height = line_height;

    let mut stack: Vec<(NodeId, usize)> = Vec::new();
    // To preserve document order, push children in reverse order so the first child is processed first.
    for &child in children.iter().rev() {
        stack.push((child, depth));
    }

    while let Some((node, node_depth)) = stack.pop() {
        if node_depth > crate::layout::MAX_DEPTH {
            continue;
        }

        if let Some(data) = dom.data(node) {
            match data {
                NodeData::Text(text) => {
                    let collapsed = collapse_whitespace(text);
                    let words = collapsed.split_inclusive(' ');

                    for word in words {
                        // Measure the word with the font's measure helper.
                        // spec: S-45, S-57
                        let word_width = font.measure(word) as f32;

                        if cursor_x + word_width > containing_width && cursor_x > 0.0 {
                            // Flush current line
                            line_boxes.push(create_line_box_adjusted(
                                std::mem::take(&mut current_line_children),
                                offset_x,
                                offset_y + cursor_y,
                                cursor_x,
                                current_line_height,
                                styles,
                                text_align,
                                containing_width,
                            ));
                            cursor_x = 0.0;
                            cursor_y += current_line_height;
                            current_line_height = line_height;
                        }

                        // Skip leading whitespace on a new line
                        if cursor_x == 0.0 && word == " " {
                            continue;
                        }

                        // Add word to current line
                        current_line_children.push(LayoutBox {
                            node: Some(node),
                            rect: Rect {
                                origin: Point {
                                    x: offset_x + cursor_x,
                                    y: offset_y + cursor_y,
                                },
                                size: Size {
                                    width: word_width,
                                    height: line_height,
                                },
                            },
                            children: Vec::new(),
                            text: Some(word.to_string()),
                        });
                        cursor_x += word_width;
                    }
                }
                NodeData::Element { .. } => {
                    if let Some(style) = styles.get(&node) {
                        // Skip out-of-flow nodes
                        if crate::layout::is_absolute_or_fixed(styles, node) {
                            continue;
                        }
                        if is_inline_block(styles, node) {
                            // Lay out the inline-block box as an atomic inline element
                            let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
                            let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
                            let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
                            let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);

                            // Position initially at current line cursor
                            let mut box_ = match crate::layout::layout_node(
                                dom,
                                styles,
                                node,
                                containing_width,
                                offset_x + cursor_x,
                                offset_y + cursor_y,
                                node_depth + 1,
                            ) {
                                Some(b) => b,
                                None => continue,
                            };

                            let margin_box_width =
                                box_.rect.size.width + margin_left + margin_right;

                            // Check wrapping
                            if cursor_x + margin_box_width > containing_width && cursor_x > 0.0 {
                                // Flush line
                                line_boxes.push(create_line_box_adjusted(
                                    std::mem::take(&mut current_line_children),
                                    offset_x,
                                    offset_y + cursor_y,
                                    cursor_x,
                                    current_line_height,
                                    styles,
                                    text_align,
                                    containing_width,
                                ));
                                cursor_x = 0.0;
                                cursor_y += current_line_height;
                                current_line_height = line_height;

                                // Re-layout at the start of the new line
                                box_ = match crate::layout::layout_node(
                                    dom,
                                    styles,
                                    node,
                                    containing_width,
                                    offset_x + cursor_x,
                                    offset_y + cursor_y,
                                    node_depth + 1,
                                ) {
                                    Some(b) => b,
                                    None => continue,
                                };
                            }

                            // Update current line height to incorporate this inline-block box's margin height
                            let margin_box_height =
                                box_.rect.size.height + margin_top + margin_bottom;
                            current_line_height = current_line_height.max(margin_box_height);

                            // Add box to current line children
                            current_line_children.push(box_);
                            cursor_x += margin_box_width;
                        } else if matches!(style.get("display"), Some(crate::css::values::CssValue::Keyword(kw)) if kw == "inline")
                            || matches!(
                                style.get("display"),
                                Some(crate::css::values::CssValue::Display(
                                    crate::css::values::DisplayValue::Inline
                                ))
                            )
                        {
                            // Descend into grandchildren.
                            // To preserve order, push them to the stack in reverse order.
                            for &grandchild in dom.children(node).iter().rev() {
                                stack.push((grandchild, node_depth + 1));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if !current_line_children.is_empty() {
        line_boxes.push(create_line_box_adjusted(
            current_line_children,
            offset_x,
            offset_y + cursor_y,
            cursor_x,
            current_line_height,
            styles,
            text_align,
            containing_width,
        ));
        cursor_y += current_line_height;
    }

    (line_boxes, cursor_y)
}

/// Layout inline content for a single parent element, wrapping its children.
///
/// spec: S-45
#[allow(clippy::too_many_arguments)]
pub fn layout_inline(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
    text_align: &str,
) -> (Vec<LayoutBox>, f32) {
    layout_inline_run(
        dom,
        styles,
        dom.children(node),
        containing_width,
        offset_x,
        offset_y,
        depth,
        text_align,
    )
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut last_was_whitespace = false;

    for c in s.chars() {
        if is_html_whitespace(c) {
            if !last_was_whitespace {
                result.push(' ');
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::style::compute_styles;

    fn collect_leaf_texts(layout_box: &crate::layout::LayoutBox) -> Vec<String> {
        if layout_box.children.is_empty() {
            if let Some(ref text) = layout_box.text {
                vec![text.clone()]
            } else {
                vec![]
            }
        } else {
            let mut res = Vec::new();
            for child in &layout_box.children {
                res.extend(collect_leaf_texts(child));
            }
            res
        }
    }

    #[test]
    fn test_inline_nested_preserves_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello ".into()));
        dom.append_child(div, t1);

        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, span);

        let t2 = dom.create_node(NodeData::Text("nested inline".into()));
        dom.append_child(span, t2);

        let t3 = dom.create_node(NodeData::Text(" world".into()));
        dom.append_child(div, t3);

        let stylesheet = parse_stylesheet("span { display: inline; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 800.0, 0.0, 0.0, 0, "left");

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        assert_eq!(
            leaf_texts,
            vec!["hello ", "nested ", "inline", " ", "world"]
        );
    }

    #[test]
    fn test_inline_deeply_nested_no_overflow() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let mut current = div;
        for _ in 0..2000 {
            let span = dom.create_node(NodeData::Element {
                name: "span".into(),
                attrs: vec![],
            });
            dom.append_child(current, span);
            current = span;
        }

        let text = dom.create_node(NodeData::Text("deep".into()));
        dom.append_child(current, text);

        let stylesheet = parse_stylesheet("span { display: inline; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 800.0, 0.0, 0.0, 0, "left");
        // The call must complete without stack overflow.
        let _ = line_boxes;
    }
}
