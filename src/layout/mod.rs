mod flex;
mod inline;
mod position;
mod table;

pub(crate) use position::is_absolute_or_fixed;

use crate::css::values::{CssValue, DisplayValue, LengthUnit};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::inline::{layout_inline, layout_inline_run};
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A box in the layout tree.
/// spec: S-11
pub struct LayoutBox {
    pub node: Option<NodeId>,
    pub rect: Rect,
    pub children: Vec<LayoutBox>,
    /// For inline text fragments, the exact substring this box paints (one word).
    /// None for non-text/structural boxes; painters fall back to the node's full text.
    pub text: Option<String>,
}

pub(crate) const MAX_DEPTH: usize = 500;

fn collapse_margins(m1: f32, m2: f32) -> f32 {
    if m1 >= 0.0 && m2 >= 0.0 {
        m1.max(m2)
    } else if m1 < 0.0 && m2 < 0.0 {
        m1.min(m2) // most-negative
    } else {
        // mixed sign (one positive or zero, one negative)
        let pos = m1.max(0.0).max(m2.max(0.0));
        let neg = m1.min(0.0).min(m2.min(0.0));
        pos + neg
    }
}

/// Performs block layout on the document.
/// spec: S-11
pub fn layout_document(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    viewport_width: f32,
) -> LayoutBox {
    let mut root_box = LayoutBox {
        node: Some(dom.document()),
        rect: Rect::new(0.0, 0.0, viewport_width, 0.0),
        children: Vec::new(),
        text: None,
    };

    // The document's children (usually just <html>)
    let mut cursor_y = 0.0;
    let mut prev_margin_bottom: Option<f32> = None;
    let mut last_child_box_max_y: Option<f32> = None;

    for &child in dom.children(dom.document()) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }

        let offset_y =
            if let (Some(prev_mb), Some(last_max_y)) = (prev_margin_bottom, last_child_box_max_y) {
                let child_style = styles.get(&child);
                let margin_top = child_style
                    .map(|s| get_px(s, "margin-top", 0.0))
                    .unwrap_or(0.0);
                let collapsed = collapse_margins(prev_mb, margin_top);
                last_max_y + collapsed - margin_top
            } else {
                cursor_y
            };

        if let Some(child_box) = layout_node(dom, styles, child, viewport_width, 0.0, offset_y, 0) {
            let margin_bottom = styles
                .get(&child)
                .map(|s| get_px(s, "margin-bottom", 0.0))
                .unwrap_or(0.0);

            last_child_box_max_y = Some(child_box.rect.max_y());
            prev_margin_bottom = Some(margin_bottom);

            cursor_y = child_box.rect.max_y() + margin_bottom;
            root_box.children.push(child_box);
        }
    }

    root_box.rect.size.height = cursor_y;

    // Apply absolute and fixed positioning
    // spec: S-31
    position::layout_absolute_and_fixed_elements(dom, styles, viewport_width, &mut root_box);

    // Apply relative positioning offsets
    // spec: S-31
    position::resolve_relative_positions(&mut root_box, styles, 0);

    root_box
}

/// Re-run the inline layout pass for an inline-only container against its final
/// shrink-to-fit `width`, replacing the children produced by the first pass and
/// returning the new bottom cursor. Kept as a separate, non-inlined function so
/// its locals do not enlarge `layout_node`'s stack frame on the deep all-block
/// recursion path (Windows 1 MiB stack regression guard).
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn recenter_inline_children(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    children: &mut Vec<LayoutBox>,
    width: f32,
    inline_origin_x: f32,
    cursor_y0: f32,
    depth: usize,
    text_align: &str,
    text_indent: f32,
    word_spacing: f32,
) -> f32 {
    children.clear();
    let (line_boxes, total_height) = layout_inline(
        dom,
        styles,
        node,
        width,
        inline_origin_x,
        cursor_y0,
        depth,
        text_align,
        text_indent,
        word_spacing,
    );
    children.extend(line_boxes);
    cursor_y0 + total_height
}

pub(crate) fn layout_node(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    containing_width: f32,
    offset_x: f32,
    offset_y: f32,
    depth: usize,
) -> Option<LayoutBox> {
    if depth > MAX_DEPTH {
        // TODO(spec): Report stack depth limit exceeded
        return None;
    }

    let style = styles.get(&node)?;

    // display: none -> no box
    if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "none") {
        return None;
    }

    // display: flex
    if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "flex") {
        return crate::layout::flex::layout_flex_container(
            dom,
            styles,
            node,
            containing_width,
            offset_x,
            offset_y,
            depth,
        );
    }

    // display: table
    let is_table_element = matches!(dom.data(node), Some(crate::dom::NodeData::Element { name, .. }) if name == "table");
    if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "table")
        || matches!(
            style.get("display"),
            Some(CssValue::Display(DisplayValue::Table))
        )
        || is_table_element
    {
        return crate::layout::table::layout_table_container(
            dom,
            styles,
            node,
            containing_width,
            offset_x,
            offset_y,
            depth,
        );
    }

    // Get box model values
    let margin_top = get_px(style, "margin-top", 0.0);

    let padding_left = get_px(style, "padding-left", 0.0);
    let padding_right = get_px(style, "padding-right", 0.0);
    let padding_top = get_px(style, "padding-top", 0.0);
    let padding_bottom = get_px(style, "padding-bottom", 0.0);

    let border_left = get_px(style, "border-left-width", 0.0);
    let border_right = get_px(style, "border-right-width", 0.0);
    let border_top = get_px(style, "border-top-width", 0.0);
    let border_bottom = get_px(style, "border-bottom-width", 0.0);

    // Resolve horizontal margins and width
    // TODO(spec): The <center> element should ideally be mapped to `display: block; text-align: center;`
    // in the UA stylesheet (src/engine/mod.rs). Since we are restricted to src/layout/,
    // we implement the layout-side behavior here by treating <center> as block-level (is_inline returns false)
    // and resolving its text-align as "center" by default.
    let is_inline = is_inline_level(styles, dom, node);
    let (resolved_margin_left, resolved_margin_right, mut content_width, auto_width) =
        resolve_margins_and_width(
            style,
            containing_width,
            is_inline,
            border_left,
            border_right,
            padding_left,
            padding_right,
        );
    let _ = resolved_margin_right; // Silence unused warning

    // Position of the border box
    let border_box_x = offset_x + resolved_margin_left;
    let border_box_y = offset_y + margin_top;

    let text_align = get_text_align(dom, node, style);
    let text_indent = get_px(style, "text-indent", 0.0);
    let word_spacing = get_px(style, "word-spacing", 0.0);

    let mut children = Vec::new();
    // TODO(spec): Parent-child margin collapse (collapse-through) is out of scope.
    // Also, collapse suppression by intervening padding/border, collapse-through empty blocks,
    // clear/clearance, floats, and BFC establishment via overflow are out of scope.
    let mut child_cursor_y = border_box_y + border_top + padding_top;

    let layoutable_children = get_layoutable_children(dom, styles, node);
    let has_inline = layoutable_children
        .iter()
        .any(|&c| is_inline_level(styles, dom, c));
    let has_block = layoutable_children
        .iter()
        .any(|&c| !is_inline_level(styles, dom, c));

    // Width used for the inline pass below; if shrink-to-fit later changes the
    // content width we re-run the inline pass to re-center against the final
    // width (see `recenter_inline_children`).
    let inline_pass_width = content_width;

    // Layout children
    if has_inline && !has_block {
        // If ALL children are inline, keep current behavior (single inline pass)
        let (line_boxes, total_height) = layout_inline(
            dom,
            styles,
            node,
            content_width,
            border_box_x + border_left + padding_left,
            child_cursor_y,
            depth,
            text_align,
            text_indent,
            word_spacing,
        );
        children.extend(line_boxes);
        child_cursor_y += total_height;
    } else if !has_inline {
        // If ALL children are block (or empty), keep current block behavior
        let mut prev_margin_bottom: Option<f32> = None;
        let mut last_child_box_max_y: Option<f32> = None;

        for &child in dom.children(node) {
            if is_absolute_or_fixed(styles, child) {
                continue;
            }

            let offset_y = if let (Some(prev_mb), Some(last_max_y)) =
                (prev_margin_bottom, last_child_box_max_y)
            {
                let child_style = styles.get(&child);
                let margin_top = child_style
                    .map(|s| get_px(s, "margin-top", 0.0))
                    .unwrap_or(0.0);
                let collapsed = collapse_margins(prev_mb, margin_top);
                last_max_y + collapsed - margin_top
            } else {
                child_cursor_y
            };

            if let Some(child_box) = layout_node(
                dom,
                styles,
                child,
                content_width,
                border_box_x + border_left + padding_left,
                offset_y,
                depth + 1,
            ) {
                let margin_bottom = styles
                    .get(&child)
                    .map(|s| get_px(s, "margin-bottom", 0.0))
                    .unwrap_or(0.0);

                last_child_box_max_y = Some(child_box.rect.max_y());
                prev_margin_bottom = Some(margin_bottom);

                child_cursor_y = child_box.rect.max_y() + margin_bottom;
                children.push(child_box);
            }
        }
    } else {
        // MIXED: wrap each maximal run of consecutive inline-level children in an anonymous block box
        // spec: S-anonymous-block-boxes
        // TODO(spec): same shrink-to-fit re-centering needed for mixed inline runs
        let mut prev_margin_bottom: Option<f32> = None;
        let mut last_child_box_max_y: Option<f32> = None;

        let mut i = 0;
        while i < layoutable_children.len() {
            let child = layoutable_children[i];
            if is_inline_level(styles, dom, child) {
                let mut inline_run = Vec::new();
                while i < layoutable_children.len()
                    && is_inline_level(styles, dom, layoutable_children[i])
                {
                    inline_run.push(layoutable_children[i]);
                    i += 1;
                }

                // Treat anonymous block as having margin_top = 0.0
                let start_y = if let (Some(prev_mb), Some(last_max_y)) =
                    (prev_margin_bottom, last_child_box_max_y)
                {
                    let collapsed = collapse_margins(prev_mb, 0.0);
                    last_max_y + collapsed
                } else {
                    child_cursor_y
                };

                let (line_boxes, total_height) = layout_inline_run(
                    dom,
                    styles,
                    &inline_run,
                    content_width,
                    border_box_x + border_left + padding_left,
                    start_y,
                    depth,
                    text_align,
                    text_indent,
                    word_spacing,
                );
                if !line_boxes.is_empty() {
                    let anon_box = LayoutBox {
                        node: None,
                        rect: Rect::new(
                            border_box_x + border_left + padding_left,
                            start_y,
                            content_width,
                            total_height,
                        ),
                        children: line_boxes,
                        text: None,
                    };

                    last_child_box_max_y = Some(anon_box.rect.max_y());
                    // Treat anonymous block as having margin_bottom = 0.0
                    prev_margin_bottom = Some(0.0);
                    child_cursor_y = anon_box.rect.max_y();

                    children.push(anon_box);
                }
            } else {
                let offset_y = if let (Some(prev_mb), Some(last_max_y)) =
                    (prev_margin_bottom, last_child_box_max_y)
                {
                    let child_style = styles.get(&child);
                    let margin_top = child_style
                        .map(|s| get_px(s, "margin-top", 0.0))
                        .unwrap_or(0.0);
                    let collapsed = collapse_margins(prev_mb, margin_top);
                    last_max_y + collapsed - margin_top
                } else {
                    child_cursor_y
                };

                if let Some(child_box) = layout_node(
                    dom,
                    styles,
                    child,
                    content_width,
                    border_box_x + border_left + padding_left,
                    offset_y,
                    depth + 1,
                ) {
                    let margin_bottom = styles
                        .get(&child)
                        .map(|s| get_px(s, "margin-bottom", 0.0))
                        .unwrap_or(0.0);

                    last_child_box_max_y = Some(child_box.rect.max_y());
                    prev_margin_bottom = Some(margin_bottom);

                    child_cursor_y = child_box.rect.max_y() + margin_bottom;
                    children.push(child_box);
                }
                i += 1;
            }
        }
    }

    if let Some(w) = calculate_shrink_to_fit_width(
        dom,
        styles,
        node,
        style,
        &children,
        border_box_x + border_left + padding_left,
        auto_width,
        has_block,
        depth,
    ) {
        content_width = w;
    }

    if !has_inline && has_block && content_width != inline_pass_width {
        child_cursor_y = relayout_block_children(
            dom,
            styles,
            node,
            &mut children,
            0,
            content_width,
            border_box_x,
            border_left,
            padding_left,
            border_box_y + border_top + padding_top,
            depth,
        );
    }

    if has_inline && !has_block && content_width != inline_pass_width {
        // shrink-to-fit changed the content width: re-run the inline pass so the
        // content re-centers against the final box width. Delegated to a
        // non-inlined helper so this branch's locals do not enlarge
        // `layout_node`'s stack frame on the deep all-block recursion path
        // (Windows 1 MiB stack regression guard: test_deep_tree_recursion_cap).
        child_cursor_y = recenter_inline_children(
            dom,
            styles,
            node,
            &mut children,
            content_width,
            border_box_x + border_left + padding_left,
            border_box_y + border_top + padding_top,
            depth,
            text_align,
            text_indent,
            word_spacing,
        );
    }

    // Calculate height
    let mut content_height = child_cursor_y - (border_box_y + border_top + padding_top);
    if get_form_control_button_label(dom, node).is_some() && children.is_empty() {
        content_height = crate::font::BitmapFont::builtin().line_height() as f32;
    }
    if let Some(crate::dom::NodeData::Element { name, .. }) = dom.data(node)
        && name.eq_ignore_ascii_case("br")
    {
        content_height = crate::font::BitmapFont::builtin().line_height() as f32;
    }

    // Apply aspect-ratio if height is not explicitly set (i.e. is auto or absent)
    // TODO(spec): The inverse direction (width-from-height using aspect-ratio) is ambiguous/not implemented in our simplified model.
    let height_is_auto_or_absent = match style.get("height") {
        None => true,
        Some(CssValue::Keyword(kw)) if kw == "auto" => true,
        Some(CssValue::Number(n)) if *n != 0.0 => true,
        _ => false,
    };
    if height_is_auto_or_absent && let Some(r) = get_aspect_ratio(style) {
        let border_box_width =
            content_width + padding_left + padding_right + border_left + border_right;
        let derived_border_box_height = border_box_width / r;
        content_height =
            (derived_border_box_height - padding_top - padding_bottom - border_top - border_bottom)
                .max(0.0);
    }

    // TODO(spec): min/max-height clamp box-sizing interaction follows the existing height treatment; percentage min/max sizes are not resolved.
    let border_box_height = clamp_height(style, get_px(style, "height", content_height))
        + padding_top
        + padding_bottom
        + border_top
        + border_bottom;

    let mut is_li = false;
    if let Some(crate::dom::NodeData::Element { name, .. }) = dom.data(node)
        && name == "li"
    {
        is_li = true;
    }

    if is_li
        && let Some(list_node) = find_nearest_list_ancestor_node(dom, node)
        && let Some(crate::dom::NodeData::Element {
            name: list_name, ..
        }) = dom.data(list_node)
    {
        let list_style_type = style.get("list-style-type");
        let suppress_marker =
            matches!(list_style_type, Some(CssValue::Keyword(val)) if val == "none");

        // TODO(spec): support other list-style-type values like circle, square
        // TODO(spec): support list-style-position: inside
        // TODO(spec): support list-style-image
        // TODO(spec): support numbering restart edge cases
        // TODO(spec): support nested-list interactions beyond nearest ancestor
        // TODO(spec): support exact browser baseline/metrics of the marker

        if !suppress_marker {
            let (first_line_y, first_line_h) = find_first_line_rect_and_height(
                &children,
                border_box_y + border_top + padding_top,
                crate::font::BitmapFont::builtin().line_height() as f32,
            );
            let first_line_center_y = first_line_y + first_line_h / 2.0;

            let fs = get_font_size(style);

            if list_name == "ul" {
                let side = 0.4 * fs;
                let marker_x = border_box_x - 20.0 - side / 2.0;
                let marker_y = first_line_center_y - side / 2.0;

                // TODO(spec): disc marker needs a paint-side fill primitive — layout cannot emit a fill for a node without background-color
                // As a fallback to actually paint a visible bullet, we render a Unicode bullet glyph per list-style-type.
                let bullet = match list_style_type {
                    Some(CssValue::Keyword(kw)) => match kw.as_str() {
                        "circle" => "\u{25E6}",
                        "square" => "\u{25AA}",
                        _ => "\u{2022}",
                    },
                    _ => "\u{2022}",
                };

                let text_node = find_first_text_node(dom, node);
                let marker_text = text_node.map(|_| bullet.to_string());

                let marker_box = LayoutBox {
                    node: text_node.or(Some(node)),
                    rect: Rect::new(marker_x, marker_y, side, side),
                    children: Vec::new(),
                    text: marker_text,
                };
                children.push(marker_box);
            } else if list_name == "ol" {
                let index = get_li_decimal_index(dom, node, list_node);
                let formatted = match list_style_type {
                    Some(CssValue::Keyword(kw)) => match kw.as_str() {
                        "lower-alpha" | "lower-latin" => {
                            if index >= 1 {
                                to_alpha(index as usize, false)
                            } else {
                                index.to_string()
                            }
                        }
                        "upper-alpha" | "upper-latin" => {
                            if index >= 1 {
                                to_alpha(index as usize, true)
                            } else {
                                index.to_string()
                            }
                        }
                        "lower-roman" => {
                            if index >= 1 {
                                to_roman(index as usize, false)
                            } else {
                                index.to_string()
                            }
                        }
                        "upper-roman" => {
                            if index >= 1 {
                                to_roman(index as usize, true)
                            } else {
                                index.to_string()
                            }
                        }
                        _ => index.to_string(),
                    },
                    _ => index.to_string(),
                };
                let marker_text = format!("{formatted}.");

                let marker_width = 8.0 * marker_text.len() as f32;
                let marker_height = first_line_h;

                let content_start_x = border_box_x + border_left + padding_left;
                let marker_x = content_start_x - marker_width - 8.0;
                let marker_y = first_line_y;

                let text_node = find_first_text_node(dom, node);
                let marker_box = LayoutBox {
                    node: text_node,
                    rect: Rect::new(marker_x, marker_y, marker_width, marker_height),
                    children: Vec::new(),
                    text: Some(marker_text),
                };
                children.push(marker_box);
            }
        }
    }

    Some(LayoutBox {
        node: Some(node),
        rect: Rect::new(
            border_box_x,
            border_box_y,
            content_width + padding_left + padding_right + border_left + border_right,
            border_box_height,
        ),
        children,
        text: None,
    })
}

pub(crate) fn get_layoutable_children(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
) -> Vec<NodeId> {
    let mut result = Vec::new();
    for &child in dom.children(node) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }
        if let Some(data) = dom.data(child) {
            match data {
                NodeData::Text(_) => {
                    result.push(child);
                }
                NodeData::Element { .. } => {
                    if let Some(style) = styles.get(&child) {
                        if matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "none")
                        {
                            continue;
                        }
                        result.push(child);
                    }
                }
                _ => {}
            }
        }
    }
    result
}

fn get_form_control_button_label(dom: &Dom, node: NodeId) -> Option<String> {
    if let Some(NodeData::Element { name, .. }) = dom.data(node) {
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn calculate_shrink_to_fit_width(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    style: &ComputedStyle,
    children: &[LayoutBox],
    content_start_x: f32,
    auto_width: f32,
    has_block_children: bool,
    depth: usize,
) -> Option<f32> {
    let is_inline_blk = matches!(style.get("display"), Some(CssValue::Keyword(kw)) if kw == "inline-block")
        || matches!(
            style.get("display"),
            Some(CssValue::Display(DisplayValue::InlineBlock))
        );
    let has_width = matches!(style.get("width"), Some(CssValue::Length(_, _)))
        || matches!(style.get("width"), Some(CssValue::Number(n)) if *n == 0.0);
    if is_inline_blk && !has_width {
        if has_block_children {
            let candidate = max_content_width(dom, styles, node, depth);
            Some(candidate.min(auto_width.max(0.0)))
        } else {
            let mut max_child_right = 0.0_f32;
            if let Some(label) = get_form_control_button_label(dom, node) {
                max_child_right = crate::font::BitmapFont::builtin().measure(&label) as f32;
            }
            for child in children {
                let child_right = child.rect.max_x() - content_start_x;
                if child_right > max_child_right {
                    max_child_right = child_right;
                }
            }
            Some(max_child_right.min(auto_width.max(0.0)))
        }
    } else {
        None
    }
}

#[inline(never)]
fn max_content_width(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    depth: usize,
) -> f32 {
    if depth > MAX_DEPTH {
        return 0.0;
    }

    if let Some(style) = styles.get(&node)
        && let Some(width_val) = style.get("width")
    {
        match width_val {
            CssValue::Length(v, LengthUnit::Px) => return *v,
            CssValue::Number(n) if *n == 0.0 => return 0.0,
            _ => {}
        }
    }

    if let Some(label) = get_form_control_button_label(dom, node) {
        return crate::font::BitmapFont::builtin().measure(&label) as f32;
    }

    if let Some(NodeData::Text(text)) = dom.data(node) {
        return crate::font::BitmapFont::builtin().measure(text) as f32;
    }

    let children = get_layoutable_children(dom, styles, node);
    if children.is_empty() {
        return 0.0;
    }

    let mut has_block_child = false;
    let mut children_contributions = Vec::with_capacity(children.len());

    for &child in &children {
        let child_content_width = max_content_width(dom, styles, child, depth + 1);
        let mut child_h_padding_border = 0.0;
        if let Some(child_style) = styles.get(&child) {
            child_h_padding_border += get_px(child_style, "padding-left", 0.0);
            child_h_padding_border += get_px(child_style, "padding-right", 0.0);
            child_h_padding_border += get_px(child_style, "border-left-width", 0.0);
            child_h_padding_border += get_px(child_style, "border-right-width", 0.0);
            if !is_inline_level(styles, dom, child) {
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

#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn relayout_block_children(
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
    node: NodeId,
    children: &mut Vec<LayoutBox>,
    prev_len: usize,
    content_width: f32,
    border_box_x: f32,
    border_left: f32,
    padding_left: f32,
    mut child_cursor_y: f32,
    depth: usize,
) -> f32 {
    children.truncate(prev_len);
    let mut prev_margin_bottom: Option<f32> = None;
    let mut last_child_box_max_y: Option<f32> = None;

    for &child in dom.children(node) {
        if is_absolute_or_fixed(styles, child) {
            continue;
        }

        let offset_y =
            if let (Some(prev_mb), Some(last_max_y)) = (prev_margin_bottom, last_child_box_max_y) {
                let child_style = styles.get(&child);
                let margin_top = child_style
                    .map(|s| get_px(s, "margin-top", 0.0))
                    .unwrap_or(0.0);
                let collapsed = collapse_margins(prev_mb, margin_top);
                last_max_y + collapsed - margin_top
            } else {
                child_cursor_y
            };

        if let Some(child_box) = layout_node(
            dom,
            styles,
            child,
            content_width,
            border_box_x + border_left + padding_left,
            offset_y,
            depth + 1,
        ) {
            let margin_bottom = styles
                .get(&child)
                .map(|s| get_px(s, "margin-bottom", 0.0))
                .unwrap_or(0.0);

            last_child_box_max_y = Some(child_box.rect.max_y());
            prev_margin_bottom = Some(margin_bottom);

            child_cursor_y = child_box.rect.max_y() + margin_bottom;
            children.push(child_box);
        }
    }

    child_cursor_y
}

fn is_inline_level(styles: &HashMap<NodeId, ComputedStyle>, dom: &Dom, child: NodeId) -> bool {
    if let Some(data) = dom.data(child) {
        match data {
            NodeData::Text(_) => true,
            NodeData::Element { name, .. } => {
                if name == "center" {
                    return false;
                }
                if name.eq_ignore_ascii_case("br") {
                    if let Some(style) = styles.get(&child) {
                        let disp = style.get("display");
                        if matches!(disp, Some(CssValue::Keyword(kw)) if kw == "block" || kw == "flex" || kw == "table" || kw == "none")
                        {
                            return false;
                        }
                    }
                    return true;
                }
                if let Some(style) = styles.get(&child) {
                    let disp = style.get("display");
                    matches!(disp, Some(CssValue::Keyword(kw)) if kw == "inline" || kw == "inline-block")
                        || matches!(
                            disp,
                            Some(CssValue::Display(DisplayValue::Inline))
                                | Some(CssValue::Display(DisplayValue::InlineBlock))
                        )
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

pub(crate) fn get_px(style: &ComputedStyle, prop: &str, default: f32) -> f32 {
    match style.get(prop) {
        Some(CssValue::Length(v, LengthUnit::Px)) => *v,
        Some(CssValue::Number(n)) if *n == 0.0 => 0.0,
        _ => default,
    }
}

/// Performs hit-testing on the layout tree.
///
/// Pre-order traversal; returns the NodeId of the deepest box whose rect contains (x, y);
/// among siblings, later (painted-on-top) wins.
/// Boxes with no `node` (None) are skipped for the result but their children are still tested.
/// Bounded to prevent stack overflow.
///
/// spec: S-36
pub fn hit_test(root: &LayoutBox, x: f32, y: f32) -> Option<NodeId> {
    let mut best_node = None;
    hit_test_impl(root, x, y, 0, &mut best_node);
    best_node
}

fn hit_test_impl(box_: &LayoutBox, x: f32, y: f32, depth: usize, best_node: &mut Option<NodeId>) {
    if depth > MAX_DEPTH {
        // TODO(spec): Report stack depth limit exceeded in hit_test
        return;
    }

    if box_.rect.contains(crate::geom::Point { x, y }) && box_.node.is_some() {
        *best_node = box_.node;
    }

    for child in &box_.children {
        hit_test_impl(child, x, y, depth + 1, best_node);
    }
}

fn clamp_width(style: &ComputedStyle, mut width: f32, containing_width: f32) -> f32 {
    if let Some(max_val) = style.get("max-width") {
        match max_val {
            CssValue::Length(v, LengthUnit::Px) => {
                if width > *v {
                    width = *v;
                }
            }
            CssValue::Number(n) if *n == 0.0 => {
                if width > 0.0 {
                    width = 0.0;
                }
            }
            CssValue::Length(p, LengthUnit::Percent) => {
                let max_width = containing_width * p / 100.0;
                if width > max_width {
                    width = max_width;
                }
            }
            _ => {}
        }
    }
    if let Some(min_val) = style.get("min-width") {
        match min_val {
            CssValue::Length(v, LengthUnit::Px) => {
                if width < *v {
                    width = *v;
                }
            }
            CssValue::Number(n) if *n == 0.0 => {
                if width < 0.0 {
                    width = 0.0;
                }
            }
            CssValue::Length(p, LengthUnit::Percent) => {
                let min_width = containing_width * p / 100.0;
                if width < min_width {
                    width = min_width;
                }
            }
            _ => {}
        }
    }
    width.max(0.0)
}

fn get_aspect_ratio(style: &ComputedStyle) -> Option<f32> {
    if let Some(val) = style.get("aspect-ratio") {
        match val {
            CssValue::Number(r) => {
                if *r > 0.0 {
                    return Some(*r);
                }
            }
            CssValue::Keyword(kw) => {
                if kw == "auto" {
                    return None;
                }
                if let Some(pos) = kw.find('/') {
                    let w_str = &kw[..pos];
                    let h_str = &kw[pos + 1..];
                    if let (Ok(w), Ok(h)) =
                        (w_str.trim().parse::<f32>(), h_str.trim().parse::<f32>())
                        && w > 0.0
                        && h > 0.0
                    {
                        return Some(w / h);
                    }
                } else if let Ok(r) = kw.trim().parse::<f32>()
                    && r > 0.0
                {
                    return Some(r);
                }
            }
            CssValue::Multiple(vals) if vals.len() == 3 => {
                if let (
                    Some(CssValue::Number(w)),
                    Some(CssValue::Keyword(op)),
                    Some(CssValue::Number(h)),
                ) = (vals.first(), vals.get(1), vals.get(2))
                    && op == "/"
                    && *w > 0.0
                    && *h > 0.0
                {
                    return Some(*w / *h);
                }
            }
            _ => {}
        }
    }
    None
}

fn clamp_height(style: &ComputedStyle, mut height: f32) -> f32 {
    let has_max_height = matches!(
        style.get("max-height"),
        Some(CssValue::Length(_, LengthUnit::Px))
    ) || matches!(style.get("max-height"), Some(CssValue::Number(n)) if *n == 0.0);
    if has_max_height {
        let max_height = get_px(style, "max-height", 0.0);
        if height > max_height {
            height = max_height;
        }
    }
    let has_min_height = matches!(
        style.get("min-height"),
        Some(CssValue::Length(_, LengthUnit::Px))
    ) || matches!(style.get("min-height"), Some(CssValue::Number(n)) if *n == 0.0);
    if has_min_height {
        let min_height = get_px(style, "min-height", 0.0);
        if height < min_height {
            height = min_height;
        }
    }
    height.max(0.0)
}

pub(crate) fn resolve_margins_and_width(
    style: &ComputedStyle,
    containing_width: f32,
    is_inline: bool,
    border_left: f32,
    border_right: f32,
    padding_left: f32,
    padding_right: f32,
) -> (f32, f32, f32, f32) {
    let margin_left_is_auto =
        matches!(style.get("margin-left"), Some(CssValue::Keyword(kw)) if kw == "auto");
    let margin_right_is_auto =
        matches!(style.get("margin-right"), Some(CssValue::Keyword(kw)) if kw == "auto");

    let mut resolved_margin_left = get_px(style, "margin-left", 0.0);
    let mut resolved_margin_right = get_px(style, "margin-right", 0.0);
    let content_width;

    let auto_width = containing_width
        - (if margin_left_is_auto {
            0.0
        } else {
            resolved_margin_left
        })
        - (if margin_right_is_auto {
            0.0
        } else {
            resolved_margin_right
        })
        - border_left
        - border_right
        - padding_left
        - padding_right;

    if !is_inline {
        let has_definite_width = matches!(style.get("width"), Some(CssValue::Length(_, _)))
            || matches!(style.get("width"), Some(CssValue::Number(n)) if *n == 0.0);

        if !has_definite_width {
            // width is auto. any auto margins are treated as 0.
            if margin_left_is_auto {
                resolved_margin_left = 0.0;
            }
            if margin_right_is_auto {
                resolved_margin_right = 0.0;
            }
            let raw_width = (containing_width
                - resolved_margin_left
                - resolved_margin_right
                - border_left
                - border_right
                - padding_left
                - padding_right)
                .max(0.0);
            content_width = clamp_width(style, raw_width, containing_width);
        } else {
            // width is definite
            let raw_width = get_px(style, "width", 0.0);
            content_width = clamp_width(style, raw_width, containing_width);
            let total_non_margin_width =
                content_width + border_left + border_right + padding_left + padding_right;

            let base_margin_left = if margin_left_is_auto {
                0.0
            } else {
                resolved_margin_left
            };
            let base_margin_right = if margin_right_is_auto {
                0.0
            } else {
                resolved_margin_right
            };

            if total_non_margin_width + base_margin_left + base_margin_right > containing_width {
                // Over-constrained or negative space with auto margins -> treat auto as 0
                resolved_margin_left = base_margin_left;
                resolved_margin_right = base_margin_right;
                if !margin_left_is_auto && !margin_right_is_auto {
                    resolved_margin_right =
                        containing_width - total_non_margin_width - resolved_margin_left;
                }
            } else {
                // There is positive remaining space
                if margin_left_is_auto && margin_right_is_auto {
                    let extra_space = containing_width - total_non_margin_width;
                    resolved_margin_left = extra_space / 2.0;
                    resolved_margin_right = extra_space / 2.0;
                } else if margin_left_is_auto {
                    resolved_margin_left =
                        containing_width - total_non_margin_width - base_margin_right;
                    resolved_margin_right = base_margin_right;
                } else if margin_right_is_auto {
                    resolved_margin_left = base_margin_left;
                    resolved_margin_right =
                        containing_width - total_non_margin_width - base_margin_left;
                } else {
                    // Over-constrained: adjust margin_right
                    resolved_margin_left = base_margin_left;
                    resolved_margin_right =
                        containing_width - total_non_margin_width - base_margin_left;
                }
            }
        }
    } else {
        // For inline-level, auto margins are treated as 0
        resolved_margin_left = if margin_left_is_auto {
            0.0
        } else {
            resolved_margin_left
        };
        resolved_margin_right = if margin_right_is_auto {
            0.0
        } else {
            resolved_margin_right
        };
        content_width = get_px(style, "width", auto_width.max(0.0));
    }

    (
        resolved_margin_left,
        resolved_margin_right,
        content_width,
        auto_width,
    )
}

fn get_text_align(dom: &Dom, node: NodeId, style: &ComputedStyle) -> &'static str {
    let is_center_element = matches!(
        dom.data(node),
        Some(NodeData::Element { name, .. }) if name == "center"
    );
    if is_center_element {
        "center"
    } else {
        style
            .get("text-align")
            .and_then(|val| match val {
                CssValue::Keyword(kw) => {
                    if kw == "center" {
                        Some("center")
                    } else if kw == "right" {
                        Some("right")
                    } else if kw == "justify" {
                        Some("justify")
                    } else {
                        Some("left")
                    }
                }
                _ => None,
            })
            .unwrap_or("left")
    }
}

fn find_nearest_list_ancestor_node(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let mut current = dom.parent(node);
    let mut depth = 0;
    while let Some(curr_id) = current {
        if depth > MAX_DEPTH {
            break;
        }
        if let Some(NodeData::Element { name, .. }) = dom.data(curr_id)
            && (name == "ul" || name == "ol")
        {
            return Some(curr_id);
        }
        current = dom.parent(curr_id);
        depth += 1;
    }
    None
}

fn find_first_line_rect_and_height(
    children: &[LayoutBox],
    default_y: f32,
    default_h: f32,
) -> (f32, f32) {
    if children.is_empty() {
        return (default_y, default_h);
    }
    let mut current = &children[0];
    while !current.children.is_empty() {
        if current.node.is_none() {
            return (current.rect.origin.y, current.rect.size.height);
        }
        current = &current.children[0];
    }
    (current.rect.origin.y, current.rect.size.height)
}

fn get_font_size(style: &ComputedStyle) -> f32 {
    match style.get("font-size") {
        Some(CssValue::Length(px, LengthUnit::Px)) => *px,
        _ => 16.0,
    }
}

fn to_alpha(mut n: usize, upper: bool) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut result = String::new();
    let base_char = if upper { b'A' } else { b'a' };
    while n > 0 {
        n -= 1;
        let rem = n % 26;
        let c = (base_char + rem as u8) as char;
        result.push(c);
        n /= 26;
    }
    result.chars().rev().collect()
}

fn to_roman(mut n: usize, upper: bool) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut result = String::new();
    let mappings = if upper {
        &[
            (1000, "M"),
            (900, "CM"),
            (500, "D"),
            (400, "CD"),
            (100, "C"),
            (90, "XC"),
            (50, "L"),
            (40, "XL"),
            (10, "X"),
            (9, "IX"),
            (5, "V"),
            (4, "IV"),
            (1, "I"),
        ]
    } else {
        &[
            (1000, "m"),
            (900, "cm"),
            (500, "d"),
            (400, "cd"),
            (100, "c"),
            (90, "xc"),
            (50, "l"),
            (40, "xl"),
            (10, "x"),
            (9, "ix"),
            (5, "v"),
            (4, "iv"),
            (1, "i"),
        ]
    };

    for &(val, sym) in mappings {
        while n >= val {
            result.push_str(sym);
            n -= val;
        }
    }
    result
}

// spec: https://html.spec.whatwg.org/multipage/grouping-content.html#the-ol-element
fn get_li_decimal_index(dom: &Dom, li_node: NodeId, list_node: NodeId) -> i64 {
    let mut lis = Vec::new();
    find_li_descendants(dom, list_node, list_node, &mut lis);

    let reversed = dom.get_attribute(list_node, "reversed").is_some();
    let start_val = dom
        .get_attribute(list_node, "start")
        .map(|s| s.trim())
        .and_then(|s| s.parse::<i64>().ok());

    let start_value = if let Some(start) = start_val {
        start
    } else if reversed {
        lis.len() as i64
    } else {
        1
    };

    let mut current = start_value;
    for &child in &lis {
        if let Some(val_str) = dom.get_attribute(child, "value")
            && let Ok(val) = val_str.trim().parse::<i64>()
        {
            current = val;
        }
        if child == li_node {
            return current;
        }
        current += if reversed { -1 } else { 1 };
    }

    1
}

fn find_li_descendants(dom: &Dom, current: NodeId, list_node: NodeId, lis: &mut Vec<NodeId>) {
    for &child in dom.children(current) {
        if let Some(NodeData::Element { name, .. }) = dom.data(child) {
            if name == "li" {
                if find_nearest_list_ancestor_node(dom, child) == Some(list_node) {
                    lis.push(child);
                }
            } else if name != "ul" && name != "ol" {
                find_li_descendants(dom, child, list_node, lis);
            }
        } else {
            find_li_descendants(dom, child, list_node, lis);
        }
    }
}

fn find_first_text_node(dom: &Dom, node: NodeId) -> Option<NodeId> {
    if let Some(NodeData::Text(_)) = dom.data(node) {
        return Some(node);
    }
    for &child in dom.children(node) {
        if let Some(found) = find_first_text_node(dom, child) {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::NodeData;
    use crate::style::compute_styles;

    const EPSILON: f32 = 0.001;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn test_text_align_center() {
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

        let text = dom.create_node(NodeData::Text("Hello".into())); // 5 characters = 40px
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; text-align: center; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];
        let line_box = &div_box.children[0];
        let word_box = &line_box.children[0];

        // Containing block width = 500px. Text width = 40px.
        // Under text-align: center, the remaining space of 460px is halved.
        // Therefore, the word_box should be shifted to x = 230.0px.
        assert!(approx_eq(word_box.rect.origin.x, 230.0));
    }

    #[test]
    fn test_text_align_justify_via_layout_document() {
        // End-to-end guard: `text-align: justify` must survive get_text_align()
        // and reach the inline justification logic through the shipping layout path.
        // (A unit test that calls layout_inline_run directly would pass even if
        // get_text_align dropped "justify" -> "left"; this one exercises the plumbing.)
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

        // Two short words on line 1 ("aa bb"), then a word too wide to share the
        // line, forcing "aa bb" to be a non-last line with large slack to justify.
        let text = dom.create_node(NodeData::Text(
            "aa bb xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".into(),
        ));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 200px; }
            div { display: block; text-align: justify; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 200.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];
        let first_line = &div_box.children[0];

        // Line 1 must hold exactly the two short words.
        assert_eq!(first_line.children.len(), 2);
        let w1 = &first_line.children[0];
        let w2 = &first_line.children[1];

        // First word stays at the left edge.
        assert!(approx_eq(w1.rect.origin.x, 0.0));
        // Justify must push the second word's right edge out to the content width
        // (200px). Without the get_text_align plumbing this stays clustered left.
        let right_edge = w2.rect.origin.x + w2.rect.size.width;
        assert!(
            (right_edge - 200.0).abs() < 1.0,
            "justify did not stretch the line: right_edge={right_edge}"
        );
    }

    #[test]
    fn test_list_item_markers() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // 1. UL List
        let ul = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![],
        });
        dom.append_child(body, ul);

        let li_a = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul, li_a);
        let text_a = dom.create_node(NodeData::Text("a".into()));
        dom.append_child(li_a, text_a);

        let li_b = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul, li_b);
        let text_b = dom.create_node(NodeData::Text("b".into()));
        dom.append_child(li_b, text_b);

        // 2. OL List
        let ol = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![],
        });
        dom.append_child(body, ol);

        let li_x = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol, li_x);
        let text_x = dom.create_node(NodeData::Text("x".into()));
        dom.append_child(li_x, text_x);

        let li_y = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol, li_y);
        let text_y = dom.create_node(NodeData::Text("y".into()));
        dom.append_child(li_y, text_y);

        // 3. UL List with list-style-type: none
        let ul_none = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("style".into(), "list-style-type: none;".into())],
        });
        dom.append_child(body, ul_none);

        let li_none = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_none, li_none);
        let text_none = dom.create_node(NodeData::Text("none".into()));
        dom.append_child(li_none, text_none);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ul, ol { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // Body has three children: ul, ol, ul_none
        assert_eq!(body_box.children.len(), 3);

        let ul_box = &body_box.children[0];
        let ol_box = &body_box.children[1];
        let ul_none_box = &body_box.children[2];

        // --- 1. Verify UL list (disc markers) ---
        // ul_box has 2 children (li_a, li_b)
        assert_eq!(ul_box.children.len(), 2);
        let li_a_box = &ul_box.children[0];
        let li_b_box = &ul_box.children[1];

        // li starts at x = 40.0 (ul's padding-left)
        assert!(approx_eq(li_a_box.rect.origin.x, 40.0));
        assert!(approx_eq(li_b_box.rect.origin.x, 40.0));

        // li_a_box should have 2 children: the text line box, and the disc marker box!
        assert_eq!(li_a_box.children.len(), 2);
        let _li_a_line = &li_a_box.children[0];
        let li_a_marker = &li_a_box.children[1];

        // Marker box is left of the content (in padding area)
        // Its center x should be around 20.0 (since li starts at 40.0, and marker center x is border_box_x - 20px)
        let marker_center_x = li_a_marker.rect.origin.x + li_a_marker.rect.size.width / 2.0;
        assert!(approx_eq(marker_center_x, 20.0));

        // Size is 0.4em of font-size 16px = 6.4px
        assert!(approx_eq(li_a_marker.rect.size.width, 6.4));
        assert!(approx_eq(li_a_marker.rect.size.height, 6.4));
        assert_eq!(li_a_marker.text.as_deref(), Some("\u{2022}"));

        // li_b_box also has 2 children
        assert_eq!(li_b_box.children.len(), 2);
        let li_b_marker = &li_b_box.children[1];
        let marker_b_center_x = li_b_marker.rect.origin.x + li_b_marker.rect.size.width / 2.0;
        assert!(approx_eq(marker_b_center_x, 20.0));

        // --- 2. Verify OL list (decimal markers) ---
        // ol_box has 2 children (li_x, li_y)
        assert_eq!(ol_box.children.len(), 2);
        let li_x_box = &ol_box.children[0];
        let li_y_box = &ol_box.children[1];

        // Each has 2 children: line box, and marker box
        assert_eq!(li_x_box.children.len(), 2);
        assert_eq!(li_y_box.children.len(), 2);

        let li_x_marker = &li_x_box.children[1];
        let li_y_marker = &li_y_box.children[1];

        // Decimal markers should be "1." and "2."
        assert_eq!(li_x_marker.text.as_deref(), Some("1."));
        assert_eq!(li_y_marker.text.as_deref(), Some("2."));

        // Height of marker should match line height
        assert!(approx_eq(
            li_x_marker.rect.size.height,
            li_x_box.children[0].rect.size.height
        ));

        // --- 3. Verify list-style-type: none ---
        // ul_none_box has 1 child (li_none)
        assert_eq!(ul_none_box.children.len(), 1);
        let li_none_box = &ul_none_box.children[0];

        // Since list-style-type is none, there should be NO marker box!
        // Only 1 child (the text line box) should exist
        assert_eq!(li_none_box.children.len(), 1);
    }

    #[test]
    fn test_to_alpha() {
        assert_eq!(to_alpha(1, false), "a");
        assert_eq!(to_alpha(26, false), "z");
        assert_eq!(to_alpha(27, false), "aa");
        assert_eq!(to_alpha(1, true), "A");
        assert_eq!(to_alpha(26, true), "Z");
        assert_eq!(to_alpha(27, true), "AA");
        assert_eq!(to_alpha(52, false), "az");
        assert_eq!(to_alpha(53, false), "ba");
        assert_eq!(to_alpha(0, false), "0");
    }

    #[test]
    fn test_to_roman() {
        assert_eq!(to_roman(1, false), "i");
        assert_eq!(to_roman(4, false), "iv");
        assert_eq!(to_roman(9, false), "ix");
        assert_eq!(to_roman(40, false), "xl");
        assert_eq!(to_roman(90, false), "xc");
        assert_eq!(to_roman(400, false), "cd");
        assert_eq!(to_roman(900, false), "cm");
        assert_eq!(to_roman(1990, true), "MCMXC");
        assert_eq!(to_roman(0, false), "0");
        assert_eq!(to_roman(3, true), "III");
    }

    #[test]
    fn test_ordered_list_marker_styles() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // OL with lower-alpha
        let ol_alpha = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![("style".into(), "list-style-type: lower-alpha;".into())],
        });
        dom.append_child(body, ol_alpha);

        let li_a1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_alpha, li_a1);
        let text_a1 = dom.create_node(NodeData::Text("a1".into()));
        dom.append_child(li_a1, text_a1);

        let li_a2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_alpha, li_a2);
        let text_a2 = dom.create_node(NodeData::Text("a2".into()));
        dom.append_child(li_a2, text_a2);

        // OL with upper-roman
        let ol_roman = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![("style".into(), "list-style-type: upper-roman;".into())],
        });
        dom.append_child(body, ol_roman);

        let li_r1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_roman, li_r1);
        let text_r1 = dom.create_node(NodeData::Text("r1".into()));
        dom.append_child(li_r1, text_r1);

        let li_r2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_roman, li_r2);
        let text_r2 = dom.create_node(NodeData::Text("r2".into()));
        dom.append_child(li_r2, text_r2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ol { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let ol_alpha_box = &body_box.children[0];
        let ol_roman_box = &body_box.children[1];

        // Verify alpha markers
        assert_eq!(ol_alpha_box.children.len(), 2);
        let li_a1_marker = &ol_alpha_box.children[0].children[1];
        let li_a2_marker = &ol_alpha_box.children[1].children[1];
        assert_eq!(li_a1_marker.text.as_deref(), Some("a."));
        assert_eq!(li_a2_marker.text.as_deref(), Some("b."));

        // Verify roman markers
        assert_eq!(ol_roman_box.children.len(), 2);
        let li_r1_marker = &ol_roman_box.children[0].children[1];
        let li_r2_marker = &ol_roman_box.children[1].children[1];
        assert_eq!(li_r1_marker.text.as_deref(), Some("I."));
        assert_eq!(li_r2_marker.text.as_deref(), Some("II."));
    }

    #[test]
    fn test_unordered_list_marker_styles() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // UL with list-style-type: disc
        let ul_disc = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("style".into(), "list-style-type: disc;".into())],
        });
        dom.append_child(body, ul_disc);

        let li_disc = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_disc, li_disc);
        let text_disc = dom.create_node(NodeData::Text("disc".into()));
        dom.append_child(li_disc, text_disc);

        // UL with list-style-type: circle
        let ul_circle = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("style".into(), "list-style-type: circle;".into())],
        });
        dom.append_child(body, ul_circle);

        let li_circle = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_circle, li_circle);
        let text_circle = dom.create_node(NodeData::Text("circle".into()));
        dom.append_child(li_circle, text_circle);

        // UL with list-style-type: square
        let ul_square = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("style".into(), "list-style-type: square;".into())],
        });
        dom.append_child(body, ul_square);

        let li_square = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_square, li_square);
        let text_square = dom.create_node(NodeData::Text("square".into()));
        dom.append_child(li_square, text_square);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ul, ol { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 3);
        let ul_disc_box = &body_box.children[0];
        let ul_circle_box = &body_box.children[1];
        let ul_square_box = &body_box.children[2];

        // Verify disc marker
        assert_eq!(ul_disc_box.children.len(), 1);
        let marker_disc = &ul_disc_box.children[0].children[1];
        assert_eq!(marker_disc.text.as_deref(), Some("\u{2022}"));

        // Verify circle marker
        assert_eq!(ul_circle_box.children.len(), 1);
        let marker_circle = &ul_circle_box.children[0].children[1];
        assert_eq!(marker_circle.text.as_deref(), Some("\u{25E6}"));

        // Verify square marker
        assert_eq!(ul_square_box.children.len(), 1);
        let marker_square = &ul_square_box.children[0].children[1];
        assert_eq!(marker_square.text.as_deref(), Some("\u{25AA}"));
    }

    #[test]
    fn test_ordered_list_numbering_attributes() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // 1. <ol start="5"> with 3 items
        let ol_start = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![("start".into(), "5".into())],
        });
        dom.append_child(body, ol_start);

        let li_s1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_start, li_s1);
        let text_s1 = dom.create_node(NodeData::Text("s1".into()));
        dom.append_child(li_s1, text_s1);

        let li_s2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_start, li_s2);
        let text_s2 = dom.create_node(NodeData::Text("s2".into()));
        dom.append_child(li_s2, text_s2);

        let li_s3 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_start, li_s3);
        let text_s3 = dom.create_node(NodeData::Text("s3".into()));
        dom.append_child(li_s3, text_s3);

        // 2. <ol reversed> with 3 items
        let ol_reversed = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![("reversed".into(), "".into())],
        });
        dom.append_child(body, ol_reversed);

        let li_r1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_reversed, li_r1);
        let text_r1 = dom.create_node(NodeData::Text("r1".into()));
        dom.append_child(li_r1, text_r1);

        let li_r2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_reversed, li_r2);
        let text_r2 = dom.create_node(NodeData::Text("r2".into()));
        dom.append_child(li_r2, text_r2);

        let li_r3 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_reversed, li_r3);
        let text_r3 = dom.create_node(NodeData::Text("r3".into()));
        dom.append_child(li_r3, text_r3);

        // 3. <ol> with second <li value="10">
        let ol_val = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![],
        });
        dom.append_child(body, ol_val);

        let li_v1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_val, li_v1);
        let text_v1 = dom.create_node(NodeData::Text("v1".into()));
        dom.append_child(li_v1, text_v1);

        let li_v2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![("value".into(), "10".into())],
        });
        dom.append_child(ol_val, li_v2);
        let text_v2 = dom.create_node(NodeData::Text("v2".into()));
        dom.append_child(li_v2, text_v2);

        let li_v3 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_val, li_v3);
        let text_v3 = dom.create_node(NodeData::Text("v3".into()));
        dom.append_child(li_v3, text_v3);

        // 4. Plain <ol> with 2 items
        let ol_plain = dom.create_node(NodeData::Element {
            name: "ol".into(),
            attrs: vec![],
        });
        dom.append_child(body, ol_plain);

        let li_p1 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_plain, li_p1);
        let text_p1 = dom.create_node(NodeData::Text("p1".into()));
        dom.append_child(li_p1, text_p1);

        let li_p2 = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ol_plain, li_p2);
        let text_p2 = dom.create_node(NodeData::Text("p2".into()));
        dom.append_child(li_p2, text_p2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ol { display: block; padding-left: 40px; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // We added 4 ol lists to the body, so body_box should have 4 children
        assert_eq!(body_box.children.len(), 4);

        let ol_start_box = &body_box.children[0];
        let ol_reversed_box = &body_box.children[1];
        let ol_val_box = &body_box.children[2];
        let ol_plain_box = &body_box.children[3];

        // 1. Verify start="5" markers
        assert_eq!(ol_start_box.children.len(), 3);
        let li_s1_marker = &ol_start_box.children[0].children[1];
        let li_s2_marker = &ol_start_box.children[1].children[1];
        let li_s3_marker = &ol_start_box.children[2].children[1];
        assert_eq!(li_s1_marker.text.as_deref(), Some("5."));
        assert_eq!(li_s2_marker.text.as_deref(), Some("6."));
        assert_eq!(li_s3_marker.text.as_deref(), Some("7."));

        // 2. Verify reversed markers
        assert_eq!(ol_reversed_box.children.len(), 3);
        let li_r1_marker = &ol_reversed_box.children[0].children[1];
        let li_r2_marker = &ol_reversed_box.children[1].children[1];
        let li_r3_marker = &ol_reversed_box.children[2].children[1];
        assert_eq!(li_r1_marker.text.as_deref(), Some("3."));
        assert_eq!(li_r2_marker.text.as_deref(), Some("2."));
        assert_eq!(li_r3_marker.text.as_deref(), Some("1."));

        // 3. Verify value="10" override markers
        assert_eq!(ol_val_box.children.len(), 3);
        let li_v1_marker = &ol_val_box.children[0].children[1];
        let li_v2_marker = &ol_val_box.children[1].children[1];
        let li_v3_marker = &ol_val_box.children[2].children[1];
        assert_eq!(li_v1_marker.text.as_deref(), Some("1."));
        assert_eq!(li_v2_marker.text.as_deref(), Some("10."));
        assert_eq!(li_v3_marker.text.as_deref(), Some("11."));

        // 4. Verify plain markers (no regression)
        assert_eq!(ol_plain_box.children.len(), 2);
        let li_p1_marker = &ol_plain_box.children[0].children[1];
        let li_p2_marker = &ol_plain_box.children[1].children[1];
        assert_eq!(li_p1_marker.text.as_deref(), Some("1."));
        assert_eq!(li_p2_marker.text.as_deref(), Some("2."));
    }

    #[test]
    fn test_margin_auto_centering() {
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
                width: 300px;
                margin-left: auto;
                margin-right: auto;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        // Container is 500px. div width is 300px with auto margins.
        // Remaining space is 200px. Left and right margins should be 100px.
        // Therefore, div's border box should start at x = 100.0px.
        assert!(approx_eq(div_box.rect.origin.x, 100.0));
        assert!(approx_eq(div_box.rect.size.width, 300.0));
    }

    #[test]
    fn test_center_element_behavior() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let center = dom.create_node(NodeData::Element {
            name: "center".into(),
            attrs: vec![],
        });
        dom.append_child(body, center);

        let text = dom.create_node(NodeData::Text("Hello".into())); // 5 characters = 40px
        dom.append_child(center, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];
        let center_box = &body_box.children[0];
        let line_box = &center_box.children[0];
        let word_box = &line_box.children[0];

        // <center> element is treated as block-level and behaves as text-align: center.
        // Container = 500px. Text width = 40px.
        // It should shift the word box to x = 230.0px.
        assert!(approx_eq(word_box.rect.origin.x, 230.0));
    }

    #[test]
    fn test_nested_block_layout() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(div1, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                margin-left: 10px;
                margin-right: 10px;
                margin-top: 10px;
                margin-bottom: 10px;
                padding-left: 20px;
                padding-right: 20px;
                padding-top: 20px;
                padding-bottom: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);

        // root_box (Document)
        assert_eq!(layout_tree.node, Some(doc));
        assert_eq!(layout_tree.children.len(), 1);

        // body_box
        let body_box = &layout_tree.children[0];
        assert_eq!(body_box.node, Some(body));
        assert!(approx_eq(body_box.rect.origin.x, 0.0));
        assert!(approx_eq(body_box.rect.origin.y, 0.0));
        assert!(approx_eq(body_box.rect.size.width, 500.0));

        // div1_box
        assert_eq!(body_box.children.len(), 1);
        let div1_box = &body_box.children[0];
        assert_eq!(div1_box.node, Some(div1));
        // body width is 500. div1 has margin 10.
        // content_width = 500 - 10 - 10 = 480.
        // border_box width = 480 + 20 + 20 = 520?
        // Wait, width property in CSS is content width.
        // "width = containing-block width minus its own horizontal margin/padding/border"
        // containing_width = 500.
        // margin_left=10, margin_right=10, padding_left=20, padding_right=20.
        // auto_width = 500 - 10 - 10 - 20 - 20 = 440.
        // content_width = 440.
        // border_box_width = 440 + 20 + 20 = 480.
        assert!(approx_eq(div1_box.rect.size.width, 480.0));
        assert!(approx_eq(div1_box.rect.origin.x, 10.0));
        assert!(approx_eq(div1_box.rect.origin.y, 10.0));

        // div2_box
        assert_eq!(div1_box.children.len(), 1);
        let div2_box = &div1_box.children[0];
        assert_eq!(div2_box.node, Some(div2));
        // containing_width for div2 is div1's content width = 440.
        // margin=10, padding=20.
        // auto_width = 440 - 10 - 10 - 20 - 20 = 380.
        // border_box_width = 380 + 20 + 20 = 420.
        assert!(approx_eq(div2_box.rect.size.width, 420.0));
        // div2 x = div1 border box x (10) + div1 padding (20) + div2 margin (10) = 40.
        assert!(approx_eq(div2_box.rect.origin.x, 40.0));
        // div2 y = div1 border box y (10) + div1 padding (20) + div2 margin (10) = 40.
        assert!(approx_eq(div2_box.rect.origin.y, 40.0));
    }

    #[test]
    fn test_display_none() {
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

        let stylesheet = parse_stylesheet("div { display: none; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        assert_eq!(body_box.children.len(), 0);
    }

    #[test]
    fn test_explicit_height() {
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

        let stylesheet = parse_stylesheet("div { display: block; height: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];
        assert!(approx_eq(div_box.rect.size.height, 100.0));
    }

    #[test]
    fn test_unitless_zero_dimensions() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // Div 1: width:0, height:20px.
        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "z1".into())],
        });
        dom.append_child(body, div1);

        // Div 2: width:0px (control), height:20px.
        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "z2".into())],
        });
        dom.append_child(body, div2);

        // Div 3: width:200px, height:0.
        let div3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "z3".into())],
        });
        dom.append_child(body, div3);

        // Div 4: width:200px, height:0px (control).
        let div4 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "z4".into())],
        });
        dom.append_child(body, div4);

        // Div 5: invalid unitless non-zero (width: 5). Should stay auto (== containing block, 400px).
        let div5 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "z5".into())],
        });
        dom.append_child(body, div5);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 400px; }
            .z1 { display: block; width: 0; height: 20px; }
            .z2 { display: block; width: 0px; height: 20px; }
            .z3 { display: block; width: 200px; height: 0; }
            .z4 { display: block; width: 200px; height: 0px; }
            .z5 { display: block; width: 5; height: 20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 400.0);
        let body_box = &layout_tree.children[0];

        let div1_box = &body_box.children[0];
        let div2_box = &body_box.children[1];
        let div3_box = &body_box.children[2];
        let div4_box = &body_box.children[3];
        let div5_box = &body_box.children[4];

        // width:0 vs width:0px (both 0.0)
        assert!(approx_eq(div1_box.rect.size.width, 0.0));
        assert!(approx_eq(div2_box.rect.size.width, 0.0));

        // height:0 vs height:0px (both 0.0)
        assert!(approx_eq(div3_box.rect.size.height, 0.0));
        assert!(approx_eq(div4_box.rect.size.height, 0.0));

        // width:5 (invalid) stays auto (== 400.0)
        assert!(approx_eq(div5_box.rect.size.width, 400.0));
    }

    #[test]
    fn test_text_wrapping() {
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

        // CHAR_WIDTH is 8.0. LINE_HEIGHT is 8.0 (font line_height).
        // Words: "Hello " (48px), "world " (48px), "this " (40px), "is " (24px), "a " (16px), "test" (32px)
        // Limit: 150px
        // Line 1: "Hello ", "world ", "this " (Total 136px)
        // Line 2: "is ", "a ", "test" (Total 72px)
        let text = dom.create_node(NodeData::Text("Hello world this is a test".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 150px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        // Should have 2 line boxes
        assert_eq!(div_box.children.len(), 2);

        // div_box height should be 2 * LINE_HEIGHT (8.0) = 16.0
        assert!(approx_eq(div_box.rect.size.height, 16.0));

        // Check first line children
        let line1 = &div_box.children[0];
        assert_eq!(line1.children.len(), 3);
        assert!(approx_eq(line1.rect.size.width, 136.0));

        let line2 = &div_box.children[1];
        assert_eq!(line2.children.len(), 3);
        assert!(approx_eq(line2.rect.size.width, 72.0));
    }

    #[test]
    fn test_inline_line_wrapping_acceptance() {
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

        // Word "aaaa " is 5 chars * 8px = 40px
        // Word "bbbb " is 5 chars * 8px = 40px
        // Word "cccc" is 4 chars * 8px = 32px
        // Total of first two is 80px <= 100px width.
        // Adding third makes 112px > 100px width.
        // It must wrap to at least 2 lines.
        let text = dom.create_node(NodeData::Text("aaaa bbbb cccc".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 100px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        assert!(div_box.children.len() >= 2);

        let line1 = &div_box.children[0];
        assert_eq!(line1.children.len(), 2); // "aaaa ", "bbbb "
        assert!(approx_eq(line1.rect.size.width, 80.0));

        let line2 = &div_box.children[1];
        assert_eq!(line2.children.len(), 1); // "cccc"
        assert!(approx_eq(line2.rect.size.width, 32.0));
    }

    #[test]
    fn test_s57_text_line_flow_wiring() {
        // spec: S-57
        // Verify that long text in a narrow container occupies multiple line-heights of block height,
        // using the real font glyph widths and inline wrapping, and advances block height accordingly.
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

        // Built-in font has character width 8.0, and height 8.0.
        // "word1 word2 word3 word4" has:
        // "word1 " (40px with space)
        // "word2 " (40px with space)
        // "word3 " (40px with space)
        // "word4"  (40px without space)
        // In a container of width 50px, each word has to be wrapped on its own line!
        // This will result in 4 lines, total height 4 * 8.0 = 32.0.
        let text = dom.create_node(NodeData::Text("word1 word2 word3 word4".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet("div { display: block; width: 50px; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];

        // Should have exactly 4 line boxes
        assert_eq!(div_box.children.len(), 4);

        // Block height must be exactly 4 * line-height (8.0) = 32.0
        assert!(approx_eq(div_box.rect.size.height, 32.0));

        // Let's check each line width:
        // "word1 " is 6 characters (5 + space), so 48px.
        // "word2 " is 48px.
        // "word3 " is 48px.
        // "word4" is 5 characters (no space), so 40px.
        assert!(approx_eq(div_box.children[0].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[1].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[2].rect.size.width, 48.0));
        assert!(approx_eq(div_box.children[3].rect.size.width, 40.0));
    }

    #[test]
    fn test_relative_position_offset() {
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
                position: relative;
                top: 10px;
                left: 20px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // First div (relative)
        let div_box = &body_box.children[0];
        assert_eq!(div_box.node, Some(div));
        // Static layout position would be (0, 0)
        // With relative top:10px; left:20px, it should be offset to (20, 10)
        assert!(approx_eq(div_box.rect.origin.x, 20.0));
        assert!(approx_eq(div_box.rect.origin.y, 10.0));

        // Second div (sibling)
        let sibling_box = &body_box.children[1];
        assert_eq!(sibling_box.node, Some(sibling));
        // Sibling position should not be affected by first div's relative offset
        // Static height of first div is 50px. Sibling should start at (0, 50).
        assert!(approx_eq(sibling_box.rect.origin.x, 0.0));
        assert!(approx_eq(sibling_box.rect.origin.y, 50.0));
    }

    #[test]
    fn test_absolute_position_and_out_of_flow() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div1);

        let div_abs = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div_abs);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                height: 50px;
            }
            body > div:nth-child(2) {
                position: absolute;
                top: 5px;
                left: 5px;
                width: 100px;
                height: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // div1 (static)
        let div1_box = &body_box.children[0];
        assert_eq!(div1_box.node, Some(div1));
        assert!(approx_eq(div1_box.rect.origin.x, 0.0));
        assert!(approx_eq(div1_box.rect.origin.y, 0.0));

        // div2 (static, sibling after the absolute div)
        let div2_box = &body_box.children[1];
        assert_eq!(div2_box.node, Some(div2));
        // Since absolute div is out of flow, div2 should immediately follow div1.
        // div1 has height 50, so div2 starts at (0, 50).
        assert!(approx_eq(div2_box.rect.origin.x, 0.0));
        assert!(approx_eq(div2_box.rect.origin.y, 50.0));

        // div_abs (absolute)
        // It is integrated into parent's children (or root_box/nearest ancestor layout box)
        // So let's find it in the layout tree.
        let div_abs_box = body_box
            .children
            .iter()
            .find(|b| b.node == Some(div_abs))
            .expect("abs box should exist in layout tree");
        // It should be placed at (5, 5) relative to viewport/containing block
        assert!(approx_eq(div_abs_box.rect.origin.x, 5.0));
        assert!(approx_eq(div_abs_box.rect.origin.y, 5.0));
        assert!(approx_eq(div_abs_box.rect.size.width, 100.0));
        assert!(approx_eq(div_abs_box.rect.size.height, 100.0));
    }

    #[test]
    fn test_relative_nested_absolute_no_shifting() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let rel_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, rel_div);

        let abs_child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(rel_div, abs_child);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            body > div {
                position: relative;
                top: 20px;
                left: 30px;
                height: 50px;
            }
            div > div {
                position: absolute;
                top: 5px;
                left: 5px;
                width: 40px;
                height: 40px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // Relative div
        let rel_box = &body_box.children[0];
        assert_eq!(rel_box.node, Some(rel_div));
        assert!(approx_eq(rel_box.rect.origin.x, 30.0));
        assert!(approx_eq(rel_box.rect.origin.y, 20.0));

        // Absolute child (it is positioned relative to viewport for v1)
        // In the layout tree, it is added under its nearest ancestor layout box, which is rel_box.
        let abs_box = rel_box
            .children
            .iter()
            .find(|b| b.node == Some(abs_child))
            .expect("abs child box should exist in layout tree");
        // Since it is positioned relative to the viewport/containing block (use viewport/root for v1),
        // its final coordinates should be (5, 5) and NOT shifted by the relative parent's offset of (30, 20)!
        assert!(approx_eq(abs_box.rect.origin.x, 5.0));
        assert!(approx_eq(abs_box.rect.origin.y, 5.0));
    }

    #[test]
    fn test_hit_test() {
        let mut dom = Dom::new();
        let node_root = dom.create_node(NodeData::Comment("root".into()));
        let node_child1 = dom.create_node(NodeData::Comment("child1".into()));
        let node_nested = dom.create_node(NodeData::Comment("nested".into()));
        let node_child2 = dom.create_node(NodeData::Comment("child2".into()));
        let node_nested_under_none = dom.create_node(NodeData::Comment("nested_under_none".into()));

        let root_box = LayoutBox {
            node: Some(node_root),
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            children: vec![
                LayoutBox {
                    node: Some(node_child1),
                    rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                    children: vec![LayoutBox {
                        node: Some(node_nested),
                        rect: Rect::new(15.0, 15.0, 20.0, 20.0),
                        children: vec![],
                        text: None,
                    }],
                    text: None,
                },
                LayoutBox {
                    node: Some(node_child2),
                    rect: Rect::new(40.0, 10.0, 40.0, 40.0),
                    children: vec![],
                    text: None,
                },
                LayoutBox {
                    node: None,
                    rect: Rect::new(0.0, 80.0, 100.0, 20.0),
                    children: vec![LayoutBox {
                        node: Some(node_nested_under_none),
                        rect: Rect::new(10.0, 85.0, 20.0, 10.0),
                        children: vec![],
                        text: None,
                    }],
                    text: None,
                },
            ],
            text: None,
        };

        // Point outside everything returns None
        assert_eq!(hit_test(&root_box, 150.0, 150.0), None);
        assert_eq!(hit_test(&root_box, -5.0, 20.0), None);

        // Point only in root box returns root
        assert_eq!(hit_test(&root_box, 5.0, 5.0), Some(node_root));

        // Point in child1 but not nested returns child1
        assert_eq!(hit_test(&root_box, 12.0, 12.0), Some(node_child1));

        // Point inside nested returns nested (deepest wins)
        assert_eq!(hit_test(&root_box, 20.0, 20.0), Some(node_nested));

        // Point inside both child1 and child2 (overlapping region: child2 is later sibling, so child2 wins)
        assert_eq!(hit_test(&root_box, 45.0, 20.0), Some(node_child2));

        // Point inside the no_node box, but not nested_under_none:
        // Returns root (since no_node is skipped, and root contains the point)
        assert_eq!(hit_test(&root_box, 5.0, 95.0), Some(node_root));

        // Point inside nested_under_none (inside no_node and root):
        // Returns nested_under_none
        assert_eq!(
            hit_test(&root_box, 15.0, 90.0),
            Some(node_nested_under_none)
        );
    }

    #[test]
    fn test_aspect_ratio_sizing() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // Case 1: A block with a fixed width (200px), height auto, aspect-ratio: 2 / 1 -> height ≈ 100px
        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div1);

        // Case 2: Single-number form aspect-ratio: 4 with width 200px, height auto -> height ≈ 50px
        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div2);

        // Case 3: A block with BOTH height: 30px and aspect-ratio: 2 / 1 -> height stays 30px
        let div3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div3);

        // Case 4: No aspect-ratio (regression guard) -> height should be 0px as it is empty
        let div4 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div4);

        let mut styles = std::collections::HashMap::new();

        // Style for body
        let mut body_style = ComputedStyle::default();
        body_style.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        body_style.insert("width".to_string(), CssValue::Length(500.0, LengthUnit::Px));
        styles.insert(body, body_style);

        // Style for div1: width 200px, aspect-ratio: 2 / 1 (as Keyword)
        let mut style1 = ComputedStyle::default();
        style1.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        style1.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        style1.insert(
            "aspect-ratio".to_string(),
            CssValue::Keyword("2 / 1".to_string()),
        );
        styles.insert(div1, style1);

        // Style for div2: width 200px, aspect-ratio: 4 (as Number)
        let mut style2 = ComputedStyle::default();
        style2.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        style2.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        style2.insert("aspect-ratio".to_string(), CssValue::Number(4.0));
        styles.insert(div2, style2);

        // Style for div3: width 200px, height 30px, aspect-ratio: 2 / 1
        let mut style3 = ComputedStyle::default();
        style3.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        style3.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        style3.insert("height".to_string(), CssValue::Length(30.0, LengthUnit::Px));
        style3.insert(
            "aspect-ratio".to_string(),
            CssValue::Keyword("2 / 1".to_string()),
        );
        styles.insert(div3, style3);

        // Style for div4: width 200px, no aspect-ratio
        let mut style4 = ComputedStyle::default();
        style4.insert(
            "display".to_string(),
            CssValue::Keyword("block".to_string()),
        );
        style4.insert("width".to_string(), CssValue::Length(200.0, LengthUnit::Px));
        styles.insert(div4, style4);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // Verify Case 1
        let box1 = &body_box.children[0];
        assert!(approx_eq(box1.rect.size.width, 200.0));
        assert!(approx_eq(box1.rect.size.height, 100.0));

        // Verify Case 2
        let box2 = &body_box.children[1];
        assert!(approx_eq(box2.rect.size.width, 200.0));
        assert!(approx_eq(box2.rect.size.height, 50.0));

        // Verify Case 3
        let box3 = &body_box.children[2];
        assert!(approx_eq(box3.rect.size.width, 200.0));
        assert!(approx_eq(box3.rect.size.height, 30.0));

        // Verify Case 4
        let box4 = &body_box.children[3];
        assert!(approx_eq(box4.rect.size.width, 200.0));
        assert!(approx_eq(box4.rect.size.height, 0.0));
    }

    #[test]
    fn test_display_none_absolute_subtree_pruning() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let container = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, container);

        let abs_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(container, abs_div);

        let stylesheet = parse_stylesheet(
            "
            div { display: none; }
            div > div { position: absolute; top: 10px; left: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // Neither container nor abs_div should have any box, so body should have 0 children
        assert_eq!(body_box.children.len(), 0);
    }

    #[test]
    fn test_deep_tree_recursion_cap() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let mut current = doc;
        // Nest 1050 levels deep
        for _ in 0..1050 {
            let div = dom.create_node(NodeData::Element {
                name: "div".into(),
                attrs: vec![],
            });
            dom.append_child(current, div);
            current = div;
        }

        let stylesheet = parse_stylesheet("div { display: block; }");
        let styles = compute_styles(&dom, &stylesheet);

        // This must not stack overflow!
        let _layout_tree = layout_document(&dom, &styles, 800.0);
    }

    #[test]
    fn test_mixed_layout_body_with_div_and_whitespace() {
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

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        let text_hi = dom.create_node(NodeData::Text("hi".into()));
        dom.append_child(p, text_hi);

        let text_ws = dom.create_node(NodeData::Text("\n".into()));
        dom.append_child(body, text_ws);

        let stylesheet = parse_stylesheet(
            "body { display: block; } div { display: block; } p { display: block; }",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // The body has a mix of div (block) and "\n" (whitespace inline).
        // The div should be laid out as a normal block box under body.
        // The whitespace text collapses and yields no line box (so no anonymous block box is created for it).
        // Therefore, body_box has exactly 1 child: the div_box.
        assert_eq!(body_box.children.len(), 1);

        let div_box = &body_box.children[0];
        assert_eq!(div_box.node, Some(div));

        // Under div, there is p (block). So div has 1 child: p_box.
        assert_eq!(div_box.children.len(), 1);

        let p_box = &div_box.children[0];
        assert_eq!(p_box.node, Some(p));

        // Under p, there is "hi" (inline). So p has 1 child: line box.
        assert_eq!(p_box.children.len(), 1);
        let line_box = &p_box.children[0];
        assert_eq!(line_box.node, None); // Anonymous line box
        assert!(line_box.rect.size.width > 0.0);
        assert!(line_box.rect.size.height > 0.0);

        let text_box = &line_box.children[0];
        assert_eq!(text_box.node, Some(text_hi));
        assert!(text_box.rect.size.width > 0.0);
        assert!(text_box.rect.size.height > 0.0);
    }

    #[test]
    fn test_mixed_layout_block_with_mixed_text_div_text() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let parent_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, parent_div);

        let text1 = dom.create_node(NodeData::Text("first ".into()));
        dom.append_child(parent_div, text1);

        let child_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(parent_div, child_div);

        let text_child = dom.create_node(NodeData::Text("second ".into()));
        dom.append_child(child_div, text_child);

        let text3 = dom.create_node(NodeData::Text(" third".into()));
        dom.append_child(parent_div, text3);

        let stylesheet = parse_stylesheet("body { display: block; } div { display: block; }");
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let parent_div_box = &body_box.children[0];

        // Mixed layout: [text1, child_div, text3]
        // Should produce:
        // 1. Anonymous block box wrapping text1
        // 2. Normal block box for child_div
        // 3. Anonymous block box wrapping text3
        assert_eq!(parent_div_box.children.len(), 3);

        let anon1 = &parent_div_box.children[0];
        assert_eq!(anon1.node, None); // Anonymous block box
        assert!(anon1.rect.size.width > 0.0);
        assert!(anon1.rect.size.height > 0.0);

        let child_div_box = &parent_div_box.children[1];
        assert_eq!(child_div_box.node, Some(child_div));
        assert!(child_div_box.rect.size.width > 0.0);
        assert!(child_div_box.rect.size.height > 0.0);

        let anon3 = &parent_div_box.children[2];
        assert_eq!(anon3.node, None); // Anonymous block box
        assert!(anon3.rect.size.width > 0.0);
        assert!(anon3.rect.size.height > 0.0);

        // Verify correct vertical ordering and positions
        assert!(anon1.rect.origin.y < child_div_box.rect.origin.y);
        assert!(child_div_box.rect.origin.y < anon3.rect.origin.y);

        // Check height matches sum of elements
        let expected_min_height =
            anon1.rect.size.height + child_div_box.rect.size.height + anon3.rect.size.height;
        assert!(parent_div_box.rect.size.height >= expected_min_height);
    }

    #[test]
    fn test_margin_collapse_positive() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: 30px; }
            .next { margin-top: 20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = max(30, 20) = 30.
        // So box2 border box y should be box1.rect.max_y() + 30 = 50.0 + 30.0 = 80.0.
        assert!(
            approx_eq(box2.rect.origin.y, 80.0),
            "Expected second sibling border box y to be 80.0, got {}",
            box2.rect.origin.y
        );
    }

    #[test]
    fn test_margin_collapse_mixed() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: 30px; }
            .next { margin-top: -10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = max(30, 0) + min(-10, 0) = 30 + (-10) = 20.
        // So box2 border box y should be box1.rect.max_y() + 20 = 50.0 + 20.0 = 70.0.
        assert!(
            approx_eq(box2.rect.origin.y, 70.0),
            "Expected second sibling border box y to be 70.0, got {}",
            box2.rect.origin.y
        );
    }

    #[test]
    fn test_margin_collapse_both_negative() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "prev".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "next".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div { display: block; height: 50px; }
            .prev { margin-bottom: -10px; }
            .next { margin-top: -20px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 2);
        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert!(approx_eq(box1.rect.origin.y, 0.0));
        assert!(approx_eq(box1.rect.size.height, 50.0));

        // Collapsed margin = min(-10, -20) = -20.
        // So box2 border box y should be box1.rect.max_y() + (-20) = 50.0 - 20.0 = 30.0.
        assert!(
            approx_eq(box2.rect.origin.y, 30.0),
            "Expected second sibling border box y to be 30.0, got {}",
            box2.rect.origin.y
        );
    }

    #[test]
    fn test_auto_positioned_absolute_elements_in_flow_t0164b() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            div {
                display: block;
                position: absolute; /* auto-positioned, no top or left */
                height: 50px;
                width: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // Since both div1 and div2 have position: absolute but no top/left specified,
        // they should be kept in normal flow (as if position: static).
        // They should NOT be collapsed to (0,0).
        assert_eq!(body_box.children.len(), 2);

        let box1 = &body_box.children[0];
        let box2 = &body_box.children[1];

        assert_eq!(box1.node, Some(div1));
        assert!(approx_eq(box1.rect.origin.x, 0.0));
        assert!(approx_eq(box1.rect.origin.y, 0.0));

        assert_eq!(box2.node, Some(div2));
        assert!(approx_eq(box2.rect.origin.x, 0.0));
        assert!(approx_eq(box2.rect.origin.y, 50.0)); // Should be in-flow after box1
    }

    #[test]
    fn test_explicit_absolute_positioned_elements_t0164b() {
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
                top: 10px;
                left: 15px;
                height: 50px;
                width: 100px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // div has explicit top/left, so it should be out-of-flow and processed by layout_absolute_and_fixed_elements
        let div_box = body_box
            .children
            .iter()
            .find(|b| b.node == Some(div))
            .expect("abs box should exist in layout tree");

        assert!(approx_eq(div_box.rect.origin.x, 15.0));
        assert!(approx_eq(div_box.rect.origin.y, 10.0));
    }

    #[test]
    fn test_inline_block_sits_beside_text() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(body, ib);

        let inner_text = dom.create_node(NodeData::Text("hello".into()));
        dom.append_child(ib, inner_text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            span {
                display: inline-block;
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        // The body should contain one anonymous line box
        assert_eq!(body_box.children.len(), 1);
        let line_box = &body_box.children[0];

        // The line box should contain the span (inline-block) box
        assert_eq!(line_box.children.len(), 1);
        let span_box = &line_box.children[0];

        assert_eq!(span_box.node, Some(ib));
        assert!(approx_eq(span_box.rect.size.width, 100.0));
        assert!(approx_eq(span_box.rect.size.height, 50.0));
        // The inline-block should contain its own laid out text node child
        assert_eq!(span_box.children.len(), 1);
    }

    #[test]
    fn test_inline_block_wrapping() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let ib1 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(body, ib1);

        let ib2 = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(body, ib2);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 150px; }
            span {
                display: inline-block;
                width: 100px;
                height: 50px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 150.0);
        let body_box = &layout_tree.children[0];

        // Since body is 150px wide and each span is 100px wide, they cannot fit on the same line.
        // Therefore, we should have 2 line boxes.
        assert_eq!(body_box.children.len(), 2);

        let line_box1 = &body_box.children[0];
        let line_box2 = &body_box.children[1];

        assert_eq!(line_box1.children.len(), 1);
        assert_eq!(line_box2.children.len(), 1);

        let box1 = &line_box1.children[0];
        let box2 = &line_box2.children[0];

        assert_eq!(box1.node, Some(ib1));
        assert_eq!(box2.node, Some(ib2));

        // The second box should be positioned below the first one
        assert!(box2.rect.origin.y >= box1.rect.origin.y + 50.0);
    }

    #[test]
    fn test_inline_block_shrink_to_fit() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let ib = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(body, ib);

        let text = dom.create_node(NodeData::Text("ab".into())); // word of 20px
        dom.append_child(ib, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            span {
                display: inline-block;
                padding-left: 5px;
                padding-right: 5px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 500.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 1);
        let line_box = &body_box.children[0];
        let span_box = &line_box.children[0];

        // "ab" is measured by font as 16px wide (8px per character).
        // Plus padding-left (5px) and padding-right (5px), total border box width should be 26.0px.
        assert!(approx_eq(span_box.rect.size.width, 26.0));
    }

    #[test]
    fn test_shrink_to_fit_inline_block_centers_child_at_left_edge() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // This is the container under text-align: center
        let outer_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "outer".into())],
        });
        dom.append_child(body, outer_div);

        // This is the shrink-to-fit inline-block element
        let stf_div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "stf".into())],
        });
        dom.append_child(outer_div, stf_div);

        // This is the child inside the shrink-to-fit container
        let child_span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(stf_div, child_span);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 1000px; }
            .outer { display: block; text-align: center; }
            .stf { display: inline-block; text-align: center; }
            .child { display: inline-block; width: 200px; height: 50px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 1000.0);
        let body_box = &layout_tree.children[0];
        let outer_box = &body_box.children[0];
        // stf_div is inline-block, so it's placed inside a line box under outer_box
        let outer_line_box = &outer_box.children[0];
        let stf_box = &outer_line_box.children[0];

        // The stf_box has shrink-to-fit width, which should be exactly 200px because its child has 200px width.
        assert!(approx_eq(stf_box.rect.size.width, 200.0));

        // The children inside stf_box are inline (the .child inline-block span).
        // Since stf_box is inline-block and has all inline children, its children are placed in a line box inside stf_box.
        let stf_line_box = &stf_box.children[0];
        let child_box = &stf_line_box.children[0];

        // The child's origin x should be exactly equal to the left of stf_box's content area.
        // Let's assert that the difference is very small (less than 1.0)
        let container_left = stf_box.rect.origin.x; // no padding/border on .stf
        let child_left = child_box.rect.origin.x;

        assert!(
            (child_left - container_left).abs() < 1.0,
            "child_left: {}, container_left: {}",
            child_left,
            container_left
        );
    }

    #[test]
    fn test_width_constrained_centered_container_text_wrapping_geometry() {
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

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        // A long text: "This domain is for use in illustrative examples in documents."
        // Under a 600px width minus 64px padding (536px content width),
        // This text should fit on 1 or 2 lines, certainly NOT one word per line (which would be ~10 line boxes).
        let text = dom.create_node(NodeData::Text(
            "This domain is for use in illustrative examples in documents.".into(),
        ));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 800px; }
            div {
                display: block;
                width: 600px;
                margin: 0 auto;
                padding-left: 32px;
                padding-right: 32px;
            }
            p {
                display: block;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);

        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];
        let div_box = &body_box.children[0];
        let p_box = &div_box.children[0];

        // The container's definite width must thread down to the inline layout of
        // its descendant <p>, so the paragraph keeps a usable content width and does
        // not collapse to near-zero (which would wrap one word per line).

        // Assert container starts centered in 800px body (origin x = 68px)
        assert!(approx_eq(div_box.rect.origin.x, 68.0));
        assert!(approx_eq(div_box.rect.size.width, 664.0)); // 600 + 32 + 32 = 664

        // p content_width should be div content_width = 600.0
        assert!(approx_eq(p_box.rect.size.width, 600.0));

        // If it wrapped 1 word per line, there would be ~9 line boxes.
        // If it wrapped correctly, it should easily fit on 1 or 2 line boxes.
        assert!(
            p_box.children.len() <= 2,
            "Paragraph wrapped prematurely into {} lines",
            p_box.children.len()
        );
    }

    #[test]
    fn test_width_constrained_centered_container_text_wrapping_via_render() {
        let html = r#"
            <div style="width:600px; margin:0 auto; padding:2em">
                <h1>Example Domain</h1>
                <p>This domain is for use in illustrative examples in documents. You may use this domain in literature without prior coordination or asking for permission.</p>
            </div>
        "#;
        struct DummyLoader;
        impl crate::loader::ResourceLoader for DummyLoader {
            fn load(&self, _url: &crate::url::Url) -> Result<Vec<u8>, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }
            fn load_request(
                &self,
                _url: &crate::url::Url,
                _method: crate::loader::HttpMethod,
                _body: &[u8],
                _content_type: Option<&str>,
            ) -> Result<crate::loader::LoaderResponse, crate::loader::LoadError> {
                Err(crate::loader::LoadError::NotFound)
            }
        }
        let base_url = crate::url::Url::parse("http://localhost/").unwrap();
        let page = crate::engine::render_page(html, &base_url, &DummyLoader, 800.0);

        // Find the paragraph box in the layout tree and inspect it!
        let mut p_box = None;
        let mut stack = vec![&page.layout];
        while let Some(b) = stack.pop() {
            if let Some(node_id) = b.node
                && let Some(NodeData::Element { name, .. }) = page.dom.data(node_id)
                && name == "p"
            {
                p_box = Some(b);
                break;
            }
            for child in &b.children {
                stack.push(child);
            }
        }

        let p = p_box.expect("Paragraph box should exist in layout tree");

        // Under 600px width minus 2em padding the paragraph wraps onto a few lines;
        // it must NOT collapse to one word per line (~15+ line boxes for 20+ words).
        assert!(
            p.children.len() <= 3,
            "Paragraph wrapped prematurely into {} lines",
            p.children.len()
        );
    }

    #[test]
    fn test_form_control_button_intrinsic_width_and_height() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let input_submit = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "submit".into()),
                ("value".into(), "Hello".into()),
            ],
        });
        dom.append_child(body, input_submit);

        let input_no_value = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "submit".into())],
        });
        dom.append_child(body, input_no_value);

        let button_go = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(body, button_go);
        let go_text = dom.create_node(NodeData::Text("Go".into()));
        dom.append_child(button_go, go_text);

        let button_override = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![("class".into(), "overridden".into())],
        });
        dom.append_child(body, button_override);
        let override_text = dom.create_node(NodeData::Text("Width Overridden".into()));
        dom.append_child(button_override, override_text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 800px; }
            input, button {
                display: inline-block;
                padding-left: 5px;
                padding-right: 5px;
                border-left-width: 1px;
                border-right-width: 1px;
            }
            .overridden {
                width: 300px;
            }
            ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        // The body should contain 1 line box with inline-block elements.
        assert!(!body_box.children.is_empty());
        let line_box = &body_box.children[0];

        // Inside the line box, we expect:
        // children[0] -> input_submit ("Hello")
        // children[1] -> input_no_value ("Submit")
        // children[2] -> button_go ("Go")
        // children[3] -> button_override (width: 300px)
        assert_eq!(line_box.children.len(), 4);

        let box_hello = &line_box.children[0];
        let box_submit = &line_box.children[1];
        let box_go = &line_box.children[2];
        let box_override = &line_box.children[3];

        let font = crate::font::BitmapFont::builtin();
        let measure_hello = font.measure("Hello") as f32;
        let measure_submit = font.measure("Submit") as f32;
        let measure_go = font.measure("Go") as f32;
        let line_height = font.line_height() as f32;

        // Total border box width = measure + padding_left (5) + padding_right (5) + border_left (1) + border_right (1)
        let expected_hello_width = measure_hello + 12.0;
        let expected_submit_width = measure_submit + 12.0;
        let expected_go_width = measure_go + 12.0;

        assert!(approx_eq(box_hello.rect.size.width, expected_hello_width));
        assert!(approx_eq(box_submit.rect.size.width, expected_submit_width));
        assert!(approx_eq(box_go.rect.size.width, expected_go_width));

        // Box override must have explicit content width 300px + 12px padding/border = 312px.
        assert!(approx_eq(box_override.rect.size.width, 312.0));

        // Let's check height.
        // box_hello is <input> (void element), so it has children.is_empty() -> content height is at least line_height.
        assert!(approx_eq(box_hello.rect.size.height, line_height));

        // box_submit is <input> -> children.is_empty() -> content height is at least line_height.
        assert!(approx_eq(box_submit.rect.size.height, line_height));
    }

    #[test]
    fn test_inline_block_with_block_child_shrinks_to_content() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let c = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "c".into())],
        });
        dom.append_child(body, c);

        let ds = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "ds".into())],
        });
        dom.append_child(c, ds);

        let lsbb = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![("class".into(), "lsbb".into())],
        });
        dom.append_child(ds, lsbb);

        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "submit".into()),
                ("value".into(), "Go".into()),
            ],
        });
        dom.append_child(lsbb, input);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 1000px; }
            .c { display: block; }
            .ds { display: inline-block; }
            .lsbb { display: block; }
            ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 1000.0);

        // Find ds and lsbb boxes
        let mut ds_box = None;
        let mut lsbb_box = None;
        let mut stack = vec![&layout_tree];
        while let Some(b) = stack.pop() {
            if let Some(node_id) = b.node {
                if node_id == ds {
                    ds_box = Some(b);
                } else if node_id == lsbb {
                    lsbb_box = Some(b);
                }
            }
            for child in &b.children {
                stack.push(child);
            }
        }

        let ds_b = ds_box.expect(".ds box should exist in layout tree");
        let lsbb_b = lsbb_box.expect(".lsbb box should exist in layout tree");

        let font = crate::font::BitmapFont::builtin();
        let measure_go = font.measure("Go") as f32;

        // Both should shrink to approximately the measure of "Go" (e.g. 16px) and be way less than containing width (1000.0)
        assert!(ds_b.rect.size.width < 300.0);
        assert!(lsbb_b.rect.size.width < 300.0);
        assert!(approx_eq(ds_b.rect.size.width, measure_go));
        assert!(approx_eq(lsbb_b.rect.size.width, measure_go));
    }

    #[test]
    fn test_min_max_sizing_constraints() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // 1. max-width clamps: width: 1000px; max-width: 200px;
        let div_max_width = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "max-width-test".into())],
        });
        dom.append_child(body, div_max_width);

        // 2. min-width clamps: width: 10px; min-width: 300px;
        let div_min_width = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "min-width-test".into())],
        });
        dom.append_child(body, div_min_width);

        // 3. min beats max: width: 500px; max-width: 100px; min-width: 300px;
        let div_min_beats_max = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "min-beats-max-test".into())],
        });
        dom.append_child(body, div_min_beats_max);

        // 4. max-height clamps: height: 1000px; max-height: 50px;
        let div_max_height = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "max-height-test".into())],
        });
        dom.append_child(body, div_max_height);

        // 5. min-height clamps: height: 10px; min-height: 400px;
        let div_min_height = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "min-height-test".into())],
        });
        dom.append_child(body, div_min_height);

        // 6. no constraints (regression guard)
        let div_no_constraints = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "no-constraints-test".into())],
        });
        dom.append_child(body, div_no_constraints);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 1200px; }
            .max-width-test { display: block; width: 1000px; max-width: 200px; }
            .min-width-test { display: block; width: 10px; min-width: 300px; }
            .min-beats-max-test { display: block; width: 500px; max-width: 100px; min-width: 300px; }
            .max-height-test { display: block; height: 1000px; max-height: 50px; }
            .min-height-test { display: block; height: 10px; min-height: 400px; }
            .no-constraints-test { display: block; width: 150px; height: 150px; }
            ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 1200.0);
        let body_box = &layout_tree.children[0];

        // Ensure we laid out the children
        assert_eq!(body_box.children.len(), 6);

        let box_max_width = &body_box.children[0];
        let box_min_width = &body_box.children[1];
        let box_min_beats_max = &body_box.children[2];
        let box_max_height = &body_box.children[3];
        let box_min_height = &body_box.children[4];
        let box_no_constraints = &body_box.children[5];

        // Assert width constraints
        assert!(approx_eq(box_max_width.rect.size.width, 200.0));
        assert!(approx_eq(box_min_width.rect.size.width, 300.0));
        assert!(approx_eq(box_min_beats_max.rect.size.width, 300.0));

        // Assert height constraints
        assert!(approx_eq(box_max_height.rect.size.height, 50.0));
        assert!(approx_eq(box_min_height.rect.size.height, 400.0));

        // Assert no constraints (regression guard)
        assert!(approx_eq(box_no_constraints.rect.size.width, 150.0));
        assert!(approx_eq(box_no_constraints.rect.size.height, 150.0));
    }

    #[test]
    fn test_percentage_min_max_sizing_constraints() {
        let mut dom = Dom::new();
        let doc = dom.document();

        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // A block with width: 1000px; max-width: 50% inside an 800px containing block resolves to content width 400px (50% of 800).
        let div_max_width = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "percent-max-width".into())],
        });
        dom.append_child(body, div_max_width);

        // A block with width: 100px; min-width: 50% inside an 800px containing block resolves to content width 400px.
        let div_min_width = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "percent-min-width".into())],
        });
        dom.append_child(body, div_min_width);

        // Keep a px max-width assertion to prove the px path is unchanged.
        let div_px_max_width = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "px-max-width".into())],
        });
        dom.append_child(body, div_px_max_width);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 800px; }
            .percent-max-width { display: block; width: 1000px; max-width: 50%; }
            .percent-min-width { display: block; width: 100px; min-width: 50%; }
            .px-max-width { display: block; width: 1000px; max-width: 200px; }
            ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 3);

        let box_percent_max_width = &body_box.children[0];
        let box_percent_min_width = &body_box.children[1];
        let box_px_max_width = &body_box.children[2];

        assert!(approx_eq(box_percent_max_width.rect.size.width, 400.0));
        assert!(approx_eq(box_percent_min_width.rect.size.width, 400.0));
        assert!(approx_eq(box_px_max_width.rect.size.width, 200.0));
    }

    #[test]
    fn test_consecutive_br_block_advance() {
        use crate::encoding::input_stream::InputStream;
        use crate::html::parse_document;

        let html = "<html><body>line one<br>line two<br><br>line four</body></html>";
        let input_stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(input_stream);

        let stylesheet = parse_stylesheet(crate::engine::UA_DEFAULT_CSS);
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 1);
        let real_body_box = &body_box.children[0];
        assert_eq!(real_body_box.children.len(), 4);

        let y_line_one = real_body_box.children[0].rect.origin.y;
        let y_line_two = real_body_box.children[1].rect.origin.y;
        let y_line_three_empty = real_body_box.children[2].rect.origin.y;
        let y_line_four = real_body_box.children[3].rect.origin.y;

        let line_height = 8.0; // The builtin bitmap font line height is 8.0px
        assert!(y_line_two >= y_line_one + line_height - EPSILON);
        assert!(y_line_three_empty >= y_line_two + line_height - EPSILON);
        assert!(y_line_four >= y_line_two + 2.0 * line_height - EPSILON);

        println!(
            "test_consecutive_br_block_advance passed with: y_one={}, y_two={}, y_three_empty={}, y_four={}",
            y_line_one, y_line_two, y_line_three_empty, y_line_four
        );
    }

    #[test]
    fn test_leading_consecutive_br_empty_line() {
        use crate::encoding::input_stream::InputStream;
        use crate::html::parse_document;

        let html = "<html><body>A<br><br>B</body></html>";
        let input_stream = InputStream::from_utf8(html.as_bytes());
        let dom = parse_document(input_stream);

        let stylesheet = parse_stylesheet(crate::engine::UA_DEFAULT_CSS);
        let styles = compute_styles(&dom, &stylesheet);
        let layout_tree = layout_document(&dom, &styles, 800.0);
        let body_box = &layout_tree.children[0];

        assert_eq!(body_box.children.len(), 1);
        let real_body_box = &body_box.children[0];
        assert_eq!(real_body_box.children.len(), 3);

        let y_of_a = real_body_box.children[0].rect.origin.y;
        let y_of_b = real_body_box.children[2].rect.origin.y;

        let line_height = 8.0;
        assert!(y_of_b >= y_of_a + 2.0 * line_height - EPSILON);
    }
}
