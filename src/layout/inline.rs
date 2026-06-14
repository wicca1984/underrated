use crate::ascii::is_html_whitespace;
use crate::dom::{Dom, NodeData};
use crate::geom::{Point, Rect, Size};
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

fn is_inline_block(styles: &HashMap<NodeId, CategorizedComputedStyle>, node: NodeId) -> bool {
    if let Some(style) = styles.get(&node) {
        style.reset_box.display == "inline-block"
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

fn get_inherited_letter_spacing(
    node: NodeId,
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> f32 {
    let mut current = Some(node);
    while let Some(curr_node) = current {
        if let Some(style) = styles.get(&curr_node)
            && style.inherited_text.letter_spacing != -1
        {
            return style.inherited_text.letter_spacing as f32;
        }
        current = dom.parent(curr_node);
    }
    0.0
}

fn get_inherited_word_spacing(
    node: NodeId,
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> Option<f32> {
    let mut current = Some(node);
    while let Some(curr_node) = current {
        if let Some(style) = styles.get(&curr_node)
            && style.inherited_text.word_spacing != -1
        {
            return Some(style.inherited_text.word_spacing as f32);
        }
        current = dom.parent(curr_node);
    }
    None
}

fn get_font_size(style: &CategorizedComputedStyle) -> f32 {
    style.inherited_text.font_size as f32
}

fn find_last_fragment_baseline(
    lb: &LayoutBox,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    _dom: &Dom,
) -> Option<f32> {
    if lb.text.is_some() {
        return Some(lb.rect.origin.y + lb.rect.size.height);
    }

    if let Some(node_id) = lb.node
        && is_inline_block(styles, node_id)
    {
        let is_visible = styles
            .get(&node_id)
            .map(|style| style.reset_box.overflow == "visible")
            .unwrap_or(true);
        if is_visible {
            for child in lb.children.iter().rev() {
                if let Some(bl) = find_last_fragment_baseline(child, styles, _dom) {
                    return Some(bl);
                }
            }
        }
        let margin_bottom = styles
            .get(&node_id)
            .map(|style| crate::layout::get_px(style, "margin-bottom", 0.0))
            .unwrap_or(0.0);
        return Some(lb.rect.origin.y + lb.rect.size.height + margin_bottom);
    }

    for child in lb.children.iter().rev() {
        if let Some(bl) = find_last_fragment_baseline(child, styles, _dom) {
            return Some(bl);
        }
    }

    None
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
            let font_size = get_font_size(style);
            let val = style.reset_box.vertical_align;
            let shift = match val {
                -1 => 0.0,
                -2 => 0.2 * font_size,
                -3 => -0.2 * font_size,
                -4 => -line_height + border_box_height,
                -5 => 0.0,
                -6 => -0.25 * font_size + (border_box_height / 2.0),
                v if v >= 150000 => {
                    // Percentage band: relative to the line-height.
                    let pct = (v - 200000) as f32;
                    -(pct / 100.0) * line_height
                }
                v if v >= 50000 => {
                    let raise = (v - 100000) as f32;
                    -raise
                }
                v => {
                    let raise = v as f32;
                    -raise
                }
            };
            total_shift += shift;
        }

        current = dom.parent(curr_node);
    }

    total_shift
}

fn is_pure_neutral(s: &str) -> bool {
    for c in s.chars() {
        if c.is_alphabetic() {
            return false;
        }
    }
    true
}

fn resolve_line_children_directions(
    children: &[LayoutBox],
    base_direction: &'static str, // "ltr" or "rtl"
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> Vec<&'static str> {
    let mut resolved = vec!["neutral"; children.len()];

    // Step 1: Assign strong directions
    for (i, child) in children.iter().enumerate() {
        if let Some(ref text) = child.text
            && is_pure_neutral(text)
        {
            resolved[i] = "neutral";
            continue;
        }
        if let Some(node_id) = child.node
            && let Some(style) = styles.get(&node_id)
        {
            let dir = style.inherited_text.direction.as_str();
            if dir == "rtl" || dir == "ltr" {
                resolved[i] = if dir == "rtl" { "rtl" } else { "ltr" };
                continue;
            }
        }
    }

    // Step 2: Resolve Neutral runs
    let mut i = 0;
    while i < children.len() {
        if resolved[i] == "neutral" {
            let start = i;
            while i < children.len() && resolved[i] == "neutral" {
                i += 1;
            }
            let end = i; // exclusive

            // The left neighbor direction
            let left_dir = if start > 0 {
                resolved[start - 1]
            } else {
                base_direction
            };

            // The right neighbor direction
            let right_dir = if end < children.len() {
                resolved[end]
            } else {
                base_direction
            };

            let final_dir = if left_dir == right_dir {
                left_dir
            } else {
                base_direction
            };

            resolved[start..end].fill(final_dir);
        } else {
            i += 1;
        }
    }

    resolved
}

#[allow(clippy::too_many_arguments)]
fn create_line_box_adjusted(
    dom: &Dom,
    block_container: Option<NodeId>,
    mut children: Vec<LayoutBox>,
    offset_x: f32,
    offset_y: f32,
    mut width: f32,
    line_height: f32,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    text_align: &str,
    containing_width: f32,
    is_last_line: bool,
) -> LayoutBox {
    let is_center_element = block_container
        .and_then(|id| dom.data(id))
        .is_some_and(|data| matches!(data, NodeData::Element { name, .. } if name == "center"));

    let (style_text_align, direction) = if is_center_element {
        ("center", "ltr")
    } else if let Some(bc_id) = block_container
        && let Some(style) = styles.get(&bc_id)
    {
        (
            style.inherited_text.text_align.as_str(),
            style.inherited_text.direction.as_str(),
        )
    } else {
        (text_align, "ltr")
    };

    let is_rtl = direction == "rtl";

    let base_align = match style_text_align {
        "start" => {
            if is_rtl {
                "right"
            } else {
                "left"
            }
        }
        "end" => {
            if is_rtl {
                "left"
            } else {
                "right"
            }
        }
        _ => text_align,
    };

    let resolved_align = if is_last_line {
        let raw_last_align = block_container
            .and_then(|id| styles.get(&id))
            .map(|style| style.inherited_text.text_align_last.as_str())
            .unwrap_or("auto");

        let last_align = match raw_last_align {
            "start" => {
                if is_rtl {
                    "right"
                } else {
                    "left"
                }
            }
            "end" => {
                if is_rtl {
                    "left"
                } else {
                    "right"
                }
            }
            other => other,
        };

        if last_align == "auto" {
            if base_align == "justify" {
                if is_rtl { "right" } else { "left" }
            } else {
                base_align
            }
        } else {
            last_align
        }
    } else {
        base_align
    };

    let target_trim_align = match resolved_align {
        "start" => {
            if is_rtl {
                "right"
            } else {
                "left"
            }
        }
        "end" => {
            if is_rtl {
                "left"
            } else {
                "right"
            }
        }
        other => other,
    };

    if target_trim_align == "right" || target_trim_align == "center" {
        // Calculate trimmed width by ignoring trailing collapsible whitespace
        let mut last_non_space_index = None;

        // Find the last child that is not entirely collapsible/hanging space
        for (i, child) in children.iter().enumerate().rev() {
            if let Some(ref text) = child.text
                && let Some(node_id) = child.node
            {
                let style_ws = if let Some(style) = styles.get(&node_id) {
                    style.inherited_text.white_space.as_str()
                } else {
                    "normal"
                };
                let collapse_or_hang =
                    matches!(style_ws, "nowrap" | "pre-line" | "normal" | "pre-wrap");

                if collapse_or_hang {
                    if text.chars().all(|c| c == ' ') {
                        // This is a collapsible space. Skip it!
                        continue;
                    } else if text.ends_with(' ') {
                        // It ends with space. This is the last non-space child, but we need to subtract the trailing space width!
                        last_non_space_index = Some((i, true));
                        break;
                    }
                }
            }
            // If it's not a collapsible text child, or doesn't end with space, it is our last non-space child!
            last_non_space_index = Some((i, false));
            break;
        }

        if let Some((idx, ends_with_space)) = last_non_space_index {
            // Trim the width of all children after idx to 0.0
            for child in children.iter_mut().skip(idx + 1) {
                child.rect.size.width = 0.0;
            }

            if ends_with_space {
                let last_child = &children[idx];
                if let Some(ref text) = last_child.text
                    && let Some(node_id) = last_child.node
                {
                    let trimmed_text = text.trim_end_matches(' ').to_string();
                    let letter_spacing = get_inherited_letter_spacing(node_id, dom, styles);
                    let font = crate::font::BitmapFont::builtin();
                    let char_count = trimmed_text.chars().count();
                    let base_width = font.measure(&trimmed_text) as f32;
                    let last_child_width = if char_count > 1 {
                        base_width + (char_count - 1) as f32 * letter_spacing
                    } else {
                        base_width
                    };
                    children[idx].rect.size.width = last_child_width;
                }
            }

            let mut trimmed_width = 0.0f32;
            for child in children.iter().take(idx + 1) {
                let right_edge = child.rect.origin.x + child.rect.size.width - offset_x;
                if right_edge > trimmed_width {
                    trimmed_width = right_edge;
                }
            }
            width = trimmed_width;
        } else {
            // All children are collapsible/hanging space
            for child in &mut children {
                child.rect.size.width = 0.0;
            }
            width = 0.0;
        }
    }

    // For each child, adjust its Y position based on its baseline.
    let line_box_bottom_y = offset_y + line_height;

    for child in &mut children {
        let mut target_y;
        let border_box_height = child.rect.size.height;

        let margin_bottom = child
            .node
            .and_then(|id| styles.get(&id))
            .map(|style| crate::layout::get_px(style, "margin-bottom", 0.0))
            .unwrap_or(0.0);

        if let Some(abs_baseline_y) = find_last_fragment_baseline(child, styles, dom) {
            let baseline_offset_from_top = abs_baseline_y - child.rect.origin.y;
            target_y = line_box_bottom_y - baseline_offset_from_top;
        } else {
            target_y = line_box_bottom_y - margin_bottom - border_box_height;
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

    // Bidi layout and neutral run resolution
    {
        struct BidiRun {
            direction: &'static str,
            children: Vec<(LayoutBox, f32)>,
        }

        let mut gaps = Vec::with_capacity(children.len());
        let mut prev_right = offset_x;
        for child in &children {
            let gap = child.rect.origin.x - prev_right;
            gaps.push(gap);
            prev_right = child.rect.origin.x + child.rect.size.width;
        }

        let static_dir = if direction == "rtl" { "rtl" } else { "ltr" };
        let resolved_dirs = resolve_line_children_directions(&children, static_dir, styles);
        let mut runs: Vec<BidiRun> = Vec::new();
        for ((child, gap), dir) in children.into_iter().zip(gaps).zip(resolved_dirs) {
            if let Some(last_run) = runs.last_mut()
                && last_run.direction == dir
            {
                last_run.children.push((child, gap));
                continue;
            }
            runs.push(BidiRun {
                direction: dir,
                children: vec![(child, gap)],
            });
        }

        let base_is_rtl = direction == "rtl";
        if base_is_rtl {
            runs.reverse();
        }

        let mut final_children = Vec::new();
        for mut run in runs {
            if run.direction == "rtl" {
                run.children.reverse();
            }
            final_children.extend(run.children);
        }

        let mut cur_x = offset_x;
        let mut resolved_children = Vec::new();
        for (mut child, gap) in final_children {
            cur_x += gap;
            let delta_x = cur_x - child.rect.origin.x;
            if delta_x != 0.0 {
                shift_x(&mut child, delta_x);
            }
            cur_x += child.rect.size.width;
            resolved_children.push(child);
        }
        children = resolved_children;
    }

    let final_align = match resolved_align {
        "start" => {
            if is_rtl {
                "right"
            } else {
                "left"
            }
        }
        "end" => {
            if is_rtl {
                "left"
            } else {
                "right"
            }
        }
        other => other,
    };

    // Adjust X positions based on text-align centering/right alignment
    let delta_x = match final_align {
        "center" => {
            let val = (containing_width - width) / 2.0;
            if is_rtl { val } else { val.max(0.0) }
        }
        "right" => {
            let val = containing_width - width;
            if is_rtl { val } else { val.max(0.0) }
        }
        _ => 0.0,
    };

    if delta_x != 0.0 {
        for child in &mut children {
            shift_x(child, delta_x);
        }
    }

    // TODO(spec): text-align justify v1 — distributes slack across inter-word gaps on non-last lines only (unless overridden by text-align-last); last-line/forced-break detection is simple word-count based; RTL, percentage widths, hyphenation, and justify-by-character are out of scope.
    if resolved_align == "justify" && children.len() >= 2 {
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

    // text-overflow: ellipsis implementation
    let has_ellipsis = if let Some(bc_id) = block_container
        && let Some(style) = styles.get(&bc_id)
    {
        style.reset_effects.text_overflow == "ellipsis" && style.reset_box.overflow != "visible"
    } else {
        false
    };

    if has_ellipsis {
        let overflow_left = delta_x < 0.0;
        let measure_for_node = |node_id: NodeId, s: &str| -> f32 {
            let font = crate::font::BitmapFont::builtin();
            let char_count = s.chars().count();
            let base_width = font.measure(s) as f32;
            let letter_spacing = get_inherited_letter_spacing(node_id, dom, styles);
            if char_count > 1 {
                base_width + (char_count - 1) as f32 * letter_spacing
            } else {
                base_width
            }
        };

        if overflow_left {
            let mut clip_idx = None;
            for (idx, child) in children.iter().enumerate().rev() {
                if child.rect.origin.x < offset_x {
                    clip_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = clip_idx {
                let mut clipped_children = children.split_off(idx);
                std::mem::swap(&mut children, &mut clipped_children);
                if !children.is_empty() {
                    let child = &mut children[0];
                    if let Some(ref text) = child.text
                        && let Some(node_id) = child.node
                    {
                        let mut fit_text = "…".to_string();
                        let max_width = child.rect.origin.x + child.rect.size.width - offset_x;
                        if max_width > 0.0 {
                            let mut best_suffix = "".to_string();
                            for i in 0..text.len() {
                                if text.is_char_boundary(i) {
                                    let suffix = &text[i..];
                                    let candidate = format!("…{}", suffix);
                                    let w = measure_for_node(node_id, &candidate);
                                    if w <= max_width {
                                        best_suffix = suffix.to_string();
                                        break;
                                    }
                                }
                            }
                            fit_text = format!("…{}", best_suffix);
                        }
                        let new_width = measure_for_node(node_id, &fit_text);
                        child.rect.origin.x =
                            child.rect.origin.x + child.rect.size.width - new_width;
                        child.rect.size.width = new_width;
                        child.text = Some(fit_text);
                    } else if let Some(node_id) = child.node {
                        let w = measure_for_node(node_id, "…");
                        child.rect.origin.x = child.rect.origin.x + child.rect.size.width - w;
                        child.rect.size.width = w;
                        child.text = Some("…".to_string());
                    }
                }
            }
        } else {
            let mut clip_idx = None;
            for (idx, child) in children.iter().enumerate() {
                if child.rect.origin.x + child.rect.size.width > offset_x + containing_width {
                    clip_idx = Some(idx);
                    break;
                }
            }
            if let Some(idx) = clip_idx {
                children.truncate(idx + 1);
                let child = &mut children[idx];
                if let Some(ref text) = child.text
                    && let Some(node_id) = child.node
                {
                    let max_width = offset_x + containing_width - child.rect.origin.x;
                    let mut fit_text = "…".to_string();
                    if max_width > 0.0 {
                        let mut best_prefix = "".to_string();
                        for i in (0..=text.len()).rev() {
                            if text.is_char_boundary(i) {
                                let prefix = &text[..i];
                                let candidate = format!("{}…", prefix);
                                let w = measure_for_node(node_id, &candidate);
                                if w <= max_width {
                                    best_prefix = prefix.to_string();
                                    break;
                                }
                            }
                        }
                        fit_text = format!("{}…", best_prefix);
                    }
                    child.rect.size.width = measure_for_node(node_id, &fit_text);
                    child.text = Some(fit_text);
                } else if let Some(node_id) = child.node {
                    child.text = Some("…".to_string());
                    child.rect.size.width = measure_for_node(node_id, "…");
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

    let mut push_line_box = |children: &mut Vec<LayoutBox>,
                             line_cx: f32,
                             is_last: bool,
                             current_lh: f32,
                             cy: f32|
     -> f32 {
        let line_box = create_line_box_adjusted(
            dom,
            block_container,
            std::mem::take(children),
            offset_x,
            offset_y + cy,
            line_cx,
            current_lh,
            styles,
            text_align,
            containing_width,
            is_last,
        );
        let height = line_box.rect.size.height;
        line_boxes.push(line_box);
        height
    };

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
                        if let Some(n) = style.inherited_text.line_height_number {
                            node_line_height = n * get_font_size(style);
                        } else if style.inherited_text.line_height
                            != crate::style::categorized::LINE_HEIGHT_NORMAL
                        {
                            node_line_height = style.inherited_text.line_height as f32;
                        }
                    }
                    let letter_spacing = get_inherited_letter_spacing(node, dom, styles);
                    let node_word_spacing =
                        get_inherited_word_spacing(node, dom, styles).unwrap_or(word_spacing);
                    current_line_height = current_line_height.max(node_line_height);

                    let measure_text = |s: &str| -> f32 {
                        let char_count = s.chars().count();
                        let base_width = font.measure(s) as f32;
                        if char_count > 1 {
                            base_width + (char_count - 1) as f32 * letter_spacing
                        } else {
                            base_width
                        }
                    };

                    let style_ws = if let Some(style) = styles.get(&node) {
                        style.inherited_text.white_space.as_str()
                    } else {
                        "normal"
                    };

                    let (collapse, preserve_newlines, allow_wrap) = match style_ws {
                        "nowrap" => (true, false, false),
                        "pre" => (false, true, false),
                        "pre-wrap" | "break-spaces" => (false, true, true),
                        "pre-line" => (true, true, true),
                        _ => (true, false, true),
                    };

                    let style_wb = if let Some(style) = styles.get(&node) {
                        style.inherited_text.word_break.as_str()
                    } else {
                        "normal"
                    };

                    let style_lb = if let Some(style) = styles.get(&node) {
                        style.inherited_text.line_break.as_str()
                    } else {
                        "auto"
                    };

                    let break_all = style_wb == "break-all" || style_lb == "anywhere";

                    let break_word = if let Some(style) = styles.get(&node) {
                        style.inherited_text.overflow_wrap == "break-word"
                            || style.inherited_text.overflow_wrap == "anywhere"
                    } else {
                        false
                    };

                    let tab_size = if let Some(style) = styles.get(&node) {
                        style.inherited_text.tab_size as usize
                    } else {
                        8
                    };

                    let preprocessed = preprocess_text(text, collapse, preserve_newlines, tab_size);

                    let transformed = if let Some(style) = styles.get(&node) {
                        let text_transform = style.inherited_text.text_transform.as_str();
                        apply_text_transform(&preprocessed, &text_transform.to_ascii_lowercase())
                    } else {
                        preprocessed
                    };

                    // spec: CSS Text Module Level 3, §3 (White Space Processing)
                    let segments: Vec<&str> = transformed.split('\n').collect();

                    for (i, segment) in segments.iter().enumerate() {
                        if i > 0 {
                            // Force a line break!
                            let lh = push_line_box(
                                &mut current_line_children,
                                cursor_x,
                                true,
                                current_line_height,
                                cursor_y,
                            );
                            cursor_x = 0.0;
                            cursor_y += lh;
                            current_line_height = node_line_height;
                        }

                        let style_hyphens = if let Some(style) = styles.get(&node) {
                            style.inherited_text.hyphens.as_str()
                        } else {
                            "manual"
                        };

                        let words = segment.split_inclusive(' ');

                        for word in words {
                            if word.is_empty() {
                                continue;
                            }

                            let word_stripped = strip_soft_hyphens(word);
                            let word_width = measure_text(&word_stripped);

                            let mut check_width = word_width;
                            if collapse && word_stripped.ends_with(' ') {
                                let trimmed = word_stripped.trim_end_matches(' ');
                                check_width = measure_text(trimmed);
                            } else if style_ws == "pre-wrap" && word_stripped == " " {
                                check_width = 0.0;
                            }

                            let is_hanging_space = style_ws == "pre-wrap" && word_stripped == " ";
                            if !allow_wrap
                                || is_hanging_space
                                || cursor_x + check_width <= containing_width
                            {
                                if collapse
                                    && current_line_children.is_empty()
                                    && word_stripped == " "
                                {
                                    continue;
                                }

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
                                    text: Some(word_stripped.clone()),
                                });
                                cursor_x += word_width;
                                if word_stripped.ends_with(' ') {
                                    cursor_x += node_word_spacing;
                                }
                            } else {
                                let mut rem_word = word.to_string();
                                while !rem_word.is_empty() {
                                    let rem_word_stripped = strip_soft_hyphens(&rem_word);
                                    let rem_width = measure_text(&rem_word_stripped);

                                    let mut check_width = rem_width;
                                    if collapse && rem_word_stripped.ends_with(' ') {
                                        let trimmed = rem_word_stripped.trim_end_matches(' ');
                                        check_width = measure_text(trimmed);
                                    } else if style_ws == "pre-wrap" && rem_word_stripped == " " {
                                        check_width = 0.0;
                                    }

                                    let is_hanging_space =
                                        style_ws == "pre-wrap" && rem_word_stripped == " ";
                                    if is_hanging_space
                                        || cursor_x + check_width <= containing_width
                                    {
                                        if !(collapse
                                            && current_line_children.is_empty()
                                            && rem_word_stripped == " ")
                                        {
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
                                                text: Some(rem_word_stripped),
                                            });
                                            cursor_x += rem_width;
                                            if rem_word.ends_with(' ') {
                                                cursor_x += node_word_spacing;
                                            }
                                        }
                                        break;
                                    }

                                    let mut opportunities = Vec::new();
                                    let chars: Vec<char> = rem_word.chars().collect();
                                    let mut current_byte_idx = 0;
                                    for &c in &chars {
                                        if c == '\u{00AD}' {
                                            if style_hyphens != "none" {
                                                let prefix = &rem_word[..current_byte_idx];
                                                let suffix =
                                                    &rem_word[current_byte_idx + c.len_utf8()..];
                                                opportunities.push((
                                                    current_byte_idx,
                                                    prefix,
                                                    suffix,
                                                    true,
                                                ));
                                            }
                                        } else if c == '\u{200B}'
                                            || c == '-'
                                            || c == '\u{2010}'
                                            || (is_cjk(c) && style_wb != "keep-all")
                                        {
                                            let end_idx = current_byte_idx + c.len_utf8();
                                            let prefix = &rem_word[..end_idx];
                                            let suffix = &rem_word[end_idx..];
                                            opportunities.push((end_idx, prefix, suffix, false));
                                        }
                                        current_byte_idx += c.len_utf8();
                                    }

                                    let mut best_opp = None;
                                    for opp in opportunities {
                                        let (split_idx, prefix, suffix, is_shy) = opp;
                                        let prefix_stripped = strip_soft_hyphens(prefix);
                                        let prefix_with_hyphen = if is_shy {
                                            format!("{}-", prefix_stripped)
                                        } else {
                                            prefix_stripped
                                        };
                                        let prefix_width = measure_text(&prefix_with_hyphen);
                                        if cursor_x + prefix_width <= containing_width
                                            || (cursor_x == 0.0 && best_opp.is_none())
                                        {
                                            best_opp = Some((
                                                split_idx,
                                                prefix_with_hyphen,
                                                suffix.to_string(),
                                                prefix_width,
                                            ));
                                        }
                                    }

                                    if let Some((_, prefix_with_hyphen, suffix, prefix_width)) =
                                        best_opp
                                    {
                                        current_line_children.push(LayoutBox {
                                            node: Some(node),
                                            rect: Rect {
                                                origin: Point {
                                                    x: offset_x + cursor_x,
                                                    y: offset_y + cursor_y,
                                                },
                                                size: Size {
                                                    width: prefix_width,
                                                    height: node_line_height,
                                                },
                                            },
                                            children: Vec::new(),
                                            text: Some(prefix_with_hyphen),
                                        });
                                        cursor_x += prefix_width;

                                        let lh = push_line_box(
                                            &mut current_line_children,
                                            cursor_x,
                                            false,
                                            current_line_height,
                                            cursor_y,
                                        );
                                        cursor_x = 0.0;
                                        cursor_y += lh;
                                        current_line_height = node_line_height;
                                        rem_word = suffix;
                                    } else {
                                        let mut chars_iter = rem_word_stripped.char_indices();
                                        let first_char_fits =
                                            if let Some((_, first_c)) = chars_iter.next() {
                                                let first_char_width =
                                                    measure_text(&first_c.to_string());
                                                cursor_x + first_char_width <= containing_width
                                            } else {
                                                false
                                            };

                                        if cursor_x > 0.0 && !(break_all && first_char_fits) {
                                            let lh = push_line_box(
                                                &mut current_line_children,
                                                cursor_x,
                                                false,
                                                current_line_height,
                                                cursor_y,
                                            );
                                            cursor_x = 0.0;
                                            cursor_y += lh;
                                            current_line_height = node_line_height;
                                            continue;
                                        }

                                        let should_break = break_all
                                            || (break_word && rem_width > containing_width);

                                        if should_break {
                                            let mut chars_iter = rem_word_stripped.char_indices();
                                            let (first_idx, first_c) = match chars_iter.next() {
                                                Some(val) => val,
                                                None => break,
                                            };
                                            let first_char_end = first_idx + first_c.len_utf8();
                                            let first_char_width =
                                                measure_text(&rem_word_stripped[..first_char_end]);

                                            let mut split_index = first_char_end;
                                            let mut last_valid_width = first_char_width;

                                            for (idx, c) in chars_iter {
                                                let candidate_end = idx + c.len_utf8();
                                                let candidate_width = measure_text(
                                                    &rem_word_stripped[..candidate_end],
                                                );
                                                if cursor_x + candidate_width <= containing_width {
                                                    split_index = candidate_end;
                                                    last_valid_width = candidate_width;
                                                } else {
                                                    break;
                                                }
                                            }

                                            let prefix =
                                                rem_word_stripped[..split_index].to_string();

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
                                                text: Some(prefix.clone()),
                                            });
                                            cursor_x += last_valid_width;
                                            if prefix.ends_with(' ') {
                                                cursor_x += node_word_spacing;
                                            }

                                            let lh = push_line_box(
                                                &mut current_line_children,
                                                cursor_x,
                                                false,
                                                current_line_height,
                                                cursor_y,
                                            );
                                            cursor_x = 0.0;
                                            cursor_y += lh;
                                            current_line_height = node_line_height;

                                            rem_word = rem_word_stripped[split_index..].to_string();
                                        } else {
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
                                                text: Some(rem_word_stripped),
                                            });
                                            cursor_x += rem_width;
                                            if rem_word.ends_with(' ') {
                                                cursor_x += node_word_spacing;
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                NodeData::Element { name, .. } => {
                    if name.eq_ignore_ascii_case("br") {
                        if styles
                            .get(&node)
                            .is_some_and(|style| style.reset_box.display == "none")
                        {
                            continue;
                        }
                        // Force a line break!
                        let lh = push_line_box(
                            &mut current_line_children,
                            cursor_x,
                            true,
                            current_line_height,
                            cursor_y,
                        );
                        cursor_x = 0.0;
                        cursor_y += lh;
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
                                let lh = push_line_box(
                                    &mut current_line_children,
                                    cursor_x,
                                    false,
                                    current_line_height,
                                    cursor_y,
                                );
                                cursor_x = 0.0;
                                cursor_y += lh;
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
                        } else if style.reset_box.display == "inline" {
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
        let lh = push_line_box(
            &mut current_line_children,
            cursor_x,
            true,
            current_line_height,
            cursor_y,
        );
        cursor_y += lh;
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

fn is_cjk(c: char) -> bool {
    let u = c as u32;
    (0x4E00..=0x9FFF).contains(&u) || // CJK Unified Ideographs
    (0x3040..=0x309F).contains(&u) || // Hiragana
    (0x30A0..=0x30FF).contains(&u) || // Katakana
    (0xAC00..=0xD7AF).contains(&u) // Hangul Syllables
}

fn strip_soft_hyphens(s: &str) -> String {
    s.replace(['\u{00AD}', '\u{200B}'], "")
}

fn preprocess_text(text: &str, collapse: bool, preserve_newlines: bool, tab_size: usize) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_whitespace = false;
    let mut col = 0;

    for c in text.chars() {
        if c == '\n' {
            if preserve_newlines {
                result.push('\n');
                last_was_whitespace = false;
                col = 0;
            } else {
                if collapse {
                    if !last_was_whitespace {
                        result.push(' ');
                        last_was_whitespace = true;
                        col += 1;
                    }
                } else {
                    result.push(' ');
                    col += 1;
                }
            }
        } else if c == '\t' {
            if collapse {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                    col += 1;
                }
            } else {
                let spaces_to_add = tab_size - (col % tab_size);
                for _ in 0..spaces_to_add {
                    result.push(' ');
                }
                col += spaces_to_add;
                last_was_whitespace = true;
            }
        } else if is_html_whitespace(c) {
            if collapse {
                if !last_was_whitespace {
                    result.push(' ');
                    last_was_whitespace = true;
                    col += 1;
                }
            } else {
                result.push(c);
                col += 1;
                last_was_whitespace = true;
            }
        } else {
            result.push(c);
            last_was_whitespace = false;
            col += 1;
        }
    }
    result
}

fn apply_text_transform(s: &str, kind: &str) -> String {
    // spec: CSS Text Module Level 3, §2.1 (text-transform property)
    // The 'capitalize' value puts the first typographical character of each word in uppercase.
    // For this purpose, a word is a sequence of alphanumeric characters (letters/numbers).
    // Any punctuation, whitespace, or separator character preceding an alphanumeric character
    // marks the start of a new word.
    match kind {
        "uppercase" => s.to_uppercase(),
        "lowercase" => s.to_lowercase(),
        "capitalize" => {
            let mut result = String::with_capacity(s.len());
            let mut in_word = false;
            for c in s.chars() {
                if c.is_alphanumeric() {
                    if !in_word {
                        result.extend(c.to_uppercase());
                        in_word = true;
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                    in_word = false;
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
    fn test_text_transform_capitalize_advanced() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("(hello-world) done't. \"quoted\"".into()));
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

        assert_eq!(leaf_texts, vec!["(Hello-World) ", "Done'T. ", "\"Quoted\""]);
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

        let stylesheet_empty = parse_stylesheet("");
        let styles_empty = compute_styles(&dom, &stylesheet_empty);
        let children = dom.children(div);

        // 1. With empty stylesheet, and word_spacing parameter = 0.0
        let (line_boxes_0, _) = layout_inline_run(
            &dom,
            &styles_empty,
            children,
            800.0,
            10.0,
            20.0,
            0,
            "left",
            0.0,
            0.0,
        );
        assert!(!line_boxes_0.is_empty());
        let line_0 = &line_boxes_0[0];
        assert_eq!(line_0.children.len(), 2);
        let first_word_0 = &line_0.children[0];
        let second_word_0 = &line_0.children[1];

        // 2. With word-spacing in stylesheet, which is inherited by the text node!
        let stylesheet_10 = parse_stylesheet("div { word-spacing: 10px; }");
        let styles_10 = compute_styles(&dom, &stylesheet_10);

        let (line_boxes_10, _) = layout_inline_run(
            &dom, &styles_10, children, 800.0, 10.0, 20.0, 0, "left", 0.0, 0.0,
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

        let styles_single_empty = compute_styles(&dom_single, &stylesheet_empty);
        let styles_single_10 = compute_styles(&dom_single, &stylesheet_10);
        let children_single = dom_single.children(div_single);

        let (line_boxes_single_0, _) = layout_inline_run(
            &dom_single,
            &styles_single_empty,
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
            &styles_single_10,
            children_single,
            800.0,
            10.0,
            20.0,
            0,
            "left",
            0.0,
            0.0,
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
    fn test_nested_word_spacing_override() {
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

        let t2 = dom.create_node(NodeData::Text("nested words ".into()));
        dom.append_child(span, t2);

        let t3 = dom.create_node(NodeData::Text("world".into()));
        dom.append_child(div, t3);

        let stylesheet =
            parse_stylesheet("div { word-spacing: 5px; } span { word-spacing: 25px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Flatten inline children
        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        // Children should be:
        // 1. "hello " (has space, word-spacing from div = 5)
        // 2. "nested " (has space, word-spacing from span = 25)
        // 3. "words " (has space, word-spacing from span = 25)
        // 4. "world" (no trailing space, no extra word-spacing)
        assert_eq!(line.children.len(), 4);

        let word1 = &line.children[0];
        let word2 = &line.children[1];
        let word3 = &line.children[2];
        let word4 = &line.children[3];

        let font = crate::font::BitmapFont::builtin();
        let w_hello = font.measure("hello ") as f32;
        let w_nested = font.measure("nested ") as f32;
        let w_words = font.measure("words ") as f32;

        // Gap between word1 and word2 should be div's word-spacing = 5.0
        let gap1 = word2.rect.origin.x - (word1.rect.origin.x + w_hello);
        assert_eq!(gap1, 5.0);

        // Gap between word2 and word3 should be span's word-spacing = 25.0
        let gap2 = word3.rect.origin.x - (word2.rect.origin.x + w_nested);
        assert_eq!(gap2, 25.0);

        // Gap between word3 and word4 should be span's word-spacing = 25.0
        let gap3 = word4.rect.origin.x - (word3.rect.origin.x + w_words);
        assert_eq!(gap3, 25.0);
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

    #[test]
    fn test_line_height_number_multiplier_nested_inheritance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, span);

        let t1 = dom.create_node(NodeData::Text("child text".into()));
        dom.append_child(span, t1);

        let stylesheet = parse_stylesheet(
            "div { font-size: 10px; line-height: 2; } \
             span { display: inline; font-size: 20px; }",
        );
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
    fn test_white_space_pre_tab_expansion() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("a\tb\t\tc".into()));
        dom.append_child(div, t);

        // Default tab-size is 8
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

        // "a\tb\t\tc" with tab-size: 8:
        // - 'a' (col 0): col becomes 1
        // - '\t' (col 1): next multiple of 8 is 8, so we add 7 spaces. col becomes 8.
        // - 'b' (col 8): col becomes 9
        // - '\t' (col 9): next multiple of 8 is 16, so we add 7 spaces. col becomes 16.
        // - '\t' (col 16): next multiple of 8 is 24, so we add 8 spaces. col becomes 24.
        // - 'c' (col 24): col becomes 25
        // Expected string is: "a" + " "*7 + "b" + " "*7 + " "*8 + "c"
        let mut expected = vec!["a "];
        expected.extend([" "; 6]);
        expected.push("b ");
        expected.extend([" "; 14]);
        expected.push("c");

        assert_eq!(leaf_texts, expected);
    }

    #[test]
    fn test_white_space_pre_tab_size_custom() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("a\tb".into()));
        dom.append_child(div, t);

        // Custom tab-size is 4
        let stylesheet = parse_stylesheet("div { white-space: pre; tab-size: 4; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        // "a\tb" with tab-size: 4:
        // - 'a' (col 0): col becomes 1
        // - '\t' (col 1): next multiple of 4 is 4, so we add 3 spaces. col becomes 4.
        // - 'b' (col 4): col becomes 5
        // Preprocessed: "a   b"
        assert_eq!(leaf_texts, vec!["a ", " ", " ", "b"]);
    }

    #[test]
    fn test_letter_spacing_layout_completeness() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(div, t);

        // 1. Without letter-spacing
        let stylesheet_0 = parse_stylesheet("div { letter-spacing: normal; }");
        let styles_0 = compute_styles(&dom, &stylesheet_0);
        let children_0 = dom.children(div);
        let (line_boxes_0, _) = layout_inline_run(
            &dom, &styles_0, children_0, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );
        assert_eq!(line_boxes_0.len(), 1);
        let width_0 = line_boxes_0[0].children[0].rect.size.width;

        // 2. With letter-spacing: 5px
        let stylesheet_5 = parse_stylesheet("div { letter-spacing: 5px; }");
        let styles_5 = compute_styles(&dom, &stylesheet_5);
        let children_5 = dom.children(div);
        let (line_boxes_5, _) = layout_inline_run(
            &dom, &styles_5, children_5, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );
        assert_eq!(line_boxes_5.len(), 1);
        let width_5 = line_boxes_5[0].children[0].rect.size.width;

        // "hello" has 5 characters, so 4 inter-character spacing intervals of 5px = 20px extra.
        assert_eq!(width_5, width_0 + 20.0);

        // 3. Test that letter-spacing affects line wrapping
        let stylesheet_wrap =
            parse_stylesheet("div { letter-spacing: 10px; word-break: break-all; }");
        let styles_wrap = compute_styles(&dom, &stylesheet_wrap);
        let children_wrap = dom.children(div);

        // Let's set containing width so that the whole "hello" + letter-spacing (width_0 + 40px) doesn't fit on one line.
        // Let's use width_0 + 10px as container width. "hello" should wrap!
        let (line_boxes_wrap, _) = layout_inline_run(
            &dom,
            &styles_wrap,
            children_wrap,
            width_0 + 10.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );
        // It must have wrapped into multiple lines because width_0 + 40px > width_0 + 10px.
        assert!(line_boxes_wrap.len() > 1);
    }

    #[test]
    fn test_text_align_last_center() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("one two".into()));
        dom.append_child(div, t);

        // A stylesheet with text-align: left and text-align-last: center
        let stylesheet = parse_stylesheet("div { text-align: left; text-align-last: center; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Using a wide container so it doesn't wrap. It's only 1 line, so it's the last line.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 500.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line_box = &line_boxes[0];
        // The total width of children is some value. The slack (500 - width) / 2 should shift children's origin.x.
        assert!(line_box.children[0].rect.origin.x > 0.0);
    }

    #[test]
    fn test_text_align_last_justify() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("one two three".into()));
        dom.append_child(div, t);

        // Normally, the last line is not justified under text-align: justify.
        // But with text-align-last: justify, even the last line (here the only line) gets justified.
        let stylesheet = parse_stylesheet("div { text-align: left; text-align-last: justify; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 500.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line_box = &line_boxes[0];
        // The first child should be at 0.0, and the last child should be shifted right to align with the edge (500.0).
        let last_child = &line_box.children[line_box.children.len() - 1];
        let expected_x = 500.0 - last_child.rect.size.width;
        assert_eq!(last_child.rect.origin.x, expected_x);
    }

    #[test]
    fn test_overflow_wrap_anywhere() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("superlongunbreakableword".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { overflow-wrap: anywhere; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // We set a very small containing width so that the word must break!
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 50.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        // It should have wrapped/broken into multiple line boxes.
        assert!(line_boxes.len() > 1);
    }

    #[test]
    fn test_white_space_break_spaces() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello  world".into()));
        dom.append_child(div, t);

        // Parse with pre-wrap first to get similar layout behavior
        let stylesheet = parse_stylesheet("div { white-space: pre-wrap; }");
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually override white_space property to "break-spaces" since CSS parser doesn't support parsing it yet
        if let Some(style) = styles.get_mut(&div) {
            std::sync::Arc::make_mut(&mut style.inherited_text).white_space =
                "break-spaces".to_string();
        }

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        let mut leaf_texts = Vec::new();
        for line in &line_boxes {
            leaf_texts.extend(collect_leaf_texts(line));
        }

        // Under break-spaces, whitespace is preserved (collapse is false, preserve_newlines is true, allow_wrap is true).
        // Since collapse is false, we expect consecutive spaces to be preserved!
        assert!(leaf_texts.contains(&"  ".to_string()) || leaf_texts.contains(&" ".to_string()));
    }

    #[test]
    fn test_rtl_layout_horizontal_reversal() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // We have multiple child elements to check their ordering
        let t1 = dom.create_node(NodeData::Text("one ".into()));
        dom.append_child(div, t1);
        let t2 = dom.create_node(NodeData::Text("two".into()));
        dom.append_child(div, t2);

        // Under RTL, the horizontal positions are reversed.
        // Let's set direction: rtl.
        let stylesheet = parse_stylesheet("div { direction: rtl; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 500.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line_box = &line_boxes[0];

        // Let's collect texts of child fragments in left-to-right visual layout order.
        // Since we reversed the vector in create_line_box_adjusted, the left-to-right order (indexes 0 and 1) should be "two" and then "one ".
        let leaf_texts = collect_leaf_texts(line_box);
        assert_eq!(leaf_texts, vec!["two", "one "]);

        // "two" should be on the left (x = 0), and "one " should be to its right.
        let child_0 = &line_box.children[0];
        let child_1 = &line_box.children[1];
        assert_eq!(child_0.rect.origin.x, 0.0);
        assert!(child_1.rect.origin.x > child_0.rect.origin.x);
    }

    #[test]
    fn test_rtl_alignment_default() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(div, t);

        // Default text-align (start) on direction: rtl should align text to the right.
        let stylesheet = parse_stylesheet("div { direction: rtl; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Container width is 200px.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 200.0, 0.0, 0.0, 0, "start", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line_box = &line_boxes[0];
        let child = &line_box.children[0];

        // The right edge of the text should align with the right edge of the container (200px).
        let right_edge = child.rect.origin.x + child.rect.size.width;
        assert_eq!(right_edge, 200.0);
    }

    #[test]
    fn test_rtl_text_align_start_end() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(div, t);

        // 1. text-align: end on direction: rtl should align text to the left (origin.x = 0).
        let stylesheet_end = parse_stylesheet("div { direction: rtl; text-align: end; }");
        let styles_end = compute_styles(&dom, &stylesheet_end);
        let children_end = dom.children(div);
        let (line_boxes_end, _) = layout_inline_run(
            &dom,
            &styles_end,
            children_end,
            200.0,
            0.0,
            0.0,
            0,
            "end",
            0.0,
            0.0,
        );
        assert_eq!(line_boxes_end[0].children[0].rect.origin.x, 0.0);

        // 2. text-align: left on direction: rtl should align text to the left (origin.x = 0).
        let stylesheet_left = parse_stylesheet("div { direction: rtl; text-align: left; }");
        let styles_left = compute_styles(&dom, &stylesheet_left);
        let children_left = dom.children(div);
        let (line_boxes_left, _) = layout_inline_run(
            &dom,
            &styles_left,
            children_left,
            200.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );
        assert_eq!(line_boxes_left[0].children[0].rect.origin.x, 0.0);
    }

    #[test]
    fn test_inline_block_baseline_empty() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ib-class".into())],
        });
        dom.append_child(div, ib);

        let stylesheet = parse_stylesheet(
            "
            .ib-class { display: inline-block; width: 50px; height: 30px; margin-bottom: 5px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        let line_box_bottom_y = line.rect.size.height;
        let expected_y = line_box_bottom_y - 5.0 - 30.0;
        assert_eq!(child_box.rect.origin.y, expected_y);
    }

    #[test]
    fn test_inline_block_baseline_with_text() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ib-class".into())],
        });
        dom.append_child(div, ib);

        let t_inner = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(ib, t_inner);

        let stylesheet = parse_stylesheet(
            "
            .ib-class { display: inline-block; width: 50px; height: 40px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        assert_eq!(child_box.children.len(), 1);
        let inner_line = &child_box.children[0];

        let child_box_baseline_y = inner_line.rect.origin.y + inner_line.rect.size.height;
        let parent_line_baseline_y = line.rect.origin.y + line.rect.size.height;
        assert_eq!(child_box_baseline_y, parent_line_baseline_y);
    }

    #[test]
    fn test_trailing_whitespace_trimming_right_align() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // A word with trailing collapsible space
        let t = dom.create_node(NodeData::Text("hello ".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-align: right; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Container of width 100px.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 100.0, 0.0, 0.0, 0, "right", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        // The text box content is still preserved as "hello "
        assert_eq!(child_box.text, Some("hello ".to_string()));

        // Under standard right-alignment without trailing space trimming, "hello " (48px) would
        // start at 100.0 - 48.0 = 52.0.
        // But with trailing space trimming, "hello" (40px) is aligned to the right edge.
        // So the right edge of "hello" (trimmed) is 100.0.
        // Thus, the starting x coordinate of the box is 100.0 - 40.0 = 60.0!
        assert_eq!(child_box.rect.origin.x, 60.0);
    }

    #[test]
    fn test_leading_whitespace_collapsing_with_text_indent() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // A text starting with leading collapsible space
        let t = dom.create_node(NodeData::Text(" hello".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-indent: 40px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Container of width 200px.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 200.0, 0.0, 0.0, 0, "left", 40.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        // The leading space is collapsed/skipped, so we have exactly "hello" (40px width).
        assert_eq!(child_box.text, Some("hello".to_string()));

        // Because of text-indent: 40px, the word starts exactly at x = 40.0
        assert_eq!(child_box.rect.origin.x, 40.0);
    }

    #[test]
    fn test_text_overflow_ellipsis() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text(
            "extremely_long_text_that_will_overflow".into(),
        ));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-overflow: ellipsis; overflow: hidden; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 80.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        assert_eq!(child_box.text, Some("extremely…".to_string()));
    }

    #[test]
    fn test_soft_hyphens_manual_and_none() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("ap\u{00AD}ple".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { hyphens: manual; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 24.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        assert_eq!(line_boxes.len(), 2);
        let line1 = &line_boxes[0];
        let line2 = &line_boxes[1];

        assert_eq!(line1.children.len(), 1);
        assert_eq!(line1.children[0].text, Some("ap-".to_string()));

        assert_eq!(line2.children.len(), 1);
        assert_eq!(line2.children[0].text, Some("ple".to_string()));

        let stylesheet_none = parse_stylesheet("div { hyphens: none; }");
        let styles_none = compute_styles(&dom, &stylesheet_none);

        let children_none = dom.children(div);
        let (line_boxes_none, _) = layout_inline_run(
            &dom,
            &styles_none,
            children_none,
            24.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        assert_eq!(line_boxes_none.len(), 1);
        assert_eq!(line_boxes_none[0].children.len(), 1);
        assert_eq!(
            line_boxes_none[0].children[0].text,
            Some("apple".to_string())
        );
    }

    #[test]
    fn test_pre_wrap_hanging_spaces() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello      ".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: pre-wrap; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // width 50.0 is enough for "hello" but not with all trailing spaces.
        // Under pre-wrap, trailing spaces should hang (not wrap), so they should all be on 1 line.
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 50.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        // The spaces are kept on the line (not wrapped)
        assert!(line.children.len() > 1);
    }

    #[test]
    fn test_break_spaces_wrapping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello      ".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { white-space: pre-wrap; }");
        let mut styles = compute_styles(&dom, &stylesheet);

        for style in styles.values_mut() {
            std::sync::Arc::make_mut(&mut style.inherited_text).white_space =
                "break-spaces".to_string();
        }

        let children = dom.children(div);
        // Under break-spaces, trailing spaces must wrap/break, so they will overflow 50px and form multiple lines.
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 50.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        assert!(line_boxes.len() > 1);
    }

    #[test]
    fn test_word_break_keep_all_cjk() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("日本語".into()));
        dom.append_child(div, t);

        // Under normal word-break, CJK can break anywhere, so it will break into multiple lines on a 16px wide container.
        let stylesheet_normal = parse_stylesheet("div { word-break: normal; }");
        let styles_normal = compute_styles(&dom, &stylesheet_normal);
        let children_normal = dom.children(div);
        let (line_boxes_normal, _) = layout_inline_run(
            &dom,
            &styles_normal,
            children_normal,
            16.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );
        assert!(line_boxes_normal.len() > 1);

        // Under keep-all word-break, CJK does not break, so it overflows onto exactly 1 line.
        let stylesheet_keep = parse_stylesheet("div { word-break: keep-all; }");
        let styles_keep = compute_styles(&dom, &stylesheet_keep);
        let children_keep = dom.children(div);
        let (line_boxes_keep, _) = layout_inline_run(
            &dom,
            &styles_keep,
            children_keep,
            16.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );
        assert_eq!(line_boxes_keep.len(), 1);
    }

    #[test]
    fn test_word_break_break_all_and_line_break_anywhere() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("abc ".into()));
        dom.append_child(div, t1);
        let t2 = dom.create_node(NodeData::Text("def".into()));
        dom.append_child(div, t2);

        // Under break-all, we should break character-by-character on the current line.
        // Width 40px: "abc " is 32px (8px/char). 8px remains.
        // "def" is processed, "d" is placed on first line, "ef" is wrapped to second.
        let stylesheet_all = parse_stylesheet("div { word-break: break-all; }");
        let styles_all = compute_styles(&dom, &stylesheet_all);
        let children_all = dom.children(div);
        let (line_boxes_all, _) = layout_inline_run(
            &dom,
            &styles_all,
            children_all,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        assert!(line_boxes_all.len() >= 2);
        // The first line should have two children: "abc " and "d"
        assert_eq!(line_boxes_all[0].children.len(), 2);
        assert_eq!(line_boxes_all[0].children[0].text, Some("abc ".to_string()));
        assert_eq!(line_boxes_all[0].children[1].text, Some("d".to_string()));

        // Under line-break: anywhere, it should behave exactly the same
        let stylesheet_anywhere = parse_stylesheet("div { line-break: anywhere; }");
        let styles_anywhere = compute_styles(&dom, &stylesheet_anywhere);
        let children_anywhere = dom.children(div);
        let (line_boxes_anywhere, _) = layout_inline_run(
            &dom,
            &styles_anywhere,
            children_anywhere,
            40.0,
            0.0,
            0.0,
            0,
            "left",
            0.0,
            0.0,
        );

        assert!(line_boxes_anywhere.len() >= 2);
        assert_eq!(line_boxes_anywhere[0].children.len(), 2);
        assert_eq!(
            line_boxes_anywhere[0].children[0].text,
            Some("abc ".to_string())
        );
        assert_eq!(
            line_boxes_anywhere[0].children[1].text,
            Some("d".to_string())
        );
    }

    #[test]
    fn test_rtl_text_overflow_ellipsis() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("extremely_long_text_overflow_rtl".into()));
        dom.append_child(div, t);

        let stylesheet =
            parse_stylesheet("div { direction: rtl; text-overflow: ellipsis; overflow: hidden; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Width 80.0 forces overflow. Ellipsis should be placed on the left side because it's RTL overflow.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 80.0, 0.0, 0.0, 0, "right", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);
        let child_box = &line.children[0];

        // RTL ellipsis should start with '…' on the left side!
        assert!(child_box.text.as_ref().unwrap().starts_with('…'));
    }

    #[test]
    fn test_zero_width_space_opportunity() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("abc\u{200b}def".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { word-break: normal; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Width 24.0 fits "abc" but not "abcdef".
        // It should break exactly at the zero-width space!
        let (line_boxes, _) =
            layout_inline_run(&dom, &styles, children, 24.0, 0.0, 0.0, 0, "left", 0.0, 0.0);

        assert_eq!(line_boxes.len(), 2);
        assert_eq!(line_boxes[0].children.len(), 1);
        assert_eq!(line_boxes[0].children[0].text, Some("abc".to_string()));
        assert_eq!(line_boxes[1].children.len(), 1);
        assert_eq!(line_boxes[1].children[0].text, Some("def".to_string()));
    }

    #[test]
    fn test_trimming_entirely_space_children() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(div, t1);

        let t2 = dom.create_node(NodeData::Text("   ".into()));
        dom.append_child(div, t2);

        let stylesheet = parse_stylesheet("div { text-align: right; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        // Container of width 100px.
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 100.0, 0.0, 0.0, 0, "right", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 2);

        let box_text = &line.children[0];
        let box_space = &line.children[1];

        // "hello" has width 40.0.
        // Entirely-space "   " box has width 0.0 after trimming.
        assert_eq!(box_text.rect.size.width, 40.0);
        assert_eq!(box_space.rect.size.width, 0.0);

        // Aligning to the right of 100px.
        // "hello" should start at 100 - 40 = 60.
        assert_eq!(box_text.rect.origin.x, 60.0);
    }

    #[test]
    fn test_t1051_unified_baseline_alignment() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t1 = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(div, t1);

        let ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ib-class".into())],
        });
        dom.append_child(div, ib);

        let t_inner = dom.create_node(NodeData::Text("world".into()));
        dom.append_child(ib, t_inner);

        let stylesheet = parse_stylesheet(
            "
            .ib-class { display: inline-block; width: 50px; height: 30px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 800.0, 0.0, 0.0, 0, "left", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 2);

        let box_text = &line.children[0];
        let box_ib = &line.children[1];

        // Both are aligned via their baselines to line_box_bottom_y.
        // For text: baseline is bottom of text box, so baseline is box_text.rect.origin.y + 8.0.
        // For inline-block: baseline is the baseline of its last line, which is at origin.y + 8.0.
        // Therefore, their baseline Y coordinates must be perfectly equal.
        let text_baseline = box_text.rect.origin.y + box_text.rect.size.height;
        let ib_baseline = find_last_fragment_baseline(box_ib, &styles, &dom).unwrap();
        assert_eq!(text_baseline, ib_baseline);
    }

    #[test]
    fn test_right_align_multiple_spaces_trimming() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let t = dom.create_node(NodeData::Text("hello   ".into()));
        dom.append_child(div, t);

        let stylesheet = parse_stylesheet("div { text-align: right; }");
        let styles = compute_styles(&dom, &stylesheet);

        let children = dom.children(div);
        let (line_boxes, _) = layout_inline_run(
            &dom, &styles, children, 100.0, 0.0, 0.0, 0, "right", 0.0, 0.0,
        );

        assert_eq!(line_boxes.len(), 1);
        let line = &line_boxes[0];
        assert_eq!(line.children.len(), 1);

        let child_box = &line.children[0];

        // The text is preserved as "hello ".
        assert_eq!(child_box.text, Some("hello ".to_string()));

        // But its width must be trimmed to 40.0.
        assert_eq!(child_box.rect.size.width, 40.0);

        // So it starts at 100 - 40 = 60.
        assert_eq!(child_box.rect.origin.x, 60.0);
    }
}
