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
            let has_explicit_bottom = style.reset_surround.bottom != -1;

            // TODO(spec): True CSS static-position-for-out-of-flow semantics is deferred:
            // we use an interim decision where if both top and left are unspecified (auto)
            // (and we also check right here to avoid normal flow when right is specified),
            // we keep the element in normal flow (as if position: static) to avoid collapsing to (0,0).
            has_explicit_top || has_explicit_left || has_explicit_right || has_explicit_bottom
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
        let dx = if style.reset_surround.left != -1 && style.reset_surround.right != -1 {
            if style.inherited_text.direction == "rtl" {
                -(style.reset_surround.right as f32)
            } else {
                style.reset_surround.left as f32
            }
        } else if style.reset_surround.left != -1 {
            style.reset_surround.left as f32
        } else if style.reset_surround.right != -1 {
            -(style.reset_surround.right as f32)
        } else {
            0.0
        };
        let dy = if style.reset_surround.top != -1 {
            style.reset_surround.top as f32
        } else if style.reset_surround.bottom != -1 {
            -(style.reset_surround.bottom as f32)
        } else {
            0.0
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

fn get_form_control_button_label_local(dom: &Dom, node: NodeId) -> Option<String> {
    if let Some(crate::dom::NodeData::Element { name, .. }) = dom.data(node) {
        if name.eq_ignore_ascii_case("button") {
            return Some(dom.text_content(node));
        } else if name.eq_ignore_ascii_case("input")
            && let Some(type_attr) = dom.get_attribute(node, "type")
        {
            let t_trimmed = type_attr.trim();
            if t_trimmed.eq_ignore_ascii_case("submit")
                || t_trimmed.eq_ignore_ascii_case("button")
                || t_trimmed.eq_ignore_ascii_case("reset")
            {
                let label = dom
                    .get_attribute(node, "value")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Submit".to_string());
                return Some(label);
            }
        }
    }
    None
}

fn is_inline_level_local(
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    dom: &Dom,
    child: NodeId,
) -> bool {
    if let Some(data) = dom.data(child) {
        match data {
            crate::dom::NodeData::Text(_) => true,
            crate::dom::NodeData::Element { .. } => {
                if let Some(style) = styles.get(&child) {
                    let disp = &style.reset_box.display;
                    disp == "inline" || disp == "inline-block"
                } else {
                    false
                }
            }
            _ => false,
        }
    } else {
        false
    }
}

fn max_content_width_local(
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    node: NodeId,
    depth: usize,
) -> f32 {
    if depth > crate::layout::MAX_DEPTH {
        return 0.0;
    }

    if let Some(style) = styles.get(&node)
        && style.reset_box.width != -1
    {
        return style.reset_box.width as f32;
    }

    if let Some(label) = get_form_control_button_label_local(dom, node) {
        return crate::font::BitmapFont::builtin().measure(&label) as f32;
    }

    if let Some(crate::dom::NodeData::Text(text)) = dom.data(node) {
        return crate::font::BitmapFont::builtin().measure(text) as f32;
    }

    let children = crate::layout::get_layoutable_children(dom, styles, node);
    if children.is_empty() {
        return 0.0;
    }

    let mut has_block_child = false;
    let mut children_contributions = Vec::with_capacity(children.len());

    for &child in &children {
        let child_content_width = max_content_width_local(dom, styles, child, depth + 1);
        let mut child_h_padding_border = 0.0;
        if let Some(child_style) = styles.get(&child) {
            child_h_padding_border += crate::layout::get_px(child_style, "padding-left", 0.0);
            child_h_padding_border += crate::layout::get_px(child_style, "padding-right", 0.0);
            child_h_padding_border += crate::layout::get_px(child_style, "border-left-width", 0.0);
            child_h_padding_border += crate::layout::get_px(child_style, "border-right-width", 0.0);
            if !is_inline_level_local(styles, dom, child) {
                has_block_child = true;
            }
        }
        children_contributions.push(child_content_width + child_h_padding_border);
    }

    if has_block_child {
        children_contributions
            .into_iter()
            .fold(0.0_f32, |acc, w| acc.max(w))
    } else {
        children_contributions.into_iter().sum()
    }
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

        // Determine containing block's origin and size (default to viewport)
        let mut ancestor_origin = (0.0, 0.0);
        let mut container_width = viewport_width;
        let mut container_height = root_box.rect.size.height;

        if style.reset_box.position == "absolute" {
            let mut current = dom.parent(node);
            let mut positioned_ancestor = None;
            while let Some(ancestor) = current {
                if let Some(anc_style) = styles.get(&ancestor) {
                    let pos = &anc_style.reset_box.position;
                    if pos == "relative" || pos == "absolute" || pos == "fixed" || pos == "sticky" {
                        positioned_ancestor = Some(ancestor);
                        break;
                    }
                }
                current = dom.parent(ancestor);
            }

            if let Some(ancestor_id) = positioned_ancestor
                && let Some(ancestor_box) = find_layout_box_mut(root_box, ancestor_id, 0)
            {
                let mut border_left = 0.0;
                let mut border_top = 0.0;
                let mut border_right = 0.0;
                let mut border_bottom = 0.0;
                if let Some(anc_style) = styles.get(&ancestor_id) {
                    border_left = crate::layout::get_px(anc_style, "border-left-width", 0.0);
                    border_top = crate::layout::get_px(anc_style, "border-top-width", 0.0);
                    border_right = crate::layout::get_px(anc_style, "border-right-width", 0.0);
                    border_bottom = crate::layout::get_px(anc_style, "border-bottom-width", 0.0);
                }
                ancestor_origin = (
                    ancestor_box.rect.origin.x + border_left,
                    ancestor_box.rect.origin.y + border_top,
                );
                container_width =
                    (ancestor_box.rect.size.width - border_left - border_right).max(0.0);
                container_height =
                    (ancestor_box.rect.size.height - border_top - border_bottom).max(0.0);
            }
        }

        // absolute/fixed position: top/left basic relative to containing block (viewport/root)
        // spec: S-31
        let left = if style.reset_surround.left == -1 {
            ancestor_origin.0
        } else {
            ancestor_origin.0 + style.reset_surround.left as f32
        };

        // TODO(spec): bottom offset needs containing-block height (not threaded into this signature)
        let top = if style.reset_surround.top == -1 {
            ancestor_origin.1
        } else {
            ancestor_origin.1 + style.reset_surround.top as f32
        };

        // Determine available width based on offsets
        let has_left = style.reset_surround.left != -1;
        let has_right = style.reset_surround.right != -1;

        let available_width = if has_left && has_right {
            let left_val = style.reset_surround.left as f32;
            let right_val = style.reset_surround.right as f32;
            (container_width - left_val - right_val).max(0.0)
        } else if has_left {
            let left_val = style.reset_surround.left as f32;
            (container_width - left_val).max(0.0)
        } else if has_right {
            let right_val = style.reset_surround.right as f32;
            (container_width - right_val).max(0.0)
        } else {
            container_width
        };

        // Determine containing width parameter for layout_node
        let layout_containing_width = if style.reset_box.width == -1 {
            if has_left && has_right {
                // Stretched width!
                available_width
            } else {
                // Shrink-to-fit width!
                let preferred_width = max_content_width_local(dom, styles, node, 0);
                let padding_left = crate::layout::get_px(style, "padding-left", 0.0);
                let padding_right = crate::layout::get_px(style, "padding-right", 0.0);
                let border_left = crate::layout::get_px(style, "border-left-width", 0.0);
                let border_right = crate::layout::get_px(style, "border-right-width", 0.0);
                let margin_left = crate::layout::get_px(style, "margin-left", 0.0);
                let margin_right = crate::layout::get_px(style, "margin-right", 0.0);
                let h_padding_border_margin = padding_left
                    + padding_right
                    + border_left
                    + border_right
                    + margin_left
                    + margin_right;
                let target_width = preferred_width + h_padding_border_margin;
                target_width.min(available_width).max(0.0)
            }
        } else {
            container_width
        };

        // Layout the node with computed containing width, and top/left as offsets
        if let Some(mut child_box) =
            layout_node(dom, styles, node, layout_containing_width, left, top, 0)
        {
            let has_left = style.reset_surround.left != -1;
            let has_right = style.reset_surround.right != -1;

            if has_left && has_right && style.reset_box.width != -1 {
                // If both left and right are specified, and width is specified (not auto),
                // we check if we should perform margin-auto horizontal centering.
                if style.reset_surround.margin_left == -1 && style.reset_surround.margin_right == -1
                {
                    let left_val = style.reset_surround.left as f32;
                    let right_val = style.reset_surround.right as f32;
                    let border_box_width = child_box.rect.size.width;
                    let extra_space = container_width - left_val - right_val - border_box_width;
                    let target_x = if extra_space >= 0.0 {
                        ancestor_origin.0 + left_val + (extra_space / 2.0)
                    } else if style.inherited_text.direction == "rtl" {
                        ancestor_origin.0 + container_width - right_val - border_box_width
                    } else {
                        ancestor_origin.0 + left_val
                    };
                    let shift_dx = target_x - child_box.rect.origin.x;
                    if shift_dx != 0.0 {
                        child_box.rect.origin.x += shift_dx;
                        for child in &mut child_box.children {
                            shift_layout_box(child, styles, shift_dx, 0.0, 1);
                        }
                    }
                } else if style.inherited_text.direction == "rtl" {
                    // Right wins over left
                    let right = style.reset_surround.right as f32;
                    let margin_right = if style.reset_surround.margin_right == -1 {
                        0.0
                    } else {
                        style.reset_surround.margin_right as f32
                    };
                    let target_x = ancestor_origin.0 + container_width
                        - right
                        - margin_right
                        - child_box.rect.size.width;
                    let shift_dx = target_x - child_box.rect.origin.x;
                    if shift_dx != 0.0 {
                        child_box.rect.origin.x += shift_dx;
                        for child in &mut child_box.children {
                            shift_layout_box(child, styles, shift_dx, 0.0, 1);
                        }
                    }
                }
            } else {
                // If left is auto (-1) and right is set (not -1), position from the right offset.
                // Also, if both are set and direction is RTL, right wins over left.
                let use_right_for_rtl =
                    has_left && has_right && style.inherited_text.direction == "rtl";

                if (!has_left && has_right) || use_right_for_rtl {
                    let right = style.reset_surround.right as f32;
                    let margin_right = if style.reset_surround.margin_right == -1 {
                        0.0
                    } else {
                        style.reset_surround.margin_right as f32
                    };
                    let target_x = ancestor_origin.0 + container_width
                        - right
                        - margin_right
                        - child_box.rect.size.width;
                    let shift_dx = target_x - child_box.rect.origin.x;
                    if shift_dx != 0.0 {
                        child_box.rect.origin.x += shift_dx;
                        for child in &mut child_box.children {
                            shift_layout_box(child, styles, shift_dx, 0.0, 1);
                        }
                    }
                }
            }

            let has_top = style.reset_surround.top != -1;
            let has_bottom = style.reset_surround.bottom != -1;

            if has_top && has_bottom && style.reset_box.height != -1 {
                // If both top and bottom are specified, and height is specified (not auto),
                // we check if we should perform margin-auto vertical centering.
                if style.reset_surround.margin_top == -1 && style.reset_surround.margin_bottom == -1
                {
                    let top_val = style.reset_surround.top as f32;
                    let bottom_val = style.reset_surround.bottom as f32;
                    let border_box_height = child_box.rect.size.height;
                    let extra_space = container_height - top_val - bottom_val - border_box_height;
                    let target_y = if extra_space >= 0.0 {
                        ancestor_origin.1 + top_val + (extra_space / 2.0)
                    } else {
                        ancestor_origin.1 + top_val
                    };
                    let shift_dy = target_y - child_box.rect.origin.y;
                    if shift_dy != 0.0 {
                        child_box.rect.origin.y += shift_dy;
                        for child in &mut child_box.children {
                            shift_layout_box(child, styles, 0.0, shift_dy, 1);
                        }
                    }
                }
            } else {
                // If top is auto (-1) and bottom is set (not -1), position from the bottom offset.
                if !has_top && has_bottom {
                    let bottom = style.reset_surround.bottom as f32;
                    let margin_bottom = if style.reset_surround.margin_bottom == -1 {
                        0.0
                    } else {
                        style.reset_surround.margin_bottom as f32
                    };
                    let target_y =
                        container_height - bottom - margin_bottom - child_box.rect.size.height;
                    let shift_dy = ancestor_origin.1 + target_y - child_box.rect.origin.y;
                    if shift_dy != 0.0 {
                        child_box.rect.origin.y += shift_dy;
                        for child in &mut child_box.children {
                            shift_layout_box(child, styles, 0.0, shift_dy, 1);
                        }
                    }
                }
            }

            // If both top and bottom are set, and height is auto (-1), stretch the height of the border box
            if style.reset_surround.top != -1
                && style.reset_surround.bottom != -1
                && style.reset_box.height == -1
            {
                let top_val = style.reset_surround.top as f32;
                let bottom_val = style.reset_surround.bottom as f32;
                let margin_top = crate::layout::get_px(style, "margin-top", 0.0);
                let margin_bottom = crate::layout::get_px(style, "margin-bottom", 0.0);
                let target_height =
                    (container_height - top_val - bottom_val - margin_top - margin_bottom).max(0.0);
                child_box.rect.size.height = target_height;
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

    #[test]
    fn test_absolute_position_bottom_offset() {
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
            body { display: block; width: 500px; height: 400px; }
            div {
                display: block;
                position: absolute;
                bottom: 50px;
                width: 100px;
                height: 80px;
                left: 20px;
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
        // The container height is the root_box height, which is 400.0 (plus any body margins/padding, but default margin is 8px for body usually if UA sheet is applied. Wait, is body UA stylesheet applied? In layout_document, we computed layout using parsed stylesheet. Let's look at the result).
        // Let's assert div_box properties:
        assert_eq!(div_box.rect.size.width, 100.0);
        assert_eq!(div_box.rect.size.height, 80.0);
        assert_eq!(div_box.rect.origin.x, 20.0);
        // target_y = container_height - bottom - height = container_height - 50.0 - 80.0
        // Let's assert it is correct:
        let expected_y = layout_tree.rect.size.height - 50.0 - 80.0;
        assert_eq!(div_box.rect.origin.y, expected_y);
    }

    #[test]
    fn test_relative_position_right_and_bottom_offsets() {
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
                position: relative;
                right: 30px;
                bottom: 25px;
                width: 100px;
                height: 50px;
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

        let div_box = div_box.expect("Relative div box not found in layout tree");
        // Static position would be (0.0, 0.0)
        // With right: 30px, dx = -30.0. With bottom: 25px, dy = -25.0.
        // So the offset position should be (-30.0, -25.0).
        assert_eq!(div_box.rect.origin.x, -30.0);
        assert_eq!(div_box.rect.origin.y, -25.0);
    }

    #[test]
    fn test_relative_position_rtl_both_offsets() {
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
                position: relative;
                left: 20px;
                right: 30px;
                direction: rtl;
                width: 100px;
                height: 50px;
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

        let div_box = div_box.expect("Relative div box not found");
        // Since direction is rtl and both are specified, right (30px) wins over left (20px).
        // Vertical shift is 0.0. Horizontal shift is -30.0.
        assert_eq!(div_box.rect.origin.x, -30.0);
        assert_eq!(div_box.rect.origin.y, 0.0);
    }

    #[test]
    fn test_absolute_position_nested_inside_relative() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let relative_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, relative_parent);

        let absolute_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(relative_parent, absolute_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .parent {
                display: block;
                position: relative;
                left: 50px;
                top: 40px;
                width: 400px;
                height: 300px;
            }
            .child {
                display: block;
                position: absolute;
                left: 15px;
                top: 25px;
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut child_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(absolute_child) {
                child_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let child_box = child_box.expect("Absolute child box not found");
        // In this engine (V1), absolute descendants of relative parents are positioned relative to viewport.
        // So they are at (15, 25).
        assert_eq!(child_box.rect.origin.x, 15.0);
        assert_eq!(child_box.rect.origin.y, 25.0);
    }

    #[test]
    fn test_absolute_position_rtl_both_offsets() {
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
                left: 10px;
                right: 20px;
                width: 100px;
                height: 50px;
                top: 10px;
                direction: rtl;
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

        let div_box = div_box.expect("Absolute div box not found");
        // Since both are specified and direction is rtl, right wins.
        // x = viewport_width - right - width = 800.0 - 20.0 - 100.0 = 680.0
        assert_eq!(div_box.rect.origin.x, 680.0);
    }

    #[test]
    fn test_fixed_position_ignores_positioned_ancestor() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let relative_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, relative_parent);

        let fixed_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(relative_parent, fixed_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .parent {
                display: block;
                position: relative;
                left: 100px;
                top: 100px;
                width: 400px;
                height: 300px;
            }
            .child {
                display: block;
                position: fixed;
                left: 15px;
                top: 25px;
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut child_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(fixed_child) {
                child_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let child_box = child_box.expect("Fixed child box not found");
        // Fixed child ignores relative ancestor shift and position. It should be positioned at viewport (15, 25).
        assert_eq!(child_box.rect.origin.x, 15.0);
        assert_eq!(child_box.rect.origin.y, 25.0);
    }

    #[test]
    fn test_absolute_position_shrink_to_fit() {
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

        let child = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, child);

        let text = dom.create_node(NodeData::Text("Hello".into()));
        dom.append_child(child, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                position: absolute;
                left: 10px;
                top: 10px;
                /* width is auto by default */
            }
            span {
                display: inline;
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

        let div_box = div_box.expect("Absolute div box not found");
        // Hello width should be 5 * 8 = 40px
        assert_eq!(div_box.rect.size.width, 40.0);
    }

    #[test]
    fn test_absolute_position_stretch_width() {
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
                left: 50px;
                right: 70px;
                top: 10px;
                /* width is auto */
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

        let div_box = div_box.expect("Absolute div box not found");
        // x = 50px, width = viewport_width - left - right = 800.0 - 50.0 - 70.0 = 680.0
        assert_eq!(div_box.rect.origin.x, 50.0);
        assert_eq!(div_box.rect.size.width, 680.0);
    }

    #[test]
    fn test_absolute_position_stretch_height() {
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
            body { display: block; width: 500px; height: 400px; }
            div {
                display: block;
                position: absolute;
                left: 10px;
                top: 40px;
                bottom: 60px;
                /* height is auto */
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

        let div_box = div_box.expect("Absolute div box not found");
        // y = 40px
        assert_eq!(div_box.rect.origin.y, 40.0);
        let expected_height = layout_tree.rect.size.height - 40.0 - 60.0;
        assert_eq!(div_box.rect.size.height, expected_height);
    }

    #[test]
    fn test_absolute_position_right_margin_t0902() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(crate::dom::NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(crate::dom::NodeData::Element {
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
                width: 100px;
                height: 50px;
                top: 10px;
                margin-right: 20px;
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

        let div_box = div_box.expect("Absolute div box not found");
        // viewport_width (800.0) - right (40.0) - margin_right (20.0) - width (100.0) = 640.0
        assert_eq!(div_box.rect.origin.x, 640.0);
    }

    #[test]
    fn test_absolute_position_bottom_margin_t0902() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(crate::dom::NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; height: 400px; }
            div {
                display: block;
                position: absolute;
                bottom: 50px;
                width: 100px;
                height: 80px;
                left: 20px;
                margin-bottom: 15px;
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

        let div_box = div_box.expect("Absolute div box not found");
        // container_height (400.0 usually, let's use actual layout_tree height) - bottom (50.0) - margin_bottom (15.0) - height (80.0)
        let expected_y = layout_tree.rect.size.height - 50.0 - 15.0 - 80.0;
        assert_eq!(div_box.rect.origin.y, expected_y);
    }

    #[test]
    fn test_absolute_position_horizontal_centering_auto_margins_t0902() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(crate::dom::NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(crate::dom::NodeData::Element {
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
                left: 100px;
                right: 100px;
                width: 200px;
                height: 50px;
                top: 10px;
                margin-left: auto;
                margin-right: auto;
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

        let div_box = div_box.expect("Absolute div box not found");
        // viewport_width (800.0) - left (100.0) - right (100.0) - width (200.0) = 400.0 extra space.
        // split equally -> 200.0 each margin.
        // target_x = left (100.0) + margin_left (200.0) = 300.0
        assert_eq!(div_box.rect.origin.x, 300.0);
    }

    #[test]
    fn test_absolute_position_vertical_centering_auto_margins_t0902() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(crate::dom::NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; height: 400px; }
            div {
                display: block;
                position: absolute;
                top: 50px;
                bottom: 50px;
                width: 100px;
                height: 100px;
                left: 20px;
                margin-top: auto;
                margin-bottom: auto;
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

        let div_box = div_box.expect("Absolute div box not found");
        // container_height (400.0 usually) - top (50.0) - bottom (50.0) - height (100.0) = 200.0 extra space.
        // split equally -> 100.0 each margin.
        // target_y = top (50.0) + margin_top (100.0) = 150.0
        let expected_y = 50.0 + (layout_tree.rect.size.height - 50.0 - 50.0 - 100.0) / 2.0;
        assert_eq!(div_box.rect.origin.y, expected_y);
    }

    #[test]
    fn test_nested_absolute_positioning_t0932() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(crate::dom::NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let parent = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, parent);

        let child = dom.create_node(crate::dom::NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(parent, child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 800px; }
            .parent {
                display: block;
                position: absolute;
                left: 100px;
                top: 120px;
                width: 300px;
                height: 200px;
            }
            .child {
                display: block;
                position: absolute;
                left: 20px;
                top: 30px;
                width: 50px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut parent_box = None;
        let mut child_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(parent) {
                parent_box = Some(current);
            }
            if current.node == Some(child) {
                child_box = Some(current);
            }
            for child_elem in &current.children {
                stack.push(child_elem);
            }
        }

        let parent_box = parent_box.expect("Parent box not found");
        let child_box = child_box.expect("Child box not found");

        // Parent should be placed relative to viewport
        assert_eq!(parent_box.rect.origin.x, 100.0);
        assert_eq!(parent_box.rect.origin.y, 120.0);
        assert_eq!(parent_box.rect.size.width, 300.0);
        assert_eq!(parent_box.rect.size.height, 200.0);

        // Child should be placed relative to Parent
        assert_eq!(child_box.rect.origin.x, 120.0);
        assert_eq!(child_box.rect.origin.y, 150.0);
        assert_eq!(child_box.rect.size.width, 50.0);
        assert_eq!(child_box.rect.size.height, 50.0);
    }

    #[test]
    fn test_absolute_position_nested_inside_relative_with_borders() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let relative_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, relative_parent);

        let absolute_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(relative_parent, absolute_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .parent {
                display: block;
                position: relative;
                left: 50px;
                top: 40px;
                width: 400px;
                height: 300px;
                border-left-width: 15px;
                border-top-width: 25px;
                border-right-width: 10px;
                border-bottom-width: 20px;
            }
            .child {
                display: block;
                position: absolute;
                left: 10px;
                top: 10px;
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut child_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(absolute_child) {
                child_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let child_box = child_box.expect("Absolute child box not found");
        // Bounded by the padding edge of the ancestor.
        // Parent's static origin is (0, 0) prior to relative shift in layout_absolute_and_fixed_elements,
        // and its border-left is 15px, border-top is 25px.
        // So containing block origin is (15, 25).
        // child left = 10px, top = 10px.
        // child.x should be 15 + 10 = 25.0
        // child.y should be 25 + 10 = 35.0
        assert_eq!(child_box.rect.origin.x, 25.0);
        assert_eq!(child_box.rect.origin.y, 35.0);
    }

    #[test]
    fn test_absolute_position_stretch_nested_inside_relative_with_borders() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let relative_parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, relative_parent);

        let absolute_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(relative_parent, absolute_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            .parent {
                display: block;
                position: relative;
                left: 50px;
                top: 40px;
                width: 400px;
                height: 300px;
                border-left-width: 15px;
                border-top-width: 25px;
                border-right-width: 10px;
                border-bottom-width: 20px;
            }
            .child {
                display: block;
                position: absolute;
                left: 0px;
                right: 0px;
                top: 0px;
                bottom: 0px;
                /* width/height are auto */
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let viewport_width = 800.0;
        let layout_tree = layout_document(&dom, &styles, viewport_width);

        let mut child_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(current) = stack.pop() {
            if current.node == Some(absolute_child) {
                child_box = Some(current);
                break;
            }
            for child in &current.children {
                stack.push(child);
            }
        }

        let child_box = child_box.expect("Absolute child box not found");
        // Bounded by the padding edge of the ancestor.
        // Parent content width is 400px, border-left is 15px, border-right is 10px.
        // So padding box (containing block) width is 400px (with border box width being 425px).
        // Parent content height is 300px, border-top is 25px, border-bottom is 20px.
        // So padding box (containing block) height is 300px (with border box height being 345px).
        // child left = 0, right = 0, top = 0, bottom = 0.
        // child.x should be 15.0
        // child.y should be 25.0
        // child width should be 400.0
        // child height should be 300.0
        assert_eq!(child_box.rect.origin.x, 15.0);
        assert_eq!(child_box.rect.origin.y, 25.0);
        assert_eq!(child_box.rect.size.width, 400.0);
        assert_eq!(child_box.rect.size.height, 300.0);
    }
}
