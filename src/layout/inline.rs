use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::geom::{Point, Rect, Size};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

const CHAR_WIDTH: f32 = 8.0;
const LINE_HEIGHT: f32 = 16.0;

pub fn layout_inline(
    dom: &Dom,
    _styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
) -> (Vec<LayoutBox>, f32) {
    let mut line_boxes = Vec::new();
    let mut current_line_children = Vec::new();
    let mut cursor_x = 0.0;
    let mut cursor_y = 0.0;

    for &child in dom.children(node) {
        if let Some(NodeData::Text(text)) = dom.data(child) {
            let collapsed = collapse_whitespace(text);
            let words = collapsed.split_inclusive(' ');

            for word in words {
                let word_width = word.len() as f32 * CHAR_WIDTH;

                if cursor_x + word_width > containing_width && cursor_x > 0.0 {
                    // Flush current line
                    line_boxes.push(create_line_box(
                        current_line_children,
                        offset_x,
                        offset_y + cursor_y,
                        cursor_x,
                    ));
                    current_line_children = Vec::new();
                    cursor_x = 0.0;
                    cursor_y += LINE_HEIGHT;
                }

                // Skip leading whitespace on a new line
                if cursor_x == 0.0 && word == " " {
                    continue;
                }

                // Add word to current line
                current_line_children.push(LayoutBox {
                    node: Some(child),
                    rect: Rect {
                        origin: Point {
                            x: offset_x + cursor_x,
                            y: offset_y + cursor_y,
                        },
                        size: Size {
                            width: word_width,
                            height: LINE_HEIGHT,
                        },
                    },
                    children: Vec::new(),
                });
                cursor_x += word_width;
            }
        }
        // TODO(spec): Support display: inline elements
    }

    if !current_line_children.is_empty() {
        line_boxes.push(create_line_box(
            current_line_children,
            offset_x,
            offset_y + cursor_y,
            cursor_x,
        ));
        cursor_y += LINE_HEIGHT;
    }

    (line_boxes, cursor_y)
}

fn create_line_box(
    children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
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
                height: LINE_HEIGHT,
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
