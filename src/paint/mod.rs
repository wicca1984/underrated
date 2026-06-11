use crate::css::values::{Color, CssValue};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::ComputedStyle;
use std::collections::HashMap;

/// A single item to be displayed on the screen.
/// spec: S-12
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayItem {
    SolidRect {
        rect: Rect,
        color: Color,
    },
    Text {
        rect: Rect,
        text: String,
        color: Color,
    },
    Image {
        rect: Rect,
        src: String,
        base_url: Option<String>,
        decoded: Option<crate::image::DecodedImage>,
    },
}

/// A list of display items, representing the final visual output.
/// spec: S-12
pub struct DisplayList(pub Vec<DisplayItem>);

/// Helper to extract border width from computed styles in px.
/// spec: S-39
fn get_border_width(style: &ComputedStyle, prop: &str) -> f32 {
    match style.get(prop) {
        Some(CssValue::Length(v, crate::css::values::LengthUnit::Px)) => *v,
        _ => 0.0,
    }
}

/// Helper to extract outline width from computed styles in px, defaulting to medium (3.0px).
fn get_outline_width(style: &ComputedStyle) -> f32 {
    match style.get("outline-width") {
        Some(CssValue::Length(v, crate::css::values::LengthUnit::Px)) => *v,
        Some(CssValue::Number(v)) => *v,
        Some(CssValue::Keyword(s)) => match s.to_ascii_lowercase().as_str() {
            "thin" => 1.0,
            "medium" => 3.0,
            "thick" => 5.0,
            _ => 3.0,
        },
        _ => 3.0,
    }
}

/// Helper to extract outline offset from computed styles in px, defaulting to 0.0px.
fn get_outline_offset(style: &ComputedStyle) -> f32 {
    match style.get("outline-offset") {
        Some(CssValue::Length(v, crate::css::values::LengthUnit::Px)) => *v,
        Some(CssValue::Number(v)) => *v,
        _ => 0.0,
    }
}

/// Helper to recursively find any `Color` value inside a CSS property.
/// spec: S-39
fn find_color(value: &CssValue) -> Option<Color> {
    match value {
        CssValue::Color(c) => Some(c.clone()),
        CssValue::Multiple(values) => {
            for v in values {
                if let Some(c) = find_color(v) {
                    return Some(c);
                }
            }
            None
        }
        _ => None,
    }
}

/// Helper to recursively flatten a CSS value into leaf values.
fn flatten_value<'a>(value: &'a CssValue, leaves: &mut Vec<&'a CssValue>) {
    match value {
        CssValue::Multiple(values) => {
            for v in values {
                flatten_value(v, leaves);
            }
        }
        _ => {
            leaves.push(value);
        }
    }
}

/// Helper to resolve the general border color of an element style,
/// with fallbacks to shorthand `border`, computed text `color`, and finally black.
/// spec: S-39
fn get_border_color(style: &ComputedStyle) -> Color {
    // 1. Try "border-color" property
    if let Some(val) = style.get("border-color")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 2. Try the shorthand "border" property
    if let Some(val) = style.get("border")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 3. Fall back to computed text "color" (as standard currentColor fallback)
    if let Some(val) = style.get("color")
        && let Some(c) = find_color(val)
    {
        return c;
    }
    // 4. Default to black
    Color::Rgba(0, 0, 0, 255)
}

/// Helper to resolve a specific edge border color, falling back to the resolved border color.
/// spec: S-39
fn get_edge_color(style: &ComputedStyle, edge_prop: &str, border_color: &Color) -> Color {
    if let Some(val) = style.get(edge_prop)
        && let Some(c) = find_color(val)
    {
        return c;
    }
    border_color.clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextDecorationStyle {
    Solid,
    Double,
    Dotted,
    Dashed,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct TextDecorations {
    underline: bool,
    overline: bool,
    line_through: bool,
    color: Option<Color>,
    style: Option<TextDecorationStyle>,
}

/// Helper to get the set of text decorations from a ComputedStyle.
/// spec: S-55
fn get_text_decorations(style: &ComputedStyle) -> TextDecorations {
    let mut dec = TextDecorations::default();

    // spec: the `text-decoration` shorthand set to `none` clears the line,
    // overriding any keywords (e.g. from `text-decoration-line`).
    if let Some(CssValue::Keyword(s)) = style.get("text-decoration")
        && s.eq_ignore_ascii_case("none")
    {
        return dec;
    }

    for prop in &["text-decoration", "text-decoration-line"] {
        if let Some(val) = style.get(prop) {
            match val {
                CssValue::Keyword(s) => {
                    if s.eq_ignore_ascii_case("underline") {
                        dec.underline = true;
                    } else if s.eq_ignore_ascii_case("overline") {
                        dec.overline = true;
                    } else if s.eq_ignore_ascii_case("line-through") {
                        dec.line_through = true;
                    }
                }
                CssValue::Multiple(values) => {
                    for v in values {
                        if let CssValue::Keyword(s) = v {
                            if s.eq_ignore_ascii_case("underline") {
                                dec.underline = true;
                            } else if s.eq_ignore_ascii_case("overline") {
                                dec.overline = true;
                            } else if s.eq_ignore_ascii_case("line-through") {
                                dec.line_through = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Parse text-decoration-color property
    // TODO(spec): text-decoration-color v1 — single computed color only; `currentColor` keyword falls back to
    // the text color (i.e. treat unset/`currentColor` as None). text-decoration shorthand color parsing and
    // per-line distinct colors are out of scope.
    if let Some(val) = style.get("text-decoration-color") {
        dec.color = find_color(val);
    }

    // Parse text-decoration-style property
    if let Some(CssValue::Keyword(s)) = style.get("text-decoration-style") {
        if s.eq_ignore_ascii_case("solid") {
            dec.style = Some(TextDecorationStyle::Solid);
        } else if s.eq_ignore_ascii_case("double") {
            dec.style = Some(TextDecorationStyle::Double);
        } else if s.eq_ignore_ascii_case("dotted") {
            dec.style = Some(TextDecorationStyle::Dotted);
        } else if s.eq_ignore_ascii_case("dashed") {
            dec.style = Some(TextDecorationStyle::Dashed);
        }
    }

    // Also accept the style value when present in the `text-decoration` shorthand
    if let Some(val) = style.get("text-decoration") {
        match val {
            CssValue::Keyword(s) => {
                if s.eq_ignore_ascii_case("solid") {
                    dec.style = Some(TextDecorationStyle::Solid);
                } else if s.eq_ignore_ascii_case("double") {
                    dec.style = Some(TextDecorationStyle::Double);
                } else if s.eq_ignore_ascii_case("dotted") {
                    dec.style = Some(TextDecorationStyle::Dotted);
                } else if s.eq_ignore_ascii_case("dashed") {
                    dec.style = Some(TextDecorationStyle::Dashed);
                }
            }
            CssValue::Multiple(values) => {
                for v in values {
                    if let CssValue::Keyword(s) = v {
                        if s.eq_ignore_ascii_case("solid") {
                            dec.style = Some(TextDecorationStyle::Solid);
                        } else if s.eq_ignore_ascii_case("double") {
                            dec.style = Some(TextDecorationStyle::Double);
                        } else if s.eq_ignore_ascii_case("dotted") {
                            dec.style = Some(TextDecorationStyle::Dotted);
                        } else if s.eq_ignore_ascii_case("dashed") {
                            dec.style = Some(TextDecorationStyle::Dashed);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    dec
}

/// Helper to recursively check if a node or any of its DOM ancestors is an anchor `<a>` element.
/// spec: S-82
fn is_inside_link(dom: &Dom, node_id: NodeId) -> bool {
    let mut current = Some(node_id);
    let mut depth = 0;
    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(NodeData::Element { name, .. }) = dom.data(curr_id)
            && name.eq_ignore_ascii_case("a")
        {
            return true;
        }
        current = dom.parent(curr_id);
        depth += 1;
    }
    false
}

/// Helper to resolve the computed text color of a text node, following the DOM tree upwards.
/// If we hit an anchor `<a>` element and haven't found any explicit color yet, we default
/// to a blue-ish color.
/// spec: S-82
fn resolve_text_color(
    dom: &Dom,
    node_id: NodeId,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Color {
    let mut current = Some(node_id);
    let mut depth = 0;
    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(style) = styles.get(&curr_id)
            && let Some(val) = style.get("color")
            && let Some(c) = find_color(val)
        {
            return c;
        }
        if let Some(NodeData::Element { name, .. }) = dom.data(curr_id)
            && name.eq_ignore_ascii_case("a")
        {
            return Color::Rgba(0, 0, 238, 255); // Default link blue-ish
        }
        current = dom.parent(curr_id);
        depth += 1;
    }
    Color::Rgba(0, 0, 0, 255) // Default fallback color is black
}

/// Helper to recursively check and resolve the active text decorations for a node from its DOM ancestors.
/// spec: S-55
fn resolve_text_decorations(
    dom: &Dom,
    node_id: NodeId,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> TextDecorations {
    let mut resolved = TextDecorations::default();
    let is_link = is_inside_link(dom, node_id);

    let mut current = Some(node_id);
    let mut depth = 0;

    let mut underline_blocked = false;
    let mut overline_blocked = false;
    let mut line_through_blocked = false;

    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(style) = styles.get(&curr_id) {
            if let Some(CssValue::Keyword(s)) = style.get("text-decoration")
                && s.eq_ignore_ascii_case("none")
            {
                underline_blocked = true;
                overline_blocked = true;
                line_through_blocked = true;
            }

            let local_dec = get_text_decorations(style);

            if local_dec.underline && !underline_blocked {
                resolved.underline = true;
            }
            if local_dec.overline && !overline_blocked {
                resolved.overline = true;
            }
            if local_dec.line_through && !line_through_blocked {
                resolved.line_through = true;
            }
            if local_dec.color.is_some() && resolved.color.is_none() {
                resolved.color = local_dec.color;
            }
            if local_dec.style.is_some() && resolved.style.is_none() {
                resolved.style = local_dec.style;
            }
        }
        current = dom.parent(curr_id);
        depth += 1;
    }

    if is_link && !underline_blocked {
        resolved.underline = true;
    }

    resolved
}

/// Resolve element opacity from style.
/// spec: <https://www.w3.org/TR/css-color-3/#transparency>
/// TODO(spec): True group/stacking-context opacity (compositing the element subtree as a single group, so overlapping descendants do not double-blend) is NOT implemented — this uses a multiplicative per-element alpha approximation.
fn get_opacity(style: &ComputedStyle) -> f32 {
    match style.get("opacity") {
        Some(CssValue::Number(v)) => v.clamp(0.0, 1.0),
        Some(CssValue::Length(p, crate::css::values::LengthUnit::Percent)) => {
            (p / 100.0).clamp(0.0, 1.0)
        }
        _ => 1.0,
    }
}

/// Helper to paint a decoration line with a specific style, color, and geometry.
fn paint_decoration_line(
    items: &mut Vec<DisplayItem>,
    style: Option<TextDecorationStyle>,
    x: f32,
    y: f32,
    width: f32,
    color: Color,
) {
    // TODO(spec): text-decoration-style v1 — solid/double/dotted/dashed via repeated SolidRects; wavy is approximated/out-of-scope, dash/dot metrics are fixed heuristics, and shorthand-embedded style keyword parsing is best-effort.
    match style {
        Some(TextDecorationStyle::Solid) | None => {
            items.push(DisplayItem::SolidRect {
                rect: Rect::new(x, y, width, 1.0),
                color,
            });
        }
        Some(TextDecorationStyle::Double) => {
            items.push(DisplayItem::SolidRect {
                rect: Rect::new(x, y, width, 1.0),
                color: color.clone(),
            });
            items.push(DisplayItem::SolidRect {
                rect: Rect::new(x, y + 2.0, width, 1.0),
                color,
            });
        }
        Some(TextDecorationStyle::Dotted) => {
            let dot_width = 1.0;
            let gap = 1.0;
            let mut current_x = x;
            while current_x < x + width {
                let current_dot_width = if current_x + dot_width > x + width {
                    x + width - current_x
                } else {
                    dot_width
                };
                items.push(DisplayItem::SolidRect {
                    rect: Rect::new(current_x, y, current_dot_width, 1.0),
                    color: color.clone(),
                });
                current_x += dot_width + gap;
            }
        }
        Some(TextDecorationStyle::Dashed) => {
            let dash_width = 4.0;
            let gap = 4.0;
            let mut current_x = x;
            while current_x < x + width {
                let current_dash_width = if current_x + dash_width > x + width {
                    x + width - current_x
                } else {
                    dash_width
                };
                items.push(DisplayItem::SolidRect {
                    rect: Rect::new(current_x, y, current_dash_width, 1.0),
                    color: color.clone(),
                });
                current_x += dash_width + gap;
            }
        }
    }
}

/// Scale a color's alpha channel by an effective opacity factor.
/// spec: <https://www.w3.org/TR/css-color-3/#transparency>
fn scale_color_alpha(color: &Color, factor: f32) -> Color {
    match color {
        Color::Rgba(r, g, b, a) => {
            let new_alpha = ((*a as f32) * factor).round();
            let new_alpha_clamped = new_alpha.clamp(0.0, 255.0) as u8;
            Color::Rgba(*r, *g, *b, new_alpha_clamped)
        }
    }
}

/// Builds a display list from the layout tree.
/// spec: S-12
pub fn build_display_list(
    layout: &LayoutBox,
    dom: &Dom,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> DisplayList {
    let mut items = Vec::new();
    let mut stack = vec![(layout, 1.0)];

    // spec: iterative pre-order traversal (no unbounded recursion — I-6)
    while let Some((layout_box, inherited_opacity)) = stack.pop() {
        let mut skip_children = false;

        let mut effective_opacity = inherited_opacity;

        if let Some((node_id, style)) = layout_box
            .node
            .and_then(|id| styles.get(&id).map(|s| (id, s)))
        {
            let own_opacity = get_opacity(style);
            effective_opacity = inherited_opacity * own_opacity;
            // Treat `collapse` the same as `hidden` for this task.
            // TODO(spec): S-12 visibility: collapse differs from hidden for table-row/column content.
            let node_hidden = matches!(
                style.get("visibility"),
                Some(CssValue::Keyword(k)) if k == "hidden" || k == "collapse"
            );

            // Check if this node is a button or input[type=submit] (S-79)
            let mut is_btn_or_submit = false;
            let mut label_text = String::new();
            if let Some(NodeData::Element { name, .. }) = dom.data(node_id) {
                if name.eq_ignore_ascii_case("button") {
                    label_text = dom.text_content(node_id);
                    is_btn_or_submit = true;
                } else if name.eq_ignore_ascii_case("input") {
                    let type_attr = dom.get_attribute(node_id, "type");
                    let is_submit = type_attr.is_some_and(|t| {
                        t.eq_ignore_ascii_case("submit")
                            || t.eq_ignore_ascii_case("button")
                            || t.eq_ignore_ascii_case("reset")
                    });
                    if is_submit {
                        label_text = dom
                            .get_attribute(node_id, "value")
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "Submit".to_string());
                        is_btn_or_submit = true;
                    }
                }
            }

            // Check if this node is a text-like input field (S-94)
            let mut is_text_input = false;
            if !is_btn_or_submit
                && let Some(NodeData::Element { name, .. }) = dom.data(node_id)
                && name.eq_ignore_ascii_case("input")
            {
                let type_attr = dom.get_attribute(node_id, "type");
                match type_attr {
                    None => {
                        is_text_input = true;
                    }
                    Some(t) => {
                        let t_trimmed = t.trim();
                        if t_trimmed.eq_ignore_ascii_case("text")
                            || t_trimmed.eq_ignore_ascii_case("search")
                            || t_trimmed.eq_ignore_ascii_case("email")
                            || t_trimmed.eq_ignore_ascii_case("url")
                            || t_trimmed.eq_ignore_ascii_case("tel")
                            || t_trimmed.eq_ignore_ascii_case("password")
                            || t_trimmed.eq_ignore_ascii_case("number")
                        {
                            is_text_input = true;
                        }
                    }
                }
            }

            let mut btn_label_item = None;

            if is_btn_or_submit {
                skip_children = true;

                let rect = layout_box.rect;
                let x = rect.origin.x;
                let y = rect.origin.y;
                let w = rect.size.width.max(0.0);
                let h = rect.size.height.max(0.0);

                // Center the label text within the button box
                let font = crate::font::BitmapFont::builtin();
                let text_w = font.measure(&label_text) as f32;
                let text_h = font.line_height() as f32;

                let text_x = x + ((w - text_w) / 2.0).max(0.0);
                let text_y = y + ((h - text_h) / 2.0).max(0.0);

                let text_color = match style.get("color") {
                    Some(CssValue::Color(color)) => color.clone(),
                    _ => Color::Rgba(0, 0, 0, 255), // default black label
                };

                let corrected_rect = Rect::new(text_x, text_y, text_w, text_h);
                btn_label_item = Some(DisplayItem::Text {
                    rect: corrected_rect,
                    text: label_text,
                    color: scale_color_alpha(&text_color, effective_opacity),
                });
            } else if is_text_input {
                skip_children = true;
                // TODO(spec): render input value/placeholder/caret
            }

            if !node_hidden {
                // Paint box-shadow if present
                if let Some(box_shadow_val) = style.get("box-shadow") {
                    // Flatten values to check for none, inset, or comma
                    let mut leaves = Vec::new();
                    flatten_value(box_shadow_val, &mut leaves);

                    let mut has_inset = false;
                    let mut has_none = false;
                    let mut has_comma = false;
                    for leaf in &leaves {
                        if let CssValue::Keyword(kw) = leaf {
                            let kw_lower = kw.to_ascii_lowercase();
                            if kw_lower == "inset" {
                                has_inset = true;
                            } else if kw_lower == "none" {
                                has_none = true;
                            } else if kw_lower == "," {
                                has_comma = true;
                            }
                        }
                    }

                    if leaves.is_empty() || has_none || has_inset || has_comma {
                        // TODO(spec): box-shadow v2 — single outer shadow with spread (blur is still unimplemented; inset & multiple shadows out of scope).
                    } else {
                        // Parse lengths and colors
                        let mut length_values = Vec::new();
                        let mut color_value = None;

                        for leaf in &leaves {
                            match leaf {
                                CssValue::Length(v, _) => {
                                    length_values.push(*v);
                                }
                                CssValue::Number(v) => {
                                    length_values.push(*v);
                                }
                                CssValue::Color(c) => {
                                    color_value = Some(c.clone());
                                }
                                _ => {
                                    if let Some(c) = find_color(leaf) {
                                        color_value = Some(c);
                                    }
                                }
                            }
                        }

                        if length_values.len() >= 2 {
                            let offset_x = length_values[0];
                            let offset_y = length_values[1];
                            let spread = if length_values.len() >= 4 {
                                length_values[3]
                            } else {
                                0.0
                            };

                            // Determine shadow color: default to text color, falling back to black
                            let shadow_color = if let Some(c) = color_value {
                                c
                            } else if let Some(val) = style.get("color")
                                && let Some(c) = find_color(val)
                            {
                                c
                            } else {
                                Color::Rgba(0, 0, 0, 255)
                            };

                            // Only paint shadow if the border box is valid (has positive dimensions)
                            if layout_box.rect.size.width > 0.0 && layout_box.rect.size.height > 0.0
                            {
                                // Translate and inflate/deflate by spread
                                let mut shadow_rect = layout_box.rect;
                                shadow_rect.origin.x += offset_x - spread;
                                shadow_rect.origin.y += offset_y - spread;
                                shadow_rect.size.width += 2.0 * spread;
                                shadow_rect.size.height += 2.0 * spread;

                                // Only paint if resulting dimensions are positive (fully shrunk shadow is invisible)
                                if shadow_rect.size.width > 0.0 && shadow_rect.size.height > 0.0 {
                                    items.push(DisplayItem::SolidRect {
                                        rect: shadow_rect,
                                        color: scale_color_alpha(&shadow_color, effective_opacity),
                                    });
                                }
                            }
                            // TODO(spec): box-shadow v2 — single outer shadow with spread (blur is still unimplemented; inset & multiple shadows out of scope).
                        }
                    }
                }

                // spec: if node has background-color -> SolidRect
                if let Some(CssValue::Color(color)) = style.get("background-color") {
                    // B-4: do not paint background for zero/negative-area boxes
                    if layout_box.rect.size.width > 0.0 && layout_box.rect.size.height > 0.0 {
                        // TODO(spec): border/images/gradients/rasterization
                        items.push(DisplayItem::SolidRect {
                            rect: layout_box.rect,
                            color: scale_color_alpha(color, effective_opacity),
                        });
                    }
                }

                // spec: S-39: read computed border widths
                let border_top = get_border_width(style, "border-top-width");
                let border_right = get_border_width(style, "border-right-width");
                let border_bottom = get_border_width(style, "border-bottom-width");
                let border_left = get_border_width(style, "border-left-width");

                // If any border has a non-zero width, emit SolidRects for the active edges
                if border_top > 0.0
                    || border_right > 0.0
                    || border_bottom > 0.0
                    || border_left > 0.0
                {
                    let rect = layout_box.rect;
                    let x = rect.origin.x;
                    let y = rect.origin.y;
                    let w = rect.size.width.max(0.0);
                    let h = rect.size.height.max(0.0);

                    // Clip/no-panic on weird sizes (I-6) by clamping border widths to fit within box dimensions
                    let t = border_top.max(0.0).min(h);
                    let b = border_bottom.max(0.0).min(h - t);
                    let l = border_left.max(0.0).min(w);
                    let r = border_right.max(0.0).min(w - l);

                    let border_color = get_border_color(style);
                    let mut top_color = get_edge_color(style, "border-top-color", &border_color);
                    let mut right_color =
                        get_edge_color(style, "border-right-color", &border_color);
                    let mut bottom_color =
                        get_edge_color(style, "border-bottom-color", &border_color);
                    let mut left_color = get_edge_color(style, "border-left-color", &border_color);

                    if is_btn_or_submit
                        && style.get("border-top-color").is_none()
                        && style.get("border-right-color").is_none()
                        && style.get("border-bottom-color").is_none()
                        && style.get("border-left-color").is_none()
                        && border_color == Color::Rgba(192, 192, 192, 255)
                    {
                        top_color = Color::Rgba(240, 240, 240, 255); // light highlight
                        left_color = Color::Rgba(240, 240, 240, 255); // light highlight
                        bottom_color = Color::Rgba(140, 140, 140, 255); // dark shadow
                        right_color = Color::Rgba(140, 140, 140, 255); // dark shadow
                    }

                    // Emit 4 border edges as SolidRects, ensuring no empty or negative rects are painted
                    // Top border strip
                    if t > 0.0 && w > 0.0 {
                        items.push(DisplayItem::SolidRect {
                            rect: Rect::new(x, y, w, t),
                            color: scale_color_alpha(&top_color, effective_opacity),
                        });
                    }
                    // Bottom border strip
                    if b > 0.0 && w > 0.0 {
                        items.push(DisplayItem::SolidRect {
                            rect: Rect::new(x, y + h - b, w, b),
                            color: scale_color_alpha(&bottom_color, effective_opacity),
                        });
                    }
                    // Left border strip
                    if l > 0.0 && h - t - b > 0.0 {
                        items.push(DisplayItem::SolidRect {
                            rect: Rect::new(x, y + t, l, h - t - b),
                            color: scale_color_alpha(&left_color, effective_opacity),
                        });
                    }
                    // Right border strip
                    if r > 0.0 && h - t - b > 0.0 {
                        items.push(DisplayItem::SolidRect {
                            rect: Rect::new(x + w - r, y + t, r, h - t - b),
                            color: scale_color_alpha(&right_color, effective_opacity),
                        });
                    }
                }

                // Paint outline if style is present and not none
                let outline_style = style.get("outline-style");
                let has_outline = matches!(outline_style, Some(CssValue::Keyword(s)) if !s.eq_ignore_ascii_case("none"));

                if has_outline {
                    let ow = get_outline_width(style);
                    if ow > 0.0 {
                        let offset = get_outline_offset(style);
                        let d = offset + ow;
                        let rect = layout_box.rect;
                        let x = rect.origin.x;
                        let y = rect.origin.y;
                        let w = rect.size.width.max(0.0);
                        let h = rect.size.height.max(0.0);

                        // TODO(spec): outline non-solid styles and color:invert are not implemented (solid frame, color falls back to text color).

                        let mut outline_color = Color::Rgba(0, 0, 0, 255);
                        let mut resolved = false;

                        if let Some(val) = style.get("outline-color")
                            && let Some(c) = find_color(val)
                        {
                            outline_color = c;
                            resolved = true;
                        }

                        if !resolved {
                            if let Some(val) = style.get("color")
                                && let Some(c) = find_color(val)
                            {
                                outline_color = c;
                            } else {
                                outline_color = Color::Rgba(0, 0, 0, 255);
                            }
                        }

                        let scaled_color = scale_color_alpha(&outline_color, effective_opacity);

                        // Top strip
                        let top_rect = Rect::new(x - d, y - d, w + 2.0 * d, ow);
                        if top_rect.size.width > 0.0 && top_rect.size.height > 0.0 {
                            items.push(DisplayItem::SolidRect {
                                rect: top_rect,
                                color: scaled_color.clone(),
                            });
                        }

                        // Bottom strip
                        let bottom_rect = Rect::new(x - d, y + h + offset, w + 2.0 * d, ow);
                        if bottom_rect.size.width > 0.0 && bottom_rect.size.height > 0.0 {
                            items.push(DisplayItem::SolidRect {
                                rect: bottom_rect,
                                color: scaled_color.clone(),
                            });
                        }

                        // Left strip
                        let left_rect = Rect::new(x - d, y - offset, ow, h + 2.0 * offset);
                        if left_rect.size.width > 0.0 && left_rect.size.height > 0.0 {
                            items.push(DisplayItem::SolidRect {
                                rect: left_rect,
                                color: scaled_color.clone(),
                            });
                        }

                        // Right strip
                        let right_rect =
                            Rect::new(x + w + offset, y - offset, ow, h + 2.0 * offset);
                        if right_rect.size.width > 0.0 && right_rect.size.height > 0.0 {
                            items.push(DisplayItem::SolidRect {
                                rect: right_rect,
                                color: scaled_color.clone(),
                            });
                        }
                    }
                }

                // spec: if node is a Text node -> Text item
                if let Some(NodeData::Text(text)) = dom.data(node_id) {
                    // spec: S-82: reliably resolve text color, defaulting to blue-ish for links
                    let color = resolve_text_color(dom, node_id, styles);

                    // spec: S-82: Correct text baseline vertical y positioning within line height.
                    // Center-align 8px font glyphs vertically inside the line box height.
                    let font = crate::font::BitmapFont::builtin();
                    let font_height = font.line_height() as f32;
                    let height = layout_box.rect.size.height;
                    let dy = ((height - font_height) / 2.0).max(0.0);

                    let corrected_rect = Rect::new(
                        layout_box.rect.origin.x,
                        layout_box.rect.origin.y + dy,
                        layout_box.rect.size.width,
                        font_height,
                    );

                    let display_text = match &layout_box.text {
                        Some(fragment) => fragment.clone(),
                        None => text.clone(),
                    };

                    items.push(DisplayItem::Text {
                        rect: corrected_rect,
                        text: display_text,
                        color: scale_color_alpha(&color, effective_opacity),
                    });

                    // spec: S-82: if computed text-decorations are present (underline, overline, line-through)
                    // or we are inside an <a> (and not explicitly styled text-decoration: none),
                    // emit the corresponding SolidRects.
                    let decorations = resolve_text_decorations(dom, node_id, styles);

                    let x = corrected_rect.origin.x;
                    let width = corrected_rect.size.width;

                    let deco_color = decorations.color.unwrap_or(color.clone());

                    let scaled_deco_color = scale_color_alpha(&deco_color, effective_opacity);

                    if decorations.underline {
                        // Position underline relative to the baseline-corrected text position
                        let underline_y = corrected_rect.origin.y + font_height - 1.0;
                        paint_decoration_line(
                            &mut items,
                            decorations.style,
                            x,
                            underline_y,
                            width,
                            scaled_deco_color.clone(),
                        );
                    }

                    if decorations.overline {
                        // Position overline at the top of the baseline-corrected text position
                        let overline_y = corrected_rect.origin.y;
                        paint_decoration_line(
                            &mut items,
                            decorations.style,
                            x,
                            overline_y,
                            width,
                            scaled_deco_color.clone(),
                        );
                    }

                    if decorations.line_through {
                        // Position line-through in the middle of the baseline-corrected text position
                        let line_through_y = corrected_rect.origin.y + (font_height / 2.0);
                        paint_decoration_line(
                            &mut items,
                            decorations.style,
                            x,
                            line_through_y,
                            width,
                            scaled_deco_color.clone(),
                        );
                    }
                }

                // spec: if node is an img Element node -> Image item (S-58)
                if let Some(NodeData::Element { name, .. }) = dom.data(node_id)
                    && name == "img"
                    && let Some(src) = dom.get_attribute(node_id, "src")
                {
                    // Scan DOM for a <base href="..."> tag
                    let mut base_url = None;
                    for n_id in dom.descendants(dom.document()) {
                        if let Some(NodeData::Element { name: el_name, .. }) = dom.data(n_id)
                            && el_name.eq_ignore_ascii_case("base")
                            && let Some(href) = dom.get_attribute(n_id, "href")
                        {
                            base_url = Some(href.to_string());
                            break;
                        }
                    }

                    let base_url_parsed = base_url
                        .as_ref()
                        .and_then(|b| crate::url::Url::parse(b).ok());

                    if let Some(pre_decoded) = dom.get_image(src) {
                        items.push(DisplayItem::Image {
                            rect: layout_box.rect,
                            src: src.to_string(),
                            base_url,
                            decoded: Some(pre_decoded),
                        });
                    } else {
                        let mut painted_as_pixels = false;
                        if let Some(bytes) =
                            crate::loader::load_image_safely(src, base_url_parsed.as_ref())
                            && let Some(decoded) = crate::image::decode_png(&bytes)
                        {
                            let rect_w = layout_box.rect.size.width;
                            let rect_h = layout_box.rect.size.height;
                            if rect_w > 0.0
                                && rect_h > 0.0
                                && decoded.width > 0
                                && decoded.height > 0
                            {
                                painted_as_pixels = true;
                                let sub_w = rect_w / decoded.width as f32;
                                let sub_h = rect_h / decoded.height as f32;
                                for y in 0..decoded.height {
                                    for x in 0..decoded.width {
                                        let idx = ((y * decoded.width + x) * 4) as usize;
                                        if idx + 3 < decoded.rgba.len() {
                                            let r = decoded.rgba[idx];
                                            let g = decoded.rgba[idx + 1];
                                            let b = decoded.rgba[idx + 2];
                                            let a = decoded.rgba[idx + 3];
                                            let color = Color::Rgba(r, g, b, a);
                                            let sub_rect = Rect::new(
                                                layout_box.rect.origin.x + x as f32 * sub_w,
                                                layout_box.rect.origin.y + y as f32 * sub_h,
                                                sub_w,
                                                sub_h,
                                            );
                                            items.push(DisplayItem::SolidRect {
                                                rect: sub_rect,
                                                color: scale_color_alpha(&color, effective_opacity),
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        if !painted_as_pixels {
                            items.push(DisplayItem::Image {
                                rect: layout_box.rect,
                                src: src.to_string(),
                                base_url,
                                decoded: None,
                            });
                        }
                    }
                }
            }

            if !node_hidden && let Some(item) = btn_label_item {
                items.push(item);
            }
        }

        // Pre-order traversal: process current, then children left-to-right.
        // Since we use a stack (LIFO), we push children in reverse order.
        if !skip_children {
            for child in layout_box.children.iter().rev() {
                stack.push((child, effective_opacity));
            }
        }
    }

    DisplayList(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::parse_stylesheet;
    use crate::dom::{Dom, NodeData};
    use crate::layout::layout_document;
    use crate::style::compute_styles;

    #[test]
    fn test_paint_basic() {
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

        let text = dom.create_node(NodeData::Text("paint me".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div { background-color: #ff0000; color: #0000ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // items should contain:
        // 1. SolidRect for div
        // 2. Text for "paint me"

        let mut found_rect = false;
        let mut text_fragments = Vec::new();

        for item in &items {
            match item {
                DisplayItem::SolidRect { color, .. } => {
                    if *color == Color::Rgba(255, 0, 0, 255) {
                        found_rect = true;
                    }
                }
                DisplayItem::Text { text, color, .. } if *color == Color::Rgba(0, 0, 255, 255) => {
                    text_fragments.push(text.clone());
                }
                _ => {}
            }
        }

        assert!(found_rect, "SolidRect for div not found");
        assert_eq!(text_fragments.concat(), "paint me");
    }

    #[test]
    fn test_paint_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div1);
        dom.append_child(doc, div2);

        let stylesheet = parse_stylesheet(
            "
            div { background-color: blue; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Should have 2 items in order
        assert_eq!(items.len(), 2);
        for item in &items {
            match item {
                DisplayItem::SolidRect { color, .. } => {
                    assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
                }
                _ => panic!("Expected SolidRect"),
            }
        }
    }

    #[test]
    fn test_paint_border() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
                border-right-width: 3px;
                border-bottom-width: 4px;
                border-left-width: 5px;
                border-color: #00ff00;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(rects.len(), 4);

        let mut top_found = false;
        let mut bottom_found = false;
        let mut left_found = false;
        let mut right_found = false;

        let green = Color::Rgba(0, 255, 0, 255);

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &green);
                if rect.origin.x == 0.0
                    && rect.origin.y == 0.0
                    && rect.size.width == 108.0
                    && rect.size.height == 2.0
                {
                    top_found = true;
                } else if rect.origin.x == 0.0
                    && rect.origin.y == 102.0
                    && rect.size.width == 108.0
                    && rect.size.height == 4.0
                {
                    bottom_found = true;
                } else if rect.origin.x == 0.0
                    && rect.origin.y == 2.0
                    && rect.size.width == 5.0
                    && rect.size.height == 100.0
                {
                    left_found = true;
                } else if rect.origin.x == 105.0
                    && rect.origin.y == 2.0
                    && rect.size.width == 3.0
                    && rect.size.height == 100.0
                {
                    right_found = true;
                }
            }
        }

        assert!(top_found, "Top border rect mismatch");
        assert!(bottom_found, "Bottom border rect mismatch");
        assert!(left_found, "Left border rect mismatch");
        assert!(right_found, "Right border rect mismatch");
    }

    #[test]
    fn test_paint_outline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                outline: 2px solid red;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(rects.len(), 4, "Expected exactly 4 outline strips");

        let red = Color::Rgba(255, 0, 0, 255);

        let mut top_found = false;
        let mut bottom_found = false;
        let mut left_found = false;
        let mut right_found = false;

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &red);
                if rect.origin.x == -2.0
                    && rect.origin.y == -2.0
                    && rect.size.width == 104.0
                    && rect.size.height == 2.0
                {
                    top_found = true;
                } else if rect.origin.x == -2.0
                    && rect.origin.y == 100.0
                    && rect.size.width == 104.0
                    && rect.size.height == 2.0
                {
                    bottom_found = true;
                } else if rect.origin.x == -2.0
                    && rect.origin.y == 0.0
                    && rect.size.width == 2.0
                    && rect.size.height == 100.0
                {
                    left_found = true;
                } else if rect.origin.x == 100.0
                    && rect.origin.y == 0.0
                    && rect.size.width == 2.0
                    && rect.size.height == 100.0
                {
                    right_found = true;
                }
            }
        }

        assert!(top_found, "Top outline rect mismatch");
        assert!(bottom_found, "Bottom outline rect mismatch");
        assert!(left_found, "Left outline rect mismatch");
        assert!(right_found, "Right outline rect mismatch");

        // Assert outline-style: none produces NO such outside rect
        let stylesheet_none = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                outline: 2px none red;
            }
        ",
        );
        let styles_none = compute_styles(&dom, &stylesheet_none);
        let layout_none = layout_document(&dom, &styles_none, 800.0);
        let display_list_none = build_display_list(&layout_none, &dom, &styles_none);
        let rects_none: Vec<&DisplayItem> = display_list_none
            .0
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(
            rects_none.len(),
            0,
            "Expected no outline rects with outline-style: none"
        );
    }

    #[test]
    fn test_paint_outline_with_positive_offset() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                outline: 2px solid red;
                outline-offset: 4px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(rects.len(), 4, "Expected exactly 4 outline strips");

        let red = Color::Rgba(255, 0, 0, 255);

        let mut top_found = false;
        let mut bottom_found = false;
        let mut left_found = false;
        let mut right_found = false;

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &red);
                if rect.origin.x == -6.0
                    && rect.origin.y == -6.0
                    && rect.size.width == 112.0
                    && rect.size.height == 2.0
                {
                    top_found = true;
                } else if rect.origin.x == -6.0
                    && rect.origin.y == 104.0
                    && rect.size.width == 112.0
                    && rect.size.height == 2.0
                {
                    bottom_found = true;
                } else if rect.origin.x == -6.0
                    && rect.origin.y == -4.0
                    && rect.size.width == 2.0
                    && rect.size.height == 108.0
                {
                    left_found = true;
                } else if rect.origin.x == 104.0
                    && rect.origin.y == -4.0
                    && rect.size.width == 2.0
                    && rect.size.height == 108.0
                {
                    right_found = true;
                }
            }
        }

        assert!(top_found, "Top outline rect mismatch with positive offset");
        assert!(
            bottom_found,
            "Bottom outline rect mismatch with positive offset"
        );
        assert!(
            left_found,
            "Left outline rect mismatch with positive offset"
        );
        assert!(
            right_found,
            "Right outline rect mismatch with positive offset"
        );
    }

    #[test]
    fn test_paint_outline_with_negative_offset() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let stylesheet = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                outline: 2px solid red;
                outline-offset: -3px;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(rects.len(), 4, "Expected exactly 4 outline strips");

        let red = Color::Rgba(255, 0, 0, 255);

        let mut top_found = false;
        let mut bottom_found = false;
        let mut left_found = false;
        let mut right_found = false;

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &red);
                if rect.origin.x == 1.0
                    && rect.origin.y == 1.0
                    && rect.size.width == 98.0
                    && rect.size.height == 2.0
                {
                    top_found = true;
                } else if rect.origin.x == 1.0
                    && rect.origin.y == 97.0
                    && rect.size.width == 98.0
                    && rect.size.height == 2.0
                {
                    bottom_found = true;
                } else if rect.origin.x == 1.0
                    && rect.origin.y == 3.0
                    && rect.size.width == 2.0
                    && rect.size.height == 94.0
                {
                    left_found = true;
                } else if rect.origin.x == 97.0
                    && rect.origin.y == 3.0
                    && rect.size.width == 2.0
                    && rect.size.height == 94.0
                {
                    right_found = true;
                }
            }
        }

        assert!(top_found, "Top outline rect mismatch with negative offset");
        assert!(
            bottom_found,
            "Bottom outline rect mismatch with negative offset"
        );
        assert!(
            left_found,
            "Left outline rect mismatch with negative offset"
        );
        assert!(
            right_found,
            "Right outline rect mismatch with negative offset"
        );
    }

    #[test]
    fn test_paint_border_fallback() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Scenario A: border-color is absent, but text "color" is specified -> should use text color
        let stylesheet_a = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
                color: #ff00ff;
            }
        ",
        );
        let styles_a = compute_styles(&dom, &stylesheet_a);
        let layout_a = layout_document(&dom, &styles_a, 800.0);
        let display_list_a = build_display_list(&layout_a, &dom, &styles_a);
        let items_a = display_list_a.0;

        let rects_a: Vec<&DisplayItem> = items_a
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();
        assert_eq!(rects_a.len(), 1);
        if let DisplayItem::SolidRect { color, .. } = rects_a[0] {
            assert_eq!(color, &Color::Rgba(255, 0, 255, 255));
        } else {
            panic!("Expected SolidRect");
        }

        // Scenario B: border-color and color are both absent -> should default to black
        let stylesheet_b = parse_stylesheet(
            "
            div {
                width: 100px;
                height: 100px;
                border-top-width: 2px;
            }
        ",
        );
        let styles_b = compute_styles(&dom, &stylesheet_b);
        let layout_b = layout_document(&dom, &styles_b, 800.0);
        let display_list_b = build_display_list(&layout_b, &dom, &styles_b);
        let items_b = display_list_b.0;

        let rects_b: Vec<&DisplayItem> = items_b
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();
        assert_eq!(rects_b.len(), 1);
        if let DisplayItem::SolidRect { color, .. } = rects_b[0] {
            assert_eq!(color, &Color::Rgba(0, 0, 0, 255));
        } else {
            panic!("Expected SolidRect");
        }
    }

    #[test]
    fn test_paint_border_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let text = dom.create_node(NodeData::Text("paint me".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div {
                background-color: #ff0000;
                border-top-width: 2px;
                color: #0000ff;
            }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // The expected order of display list items:
        // 1. SolidRect (background-color: red)
        // 2. SolidRect (border-color: text color blue, because border-color is absent)
        // 3. Text ("paint ", color: blue) - fragment 1
        // 4. Text ("me", color: blue) - fragment 2

        assert_eq!(items.len(), 4);

        // 1. Background
        if let DisplayItem::SolidRect { color, .. } = &items[0] {
            assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
        } else {
            panic!("Expected background SolidRect first");
        }

        // 2. Border
        if let DisplayItem::SolidRect { color, .. } = &items[1] {
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected border SolidRect second");
        }

        // 3. Text - Fragment 1
        if let DisplayItem::Text { text, color, .. } = &items[2] {
            assert_eq!(text, "paint ");
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected Text item third");
        }

        // 4. Text - Fragment 2
        if let DisplayItem::Text { text, color, .. } = &items[3] {
            assert_eq!(text, "me");
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected Text item fourth");
        }
    }

    #[test]
    fn test_paint_text_color() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("colored text".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { color: #00ff00; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert!(!text_items.is_empty(), "Expected at least one Text item");
        for item in text_items {
            if let DisplayItem::Text { color, .. } = item {
                assert_eq!(color, &Color::Rgba(0, 255, 0, 255));
            }
        }
    }

    #[test]
    fn test_paint_text_decoration_color() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("colored underline".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline; text-decoration-color: #00ff00; color: #ff0000; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert!(!text_items.is_empty(), "Expected at least one Text item");
        assert!(
            !solid_rects.is_empty(),
            "Expected at least one SolidRect item"
        );

        for item in text_items {
            if let DisplayItem::Text { color, .. } = item {
                // Glyph color must be red (#ff0000)
                assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
            } else {
                panic!("Expected Text");
            }
        }

        for item in solid_rects {
            if let DisplayItem::SolidRect { color, .. } = item {
                // Underline color must be green (#00ff00)
                assert_eq!(color, &Color::Rgba(0, 255, 0, 255));
            } else {
                panic!("Expected SolidRect");
            }
        }
    }

    #[test]
    fn test_paint_text_underline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("underlined".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline; color: #ff00ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        // Since "underlined" collapses to one word fragment, there should be 1 Text item and 1 underline SolidRect
        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            rect: text_rect,
            color: text_color,
            ..
        } = text_items[0]
        {
            assert_eq!(text_color, &Color::Rgba(255, 0, 255, 255));
            if let DisplayItem::SolidRect {
                rect: rect_rect,
                color: rect_color,
            } = solid_rects[0]
            {
                assert_eq!(rect_color, &Color::Rgba(255, 0, 255, 255));
                assert_eq!(rect_rect.origin.x, text_rect.origin.x);
                assert_eq!(rect_rect.size.width, text_rect.size.width);
                assert_eq!(rect_rect.size.height, 1.0);
                assert!(rect_rect.origin.y > text_rect.origin.y);
            } else {
                panic!("Expected SolidRect");
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_underline_ancestor() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        let text = dom.create_node(NodeData::Text("nested".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            div { text-decoration: underline; color: #ff0000; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            color: text_color, ..
        } = text_items[0]
        {
            assert_eq!(text_color, &Color::Rgba(255, 0, 0, 255));
        }
        if let DisplayItem::SolidRect {
            color: rect_color, ..
        } = solid_rects[0]
        {
            assert_eq!(rect_color, &Color::Rgba(255, 0, 0, 255));
        }
    }

    #[test]
    fn test_paint_text_decoration_style_dotted() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("dotted_underlined".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline; text-decoration-style: dotted; color: #ff00ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        // "dotted_underlined" spans some width, so we expect multiple dotted SolidRects
        assert!(solid_rects.len() > 1);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            // All dots must be at the underline's y and within the text's horizontal span
            for item in &solid_rects {
                if let DisplayItem::SolidRect { rect, color } = item {
                    assert_eq!(color, &Color::Rgba(255, 0, 255, 255));
                    assert!(rect.origin.x >= text_rect.origin.x);
                    assert!(
                        rect.origin.x + rect.size.width
                            <= text_rect.origin.x + text_rect.size.width
                    );
                    // Dotted line height is 1.0, dot width is at most 1.0
                    assert!(rect.size.width <= 1.0);
                    assert_eq!(rect.size.height, 1.0);
                } else {
                    panic!("Expected SolidRect");
                }
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_decoration_style_dashed_shorthand() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("dashed_underlined".into()));
        dom.append_child(p, text);

        // Test shorthand-embedded parsing!
        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline dashed; color: #00ff00; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert!(solid_rects.len() > 1);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            for item in &solid_rects {
                if let DisplayItem::SolidRect { rect, color } = item {
                    assert_eq!(color, &Color::Rgba(0, 255, 0, 255));
                    assert!(rect.origin.x >= text_rect.origin.x);
                    assert!(
                        rect.origin.x + rect.size.width
                            <= text_rect.origin.x + text_rect.size.width
                    );
                    assert_eq!(rect.size.height, 1.0);
                } else {
                    panic!("Expected SolidRect");
                }
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_decoration_style_double() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("double_underlined".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            p { text-decoration: underline; text-decoration-style: double; color: #0000ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        // Double style emits exactly TWO parallel line rectangles
        assert_eq!(solid_rects.len(), 2);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            let r1 = match solid_rects[0] {
                DisplayItem::SolidRect { rect, color } => {
                    assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
                    assert_eq!(rect.origin.x, text_rect.origin.x);
                    assert_eq!(rect.size.width, text_rect.size.width);
                    assert_eq!(rect.size.height, 1.0);
                    rect
                }
                _ => panic!("Expected SolidRect"),
            };

            let r2 = match solid_rects[1] {
                DisplayItem::SolidRect { rect, color } => {
                    assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
                    assert_eq!(rect.origin.x, text_rect.origin.x);
                    assert_eq!(rect.size.width, text_rect.size.width);
                    assert_eq!(rect.size.height, 1.0);
                    rect
                }
                _ => panic!("Expected SolidRect"),
            };

            // Two parallel lines must be at distinct y offsets
            assert_ne!(r1.origin.y, r2.origin.y);
            assert_eq!(r2.origin.y, r1.origin.y + 2.0);
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_image() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), "test.png".into())],
        });
        dom.append_child(doc, img);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(image_items.len(), 1);
        if let DisplayItem::Image { src, .. } = image_items[0] {
            assert_eq!(src, "test.png");
        } else {
            panic!("Expected DisplayItem::Image");
        }
    }

    #[test]
    fn test_paint_image_decoded_blit() {
        use crate::raster::Canvas;

        // 1. Generate 2x2 image
        let mut source_canvas = Canvas::new(2, 2);
        // 0xAARRGGBB
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        source_canvas.pixels[2] = 0xFF0000FF; // Blue
        source_canvas.pixels[3] = 0xFFFFFF00; // Yellow
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_paint_test_image.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), temp_filename.into())],
        });
        dom.append_child(doc, img);

        let stylesheet = parse_stylesheet("img { width: 4px; height: 4px; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Clean up the temp file
        let _ = std::fs::remove_file(temp_filename);

        // We expect exactly 4 SolidRect items (representing the 2x2 pixels of the image)
        // rather than 1 DisplayItem::Image item!
        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(
            solid_rects.len(),
            4,
            "Expected 4 SolidRects representing decoded PNG pixels"
        );

        // Let's verify the positions and colors of the SolidRects
        // Because the image has width 4px and height 4px, sub_w = 2.0, sub_h = 2.0
        // Rects should be at (0,0), (2,0), (0,2), (2,2) with size 2x2
        let mut found_red = false;
        let mut found_green = false;
        let mut found_blue = false;
        let mut found_yellow = false;

        for item in solid_rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(rect.size.width, 2.0);
                assert_eq!(rect.size.height, 2.0);
                if rect.origin.x == 0.0 && rect.origin.y == 0.0 {
                    assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
                    found_red = true;
                } else if rect.origin.x == 2.0 && rect.origin.y == 0.0 {
                    assert_eq!(color, &Color::Rgba(0, 255, 0, 255));
                    found_green = true;
                } else if rect.origin.x == 0.0 && rect.origin.y == 2.0 {
                    assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
                    found_blue = true;
                } else if rect.origin.x == 2.0 && rect.origin.y == 2.0 {
                    assert_eq!(color, &Color::Rgba(255, 255, 0, 255));
                    found_yellow = true;
                }
            }
        }

        assert!(found_red, "Red pixel not found or incorrect");
        assert!(found_green, "Green pixel not found or incorrect");
        assert!(found_blue, "Blue pixel not found or incorrect");
        assert!(found_yellow, "Yellow pixel not found or incorrect");
    }

    #[test]
    fn test_paint_link_decoration() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![("href".into(), "https://example.com".into())],
        });
        dom.append_child(doc, a);

        let text = dom.create_node(NodeData::Text("link".into()));
        dom.append_child(a, text);

        let stylesheet = parse_stylesheet("");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        // Default link gets a default blue-ish color: Color::Rgba(0, 0, 238, 255)
        if let DisplayItem::Text { color, .. } = text_items[0] {
            assert_eq!(color, &Color::Rgba(0, 0, 238, 255));
        }

        // And it should get an underline solid rect
        assert_eq!(solid_rects.len(), 1);
        if let DisplayItem::SolidRect { color, .. } = solid_rects[0] {
            assert_eq!(color, &Color::Rgba(0, 0, 238, 255));
        }
    }

    #[test]
    fn test_paint_link_explicit_color() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        dom.append_child(doc, a);

        let text = dom.create_node(NodeData::Text("colored".into()));
        dom.append_child(a, text);

        // Explicitly override link color
        let stylesheet = parse_stylesheet("a { color: #ff0000; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        if let DisplayItem::Text { color, .. } = text_items[0] {
            assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
        }
    }

    #[test]
    fn test_paint_link_explicit_no_underline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let a = dom.create_node(NodeData::Element {
            name: "a".into(),
            attrs: vec![],
        });
        dom.append_child(doc, a);

        let text = dom.create_node(NodeData::Text("nodecoration".into()));
        dom.append_child(a, text);

        // Explicitly set text-decoration: none
        let stylesheet = parse_stylesheet("a { text-decoration: none; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        // Underline should be disabled
        assert_eq!(solid_rects.len(), 0);
    }

    #[test]
    fn test_paint_text_baseline_correction() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("tall".into()));
        dom.append_child(p, text);

        // Explicitly set a tall line height.
        // Pt unit resolves to 16.0px (e.g. 12pt).
        let stylesheet = parse_stylesheet("p { line-height: 12pt; text-decoration: underline; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            // Font height is 8.0px
            assert_eq!(text_rect.size.height, 8.0);

            // Layout box height is 16.0px (12pt * 96 / 72 = 16.0px)
            // dy = (16.0 - 8.0) / 2.0 = 4.0
            // The text rect origin.y should be shifted by dy (4.0) relative to layout_box.rect.origin.y
            // Let's assert the relationship
            if let DisplayItem::SolidRect {
                rect: rect_rect, ..
            } = solid_rects[0]
            {
                // Underline should be at text_rect.origin.y + 7.0
                assert_eq!(rect_rect.origin.y, text_rect.origin.y + 7.0);
            }
        }
    }

    #[test]
    fn test_paint_text_overline() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("overlined".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet("p { text-decoration: overline; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            if let DisplayItem::SolidRect {
                rect: rect_rect, ..
            } = solid_rects[0]
            {
                assert_eq!(rect_rect.origin.y, text_rect.origin.y);
                assert_eq!(rect_rect.size.height, 1.0);
            } else {
                panic!("Expected SolidRect");
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_line_through() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("striked".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet("p { text-decoration: line-through; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 1);

        if let DisplayItem::Text {
            rect: text_rect, ..
        } = text_items[0]
        {
            if let DisplayItem::SolidRect {
                rect: rect_rect, ..
            } = solid_rects[0]
            {
                assert_eq!(rect_rect.origin.y, text_rect.origin.y + 4.0);
                assert_eq!(rect_rect.size.height, 1.0);
            } else {
                panic!("Expected SolidRect");
            }
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_paint_text_multiple_decorations() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(doc, p);

        let text = dom.create_node(NodeData::Text("all".into()));
        dom.append_child(p, text);

        let stylesheet =
            parse_stylesheet("p { text-decoration: underline overline line-through; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(text_items.len(), 1);
        assert_eq!(solid_rects.len(), 3);
    }

    #[test]
    fn test_paint_text_decorations_override() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let p = dom.create_node(NodeData::Element {
            name: "p".into(),
            attrs: vec![],
        });
        dom.append_child(div, p);

        let text = dom.create_node(NodeData::Text("none".into()));
        dom.append_child(p, text);

        let stylesheet = parse_stylesheet(
            "
            div { text-decoration: underline overline; }
            p { text-decoration: none; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        // No underlines or overlines should be drawn for the child with none
        assert_eq!(solid_rects.len(), 0);
    }

    #[test]
    fn test_paint_button_default_ua_chrome() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let btn = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, btn);

        let text = dom.create_node(NodeData::Text("Click me".into()));
        dom.append_child(btn, text);

        let stylesheet = parse_stylesheet(crate::engine::UA_DEFAULT_CSS);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Items should be:
        // 1. Filled background rect (light grey Color::Rgba(239, 239, 239, 255))
        // 2-5. 4 border strips (beveled colors: Rgba(240, 240, 240, 255) and Rgba(140, 140, 140, 255))
        // 6. Centered button label text DisplayItem::Text with black color

        let background_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(239, 239, 239, 255)))
            .collect();

        let border_highlight_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(240, 240, 240, 255)))
            .collect();

        let border_shadow_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(140, 140, 140, 255)))
            .collect();

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            background_items.len(),
            1,
            "Expected 1 default button background"
        );
        assert_eq!(
            border_highlight_items.len(),
            2,
            "Expected 2 top/left border highlights"
        );
        assert_eq!(
            border_shadow_items.len(),
            2,
            "Expected 2 bottom/right border shadows"
        );
        assert_eq!(text_items.len(), 1, "Expected 1 centered button text item");

        if let DisplayItem::Text { text, color, rect } = text_items[0] {
            assert_eq!(text, "Click me");
            assert_eq!(color, &Color::Rgba(0, 0, 0, 255));
            // Assert centered/correct offsets
            assert!(rect.origin.x >= 0.0);
            assert!(rect.origin.y >= 0.0);
        } else {
            panic!("Expected text item");
        }
    }

    #[test]
    fn test_paint_submit_input_default_ua_chrome() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "submit".into()),
                ("value".into(), "Search Here".into()),
            ],
        });
        dom.append_child(doc, input);

        let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
        css.push_str("input { width: 100px; height: 30px; }");
        let stylesheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let background_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(239, 239, 239, 255)))
            .collect();

        let border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(240, 240, 240, 255) || color == &Color::Rgba(140, 140, 140, 255)))
            .collect();

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(background_items.len(), 1);
        assert_eq!(border_items.len(), 4);
        assert_eq!(text_items.len(), 1);

        if let DisplayItem::Text { text, color, .. } = text_items[0] {
            assert_eq!(text, "Search Here");
            assert_eq!(color, &Color::Rgba(0, 0, 0, 255));
        } else {
            panic!("Expected text item");
        }
    }

    #[test]
    fn test_paint_button_custom_style_override() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let btn = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, btn);

        let text = dom.create_node(NodeData::Text("Custom".into()));
        dom.append_child(btn, text);

        let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
        css.push_str("button { background-color: rgb(255, 0, 0); border-color: rgb(0, 255, 0); color: rgb(0, 0, 255); }");
        let stylesheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Custom styling should override defaults
        let custom_bg_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(255, 0, 0, 255)))
            .collect();

        let custom_border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(0, 255, 0, 255)))
            .collect();

        let custom_text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            custom_bg_items.len(),
            1,
            "Expected overridden custom red background"
        );
        assert!(
            !custom_border_items.is_empty(),
            "Expected overridden custom green borders"
        );
        assert_eq!(
            custom_text_items.len(),
            1,
            "Expected centered overridden custom blue label text"
        );

        if let DisplayItem::Text { text, color, .. } = custom_text_items[0] {
            assert_eq!(text, "Custom");
            assert_eq!(color, &Color::Rgba(0, 0, 255, 255));
        } else {
            panic!("Expected text item");
        }
    }

    #[test]
    fn test_paint_text_input_default_ua_chrome() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "text".into())],
        });
        dom.append_child(doc, input);

        let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
        css.push_str("input { width: 100px; height: 30px; }");
        let stylesheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Items should be:
        // 1. Filled white background rect (Color::Rgba(255, 255, 255, 255))
        // 2-5. 4 border strips (default border color #767676 / Color::Rgba(118, 118, 118, 255))

        let background_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(255, 255, 255, 255)))
            .collect();

        let border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(118, 118, 118, 255)))
            .collect();

        assert_eq!(
            background_items.len(),
            1,
            "Expected 1 default white background"
        );
        assert_eq!(border_items.len(), 4, "Expected 4 default border strips");
    }

    #[test]
    fn test_paint_input_no_type_ua_chrome() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![],
        });
        dom.append_child(doc, input);

        let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
        css.push_str("input { width: 100px; height: 30px; }");
        let stylesheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let background_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(255, 255, 255, 255)))
            .collect();

        let border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(118, 118, 118, 255)))
            .collect();

        assert_eq!(
            background_items.len(),
            1,
            "Expected 1 default white background for input without type attribute"
        );
        assert_eq!(
            border_items.len(),
            4,
            "Expected 4 default border strips for input without type attribute"
        );
    }

    #[test]
    fn test_paint_input_submit_stays_on_button_path() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "submit".into()),
                ("value".into(), "Submit".into()),
            ],
        });
        dom.append_child(doc, input);

        let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
        css.push_str("input { width: 100px; height: 30px; }");
        let stylesheet = parse_stylesheet(&css);
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Ensure we do NOT emit white background or #767676 border (which are the text input defaults)
        let text_bg_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(255, 255, 255, 255)))
            .collect();

        let text_border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(118, 118, 118, 255)))
            .collect();

        assert_eq!(text_bg_items.len(), 0);
        assert_eq!(text_border_items.len(), 0);

        // Ensure it emits default button gray background and highlights/shadows
        let button_bg_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(239, 239, 239, 255)))
            .collect();

        assert_eq!(button_bg_items.len(), 1);
    }

    #[test]
    fn test_paint_input_custom_style_override() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![("type".into(), "text".into())],
        });
        dom.append_child(doc, input);

        let stylesheet = parse_stylesheet(
            "input { width: 100px; height: 30px; background-color: rgb(255, 0, 0); border-color: rgb(0, 255, 0); border-width: 2px; }",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 100.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Overridden background-color: rgb(255, 0, 0)
        let custom_bg_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(255, 0, 0, 255)))
            .collect();

        // Overridden border-color: rgb(0, 255, 0)
        let custom_border_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(0, 255, 0, 255)))
            .collect();

        assert_eq!(
            custom_bg_items.len(),
            1,
            "Expected overridden custom red background"
        );
        assert_eq!(
            custom_border_items.len(),
            4,
            "Expected overridden custom green borders"
        );
    }

    /// Guards the layout->paint text-fragment contract.
    /// Ensures each inline word is painted once at its own x, preventing full-node-text duplication per word.
    #[test]
    fn test_inline_text_rendering_contract() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        let text = dom.create_node(NodeData::Text("ab cd".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div { color: #0000ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let mut text_items: Vec<(f32, String)> = display_list
            .0
            .into_iter()
            .filter_map(|item| {
                if let DisplayItem::Text { rect, text, .. } = item {
                    Some((rect.origin.x, text))
                } else {
                    None
                }
            })
            .collect();

        // Order by their rect origin x
        text_items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Assert strictly increasing x origins (no two word fragments share the same x)
        for i in 0..text_items.len() - 1 {
            assert!(
                text_items[i].0 < text_items[i + 1].0,
                "x origins must be strictly increasing, but found adjacent coords: {} and {}",
                text_items[i].0,
                text_items[i + 1].0
            );
        }

        // Concatenate their text fields
        let concatenated_text: String = text_items.iter().map(|(_, t)| t.clone()).collect();

        // Assert reconstruction of the original node text exactly once
        assert_eq!(concatenated_text, "ab cd");
        assert_ne!(concatenated_text, "ab cdab cd");
    }

    #[test]
    fn test_ua_form_styling_paint_and_computed_properties() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let btn = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, btn);

        let input = dom.create_node(NodeData::Element {
            name: "input".into(),
            attrs: vec![
                ("type".into(), "submit".into()),
                ("value".into(), "Submit".into()),
            ],
        });
        dom.append_child(doc, input);

        let stylesheet = parse_stylesheet(crate::engine::UA_DEFAULT_CSS);
        let styles = compute_styles(&dom, &stylesheet);

        // (a) a <button> computed style has display:inline-block and a non-empty border-width and a background-color from UA CSS
        let btn_style = styles.get(&btn).expect("button should have computed style");
        assert_eq!(
            btn_style.get("display"),
            Some(&crate::css::values::CssValue::Keyword(
                "inline-block".to_string()
            ))
        );
        assert!(btn_style.get("background-color").is_some());
        assert!(btn_style.get("border-top-width").is_some());

        // (b) the paint display list for a submit input contains exactly one background SolidRect and four border strips, plus the centered label Text item — no duplicate fills.
        let layout = layout_document(&dom, &styles, 800.0);
        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Gather painted items belonging to the submit input box
        let input_box = layout
            .children
            .iter()
            .find(|child| child.node == Some(input))
            .expect("input should have a layout box");
        let input_rect = input_box.rect;

        let input_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| match item {
                DisplayItem::SolidRect { rect, .. } => {
                    rect.origin.x >= input_rect.origin.x
                        && rect.origin.x < input_rect.origin.x + input_rect.size.width
                        && rect.origin.y >= input_rect.origin.y
                        && rect.origin.y < input_rect.origin.y + input_rect.size.height
                }
                DisplayItem::Text { rect, .. } => {
                    rect.origin.x >= input_rect.origin.x
                        && rect.origin.x < input_rect.origin.x + input_rect.size.width
                        && rect.origin.y >= input_rect.origin.y
                        && rect.origin.y < input_rect.origin.y + input_rect.size.height
                }
                _ => false,
            })
            .collect();

        let background_rects: Vec<&&DisplayItem> = input_items.iter().filter(|item| {
            matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(239, 239, 239, 255))
        }).collect();

        let border_rects: Vec<&&DisplayItem> = input_items.iter().filter(|item| {
            matches!(item, DisplayItem::SolidRect { color, .. } if color == &Color::Rgba(240, 240, 240, 255) || color == &Color::Rgba(140, 140, 140, 255))
        }).collect();

        let text_items: Vec<&&DisplayItem> = input_items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            background_rects.len(),
            1,
            "Expected exactly one background SolidRect"
        );
        assert_eq!(border_rects.len(), 4, "Expected exactly four border strips");
        assert_eq!(
            text_items.len(),
            1,
            "Expected exactly one centered label Text item"
        );
    }

    #[test]
    fn test_button_label_z_order() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let btn = dom.create_node(NodeData::Element {
            name: "button".into(),
            attrs: vec![],
        });
        dom.append_child(doc, btn);

        let text = dom.create_node(NodeData::Text("Click me".into()));
        dom.append_child(btn, text);

        let stylesheet = parse_stylesheet(crate::engine::UA_DEFAULT_CSS);
        let styles = compute_styles(&dom, &stylesheet);

        let layout = layout_document(&dom, &styles, 800.0);
        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Find the layout box of the button
        let btn_box = layout
            .children
            .iter()
            .find(|child| child.node == Some(btn))
            .expect("button should have a layout box");
        let btn_rect = btn_box.rect;

        // Find the index of the background SolidRect and the label Text
        let mut bg_index = None;
        let mut text_index = None;

        for (idx, item) in items.iter().enumerate() {
            match item {
                DisplayItem::SolidRect { rect, color }
                    if rect.origin.x >= btn_rect.origin.x
                        && rect.origin.x < btn_rect.origin.x + btn_rect.size.width
                        && rect.origin.y >= btn_rect.origin.y
                        && rect.origin.y < btn_rect.origin.y + btn_rect.size.height
                        && *color == Color::Rgba(239, 239, 239, 255) =>
                {
                    bg_index = Some(idx);
                }
                DisplayItem::Text { rect, text, .. }
                    if rect.origin.x >= btn_rect.origin.x
                        && rect.origin.x < btn_rect.origin.x + btn_rect.size.width
                        && rect.origin.y >= btn_rect.origin.y
                        && rect.origin.y < btn_rect.origin.y + btn_rect.size.height
                        && text == "Click me" =>
                {
                    text_index = Some(idx);
                }
                _ => {}
            }
        }

        let bg_idx = bg_index.expect("Should find button background SolidRect");
        let text_idx = text_index.expect("Should find button label Text");

        assert!(
            text_idx > bg_idx,
            "Label text (index {}) must paint on top of background SolidRect (index {})",
            text_idx,
            bg_idx
        );
    }

    #[test]
    fn test_paint_visibility_hidden() {
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

        let text = dom.create_node(NodeData::Text("hidden text".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div { visibility: hidden; background-color: #ff0000; border: 1px solid #00ff00; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Since parent is hidden and child inherits hidden, no SolidRect or Text should be emitted
        for item in &items {
            match item {
                DisplayItem::SolidRect { .. } => {
                    panic!("Should not paint any SolidRect for hidden elements");
                }
                DisplayItem::Text { .. } => {
                    panic!("Should not paint any Text for hidden elements");
                }
                _ => {}
            }
        }
        assert!(items.is_empty(), "Display list should be empty");
    }

    #[test]
    fn test_paint_visibility_hidden_visible_child() {
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

        let span = dom.create_node(NodeData::Element {
            name: "span".into(),
            attrs: vec![],
        });
        dom.append_child(div, span);

        let stylesheet = parse_stylesheet(
            "
            div { visibility: hidden; background-color: #ff0000; height: 10px; }
            span { visibility: visible; background-color: #0000ff; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_blue = false;
        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                if *color == Color::Rgba(255, 0, 0, 255) {
                    panic!("Should not paint red background for hidden parent div");
                }
                if *color == Color::Rgba(0, 0, 255, 255) {
                    found_blue = true;
                }
            }
        }
        assert!(
            found_blue,
            "Should paint blue background for visible child span"
        );
    }

    #[test]
    fn test_paint_visibility_visible_regression() {
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

        let text = dom.create_node(NodeData::Text("visible text".into()));
        dom.append_child(div, text);

        let stylesheet = parse_stylesheet(
            "
            div { visibility: visible; background-color: #ff0000; color: #0000ff; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_rect = false;
        let mut text_fragments = Vec::new();

        for item in &items {
            match item {
                DisplayItem::SolidRect { color, .. } => {
                    if *color == Color::Rgba(255, 0, 0, 255) {
                        found_rect = true;
                    }
                }
                DisplayItem::Text { text, color, .. } if *color == Color::Rgba(0, 0, 255, 255) => {
                    text_fragments.push(text.clone());
                }
                _ => {}
            }
        }

        assert!(found_rect, "Should paint background for visible div");
        assert_eq!(text_fragments.concat(), "visible text");
    }

    #[test]
    fn test_paint_opacity_basic() {
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
            div { opacity: 0.5; background-color: #ff0000; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_bg = false;
        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                let Color::Rgba(r, g, b, alpha) = color;
                if *r == 255 && *g == 0 && *b == 0 {
                    found_bg = true;
                    assert!(
                        (*alpha as i32 - 127).abs() <= 1,
                        "Expected alpha close to 127, got {}",
                        alpha
                    );
                }
            }
        }
        assert!(found_bg, "Background SolidRect should be present");
    }

    #[test]
    fn test_paint_opacity_nested() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let parent = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "parent".into())],
        });
        dom.append_child(body, parent);

        let child = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "child".into())],
        });
        dom.append_child(parent, child);

        let stylesheet = parse_stylesheet(
            "
            .parent { opacity: 0.5; height: 10px; }
            .child { opacity: 0.5; background-color: #ff0000; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_child_bg = false;
        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                let Color::Rgba(r, g, b, alpha) = color;
                if *r == 255 && *g == 0 && *b == 0 {
                    found_child_bg = true;
                    assert!(
                        (*alpha as i32 - 64).abs() <= 1,
                        "Expected nested child alpha close to 64, got {}",
                        alpha
                    );
                }
            }
        }
        assert!(
            found_child_bg,
            "Child background SolidRect should be present"
        );
    }

    #[test]
    fn test_paint_opacity_one_and_absent() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "div1".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "div2".into())],
        });
        dom.append_child(body, div2);

        let stylesheet = parse_stylesheet(
            "
            .div1 { opacity: 1.0; background-color: #ff0000; height: 10px; }
            .div2 { background-color: #00ff00; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_div1 = false;
        let mut found_div2 = false;

        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                let Color::Rgba(r, g, b, alpha) = color;
                if *r == 255 && *g == 0 && *b == 0 {
                    found_div1 = true;
                    assert_eq!(*alpha, 255, "div1 with opacity 1.0 should have alpha 255");
                } else if *r == 0 && *g == 255 && *b == 0 {
                    found_div2 = true;
                    assert_eq!(*alpha, 255, "div2 without opacity should have alpha 255");
                }
            }
        }
        assert!(found_div1 && found_div2, "Both divs should be rendered");
    }

    #[test]
    fn test_paint_opacity_zero() {
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
            div { opacity: 0; background-color: #ff0000; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_bg = false;
        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                let Color::Rgba(r, g, b, alpha) = color;
                if *r == 255 && *g == 0 && *b == 0 {
                    found_bg = true;
                    assert_eq!(*alpha, 0, "div with opacity 0 should have alpha 0");
                }
            }
        }
        assert!(found_bg, "Background SolidRect should be present");
    }

    #[test]
    fn test_paint_opacity_percentage() {
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
            div { opacity: 50%; background-color: #ff0000; height: 10px; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_bg = false;
        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                let Color::Rgba(r, g, b, alpha) = color;
                if *r == 255 && *g == 0 && *b == 0 {
                    found_bg = true;
                    assert!(
                        (*alpha as i32 - 127).abs() <= 1,
                        "Expected percentage opacity alpha close to 127, got {}",
                        alpha
                    );
                }
            }
        }
        assert!(found_bg, "Background SolidRect should be present");
    }

    #[test]
    fn test_zero_area_box_emits_no_background_rect() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div1 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "zero-width".into())],
        });
        dom.append_child(body, div1);

        let div2 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "zero-height".into())],
        });
        dom.append_child(body, div2);

        let div3 = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "normal".into())],
        });
        dom.append_child(body, div3);

        let stylesheet = parse_stylesheet(
            "
            .zero-width { width: 0px; height: 10px; background-color: rgb(255, 0, 0); }
            .zero-height { width: 10px; height: 0px; background-color: rgb(0, 255, 0); }
            .normal { width: 10px; height: 10px; background-color: rgb(0, 0, 255); }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_red = false;
        let mut found_green = false;
        let mut found_blue = false;

        for item in &items {
            if let DisplayItem::SolidRect { color, .. } = item {
                if *color == Color::Rgba(255, 0, 0, 255) {
                    found_red = true;
                } else if *color == Color::Rgba(0, 255, 0, 255) {
                    found_green = true;
                } else if *color == Color::Rgba(0, 0, 255, 255) {
                    found_blue = true;
                }
            }
        }

        assert!(
            !found_red,
            "Should not emit SolidRect for zero-width box (red background)"
        );
        assert!(
            !found_green,
            "Should not emit SolidRect for zero-height box (green background)"
        );
        assert!(
            found_blue,
            "Should emit exactly one SolidRect for normal box (blue background)"
        );
    }

    #[test]
    fn test_box_shadow_emits_offset_rect() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div_shadow = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "shadowed".into())],
        });
        dom.append_child(body, div_shadow);

        let div_normal = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "normal".into())],
        });
        dom.append_child(body, div_normal);

        let stylesheet = parse_stylesheet(
            "
            .shadowed { width: 100px; height: 50px; background-color: rgb(0, 255, 0); box-shadow: 5px 5px #ff0000; }
            .normal { width: 100px; height: 50px; background-color: rgb(0, 0, 255); box-shadow: none; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Verify shadow rect and background rect ordering
        let mut shadow_idx = None;
        let mut bg_idx = None;
        let mut shadow_rect_found = None;
        let mut bg_rect_found = None;

        for (idx, item) in items.iter().enumerate() {
            if let DisplayItem::SolidRect { rect, color } = item {
                if *color == Color::Rgba(255, 0, 0, 255) {
                    shadow_idx = Some(idx);
                    shadow_rect_found = Some(*rect);
                } else if *color == Color::Rgba(0, 255, 0, 255) {
                    bg_idx = Some(idx);
                    bg_rect_found = Some(*rect);
                }
            }
        }

        let s_idx = shadow_idx.expect("Should find red shadow SolidRect");
        let b_idx = bg_idx.expect("Should find green background SolidRect");
        let s_rect = shadow_rect_found.expect("Should find red shadow rect");
        let b_rect = bg_rect_found.expect("Should find green background rect");

        // The shadow SolidRect must be ordered BEFORE the background rect
        assert!(
            s_idx < b_idx,
            "Shadow rect must be painted before background rect"
        );

        // Shadow origin is offset by (5,5) relative to the box's border-box (the background rect)
        assert_eq!(
            s_rect.origin.x,
            b_rect.origin.x + 5.0,
            "Shadow X origin should be offset by 5px"
        );
        assert_eq!(
            s_rect.origin.y,
            b_rect.origin.y + 5.0,
            "Shadow Y origin should be offset by 5px"
        );

        // Shadow size equals the border-box size (background rect size)
        assert_eq!(
            s_rect.size.width, b_rect.size.width,
            "Shadow width should match background width"
        );
        assert_eq!(
            s_rect.size.height, b_rect.size.height,
            "Shadow height should match background height"
        );

        // Assert box-shadow: none (or absent) emits no red or extra shadow rect (there should be only one red rect in total)
        let red_rect_count = items
            .iter()
            .filter(|item| {
                if let DisplayItem::SolidRect { color, .. } = item {
                    *color == Color::Rgba(255, 0, 0, 255)
                } else {
                    false
                }
            })
            .count();
        assert_eq!(
            red_rect_count, 1,
            "There should be exactly one red shadow rect emitted in total"
        );
    }

    #[test]
    fn test_box_shadow_spread() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div_shadow = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "shadowed".into())],
        });
        dom.append_child(body, div_shadow);

        let div_collapsed = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "collapsed".into())],
        });
        dom.append_child(body, div_collapsed);

        let stylesheet = parse_stylesheet(
            "
            .shadowed { width: 100px; height: 50px; background-color: rgb(0, 255, 0); box-shadow: 0px 0px 0px 10px #ff0000; }
            .collapsed { width: 100px; height: 50px; background-color: rgb(0, 0, 255); box-shadow: 0px 0px 0px -100px #ff0000; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Verify positive spread shadow
        let mut shadow_rect = None;
        let mut bg_rect = None;
        let mut red_count = 0;

        for item in &items {
            if let DisplayItem::SolidRect { rect, color } = item {
                if *color == Color::Rgba(255, 0, 0, 255) {
                    shadow_rect = Some(*rect);
                    red_count += 1;
                } else if *color == Color::Rgba(0, 255, 0, 255) {
                    bg_rect = Some(*rect);
                }
            }
        }

        // We expect exactly 1 red shadow rect to be emitted (for the .shadowed box, but NOT the .collapsed box)
        assert_eq!(
            red_count, 1,
            "Should emit exactly one red shadow rect across both divs"
        );

        let s_rect = shadow_rect.expect("Should find red shadow rect");
        let b_rect = bg_rect.expect("Should find green background rect");

        // The positive spread is 10px. It should inflate on all sides:
        // origin.x -= 10, origin.y -= 10, size.width += 20, size.height += 20
        assert_eq!(
            s_rect.origin.x,
            b_rect.origin.x - 10.0,
            "Shadow X origin should be inflated by 10px"
        );
        assert_eq!(
            s_rect.origin.y,
            b_rect.origin.y - 10.0,
            "Shadow Y origin should be inflated by 10px"
        );
        assert_eq!(
            s_rect.size.width,
            b_rect.size.width + 20.0,
            "Shadow width should be inflated by 20px"
        );
        assert_eq!(
            s_rect.size.height,
            b_rect.size.height + 20.0,
            "Shadow height should be inflated by 20px"
        );
    }
}
