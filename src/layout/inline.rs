use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::geom::{Point, Rect, Size};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::CategorizedCategorizedComputedStyle;
use std::collections::HashMap;

fn is_inline_block(styles: &HashMap<NodeId, CategorizedComputedStyle>, node: NodeId) -> bool {
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

fn get_font_size(style: &CategorizedComputedStyle) -> f32 {
    match style.get("font-size") {
        Some(crate::css::values::CssValue::Length(px, _)) => *px,
        _ => 16.0,
    }
}

#[allow(clippy::collapsible_if)]
fn get_vertical_align_shift(
    node: NodeId,
    block_container: Option<NodeId>,
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    line_height: f32,
    border_box_height: f32,
) -> f32 {
    let mut current = Some(node);
    let mut total_shift = 0.0;

    while let Some(curr_node) = current {
        if Some(curr_node) == block_container {
            break;
        }

        if let Some(style) = styles.get(&curr_node) {
            if let Some(val) = style.get("vertical-align") {
                match val {
                    crate::css::values::CssValue::Keyword(kw) => {
                        let font_size = get_font_size(style);
                        let shift = match kw.as_str() {
                            "baseline" => 0.0,
                            "sub" => 0.2 * font_size,
                            "super" => -0.2 * font_size,
                            "text-top" | "top" => -line_height + border_box_height,
                            "text-bottom" | "bottom" => 0.0,
                            "middle" => -0.25 * font_size + (border_box_height / 2.0),
                            _ => {
                                // TODO(spec): <percentage> and <length> vertical-align values and precise font-metric-based x-height/text-top/text-bottom are out of scope for v1
                                0.0
                            }
                        };
                        total_shift += shift;
                    }
                    crate::css::values::CssValue::Length(v, unit) => {
                        let raise = match unit {
                            crate::css::values::LengthUnit::Px
                            | crate::css::values::LengthUnit::Pt => *v,
                            crate::css::values::LengthUnit::Em => *v * get_font_size(style),
                            crate::css::values::LengthUnit::Rem => {
                                // NOTE: Approximate rem to em as the layout engine has no separate root font-size plumbing here
                                *v * get_font_size(style)
                            }
                            crate::css::values::LengthUnit::Percent => (*v / 100.0) * line_height,
                            crate::css::values::LengthUnit::Vw
                            | crate::css::values::LengthUnit::Vh => {
                                // TODO(spec): viewport units vertical-align are out of scope
                                0.0
                            }
                        };
                        total_shift += -raise;
                    }
                    _ => {}
                }
            }
        }

        current = dom.parent(curr_node);
    }

    total_shift
}

#[allow(clippy::too_many_arguments)]
fn create_line_box_adjusted(
    dom: &Dom,
    block_container: Option<NodeId>,
    mut children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    line_height: f32,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    text_align: &str,
    containing_width: f32,
    is_last_line: bool,
) -> LayoutBox {
    // For each child, adjust its Y position to align its bottom edge with the bottom of the line box.
    let line_box_bottom_y = offset_y + line_height;

    for child in &mut children {
        let mut target_y;
        let border_box_height = child.rect.size.height;

        if let Some(style) = child
            .node
            .filter(|&id| is_inline_block(styles, id))
            .and_then(|id| styles.get(&id))
        {
            let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);
            target_y = line_box_bottom_y - margin_bottom - border_box_height;
        } else {
            target_y = line_box_bottom_y - border_box_height;
        }

        // Apply vertical-align shift
        if let Some(node_id) = child.node {
            let shift = get_vertical_align_shift(
                node_id,
                block_container,
                dom,
                styles,
                line_height,
                border_box_height,
            );
            target_y += shift;
        }

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

    // TODO(spec): text-align justify v1 — distributes slack across inter-word gaps on non-last lines only; last-line/forced-break detection is simple word-count based; RTL, percentage widths, hyphenation, and justify-by-character are out of scope.
    if text_align == "justify" && !is_last_line && children.len() >= 2 {
        let slack = containing_width - width;
        if slack > 0.0 {
            let n = children.len();
            let gap_increment = slack / (n - 1) as f32;
            for (i, child) in children.iter_mut().enumerate() {
                let shift = (i as f32) * gap_increment;
                if shift > 0.0 {
                    shift_x(child, shift);
                }
            }
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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    children: &[NodeId],
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
    text_align: &str,
    text_indent: f32,
    word_spacing: f32,
) -> (Vec<LayoutBox>, f32) {
    let font = crate::font::BitmapFont::builtin();
    let line_height = font.line_height() as f32;

    let block_container = children.first().and_then(|&child| dom.parent(child));

    let mut line_boxes = Vec::new();
    let mut current_line_children = Vec::new();
    // TODO(spec): text-indent interaction with text-align (center/right/justify) and RTL, and percentage text-indent resolution, are out of scope; only length values shift the first-line start.
    let mut cursor_x = text_indent;
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
                    let mut node_line_height = line_height;
                    if let Some(style) = styles.get(&node) {
                        match style.get("line-height") {
                            Some(crate::css::values::CssValue::Length(px, _)) => {
                                node_line_height = *px;
                            }
                            Some(crate::css::values::CssValue::Number(n)) => {
                                let font_size = get_font_size(style);
                                node_line_height = n * font_size;
                                // TODO(spec): confirm line-height number resolution locus
                            }
                            _ => {}
                        }
                    }
                    current_line_height = current_line_height.max(node_line_height);

                    let style_ws = if let Some(style) = styles.get(&node) {
                        style.get("white-space")
                    } else {
                        None
                    };

                    let (collapse, preserve_newlines, allow_wrap) = match style_ws {
                        Some(crate::css::values::CssValue::Keyword(kw)) => match kw.as_str() {
                            "nowrap" => (true, false, false),
                            "pre" => (false, true, false),
                            "pre-wrap" => (false, true, true),
                            "pre-line" => (true, true, true),
                            _ => (true, false, true),
                        },
                        _ => (true, false, true),
                    };

                    let style_wb = if let Some(style) = styles.get(&node) {
                        style.get("word-break")
                    } else {
                        None
                    };

                    let break_all = match style_wb {
                        Some(crate::css::values::CssValue::Keyword(kw)) => {
                            kw.as_str() == "break-all"
                        }
                        _ => false,
                    };

                    let mut current_node = Some(node);
                    let mut overflow_wrap_val = None;
                    while let Some(n) = current_node {
                        if let Some(style) = styles.get(&n) {
                            if let Some(val) = style.get("overflow-wrap") {
                                overflow_wrap_val = Some(val);
                                break;
                            } else if let Some(val) = style.get("word-wrap") {
                                overflow_wrap_val = Some(val);
                                break;
                            }
                        }
                        current_node = dom.parent(n);
                    }

                    // TODO(spec): overflow-wrap: anywhere affects min-content sizing, which is out of scope.
                    // Only `break-word` and `normal` are implemented here.
                    let break_word = match overflow_wrap_val {
                        Some(crate::css::values::CssValue::Keyword(kw)) => {
                            kw.as_str() == "break-word"
                        }
                        _ => false,
                    };

                    let preprocessed = preprocess_text(text, collapse, preserve_newlines);

                    let transformed = if let Some(style) = styles.get(&node) {
                        if let Some(crate::css::values::CssValue::Keyword(kw)) =
                            style.get("text-transform")
                        {
                            apply_text_transform(&preprocessed, &kw.to_ascii_lowercase())
                        } else {
                            preprocessed
                        }
                    } else {
                        preprocessed
                    };

                    // spec: CSS Text Module Level 3, §3 (White Space Processing)
                    let segments: Vec<&str> = transformed.split('\n').collect();

                    for (i, segment) in segments.iter().enumerate() {
                        if i > 0 {
                            // Force a line break!
                            line_boxes.push(create_line_box_adjusted(
                                dom,
                                block_container,
                                std::mem::take(&mut current_line_children),
                                offset_x,
                                offset_y + cursor_y,
                                cursor_x,
                                current_line_height,
                                styles,
                                text_align,
                                containing_width,
                                true,
                            ));
                            cursor_x = 0.0;
                            cursor_y += current_line_height;
                            current_line_height = node_line_height;
                        }

                        let words = segment.split_inclusive(' ');

                        for word in words {
                            if word.is_empty() {
                                continue;
                            }

                            // Measure the word with the font's measure helper.
                            // spec: S-45, S-57
                            let word_width = font.measure(word) as f32;

                            let should_break =
                                break_all || (break_word && word_width > containing_width);

                            if allow_wrap
                                && should_break
                                && cursor_x + word_width > containing_width
                            {
                                let mut rem_word = word;
                                while !rem_word.is_empty() {
                                    let rem_width = font.measure(rem_word) as f32;
                                    if cursor_x + rem_width <= containing_width {
                                        // The remaining word fits completely on the current line!
                                        // Push it as a LayoutBox
                                        current_line_children.push(LayoutBox {
                                            node: Some(node),
                                            rect: Rect {
                                                origin: Point {
                                                    x: offset_x + cursor_x,
                                                    y: offset_y + cursor_y,
                                                },
                                                size: Size {
                                                    width: rem_width,
                                                    height: node_line_height,
                                                },
                                            },
                                            children: Vec::new(),
                                            text: Some(rem_word.to_string()),
                                        });
                                        cursor_x += rem_width;
                                        if rem_word.ends_with(" ") {
                                            cursor_x += word_spacing;
                                        }
                                        break; // we are done with this word!
                                    }

                                    // It does not fit. We need to split.
                                    // Let's find the longest prefix that fits on the current line.
                                    // If cursor_x > 0.0, we can try to fit characters in the remaining space.
                                    // But we must check if at least 1 character fits.
                                    // If not even 1 character fits, we must flush first.

                                    // Let's find how many characters we can fit.
                                    let mut chars_iter = rem_word.char_indices();
                                    // Get the first character
                                    let (first_idx, first_c) = match chars_iter.next() {
                                        Some(val) => val,
                                        None => break, // Should not happen since !rem_word.is_empty()
                                    };
                                    let first_char_end = first_idx + first_c.len_utf8();
                                    let first_char_width =
                                        font.measure(&rem_word[..first_char_end]) as f32;

                                    if cursor_x > 0.0
                                        && cursor_x + first_char_width > containing_width
                                    {
                                        // Not even the first character fits in the remaining space.
                                        // Flush the current line.
                                        line_boxes.push(create_line_box_adjusted(
                                            dom,
                                            block_container,
                                            std::mem::take(&mut current_line_children),
                                            offset_x,
                                            offset_y + cursor_y,
                                            cursor_x,
                                            current_line_height,
                                            styles,
                                            text_align,
                                            containing_width,
                                            false,
                                        ));
                                        cursor_x = 0.0;
                                        cursor_y += current_line_height;
                                        current_line_height = node_line_height;
                                        // Continue loop - now cursor_x is 0.0, so the next iteration will retry with the full line.
                                        continue;
                                    }

                                    // Now, either cursor_x == 0.0, or the first character fits.
                                    // We want to find the maximum prefix that fits.
                                    // We already know the first character is included.
                                    let mut split_index = first_char_end;
                                    let mut last_valid_width = first_char_width;

                                    // Iterate through subsequent characters to see how many more fit.
                                    for (idx, c) in chars_iter {
                                        let candidate_end = idx + c.len_utf8();
                                        let candidate_width =
                                            font.measure(&rem_word[..candidate_end]) as f32;
                                        if cursor_x + candidate_width <= containing_width {
                                            split_index = candidate_end;
                                            last_valid_width = candidate_width;
                                        } else {
                                            // Cannot fit any more characters on this line.
                                            break;
                                        }
                                    }

                                    // Split the word at split_index
                                    let prefix = &rem_word[..split_index];
                                    rem_word = &rem_word[split_index..];

                                    // Push the prefix to the current line
                                    current_line_children.push(LayoutBox {
                                        node: Some(node),
                                        rect: Rect {
                                            origin: Point {
                                                x: offset_x + cursor_x,
                                                y: offset_y + cursor_y,
                                            },
                                            size: Size {
                                                width: last_valid_width,
                                                height: node_line_height,
                                            },
                                        },
                                        children: Vec::new(),
                                        text: Some(prefix.to_string()),
                                    });
                                    cursor_x += last_valid_width;
                                    if prefix.ends_with(" ") {
                                        cursor_x += word_spacing;
                                    }

                                    // Since we didn't fit the whole rem_word, we must flush the line now.
                                    line_boxes.push(create_line_box_adjusted(
                                        dom,
                                        block_container,
                                        std::mem::take(&mut current_line_children),
                                        offset_x,
                                        offset_y + cursor_y,
                                        cursor_x,
                                        current_line_height,
                                        styles,
                                        text_align,
                                        containing_width,
                                        false,
                                    ));
                                    cursor_x = 0.0;
                                    cursor_y += current_line_height;
                                    current_line_height = node_line_height;
                                }
                            } else {
                                if allow_wrap
                                    && cursor_x + word_width > containing_width
                                    && cursor_x > 0.0
                                {
                                    // Flush current line
                                    line_boxes.push(create_line_box_adjusted(
                                        dom,
                                        block_container,
                                        std::mem::take(&mut current_line_children),
                                        offset_x,
                                        offset_y + cursor_y,
                                        cursor_x,
                                        current_line_height,
                                        styles,
                                        text_align,
                                        containing_width,
                                        false,
                                    ));
                                    cursor_x = 0.0;
                                    cursor_y += current_line_height;
                                    current_line_height = node_line_height;
                                }

                                // Skip leading whitespace on a new line (only if collapsing whitespace)
                                if collapse && cursor_x == 0.0 && word == " " {
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
                                            height: node_line_height,
                                        },
                                    },
                                    children: Vec::new(),
                                    text: Some(word.to_string()),
                                });
                                cursor_x += word_width;
                                // TODO(spec): word-spacing v1 adds a fixed advance after each word that carries a trailing space; interaction with text-align justify, percentage values, and full Unicode space-separator handling are out of scope.
                                if word.ends_with(" ") {
                                    cursor_x += word_spacing;
                                }
                            }
                        }
                    }
                }
                NodeData::Element { name, .. } => {
                    if name.eq_ignore_ascii_case("br") {
                        if styles.get(&node).is_some_and(|style| matches!(style.get("display"), Some(crate::css::values::CssValue::Keyword(kw)) if kw == "none")) {
                            continue;
                        }
                        // Force a line break!
                        line_boxes.push(create_line_box_adjusted(
                            dom,
                            block_container,
                            std::mem::take(&mut current_line_children),
                            offset_x,
                            offset_y + cursor_y,
                            cursor_x,
                            current_line_height,
                            styles,
                            text_align,
                            containing_width,
                            true,
                        ));
                        cursor_x = 0.0;
                        cursor_y += current_line_height;
                        current_line_height = line_height;
                        continue;
                    }

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
                                    dom,
                                    block_container,
                                    std::mem::take(&mut current_line_children),
                                    offset_x,
                                    offset_y + cursor_y,
                                    cursor_x,
                                    current_line_height,
                                    styles,
                                    text_align,
                                    containing_width,
                                    false,
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
            dom,
            block_container,
            current_line_children,
            offset_x,
            offset_y + cursor_y,
            cursor_x,
            current_line_height,
            styles,
            text_align,
            containing_width,
            true,
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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
    text_align: &str,
    text_indent: f32,
    word_spacing: f32,
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
        text_indent,
        word_spacing,
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

fn preprocess_text(text: &str, collapse: bool, preserve_newlines: bool) -> String {
    if collapse && !preserve_newlines {
        return collapse_whitespace(text);
    }

    let mut result = String::with_capacity(text.len());
    let mut last_was_whitespace = false;

    for c in text.chars() {
        if c == '\n' {
            if preserve_newlines {
                result.push('\n');
                last_was_whitespace = false;
            } else {
                if collapse {
                    if !last_was_whitespace {
                        result.push(' ');
                        last_was_whitespace = true;
                    }
                } else {
                    result.push(' ');
                }
            }
        } else if is_html_whitespace(c) {
            if collapse {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
        }
    }
    result
}

fn apply_text_transform(s: &str, kind: &str) -> String {
    // TODO(spec): Simplified capitalization logic. For full Unicode word-boundary nuance, a more complex boundary analysis is required.
    match kind {
        "uppercase" => s.to_uppercase(),
        "lowercase" => s.to_lowercase(),
        "capitalize" => {
            let mut result = String::with_capacity(s.len());
            let mut capitalize_next = true;
            for c in s.chars() {
                if c == ' ' {
                    result.push(c);
                    capitalize_next = true;
                } else if capitalize_next {
                    result.extend(c.to_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            }
            result
        }
        _ => s.to_string(),
    }
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
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

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
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );
        // The call must complete without stack overflow.
        let _ = line_boxes;
    }

    #[test]
    fn test_text_transform_uppercase() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-transform: uppercase; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        assert_eq!(leaf_texts, vec!["HELLO ", "WORLD"]);
    }

    #[test]
    fn test_text_transform_lowercase() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("HELLO World".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-transform: lowercase; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        assert_eq!(leaf_texts, vec!["hello ", "world"]);
    }

    #[test]
    fn test_text_transform_capitalize() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-transform: capitalize; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        assert_eq!(leaf_texts, vec!["Hello ", "World"]);
    }

    #[test]
    fn test_text_transform_none() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-transform: none; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        assert_eq!(leaf_texts, vec!["hello ", "world"]);
    }

    #[test]
    fn test_white_space_nowrap() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // A long string of text that would wrap normally at a width of 100px.
        let t = dom.create_node(NodeData::Text(
            "this is a very long text that should not wrap".into(),
        ));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: nowrap; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Container width is 100px.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 100.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        // Since white-space is nowrap, it must be on a single line.
        assert_eq!(line_boxes.len(), 1);

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }
        // Verify all words are kept on that single line.
        assert_eq!(
            leaf_texts,
            vec![
                "this ", "is ", "a ", "very ", "long ", "text ", "that ", "should ", "not ", "wrap"
            ]
        );
    }

    #[test]
    fn test_white_space_pre_preserves_consecutive_spaces() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello   world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: pre; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        // Under white-space: pre, consecutive spaces should not collapse.
        // Rust's split_inclusive(' ') on "hello   world" will split into: "hello ", " ", " ", "world".
        assert_eq!(leaf_texts, vec!["hello ", " ", " ", "world"]);
    }

    #[test]
    fn test_white_space_pre_forced_newline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello\nworld\n".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: pre; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        // Embedded newlines must produce forced line breaks.
        // "hello\nworld\n" -> segments "hello", "world", "".
        // First line: "hello"
        // Second line: "world"
        // Third line: empty (not flushed because current_line_children is empty)
        // Total line boxes flushed: 2.
        assert_eq!(line_boxes.len(), 2);

        let line_texts: Vec<Vec<String>> = line_boxes.iter().map(collect_leaf_texts).collect();

        assert_eq!(line_texts[0], vec!["hello"]);
        assert_eq!(line_texts[1], vec!["world"]);
    }

    #[test]
    fn test_white_space_pre_line_collapses_and_forces_newlines() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello   \n   world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: pre-line; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        // Under pre-line:
        // - Multiple spaces collapse to one.
        // - Newlines are preserved as forced breaks.
        // "hello   \n   world" collapses non-newline whitespace, so:
        // Before newline, "hello   " collapses to "hello ".
        // After newline, "   world" collapses to " world" (Wait, " " + "world").
        // Since collapse is true, skip leading whitespace on a new line is active.
        // So the " " before "world" on the new line gets skipped.
        assert_eq!(line_boxes.len(), 2);

        let line_texts: Vec<Vec<String>> = line_boxes.iter().map(collect_leaf_texts).collect();

        assert_eq!(line_texts[0], vec!["hello "]);
        assert_eq!(line_texts[1], vec!["world"]);
    }

    #[test]
    fn test_white_space_normal_regression_guard() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello   \n   world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: normal; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        // Under normal:
        // - All spaces/newlines collapse to a single space.
        // - Soft wrapping is allowed.
        // "hello   \n   world" collapses completely to "hello world".
        // Line count should be 1.
        assert_eq!(line_boxes.len(), 1);

        let leaf_texts = collect_leaf_texts(&line_boxes[0]);
        assert_eq!(leaf_texts, vec!["hello ", "world"]);
    }

    #[test]
    fn test_text_indent_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-indent: 40px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);

        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 10.0, 20.0, 0, "left", 40.0, 0.0,
        );

        assert!(!line_boxes.is_empty());
        let first_line = &line_boxes[0];
        assert!(!first_line.children.is_empty());
        let first_fragment = &first_line.children[0];
        assert_eq!(first_fragment.rect.origin.x, 50.0);
    }

    #[test]
    fn test_text_indent_wrapping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world wraps completely".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-indent: 40px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);

        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 120.0, 10.0, 20.0, 0, "left", 40.0, 0.0,
        );

        assert!(line_boxes.len() >= 2);

        let first_line = &line_boxes[0];
        assert!(!first_line.children.is_empty());
        assert_eq!(first_line.children[0].rect.origin.x, 50.0);

        let second_line = &line_boxes[1];
        assert!(!second_line.children.is_empty());
        assert_eq!(second_line.children[0].rect.origin.x, 10.0);
    }

    #[test]
    fn test_text_indent_zero_regression() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-indent: 0px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);

        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 10.0, 20.0, 0, "left", 0.0, 0.0,
        );

        assert!(!line_boxes.is_empty());
        let first_line = &line_boxes[0];
        assert!(!first_line.children.is_empty());
        let first_fragment = &first_line.children[0];
        assert_eq!(first_fragment.rect.origin.x, 10.0);
    }

    #[test]
    fn test_word_spacing_behavior() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello world".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { word-spacing: 10px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);

        // 1. With word_spacing = 0.0
        let (line_boxes_0, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 10.0, 20.0, 0, "left", 0.0, 0.0,
        );
        assert!(!line_boxes_0.is_empty());
        let line_0 = &line_boxes_0[0];
        assert_eq!(line_0.children.len(), 2);
        let first_word_0 = &line_0.children[0];
        let second_word_0 = &line_0.children[1];

        // 2. With word_spacing = 10.0
        let (line_boxes_10, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 10.0, 20.0, 0, "left", 0.0, 10.0,
        );
        assert!(!line_boxes_10.is_empty());
        let line_10 = &line_boxes_10[0];
        assert_eq!(line_10.children.len(), 2);
        let first_word_10 = &line_10.children[0];
        let second_word_10 = &line_10.children[1];

        // Assert the first word is in the same position and has the same width
        assert_eq!(first_word_0.rect.origin.x, first_word_10.rect.origin.x);
        assert_eq!(first_word_0.rect.size.width, first_word_10.rect.size.width);

        // Assert the second word fragment x for word_spacing=10.0 is exactly 10.0 greater
        assert_eq!(
            second_word_10.rect.origin.x,
            second_word_0.rect.origin.x + 10.0
        );

        // 3. A single word with no trailing space produces the SAME layout
        let mut dom_single = Dom::new();
        let doc_single = dom_single.document();
        let div_single = dom_single.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_single.append_child(doc_single, div_single);

        let t_single = dom_single.create_node(NodeData::Text("hello".into()));
        dom_single.append_child(div_single, t_single);

        let styles_single = compute_styles(&dom_single, &stylesheet);
        let children_single = dom_single.children(div_single);

        let (line_boxes_single_0, _) = layout_inline_run(
            &dom_single,
            &styles_single,
            children_single,
            800.0,
            10.0,
            20.0,
            0,
            "left",
            0.0,
            0.0,
        );
        let (line_boxes_single_10, _) = layout_inline_run(
            &dom_single,
            &styles_single,
            children_single,
            800.0,
            10.0,
            20.0,
            0,
            "left",
            0.0,
            10.0,
        );

        assert_eq!(line_boxes_single_0.len(), 1);
        assert_eq!(line_boxes_single_10.len(), 1);
        assert_eq!(
            line_boxes_single_0[0].children[0].rect.origin.x,
            line_boxes_single_10[0].children[0].rect.origin.x
        );
        assert_eq!(
            line_boxes_single_0[0].children[0].rect.size.width,
            line_boxes_single_10[0].children[0].rect.size.width
        );
    }

    #[test]
    fn test_text_align_justify_distributes_gaps() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // "hello world wrap now" has 4 words.
        // monospaced width of each character is 8px.
        // "hello " has 6 chars -> 48px
        // "world " has 6 chars -> 48px
        // "wrap " has 5 chars -> 40px
        // "now" has 3 chars -> 24px
        // Let's set containing_width to 120px.
        // - Line 1: "hello " (48px) + "world " (48px) = 96px <= 120px. Fits.
        // - "wrap " would make it 96px + 40px = 136px > 120px. Overflows, so wraps.
        // - Line 2: "wrap " (40px) + "now" (24px) = 64px <= 120px. Fits. This is the last line.
        let t = dom.create_node(NodeData::Text("hello world wrap now".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-align: justify; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);

        // Run layout with justify
        let (line_boxes_justify, _) = layout_inline_run(
            &dom, &styles, children, 120.0, 0.0, 0.0, 0, "justify", 0.0, 0.0,
        );

        // Run layout with left for baseline comparison
        let children2 = dom.children(div);
        let (line_boxes_left, _) = layout_inline_run(
            &dom, &styles, children2, 120.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes_justify.len(), 2);
        assert_eq!(line_boxes_left.len(), 2);

        // --- LINE 1 Verification ---
        // Justified line 1 should have "hello " and "world ".
        let leaf_texts_j1 = collect_leaf_texts(&line_boxes_justify[0]);
        assert_eq!(leaf_texts_j1, vec!["hello ", "world "]);

        // "hello " should start at x = 0.0.
        let word1_j = &line_boxes_justify[0].children[0];
        let word2_j = &line_boxes_justify[0].children[1];
        assert_eq!(word1_j.rect.origin.x, 0.0);

        // The last word's right edge ("world ") must meet containing_width (120px).
        let last_word_right_edge_j = word2_j.rect.origin.x + word2_j.rect.size.width;
        assert!((last_word_right_edge_j - 120.0).abs() < 1.0);

        // Compare with left layout
        let word1_l = &line_boxes_left[0].children[0];
        let word2_l = &line_boxes_left[0].children[1];
        assert_eq!(word1_l.rect.origin.x, 0.0);
        assert_eq!(word2_l.rect.origin.x, 48.0);

        // Gap in left: word2_l.x - (word1_l.x + word1_l.width) = 48.0 - 48.0 = 0.0.
        // Gap in justify: word2_j.x - (word1_j.x + word1_j.width) = 72.0 - 48.0 = 24.0.
        let left_gap = word2_l.rect.origin.x - (word1_l.rect.origin.x + word1_l.rect.size.width);
        let justify_gap = word2_j.rect.origin.x - (word1_j.rect.origin.x + word1_j.rect.size.width);
        assert!(justify_gap > left_gap);

        // --- LINE 2 Verification (Last Line) ---
        // Justified line 2 is the last line, so it must stay left-aligned (no stretching).
        let word3_j = &line_boxes_justify[1].children[0];
        let word4_j = &line_boxes_justify[1].children[1];

        let word3_l = &line_boxes_left[1].children[0];
        let word4_l = &line_boxes_left[1].children[1];

        // Position of words in justified line 2 should match left-aligned layout.
        assert_eq!(word3_j.rect.origin.x, word3_l.rect.origin.x);
        assert_eq!(word4_j.rect.origin.x, word4_l.rect.origin.x);
    }

    #[test]
    fn test_word_break_break_all() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("abcdefghijklmnop".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { word-break: break-all; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Narrow container of 40px width (fits up to 5 chars of 8px each per line)
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 40.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        // It must be split across multiple line boxes
        assert!(line_boxes.len() > 1);

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }
        assert_eq!(leaf_texts, vec!["abcde", "fghij", "klmno", "p"]);

        // Test case 2: word-break: normal (default) on the exact same long word
        let mut dom_normal = Dom::new();
        let doc_normal = dom_normal.document();
        let div_normal = dom_normal.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_normal.append_child(doc_normal, div_normal);

        let t_normal = dom_normal.create_node(NodeData::Text("abcdefghijklmnop".into()));
        dom_normal.append_child(div_normal, t_normal);

        // Normal word-break
        let stylesheet_normal = parse_stylesheet("div { word-break: normal; }");
        let styles_normal = compute_styles(&dom_normal, &stylesheet_normal);

        let children_normal = dom_normal.children(div_normal);
        let (line_boxes_normal, _) = layout_inline_run(
            &dom_normal,
            &styles_normal,
            children_normal,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // Under normal word-break, single word overflows and stays on a single line
        assert_eq!(line_boxes_normal.len(), 1);
    }

    #[test]
    fn test_overflow_wrap_break_word() {
        // Test case 1: A long unbreakable word (e.g., 60 chars) in a narrow container with overflow-wrap: break-word
        // must produce MORE THAN ONE line box (the word is split across lines).
        let mut dom_break = Dom::new();
        let doc_break = dom_break.document();
        let div_break = dom_break.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_break.append_child(doc_break, div_break);

        let t_break = dom_break.create_node(NodeData::Text(
            "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefgh".into(),
        ));
        dom_break.append_child(div_break, t_break);

        let stylesheet_break = parse_stylesheet("div { overflow-wrap: break-word; }");
        let styles_break = compute_styles(&dom_break, &stylesheet_break);

        let children_break = dom_break.children(div_break);
        // Narrow container of 40px width (fits up to 5 chars of 8px each per line)
        let (line_boxes_break, _) = layout_inline_run(
            &dom_break,
            &styles_break,
            children_break,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // It must be split across multiple line boxes
        assert!(line_boxes_break.len() > 1);

        // Test case 2: The SAME long word with overflow-wrap: normal must stay on a SINGLE line box (overflow, no split)
        let mut dom_normal = Dom::new();
        let doc_normal = dom_normal.document();
        let div_normal = dom_normal.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_normal.append_child(doc_normal, div_normal);

        let t_normal = dom_normal.create_node(NodeData::Text(
            "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefgh".into(),
        ));
        dom_normal.append_child(div_normal, t_normal);

        let stylesheet_normal = parse_stylesheet("div { overflow-wrap: normal; }");
        let styles_normal = compute_styles(&dom_normal, &stylesheet_normal);

        let children_normal = dom_normal.children(div_normal);
        let (line_boxes_normal, _) = layout_inline_run(
            &dom_normal,
            &styles_normal,
            children_normal,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // Under normal overflow-wrap, the single word overflows and stays on a single line
        assert_eq!(line_boxes_normal.len(), 1);

        // Test case 3: A SHORT word that fits on its own line, with overflow-wrap: break-word,
        // must NOT be split (it wraps whole) — this proves break-word differs from break-all.
        let mut dom_short = Dom::new();
        let doc_short = dom_short.document();
        let div_short = dom_short.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_short.append_child(doc_short, div_short);

        // Let's create two words: "abc" and "def"
        // Let's make sure the width of container fits "abc" (3 chars, 24px) but "abc def" (7 chars, 56px) overflows.
        // Container width: 40px.
        let t_short = dom_short.create_node(NodeData::Text("abc def".into()));
        dom_short.append_child(div_short, t_short);

        let stylesheet_short = parse_stylesheet("div { overflow-wrap: break-word; }");
        let styles_short = compute_styles(&dom_short, &stylesheet_short);

        let children_short = dom_short.children(div_short);
        let (line_boxes_short, _) = layout_inline_run(
            &dom_short,
            &styles_short,
            children_short,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // Since the second word fits on its own line, it should wrap whole.
        // It must NOT be split (meaning we should have exactly two line boxes, with whole words)
        assert_eq!(line_boxes_short.len(), 2);
        let mut leaf_texts = Vec::new();
        for line in &line_boxes_short {
            leaf_texts.extend(collect_leaf_texts(line));
        }
        assert_eq!(leaf_texts, vec!["abc ", "def"]);

        // Test case 4: Legacy alias word-wrap: break-word is honored
        let mut dom_alias = Dom::new();
        let doc_alias = dom_alias.document();
        let div_alias = dom_alias.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom_alias.append_child(doc_alias, div_alias);

        let t_alias = dom_alias.create_node(NodeData::Text(
            "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefgh".into(),
        ));
        dom_alias.append_child(div_alias, t_alias);

        let stylesheet_alias = parse_stylesheet("div { word-wrap: break-word; }");
        let styles_alias = compute_styles(&dom_alias, &stylesheet_alias);

        let children_alias = dom_alias.children(div_alias);
        let (line_boxes_alias, _) = layout_inline_run(
            &dom_alias,
            &styles_alias,
            children_alias,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // It must be split across multiple line boxes under word-wrap: break-word
        assert!(line_boxes_alias.len() > 1);
    }

    #[test]
    fn test_vertical_align_baseline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let s1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, s1);

        let t1 = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(s1, t1);

        let s2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, s2);

        let t2 = dom.create_node(NodeData::Text("world".into()));
        dom.append_child(s2, t2);

        let stylesheet = parse_stylesheet("span { display: inline; vertical-align: baseline; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 2);
        let box1 = &line.children[0];
        let box2 = &line.children[1];

        // Their y coordinates should be identical (both align with baseline)
        assert_eq!(box1.rect.origin.y, box2.rect.origin.y);
    }

    #[test]
    fn test_vertical_align_super_sub() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let s1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "sup-class".into())],
        });
        dom.append_child(div, s1);

        let t1 = dom.create_node(NodeData::Text("sup".into()));
        dom.append_child(s1, t1);

        let s2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, s2);

        let t2 = dom.create_node(NodeData::Text("base".into()));
        dom.append_child(s2, t2);

        let s3 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "sub-class".into())],
        });
        dom.append_child(div, s3);

        let t3 = dom.create_node(NodeData::Text("sub".into()));
        dom.append_child(s3, t3);

        let stylesheet = parse_stylesheet(
            "
            span { display: inline; font-size: 20px; }
            .sup-class { vertical-align: super; }
            .sub-class { vertical-align: sub; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 3);

        let box_sup = &line.children[0];
        let box_base = &line.children[1];
        let box_sub = &line.children[2];

        // super shifts UP relative to baseline, so its Y is smaller (since Y increases downwards)
        assert!(box_sup.rect.origin.y < box_base.rect.origin.y);

        // sub shifts DOWN relative to baseline, so its Y is larger
        assert!(box_sub.rect.origin.y > box_base.rect.origin.y);
    }

    #[test]
    fn test_vertical_align_middle_top() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let s1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "mid-class".into())],
        });
        dom.append_child(div, s1);

        let t1 = dom.create_node(NodeData::Text("mid".into()));
        dom.append_child(s1, t1);

        let s2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, s2);

        let t2 = dom.create_node(NodeData::Text("base".into()));
        dom.append_child(s2, t2);

        let s3 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "top-class".into())],
        });
        dom.append_child(div, s3);

        let t3 = dom.create_node(NodeData::Text("top".into()));
        dom.append_child(s3, t3);

        // Put an inline-block with height 50px on the line box to force line_height to be large (50px).
        // That way text-top has a distinct effect (aligns top of text with top of the line box).
        let s4 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ib-class".into())],
        });
        dom.append_child(div, s4);

        let t4 = dom.create_node(NodeData::Text("ib".into()));
        dom.append_child(s4, t4);

        let stylesheet = parse_stylesheet(
            "
            span { display: inline; }
            .mid-class { vertical-align: middle; font-size: 24px; }
            .top-class { vertical-align: text-top; }
            .ib-class { display: inline-block; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        // 4 elements on the line
        assert_eq!(line.children.len(), 4);

        let box_mid = &line.children[0];
        let box_base = &line.children[1];
        let box_top = &line.children[2];

        // text-top alignments should place the top of the fragment at the top of the line box (which is y = 0.0)
        assert_eq!(box_top.rect.origin.y, 0.0);

        // baseline aligns its bottom with bottom of line box (which is y = 50.0), so its top (y) is at 50.0 - 8.0 = 42.0
        assert_eq!(box_base.rect.origin.y, 42.0);

        // middle should align its vertical center with baseline - 0.25 * font_size
        // Since font_size = 24px, 0.25 * 24.0 = 6px.
        // Baseline of parent is line_box_bottom_y = 50.0.
        // Target vertical center is 50.0 - 6.0 = 44.0.
        // Fragment height is 8.0.
        // So target y is 44.0 - 4.0 = 40.0.
        // We can assert target y is 40.0, which is smaller than base (42.0) and larger than top (0.0)
        assert_eq!(box_mid.rect.origin.y, 40.0);
    }

    #[test]
    fn test_br_element_forced_line_break() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let t1 = dom.create_node(NodeData::Text("a".into()));
        dom.append_child(body, t1);

        let br = dom.create_node(NodeData::Element {
            name: "br".into(),
            attrs: vec![],
        });
        dom.append_child(body, br);

        let t2 = dom.create_node(NodeData::Text("b".into()));
        dom.append_child(body, t2);

        let stylesheet = parse_stylesheet("br { display: inline; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(body);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        // Prove that "a" and "b" are on separate line boxes.
        assert_eq!(line_boxes.len(), 2);

        // Let's assert the second line's origin y is greater than the first's
        assert!(line_boxes[1].rect.origin.y > line_boxes[0].rect.origin.y);

        // Verify that without <br>, "a b" stays on 1 line.
        let mut dom_nobr = Dom::new();
        let doc_nobr = dom_nobr.document();
        let body_nobr = dom_nobr.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom_nobr.append_child(doc_nobr, body_nobr);

        let t_nobr = dom_nobr.create_node(NodeData::Text("a b".into()));
        dom_nobr.append_child(body_nobr, t_nobr);

        let styles_nobr = compute_styles(&dom_nobr, &stylesheet);
        let children_nobr = dom_nobr.children(body_nobr);
        let (line_boxes_nobr, _) = layout_inline_run(
            &dom_nobr,
            &styles_nobr,
            children_nobr,
            800.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );
        assert_eq!(line_boxes_nobr.len(), 1);

        // Verify multiple consecutive `<br><br>`
        let mut dom_multi = Dom::new();
        let doc_multi = dom_multi.document();
        let body_multi = dom_multi.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom_multi.append_child(doc_multi, body_multi);

        let t_m1 = dom_multi.create_node(NodeData::Text("a".into()));
        dom_multi.append_child(body_multi, t_m1);

        let br_m1 = dom_multi.create_node(NodeData::Element {
            name: "br".into(),
            attrs: vec![],
        });
        dom_multi.append_child(body_multi, br_m1);

        let br_m2 = dom_multi.create_node(NodeData::Element {
            name: "br".into(),
            attrs: vec![],
        });
        dom_multi.append_child(body_multi, br_m2);

        let t_m2 = dom_multi.create_node(NodeData::Text("b".into()));
        dom_multi.append_child(body_multi, t_m2);

        let styles_multi = compute_styles(&dom_multi, &stylesheet);
        let children_multi = dom_multi.children(body_multi);
        let (line_boxes_multi, _) = layout_inline_run(
            &dom_multi,
            &styles_multi,
            children_multi,
            800.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        // "a" then break, then break (producing an empty line box), then "b"
        assert_eq!(line_boxes_multi.len(), 3);
        assert!(line_boxes_multi[1].children.is_empty()); // second line is empty due to consecutive <br>
        assert!(line_boxes_multi[2].rect.origin.y > line_boxes_multi[1].rect.origin.y);
        assert!(line_boxes_multi[1].rect.origin.y > line_boxes_multi[0].rect.origin.y);
    }

    #[test]
    fn test_vertical_align_length_and_percentage_direct() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let s1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "px-class".into())],
        });
        dom.append_child(div, s1);

        let s2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "px-neg-class".into())],
        });
        dom.append_child(div, s2);

        let s3 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "em-class".into())],
        });
        dom.append_child(div, s3);

        let s4 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "percent-class".into())],
        });
        dom.append_child(div, s4);

        let s5 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "rem-class".into())],
        });
        dom.append_child(div, s5);

        let stylesheet = parse_stylesheet(
            "
            .px-class { vertical-align: 4px; }
            .px-neg-class { vertical-align: -3px; }
            .em-class { vertical-align: 0.5em; font-size: 20px; }
            .percent-class { vertical-align: 50%; }
            .rem-class { vertical-align: 0.25rem; font-size: 16px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // 1. vertical-align: 4px
        // raise = 4px, shift = -raise = -4px (up)
        let shift1 = get_vertical_align_shift(s1, Some(div), &dom, &styles, 30.0, 10.0);
        assert_eq!(shift1, -4.0);

        // 2. vertical-align: -3px
        // raise = -3px, shift = -raise = 3px (down)
        let shift2 = get_vertical_align_shift(s2, Some(div), &dom, &styles, 30.0, 10.0);
        assert_eq!(shift2, 3.0);

        // 3. vertical-align: 0.5em with font-size: 20px
        // raise = 0.5 * 20 = 10px, shift = -raise = -10px
        let shift3 = get_vertical_align_shift(s3, Some(div), &dom, &styles, 30.0, 10.0);
        assert_eq!(shift3, -10.0);

        // 4. vertical-align: 50% with line-height: 30.0
        // raise = 0.50 * 30 = 15px, shift = -raise = -15px
        let shift4 = get_vertical_align_shift(s4, Some(div), &dom, &styles, 30.0, 10.0);
        assert_eq!(shift4, -15.0);

        // 5. vertical-align: 0.25rem (resolved to element's font-size = 16px)
        // raise = 0.25 * 16 = 4px, shift = -raise = -4px
        let shift5 = get_vertical_align_shift(s5, Some(div), &dom, &styles, 30.0, 10.0);
        assert_eq!(shift5, -4.0);
    }

    #[test]
    fn test_vertical_align_length_and_percentage_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // s1: baseline reference
        let s_base = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "base-class".into())],
        });
        dom.append_child(div, s_base);
        let t_base = dom.create_node(NodeData::Text("base".into()));
        dom.append_child(s_base, t_base);

        // s2: 4px raise
        let s_px = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "px-class".into())],
        });
        dom.append_child(div, s_px);
        let t_px = dom.create_node(NodeData::Text("4px".into()));
        dom.append_child(s_px, t_px);

        // s3: -3px lower
        let s_px_neg = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "px-neg-class".into())],
        });
        dom.append_child(div, s_px_neg);
        let t_px_neg = dom.create_node(NodeData::Text("-3px".into()));
        dom.append_child(s_px_neg, t_px_neg);

        // s4: 0.5em raise (font-size 20px)
        let s_em = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "em-class".into())],
        });
        dom.append_child(div, s_em);
        let t_em = dom.create_node(NodeData::Text("em".into()));
        dom.append_child(s_em, t_em);

        // s5: 50% raise
        let s_pct = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "pct-class".into())],
        });
        dom.append_child(div, s_pct);
        let t_pct = dom.create_node(NodeData::Text("pct".into()));
        dom.append_child(s_pct, t_pct);

        // Put an inline-block with height 50px on the line box to force line_height to be large (50px).
        // That way percentage and em effects have a distinct line-height context.
        let s_ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ib-class".into())],
        });
        dom.append_child(div, s_ib);
        let t_ib = dom.create_node(NodeData::Text("ib".into()));
        dom.append_child(s_ib, t_ib);

        let stylesheet = parse_stylesheet(
            "
            span { display: inline; }
            .base-class { vertical-align: baseline; }
            .px-class { vertical-align: 4px; }
            .px-neg-class { vertical-align: -3px; }
            .em-class { vertical-align: 0.5em; font-size: 20px; }
            .pct-class { vertical-align: 50%; }
            .ib-class { display: inline-block; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 6);

        let box_base = &line.children[0];
        let box_px = &line.children[1];
        let box_px_neg = &line.children[2];
        let box_em = &line.children[3];
        let box_pct = &line.children[4];

        // 1. vertical-align: 4px raises span (smaller y) by 4px delta
        assert_eq!(box_px.rect.origin.y, box_base.rect.origin.y - 4.0);

        // 2. vertical-align: -3px lowers span (larger y) by 3px delta
        assert_eq!(box_px_neg.rect.origin.y, box_base.rect.origin.y + 3.0);

        // 3. vertical-align: 0.5em with font-size: 20px raises span by 10px delta
        assert_eq!(box_em.rect.origin.y, box_base.rect.origin.y - 10.0);

        // 4. vertical-align: 50% raises span by 50% of line_height
        // line_height is 50px, so 50% of 50px = 25px delta
        assert_eq!(box_pct.rect.origin.y, box_base.rect.origin.y - 25.0);
    }

    #[test]
    fn test_line_height_px_sets_line_box() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello line height px".into()));
        dom.append_child(div, t1);

        let stylesheet = parse_stylesheet("div { line-height: 40px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, total_height) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.rect.size.height, 40.0);
        assert_eq!(total_height, 40.0);
    }

    #[test]
    fn test_line_height_absent_uses_font_default() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello default font height".into()));
        dom.append_child(div, t1);

        let stylesheet = parse_stylesheet(""); // No styles
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, total_height) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let font = crate::font::BitmapFont::builtin();
        let expected_default_height = font.line_height() as f32;

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.rect.size.height, expected_default_height);
        assert_eq!(total_height, expected_default_height);
    }

    #[test]
    fn test_line_height_number_multiplier() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello line height multiplier".into()));
        dom.append_child(div, t1);

        let stylesheet = parse_stylesheet("div { font-size: 20px; line-height: 2; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, total_height) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.rect.size.height, 40.0);
        assert_eq!(total_height, 40.0);
    }
}
