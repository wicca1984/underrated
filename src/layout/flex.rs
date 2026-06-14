use crate::dom::Dom;
use crate::geom::{Point, Rect};
use crate::infra::NodeId;
use crate::layout::{LayoutBox, get_px, is_absolute_or_fixed, layout_node};
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

pub fn layout_flex_container(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
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
    let flex_direction = if style.reset_flex.flex_direction == "column" {
        FlexDirection::Column
    } else {
        FlexDirection::Row
    };

    let justify_content = match style.reset_flex.justify_content.as_str() {
        "flex-start" => JustifyContent::FlexStart,
        "flex-end" => JustifyContent::FlexEnd,
        "center" => JustifyContent::Center,
        "space-between" => JustifyContent::SpaceBetween,
        "space-around" => JustifyContent::SpaceAround,
        "space-evenly" => JustifyContent::SpaceEvenly,
        _ => JustifyContent::FlexStart,
    };

    let align_items = match style.reset_flex.align_items.as_str() {
        "stretch" => AlignItems::Stretch,
        "flex-start" => AlignItems::FlexStart,
        "flex-end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        "baseline" => AlignItems::Baseline,
        _ => AlignItems::Stretch,
    };

    let align_content = match style.reset_flex.align_content.as_str() {
        "center" => AlignContent::Center,
        "space-between" => AlignContent::SpaceBetween,
        _ => AlignContent::FlexStart,
    };

    let flex_wrap = if style.reset_flex.flex_wrap == "wrap" {
        FlexWrap::Wrap
    } else {
        FlexWrap::Nowrap
    };

    // Resolve main_gap and cross_gap from style
    let row_gap = if style.reset_flex.row_gap == -1 {
        0.0
    } else {
        (style.reset_flex.row_gap as f32).max(0.0)
    };
    let col_gap = if style.reset_flex.column_gap == -1 {
        0.0
    } else {
        (style.reset_flex.column_gap as f32).max(0.0)
    };

    let (main_gap, cross_gap) = match flex_direction {
        FlexDirection::Row => (col_gap, row_gap),
        FlexDirection::Column => (row_gap, col_gap),
    };

    // 1. Layout children to determine their base sizes.
    // For now, we layout them as blocks to get their natural height/width.
    let mut temp_children = Vec::new();
    let inner_x = border_box_x + border_left + padding_left;
    let inner_y = border_box_y + border_top + padding_top;

    for &child in dom.children(node) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }
        if let Some(mut child_box) = layout_node(
            dom,
            styles,
            child,
            content_width,
            inner_x,
            inner_y,
            depth + 1,
        ) {
            if let Some(child_style) = styles.get(&child) {
                // Determine initial main size (either flex-basis or laid out size)
                let flex_basis = child_style.reset_flex.flex_basis;
                let mut main_val = if flex_basis != -1 {
                    flex_basis as f32
                } else {
                    match flex_direction {
                        FlexDirection::Row => child_box.rect.size.width,
                        FlexDirection::Column => child_box.rect.size.height,
                    }
                };

                // Clamp using min/max main size
                let container_height = get_px(style, "height", 0.0);
                main_val = clamp_main_size(
                    child_style,
                    main_val,
                    flex_direction,
                    content_width,
                    container_height,
                );

                match flex_direction {
                    FlexDirection::Row => {
                        child_box.rect.size.width = main_val;
                    }
                    FlexDirection::Column => {
                        child_box.rect.size.height = main_val;
                    }
                }
            }

            let order = styles.get(&child).map(|s| s.reset_flex.order).unwrap_or(0);
            temp_children.push((order, child_box));
        }
    }

    // Sort stably by order ascending so that items with equal order retain source order.
    temp_children.sort_by_key(|a| a.0);

    let children: Vec<LayoutBox> = temp_children
        .into_iter()
        .map(|(_, child_box)| child_box)
        .collect();

    // 2. Distribute free space along the main axis.
    let (main_size, _cross_size) = match flex_direction {
        FlexDirection::Row => (content_width, get_px(style, "height", 0.0)),
        FlexDirection::Column => (get_px(style, "height", 0.0), content_width),
    };

    // Group children into lines based on flex_wrap
    struct FlexLine {
        children: Vec<LayoutBox>,
    }

    let has_main_constraint = match flex_direction {
        FlexDirection::Row => true,
        FlexDirection::Column => has_explicit_size(Some(style), "height"),
    };

    let mut lines = Vec::new();
    if flex_wrap == FlexWrap::Nowrap || !has_main_constraint || children.is_empty() {
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

            let gap_to_add = if current_line.children.is_empty() {
                0.0
            } else {
                main_gap
            };

            if !current_line.children.is_empty()
                && current_line_main_size + gap_to_add + child_main_size > main_size
            {
                lines.push(current_line);
                current_line = FlexLine {
                    children: Vec::new(),
                };
                current_line_main_size = 0.0;
            } else if !current_line.children.is_empty() {
                current_line_main_size += gap_to_add;
            }

            current_line_main_size += child_main_size;
            current_line.children.push(child);
        }

        if !current_line.children.is_empty() {
            lines.push(current_line);
        }
    }

    // Distribute free space along the main axis for each line separately (flex-grow / flex-shrink)
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

        let gap_count = if line.children.is_empty() {
            0
        } else {
            line.children.len() - 1
        };
        let total_gap_size = gap_count as f32 * main_gap;
        let line_free_space = main_size - total_line_main_size - total_gap_size;
        let has_explicit_main_size = match flex_direction {
            FlexDirection::Row => true,
            FlexDirection::Column => has_explicit_size(Some(style), "height"),
        };

        if line_free_space > 0.0 && total_line_flex_grow > 0.0 {
            for child_box in &mut line.children {
                if let Some(child_style) = child_box.node.and_then(|id| styles.get(&id)) {
                    let grow = get_number(child_style, "flex-grow", 0.0);
                    let extra = (grow / total_line_flex_grow) * line_free_space;
                    let container_height = get_px(style, "height", 0.0);
                    match flex_direction {
                        FlexDirection::Row => {
                            let mut new_width = child_box.rect.size.width + extra;
                            new_width = clamp_main_size(
                                child_style,
                                new_width,
                                flex_direction,
                                content_width,
                                container_height,
                            );
                            child_box.rect.size.width = new_width;
                        }
                        FlexDirection::Column => {
                            let mut new_height = child_box.rect.size.height + extra;
                            new_height = clamp_main_size(
                                child_style,
                                new_height,
                                flex_direction,
                                content_width,
                                container_height,
                            );
                            child_box.rect.size.height = new_height;
                        }
                    }
                }
            }
        } else if line_free_space < 0.0 && has_explicit_main_size {
            let negative_free_space = -line_free_space;
            let mut total_scaled_shrink = 0.0;

            for child_box in &line.children {
                if let Some(child_style) = child_box.node.and_then(|id| styles.get(&id)) {
                    let base_size = match flex_direction {
                        FlexDirection::Row => child_box.rect.size.width,
                        FlexDirection::Column => child_box.rect.size.height,
                    };
                    let shrink = get_number(child_style, "flex-shrink", 1.0);
                    total_scaled_shrink += shrink * base_size;
                }
            }

            if total_scaled_shrink > 0.0 {
                for child_box in &mut line.children {
                    if let Some(child_style) = child_box.node.and_then(|id| styles.get(&id)) {
                        let base_size = match flex_direction {
                            FlexDirection::Row => child_box.rect.size.width,
                            FlexDirection::Column => child_box.rect.size.height,
                        };
                        let shrink = get_number(child_style, "flex-shrink", 1.0);
                        let scaled_shrink = shrink * base_size;
                        let shrink_amount =
                            (scaled_shrink / total_scaled_shrink) * negative_free_space;
                        let mut new_size = (base_size - shrink_amount).max(0.0);
                        let container_height = get_px(style, "height", 0.0);
                        new_size = clamp_main_size(
                            child_style,
                            new_size,
                            flex_direction,
                            content_width,
                            container_height,
                        );
                        match flex_direction {
                            FlexDirection::Row => child_box.rect.size.width = new_size,
                            FlexDirection::Column => child_box.rect.size.height = new_size,
                        }
                    }
                }
            }
        }
    }

    // Calculate cross size and total main size for each line after flex-grow / flex-shrink
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

    let num_lines = lines.len();
    let total_cross_gap = if num_lines > 1 {
        (num_lines - 1) as f32 * cross_gap
    } else {
        0.0
    };
    let sum_of_each_line_max_cross_size: f32 = line_max_cross_sizes.iter().sum();
    let total_lines_cross_size = sum_of_each_line_max_cross_size + total_cross_gap;

    let container_cross_size = match flex_direction {
        FlexDirection::Row => {
            get_px(style, "height", total_lines_cross_size).max(total_lines_cross_size)
        }
        FlexDirection::Column => content_width.max(total_lines_cross_size),
    };

    // Calculate cross offsets for each line based on align-content and cross_gap
    let mut line_cross_offsets = Vec::new();
    let free_space = (container_cross_size - total_lines_cross_size).max(0.0);

    if num_lines > 1 {
        match align_content {
            AlignContent::FlexStart => {
                let mut current_offset = 0.0;
                for &size in &line_max_cross_sizes {
                    line_cross_offsets.push(current_offset);
                    current_offset += size + cross_gap;
                }
            }
            AlignContent::Center => {
                let start_offset = free_space / 2.0;
                let mut current_offset = start_offset;
                for &size in &line_max_cross_sizes {
                    line_cross_offsets.push(current_offset);
                    current_offset += size + cross_gap;
                }
            }
            AlignContent::SpaceBetween => {
                let extra_gap = free_space / (num_lines - 1) as f32;
                let mut current_offset = 0.0;
                for &size in &line_max_cross_sizes {
                    line_cross_offsets.push(current_offset);
                    current_offset += size + cross_gap + extra_gap;
                }
            }
        }
    } else {
        let mut current_offset = 0.0;
        for &size in &line_max_cross_sizes {
            line_cross_offsets.push(current_offset);
            current_offset += size;
        }
    }

    let mut line_total_main_sizes_with_gap = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let gap_count = if line.children.is_empty() {
            0
        } else {
            line.children.len() - 1
        };
        let line_total_gap_size = gap_count as f32 * main_gap;
        line_total_main_sizes_with_gap.push(line_total_main_sizes[idx] + line_total_gap_size);
    }
    let max_line_total_main_size_with_gap = line_total_main_sizes_with_gap
        .iter()
        .cloned()
        .fold(0.0f32, f32::max);

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
        let gap_count = if line.children.is_empty() {
            0
        } else {
            line.children.len() - 1
        };
        let line_total_gap_size = gap_count as f32 * main_gap;
        let line_total_main_size_with_gap = line_total_main_size + line_total_gap_size;

        let (mut main_cursor, spacing) = match justify_content {
            JustifyContent::FlexStart => (0.0, 0.0),
            JustifyContent::FlexEnd => (main_size - line_total_main_size_with_gap, 0.0),
            JustifyContent::Center => ((main_size - line_total_main_size_with_gap) / 2.0, 0.0),
            JustifyContent::SpaceBetween => {
                let spacing = if line.children.len() > 1 {
                    ((main_size - line_total_main_size_with_gap) / (line.children.len() - 1) as f32)
                        .max(0.0)
                } else {
                    0.0
                };
                (0.0, spacing)
            }
            JustifyContent::SpaceAround => {
                if line.children.is_empty() {
                    (0.0, 0.0)
                } else if line.children.len() == 1 || main_size < line_total_main_size_with_gap {
                    ((main_size - line_total_main_size_with_gap) / 2.0, 0.0)
                } else {
                    let free_space = main_size - line_total_main_size_with_gap;
                    let spacing = free_space / line.children.len() as f32;
                    (spacing / 2.0, spacing)
                }
            }
            JustifyContent::SpaceEvenly => {
                if line.children.is_empty() {
                    (0.0, 0.0)
                } else if line.children.len() == 1 || main_size < line_total_main_size_with_gap {
                    ((main_size - line_total_main_size_with_gap) / 2.0, 0.0)
                } else {
                    let free_space = main_size - line_total_main_size_with_gap;
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

            let child_align = get_align_self(child_style, align_items);

            let cross_offset = match child_align {
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
                            FlexDirection::Row => {
                                let container_height = get_px(style, "height", 0.0);
                                let mut stretched = line_cross_size;
                                if let Some(cs) = child_style {
                                    stretched = clamp_cross_size(
                                        cs,
                                        stretched,
                                        flex_direction,
                                        content_width,
                                        container_height,
                                    );
                                }
                                child_box.rect.size.height = stretched;
                            }
                            FlexDirection::Column => {
                                let container_height = get_px(style, "height", 0.0);
                                let mut stretched = line_cross_size;
                                if let Some(cs) = child_style {
                                    stretched = clamp_cross_size(
                                        cs,
                                        stretched,
                                        flex_direction,
                                        content_width,
                                        container_height,
                                    );
                                }
                                child_box.rect.size.width = stretched;
                            }
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

            let advance = match flex_direction {
                FlexDirection::Row => child_box.rect.size.width,
                FlexDirection::Column => child_box.rect.size.height,
            };
            main_cursor += advance + main_gap + spacing;
        }

        positioned_children.extend(line.children);
    }

    let border_box_height = match flex_direction {
        FlexDirection::Row => container_cross_size,
        FlexDirection::Column => get_px(style, "height", max_line_total_main_size_with_gap),
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
        text: None,
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
enum AlignContent {
    FlexStart,
    Center,
    SpaceBetween,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlexWrap {
    Nowrap,
    Wrap,
}

fn get_number(style: &CategorizedComputedStyle, prop: &str, default: f32) -> f32 {
    match prop {
        "flex-grow" => style.reset_flex.flex_grow,
        "flex-shrink" => style.reset_flex.flex_shrink,
        "opacity" => style.reset_effects.opacity,
        _ => default,
    }
}

fn has_explicit_size(style: Option<&CategorizedComputedStyle>, prop: &str) -> bool {
    let Some(style) = style else {
        return false;
    };
    match prop {
        "width" => style.reset_box.width != -1,
        "height" => style.reset_box.height != -1,
        "flex-basis" => style.reset_flex.flex_basis != -1,
        _ => false,
    }
}

fn get_align_self(style: Option<&CategorizedComputedStyle>, default: AlignItems) -> AlignItems {
    let Some(style) = style else {
        return default;
    };
    match style.reset_flex.align_self.to_ascii_lowercase().as_str() {
        "auto" => default,
        "stretch" => AlignItems::Stretch,
        "flex-start" => AlignItems::FlexStart,
        "flex-end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        "baseline" => AlignItems::Baseline,
        _ => default,
    }
}

fn clamp_main_size(
    style: &CategorizedComputedStyle,
    mut size: f32,
    flex_direction: FlexDirection,
    container_width: f32,
    container_height: f32,
) -> f32 {
    let resolve = |stored: i32, ref_size: f32| -> Option<f32> {
        if stored == -1 {
            None
        } else if stored >= crate::style::categorized::WIDTH_PERCENT_BAND {
            Some((stored - crate::style::categorized::WIDTH_PERCENT_BAND) as f32 / 100.0 * ref_size)
        } else {
            Some(stored as f32)
        }
    };

    let (min_stored, max_stored, ref_size) = match flex_direction {
        FlexDirection::Row => (
            style.reset_box.min_width,
            style.reset_box.max_width,
            container_width,
        ),
        FlexDirection::Column => (
            style.reset_box.min_height,
            style.reset_box.max_height,
            container_height,
        ),
    };

    if let Some(max_val) = resolve(max_stored, ref_size)
        && size > max_val
    {
        size = max_val;
    }
    if let Some(min_val) = resolve(min_stored, ref_size)
        && size < min_val
    {
        size = min_val;
    }
    size.max(0.0)
}

fn clamp_cross_size(
    style: &CategorizedComputedStyle,
    mut size: f32,
    flex_direction: FlexDirection,
    container_width: f32,
    container_height: f32,
) -> f32 {
    let resolve = |stored: i32, ref_size: f32| -> Option<f32> {
        if stored == -1 {
            None
        } else if stored >= crate::style::categorized::WIDTH_PERCENT_BAND {
            Some((stored - crate::style::categorized::WIDTH_PERCENT_BAND) as f32 / 100.0 * ref_size)
        } else {
            Some(stored as f32)
        }
    };

    let (min_stored, max_stored, ref_size) = match flex_direction {
        FlexDirection::Row => (
            style.reset_box.min_height,
            style.reset_box.max_height,
            container_height,
        ),
        FlexDirection::Column => (
            style.reset_box.min_width,
            style.reset_box.max_width,
            container_width,
        ),
    };

    if let Some(max_val) = resolve(max_stored, ref_size)
        && size > max_val
    {
        size = max_val;
    }
    if let Some(min_val) = resolve(min_stored, ref_size)
        && size < min_val
    {
        size = min_val;
    }
    size.max(0.0)
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
    fn test_align_self_overrides() {
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
        dom.append_child(container, child1);

        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        dom.append_child(container, child2);

        let child3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child3".into())],
        });
        dom.append_child(container, child3);

        let child4 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child4".into())],
        });
        dom.append_child(container, child4);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: flex-start;
            }
            div {
                height: 50px;
                width: 100px;
            }
            #child1 {
                align-self: center;
            }
            #child2 {
                align-self: flex-end;
            }
            #child3 {
                align-self: auto;
            }
            /* child4 has no align-self property */
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 4);

        // child1: align-self is center, so offset is (200 - 50) / 2 = 75.0
        assert!(approx_eq(container_box.children[0].rect.origin.y, 75.0));

        // child2: align-self is flex-end, so offset is 200 - 50 = 150.0
        assert!(approx_eq(container_box.children[1].rect.origin.y, 150.0));

        // child3: align-self is auto, so follows container's flex-start, which is 0.0
        assert!(approx_eq(container_box.children[2].rect.origin.y, 0.0));

        // child4: no align-self, so follows container's flex-start, which is 0.0
        assert!(approx_eq(container_box.children[3].rect.origin.y, 0.0));
    }

    #[test]
    fn test_align_self_stretch() {
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
        dom.append_child(container, child1);

        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        dom.append_child(container, child2);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                height: 200px;
                align-items: flex-start;
            }
            div {
                width: 100px;
            }
            #child1 {
                align-self: stretch;
            }
            #child2 {
                /* has no explicit height and no align-self, so matches align-items: flex-start and should NOT stretch (it will have height 0 as content height is 0) */
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);

        // child1: align-self is stretch, so height is stretched to 200.0
        assert!(approx_eq(container_box.children[0].rect.size.height, 200.0));

        // child2: follows container's flex-start, height is not stretched
        assert!(approx_eq(container_box.children[1].rect.size.height, 0.0));
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
            #container div {
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

    #[test]
    fn test_align_content_center() {
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
                width: 200px;
                height: 300px;
                align-content: center;
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

        // Under align-content: center, free space of 200px (300 - 100) is split evenly (100px before, 100px after)
        // First line: child0 and child1
        assert!(approx_eq(container_box.children[0].rect.origin.y, 100.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 100.0));

        // Second line: child2 and child3
        assert!(approx_eq(container_box.children[2].rect.origin.y, 150.0));
        assert!(approx_eq(container_box.children[3].rect.origin.y, 150.0));
    }

    #[test]
    fn test_align_content_space_between() {
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
                width: 200px;
                height: 300px;
                align-content: space-between;
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

        // Under align-content: space-between, free space of 200px (300 - 100) is placed between lines
        // First line: child0 and child1 at top (y=0)
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 0.0));

        // Second line: child2 and child3 at bottom (y=250)
        assert!(approx_eq(container_box.children[2].rect.origin.y, 250.0));
        assert!(approx_eq(container_box.children[3].rect.origin.y, 250.0));

        // The gap between line 1 and line 2 is 200.0 (from y=50 to y=250), which is > sum of max_cross_sizes (100.0)
        let line1_end = 50.0;
        let line2_start = container_box.children[2].rect.origin.y;
        let gap = line2_start - line1_end;
        assert!(gap > 100.0);
        assert!(approx_eq(gap, 200.0));
    }

    #[test]
    fn test_flex_shrink_basic() {
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
                flex-direction: row;
                width: 300px;
            }
            #child1 {
                width: 200px;
                flex-shrink: 1;
            }
            #child2 {
                width: 200px;
                flex-shrink: 3;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Container: 300px. Children base width sum: 400px. Negative free space: 100px.
        // Child 1 scaled shrink factor: 1 * 200 = 200.
        // Child 2 scaled shrink factor: 3 * 200 = 600.
        // Total scaled shrink: 800.
        // Child 1 shrink: (200 / 800) * 100 = 25. New width: 200 - 25 = 175.
        // Child 2 shrink: (600 / 800) * 100 = 75. New width: 200 - 75 = 125.
        assert!(approx_eq(container_box.children[0].rect.size.width, 175.0));
        assert!(approx_eq(container_box.children[1].rect.size.width, 125.0));
    }

    #[test]
    fn test_flex_shrink_zero() {
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
                flex-direction: row;
                width: 300px;
            }
            #child1 {
                width: 200px;
                flex-shrink: 0;
            }
            #child2 {
                width: 200px;
                flex-shrink: 1;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Child 1: width remains 200px.
        // Child 2: absorbs all 100px negative space. Width: 200 - 100 = 100px.
        assert!(approx_eq(container_box.children[0].rect.size.width, 200.0));
        assert!(approx_eq(container_box.children[1].rect.size.width, 100.0));
    }

    #[test]
    fn test_flex_shrink_default() {
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
                flex-direction: row;
                width: 300px;
            }
            #child1 {
                width: 250px;
            }
            #child2 {
                width: 150px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Default flex-shrink is 1.
        // Container: 300px. Children base width sum: 400px. Negative free space: 100px.
        // Child 1 scaled shrink factor: 1 * 250 = 250.
        // Child 2 scaled shrink factor: 1 * 150 = 150.
        // Total scaled shrink: 400.
        // Child 1 shrink: (250 / 400) * 100 = 62.5. New width: 250 - 62.5 = 187.5.
        // Child 2 shrink: (150 / 400) * 100 = 37.5. New width: 150 - 37.5 = 112.5.
        assert!(approx_eq(container_box.children[0].rect.size.width, 187.5));
        assert!(approx_eq(container_box.children[1].rect.size.width, 112.5));
    }

    #[test]
    fn test_flex_shrink_column() {
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
                flex-direction: column;
                height: 300px;
            }
            #child1 {
                height: 200px;
                flex-shrink: 1;
            }
            #child2 {
                height: 200px;
                flex-shrink: 3;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Height distribution
        // Container: 300px. Children base height sum: 400px. Negative free space: 100px.
        // Child 1 scaled shrink: (200 / 800) * 100 = 25. New height: 200 - 25 = 175.
        // Child 2 scaled shrink: (600 / 800) * 100 = 75. New height: 200 - 75 = 125.
        assert!(approx_eq(container_box.children[0].rect.size.height, 175.0));
        assert!(approx_eq(container_box.children[1].rect.size.height, 125.0));
    }

    #[test]
    fn test_flex_row_gap() {
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
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                width: 500px;
                column-gap: 10px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
            #child3 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Check positions
        // inner_x defaults to 0.0 (since no margins/padding/border)
        // Child 1 starts at 0.0, ends at 100.0
        // Child 2 starts at 0.0 + 100.0 + 10.0 (gap) = 110.0
        // Child 3 starts at 110.0 + 100.0 + 10.0 (gap) = 220.0
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 110.0));
        assert!(approx_eq(container_box.children[2].rect.origin.x, 220.0));
    }

    #[test]
    fn test_flex_wrap_row_gap() {
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
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                flex-wrap: wrap;
                width: 250px;
                column-gap: 10px;
                row-gap: 8px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
            #child3 {
                width: 100px;
                height: 40px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Wrapping details:
        // Child 1 (100px) + Gap (10px) + Child 2 (100px) = 210px <= 250px.
        // If we tried to fit Child 3: 210px + Gap (10px) + Child 3 (100px) = 320px > 250px, so Child 3 wraps to Line 2.
        // Line 1 contains Child 1 and Child 2. Max cross size = 50.0.
        // Line 2 contains Child 3. Max cross size = 40.0.
        // Since row-gap is 8px, Line 2's cross offset (y) is: Line 1 cross offset (0.0) + Line 1 max height (50.0) + row_gap (8.0) = 58.0.
        assert_eq!(container_box.children.len(), 3);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));

        assert!(approx_eq(container_box.children[1].rect.origin.x, 110.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 0.0));

        assert!(approx_eq(container_box.children[2].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[2].rect.origin.y, 58.0));
    }

    #[test]
    fn test_flex_single_value_gap() {
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
                flex-direction: row;
                width: 300px;
                gap: 12px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Since column-gap and row-gap are absent, `gap` (12px) acts as both column-gap and row-gap.
        // Since flex-direction is row, main_gap is column-gap = 12px.
        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 112.0));
    }

    #[test]
    fn test_flex_gap_justify_content() {
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
                flex-direction: row;
                width: 300px;
                justify-content: center;
                gap: 20px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Container: 300px. Children: 100px + 100px = 200px. Gap: 20px.
        // Total main size of items + gaps = 220px.
        // Free space = 300 - 220 = 80px.
        // justify-content: center -> start cursor offset = 80 / 2 = 40px.
        // Child 1 starts at 40px.
        // Child 2 starts at 40 + 100 (child1) + 20 (gap) = 160px.
        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 40.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 160.0));
    }

    #[test]
    fn test_flex_wrap_column_auto_height() {
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
                flex-direction: column;
                flex-wrap: wrap;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 50.0));
    }

    #[test]
    fn test_flex_wrap_column_explicit_height() {
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
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: column;
                flex-wrap: wrap;
                height: 120px;
                column-gap: 15px;
                row-gap: 10px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
            #child3 {
                width: 80px;
                height: 40px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Height is 120px.
        // child1: height=50px.
        // gap: row_gap=10px.
        // child2: height=50px.
        // child1 (50) + gap (10) + child2 (50) = 110px <= 120px. They fit on Line 1.
        // If child3 were to fit: 110 + gap (10) + child3 (40) = 160px > 120px.
        // So child3 wraps to Line 2!
        // Line 1: child1, child2
        // Line 2: child3
        // Line 1 cross size: max(child1.width, child2.width) = max(100, 100) = 100px.
        // Line 2 cross size: max(child3.width) = 80px.
        // Line 2 cross offset (x): Line 1 cross offset (0) + Line 1 cross size (100) + col_gap (15) = 115px.
        assert_eq!(container_box.children.len(), 3);
        // child1: x=0, y=0, w=100, h=50
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[0].rect.size.width, 100.0));
        assert!(approx_eq(container_box.children[0].rect.size.height, 50.0));

        // child2: x=0, y=50+10=60, w=100, h=50
        assert!(approx_eq(container_box.children[1].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 60.0));
        assert!(approx_eq(container_box.children[1].rect.size.width, 100.0));
        assert!(approx_eq(container_box.children[1].rect.size.height, 50.0));

        // child3: x=115, y=0, w=80, h=40
        assert!(approx_eq(container_box.children[2].rect.origin.x, 115.0));
        assert!(approx_eq(container_box.children[2].rect.origin.y, 0.0));
        assert!(approx_eq(container_box.children[2].rect.size.width, 80.0));
        assert!(approx_eq(container_box.children[2].rect.size.height, 40.0));
    }

    #[test]
    fn test_flex_two_value_gap() {
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
                flex-direction: row;
                width: 300px;
                gap: 10px 20px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // gap: 10px 20px -> row-gap = 10px, column-gap = 20px.
        // flex-direction: row -> main_gap is column-gap = 20px.
        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 120.0)); // child1 width (100) + col_gap (20)
    }

    #[test]
    fn test_flex_two_value_gap_wrap() {
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
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                flex-wrap: wrap;
                width: 250px;
                gap: 15px 30px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
            #child3 {
                width: 100px;
                height: 40px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // gap: 15px 30px -> row-gap (cross-gap in row layout) = 15px, column-gap (main-gap in row layout) = 30px.
        // Line 1: Child 1 (100px) + Gap (30px) + Child 2 (100px) = 230px <= 250px.
        // If Child 3 tried to fit: 230px + Gap (30px) + Child 3 (100px) = 360px > 250px. So Child 3 wraps to Line 2.
        // Line 1 height = 50px.
        // Line 2 y-offset = Line 1 cross offset (0.0) + Line 1 height (50.0) + row-gap (15.0) = 65.0.
        assert_eq!(container_box.children.len(), 3);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[0].rect.origin.y, 0.0));

        assert!(approx_eq(container_box.children[1].rect.origin.x, 130.0));
        assert!(approx_eq(container_box.children[1].rect.origin.y, 0.0));

        assert!(approx_eq(container_box.children[2].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[2].rect.origin.y, 65.0));
    }

    #[test]
    fn test_flex_gap_precedence() {
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
                flex-direction: row;
                width: 300px;
                column-gap: 5px;
                gap: 10px 20px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // column-gap: 5px has precedence over gap column-gap (20px).
        // So main_gap should be 5px.
        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 105.0)); // child1 width (100) + col_gap (5)
    }

    #[test]
    fn test_flex_negative_gap_clamping() {
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
                flex-direction: row;
                width: 300px;
            }
            #child1 {
                width: 100px;
                height: 50px;
            }
            #child2 {
                width: 100px;
                height: 50px;
            }
        ",
        );
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually inject negative row-gap and column-gap values to bypass any parser/validator clamps
        // and test the layout's defensive clamping directly.
        if let Some(container_style) = styles.get_mut(&container) {
            let mut flex = (*container_style.reset_flex).clone();
            flex.row_gap = -15;
            flex.column_gap = -25;
            container_style.reset_flex = std::sync::Arc::new(flex);
        }

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        // Check positions:
        // With defensive clamping, col_gap should be floored to 0.0, meaning:
        // child2 x should be child1 x + child1 width + 0.0 = 100.0 (not 75.0!)
        assert_eq!(container_box.children.len(), 2);
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 100.0));
    }

    #[test]
    fn test_flex_order_sequencing() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "container".into())],
        });
        dom.append_child(doc, container);

        // child1: order 2
        let child1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child1".into())],
        });
        // child2: order 0
        let child2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child2".into())],
        });
        // child3: order 1
        let child3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child3".into())],
        });
        // child4: order -1 (negative order)
        let child4 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child4".into())],
        });
        // child5: order 0 (equal order to child2, should be stable, meaning child2 comes before child5)
        let child5 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("id".into(), "child5".into())],
        });

        // Append in order of DOM: child1, child2, child3, child4, child5
        dom.append_child(container, child1);
        dom.append_child(container, child2);
        dom.append_child(container, child3);
        dom.append_child(container, child4);
        dom.append_child(container, child5);

        let stylesheet = parse_stylesheet(
            "
            #container {
                display: flex;
                flex-direction: row;
                width: 500px;
            }
            div {
                width: 100px;
                height: 50px;
            }
            #child1 {
                order: 2;
            }
            #child2 {
                order: 0;
            }
            #child3 {
                order: 1;
            }
            #child4 {
                order: -1;
            }
            #child5 {
                order: 0;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 5);

        // Expected sorted order:
        // 1. child4 (order -1)
        // 2. child2 (order 0)
        // 3. child5 (order 0, stable after child2)
        // 4. child3 (order 1)
        // 5. child1 (order 2)

        // Check node mappings
        assert_eq!(container_box.children[0].node, Some(child4));
        assert_eq!(container_box.children[1].node, Some(child2));
        assert_eq!(container_box.children[2].node, Some(child5));
        assert_eq!(container_box.children[3].node, Some(child3));
        assert_eq!(container_box.children[4].node, Some(child1));

        // Check positions: width is 100.0, so:
        // child4 (0th): x = 0.0
        // child2 (1st): x = 100.0
        // child5 (2nd): x = 200.0
        // child3 (3rd): x = 300.0
        // child1 (4th): x = 400.0
        assert!(approx_eq(container_box.children[0].rect.origin.x, 0.0));
        assert!(approx_eq(container_box.children[1].rect.origin.x, 100.0));
        assert!(approx_eq(container_box.children[2].rect.origin.x, 200.0));
        assert!(approx_eq(container_box.children[3].rect.origin.x, 300.0));
        assert!(approx_eq(container_box.children[4].rect.origin.x, 400.0));
    }

    #[test]
    fn test_flex_basis_resolution() {
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
                flex-direction: row;
                width: 500px;
            }
            div {
                height: 50px;
            }
            #child1 {
                width: 100px;
                flex-basis: 150px;
            }
            #child2 {
                width: 100px;
                /* flex-basis not specified, so uses width */
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 width should be its flex-basis (150px) instead of 100px
        assert!(approx_eq(container_box.children[0].rect.size.width, 150.0));
        // child2 width should remain its default/specified width (100px)
        assert!(approx_eq(container_box.children[1].rect.size.width, 100.0));
    }

    #[test]
    fn test_flex_main_min_max_clamping() {
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
                flex-direction: row;
                width: 500px;
            }
            div {
                height: 50px;
            }
            #child1 {
                width: 250px;
                max-width: 200px;
            }
            #child2 {
                width: 50px;
                min-width: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 width should be clamped down to max-width: 200.0 from 250.0
        assert!(approx_eq(container_box.children[0].rect.size.width, 200.0));
        // child2 width should be clamped up to min-width: 100.0 from 50.0
        assert!(approx_eq(container_box.children[1].rect.size.width, 100.0));
    }

    #[test]
    fn test_flex_cross_min_max_clamping_on_stretch() {
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
                flex-direction: row;
                height: 200px;
                align-items: stretch;
            }
            div {
                width: 100px;
            }
            #child1 {
                /* stretch would make height 200px, but clamped by max-height */
                max-height: 150px;
            }
            #child2 {
                /* stretch would make height 200px, but min-height is 250px */
                min-height: 250px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let container_box =
            layout_flex_container(&dom, &styles, container, 800.0, 0.0, 0.0, 0).unwrap();

        assert_eq!(container_box.children.len(), 2);
        // child1 height should be clamped to max-height (150px)
        assert!(approx_eq(container_box.children[0].rect.size.height, 150.0));
        // child2 height should be clamped to min-height (250px)
        assert!(approx_eq(container_box.children[1].rect.size.height, 250.0));
    }
}
