pub mod invalidate;
pub mod stacking;
pub mod table;

use crate::css::values::{Color, CssValue};
use crate::dom::{Dom, NodeData};
use crate::geom::Rect;
use crate::infra::NodeId;
use crate::layout::LayoutBox;
use crate::style::CategorizedComputedStyle;
use std::collections::HashMap;

/// spec: object-fit keyword values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

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
        object_fit: ObjectFit,
    },
}

/// A list of display items, representing the final visual output.
/// spec: S-12
pub struct DisplayList(pub Vec<DisplayItem>);

/// Helper to extract border width from computed styles in px.
/// spec: S-39
fn get_border_width(style: &CategorizedComputedStyle, prop: &str) -> f32 {
    let w = match prop {
        "border-top-width" => style.reset_surround.border_top_width,
        "border-right-width" => style.reset_surround.border_right_width,
        "border-bottom-width" => style.reset_surround.border_bottom_width,
        "border-left-width" => style.reset_surround.border_left_width,
        _ => -1,
    };
    if w < 0 { 0.0 } else { w as f32 }
}

/// Helper to extract outline width from computed styles in px, defaulting to medium (3.0px).
fn get_outline_width(style: &CategorizedComputedStyle) -> f32 {
    let ow = style.reset_effects.outline_width;
    if ow == -1 { 3.0 } else { ow as f32 }
}

/// Helper to extract outline offset from computed styles in px, defaulting to 0.0px.
fn get_outline_offset(style: &CategorizedComputedStyle) -> f32 {
    style.reset_effects.outline_offset as f32
}

/// Helper to paint standard solid outline strips on all four sides.
fn paint_outline_strips(
    items: &mut Vec<DisplayItem>,
    rect: Rect,
    ow: f32,
    offset: f32,
    color: Color,
) {
    let x = rect.origin.x;
    let y = rect.origin.y;
    let w = rect.size.width.max(0.0);
    let h = rect.size.height.max(0.0);
    let d = offset + ow;

    // Top strip
    let top_rect = Rect::new(x - d, y - d, w + 2.0 * d, ow);
    if top_rect.size.width > 0.0 && top_rect.size.height > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: top_rect,
            color: color.clone(),
        });
    }

    // Bottom strip
    let bottom_rect = Rect::new(x - d, y + h + offset, w + 2.0 * d, ow);
    if bottom_rect.size.width > 0.0 && bottom_rect.size.height > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: bottom_rect,
            color: color.clone(),
        });
    }

    // Left strip
    let left_rect = Rect::new(x - d, y - offset, ow, h + 2.0 * offset);
    if left_rect.size.width > 0.0 && left_rect.size.height > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: left_rect,
            color: color.clone(),
        });
    }

    // Right strip
    let right_rect = Rect::new(x + w + offset, y - offset, ow, h + 2.0 * offset);
    if right_rect.size.width > 0.0 && right_rect.size.height > 0.0 {
        items.push(DisplayItem::SolidRect {
            rect: right_rect,
            color,
        });
    }
}

/// Helper to paint a dotted or dashed segmented outline edge.
fn paint_segmented_outline_edge(
    items: &mut Vec<DisplayItem>,
    horizontal: bool,
    edge_rect: Rect,
    dash_gap: (f32, f32),
    color: Color,
) {
    let (length, gap) = dash_gap;
    if horizontal {
        let start_coord = edge_rect.origin.x;
        let cross_coord = edge_rect.origin.y;
        let total_len = edge_rect.size.width;
        let ow = edge_rect.size.height;

        if total_len <= 0.0 || ow <= 0.0 {
            return;
        }

        let mut current_x = start_coord;
        let end_x = start_coord + total_len;
        while current_x < end_x {
            let current_width = if current_x + length > end_x {
                end_x - current_x
            } else {
                length
            };
            if current_width > 0.0 {
                items.push(DisplayItem::SolidRect {
                    rect: Rect::new(current_x, cross_coord, current_width, ow),
                    color: color.clone(),
                });
            }
            current_x += length + gap;
        }
    } else {
        let start_coord = edge_rect.origin.y;
        let cross_coord = edge_rect.origin.x;
        let total_len = edge_rect.size.height;
        let ow = edge_rect.size.width;

        if total_len <= 0.0 || ow <= 0.0 {
            return;
        }

        let mut current_y = start_coord;
        let end_y = start_coord + total_len;
        while current_y < end_y {
            let current_height = if current_y + length > end_y {
                end_y - current_y
            } else {
                length
            };
            if current_height > 0.0 {
                items.push(DisplayItem::SolidRect {
                    rect: Rect::new(cross_coord, current_y, ow, current_height),
                    color: color.clone(),
                });
            }
            current_y += length + gap;
        }
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

fn parse_css_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if s.eq_ignore_ascii_case("currentcolor") {
        return None;
    }
    if let Some(hex) = s.strip_prefix('#') {
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                return Some(Color::Rgba(r * 17, g * 17, b * 17, 255));
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                return Some(Color::Rgba(r * 17, g * 17, b * 17, a * 17));
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::Rgba(r, g, b, 255));
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color::Rgba(r, g, b, a));
            }
            _ => return None,
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inside = &s[4..s.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Color::Rgba(r, g, b, 255));
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inside = &s[5..s.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|p| p.trim()).collect();
        if parts.len() == 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let alpha = parts[3].parse::<f32>().ok()?;
            let a = (alpha * 255.0).clamp(0.0, 255.0) as u8;
            return Some(Color::Rgba(r, g, b, a));
        }
    }
    crate::css::colors::named_color(s)
}

fn parse_background_size(s: &str) -> CssValue {
    let s = s.trim();
    if s.eq_ignore_ascii_case("contain") {
        return CssValue::Keyword("contain".to_string());
    }
    if s.eq_ignore_ascii_case("cover") {
        return CssValue::Keyword("cover".to_string());
    }
    if s.eq_ignore_ascii_case("auto") {
        return CssValue::Keyword("auto".to_string());
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return CssValue::Keyword("auto".to_string());
    }

    fn parse_single(p: &str) -> CssValue {
        if p.eq_ignore_ascii_case("auto") {
            return CssValue::Keyword("auto".to_string());
        }
        if let Some(num_str) = p.strip_suffix("px") {
            if let Ok(v) = num_str.parse::<f32>() {
                return CssValue::Length(v, crate::css::values::LengthUnit::Px);
            }
        }
        if let Some(num_str) = p.strip_suffix('%') {
            if let Ok(v) = num_str.parse::<f32>() {
                return CssValue::Length(v, crate::css::values::LengthUnit::Percent);
            }
        }
        if let Ok(v) = p.parse::<f32>() {
            return CssValue::Number(v);
        }
        CssValue::Keyword(p.to_string())
    }

    if parts.len() == 1 {
        parse_single(parts[0])
    } else {
        let list = parts.into_iter().map(parse_single).collect();
        CssValue::Multiple(list)
    }
}

fn parse_background_position(s: &str) -> CssValue {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.is_empty() {
        return CssValue::Keyword("0% 0%".to_string());
    }

    fn parse_single(p: &str) -> CssValue {
        if p.eq_ignore_ascii_case("left")
            || p.eq_ignore_ascii_case("right")
            || p.eq_ignore_ascii_case("center")
            || p.eq_ignore_ascii_case("top")
            || p.eq_ignore_ascii_case("bottom")
        {
            return CssValue::Keyword(p.to_string());
        }
        if let Some(num_str) = p.strip_suffix("px") {
            if let Ok(v) = num_str.parse::<f32>() {
                return CssValue::Length(v, crate::css::values::LengthUnit::Px);
            }
        }
        if let Some(num_str) = p.strip_suffix('%') {
            if let Ok(v) = num_str.parse::<f32>() {
                return CssValue::Length(v, crate::css::values::LengthUnit::Percent);
            }
        }
        if let Ok(v) = p.parse::<f32>() {
            return CssValue::Number(v);
        }
        CssValue::Keyword(p.to_string())
    }

    if parts.len() == 1 {
        parse_single(parts[0])
    } else {
        let list = parts.into_iter().map(parse_single).collect();
        CssValue::Multiple(list)
    }
}

/// Helper to resolve the general border color of an element style,
/// with fallbacks to shorthand `border`, computed text `color`, and finally black.
/// spec: S-39
fn get_border_color(style: &CategorizedComputedStyle) -> Color {
    // Try the individual border colors first (e.g. top color)
    if style.reset_surround.border_top_color != "currentcolor" {
        if let Some(c) = parse_css_color(&style.reset_surround.border_top_color) {
            return c;
        }
    }
    // Then the base `border-color` shorthand value. This survives the outset/inset
    // per-edge color stripping, letting the UA button bevel recover its silver border.
    if style.reset_surround.border_color != "currentcolor" {
        if let Some(c) = parse_css_color(&style.reset_surround.border_color) {
            return c;
        }
    }
    // Fall back to computed text "color"
    if let Some(c) = parse_css_color(&style.inherited_text.color) {
        return c;
    }
    // Default to black
    Color::Rgba(0, 0, 0, 255)
}

/// Helper to resolve a specific edge border color, falling back to the resolved border color.
/// spec: S-39
fn get_edge_color(
    style: &CategorizedComputedStyle,
    edge_prop: &str,
    border_color: &Color,
) -> Color {
    let color_str = match edge_prop {
        "border-top-color" => &style.reset_surround.border_top_color,
        "border-right-color" => &style.reset_surround.border_right_color,
        "border-bottom-color" => &style.reset_surround.border_bottom_color,
        "border-left-color" => &style.reset_surround.border_left_color,
        _ => "currentcolor",
    };
    if color_str != "currentcolor" {
        if let Some(c) = parse_css_color(color_str) {
            return c;
        }
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

/// Helper to get the set of text decorations from a CategorizedComputedStyle.
/// spec: S-55
fn get_text_decorations(style: &CategorizedComputedStyle) -> TextDecorations {
    let mut dec = TextDecorations::default();

    let line_str = &style.reset_effects.text_decoration_line;
    if line_str.eq_ignore_ascii_case("none") {
        return dec;
    }
    // `text_decoration_line` may carry both line keywords and (when authored via the
    // `text-decoration` shorthand) a style keyword, e.g. "underline dashed".
    for part in line_str.split_whitespace() {
        if part.eq_ignore_ascii_case("underline") {
            dec.underline = true;
        } else if part.eq_ignore_ascii_case("overline") {
            dec.overline = true;
        } else if part.eq_ignore_ascii_case("line-through") {
            dec.line_through = true;
        } else if let Some(s) = parse_text_decoration_style(part) {
            dec.style = Some(s);
        }
    }

    let color_str = &style.reset_effects.text_decoration_color;
    if color_str != "currentcolor" {
        dec.color = parse_css_color(color_str);
    }

    // The longhand `text-decoration-style` (empty = unspecified) wins over a style
    // keyword embedded in the shorthand line above.
    let style_str = &style.reset_effects.text_decoration_style;
    if let Some(s) = parse_text_decoration_style(style_str) {
        dec.style = Some(s);
    }

    dec
}

/// Map a CSS `text-decoration-style` keyword to its enum, or `None` if it is not a
/// recognized style keyword (including the empty "unspecified" sentinel).
fn parse_text_decoration_style(s: &str) -> Option<TextDecorationStyle> {
    if s.eq_ignore_ascii_case("solid") {
        Some(TextDecorationStyle::Solid)
    } else if s.eq_ignore_ascii_case("double") {
        Some(TextDecorationStyle::Double)
    } else if s.eq_ignore_ascii_case("dotted") {
        Some(TextDecorationStyle::Dotted)
    } else if s.eq_ignore_ascii_case("dashed") {
        Some(TextDecorationStyle::Dashed)
    } else {
        None
    }
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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> Color {
    let mut current = Some(node_id);
    let mut depth = 0;
    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(style) = styles.get(&curr_id) {
            if let Some(c) = parse_css_color(&style.inherited_text.color) {
                return c;
            }
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

/// Helper to resolve the computed text shadow of a text node, following the DOM tree upwards.
fn resolve_text_shadow(
    dom: &Dom,
    node_id: NodeId,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> Option<CssValue> {
    let mut current = Some(node_id);
    let mut depth = 0;
    while let Some(curr_id) = current
        && depth < 1000
    {
        if let Some(style) = styles.get(&curr_id)
            && let Some(ref val) = style.inherited_text.text_shadow
        {
            return Some(val.clone());
        }
        current = dom.parent(curr_id);
        depth += 1;
    }
    None
}

/// Helper to recursively check and resolve the active text decorations for a node from its DOM ancestors.
/// spec: S-55
fn resolve_text_decorations(
    dom: &Dom,
    node_id: NodeId,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
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
            if style
                .reset_effects
                .text_decoration_line
                .eq_ignore_ascii_case("none")
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
fn get_opacity(style: &CategorizedComputedStyle) -> f32 {
    style.reset_effects.opacity
}

/// Resolve object-fit from style.
fn get_object_fit(style: &CategorizedComputedStyle) -> ObjectFit {
    match style.reset_box.object_fit.to_ascii_lowercase().as_str() {
        "contain" => ObjectFit::Contain,
        "cover" => ObjectFit::Cover,
        "none" => ObjectFit::None,
        "scale-down" => ObjectFit::ScaleDown,
        _ => ObjectFit::Fill,
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

fn resolve_position_coord(val: &CssValue, box_dim: f32, img_dim: f32, _is_vertical: bool) -> f32 {
    match val {
        CssValue::Length(v, unit) => match unit {
            crate::css::values::LengthUnit::Percent => {
                // Percentage resolves against (box_dim - img_dim) per CSS spec.
                // spec: https://www.w3.org/TR/css-backgrounds-3/#background-position
                (box_dim - img_dim) * (*v / 100.0)
            }
            _ => *v, // Treat other units as raw px for simplicity
        },
        CssValue::Number(v) => *v,
        CssValue::Keyword(kw) => {
            let kw_lower = kw.to_ascii_lowercase();
            match kw_lower.as_str() {
                "left" => 0.0,
                "right" => box_dim - img_dim,
                "top" => 0.0,
                "bottom" => box_dim - img_dim,
                "center" => (box_dim - img_dim) / 2.0,
                _ => 0.0,
            }
        }
        _ => 0.0,
    }
}

fn get_background_position_offsets(
    val: &CssValue,
    box_w: f32,
    box_h: f32,
    img_w: f32,
    img_h: f32,
) -> (f32, f32) {
    match val {
        CssValue::Multiple(list) if list.len() >= 2 => {
            let first = &list[0];
            let second = &list[1];
            // Check if the first keyword is top/bottom or second keyword is left/right to handle order swap if needed.
            let mut x_val = first;
            let mut y_val = second;
            if let CssValue::Keyword(k1) = first
                && (k1.eq_ignore_ascii_case("top") || k1.eq_ignore_ascii_case("bottom"))
                && let CssValue::Keyword(k2) = second
                && {
                    let k2_l = k2.to_ascii_lowercase();
                    k2_l == "left" || k2_l == "right" || k2_l == "center"
                }
            {
                x_val = second;
                y_val = first;
            }
            let x = resolve_position_coord(x_val, box_w, img_w, false);
            let y = resolve_position_coord(y_val, box_h, img_h, true);
            (x, y)
        }
        _ => {
            // Single value
            let is_vert = if let CssValue::Keyword(kw) = val {
                let kw_l = kw.to_ascii_lowercase();
                kw_l == "top" || kw_l == "bottom"
            } else {
                false
            };

            if is_vert {
                let x = (box_w - img_w) / 2.0;
                let y = resolve_position_coord(val, box_h, img_h, true);
                (x, y)
            } else {
                let x = resolve_position_coord(val, box_w, img_w, false);
                let y = (box_h - img_h) / 2.0;
                (x, y)
            }
        }
    }
}

/// Resolve the background-size value to specific painted (width, height) dimensions.
/// spec: <https://drafts.csswg.org/css-backgrounds-3/#the-background-size>
pub fn resolve_bg_size(area: (f32, f32), intrinsic: (f32, f32), value: &CssValue) -> (f32, f32) {
    let (area_w, area_h) = area;
    let (img_w, img_h) = intrinsic;

    // Guard against zero or negative intrinsic dimensions to avoid division by zero or negative sizing.
    if img_w <= 0.0 || img_h <= 0.0 {
        return (0.0, 0.0);
    }

    match value {
        CssValue::Keyword(kw) => {
            let kw_trimmed = kw.trim().to_ascii_lowercase();
            match kw_trimmed.as_str() {
                "contain" => {
                    let scale_w = area_w / img_w;
                    let scale_h = area_h / img_h;
                    let scale = scale_w.min(scale_h);
                    (img_w * scale, img_h * scale)
                }
                "cover" => {
                    let scale_w = area_w / img_w;
                    let scale_h = area_h / img_h;
                    let scale = scale_w.max(scale_h);
                    (img_w * scale, img_h * scale)
                }
                "auto" => (img_w, img_h),
                _ => (img_w, img_h), // Fallback to intrinsic size
            }
        }
        CssValue::Length(v, u) => {
            // Single value: width is set, height is auto (preserving aspect ratio)
            let w = if *u == crate::css::values::LengthUnit::Px {
                *v
            } else if *u == crate::css::values::LengthUnit::Percent {
                area_w * (*v / 100.0)
            } else {
                img_w
            };
            let w_clamped = w.max(0.0);
            let h = w_clamped * (img_h / img_w);
            (w_clamped, h)
        }
        CssValue::Multiple(list) => {
            if list.is_empty() {
                (img_w, img_h)
            } else if list.len() == 1 {
                resolve_bg_size(area, intrinsic, &list[0])
            } else {
                let w = match &list[0] {
                    CssValue::Length(v1, u1) => {
                        let val = if *u1 == crate::css::values::LengthUnit::Px {
                            *v1
                        } else if *u1 == crate::css::values::LengthUnit::Percent {
                            area_w * (*v1 / 100.0)
                        } else {
                            img_w
                        };
                        val.max(0.0)
                    }
                    CssValue::Keyword(kw) if kw.trim().eq_ignore_ascii_case("auto") => {
                        -1.0 // Sentinel for auto
                    }
                    _ => img_w,
                };

                let h = match &list[1] {
                    CssValue::Length(v2, u2) => {
                        let val = if *u2 == crate::css::values::LengthUnit::Px {
                            *v2
                        } else if *u2 == crate::css::values::LengthUnit::Percent {
                            area_h * (*v2 / 100.0)
                        } else {
                            img_h
                        };
                        val.max(0.0)
                    }
                    CssValue::Keyword(kw) if kw.trim().eq_ignore_ascii_case("auto") => {
                        -1.0 // Sentinel for auto
                    }
                    _ => img_h,
                };

                match (w < 0.0, h < 0.0) {
                    (true, true) => (img_w, img_h),
                    (true, false) => {
                        // width is auto, height is explicit
                        let resolved_w = h * (img_w / img_h);
                        (resolved_w.max(0.0), h)
                    }
                    (false, true) => {
                        // width is explicit, height is auto
                        let resolved_h = w * (img_h / img_w);
                        (w, resolved_h.max(0.0))
                    }
                    (false, false) => (w, h),
                }
            }
        }
        _ => (img_w, img_h),
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
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
) -> DisplayList {
    build_display_list_with_caret(layout, dom, styles, None)
}

/// Builds a display list from the layout tree, with optional caret state.
/// spec: S-12
pub fn build_display_list_with_caret(
    layout: &LayoutBox,
    dom: &Dom,
    styles: &HashMap<NodeId, CategorizedComputedStyle>,
    caret: Option<(NodeId, usize)>,
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
            let node_hidden = style.inherited_effects.visibility == "hidden"
                || style.inherited_effects.visibility == "collapse";

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
            let mut input_text_item = None;
            let mut caret_item = None;

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

                let text_color = parse_css_color(&style.inherited_text.color)
                    .unwrap_or(Color::Rgba(0, 0, 0, 255));

                let corrected_rect = Rect::new(text_x, text_y, text_w, text_h);
                btn_label_item = Some(DisplayItem::Text {
                    rect: corrected_rect,
                    text: label_text,
                    color: scale_color_alpha(&text_color, effective_opacity),
                });
            } else if is_text_input {
                skip_children = true;

                let rect = layout_box.rect;
                let x = rect.origin.x;
                let y = rect.origin.y;
                let w = rect.size.width.max(0.0);
                let h = rect.size.height.max(0.0);

                let value = dom.get_attribute(node_id, "value").map(|v| v.to_string());
                let placeholder = dom
                    .get_attribute(node_id, "placeholder")
                    .map(|v| v.to_string());

                let mut to_draw = None;
                let mut text_color = Color::Rgba(0, 0, 0, 255);

                if let Some(val) = value.as_ref().filter(|v| !v.is_empty()) {
                    to_draw = Some(val.clone());
                    text_color = parse_css_color(&style.inherited_text.color)
                        .unwrap_or(Color::Rgba(0, 0, 0, 255));
                } else if let Some(ph) = placeholder.as_ref().filter(|p| !p.is_empty()) {
                    to_draw = Some(ph.clone());
                    text_color = Color::Rgba(117, 117, 117, 255); // CSS #757575
                }

                if let Some(text) = to_draw {
                    let font = crate::font::BitmapFont::builtin();
                    let text_w = font.measure(&text) as f32;
                    let text_h = font.line_height() as f32;

                    let text_x = x + 2.0;
                    let text_y = y + ((h - text_h) / 2.0).max(0.0);

                    let avail = (w - 2.0).max(0.0);
                    let draw_w = text_w.min(avail);

                    let corrected_rect = Rect::new(text_x, text_y, draw_w, text_h);
                    input_text_item = Some(DisplayItem::Text {
                        rect: corrected_rect,
                        text,
                        color: scale_color_alpha(&text_color, effective_opacity),
                    });
                }

                if let Some((focus_id, caret_index)) = caret
                    && focus_id == node_id
                {
                    let font = crate::font::BitmapFont::builtin();
                    let text_h = font.line_height() as f32;
                    let text_x = x + 2.0;

                    let val_str = value.clone().unwrap_or_default();
                    let val_chars: Vec<char> = val_str.chars().collect();
                    let clamped_index = caret_index.min(val_chars.len());
                    let prefix: String = val_chars[..clamped_index].iter().collect();

                    let caret_x = text_x + font.measure(&prefix) as f32;
                    let caret_x = caret_x.min(x + w);

                    let caret_y = y + ((h - text_h) / 2.0).max(0.0);
                    let caret_color =
                        scale_color_alpha(&Color::Rgba(0, 0, 0, 255), effective_opacity);

                    caret_item = Some(DisplayItem::SolidRect {
                        rect: Rect::new(caret_x, caret_y, 1.0, text_h),
                        color: caret_color,
                    });
                }
            }

            if !node_hidden {
                if let Some(NodeData::Element { name, .. }) = dom.data(node_id) {
                    let mut current = Some(node_id);
                    let mut collapse = false;
                    while let Some(curr) = current {
                        if let Some(NodeData::Element { name: el_name, .. }) = dom.data(curr)
                            && el_name.eq_ignore_ascii_case("table")
                        {
                            if let Some(table_style) = styles.get(&curr) {
                                collapse = crate::layout::is_border_collapse(table_style);
                            }
                            break;
                        }
                        current = dom.parent(curr);
                    }

                    items.extend(table::table_border_items(
                        dom,
                        node_id,
                        name,
                        layout_box.rect,
                        collapse,
                    ));
                }

                // Paint box-shadow if present
                if let Some(ref box_shadow_val) = style.reset_effects.box_shadow {
                    // Flatten values to check for none, inset, or comma
                    let mut leaves = Vec::new();
                    flatten_value(box_shadow_val, &mut leaves);

                    let mut has_inset = false;
                    let mut has_none = false;
                    for leaf in &leaves {
                        if let CssValue::Keyword(kw) = leaf {
                            let kw_lower = kw.to_ascii_lowercase();
                            if kw_lower == "inset" {
                                has_inset = true;
                            } else if kw_lower == "none" {
                                has_none = true;
                            }
                        }
                    }

                    if leaves.is_empty() || has_none || has_inset {
                        // TODO(spec): box-shadow v2 — outer shadows with spread (blur is still unimplemented; inset remains out of scope).
                    } else {
                        // Helper closure to parse a single shadow segment and push its DisplayItem if valid.
                        let parse_and_paint_shadow =
                            |segment_leaves: &[&CssValue], items: &mut Vec<DisplayItem>| {
                                // Parse lengths and colors
                                let mut length_values = Vec::new();
                                let mut color_value = None;

                                for leaf in segment_leaves {
                                    match *leaf {
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
                                    } else if let Some(c) =
                                        parse_css_color(&style.inherited_text.color)
                                    {
                                        c
                                    } else {
                                        Color::Rgba(0, 0, 0, 255)
                                    };

                                    // Only paint shadow if the border box is valid (has positive dimensions)
                                    if layout_box.rect.size.width > 0.0
                                        && layout_box.rect.size.height > 0.0
                                    {
                                        // Translate and inflate/deflate by spread
                                        let mut shadow_rect = layout_box.rect;
                                        shadow_rect.origin.x += offset_x - spread;
                                        shadow_rect.origin.y += offset_y - spread;
                                        shadow_rect.size.width += 2.0 * spread;
                                        shadow_rect.size.height += 2.0 * spread;

                                        // Only paint if resulting dimensions are positive (fully shrunk shadow is invisible)
                                        if shadow_rect.size.width > 0.0
                                            && shadow_rect.size.height > 0.0
                                        {
                                            items.push(DisplayItem::SolidRect {
                                                rect: shadow_rect,
                                                color: scale_color_alpha(
                                                    &shadow_color,
                                                    effective_opacity,
                                                ),
                                            });
                                        }
                                    }
                                }
                            };

                        // Split leaves into segments delimited by CssValue::Keyword(",")
                        let segments: Vec<&[&CssValue]> = leaves
                            .split(|leaf| {
                                if let CssValue::Keyword(kw) = leaf {
                                    kw == ","
                                } else {
                                    false
                                }
                            })
                            .collect();

                        // Paint order: per CSS spec, the first listed shadow is on top,
                        // so later shadows should be painted first (pushed before) so earlier ones overlay them.
                        // We iterate and parse/paint segments in reverse order.
                        for segment in segments.iter().rev() {
                            parse_and_paint_shadow(segment, &mut items);
                        }
                    }
                }

                // spec: if node has background-color -> SolidRect
                if let Some(color) = parse_css_color(&style.reset_background.background_color) {
                    // Do not paint fully transparent background colors
                    let Color::Rgba(_, _, _, a) = color;
                    if a > 0 {
                        // B-4: do not paint background for zero/negative-area boxes
                        if layout_box.rect.size.width > 0.0 && layout_box.rect.size.height > 0.0 {
                            // TODO(spec): border/images/gradients/rasterization
                            items.push(DisplayItem::SolidRect {
                                rect: layout_box.rect,
                                color: scale_color_alpha(&color, effective_opacity),
                            });
                        }
                    }
                }

                // Paint background-image (t0382)
                let mut bg_img_src = None;
                let kw = &style.reset_background.background_image;
                if kw.starts_with("url(") && kw.ends_with(')') {
                    let inner = &kw[4..kw.len() - 1];
                    bg_img_src = Some(inner.trim_matches(|c| c == '"' || c == '\'').to_string());
                } else if !kw.is_empty() && kw != "none" {
                    bg_img_src = Some(kw.clone());
                }

                if let Some(src) = bg_img_src
                    && let Some(decoded) = dom.get_image(&src)
                {
                    let box_rect = layout_box.rect;
                    if box_rect.size.width > 0.0 && box_rect.size.height > 0.0 {
                        // Extract repeat
                        let repeat_val = style
                            .reset_background
                            .background_repeat
                            .to_ascii_lowercase();

                        let (repeat_x, repeat_y) = match repeat_val.as_str() {
                            "no-repeat" => (false, false),
                            "repeat-x" => (true, false),
                            "repeat-y" => (false, true),
                            _ => (true, true),
                        };

                        // Determine image dimensions.
                        let mut img_w = decoded.width as f32;
                        let mut img_h = decoded.height as f32;

                        let size_val =
                            parse_background_size(&style.reset_background.background_size);
                        let (resolved_w, resolved_h) = resolve_bg_size(
                            (box_rect.size.width, box_rect.size.height),
                            (img_w, img_h),
                            &size_val,
                        );
                        img_w = resolved_w;
                        img_h = resolved_h;

                        if img_w > 0.0 && img_h > 0.0 {
                            // Extract position
                            let p_val = parse_background_position(
                                &style.reset_background.background_position,
                            );
                            let (x_offset, y_offset) = get_background_position_offsets(
                                &p_val,
                                box_rect.size.width,
                                box_rect.size.height,
                                img_w,
                                img_h,
                            );

                            // Calculate repeat indices
                            let min_i = if repeat_x {
                                ((-x_offset - img_w) / img_w).floor() as i32
                            } else {
                                0
                            };
                            let max_i = if repeat_x {
                                ((box_rect.size.width - x_offset) / img_w).ceil() as i32
                            } else {
                                0
                            };

                            let min_j = if repeat_y {
                                ((-y_offset - img_h) / img_h).floor() as i32
                            } else {
                                0
                            };
                            let max_j = if repeat_y {
                                ((box_rect.size.height - y_offset) / img_h).ceil() as i32
                            } else {
                                0
                            };

                            // Emit tiles
                            for i in min_i..=max_i {
                                for j in min_j..=max_j {
                                    let tile_x = x_offset + i as f32 * img_w;
                                    let tile_y = y_offset + j as f32 * img_h;
                                    let tile_rect = Rect::new(
                                        box_rect.origin.x + tile_x,
                                        box_rect.origin.y + tile_y,
                                        img_w,
                                        img_h,
                                    );

                                    if let Some(clipped_rect) = tile_rect.intersection(box_rect)
                                        && clipped_rect.size.width > 0.0
                                        && clipped_rect.size.height > 0.0
                                    {
                                        // Crop the decoded image to match the clipped_rect
                                        let dx = clipped_rect.origin.x - tile_rect.origin.x;
                                        let dy = clipped_rect.origin.y - tile_rect.origin.y;

                                        // Map destination px to source image pixels.
                                        let scale_x = decoded.width as f32 / img_w;
                                        let scale_y = decoded.height as f32 / img_h;

                                        let start_x = (dx * scale_x).round() as u32;
                                        let start_y = (dy * scale_y).round() as u32;
                                        let clipped_w =
                                            (clipped_rect.size.width * scale_x).round() as u32;
                                        let clipped_h =
                                            (clipped_rect.size.height * scale_y).round() as u32;

                                        let start_x = start_x.min(decoded.width);
                                        let start_y = start_y.min(decoded.height);
                                        let clipped_w = clipped_w.min(decoded.width - start_x);
                                        let clipped_h = clipped_h.min(decoded.height - start_y);

                                        if clipped_w > 0 && clipped_h > 0 {
                                            let mut cropped_rgba = Vec::with_capacity(
                                                (clipped_w * clipped_h * 4) as usize,
                                            );
                                            for sy in start_y..(start_y + clipped_h) {
                                                let src_row_start =
                                                    ((sy * decoded.width + start_x) * 4) as usize;
                                                let src_row_end = (src_row_start
                                                    + (clipped_w * 4) as usize)
                                                    .min(decoded.rgba.len());
                                                cropped_rgba.extend_from_slice(
                                                    &decoded.rgba[src_row_start..src_row_end],
                                                );
                                            }

                                            let cropped_img = crate::image::DecodedImage {
                                                width: clipped_w,
                                                height: clipped_h,
                                                rgba: cropped_rgba,
                                            };

                                            // Retrieve base URL if any
                                            let mut base_url = None;
                                            for n_id in dom.descendants(dom.document()) {
                                                if let Some(NodeData::Element {
                                                    name: el_name, ..
                                                }) = dom.data(n_id)
                                                    && el_name.eq_ignore_ascii_case("base")
                                                    && let Some(href) =
                                                        dom.get_attribute(n_id, "href")
                                                {
                                                    base_url = Some(href.to_string());
                                                    break;
                                                }
                                            }

                                            items.push(DisplayItem::Image {
                                                rect: clipped_rect,
                                                src: src.clone(),
                                                base_url,
                                                decoded: Some(cropped_img),
                                                object_fit: ObjectFit::Fill,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // spec: S-39: read computed border widths
                let border_top = get_border_width(style, "border-top-width");
                let border_right = get_border_width(style, "border-right-width");
                let border_bottom = get_border_width(style, "border-bottom-width");
                let border_left = get_border_width(style, "border-left-width");

                // Borders from the presentational `border` attribute on a table are
                // drawn by `table::table_border_items` (gray frame, empty-cells aware).
                // Skip the generic painter for those so the frame is not double-drawn.
                let skip_presentational_table_border =
                    table::has_presentational_table_border(dom, node_id);

                // If any border has a non-zero width, emit SolidRects for the active edges
                if !skip_presentational_table_border
                    && (border_top > 0.0
                        || border_right > 0.0
                        || border_bottom > 0.0
                        || border_left > 0.0)
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
                        && style.reset_surround.border_top_color == "currentcolor"
                        && style.reset_surround.border_right_color == "currentcolor"
                        && style.reset_surround.border_bottom_color == "currentcolor"
                        && style.reset_surround.border_left_color == "currentcolor"
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
                let outline_style = &style.reset_effects.outline_style;
                let has_outline = !outline_style.eq_ignore_ascii_case("none");

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

                        // TODO(spec): outline color:invert and groove/ridge/inset/outset are not implemented (solid frame fallback, color falls back to text color). double/dotted/dashed are implemented.

                        let mut outline_color = Color::Rgba(0, 0, 0, 255);
                        let mut resolved = false;

                        let color_str = &style.reset_effects.outline_color;
                        if color_str != "invert" {
                            if let Some(c) = parse_css_color(color_str) {
                                outline_color = c;
                                resolved = true;
                            }
                        }

                        if !resolved {
                            if let Some(c) = parse_css_color(&style.inherited_text.color) {
                                outline_color = c;
                            } else {
                                outline_color = Color::Rgba(0, 0, 0, 255);
                            }
                        }

                        let scaled_color = scale_color_alpha(&outline_color, effective_opacity);

                        let style_keyword = outline_style.to_ascii_lowercase();

                        match style_keyword.as_str() {
                            "double" => {
                                let t_inner = (ow / 3.0).round().max(1.0);
                                let t_outer = (ow / 3.0).round().max(1.0);
                                let t_gap = ow - t_inner - t_outer;
                                if t_gap <= 0.0 || t_inner + t_outer >= ow {
                                    // fall back to solid
                                    paint_outline_strips(
                                        &mut items,
                                        rect,
                                        ow,
                                        offset,
                                        scaled_color,
                                    );
                                } else {
                                    // outer frame
                                    paint_outline_strips(
                                        &mut items,
                                        rect,
                                        t_outer,
                                        offset + ow - t_outer,
                                        scaled_color.clone(),
                                    );
                                    // inner frame
                                    paint_outline_strips(
                                        &mut items,
                                        rect,
                                        t_inner,
                                        offset,
                                        scaled_color,
                                    );
                                }
                            }
                            "dotted" => {
                                let dash_gap = (ow, ow);
                                // Top
                                paint_segmented_outline_edge(
                                    &mut items,
                                    true,
                                    Rect::new(x - d, y - d, w + 2.0 * d, ow),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Bottom
                                paint_segmented_outline_edge(
                                    &mut items,
                                    true,
                                    Rect::new(x - d, y + h + offset, w + 2.0 * d, ow),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Left
                                paint_segmented_outline_edge(
                                    &mut items,
                                    false,
                                    Rect::new(x - d, y - offset, ow, h + 2.0 * offset),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Right
                                paint_segmented_outline_edge(
                                    &mut items,
                                    false,
                                    Rect::new(x + w + offset, y - offset, ow, h + 2.0 * offset),
                                    dash_gap,
                                    scaled_color,
                                );
                            }
                            "dashed" => {
                                let dash_gap = (3.0 * ow, 3.0 * ow);
                                // Top
                                paint_segmented_outline_edge(
                                    &mut items,
                                    true,
                                    Rect::new(x - d, y - d, w + 2.0 * d, ow),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Bottom
                                paint_segmented_outline_edge(
                                    &mut items,
                                    true,
                                    Rect::new(x - d, y + h + offset, w + 2.0 * d, ow),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Left
                                paint_segmented_outline_edge(
                                    &mut items,
                                    false,
                                    Rect::new(x - d, y - offset, ow, h + 2.0 * offset),
                                    dash_gap,
                                    scaled_color.clone(),
                                );
                                // Right
                                paint_segmented_outline_edge(
                                    &mut items,
                                    false,
                                    Rect::new(x + w + offset, y - offset, ow, h + 2.0 * offset),
                                    dash_gap,
                                    scaled_color,
                                );
                            }
                            _ => {
                                // Fall back to solid for groove/ridge/inset/outset etc.
                                paint_outline_strips(&mut items, rect, ow, offset, scaled_color);
                            }
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

                    let is_disc = layout_box.text.as_deref() == Some("\u{2022}");
                    let is_circle = layout_box.text.as_deref() == Some("\u{25E6}");
                    let is_square = layout_box.text.as_deref() == Some("\u{25AA}");

                    if is_disc || is_circle || is_square {
                        // TODO(spec): a true circular `disc` or `circle` would need a filled or stroked-ellipse paint primitive;
                        // this uses a filled square as a pragmatic, ASCII-font-safe stand-in for `disc` (visually matches `list-style-type: square` which is correct),
                        // and a hollow square outline as a stand-in for `circle`. Image markers and `list-style-position: inside` remain out of scope.
                        let s = (corrected_rect.size.width.min(corrected_rect.size.height) * 0.5)
                            .clamp(2.0, 10.0);
                        let x = corrected_rect.origin.x + (corrected_rect.size.width - s) / 2.0;
                        let y = corrected_rect.origin.y + (corrected_rect.size.height - s) / 2.0;
                        let marker_rect = Rect::new(x, y, s, s);
                        let color = scale_color_alpha(&color, effective_opacity);

                        if is_circle {
                            let t = (s * 0.25).clamp(1.0, 2.0);
                            if s <= 2.0 * t {
                                items.push(DisplayItem::SolidRect {
                                    rect: marker_rect,
                                    color,
                                });
                            } else {
                                // Push four thin SolidRect edges
                                // Top
                                items.push(DisplayItem::SolidRect {
                                    rect: Rect::new(x, y, s, t),
                                    color: color.clone(),
                                });
                                // Bottom
                                items.push(DisplayItem::SolidRect {
                                    rect: Rect::new(x, y + s - t, s, t),
                                    color: color.clone(),
                                });
                                // Left
                                items.push(DisplayItem::SolidRect {
                                    rect: Rect::new(x, y + t, t, s - 2.0 * t),
                                    color: color.clone(),
                                });
                                // Right
                                items.push(DisplayItem::SolidRect {
                                    rect: Rect::new(x + s - t, y + t, t, s - 2.0 * t),
                                    color,
                                });
                            }
                        } else {
                            items.push(DisplayItem::SolidRect {
                                rect: marker_rect,
                                color,
                            });
                        }
                    } else {
                        // Paint text-shadow if present
                        if let Some(text_shadow_val) = resolve_text_shadow(dom, node_id, styles) {
                            let mut leaves = Vec::new();
                            flatten_value(&text_shadow_val, &mut leaves);

                            // Check for Keyword("none")
                            let mut is_none = false;
                            for leaf in &leaves {
                                if let CssValue::Keyword(kw) = leaf
                                    && kw.eq_ignore_ascii_case("none")
                                {
                                    is_none = true;
                                }
                            }

                            if !is_none && !leaves.is_empty() {
                                // Split leaves by comma to support multiple shadows
                                // CSS paints the first shadow on top, so paint later list entries
                                // first: iterate in reverse so the first shadow is pushed (and drawn) last.
                                let mut shadow_groups = Vec::new();
                                let mut current_group = Vec::new();
                                for leaf in leaves {
                                    if let CssValue::Keyword(kw) = leaf
                                        && kw == ","
                                    {
                                        if !current_group.is_empty() {
                                            shadow_groups.push(current_group);
                                            current_group = Vec::new();
                                        }
                                    } else {
                                        current_group.push(leaf);
                                    }
                                }
                                if !current_group.is_empty() {
                                    shadow_groups.push(current_group);
                                }

                                for group in shadow_groups.into_iter().rev() {
                                    let mut length_values = Vec::new();
                                    let mut color_value = None;

                                    for leaf in group {
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
                                        if length_values.len() >= 3 {
                                            // TODO(spec): text-shadow blur not rasterized
                                        }

                                        // Determine shadow color: default to resolved text color
                                        let shadow_color = if let Some(c) = color_value {
                                            c
                                        } else {
                                            color.clone()
                                        };

                                        let shadow_rect = Rect::new(
                                            corrected_rect.origin.x + offset_x,
                                            corrected_rect.origin.y + offset_y,
                                            corrected_rect.size.width,
                                            corrected_rect.size.height,
                                        );

                                        items.push(DisplayItem::Text {
                                            rect: shadow_rect,
                                            text: display_text.clone(),
                                            color: scale_color_alpha(
                                                &shadow_color,
                                                effective_opacity,
                                            ),
                                        });
                                    }
                                }
                            }
                        }

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
                            object_fit: get_object_fit(style),
                        });
                    } else {
                        let obj_fit = get_object_fit(style);
                        // Non-fill object-fit is delegated to the DisplayItem::Image path (raster).
                        let mut painted_as_pixels = false;
                        if obj_fit == ObjectFit::Fill
                            && let Some(bytes) =
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
                                object_fit: get_object_fit(style),
                            });
                        }
                    }
                }
            }

            if !node_hidden {
                if let Some(item) = btn_label_item {
                    items.push(item);
                }
                if let Some(item) = input_text_item {
                    items.push(item);
                }
                if let Some(item) = caret_item {
                    items.push(item);
                }
            }
        }

        // Pre-order traversal: process current, then children in painting order.
        // Since we use a stack (LIFO), we push children in reverse of their painting order.
        if !skip_children {
            let sorted_children = stacking::sort_siblings(&layout_box.children, styles);
            for child in sorted_children.into_iter().rev() {
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
    use crate::css::values::LengthUnit;
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
        let div_style = styles.get(&div).unwrap();
        println!("TEST_DIV_STYLE: {:?}", div_style);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;
        println!("TEST_PAINT_BASIC_ITEMS: {:?}", items);

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
    fn test_paint_outline_double() {
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
                outline: 6px double blue;
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

        // 2 concentric frames * 4 strips each = 8 strips total
        assert_eq!(rects.len(), 8, "Expected exactly 8 double outline strips");

        let blue = Color::Rgba(0, 0, 255, 255);
        let mut outer_top_found = false;
        let mut inner_top_found = false;

        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &blue);
                // Outer top rect: x = -6.0, y = -6.0, width = 112.0, height = 2.0
                if rect.origin.x == -6.0
                    && rect.origin.y == -6.0
                    && rect.size.width == 112.0
                    && rect.size.height == 2.0
                {
                    outer_top_found = true;
                }
                // Inner top rect: x = -2.0, y = -2.0, width = 104.0, height = 2.0
                else if rect.origin.x == -2.0
                    && rect.origin.y == -2.0
                    && rect.size.width == 104.0
                    && rect.size.height == 2.0
                {
                    inner_top_found = true;
                }
            }
        }

        assert!(
            outer_top_found,
            "Expected outer top strip of double outline"
        );
        assert!(
            inner_top_found,
            "Expected inner top strip of double outline"
        );
    }

    #[test]
    fn test_paint_outline_dotted() {
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
                outline: 4px dotted green;
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

        // Should have many small dot rects
        assert!(
            rects.len() > 10,
            "Expected dotted outline to produce many dot rects"
        );

        let green = Color::Rgba(0, 128, 0, 255);
        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &green);
                // The dots should have thickness 4.0 in either height or width
                assert!(rect.size.width <= 4.0 || rect.size.height <= 4.0);
            }
        }
    }

    #[test]
    fn test_paint_outline_dashed() {
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
                outline: 2px dashed red;
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

        // Should have many dash rects
        assert!(
            rects.len() > 10,
            "Expected dashed outline to produce many dash rects"
        );

        let red = Color::Rgba(255, 0, 0, 255);
        for item in rects {
            if let DisplayItem::SolidRect { rect, color } = item {
                assert_eq!(color, &red);
                // Dash length should be at most 3.0 * ow = 6.0
                assert!(rect.size.width <= 6.0 || rect.size.height <= 6.0);
            }
        }
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
    fn test_background_no_repeat_single_blit() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Inject 10x10 decoded image to DOM
        let img = crate::image::DecodedImage {
            width: 10,
            height: 10,
            rgba: vec![255; 400], // 10 * 10 * 4
        };
        dom.add_image("bg.png".to_string(), img);

        let stylesheet = parse_stylesheet(
            "div {
                width: 40px;
                height: 40px;
                background-image: url('bg.png');
                background-repeat: no-repeat;
            }",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(
            image_items.len(),
            1,
            "Expected exactly one background image tile with no-repeat"
        );

        if let DisplayItem::Image { rect, src, .. } = image_items[0] {
            assert_eq!(src, "bg.png");
            // The tile should be positioned at the box origin (0, 0)
            assert_eq!(rect.origin.x, 0.0);
            assert_eq!(rect.origin.y, 0.0);
            // The size should be the image's intrinsic size (10, 10)
            assert_eq!(rect.size.width, 10.0);
            assert_eq!(rect.size.height, 10.0);
        } else {
            panic!("Expected DisplayItem::Image");
        }
    }

    #[test]
    fn test_background_repeat_tiles() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Inject 10x10 decoded image to DOM
        let img = crate::image::DecodedImage {
            width: 10,
            height: 10,
            rgba: vec![255; 400],
        };
        dom.add_image("bg.png".to_string(), img);

        let stylesheet = parse_stylesheet(
            "div {
                width: 40px;
                height: 40px;
                background-image: url('bg.png');
                background-repeat: repeat;
            }",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        // 40x40 box with 10x10 repeating tiles should yield 4x4 = 16 tiles
        assert_eq!(
            image_items.len(),
            16,
            "Expected 16 background image tiles with repeat"
        );
    }

    #[test]
    fn test_background_position_offset() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![],
        });
        dom.append_child(doc, div);

        // Inject 10x10 decoded image to DOM
        let img = crate::image::DecodedImage {
            width: 10,
            height: 10,
            rgba: vec![255; 400],
        };
        dom.add_image("bg.png".to_string(), img);

        let stylesheet = parse_stylesheet(
            "div {
                width: 40px;
                height: 40px;
                background-image: url('bg.png');
                background-repeat: no-repeat;
                background-position: 10px 20px;
            }",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(
            image_items.len(),
            1,
            "Expected exactly one background image tile"
        );

        if let DisplayItem::Image { rect, src, .. } = image_items[0] {
            assert_eq!(src, "bg.png");
            // Positioned at (10, 20) relative to the box origin (which is (0,0))
            assert_eq!(rect.origin.x, 10.0);
            assert_eq!(rect.origin.y, 20.0);
            assert_eq!(rect.size.width, 10.0);
            assert_eq!(rect.size.height, 10.0);
        } else {
            panic!("Expected DisplayItem::Image");
        }
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
        assert_eq!(btn_style.reset_box.display, "inline-block");
        assert_ne!(btn_style.reset_background.background_color, "transparent");
        assert_ne!(btn_style.reset_surround.border_top_width, -1);

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

        // The child is given `display: block` so it has a box to paint. This test
        // has no UA stylesheet, where `span` would be `display: inline`; an *empty*
        // inline element has a zero-width box and would paint no visible background
        // (as in a real browser), which is unrelated to the visibility behavior under
        // test. (Pre-migration this relied on the initial display defaulting to a
        // block box; the typed style uses the spec-correct `inline` initial value.)
        let stylesheet = parse_stylesheet(
            "
            div { visibility: hidden; background-color: #ff0000; height: 10px; }
            span { display: block; visibility: visible; background-color: #0000ff; height: 10px; }
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

    // Guards comma-separated multiple box-shadows parsing and rendering order.
    #[test]
    fn test_box_shadow_multiple_comma_shadows() {
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

        let stylesheet = parse_stylesheet(
            "
            .shadowed {
                width: 100px;
                height: 50px;
                background-color: rgb(0, 255, 0);
            }
        ",
        );
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually insert multiple box-shadows since the CSS parser's parse_value does not support parsing commas yet
        if let Some(style) = styles.get_mut(&div_shadow) {
            let shadow_val = CssValue::Multiple(vec![
                CssValue::Length(5.0, LengthUnit::Px),
                CssValue::Length(5.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(255, 0, 0, 255)),
                CssValue::Keyword(",".to_string()),
                CssValue::Length(10.0, LengthUnit::Px),
                CssValue::Length(10.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(0, 0, 255, 255)),
            ]);
            style.insert("box-shadow".to_string(), shadow_val);
        }

        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Verify shadow rects are emitted.
        // The first listed shadow (5px 5px #ff0000) must overlay the second listed shadow (10px 10px #0000ff).
        // Therefore, the second listed shadow (blue) must be painted (pushed) BEFORE the first listed shadow (red).
        // And both must be painted before the background rect (green).
        let mut blue_idx = None;
        let mut red_idx = None;
        let mut bg_idx = None;

        let mut blue_rect = None;
        let mut red_rect = None;
        let mut bg_rect = None;

        for (idx, item) in items.iter().enumerate() {
            if let DisplayItem::SolidRect { rect, color } = item {
                if *color == Color::Rgba(0, 0, 255, 255) {
                    blue_idx = Some(idx);
                    blue_rect = Some(*rect);
                } else if *color == Color::Rgba(255, 0, 0, 255) {
                    red_idx = Some(idx);
                    red_rect = Some(*rect);
                } else if *color == Color::Rgba(0, 255, 0, 255) {
                    bg_idx = Some(idx);
                    bg_rect = Some(*rect);
                }
            }
        }

        let b_idx = blue_idx.expect("Should find blue shadow SolidRect");
        let r_idx = red_idx.expect("Should find red shadow SolidRect");
        let bg_idx = bg_idx.expect("Should find green background SolidRect");

        let b_rect = blue_rect.expect("Should find blue shadow rect");
        let r_rect = red_rect.expect("Should find red shadow rect");
        let bg_rect = bg_rect.expect("Should find green background rect");

        // Stacking order check:
        // Later listed shadows are painted first (drawn underneath), so blue must be before red.
        // Both shadows must be before the background.
        assert!(
            b_idx < r_idx,
            "Blue shadow (second listed) must be painted before red shadow (first listed)"
        );
        assert!(
            r_idx < bg_idx,
            "Red shadow (first listed) must be painted before green background"
        );

        // Check offset calculations
        // Blue shadow offset is 10px, 10px
        assert_eq!(
            b_rect.origin.x,
            bg_rect.origin.x + 10.0,
            "Blue shadow X origin should be offset by 10px"
        );
        assert_eq!(
            b_rect.origin.y,
            bg_rect.origin.y + 10.0,
            "Blue shadow Y origin should be offset by 10px"
        );

        // Red shadow offset is 5px, 5px
        assert_eq!(
            r_rect.origin.x,
            bg_rect.origin.x + 5.0,
            "Red shadow X origin should be offset by 5px"
        );
        assert_eq!(
            r_rect.origin.y,
            bg_rect.origin.y + 5.0,
            "Red shadow Y origin should be offset by 5px"
        );

        // Both shadows size must match background size
        assert_eq!(b_rect.size.width, bg_rect.size.width);
        assert_eq!(b_rect.size.height, bg_rect.size.height);
        assert_eq!(r_rect.size.width, bg_rect.size.width);
        assert_eq!(r_rect.size.height, bg_rect.size.height);
    }

    #[test]
    fn test_text_shadow_basic() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "shadowed".into())],
        });
        dom.append_child(body, div);

        let text_node = dom.create_node(NodeData::Text("paint".into()));
        dom.append_child(div, text_node);

        let stylesheet = parse_stylesheet(
            "
            .shadowed { color: #0000ff; }
        ",
        );
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually insert text-shadow on the div
        if let Some(style) = styles.get_mut(&div) {
            let shadow_val = CssValue::Multiple(vec![
                CssValue::Length(2.0, LengthUnit::Px),
                CssValue::Length(3.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(255, 0, 0, 255)),
            ]);
            style.insert("text-shadow".to_string(), shadow_val);
        }

        let layout = layout_document(&dom, &styles, 800.0);
        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            text_items.len(),
            2,
            "Should emit exactly two text items (one shadow, one main)"
        );

        if let DisplayItem::Text {
            rect: shadow_rect,
            text: shadow_txt,
            color: shadow_col,
        } = text_items[0]
        {
            assert_eq!(shadow_txt, "paint");
            assert_eq!(*shadow_col, Color::Rgba(255, 0, 0, 255));

            if let DisplayItem::Text {
                rect: main_rect,
                text: main_txt,
                color: main_col,
            } = text_items[1]
            {
                assert_eq!(main_txt, "paint");
                assert_eq!(*main_col, Color::Rgba(0, 0, 255, 255));

                // Verify offset
                assert_eq!(shadow_rect.origin.x, main_rect.origin.x + 2.0);
                assert_eq!(shadow_rect.origin.y, main_rect.origin.y + 3.0);
            } else {
                panic!("Second item should be the main text");
            }
        } else {
            panic!("First item should be the text shadow");
        }
    }

    #[test]
    fn test_text_shadow_absent() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "normal".into())],
        });
        dom.append_child(body, div);

        let text_node = dom.create_node(NodeData::Text("paint".into()));
        dom.append_child(div, text_node);

        let stylesheet = parse_stylesheet(
            "
            .normal { color: #0000ff; }
        ",
        );
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually insert text-shadow: none
        if let Some(style) = styles.get_mut(&div) {
            style.insert(
                "text-shadow".to_string(),
                CssValue::Keyword("none".to_string()),
            );
        }

        let layout = layout_document(&dom, &styles, 800.0);
        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            text_items.len(),
            1,
            "Should emit exactly one text item since text-shadow: none"
        );
    }

    #[test]
    fn test_text_shadow_multiple() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let div = dom.create_node(NodeData::Element {
            name: "div".into(),
            attrs: vec![("class".into(), "multi-shadowed".into())],
        });
        dom.append_child(body, div);

        let text_node = dom.create_node(NodeData::Text("paint".into()));
        dom.append_child(div, text_node);

        let stylesheet = parse_stylesheet(
            "
            .multi-shadowed { color: #0000ff; }
        ",
        );
        let mut styles = compute_styles(&dom, &stylesheet);

        // Manually insert multiple text-shadows
        if let Some(style) = styles.get_mut(&div) {
            let shadow_val = CssValue::Multiple(vec![
                CssValue::Length(1.0, LengthUnit::Px),
                CssValue::Length(2.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(0, 255, 0, 255)),
                CssValue::Keyword(",".to_string()),
                CssValue::Length(3.0, LengthUnit::Px),
                CssValue::Length(4.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(255, 0, 0, 255)),
            ]);
            style.insert("text-shadow".to_string(), shadow_val);
        }

        let layout = layout_document(&dom, &styles, 800.0);
        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let text_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Text { .. }))
            .collect();

        assert_eq!(
            text_items.len(),
            3,
            "Should emit exactly three text items (two shadows, one main)"
        );

        // 1. Second shadow (red)
        if let DisplayItem::Text {
            rect: s2_rect,
            text: s2_txt,
            color: s2_col,
        } = text_items[0]
        {
            assert_eq!(s2_txt, "paint");
            assert_eq!(*s2_col, Color::Rgba(255, 0, 0, 255));

            // 2. First shadow (green)
            if let DisplayItem::Text {
                rect: s1_rect,
                text: s1_txt,
                color: s1_col,
            } = text_items[1]
            {
                assert_eq!(s1_txt, "paint");
                assert_eq!(*s1_col, Color::Rgba(0, 255, 0, 255));

                // 3. Main text (blue)
                if let DisplayItem::Text {
                    rect: main_rect,
                    text: main_txt,
                    color: main_col,
                } = text_items[2]
                {
                    assert_eq!(main_txt, "paint");
                    assert_eq!(*main_col, Color::Rgba(0, 0, 255, 255));

                    // Verify offsets
                    assert_eq!(s2_rect.origin.x, main_rect.origin.x + 3.0);
                    assert_eq!(s2_rect.origin.y, main_rect.origin.y + 4.0);

                    assert_eq!(s1_rect.origin.x, main_rect.origin.x + 1.0);
                    assert_eq!(s1_rect.origin.y, main_rect.origin.y + 2.0);
                } else {
                    panic!("Third item should be main text");
                }
            } else {
                panic!("Second item should be first shadow");
            }
        } else {
            panic!("First item should be second shadow");
        }
    }

    #[test]
    fn test_ul_disc_marker_paints_solid_rect_not_tofu_glyph() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        let ul = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![],
        });
        dom.append_child(body, ul);

        let li = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul, li);

        let text = dom.create_node(NodeData::Text("item".into()));
        dom.append_child(li, text);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ul { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; list-style-type: disc; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 500.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_bullet_text = false;
        let mut bullet_rect = None;
        let mut item_text_origin_x = None;

        for item in &items {
            match item {
                DisplayItem::Text { text, rect, .. } => {
                    if text == "\u{2022}" {
                        found_bullet_text = true;
                    } else if text == "item" {
                        item_text_origin_x = Some(rect.origin.x);
                    }
                }
                DisplayItem::SolidRect { rect, .. }
                    if rect.size.width <= 10.0 && rect.size.height <= 10.0 =>
                {
                    bullet_rect = Some(*rect);
                }
                _ => {}
            }
        }

        // 1. Assert NO DisplayItem::Text item has text == "\u{2022}"
        assert!(
            !found_bullet_text,
            "Bullet glyph should not be drawn as text"
        );

        // 2. Assert at least one DisplayItem::SolidRect exists whose rect is the small marker square positioned to the LEFT of the list item's content
        if let Some(b_rect) = bullet_rect {
            if let Some(t_x) = item_text_origin_x {
                assert!(
                    b_rect.origin.x < t_x,
                    "Marker (x={}) should be to the left of the item text (x={})",
                    b_rect.origin.x,
                    t_x
                );
            } else {
                panic!(
                    "Did not find 'item' text fragment in display items: {:?}",
                    items
                );
            }
        } else {
            panic!(
                "Did not find bullet SolidRect in display items: {:?}",
                items
            );
        }
    }

    #[test]
    fn test_ul_circle_and_square_markers_paint_solid_rect_not_tofu() {
        let mut dom = Dom::new();
        let doc = dom.document();
        let body = dom.create_node(NodeData::Element {
            name: "body".into(),
            attrs: vec![],
        });
        dom.append_child(doc, body);

        // Circle List
        let ul_circle = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("class".into(), "circle-list".into())],
        });
        dom.append_child(body, ul_circle);

        let li_circle = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_circle, li_circle);

        let text_circle = dom.create_node(NodeData::Text("circle-item".into()));
        dom.append_child(li_circle, text_circle);

        // Square List
        let ul_square = dom.create_node(NodeData::Element {
            name: "ul".into(),
            attrs: vec![("class".into(), "square-list".into())],
        });
        dom.append_child(body, ul_square);

        let li_square = dom.create_node(NodeData::Element {
            name: "li".into(),
            attrs: vec![],
        });
        dom.append_child(ul_square, li_square);

        let text_square = dom.create_node(NodeData::Text("square-item".into()));
        dom.append_child(li_square, text_square);

        let stylesheet = parse_stylesheet(
            "
            body { display: block; width: 500px; }
            ul.circle-list { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; list-style-type: circle; }
            ul.square-list { display: block; padding-left: 40px; margin-top: 16px; margin-bottom: 16px; list-style-type: square; }
            li { display: block; }
        ",
        );
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 500.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        let mut found_circle_glyph = false;
        let mut found_square_glyph = false;
        let mut circle_item_text_x = None;
        let mut square_item_text_x = None;
        let mut marker_rects = Vec::new();

        for item in &items {
            match item {
                DisplayItem::Text { text, rect, .. } => {
                    if text == "\u{25E6}" {
                        found_circle_glyph = true;
                    } else if text == "\u{25AA}" {
                        found_square_glyph = true;
                    } else if text == "circle-item" {
                        circle_item_text_x = Some(rect.origin.x);
                    } else if text == "square-item" {
                        square_item_text_x = Some(rect.origin.x);
                    }
                }
                DisplayItem::SolidRect { rect, .. }
                    if rect.size.width <= 10.0 && rect.size.height <= 10.0 =>
                {
                    marker_rects.push(*rect);
                }
                _ => {}
            }
        }

        // 1. Assert NO DisplayItem::Text item has text equal to \u{25E6} or \u{25AA}
        assert!(
            !found_circle_glyph,
            "Circle bullet glyph should not be drawn as text (tofu)"
        );
        assert!(
            !found_square_glyph,
            "Square bullet glyph should not be drawn as text (tofu)"
        );

        // 2. Assert that markers are positioned to the LEFT of the corresponding text items
        let circle_text_x = circle_item_text_x.expect("Did not find 'circle-item' text");
        let square_text_x = square_item_text_x.expect("Did not find 'square-item' text");

        // There should be at least some SolidRects representing the markers (square marker is 1, circle marker outline has 4)
        assert!(
            !marker_rects.is_empty(),
            "Should have emitted marker SolidRects"
        );

        // Check if we have at least one SolidRect positioned to the left of each item's text
        let has_circle_marker = marker_rects.iter().any(|r| r.origin.x < circle_text_x);
        let has_square_marker = marker_rects.iter().any(|r| r.origin.x < square_text_x);

        assert!(
            has_circle_marker,
            "Should have a marker to the left of 'circle-item'"
        );
        assert!(
            has_square_marker,
            "Should have a marker to the left of 'square-item'"
        );
    }

    #[test]
    fn test_paint_image_non_square_object_fit_contain() {
        use crate::raster::Canvas;

        // Generate 2x1 image: Red on left, Green on right
        let mut source_canvas = Canvas::new(2, 1);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_paint_test_image_contain.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), temp_filename.into())],
        });
        dom.append_child(doc, img);

        // Apply object-fit: contain to a non-square container
        let stylesheet = parse_stylesheet("img { width: 4px; height: 4px; object-fit: contain; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Clean up the temp file
        let _ = std::fs::remove_file(temp_filename);

        // We expect:
        // 1. One DisplayItem::Image item with ObjectFit::Contain and decoded: None (which will be processed by raster with correct object-fit logic)
        // 2. ZERO SolidRect items because it should NOT paint as pixels (blit fallback is gated)
        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert_eq!(
            image_items.len(),
            1,
            "Expected exactly one Image display item"
        );
        if let DisplayItem::Image {
            object_fit,
            decoded,
            ..
        } = image_items[0]
        {
            assert_eq!(*object_fit, ObjectFit::Contain);
            assert!(
                decoded.is_none(),
                "Expected decoded to be None (fallback path)"
            );
        } else {
            panic!("Expected DisplayItem::Image");
        }

        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert!(
            solid_rects.is_empty(),
            "Expected 0 SolidRects since blit path should be bypassed for non-Fill object-fit"
        );
    }

    #[test]
    fn test_paint_image_non_square_object_fit_regression() {
        use crate::raster::Canvas;

        // Generate 2x1 image: Red on left, Green on right
        let mut source_canvas = Canvas::new(2, 1);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_paint_test_image_regression_fill.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        let mut dom = Dom::new();
        let doc = dom.document();
        let img = dom.create_node(NodeData::Element {
            name: "img".into(),
            attrs: vec![("src".into(), temp_filename.into())],
        });
        dom.append_child(doc, img);

        // No object-fit specified (defaults to fill)
        let stylesheet = parse_stylesheet("img { width: 4px; height: 4px; }");
        let styles = compute_styles(&dom, &stylesheet);
        let layout = layout_document(&dom, &styles, 800.0);

        let display_list = build_display_list(&layout, &dom, &styles);
        let items = display_list.0;

        // Clean up the temp file
        let _ = std::fs::remove_file(temp_filename);

        // We expect:
        // 1. Two SolidRects (representing the 2x1 pixels of the image)
        // 2. ZERO DisplayItem::Image items
        let solid_rects: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::SolidRect { .. }))
            .collect();

        assert_eq!(
            solid_rects.len(),
            2,
            "Expected 2 SolidRects representing decoded PNG pixels for Fill default"
        );

        let image_items: Vec<&DisplayItem> = items
            .iter()
            .filter(|item| matches!(item, DisplayItem::Image { .. }))
            .collect();

        assert!(
            image_items.is_empty(),
            "Expected 0 Image items since default Fill stretches using inline blit"
        );
    }

    #[test]
    fn test_paint_input_value_and_placeholder() {
        // Case 1: Input with non-empty value
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("value".into(), "hello".into()),
                ],
            });
            dom.append_child(doc, input);

            let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
            css.push_str("input { width: 100px; height: 30px; color: rgb(255, 0, 0); }");
            let stylesheet = parse_stylesheet(&css);
            let styles = compute_styles(&dom, &stylesheet);
            let layout = layout_document(&dom, &styles, 100.0);

            let display_list = build_display_list(&layout, &dom, &styles);
            let items = display_list.0;

            let text_items: Vec<&DisplayItem> = items
                .iter()
                .filter(|item| matches!(item, DisplayItem::Text { .. }))
                .collect();

            assert_eq!(text_items.len(), 1);
            if let DisplayItem::Text { text, color, rect } = text_items[0] {
                assert_eq!(text, "hello");
                assert_eq!(color, &Color::Rgba(255, 0, 0, 255));
                assert!(rect.origin.x > layout.rect.origin.x); // Left padding applied
            } else {
                panic!("Expected Text display item");
            }
        }

        // Case 2: Input with only non-empty placeholder
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("placeholder".into(), "Search...".into()),
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

            let text_items: Vec<&DisplayItem> = items
                .iter()
                .filter(|item| matches!(item, DisplayItem::Text { .. }))
                .collect();

            assert_eq!(text_items.len(), 1);
            if let DisplayItem::Text { text, color, .. } = text_items[0] {
                assert_eq!(text, "Search...");
                assert_eq!(color, &Color::Rgba(117, 117, 117, 255)); // Gray color
            } else {
                panic!("Expected Text display item");
            }
        }

        // Case 3: Input with both value and placeholder (value takes precedence)
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("value".into(), "real value".into()),
                    ("placeholder".into(), "Search...".into()),
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

            let text_items: Vec<&DisplayItem> = items
                .iter()
                .filter(|item| matches!(item, DisplayItem::Text { .. }))
                .collect();

            assert_eq!(text_items.len(), 1);
            if let DisplayItem::Text { text, color, .. } = text_items[0] {
                assert_eq!(text, "real value");
                assert_eq!(color, &Color::Rgba(0, 0, 0, 255)); // Default color
            } else {
                panic!("Expected Text display item");
            }
        }

        // Case 4: Input with empty value and empty placeholder
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("value".into(), "".into()),
                    ("placeholder".into(), "".into()),
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

            let text_items: Vec<&DisplayItem> = items
                .iter()
                .filter(|item| matches!(item, DisplayItem::Text { .. }))
                .collect();

            assert!(
                text_items.is_empty(),
                "Should not paint any text if both value and placeholder are empty"
            );
        }
    }

    #[test]
    fn test_paint_input_caret_rendering() {
        let font = crate::font::BitmapFont::builtin();
        let font_h = font.line_height() as f32;

        // Test 1: Focused empty input
        {
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

            let input_rect =
                crate::layout::find_box_rect(&layout, input).expect("Input rect should exist");

            // Build display list with caret focused on this input
            let display_list =
                build_display_list_with_caret(&layout, &dom, &styles, Some((input, 0)));
            let items = display_list.0;

            // Find caret SolidRect item. Remember UA chrome of input also emits SolidRects (e.g. background & border),
            // so we should identify the thin caret rect: width == 1.0, height == font_h.
            let caret_item = items
                .iter()
                .find(|item| {
                    if let DisplayItem::SolidRect { rect, .. } = item {
                        rect.size.width == 1.0 && rect.size.height == font_h
                    } else {
                        false
                    }
                })
                .expect("Should find a caret SolidRect");

            if let DisplayItem::SolidRect { rect, color } = caret_item {
                let input_x = input_rect.origin.x;
                assert!((rect.origin.x - (input_x + 2.0)).abs() < 0.1);
                assert_eq!(color, &Color::Rgba(0, 0, 0, 255));
            }
        }

        // Test 2: Focused input with value "ab" and caret_index 2
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("value".into(), "ab".into()),
                ],
            });
            dom.append_child(doc, input);

            let mut css = crate::engine::UA_DEFAULT_CSS.to_string();
            css.push_str("input { width: 100px; height: 30px; }");
            let stylesheet = parse_stylesheet(&css);
            let styles = compute_styles(&dom, &stylesheet);
            let layout = layout_document(&dom, &styles, 100.0);

            let input_rect =
                crate::layout::find_box_rect(&layout, input).expect("Input rect should exist");

            let display_list =
                build_display_list_with_caret(&layout, &dom, &styles, Some((input, 2)));
            let items = display_list.0;

            let caret_item = items
                .iter()
                .find(|item| {
                    if let DisplayItem::SolidRect { rect, .. } = item {
                        rect.size.width == 1.0 && rect.size.height == font_h
                    } else {
                        false
                    }
                })
                .expect("Should find a caret SolidRect");

            if let DisplayItem::SolidRect { rect, .. } = caret_item {
                let expected_x = input_rect.origin.x + 2.0 + font.measure("ab") as f32;
                assert!((rect.origin.x - expected_x).abs() < 0.1);
            }
        }

        // Test 3: Regression: build_display_list yields NO caret rect
        {
            let mut dom = Dom::new();
            let doc = dom.document();
            let input = dom.create_node(NodeData::Element {
                name: "input".into(),
                attrs: vec![
                    ("type".into(), "text".into()),
                    ("value".into(), "ab".into()),
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

            // Caret rect should NOT be present (since we pass None for caret)
            let caret_item = items.iter().find(|item| {
                if let DisplayItem::SolidRect { rect, .. } = item {
                    rect.size.width == 1.0 && rect.size.height == font_h
                } else {
                    false
                }
            });
            assert!(
                caret_item.is_none(),
                "Caret should not render when caret state is None"
            );

            // But the text should still render
            let text_item = items
                .iter()
                .find(|item| matches!(item, DisplayItem::Text { text, .. } if text == "ab"));
            assert!(text_item.is_some(), "Value text 'ab' should still render");
        }
    }

    #[test]
    fn test_bg_size_contain() {
        let val = CssValue::Keyword("contain".to_string());
        let res = resolve_bg_size((200.0, 100.0), (50.0, 50.0), &val);
        assert_eq!(res, (100.0, 100.0));
    }

    #[test]
    fn test_bg_size_cover() {
        let val = CssValue::Keyword("cover".to_string());
        let res = resolve_bg_size((200.0, 100.0), (50.0, 50.0), &val);
        assert_eq!(res, (200.0, 200.0));
    }

    #[test]
    fn test_bg_size_auto() {
        let val = CssValue::Keyword("auto".to_string());
        let res = resolve_bg_size((200.0, 100.0), (50.0, 50.0), &val);
        assert_eq!(res, (50.0, 50.0));
    }

    #[test]
    fn test_bg_size_zero_intrinsic_safe() {
        let val = CssValue::Keyword("contain".to_string());
        let res = resolve_bg_size((200.0, 100.0), (0.0, 0.0), &val);
        assert_eq!(res, (0.0, 0.0));
    }

    #[test]
    fn test_bg_size_single_length() {
        let val = CssValue::Length(100.0, LengthUnit::Px);
        let res = resolve_bg_size((200.0, 100.0), (50.0, 25.0), &val);
        // width 100, height auto (preserving ratio: 25/50 = 0.5) -> (100.0, 50.0)
        assert_eq!(res, (100.0, 50.0));
    }

    #[test]
    fn test_bg_size_multiple_length_and_auto() {
        let val = CssValue::Multiple(vec![
            CssValue::Keyword("auto".to_string()),
            CssValue::Length(50.0, LengthUnit::Percent),
        ]);
        let res = resolve_bg_size((200.0, 100.0), (50.0, 25.0), &val);
        // height 50% of 100.0 is 50.0. width is auto (preserving ratio: 50 * (50/25) = 100.0) -> (100.0, 50.0)
        assert_eq!(res, (100.0, 50.0));
    }
}
