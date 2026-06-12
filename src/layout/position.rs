use crate::dom::Dom;
use crate::infra::NodeId;
use crate::layout::{LayoutBox, layout_node};
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

/// Helper to check if a node is absolutely or fixed positioned.
/// spec: S-31
pub fn is_absolute_or_fixed(
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
) -> bool {
    if let Some(style) = styles.get(&node) {
        let is_abs_or_fixed =
            style.reset_box.position == "absolute" || style.reset_box.position == "fixed";
        if is_abs_or_fixed {
            let has_explicit_top = style.reset_surround.top != -1;
            let has_explicit_left = style.reset_surround.left != -1;

            // TODO(spec): True CSS static-position-for-out-of-flow semantics is deferred:
            // we use an interim decision where if both top and left are unspecified (auto),
            // we keep the element in normal flow (as if position: static) to avoid collapsing to (0,0).
            has_explicit_top || has_explicit_left
        } else {
            false
        }
    } else {
        false
    }
}

/// Shifts a LayoutBox and its non-absolute/non-fixed descendants by (dx, dy).
/// spec: S-31
pub fn shift_layout_box(
    layout_box: &mut LayoutBox,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    dx: f32,
    dy: f32,
    depth: usize,
) {
    if depth > crate::layout::MAX_DEPTH {
        return;
    }
    if dx == 0.0 && dy == 0.0 {
        return;
    }
    if let Some(node_id) = layout_box.node
        && is_absolute_or_fixed(styles, node_id)
    {
        // Do not shift absolute or fixed elements or their descendants.
        // spec: S-31
        return;
    }

    layout_box.rect.origin.x += dx;
    layout_box.rect.origin.y += dy;
    for child in &mut layout_box.children {
        shift_layout_box(child, styles, dx, dy, depth + 1);
    }
}

/// Recursively resolves relative positions for the entire layout tree.
/// spec: S-31
pub fn resolve_relative_positions(
    layout_box: &mut LayoutBox,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    depth: usize,
) {
    if depth > crate::layout::MAX_DEPTH {
        return;
    }
    for child in &mut layout_box.children {
        resolve_relative_positions(child, styles, depth + 1);
    }

    if let Some(style) = layout_box.node.and_then(|node_id| styles.get(&node_id))
        && style.reset_box.position == "relative"
    {
        let dx = if style.reset_surround.left == -1 {
            0.0
        } else {
            style.reset_surround.left as f32
        };
        let dy = if style.reset_surround.top == -1 {
            0.0
        } else {
            style.reset_surround.top as f32
        };
        shift_layout_box(layout_box, styles, dx, dy, depth);
    }
}

/// Recursively finds all absolute and fixed elements in pre-order, pruning on display: none.
/// spec: S-31
pub fn find_absolute_and_fixed(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
    out: &mut Vec<NodeId>,
    depth: usize,
) {
    if depth > crate::layout::MAX_DEPTH {
        return;
    }
    if let Some(style) = styles.get(&node)
        && style.reset_box.display == "none"
    {
        // Prune the subtree if display: none
        return;
    }
    if is_absolute_or_fixed(styles, node) {
        out.push(node);
    }
    for &child in dom.children(node) {
        find_absolute_and_fixed(dom, styles, child, out, depth + 1);
    }
}

/// Helper to recursively check if a LayoutBox with given node_id exists.
/// spec: S-31
pub fn has_layout_box(layout_box: &LayoutBox, node_id: NodeId, depth: usize) -> bool {
    if depth > crate::layout::MAX_DEPTH {
        return false;
    }
    if layout_box.node == Some(node_id) {
        return true;
    }
    for child in &layout_box.children {
        if has_layout_box(child, node_id, depth + 1) {
            return true;
        }
    }
    false
}

/// Recursively searches for the LayoutBox with given node_id and returns a mutable reference.
/// spec: S-31
pub fn find_layout_box_mut(
    layout_box: &mut LayoutBox,
    node_id: NodeId,
    depth: usize,
) -> Option<&mut LayoutBox> {
    if depth > crate::layout::MAX_DEPTH {
        return None;
    }
    if layout_box.node == Some(node_id) {
        return Some(layout_box);
    }
    for child in &mut layout_box.children {
        if let Some(found) = find_layout_box_mut(child, node_id, depth + 1) {
            return Some(found);
        }
    }
    None
}

/// Finds the nearest ancestor of `node` that has a LayoutBox in `layout_tree`,
/// and appends `child_box` to its `children`.
/// spec: S-31
pub fn insert_into_nearest_ancestor_layout_box(
    dom: &Dom,
    layout_tree: &mut LayoutBox,
    node: NodeId,
    child_box: LayoutBox,
) {
    let mut current = dom.parent(node);
    let mut target_ancestor = None;
    while let Some(ancestor) = current {
        if has_layout_box(layout_tree, ancestor, 0) {
            target_ancestor = Some(ancestor);
            break;
        }
        current = dom.parent(ancestor);
    }

    if let Some(ancestor) = target_ancestor
        && let Some(parent_box) = find_layout_box_mut(layout_tree, ancestor, 0)
    {
        parent_box.children.push(child_box);
        return;
    }
    // Default to root_box
    layout_tree.children.push(child_box);
}

/// Performs layout for absolute and fixed elements and integrates them into the layout tree.
/// spec: S-31
pub fn layout_absolute_and_fixed_elements(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    viewport_width: f32,
    root_box: &mut LayoutBox,
) {
    let mut absolute_nodes = Vec::new();
    find_absolute_and_fixed(dom, styles, dom.document(), &mut absolute_nodes, 0);

    for node in absolute_nodes {
        // Retrieve computed styles
        let style = match styles.get(&node) {
            Some(s) => s,
            None => continue,
        };

        // If display is none, it doesn't get a box
        if style.reset_box.display == "none" {
            continue;
        }

        // absolute/fixed position: top/left basic relative to containing block (viewport/root)
        // spec: S-31
        let left = if style.reset_surround.left == -1 {
            0.0
        } else {
            style.reset_surround.left as f32
        };
        let top = if style.reset_surround.top == -1 {
            0.0
        } else {
            style.reset_surround.top as f32
        };

        // Layout the node with viewport width as containing width, and top/left as offsets
        if let Some(child_box) = layout_node(dom, styles, node, viewport_width, left, top, 0) {
            // Find nearest ancestor in the layout tree and append to its children
            insert_into_nearest_ancestor_layout_box(dom, root_box, node, child_box);
        }
    }
}
