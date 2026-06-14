use super::LayoutBox;
use crate::infra::NodeId;
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

/// Helper to get the computed float value of a style.
/// Returns Some("left"), Some("right"), or None.
///
/// Supports line-box shortening and float stacking of multiple floats side-by-side per CSS2.1 §9.5.
pub(crate) fn get_float_value(style: &CategorizedComputedStyle) -> Option<&str> {
    let fl = style.reset_box.float.as_str();
    if fl == "left" || fl == "right" {
        Some(fl)
    } else {
        None
    }
}

thread_local! {
    static CURRENT_STYLE_PTR: RefCell<usize> = const { RefCell::new(0) };
}

/// Helper to get the computed clear value of a style.
/// Returns Some("left"), Some("right"), Some("both"), or None.
pub(crate) fn get_clear_value(style: &CategorizedComputedStyle) -> Option<&str> {
    CURRENT_STYLE_PTR.with(|ptr| {
        *ptr.borrow_mut() = style as *const _ as usize;
    });
    let cl = style.reset_box.clear.as_str();
    if cl == "left" || cl == "right" || cl == "both" {
        Some(cl)
    } else {
        None
    }
}

fn get_node_index(node_id: NodeId) -> u32 {
    let s = format!("{:?}", node_id);
    if let Some(idx_start) = s.find("index: ") {
        let sub = &s[idx_start + 7..];
        if let Some(idx_end) = sub.find(',')
            && let Ok(idx) = sub[..idx_end].trim().parse::<u32>()
        {
            return idx;
        }
    }
    0
}

fn is_isolated_by_bfc(
    rf_node: NodeId,
    clearing_node: NodeId,
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> bool {
    let rf_idx = get_node_index(rf_node);
    let cl_idx = get_node_index(clearing_node);

    if rf_node == clearing_node {
        return false;
    }

    // Find the minimum index of any sibling in children to define the upper bound of ancestors.
    let mut min_sibling_idx = cl_idx;
    for sibling in children {
        if let Some(node_id) = sibling.node {
            let idx = get_node_index(node_id);
            if idx < min_sibling_idx {
                min_sibling_idx = idx;
            }
        }
    }

    // Search for the BFC container with the largest index that is less than min_sibling_idx.
    let mut nearest_bfc_idx = 0;
    for (&node_id, style) in styles {
        if establishes_bfc(style) {
            let idx = get_node_index(node_id);
            if idx < min_sibling_idx {
                // If it is a float, check if it's already in SESSION.
                // If it is already in SESSION, then it's a completed float, not an ancestor of clearing_node!
                let is_completed_float = if get_float_value(style).is_some() {
                    let mut in_session = false;
                    SESSION.with(|session| {
                        for rf in &session.borrow().floats {
                            if rf.node_id == node_id {
                                in_session = true;
                                break;
                            }
                        }
                    });
                    in_session
                } else {
                    false
                };

                if !is_completed_float && idx > nearest_bfc_idx {
                    nearest_bfc_idx = idx;
                }
            }
        }
    }

    // If we found a BFC ancestor, and the registered float was laid out before that BFC ancestor,
    // then the registered float is isolated.
    if nearest_bfc_idx > 0 && rf_idx < nearest_bfc_idx {
        return true;
    }

    false
}

fn establishes_bfc(style: &CategorizedComputedStyle) -> bool {
    get_float_value(style).is_some()
        || style.reset_box.display == "inline-block"
        || style.reset_box.display == "flex"
        || style.reset_box.display == "table"
        || style.reset_box.display == "table-cell"
        || style.reset_box.position == "absolute"
        || style.reset_box.position == "fixed"
}

use std::cell::RefCell;

struct RegisteredFloat {
    node_id: NodeId,
    float_type: String, // "left" or "right"
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct FloatSession {
    styles_ptr: usize,
    styles_len: usize,
    floats: Vec<RegisteredFloat>,
}

thread_local! {
    static SESSION: RefCell<FloatSession> = const { RefCell::new(FloatSession {
        styles_ptr: 0,
        styles_len: 0,
        floats: Vec::new(),
    }) };
}

fn sync_session(styles: &HashMap<NodeId, CategorizedComputedStyle>) {
    let current_ptr = styles as *const _ as usize;
    let current_len = styles.len();
    SESSION.with(|session| {
        let mut s = session.borrow_mut();
        if s.styles_ptr != current_ptr || s.styles_len != current_len {
            s.styles_ptr = current_ptr;
            s.styles_len = current_len;
            s.floats.clear();
        }
    });
}

fn is_descendant_of_any_bfc(
    box_: &LayoutBox,
    target_node: NodeId,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> bool {
    let mut stack = vec![(box_, false)];
    while let Some((current, mut in_bfc)) = stack.pop() {
        if let Some(node_id) = current.node {
            if let Some(style) = styles.get(&node_id)
                && establishes_bfc(style)
            {
                in_bfc = true;
            }
            if node_id == target_node {
                return in_bfc;
            }
        }
        for child in &current.children {
            stack.push((child, in_bfc));
        }
    }
    false
}

/// Computes the maximum bottom edge of the relevant active floats based on `clear_val`.
pub(crate) fn find_clearance_y(
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    clear_val: &str,
) -> Option<f32> {
    let clearing_node = CURRENT_STYLE_PTR.with(|ptr| {
        let p = *ptr.borrow();
        for (&node_id, s) in styles {
            if (s as *const _ as usize) == p {
                return Some(node_id);
            }
        }
        None
    });

    let mut max_float_y = None;
    let mut collected_nodes = std::collections::HashSet::new();

    // 1. Find floats from siblings recursively
    let mut stack = Vec::new();
    for child in children {
        stack.push(child);
    }

    while let Some(current) = stack.pop() {
        let mut is_bfc = false;
        if let Some(node_id) = current.node
            && let Some(style) = styles.get(&node_id)
        {
            is_bfc = establishes_bfc(style);
            if let Some(fv) = get_float_value(style) {
                let matches_side = match clear_val {
                    "left" => fv == "left",
                    "right" => fv == "right",
                    "both" => fv == "left" || fv == "right",
                    _ => false,
                };
                if matches_side {
                    let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);
                    let bottom_edge = current.rect.max_y() + margin_bottom;
                    max_float_y = Some(match max_float_y {
                        Some(y) => f32::max(y, bottom_edge),
                        None => bottom_edge,
                    });
                    collected_nodes.insert(node_id);
                }
            }
        }

        if !is_bfc {
            for child in &current.children {
                stack.push(child);
            }
        }
    }

    // 2. Add floats from session registry
    sync_session(styles);
    SESSION.with(|session| {
        let s = session.borrow();
        for rf in &s.floats {
            if collected_nodes.contains(&rf.node_id) {
                continue;
            }

            let matches_side = match clear_val {
                "left" => rf.float_type == "left",
                "right" => rf.float_type == "right",
                "both" => rf.float_type == "left" || rf.float_type == "right",
                _ => false,
            };

            if matches_side {
                // Check BFC isolation
                let mut inside_nested_bfc = false;
                for sibling in children {
                    if is_descendant_of_any_bfc(sibling, rf.node_id, styles) {
                        inside_nested_bfc = true;
                        break;
                    }
                }

                if !inside_nested_bfc {
                    let is_isolated = if let Some(cl_node) = clearing_node {
                        is_isolated_by_bfc(rf.node_id, cl_node, children, styles)
                    } else {
                        false
                    };

                    if !is_isolated {
                        let bottom_edge = rf.y + rf.height;
                        max_float_y = Some(match max_float_y {
                            Some(y) => f32::max(y, bottom_edge),
                            None => bottom_edge,
                        });
                    }
                }
            }
        }
    });

    max_float_y
}

struct PrecedingFloat {
    float_type: String, // "left" or "right"
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn collect_preceding_floats(
    child_node_id: NodeId,
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> Vec<PrecedingFloat> {
    // 1. Collect floats from sibling layout boxes recursively
    let mut floats = Vec::new();
    let mut collected_nodes = std::collections::HashSet::new();

    let mut stack = Vec::new();
    for child in children.iter().rev() {
        stack.push(child);
    }

    while let Some(current) = stack.pop() {
        let mut is_bfc = false;
        if let Some(node_id) = current.node
            && let Some(style) = styles.get(&node_id)
        {
            is_bfc = establishes_bfc(style);
            if let Some(fv) = get_float_value(style) {
                let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
                let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
                let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
                let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);

                let x = current.rect.origin.x - margin_left;
                let y = current.rect.origin.y - margin_top;
                let width = f32::max(0.0, current.rect.size.width + margin_left + margin_right);
                let height = f32::max(0.0, current.rect.size.height + margin_top + margin_bottom);

                floats.push(PrecedingFloat {
                    float_type: fv.to_string(),
                    x,
                    y,
                    width,
                    height,
                });
                collected_nodes.insert(node_id);
            }
        }

        if !is_bfc {
            for child in current.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    // 2. Add floats from the session registry
    sync_session(styles);
    SESSION.with(|session| {
        let s = session.borrow();
        for rf in &s.floats {
            if collected_nodes.contains(&rf.node_id) {
                continue;
            }

            // Check BFC isolation
            let mut inside_nested_bfc = false;
            for sibling in children {
                if is_descendant_of_any_bfc(sibling, rf.node_id, styles) {
                    inside_nested_bfc = true;
                    break;
                }
            }

            if !inside_nested_bfc {
                let is_isolated = is_isolated_by_bfc(rf.node_id, child_node_id, children, styles);
                if !is_isolated {
                    floats.push(PrecedingFloat {
                        float_type: rf.float_type.clone(),
                        x: rf.x,
                        y: rf.y,
                        width: rf.width,
                        height: rf.height,
                    });
                }
            }
        }
    });

    floats
}

fn floats_overlap_vertically(y1: f32, h1: f32, y2: f32, h2: f32) -> bool {
    let h1 = if h1 < 0.0 { 0.0 } else { h1 };
    let h2 = if h2 < 0.0 { 0.0 } else { h2 };
    if h1 > 0.0 && h2 > 0.0 {
        y1 < y2 + h2 && y2 < y1 + h1
    } else if h1 == 0.0 && h2 > 0.0 {
        y2 <= y1 && y1 < y2 + h2
    } else if h1 > 0.0 && h2 == 0.0 {
        y1 <= y2 && y2 < y1 + h1
    } else {
        // h1 == 0.0 && h2 == 0.0
        y1 == y2
    }
}

fn get_bounds_at_y(
    floats: &[PrecedingFloat],
    candidate_y: f32,
    float_outer_height: f32,
    containing_left: f32,
    containing_width: f32,
) -> (f32, f32) {
    let mut left_bound = containing_left;
    let mut right_bound = containing_left + containing_width;

    for f in floats {
        let overlap = floats_overlap_vertically(f.y, f.height, candidate_y, float_outer_height);
        if overlap {
            if f.float_type == "left" {
                let right_edge = f.x + f.width;
                if right_edge > left_bound {
                    left_bound = right_edge;
                }
            } else if f.float_type == "right" {
                let left_edge = f.x;
                if left_edge < right_bound {
                    right_bound = left_edge;
                }
            }
        }
    }

    (left_bound, right_bound)
}

/// Positions and shifts a float LayoutBox correctly, accounting for:
/// - clearance (left/right/both)
/// - stacking side-by-side with preceding floats
/// - constrained containing block width (shifting down if it doesn't fit)
#[allow(clippy::too_many_arguments)]
pub(crate) fn layout_and_position_float(
    child_box: &mut LayoutBox,
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    float_val: &str,
    clear_val: Option<&str>,
    containing_left: f32,
    containing_width: f32,
    starting_y: f32,
) {
    let node_id = match child_box.node {
        Some(id) => id,
        None => return,
    };
    let style = match styles.get(&node_id) {
        Some(s) => s,
        None => return,
    };

    let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
    let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
    let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
    let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);

    let child_box_width = child_box.rect.size.width;
    let child_box_height = child_box.rect.size.height;

    let float_outer_width = f32::max(0.0, child_box_width + margin_left + margin_right);
    let float_outer_height = f32::max(0.0, child_box_height + margin_top + margin_bottom);

    // Sync session and prune duplicate/stale floats
    sync_session(styles);
    SESSION.with(|session| {
        let mut s = session.borrow_mut();
        let current_idx = get_node_index(node_id);
        if let Some(idx) = s
            .floats
            .iter()
            .position(|f| f.node_id == node_id || get_node_index(f.node_id) >= current_idx)
        {
            s.floats.truncate(idx);
        }
    });

    let floats = collect_preceding_floats(node_id, children, styles);

    // Initial candidate Y starts at starting_y.
    // Apply clearance first.
    let mut candidate_y = starting_y;
    if let Some(cv) = clear_val
        && let Some(cy) = find_clearance_y(children, styles, cv)
        && cy > candidate_y
    {
        candidate_y = cy;
    }

    // CSS 2.1 Section 9.5.1 Rule 5:
    // A floating box's outer top edge may not be higher than the outer top edge of any preceding floating box.
    for f in &floats {
        if f.y > candidate_y {
            candidate_y = f.y;
        }
    }

    let final_x;
    let final_y;

    loop {
        let (left_bound, right_bound) = get_bounds_at_y(
            &floats,
            candidate_y,
            float_outer_height,
            containing_left,
            containing_width,
        );

        let available_width = right_bound - left_bound;
        if float_outer_width <= available_width
            || (left_bound == containing_left && right_bound == containing_left + containing_width)
        {
            // Fits here!
            final_x = if float_val == "left" {
                left_bound + margin_left
            } else {
                right_bound - margin_right - child_box_width
            };
            final_y = candidate_y + margin_top;
            break;
        }

        // Find next candidate Y
        let mut next_y = None;
        for f in &floats {
            let overlap = floats_overlap_vertically(f.y, f.height, candidate_y, float_outer_height);
            if overlap {
                let bottom = f.y + f.height;
                if bottom > candidate_y {
                    next_y = Some(match next_y {
                        Some(ny) => f32::min(ny, bottom),
                        None => bottom,
                    });
                }
            }
        }

        match next_y {
            Some(ny) => {
                candidate_y = ny;
            }
            None => {
                // Fallback
                final_x = if float_val == "left" {
                    containing_left + margin_left
                } else {
                    containing_left + containing_width - margin_right - child_box_width
                };
                final_y = candidate_y + margin_top;
                break;
            }
        }
    }

    // Shift child_box to (final_x, final_y)
    let initial_x = child_box.rect.origin.x;
    let initial_y = child_box.rect.origin.y;
    let dx = final_x - initial_x;
    let dy = final_y - initial_y;

    if dx != 0.0 || dy != 0.0 {
        super::position::shift_layout_box(child_box, styles, dx, dy, 0);
    }

    // Record the newly positioned float in the session registry
    SESSION.with(|session| {
        let mut s = session.borrow_mut();
        s.floats.push(RegisteredFloat {
            node_id,
            float_type: float_val.to_string(),
            x: final_x - margin_left,
            y: final_y - margin_top,
            width: float_outer_width,
            height: float_outer_height,
        });
    });
}

/// Computes the adjusted horizontal bounds (left_bound, right_bound) for a line-box
/// at a given Y coordinate and line height, taking into account any active floats in the same BFC.
#[allow(dead_code)]
pub(crate) fn get_line_box_bounds_at_y(
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    y: f32,
    height: f32,
    containing_left: f32,
    containing_width: f32,
) -> (f32, f32) {
    let mut floats = Vec::new();
    let mut collected_nodes = std::collections::HashSet::new();

    // 1. Collect floats from siblings recursively
    let mut stack = Vec::new();
    for child in children.iter().rev() {
        stack.push(child);
    }

    while let Some(current) = stack.pop() {
        let mut is_bfc = false;
        if let Some(node_id) = current.node
            && let Some(style) = styles.get(&node_id)
        {
            is_bfc = establishes_bfc(style);
            if let Some(fv) = get_float_value(style) {
                let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
                let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
                let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
                let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);

                let fx = current.rect.origin.x - margin_left;
                let fy = current.rect.origin.y - margin_top;
                let w = f32::max(0.0, current.rect.size.width + margin_left + margin_right);
                let h = f32::max(0.0, current.rect.size.height + margin_top + margin_bottom);

                floats.push(PrecedingFloat {
                    float_type: fv.to_string(),
                    x: fx,
                    y: fy,
                    width: w,
                    height: h,
                });
                collected_nodes.insert(node_id);
            }
        }

        if !is_bfc {
            for child in current.children.iter().rev() {
                stack.push(child);
            }
        }
    }

    // 2. Add floats from the session registry
    sync_session(styles);
    SESSION.with(|session| {
        let s = session.borrow();
        for rf in &s.floats {
            if collected_nodes.contains(&rf.node_id) {
                continue;
            }

            // Check BFC isolation
            let mut inside_nested_bfc = false;
            for sibling in children {
                if is_descendant_of_any_bfc(sibling, rf.node_id, styles) {
                    inside_nested_bfc = true;
                    break;
                }
            }

            if !inside_nested_bfc {
                let is_isolated = {
                    let mut isolated = false;
                    for child in children {
                        if let Some(node_id) = child.node
                            && is_isolated_by_bfc(rf.node_id, node_id, children, styles)
                        {
                            isolated = true;
                            break;
                        }
                    }
                    isolated
                };

                if !is_isolated {
                    floats.push(PrecedingFloat {
                        float_type: rf.float_type.clone(),
                        x: rf.x,
                        y: rf.y,
                        width: rf.width,
                        height: rf.height,
                    });
                }
            }
        }
    });

    get_bounds_at_y(&floats, y, height, containing_left, containing_width)
}

/// Adjusts the width and position of a block-level layout box that establishes a BFC
/// (e.g., display: flex, table, or inline-block) to avoid overlapping with any active floats.
///
/// Under CSS 2.1 §9.5, the border box of such an element must not overlap the margin box
/// of any floats in the same block formatting context.
#[allow(dead_code)]
pub(crate) fn adjust_bfc_width_and_position(
    bfc_box: &mut LayoutBox,
    children: &[LayoutBox],
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    containing_left: f32,
    containing_width: f32,
) {
    let node_id = match bfc_box.node {
        Some(id) => id,
        None => return,
    };
    let style = match styles.get(&node_id) {
        Some(s) => s,
        None => return,
    };

    if !establishes_bfc(style) {
        return;
    }

    let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
    let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
    let margin_top = crate::layout::get_px(style, "margin-top", 0.0);

    let mut candidate_y = bfc_box.rect.origin.y - margin_top;

    let floats = collect_preceding_floats(node_id, children, styles);
    if floats.is_empty() {
        return;
    }

    let bfc_outer_width = f32::max(0.0, bfc_box.rect.size.width + margin_left + margin_right);
    let bfc_outer_height = f32::max(
        0.0,
        bfc_box.rect.size.height + margin_top + crate::layout::get_px(style, "margin-bottom", 0.0),
    );

    let (left_bound, right_bound) = get_bounds_at_y(
        &floats,
        candidate_y,
        bfc_outer_height,
        containing_left,
        containing_width,
    );

    let overlapping_left = left_bound > containing_left;
    let overlapping_right = right_bound < containing_left + containing_width;

    if overlapping_left || overlapping_right {
        let available_width = right_bound - left_bound;
        let mut new_width = bfc_box.rect.size.width;

        // If the remaining space is too narrow for the BFC element's margin box,
        // we can shift the BFC element down (clearing it) below the floats!
        if bfc_outer_width > available_width {
            // Find next candidate Y below the overlapping floats
            let mut next_y = candidate_y;
            loop {
                let mut found_overlap = false;
                let mut min_bottom = None;

                for f in &floats {
                    let overlap =
                        floats_overlap_vertically(f.y, f.height, next_y, bfc_outer_height);
                    if overlap {
                        found_overlap = true;
                        let bottom = f.y + f.height;
                        min_bottom = Some(match min_bottom {
                            Some(mb) => f32::min(mb, bottom),
                            None => bottom,
                        });
                    }
                }

                if found_overlap && let Some(mb) = min_bottom {
                    next_y = mb;
                } else {
                    break;
                }
            }

            if next_y > candidate_y {
                let dy = next_y - candidate_y;
                super::position::shift_layout_box(bfc_box, styles, 0.0, dy, 0);
                candidate_y = next_y;
            }
        }

        // Now compute bounds again at the final candidate_y
        let (final_left, final_right) = get_bounds_at_y(
            &floats,
            candidate_y,
            bfc_outer_height,
            containing_left,
            containing_width,
        );

        let final_avail = final_right - final_left;
        let mut final_x = bfc_box.rect.origin.x;

        // Place it adjacent to the floats and reduce width if necessary
        if final_left > containing_left {
            final_x = final_left + margin_left;
        }

        let outer_avail_width = final_avail - margin_left - margin_right;
        if outer_avail_width < new_width {
            new_width = f32::max(0.0, outer_avail_width);
        }

        let dx = final_x - bfc_box.rect.origin.x;
        if dx != 0.0 {
            super::position::shift_layout_box(bfc_box, styles, dx, 0.0, 0);
        }
        bfc_box.rect.size.width = new_width;
    }
}

#[cfg(test)]
mod tests {
    use super::{LayoutBox, RegisteredFloat, SESSION};
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::layout::layout_document;
    use crate::style::compute_styles;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_float_clearing_across_margins() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
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
            .float {
                float: left;
                width: 100px;
                height: 50px;
                margin-bottom: 20px;
            }
            p {
                display: block;
                clear: left;
                margin-top: 30px;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        let float_layout = &body_box.children[0];
        let p_layout = &body_box.children[1];

        // The float bottom edge is at y = 50 + 20 = 70.
        // The normal_box (p) has clear: left, and margin-top: 30.
        // If clear: left applies, p's top border edge must be at least 70.
        assert!(approx_eq(float_layout.rect.origin.y, 0.0));
        assert!(approx_eq(p_layout.rect.origin.y, 100.0));
    }

    #[test]
    fn test_float_clearing_with_large_margin_top() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
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
            .float {
                float: left;
                width: 100px;
                height: 50px;
                margin-bottom: 20px;
            }
            p {
                display: block;
                clear: left;
                margin-top: 80px;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        let p_layout = &body_box.children[1];

        // The float bottom edge is at y = 50 + 20 = 70.
        // The normal_box (p) has clear: left, and margin-top: 80.
        // Since its hypothetical top border edge is < 70, clearance is applied.
        // It is placed at y = 150.0.
        assert!(approx_eq(p_layout.rect.origin.y, 150.0));
    }

    #[test]
    fn test_float_clearing_with_collapsed_preceding_margin() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(body, float_box);

        let prev_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, prev_box);

        let clearing_box = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(body, clearing_box);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .float {
                float: left;
                width: 100px;
                height: 50px;
                margin-bottom: 20px;
            }
            .prev {
                height: 40px;
                margin-bottom: 15px;
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

        let p_layout = &body_box.children[2];

        // p's border box top must clear the float (70.0 + margin_top 10.0 = 80.0)
        assert!(approx_eq(p_layout.rect.origin.y, 80.0));
    }

    #[test]
    fn test_clear_both_with_nested_intervening_block() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(body, float_box);

        let intervening_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "intervening".into())],
        });
        dom.append_child(body, intervening_box);

        let clearing_box = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(intervening_box, clearing_box);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .float {
                float: left;
                width: 100px;
                height: 50px;
            }
            .intervening {
                display: block;
                margin-top: 10px;
            }
            p {
                display: block;
                clear: both;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        let intervening_layout = &body_box.children[1];
        let p_layout = &intervening_layout.children[0];

        // The float bottom is 50.
        // The intervening block has no BFC, so its nested clearing p must clear the float.
        // Therefore, p's top border edge must be at least at 50.0.
        assert!(p_layout.rect.origin.y >= 50.0);
    }

    #[test]
    fn test_floats_placement_in_shrink_to_fit_container() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "container".into())],
        });
        dom.append_child(body, container);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(container, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(container, left_2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .container {
                display: inline-block;
            }
            .f1 {
                float: left;
                width: 100px;
                height: 50px;
            }
            .f2 {
                float: left;
                width: 120px;
                height: 60px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];
        let container_box = &body_box.children[0].children[0]; // account for anonymous block wrapper!

        // The container is display: inline-block, so it's shrink-to-fit.
        // During final layout, the second float wraps under the first one correctly.
        assert_eq!(container_box.children.len(), 2);
        let f1_layout = &container_box.children[0];
        let f2_layout = &container_box.children[1];

        assert!(approx_eq(f1_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 8.0));
        assert!(approx_eq(f2_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 58.0));
    }

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

    #[test]
    fn test_multiple_floats_stack_side_by_side() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, left_2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .f1 {
                float: left;
                width: 100px;
                height: 50px;
            }
            .f2 {
                float: left;
                width: 120px;
                height: 60px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let f1_layout = &body_box.children[0];
        let f2_layout = &body_box.children[1];

        // f1 is at x=0, y=0
        assert!(approx_eq(f1_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 0.0));

        // f2 stacks next to f1, so x=100, y=0
        assert!(approx_eq(f2_layout.rect.origin.x, 100.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 0.0));
    }

    #[test]
    fn test_float_wraps_vertically_when_width_constrained() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, left_2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .f1 {
                float: left;
                width: 150px;
                height: 50px;
            }
            .f2 {
                float: left;
                width: 150px;
                height: 60px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let f1_layout = &body_box.children[0];
        let f2_layout = &body_box.children[1];

        // f1 fits at x=0, y=0
        assert!(approx_eq(f1_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 0.0));

        // f2 has width 150, which doesn't fit next to f1 (150 + 150 = 300 > 200)
        // so it must wrap to below f1: x=0, y=50
        assert!(approx_eq(f2_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 50.0));
    }

    #[test]
    fn test_float_stacking_right() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let right_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, right_1);

        let right_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, right_2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .f1 {
                float: right;
                width: 100px;
                height: 50px;
            }
            .f2 {
                float: right;
                width: 120px;
                height: 60px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let f1_layout = &body_box.children[0];
        let f2_layout = &body_box.children[1];

        // f1 is at x = 500 - 100 = 400, y = 0
        assert!(approx_eq(f1_layout.rect.origin.x, 400.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 0.0));

        // f2 stacks next to f1 on the left, so x = 400 - 120 = 280, y = 0
        assert!(approx_eq(f2_layout.rect.origin.x, 280.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 0.0));
    }

    #[test]
    fn test_clearance_with_stacked_floats() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, left_2);

        let clearing_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(body, clearing_p);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .f1 {
                float: left;
                width: 150px;
                height: 50px;
            }
            .f2 {
                float: left;
                width: 150px;
                height: 60px;
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

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 3);
        let f2_layout = &body_box.children[1];
        let p_layout = &body_box.children[2];

        // f2 wrapped and is at x=0, y=50, height=60, so its bottom edge is 110.
        assert!(approx_eq(f2_layout.rect.origin.y, 50.0));
        assert!(approx_eq(f2_layout.rect.size.height, 60.0));

        // p has clear: left, so it must clear both f1 (bottom 50) and f2 (bottom 110).
        // Max bottom edge is 110.
        // p has margin-top = 10, so its border box y should be 110 + 10 = 120.
        assert!(approx_eq(p_layout.rect.origin.y, 120.0));
    }

    #[test]
    fn test_float_top_edge_not_higher_than_preceding_float_top_edge() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, left_2);

        let left_3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f3".into())],
        });
        dom.append_child(body, left_3);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .f1 {
                float: left;
                width: 150px;
                height: 50px;
            }
            .f2 {
                float: left;
                width: 150px;
                height: 60px;
            }
            .f3 {
                float: left;
                width: 30px;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 3);
        let f1_layout = &body_box.children[0];
        let f2_layout = &body_box.children[1];
        let f3_layout = &body_box.children[2];

        // f1 is at x=0, y=0
        assert!(approx_eq(f1_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 0.0));

        // f2 doesn't fit, wraps to y=50
        assert!(approx_eq(f2_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 50.0));

        // f3 must not have top edge higher than f2 (which is 50.0).
        // Since f3 width is 30, it fits next to f2 (at x=150, y=50).
        assert!(approx_eq(f3_layout.rect.origin.y, 50.0));
        assert!(approx_eq(f3_layout.rect.origin.x, 150.0));
    }

    #[test]
    fn test_bfc_isolation_for_floats() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // inline-block parent establishing a BFC
        let bfc_container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(body, bfc_container);

        // Nested float inside BFC container (should be isolated from outside)
        let inner_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "inner-float".into())],
        });
        dom.append_child(bfc_container, inner_float);

        // Outer float in body context
        let outer_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "outer-float".into())],
        });
        dom.append_child(body, outer_float);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .bfc {
                display: inline-block;
                width: 0px;
                height: 0px;
            }
            .inner-float {
                float: left;
                width: 50px;
                height: 50px;
            }
            .outer-float {
                float: left;
                width: 50px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        // Since the inner-float is isolated inside the BFC, the outer-float is positioned as if inner-float does not exist.
        // Therefore, outer-float is positioned at x = 0 (its margin-left/padding is 0).
        let outer_layout = &body_box.children[1];
        if !approx_eq(outer_layout.rect.origin.x, 0.0) {
            panic!(
                "Expected outer_layout.rect.origin.x to be 0.0, but got {}",
                outer_layout.rect.origin.x
            );
        }
    }

    #[test]
    fn test_bfc_isolation_for_clearance() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // inline-block parent establishing a BFC
        let bfc_container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(body, bfc_container);

        // Nested float inside BFC container (should be isolated)
        let inner_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "inner-float".into())],
        });
        dom.append_child(bfc_container, inner_float);

        // Clearing block in body context
        let clearing_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(body, clearing_p);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .bfc {
                display: inline-block;
                width: 0px;
                height: 0px;
            }
            .inner-float {
                float: left;
                width: 50px;
                height: 50px;
            }
            p {
                display: block;
                clear: left;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        // If inner-float is isolated, there are no active outer floats.
        // So clearing_p is positioned at its baseline offset, which is less than 40.0
        // (if it had cleared the inner-float of height 50, it would be >= 50.0).
        let p_layout = &body_box.children[1];
        if p_layout.rect.origin.y >= 40.0 {
            panic!(
                "Expected p_layout.rect.origin.y to be < 40.0, but got {}",
                p_layout.rect.origin.y
            );
        }
    }

    #[test]
    fn test_floats_overlap_vertically_various_cases() {
        use super::floats_overlap_vertically;
        // Non-zero heights, overlapping
        assert!(floats_overlap_vertically(0.0, 50.0, 25.0, 50.0));
        assert!(floats_overlap_vertically(10.0, 20.0, 15.0, 5.0));

        // Non-zero heights, touching but not overlapping
        assert!(!floats_overlap_vertically(0.0, 50.0, 50.0, 50.0));
        assert!(!floats_overlap_vertically(50.0, 50.0, 0.0, 50.0));

        // One zero height, overlapping
        assert!(floats_overlap_vertically(50.0, 50.0, 50.0, 0.0));
        assert!(floats_overlap_vertically(50.0, 50.0, 75.0, 0.0));
        assert!(!floats_overlap_vertically(50.0, 50.0, 100.0, 0.0));
        assert!(!floats_overlap_vertically(50.0, 50.0, 49.0, 0.0));

        // One zero height (preceding float is 0 height)
        assert!(floats_overlap_vertically(50.0, 0.0, 50.0, 50.0));
        assert!(floats_overlap_vertically(50.0, 0.0, 25.0, 50.0));
        assert!(!floats_overlap_vertically(50.0, 0.0, 100.0, 50.0));

        // Both zero heights
        assert!(floats_overlap_vertically(50.0, 0.0, 50.0, 0.0));
        assert!(!floats_overlap_vertically(50.0, 0.0, 51.0, 0.0));
    }

    #[test]
    fn test_zero_height_float_stacking() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "f2".into())],
        });
        dom.append_child(body, left_2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .f1 {
                float: left;
                width: 100px;
                height: 0px;
            }
            .f2 {
                float: left;
                width: 100px;
                height: 0px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let f1_layout = &body_box.children[0];
        let f2_layout = &body_box.children[1];

        // f1 is at x=0, y=0
        assert!(approx_eq(f1_layout.rect.origin.x, 0.0));
        assert!(approx_eq(f1_layout.rect.origin.y, 0.0));

        // f2 stacks next to f1, so x=100, y=0
        assert!(approx_eq(f2_layout.rect.origin.x, 100.0));
        assert!(approx_eq(f2_layout.rect.origin.y, 0.0));
    }

    #[test]
    fn test_bfc_isolation_for_clearance_nested_p() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // Outer float in body context
        let outer_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "outer-float".into())],
        });
        dom.append_child(body, outer_float);

        // inline-block parent establishing a BFC
        let bfc_container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(body, bfc_container);

        // Clearing block inside BFC container (should be isolated from outer-float!)
        let clearing_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(bfc_container, clearing_p);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .outer-float {
                float: left;
                width: 50px;
                height: 50px;
            }
            .bfc {
                display: inline-block;
                width: 100px;
                height: 40px;
            }
            p {
                display: block;
                clear: left;
                margin-top: 10px;
                height: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);

        fn print_box_recursive(lb: &super::LayoutBox, indent: usize) {
            let ind = "  ".repeat(indent);
            println!(
                "{}LayoutBox: node_id={:?}, rect={:?}, text={:?}",
                ind, lb.node, lb.rect, lb.text
            );
            for child in &lb.children {
                print_box_recursive(child, indent + 1);
            }
        }
        print_box_recursive(&layout_tree, 0);

        let body_box = &layout_tree.children[0];
        let anon_box = &body_box.children[1];
        let line_box = &anon_box.children[0];
        let bfc_layout = &line_box.children[0];
        let p_layout = &bfc_layout.children[0];

        let diff = p_layout.rect.origin.y - bfc_layout.rect.origin.y;
        println!(
            "p_layout Y: {}, bfc_layout Y: {}, diff: {}",
            p_layout.rect.origin.y, bfc_layout.rect.origin.y, diff
        );

        // The outer float has y=0, height=50.
        // bfc_layout is inline-block, placed next to outer-float at x=50, y=8.0 (default line box offset).
        // Since bfc_layout is a BFC, its internal clearing p must not clear the outer-float.
        // Since p is inside bfc_layout (which starts at y=8.0), and p has margin-top=10,
        // p's relative y inside bfc_layout should be 10.0 (absolute y = 18.0).
        // If BFC isolation is broken, p's top border edge clears outer-float (50.0), making relative y = 60.0.
        assert!(approx_eq(
            p_layout.rect.origin.y - bfc_layout.rect.origin.y,
            10.0
        ));
    }

    #[test]
    fn test_line_box_shortening_no_floats() {
        let dom = Dom::new();
        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);

        let children: Vec<LayoutBox> = Vec::new();
        // Containing block starting at left 10, width 400
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 0.0, 20.0, 10.0, 400.0);

        assert!(approx_eq(left, 10.0));
        assert!(approx_eq(right, 410.0));
    }

    #[test]
    fn test_line_box_shortening_left_float() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f_node);

        let stylesheet = parse_stylesheet(".float { float: left; width: 100px; height: 50px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Manually build LayoutBox for float
        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![float_box];

        // Query overlapping with float (y=10, h=20)
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 10.0, 20.0, 10.0, 400.0);
        // left bound should be pushed to float_right_edge = 10 (containing_left) + 100 (float_width) = 110.0
        assert!(approx_eq(left, 110.0));
        assert!(approx_eq(right, 410.0));

        // Query below the float (y=60, h=20)
        let (left_below, right_below) =
            super::get_line_box_bounds_at_y(&children, &styles, 60.0, 20.0, 10.0, 400.0);
        assert!(approx_eq(left_below, 10.0));
        assert!(approx_eq(right_below, 410.0));
    }

    #[test]
    fn test_line_box_shortening_right_float() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f_node);

        let stylesheet = parse_stylesheet(".float { float: right; width: 80px; height: 60px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Right float: placed at right edge, x = containing_right - width = 410 - 80 = 330
        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(330.0, 0.0, 80.0, 60.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![float_box];

        // Query overlapping with float (y=20, h=20)
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 20.0, 20.0, 10.0, 400.0);
        assert!(approx_eq(left, 10.0));
        // right bound should be shortened to float_left_edge = 330.0
        assert!(approx_eq(right, 330.0));
    }

    #[test]
    fn test_line_box_shortening_stacked_left_floats() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let f1_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f1_node);
        let f2_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f2_node);

        let stylesheet = parse_stylesheet(".float { float: left; width: 100px; height: 50px; }");
        let styles = compute_styles(&dom, &stylesheet);

        // Float 1: x = 10, y = 0, w = 100, h = 50
        let f1_box = LayoutBox {
            node: Some(f1_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        // Float 2: stacked next to it, x = 110, y = 0, w = 100, h = 40 (different height)
        let f2_box = LayoutBox {
            node: Some(f2_node),
            rect: crate::geom::Rect::new(110.0, 0.0, 100.0, 40.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![f1_box, f2_box];

        // Query overlapping both floats (y = 10, h = 20)
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 10.0, 20.0, 10.0, 400.0);
        // left bound should be max of both right edges: max(10 + 100, 110 + 100) = 210.0
        assert!(approx_eq(left, 210.0));
        assert!(approx_eq(right, 410.0));

        // Query overlapping only Float 1 (y = 45, h = 10) - Float 2 ends at 40
        let (left_only1, right_only1) =
            super::get_line_box_bounds_at_y(&children, &styles, 45.0, 10.0, 10.0, 400.0);
        assert!(approx_eq(left_only1, 110.0));
        assert!(approx_eq(right_only1, 410.0));
    }

    #[test]
    fn test_line_box_shortening_bfc_isolated_float() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let bfc_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(doc, bfc_node);

        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(bfc_node, f_node);

        let stylesheet = parse_stylesheet(
            "
            .bfc { display: inline-block; }
            .float { float: left; width: 100px; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // Outer layout children has bfc_box. Since it is a BFC, its internal float should be isolated from outer queries.
        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        let bfc_box = LayoutBox {
            node: Some(bfc_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: vec![float_box],
            text: None,
        };

        let children = vec![bfc_box];

        // Query outer line-box bounds (which is outside the BFC)
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 10.0, 20.0, 10.0, 400.0);
        // Should not be affected by the isolated inner float, so left bound should remain 10.0!
        assert!(approx_eq(left, 10.0));
        assert!(approx_eq(right, 410.0));
    }

    #[test]
    fn test_bfc_isolation_nested_not_immediate() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // Outer float in body context
        let outer_float = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "outer-float".into())],
        });
        dom.append_child(body, outer_float);

        // bfc container
        let bfc_container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(body, bfc_container);

        // sub-container (not BFC)
        let sub_container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "sub".into())],
        });
        dom.append_child(bfc_container, sub_container);

        // Clearing block inside sub-container
        let clearing_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(sub_container, clearing_p);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            .outer-float {
                float: left;
                width: 50px;
                height: 50px;
            }
            .bfc {
                display: inline-block;
                width: 100px;
                height: 40px;
            }
            .sub {
                display: block;
            }
            p {
                display: block;
                clear: left;
                margin-top: 10px;
                height: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];
        let anon_box = &body_box.children[1];
        let line_box = &anon_box.children[0];
        let bfc_layout = &line_box.children[0];
        let sub_layout = &bfc_layout.children[0];
        let p_layout = &sub_layout.children[0];

        // The outer float has y=0, height=50.
        // bfc_layout is inline-block, placed next to outer-float at x=50, y=8.0 (default line box offset).
        // Since bfc_layout is a BFC, its internal clearing p must not clear the outer-float,
        // even though p is nested inside a block-level .sub wrapper!
        // If BFC isolation works, p's relative y inside bfc_layout should be 10.0 (absolute y = 18.0).
        // If BFC isolation is broken, p's top border edge clears outer-float (50.0).
        assert!(approx_eq(
            p_layout.rect.origin.y - bfc_layout.rect.origin.y,
            10.0
        ));
    }

    #[test]
    fn test_line_box_shortening_with_anonymous_sibling() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let bfc_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(doc, bfc_node);

        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(bfc_node, f_node);

        let stylesheet = parse_stylesheet(
            "
            .bfc { display: inline-block; }
            .float { float: left; width: 100px; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        let bfc_box = LayoutBox {
            node: Some(bfc_node),
            rect: crate::geom::Rect::new(10.0, 0.0, 100.0, 50.0),
            children: vec![float_box],
            text: None,
        };

        let anon_box = LayoutBox {
            node: None,
            rect: crate::geom::Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![anon_box, bfc_box];

        // Query outer line-box bounds
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 10.0, 20.0, 10.0, 400.0);
        // BFC isolation should still work even when the first sibling is anonymous (node = None)!
        assert!(approx_eq(left, 10.0));
        assert!(approx_eq(right, 410.0));
    }

    #[test]
    fn test_line_box_shortening_bfc_isolates_outer_float_with_anonymous_sibling() {
        let mut dom = Dom::new();
        let doc = dom.document();

        // Outer float in body context
        let outer_float_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "outer-float".into())],
        });
        dom.append_child(doc, outer_float_node);

        // BFC container
        let bfc_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(doc, bfc_node);

        // Inside BFC container: we have some child
        let inner_child_node = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(bfc_node, inner_child_node);

        let stylesheet = parse_stylesheet(
            "
            .outer-float { float: left; width: 50px; height: 50px; }
            .bfc { display: inline-block; width: 100px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // Record the outer float in the session registry
        SESSION.with(|session| {
            let mut s = session.borrow_mut();
            s.floats.clear();
            s.floats.push(RegisteredFloat {
                node_id: outer_float_node,
                float_type: "left".to_string(),
                x: 10.0,
                y: 0.0,
                width: 50.0,
                height: 50.0,
            });
        });

        // Let's build layout box for the inner children
        let inner_anon_box = LayoutBox {
            node: None,
            rect: crate::geom::Rect::new(0.0, 0.0, 0.0, 0.0),
            children: Vec::new(),
            text: None,
        };

        let inner_child_box = LayoutBox {
            node: Some(inner_child_node),
            rect: crate::geom::Rect::new(10.0, 10.0, 80.0, 20.0),
            children: Vec::new(),
            text: None,
        };

        // Sibling children inside the BFC: first is anonymous, second is a normal element
        let children = vec![inner_anon_box, inner_child_box];

        // Query line-box bounds inside the BFC
        let (left, right) =
            super::get_line_box_bounds_at_y(&children, &styles, 10.0, 20.0, 10.0, 100.0);
        // Because the line box is inside the BFC, it must be completely isolated from the outer float!
        assert!(approx_eq(left, 10.0));
        assert!(approx_eq(right, 110.0));
    }

    #[test]
    fn test_bfc_width_reduction_and_position_shifting() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f_node);

        let bfc_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(doc, bfc_node);

        let stylesheet = parse_stylesheet(
            "
            .float { float: left; width: 100px; height: 50px; }
            .bfc { display: inline-block; width: 250px; height: 100px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // Build LayoutBox for the float: x = 0, y = 0, w = 100, h = 50
        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(0.0, 0.0, 100.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        // Build LayoutBox for the BFC: x = 0, y = 0, w = 150, h = 100
        let mut bfc_box = LayoutBox {
            node: Some(bfc_node),
            rect: crate::geom::Rect::new(0.0, 0.0, 150.0, 100.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![float_box];

        // Call our BFC width reduction function!
        // Containing left = 0.0, containing width = 300.0
        super::adjust_bfc_width_and_position(&mut bfc_box, &children, &styles, 0.0, 300.0);

        // Since bfc_box has width 150, and float_box takes 100, the available space is 300 - 100 = 200.
        // So the BFC fits, but its left edge is shifted to x = 100. Its width remains 150.
        assert!(approx_eq(bfc_box.rect.origin.x, 100.0));
        assert!(approx_eq(bfc_box.rect.size.width, 150.0));
        assert!(approx_eq(bfc_box.rect.origin.y, 0.0));
    }

    #[test]
    fn test_bfc_width_reduction_shifting_down() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let f_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float".into())],
        });
        dom.append_child(doc, f_node);

        let bfc_node = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "bfc".into())],
        });
        dom.append_child(doc, bfc_node);

        let stylesheet = parse_stylesheet(
            "
            .float { float: left; width: 150px; height: 50px; }
            .bfc { display: inline-block; width: 200px; height: 40px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        // Build LayoutBox for the float: x = 0, y = 0, w = 150, h = 50
        let float_box = LayoutBox {
            node: Some(f_node),
            rect: crate::geom::Rect::new(0.0, 0.0, 150.0, 50.0),
            children: Vec::new(),
            text: None,
        };

        // Build LayoutBox for the BFC: x = 0, y = 0, w = 200, h = 40
        let mut bfc_box = LayoutBox {
            node: Some(bfc_node),
            rect: crate::geom::Rect::new(0.0, 0.0, 200.0, 40.0),
            children: Vec::new(),
            text: None,
        };

        let children = vec![float_box];

        // Containing left = 0.0, containing width = 300.0
        super::adjust_bfc_width_and_position(&mut bfc_box, &children, &styles, 0.0, 300.0);

        // The BFC element (outer width 200) does not fit next to the float (width 150) because 200 + 150 = 350 > 300.
        // Therefore, it must be shifted down below the float (y = 50.0), where it can expand to its full width (200.0).
        assert!(approx_eq(bfc_box.rect.origin.y, 50.0));
        assert!(approx_eq(bfc_box.rect.origin.x, 0.0));
        assert!(approx_eq(bfc_box.rect.size.width, 200.0));
    }

    #[test]
    fn test_clear_both_complex_with_multiple_stacked_floats() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let left_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "left1".into())],
        });
        dom.append_child(body, left_1);

        let left_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "left2".into())],
        });
        dom.append_child(body, left_2);

        let right_1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "right1".into())],
        });
        dom.append_child(body, right_1);

        let right_2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "right2".into())],
        });
        dom.append_child(body, right_2);

        let clearing_p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(body, clearing_p);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(clearing_p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 400px; }
            .left1 { float: left; width: 80px; height: 40px; }
            .left2 { float: left; width: 80px; height: 60px; }
            .right1 { float: right; width: 90px; height: 50px; }
            .right2 { float: right; width: 90px; height: 70px; }
            p { display: block; clear: both; margin-top: 15px; height: 20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 400.0);
        let body_box = &layout_tree.children[0];

        // Clearing block should be at index 4 (since we have 4 floats)
        let p_layout = &body_box.children[4];

        // Left floats stack:
        // left1 fits at x=0, y=0. Max bottom is 40.
        // left2 fits next to it at x=80, y=0. Max bottom is 60.
        // Right floats stack:
        // right1 fits at x=310, y=0. Max bottom is 50.
        // right2 fits next to right1 (to the left) at x=220, y=0. Max bottom is 70.
        // Maximum bottom edge of any float is 70.
        // p clears both, so its top border edge must be 70.
        // Since margin-top is 15, its origin.y should be 70 + 15 = 85.
        assert!(approx_eq(p_layout.rect.origin.y, 85.0));
    }

    #[test]
    fn test_negative_margin_float_clamping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float-neg".into())],
        });
        dom.append_child(body, float_box);

        let sibling_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "sibling".into())],
        });
        dom.append_child(body, sibling_box);

        let text = dom.create_node(NodeData::Text("ab".into()));
        dom.append_child(sibling_box, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 300px; }
            .float-neg {
                float: left;
                width: 100px;
                height: 50px;
                margin-right: -120px;
                margin-bottom: -60px;
            }
            .sibling {
                display: block;
                clear: left;
                height: 30px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 300.0);
        let body_box = &layout_tree.children[0];

        // Ensure we laid out both elements safely.
        assert_eq!(body_box.children.len(), 2);
        let f_layout = &body_box.children[0];
        let s_layout = &body_box.children[1];

        // The float_neg element has a border-box height of 50.0.
        // Its margin-bottom is -60px.
        // Its outer height is max(0.0, 50 - 60) = 0.0.
        // The clearance for sibling (clear: left) should clear the float.
        // Since outer height is clamped to 0.0, the bottom edge is y = 0.0.
        // So the cleared sibling can be placed at y = 0.0 without any issues.
        assert!(approx_eq(f_layout.rect.origin.y, 0.0));
        assert!(approx_eq(s_layout.rect.origin.y, 0.0));
    }

    #[test]
    fn test_float_negative_margin_positioning() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let float_box = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "float-neg-pos".into())],
        });
        dom.append_child(body, float_box);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 300px; }
            .float-neg-pos {
                float: left;
                width: 100px;
                height: 50px;
                margin-left: -20px;
                margin-top: -10px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 300.0);
        let body_box = &layout_tree.children[0];

        let f_layout = &body_box.children[0];

        // The actual position of the float should still reflect the negative margins:
        // final_x = left_bound + margin_left = 0.0 - 20.0 = -20.0
        // final_y = candidate_y + margin_top = 0.0 - 10.0 = -10.0
        assert!(approx_eq(f_layout.rect.origin.x, -20.0));
        assert!(approx_eq(f_layout.rect.origin.y, -10.0));
    }
}
