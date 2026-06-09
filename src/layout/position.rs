use crate::css::values::CssValue;
use crate::dom::Dom;
use crate::infra::NodeId;
use crate::layout::{LayoutBox, get_px, layout_node};
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// Helper to check if a node is absolutely or fixed positioned.
/// spec: S-31
pub fn is_absolute_or_fixed(styles: &HashMap<NodeId, ComputedStyle>, node: NodeId) -> bool {
    if let Some(style) = styles.get(&node) {
        matches!(style.get("position"), Some(CssValue::Keyword(kw)) if kw == "absolute" || kw == "fixed")
    } else {
        false
    }
}

/// Shifts a LayoutBox and its non-absolute/non-fixed descendants by (dx, dy).
/// spec: S-31
pub fn shift_layout_box(
    layout_box: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    dx: f32,
    dy: f32,
) {
    if dx == 0.0 && dy == 0.0 {
        return;
    }
    if let Some(style) = layout_box.node.and_then(|node_id| styles.get(&node_id))
        && matches!(style.get("position"), Some(CssValue::Keyword(kw)) if kw == "absolute" || kw == "fixed")
    {
        // Do not shift absolute or fixed elements or their descendants.
        // spec: S-31
        return;
    }

    layout_box.rect.origin.x += dx;
    layout_box.rect.origin.y += dy;
    for child in &mut layout_box.children {
        shift_layout_box(child, styles, dx, dy);
    }
}

/// Recursively resolves relative positions for the entire layout tree.
/// spec: S-31
pub fn resolve_relative_positions(
    layout_box: &mut LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
) {
    for child in &mut layout_box.children {
        resolve_relative_positions(child, styles);
    }

    if let Some(style) = layout_box.node.and_then(|node_id| styles.get(&node_id))
        && matches!(style.get("position"), Some(CssValue::Keyword(kw)) if kw == "relative")
    {
        let dx = get_px(style, "left", 0.0);
        let dy = get_px(style, "top", 0.0);
        shift_layout_box(layout_box, styles, dx, dy);
    }
}

/// Recursively finds all absolute and fixed elements in pre-order.
/// spec: S-31
pub fn find_absolute_and_fixed(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    out: &mut Vec<NodeId>,
) {
    if let Some(style) = styles.get(&node)
        && matches!(style.get("position"), Some(CssValue::Keyword(kw)) if kw == "absolute" || kw == "fixed")
    {
        out.push(node);
    }
    for &child in dom.children(node) {
        find_absolute_and_fixed(dom, styles, child, out);
    }
}

/// Helper to recursively check if a LayoutBox with given node_id exists.
/// spec: S-31
pub fn has_layout_box(layout_box: &LayoutBox, node_id: NodeId) -> bool {
    if layout_box.node == Some(node_id) {
        return true;
    }
    for child in &layout_box.children {
        if has_layout_box(child, node_id) {
            return true;
        }
    }
    false
}

/// Recursively searches for the LayoutBox with given node_id and returns a mutable reference.
/// spec: S-31
pub fn find_layout_box_mut(layout_box: &mut LayoutBox, node_id: NodeId) -> Option<&mut LayoutBox> {
    if layout_box.node == Some(node_id) {
        return Some(layout_box);
    }
    for child in &mut layout_box.children {
        if let Some(found) = find_layout_box_mut(child, node_id) {
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
        if has_layout_box(layout_tree, ancestor) {
            target_ancestor = Some(ancestor);
            break;
        }
        current = dom.parent(ancestor);
    }

    if let Some(ancestor) = target_ancestor
        && let Some(parent_box) = find_layout_box_mut(layout_tree, ancestor)
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
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
    root_box: &mut LayoutBox,
) {
    let mut absolute_nodes = Vec::new();
    find_absolute_and_fixed(dom, styles, dom.document(), &mut absolute_nodes);

    for node in absolute_nodes {
        // Retrieve computed styles
        let style = match styles.get(&node) {
            Some(s) => s,
            None => continue,
        };

        // If display is none, it doesn't get a box
        if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "none") {
            continue;
        }

        // absolute/fixed position: top/left basic relative to containing block (viewport/root)
        // spec: S-31
        let left = get_px(style, "left", 0.0);
        let top = get_px(style, "top", 0.0);

        // Layout the node with viewport width as containing width, and top/left as offsets
        if let Some(child_box) = layout_node(dom, styles, node, viewport_width, left, top, 0) {
            // Find nearest ancestor in the layout tree and append to its children
            insert_into_nearest_ancestor_layout_box(dom, root_box, node, child_box);
        }
    }
}
