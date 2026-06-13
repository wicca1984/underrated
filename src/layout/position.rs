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
            let has_explicit_right = style.reset_surround.right != -1;

            // TODO(spec): True CSS static-position-for-out-of-flow semantics is deferred:
            // we use an interim decision where if both top and left are unspecified (auto)
            // (and we also check right here to avoid normal flow when right is specified),
            // we keep the element in normal flow (as if position: static) to avoid collapsing to (0,0).
            has_explicit_top || has_explicit_left || has_explicit_right
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
        && (style.reset_box.position == "relative" || style.reset_box.position == "sticky")
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
        if style.reset_box.position == "sticky" {
            // TODO(spec): true scroll-threshold sticky behavior is deferred (no scroll context yet).
        }
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

        // TODO(spec): bottom offset needs containing-block height (not threaded into this signature)
        let top = if style.reset_surround.top == -1 {
            0.0
        } else {
            style.reset_surround.top as f32
        };

        // Layout the node with viewport width as containing width, and top/left as offsets
        if let Some(mut child_box) = layout_node(dom, styles, node, viewport_width, left, top, 0) {
            // If left is auto (-1) and right is set (not -1), position from the right offset.
            if style.reset_surround.left == -1 && style.reset_surround.right != -1 {
                let right = style.reset_surround.right as f32;
                let target_x = viewport_width - right - child_box.rect.size.width;
                let shift_dx = target_x - child_box.rect.origin.x;
                child_box.rect.origin.x += shift_dx;
                for child in &mut child_box.children {
                    shift_layout_box(child, styles, shift_dx, 0.0, 1);
                }
            }

            // Find nearest ancestor in the layout tree and append to its children
            insert_into_nearest_ancestor_layout_box(dom, root_box, node, child_box);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::layout::layout_document;
    use crate::style::compute_styles;

    #[test]
    fn test_sticky_position_offset_static_render() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let sibling = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, sibling);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                height: 50px;
            }
            body > div:first-child {
                position: sticky;
                top: 15px;
                left: 25px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // First div (sticky)
        let div_box = &body_box.children[0];
        assert_eq!(div_box.node, Some(div));
        // Static layout position would be (0, 0)
        // With sticky top:15px; left:25px, it should behave like relative and offset to (25, 15)
        assert_eq!(div_box.rect.origin.x, 25.0);
        assert_eq!(div_box.rect.origin.y, 15.0);

        // Second div (sibling)
        let sibling_box = &body_box.children[1];
        assert_eq!(sibling_box.node, Some(sibling));
        // Sibling position should not be affected by first div's sticky offset
        // Static height of first div is 50px. Sibling should start at (0, 50).
        assert_eq!(sibling_box.rect.origin.x, 0.0);
        assert_eq!(sibling_box.rect.origin.y, 50.0);
    }

    #[test]
    fn test_absolute_position_right_offset() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                position: absolute;
                right: 40px;
                width: 120px;
                height: 50px;
                top: 10px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut div_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(div) {
                div_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let div_box = div_box.expect("Absolute div box not found in layout tree");
        // Expected x = viewport_width - right - width = 800.0 - 40.0 - 120.0 = 640.0
        assert_eq!(div_box.rect.size.width, 120.0);
        assert_eq!(div_box.rect.origin.x, 640.0);
        assert_eq!(div_box.rect.origin.y, 10.0);
    }

    #[test]
    fn test_absolute_position_left_wins_over_right() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                position: absolute;
                left: 30px;
                right: 40px;
                width: 120px;
                height: 50px;
                top: 10px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut div_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(div) {
                div_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let div_box = div_box.expect("Absolute div box not found in layout tree");
        // Since both left and right are set, left (30.0) should win.
        assert_eq!(div_box.rect.origin.x, 30.0);
        assert_eq!(div_box.rect.origin.y, 10.0);
    }
}
