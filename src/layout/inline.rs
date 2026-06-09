use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::geom::{Point, Rect, Size};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// Layout inline content, wrapping text and display: inline boxes.
///
/// spec: S-45
pub fn layout_inline(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
) -> (Vec<LayoutBox>, f32) {
    let font = crate::font::BitmapFont::builtin();
    let line_height = font.line_height() as f32;

    let mut line_boxes = Vec::new();
    let mut current_line_children = Vec::new();
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;

    layout_inline_recursive(
        dom,
        styles,
        node,
        containing_width,
        offset_x,
        offset_y,
        &mut cursor_x,
        &mut cursor_y,
        &mut current_line_children,
        &mut line_boxes,
        &font,
        line_height,
    );

    if !current_line_children.is_empty() {
        line_boxes.push(create_line_box(
            current_line_children,
            offset_x,
            offset_y + cursor_y,
            cursor_x,
            line_height,
        ));
        cursor_y += line_height;
    }

    (line_boxes, cursor_y)
}

#[allow(clippy::too_many_arguments)]
fn layout_inline_recursive(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    cursor_x: &mut f32,
    cursor_y: &mut f32,
    current_line_children: &mut Vec<LayoutBox>,
    line_boxes: &mut Vec<LayoutBox>,
    font: &crate::font::BitmapFont,
    line_height: f32,
) {
    for &child in dom.children(node) {
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Text(text) => {
                    let collapsed = collapse_whitespace(text);
                    let words = collapsed.split_inclusive(' ');

                    for word in words {
                        // Measure each character in the word with the font glyph width.
                        // spec: S-45
                        let word_width: f32 =
                            word.chars().map(|c| font.glyph_width(c) as f32).sum();

                        if *cursor_x + word_width > containing_width && *cursor_x > 0.0 {
                            // Flush current line
                            line_boxes.push(create_line_box(
                                std::mem::take(current_line_children),
                                offset_x,
                                offset_y + *cursor_y,
                                *cursor_x,
                                line_height,
                            ));
                            *cursor_x = 0.0;
                            *cursor_y += line_height;
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
                            continue;
                        }
                        // If display: inline, recurse to lay out its inline children
                        // spec: S-45
                        if matches!(style.get("display"), Some(crate::css::values::CssValue::Keyword(kw)) if kw == "inline")
                        {
                            layout_inline_recursive(
                                dom,
                                styles,
                                child,
                                containing_width,
                                offset_x,
                                offset_y,
                                cursor_x,
                                cursor_y,
                                current_line_children,
                                line_boxes,
                                font,
                                line_height,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn create_line_box(
    children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    line_height: f32,
) -> LayoutBox {
    LayoutBox {
        node: None, // Anonymous line box
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
