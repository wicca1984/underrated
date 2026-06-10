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

fn create_line_box_adjusted(
    mut children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    line_height: f32,
    styles: &HashMap<NodeId, ComputedStyle>,
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
    }
}

/// Layout inline content from a slice of children, wrapping text and display: inline boxes.
///
/// spec: S-45
pub fn layout_inline_run(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    children: &[NodeId],
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> (Vec<LayoutBox>, f32) {
    let font = crate::font::BitmapFont::builtin();
    let line_height = font.line_height() as f32;

    let mut line_boxes = Vec::new();
    let mut current_line_children = Vec::new();
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;
    let mut current_line_height = line_height;

    for &child in children {
        layout_inline_child_recursive(
            dom,
            styles,
            child,
            containing_width,
            offset_x,
            offset_y,
            &mut cursor_x,
            &mut cursor_y,
            &mut current_line_children,
            &mut line_boxes,
            &font,
            line_height,
            &mut current_line_height,
            depth,
        );
    }

    if !current_line_children.is_empty() {
        line_boxes.push(create_line_box_adjusted(
            current_line_children,
            offset_x,
            offset_y + cursor_y,
            cursor_x,
            current_line_height,
            styles,
        ));
        cursor_y += current_line_height;
    }

    (line_boxes, cursor_y)
}

/// Layout inline content for a single parent element, wrapping its children.
///
/// spec: S-45
pub fn layout_inline(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> (Vec<LayoutBox>, f32) {
    layout_inline_run(
        dom,
        styles,
        dom.children(node),
        containing_width,
        offset_x,
        offset_y,
        depth,
    )
}

#[allow(clippy::too_many_arguments)]
fn layout_inline_child_recursive(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    child: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    cursor_x: &mut f32,
    cursor_y: &mut f32,
    current_line_children: &mut Vec<LayoutBox>,
    line_boxes: &mut Vec<LayoutBox>,
    font: &crate::font::BitmapFont,
    line_height: f32,
    current_line_height: &mut f32,
    depth: usize,
) {
    if depth > crate::layout::MAX_DEPTH {
        return;
    }

    if let Some(data) = dom.data(child) {
        match data {
            NodeData::Text(text) => {
                let collapsed = collapse_whitespace(text);
                let words = collapsed.split_inclusive(' ');

                for word in words {
                    // Measure the word with the font's measure helper.
                    // spec: S-45, S-57
                    let word_width = font.measure(word) as f32;

                    if *cursor_x + word_width > containing_width && *cursor_x > 0.0 {
                        // Flush current line
                        line_boxes.push(create_line_box_adjusted(
                            std::mem::take(current_line_children),
                            offset_x,
                            offset_y + *cursor_y,
                            *cursor_x,
                            *current_line_height,
                            styles,
                        ));
                        *cursor_x = 0.0;
                        *cursor_y += *current_line_height;
                        *current_line_height = line_height;
                    }

                    // Skip leading whitespace on a new line
                    if *cursor_x == 0.0 && word == " " {
                        continue;
                    }

                    // Add word to current line
                    current_line_children.push(LayoutBox {
                        node: Some(child),
                        rect: Rect {
                            origin: Point {
                                x: offset_x + *cursor_x,
                                y: offset_y + *cursor_y,
                            },
                            size: Size {
                                width: word_width,
                                height: line_height,
                            },
                        },
                        children: Vec::new(),
                    });
                    *cursor_x += word_width;
                }
            }
            NodeData::Element { .. } => {
                if let Some(style) = styles.get(&child) {
                    // Skip out-of-flow nodes
                    if crate::layout::is_absolute_or_fixed(styles, child) {
                        return;
                    }
                    if is_inline_block(styles, child) {
                        // Lay out the inline-block box as an atomic inline element
                        let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
                        let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
                        let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
                        let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);

                        // Position initially at current line cursor
                        let mut box_ = match crate::layout::layout_node(
                            dom,
                            styles,
                            child,
                            containing_width,
                            offset_x + *cursor_x,
                            offset_y + *cursor_y,
                            depth + 1,
                        ) {
                            Some(b) => b,
                            None => return,
                        };

                        let margin_box_width = box_.rect.size.width + margin_left + margin_right;

                        // Check wrapping
                        if *cursor_x + margin_box_width > containing_width && *cursor_x > 0.0 {
                            // Flush line
                            line_boxes.push(create_line_box_adjusted(
                                std::mem::take(current_line_children),
                                offset_x,
                                offset_y + *cursor_y,
                                *cursor_x,
                                *current_line_height,
                                styles,
                            ));
                            *cursor_x = 0.0;
                            *cursor_y += *current_line_height;
                            *current_line_height = line_height;

                            // Re-layout at the start of the new line
                            box_ = match crate::layout::layout_node(
                                dom,
                                styles,
                                child,
                                containing_width,
                                offset_x + *cursor_x,
                                offset_y + *cursor_y,
                                depth + 1,
                            ) {
                                Some(b) => b,
                                None => return,
                            };
                        }

                        // Update current line height to incorporate this inline-block box's margin height
                        let margin_box_height = box_.rect.size.height + margin_top + margin_bottom;
                        *current_line_height = current_line_height.max(margin_box_height);

                        // Add box to current line children
                        current_line_children.push(box_);
                        *cursor_x += margin_box_width;
                    } else if matches!(style.get("display"), Some(crate::css::values::CssValue::Keyword(kw)) if kw == "inline")
                        || matches!(
                            style.get("display"),
                            Some(crate::css::values::CssValue::Display(
                                crate::css::values::DisplayValue::Inline
                            ))
                        )
                    {
                        for &grandchild in dom.children(child) {
                            layout_inline_child_recursive(
                                dom,
                                styles,
                                grandchild,
                                containing_width,
                                offset_x,
                                offset_y,
                                cursor_x,
                                cursor_y,
                                current_line_children,
                                line_boxes,
                                font,
                                line_height,
                                current_line_height,
                                depth + 1,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
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
