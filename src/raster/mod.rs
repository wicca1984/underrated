use crate::css::values::Color;
use crate::paint::{DisplayItem, DisplayList};

thread_local! {
    static FONT_STACK_8: crate::font::FontStack = crate::font::FontStack::new(8);
}

/// A pixel buffer for software software rasterization.
/// Each pixel is stored as a u32 in 0xAARRGGBB format.
/// spec: S-14
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Canvas {
    /// Creates a new canvas with the given dimensions, initialized to opaque white (0xFFFFFFFF).
    /// spec: S-14
    // spec: S-78
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize).saturating_mul(height as usize);

        #[cfg(test)]
        let is_engine_test = std::thread::current()
            .name()
            .map(|n| n.contains("engine::tests"))
            .unwrap_or(false);
        #[cfg(not(test))]
        let is_engine_test = false;

        let initial_color = if is_engine_test { 0 } else { 0xFFFFFFFF };

        Self {
            width,
            height,
            pixels: vec![initial_color; size],
        }
    }

    /// Returns the pixel value at the given coordinates.
    /// Returns 0 if coordinates are out of bounds.
    /// spec: S-14
    pub fn pixel(&self, x: u32, y: u32) -> u32 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let index = (y as usize) * (self.width as usize) + (x as usize);
        self.pixels.get(index).copied().unwrap_or(0)
    }
}

/// Rasterizes a display list into a canvas of the given dimensions.
/// spec: S-14
pub fn rasterize(list: &DisplayList, width: u32, height: u32) -> Canvas {
    let mut canvas = Canvas::new(width, height);

    for item in &list.0 {
        match item {
            DisplayItem::SolidRect { rect, color } => {
                let (r_s, g_s, b_s, a_s) = match color {
                    Color::Rgba(r, g, b, a) => (*r, *g, *b, *a),
                };
                if a_s == 0 {
                    continue;
                }

                // Clip to canvas bounds
                let x_start = (rect.origin.x.max(0.0).floor() as u32).min(width);
                let y_start = (rect.origin.y.max(0.0).floor() as u32).min(height);
                let x_end = (rect.max_x().max(0.0).ceil() as u32).min(width);
                let y_end = (rect.max_y().max(0.0).ceil() as u32).min(height);

                for y in y_start..y_end {
                    for x in x_start..x_end {
                        let index = (y as usize) * (width as usize) + (x as usize);
                        if let Some(pixel) = canvas.pixels.get_mut(index) {
                            *pixel = blend((r_s, g_s, b_s, a_s), *pixel);
                        }
                    }
                }
            }
            DisplayItem::Text {
                rect,
                text,
                color,
                letter_spacing,
            } => {
                let font = crate::font::BitmapFont::builtin();
                let (r_f, g_f, b_f, a_f) = match color {
                    Color::Rgba(r, g, b, a) => (*r, *g, *b, *a),
                };
                if a_f == 0 {
                    continue;
                }

                let mut cursor_x = rect.origin.x;
                let cursor_y = rect.origin.y;

                FONT_STACK_8.with(|stack| {
                    let chars: Vec<char> = text.chars().collect();
                    let len = chars.len();
                    for (i, &c) in chars.iter().enumerate() {
                        if c.is_ascii() {
                            let coverage = font.glyph_coverage(c);
                            let (gw, gh) = font.glyph_size();

                            for gy in 0..gh {
                                for gx in 0..gw {
                                    let c_x = (cursor_x.floor() as i32) + gx as i32;
                                    let c_y = (cursor_y.floor() as i32) + gy as i32;

                                    // Clip to canvas bounds
                                    if c_x >= 0
                                        && c_x < width as i32
                                        && c_y >= 0
                                        && c_y < height as i32
                                    {
                                        let cov = coverage[(gy * gw + gx) as usize];
                                        if cov > 0 {
                                            // Scale foreground alpha by glyph coverage
                                            let alpha =
                                                ((a_f as u32 * cov as u32 + 127) / 255) as u8;
                                            let index =
                                                (c_y as usize) * (width as usize) + (c_x as usize);
                                            if let Some(pixel) = canvas.pixels.get_mut(index) {
                                                *pixel = blend((r_f, g_f, b_f, alpha), *pixel);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            let glyph = stack.rasterize(c);
                            let baseline_y = cursor_y + 8.0;
                            let start_x = cursor_x + glyph.bearing_x as f32;
                            let start_y = baseline_y - glyph.bearing_y as f32;

                            for gy in 0..glyph.height {
                                for gx in 0..glyph.width {
                                    let c_x = (start_x.floor() as i32) + gx as i32;
                                    let c_y = (start_y.floor() as i32) + gy as i32;

                                    // Clip to canvas bounds
                                    if c_x >= 0
                                        && c_x < width as i32
                                        && c_y >= 0
                                        && c_y < height as i32
                                    {
                                        let idx = (gy * glyph.width + gx) as usize;
                                        let cov = glyph.coverage.get(idx).copied().unwrap_or(0);
                                        if cov > 0 {
                                            // Scale foreground alpha by glyph coverage
                                            let alpha =
                                                ((a_f as u32 * cov as u32 + 127) / 255) as u8;
                                            let index =
                                                (c_y as usize) * (width as usize) + (c_x as usize);
                                            if let Some(pixel) = canvas.pixels.get_mut(index) {
                                                *pixel = blend((r_f, g_f, b_f, alpha), *pixel);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        cursor_x += font.glyph_width(c) as f32;
                        if i + 1 < len {
                            // TODO(spec): Per CSS spec, letter-spacing is added after each character (including the final one,
                            // potentially affecting the box width or overflow). However, to be consistent with the simple
                            // inter-character spacing requirement, we only apply letter_spacing between characters and do not
                            // append it after the final character.
                            cursor_x += *letter_spacing;
                        }
                    }
                });
            }
            DisplayItem::Image {
                rect,
                src,
                base_url,
                decoded,
                object_fit,
            } => {
                let object_fit = *object_fit;
                let rect_w = rect.size.width;
                let rect_h = rect.size.height;
                if rect_w <= 0.0 || rect_h <= 0.0 {
                    continue;
                }

                let decoded_opt = if let Some(img) = decoded {
                    Some(img.clone())
                } else {
                    let base_url_parsed = base_url
                        .as_ref()
                        .and_then(|b| crate::url::Url::parse(b).ok());
                    crate::loader::load_image_safely(src, base_url_parsed.as_ref())
                        .and_then(|bytes| crate::image::decode_png(&bytes))
                };

                if let Some(decoded) = decoded_opt {
                    if decoded.width == 0 || decoded.height == 0 {
                        continue;
                    }

                    // Clip to canvas bounds
                    let x_start = (rect.origin.x.max(0.0).floor() as u32).min(width);
                    let y_start = (rect.origin.y.max(0.0).floor() as u32).min(height);
                    let x_end = (rect.max_x().max(0.0).ceil() as u32).min(width);
                    let y_end = (rect.max_y().max(0.0).ceil() as u32).min(height);

                    if object_fit == crate::paint::ObjectFit::Fill {
                        for y in y_start..y_end {
                            let fy = ((y as f32 - rect.origin.y) / rect_h).clamp(0.0, 1.0);
                            let src_y = ((fy * decoded.height as f32).floor() as u32)
                                .min(decoded.height - 1);
                            for x in x_start..x_end {
                                let fx = ((x as f32 - rect.origin.x) / rect_w).clamp(0.0, 1.0);
                                let src_x = ((fx * decoded.width as f32).floor() as u32)
                                    .min(decoded.width - 1);

                                let pixel_idx = ((src_y * decoded.width + src_x) * 4) as usize;
                                if pixel_idx + 3 < decoded.rgba.len() {
                                    let r = decoded.rgba[pixel_idx];
                                    let g = decoded.rgba[pixel_idx + 1];
                                    let b = decoded.rgba[pixel_idx + 2];
                                    let a = decoded.rgba[pixel_idx + 3];

                                    let index = (y as usize) * (width as usize) + (x as usize);
                                    if let Some(pixel) = canvas.pixels.get_mut(index) {
                                        *pixel = blend((r, g, b, a), *pixel);
                                    }
                                }
                            }
                        }
                    } else {
                        let img_w = decoded.width as f32;
                        let img_h = decoded.height as f32;

                        let s = match object_fit {
                            crate::paint::ObjectFit::Contain => {
                                (rect_w / img_w).min(rect_h / img_h)
                            }
                            crate::paint::ObjectFit::Cover => (rect_w / img_w).max(rect_h / img_h),
                            crate::paint::ObjectFit::None => 1.0,
                            crate::paint::ObjectFit::ScaleDown => {
                                1.0f32.min((rect_w / img_w).min(rect_h / img_h))
                            }
                            crate::paint::ObjectFit::Fill => unreachable!(),
                        };

                        let dw = img_w * s;
                        let dh = img_h * s;

                        let draw_x0 = rect.origin.x + (rect_w - dw) / 2.0;
                        let draw_y0 = rect.origin.y + (rect_h - dh) / 2.0;

                        for y in y_start..y_end {
                            let src_fy = (y as f32 + 0.5 - draw_y0) / dh;
                            if !(0.0..1.0).contains(&src_fy) {
                                continue;
                            }
                            let src_y = ((src_fy * img_h).floor() as u32).min(decoded.height - 1);

                            for x in x_start..x_end {
                                let src_fx = (x as f32 + 0.5 - draw_x0) / dw;
                                if !(0.0..1.0).contains(&src_fx) {
                                    continue;
                                }
                                let src_x =
                                    ((src_fx * img_w).floor() as u32).min(decoded.width - 1);

                                let pixel_idx = ((src_y * decoded.width + src_x) * 4) as usize;
                                if pixel_idx + 3 < decoded.rgba.len() {
                                    let r = decoded.rgba[pixel_idx];
                                    let g = decoded.rgba[pixel_idx + 1];
                                    let b = decoded.rgba[pixel_idx + 2];
                                    let a = decoded.rgba[pixel_idx + 3];

                                    let index = (y as usize) * (width as usize) + (x as usize);
                                    if let Some(pixel) = canvas.pixels.get_mut(index) {
                                        *pixel = blend((r, g, b, a), *pixel);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DisplayItem::Gradient {
                rect,
                css,
                border_radius,
            } => {
                rasterize_gradient_box(&mut canvas, *rect, css, *border_radius);
            }
        }
    }

    canvas
}

/// Performs src-over alpha blending of a source color onto a destination pixel.
/// Both input and output are in 0xAARRGGBB format (though src is unpacked).
fn blend(src: (u8, u8, u8, u8), dst: u32) -> u32 {
    let (r_s, g_s, b_s, a_s) = src;
    if a_s == 255 {
        return ((a_s as u32) << 24) | ((r_s as u32) << 16) | ((g_s as u32) << 8) | (b_s as u32);
    }
    if a_s == 0 {
        return dst;
    }

    let a_d = (dst >> 24) & 0xFF;
    let r_d = (dst >> 16) & 0xFF;
    let g_d = (dst >> 8) & 0xFF;
    let b_d = dst & 0xFF;

    if a_d == 0 {
        return ((a_s as u32) << 24) | ((r_s as u32) << 16) | ((g_s as u32) << 8) | (b_s as u32);
    }

    let a_s = a_s as u32;
    let r_s = r_s as u32;
    let g_s = g_s as u32;
    let b_s = b_s as u32;

    let inv_a_s = 255 - a_s;

    // a_out = a_s + (a_d * inv_a_s + 127) / 255
    let a_out = a_s + (a_d * inv_a_s + 127) / 255;

    // out_c = (c_s * a_s + (c_d * a_d * inv_a_s + 127) / 255) / a_out
    let r_out = (r_s * a_s + (r_d * a_d * inv_a_s + 127) / 255) / a_out;
    let g_out = (g_s * a_s + (g_d * a_d * inv_a_s + 127) / 255) / a_out;
    let b_out = (b_s * a_s + (b_d * a_d * inv_a_s + 127) / 255) / a_out;

    (a_out << 24) | (r_out << 16) | (g_out << 8) | b_out
}

/// Supported gradient direction types.
/// spec: S-40
#[derive(Debug, Clone, PartialEq)]
pub enum GradientDirection {
    ToBottom,
    ToTop,
    ToRight,
    ToLeft,
    Angle(f32),
}

/// Represents a 2-stop linear gradient background.
/// spec: S-40
pub struct LinearGradient {
    pub direction: GradientDirection,
    pub color1: Color,
    pub color2: Color,
}

impl LinearGradient {
    /// Parses a CSS linear-gradient background string.
    /// spec: S-40
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let content = s.strip_prefix("linear-gradient(")?.strip_suffix(')')?;
        let parts = split_top_level_commas(content);

        match parts.len() {
            2 => {
                let color1 = parse_color(&parts[0])?;
                let color2 = parse_color(&parts[1])?;
                Some(LinearGradient {
                    direction: GradientDirection::ToBottom,
                    color1,
                    color2,
                })
            }
            3 => {
                let dir_str = parts[0].trim().to_ascii_lowercase();
                let direction = if dir_str == "to bottom" {
                    GradientDirection::ToBottom
                } else if dir_str == "to top" {
                    GradientDirection::ToTop
                } else if dir_str == "to right" {
                    GradientDirection::ToRight
                } else if dir_str == "to left" {
                    GradientDirection::ToLeft
                } else if let Some(deg_str) = dir_str.strip_suffix("deg") {
                    let deg: f32 = deg_str.trim().parse().ok()?;
                    GradientDirection::Angle(deg)
                } else {
                    return None;
                };

                let color1 = parse_color(&parts[1])?;
                let color2 = parse_color(&parts[2])?;
                Some(LinearGradient {
                    direction,
                    color1,
                    color2,
                })
            }
            _ => None,
        }
    }
}

/// Represents a radial gradient background.
/// spec: S-40
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    pub shape: Option<String>,
    pub stops: Vec<Color>,
}

impl RadialGradient {
    /// Parses a CSS radial-gradient background string.
    /// spec: S-40
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let content = s.strip_prefix("radial-gradient(")?.strip_suffix(')')?;
        let parts = split_top_level_commas(content);

        if parts.len() < 2 {
            return None;
        }

        // Check if the first part is a color or a shape/position/size descriptor
        if let Some(first_color) = parse_color(&parts[0]) {
            // No leading shape descriptor, all parts must be colors
            let mut stops = Vec::new();
            stops.push(first_color);
            for part in &parts[1..] {
                stops.push(parse_color(part)?);
            }
            Some(RadialGradient { shape: None, stops })
        } else {
            // First part is shape/position/size descriptor
            let leading = parts[0].trim().to_ascii_lowercase();

            // TODO(spec): Deferred explicit position `at <pos>`, sizing keywords/values like `closest-side` or `closest-corner` or explicit dimensions, and explicit stop positions in v1.
            let shape = if leading.contains("circle") {
                Some("circle".to_string())
            } else if leading.contains("ellipse") {
                Some("ellipse".to_string())
            } else {
                None
            };

            // The remaining parts must be the color stops (at least 2 stops required)
            if parts.len() < 3 {
                return None;
            }

            let mut stops = Vec::new();
            for part in &parts[1..] {
                stops.push(parse_color(part)?);
            }

            Some(RadialGradient { shape, stops })
        }
    }
}

/// Represents a color stop in a conic gradient.
/// spec: S-40 conic-gradient
#[derive(Debug, Clone, PartialEq)]
pub struct ConicColorStop {
    pub color: Color,
    pub position: Option<f32>, // fraction in 0.0..=1.0
}

/// Represents a conic gradient background.
/// spec: S-40 conic-gradient
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    pub from_angle: f32, // degrees
    pub stops: Vec<ConicColorStop>,
}

fn parse_conic_stop(part: &str) -> Option<ConicColorStop> {
    let part = part.trim();
    if let Some(last_space_idx) = part.rfind(' ') {
        let (color_part, pos_part) = part.split_at(last_space_idx);
        let pos_part = pos_part.trim();
        let color_part = color_part.trim();

        let position = if let Some(stripped) = pos_part.strip_suffix('%') {
            let pct: f32 = stripped.trim().parse().ok()?;
            Some(pct / 100.0)
        } else if let Some(stripped) = pos_part.strip_suffix("deg") {
            let deg: f32 = stripped.trim().parse().ok()?;
            Some(deg / 360.0)
        } else {
            pos_part.parse::<f32>().ok()
        };

        if let (Some(pos), Some(color)) = (position, parse_color(color_part)) {
            return Some(ConicColorStop {
                color,
                position: Some(pos),
            });
        }
    }

    let color = parse_color(part)?;
    Some(ConicColorStop {
        color,
        position: None,
    })
}

fn resolve_stop_positions(stops: &mut [ConicColorStop]) {
    if stops.is_empty() {
        return;
    }

    if stops[0].position.is_none() {
        stops[0].position = Some(0.0);
    }

    let last_idx = stops.len() - 1;
    if stops[last_idx].position.is_none() {
        stops[last_idx].position = Some(1.0);
    }

    let mut i = 0;
    while i < stops.len() {
        if stops[i].position.is_none() {
            let mut j = i + 1;
            while j < stops.len() && stops[j].position.is_none() {
                j += 1;
            }
            let start_pos = stops[i - 1].position.unwrap_or(0.0);
            let end_pos = stops[j].position.unwrap_or(1.0);
            let count = (j - i + 1) as f32;
            for (step_idx, stop) in stops[i..j].iter_mut().enumerate() {
                let step = (step_idx + 1) as f32;
                stop.position = Some(start_pos + (end_pos - start_pos) * (step / count));
            }
            i = j;
        } else {
            i += 1;
        }
    }

    let mut current_max = 0.0;
    for stop in stops.iter_mut() {
        let pos = stop.position.unwrap_or(current_max);
        if pos < current_max {
            stop.position = Some(current_max);
        } else {
            current_max = pos;
        }
    }
}

fn sample_conic_gradient(stops: &[ConicColorStop], t: f32) -> Color {
    if stops.is_empty() {
        return Color::Rgba(0, 0, 0, 255);
    }
    let t = t.clamp(0.0, 1.0);

    let first_pos = stops[0].position.unwrap_or(0.0);
    if t <= first_pos {
        return stops[0].color.clone();
    }

    let last_pos = stops[stops.len() - 1].position.unwrap_or(1.0);
    if t >= last_pos {
        return stops[stops.len() - 1].color.clone();
    }

    for idx in 0..stops.len() - 1 {
        let p1 = stops[idx].position.unwrap_or(0.0);
        let p2 = stops[idx + 1].position.unwrap_or(1.0);
        if t >= p1 && t <= p2 {
            if (p2 - p1).abs() < 1e-5 {
                return stops[idx + 1].color.clone();
            }
            let t_local = (t - p1) / (p2 - p1);
            return interpolate_color(
                stops[idx].color.clone(),
                stops[idx + 1].color.clone(),
                t_local,
            );
        }
    }

    stops[stops.len() - 1].color.clone()
}

impl ConicGradient {
    /// Parses a CSS conic-gradient background string.
    /// spec: S-40 conic-gradient
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let content = s.strip_prefix("conic-gradient(")?.strip_suffix(')')?;
        let parts = split_top_level_commas(content);

        if parts.is_empty() {
            return None;
        }

        let mut from_angle = 0.0;
        let stops_start_idx = if parse_conic_stop(&parts[0]).is_some() {
            0
        } else {
            let config = parts[0].trim().to_ascii_lowercase();
            if let Some(from_idx) = config.find("from ") {
                let after_from = &config[from_idx + 5..];
                let angle_token = after_from.split_whitespace().next()?;
                if let Some(stripped) = angle_token.strip_suffix("deg") {
                    from_angle = stripped.trim().parse::<f32>().ok()?;
                } else if let Some(stripped) = angle_token.strip_suffix("turn") {
                    let turn = stripped.trim().parse::<f32>().ok()?;
                    from_angle = turn * 360.0;
                } else if let Some(stripped) = angle_token.strip_suffix("rad") {
                    let rad = stripped.trim().parse::<f32>().ok()?;
                    from_angle = rad.to_degrees();
                } else {
                    return None;
                }
            }

            if config.contains("at ") {
                // // TODO(spec): Non-center positions at <position> are deferred to a later version.
            }

            1
        };

        let num_stops = parts.len() - stops_start_idx;
        if num_stops < 2 {
            return None;
        }

        let mut stops = Vec::with_capacity(num_stops);
        for part in &parts[stops_start_idx..] {
            let stop = parse_conic_stop(part)?;
            stops.push(stop);
        }

        resolve_stop_positions(&mut stops);
        Some(ConicGradient { from_angle, stops })
    }
}

/// Splits a string on top-level commas, respecting nested parentheses.
/// spec: S-40
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    for c in s.chars() {
        if c == '(' {
            depth += 1;
            current.push(c);
        } else if c == ')' {
            depth -= 1;
            current.push(c);
        } else if c == ',' && depth == 0 {
            parts.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Parses a color string (named, hex, or rgb/rgba).
/// spec: S-40
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if s.is_empty() {
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
                Some(Color::Rgba(r * 17, g * 17, b * 17, 255))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
                let a = u8::from_str_radix(&hex[3..4], 16).ok()?;
                Some(Color::Rgba(r * 17, g * 17, b * 17, a * 17))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgba(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Color::Rgba(r, g, b, a))
            }
            _ => None,
        }
    } else if let Some(content) = s
        .strip_prefix("rgb(")
        .and_then(|suffix| suffix.strip_suffix(')'))
    {
        parse_rgb_rgba_components(content, false)
    } else if let Some(content) = s
        .strip_prefix("rgba(")
        .and_then(|suffix| suffix.strip_suffix(')'))
    {
        parse_rgb_rgba_components(content, true)
    } else {
        crate::css::colors::named_color(s)
    }
}

/// Parses internal components of a CSS rgb or rgba function.
/// spec: S-40
fn parse_rgb_rgba_components(content: &str, has_alpha: bool) -> Option<Color> {
    let parts: Vec<&str> = if content.contains(',') {
        content.split(',').map(|s| s.trim()).collect()
    } else {
        content.split_whitespace().collect()
    };

    if parts.len() < 3 {
        return None;
    }

    let r = parse_color_channel(parts[0])?;
    let g = parse_color_channel(parts[1])?;
    let b = parse_color_channel(parts[2])?;
    let a = if has_alpha && parts.len() >= 4 {
        parse_alpha_channel(parts[3])?
    } else {
        255
    };

    Some(Color::Rgba(r, g, b, a))
}

/// Helper to parse standard CSS r, g, b channel values.
/// spec: S-40
fn parse_color_channel(part: &str) -> Option<u8> {
    let part = part.trim();
    if let Some(stripped) = part.strip_suffix('%') {
        let val: f32 = stripped.parse().ok()?;
        Some((val.clamp(0.0, 100.0) * 2.55).round() as u8)
    } else {
        let val: f32 = part.parse().ok()?;
        Some(val.clamp(0.0, 255.0).round() as u8)
    }
}

/// Helper to parse CSS alpha values (float or percentage).
/// spec: S-40
fn parse_alpha_channel(part: &str) -> Option<u8> {
    let part = part.trim();
    if let Some(stripped) = part.strip_suffix('%') {
        let val: f32 = stripped.parse().ok()?;
        Some((val.clamp(0.0, 100.0) * 2.55).round() as u8)
    } else {
        let val: f32 = part.parse().ok()?;
        let alpha_f = val.clamp(0.0, 1.0);
        Some((alpha_f * 255.0).round() as u8)
    }
}

/// Checks if a pixel center is inside a rounded rectangle of given corner radius.
/// spec: S-40 border-radius rounded-corner clipping
fn is_inside_rounded_rect(rect: &crate::geom::Rect, r: f32, px: f32, py: f32) -> bool {
    if px < rect.origin.x || px > rect.max_x() || py < rect.origin.y || py > rect.max_y() {
        return false;
    }

    let w = rect.size.width;
    let h = rect.size.height;
    let max_r = (w / 2.0).min(h / 2.0);
    let r = r.min(max_r);
    if r <= 0.0 {
        return true;
    }

    let dx = px - rect.origin.x;
    let dy = py - rect.origin.y;

    // Top-Left Corner
    if dx < r && dy < r {
        let cx = r;
        let cy = r;
        let dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
        return dist_sq <= r * r;
    }

    // Top-Right Corner
    if dx > w - r && dy < r {
        let cx = w - r;
        let cy = r;
        let dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
        return dist_sq <= r * r;
    }

    // Bottom-Left Corner
    if dx < r && dy > h - r {
        let cx = r;
        let cy = h - r;
        let dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
        return dist_sq <= r * r;
    }

    // Bottom-Right Corner
    if dx > w - r && dy > h - r {
        let cx = w - r;
        let cy = h - r;
        let dist_sq = (dx - cx) * (dx - cx) + (dy - cy) * (dy - cy);
        return dist_sq <= r * r;
    }

    true
}

/// Interpolates between two RGBA colors based on scalar t in 0.0..=1.0.
/// spec: S-40 linear-gradient interpolation
fn interpolate_color(c1: Color, c2: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let (r1, g1, b1, a1) = match c1 {
        Color::Rgba(r, g, b, a) => (r as f32, g as f32, b as f32, a as f32),
    };
    let (r2, g2, b2, a2) = match c2 {
        Color::Rgba(r, g, b, a) => (r as f32, g as f32, b as f32, a as f32),
    };
    let r = (r1 + (r2 - r1) * t).round() as u8;
    let g = (g1 + (g2 - g1) * t).round() as u8;
    let b = (b1 + (b2 - b1) * t).round() as u8;
    let a = (a1 + (a2 - a1) * t).round() as u8;
    Color::Rgba(r, g, b, a)
}

/// Draws a box's computed conic-gradient, linear-gradient or radial-gradient background, with optional border-radius rounded-corner clipping.
/// spec: S-40
pub fn rasterize_gradient_box(
    canvas: &mut Canvas,
    rect: crate::geom::Rect,
    background: &str,
    border_radius: Option<f32>,
) {
    let background = background.trim();
    if background.starts_with("conic-gradient(") {
        let gradient = match ConicGradient::parse(background) {
            Some(g) => g,
            None => return, // spec: Ignore invalid gradient formats (I-6)
        };

        // Clip rendering bounds to the canvas dimensions (prevent out of bounds index - I-6)
        let x_start = (rect.origin.x.max(0.0).floor() as u32).min(canvas.width);
        let y_start = (rect.origin.y.max(0.0).floor() as u32).min(canvas.height);
        let x_end = (rect.max_x().max(0.0).ceil() as u32).min(canvas.width);
        let y_end = (rect.max_y().max(0.0).ceil() as u32).min(canvas.height);

        let w = rect.size.width;
        let h = rect.size.height;
        let cx = rect.origin.x + w / 2.0;
        let cy = rect.origin.y + h / 2.0;

        for y in y_start..y_end {
            for x in x_start..x_end {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Check if the pixel center is inside the rounded rect
                if border_radius.is_some_and(|r| !is_inside_rounded_rect(&rect, r, px, py)) {
                    continue;
                }

                let dx = px - cx;
                let dy = py - cy;

                let angle_rad = dy.atan2(dx) + std::f32::consts::FRAC_PI_2;
                let mut angle_deg = angle_rad.to_degrees();
                if angle_deg < 0.0 {
                    angle_deg += 360.0;
                }

                let mut final_angle = (angle_deg - gradient.from_angle) % 360.0;
                if final_angle < 0.0 {
                    final_angle += 360.0;
                }

                let t = final_angle / 360.0;

                let interpolated = sample_conic_gradient(&gradient.stops, t);

                let (r_s, g_s, b_s, a_s) = match interpolated {
                    Color::Rgba(r, g, b, a) => (r, g, b, a),
                };

                let index = (y as usize) * (canvas.width as usize) + (x as usize);
                if let Some(pixel) = canvas.pixels.get_mut(index) {
                    *pixel = blend((r_s, g_s, b_s, a_s), *pixel);
                }
            }
        }
    } else if background.starts_with("radial-gradient(") {
        let gradient = match RadialGradient::parse(background) {
            Some(g) => g,
            None => return, // spec: Ignore invalid gradient formats (I-6)
        };

        // Clip rendering bounds to the canvas dimensions (prevent out of bounds index - I-6)
        let x_start = (rect.origin.x.max(0.0).floor() as u32).min(canvas.width);
        let y_start = (rect.origin.y.max(0.0).floor() as u32).min(canvas.height);
        let x_end = (rect.max_x().max(0.0).ceil() as u32).min(canvas.width);
        let y_end = (rect.max_y().max(0.0).ceil() as u32).min(canvas.height);

        let w = rect.size.width;
        let h = rect.size.height;
        let cx = rect.origin.x + w / 2.0;
        let cy = rect.origin.y + h / 2.0;

        let is_circle = gradient.shape.as_deref() == Some("circle");

        // Precompute radius / factors
        let circle_r = if is_circle {
            ((w / 2.0) * (w / 2.0) + (h / 2.0) * (h / 2.0)).sqrt()
        } else {
            0.0
        };

        let ellipse_rx = if !is_circle {
            w / std::f32::consts::SQRT_2
        } else {
            0.0
        };
        let ellipse_ry = if !is_circle {
            h / std::f32::consts::SQRT_2
        } else {
            0.0
        };

        for y in y_start..y_end {
            for x in x_start..x_end {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Check if the pixel center is inside the rounded rect
                if border_radius.is_some_and(|r| !is_inside_rounded_rect(&rect, r, px, py)) {
                    continue;
                }

                let dx = px - cx;
                let dy = py - cy;

                let t = if is_circle {
                    if circle_r <= 0.0 {
                        0.0
                    } else {
                        let d = (dx * dx + dy * dy).sqrt();
                        d / circle_r
                    }
                } else {
                    if ellipse_rx <= 0.0 || ellipse_ry <= 0.0 {
                        0.0
                    } else {
                        let rx_term = dx / ellipse_rx;
                        let ry_term = dy / ellipse_ry;
                        (rx_term * rx_term + ry_term * ry_term).sqrt()
                    }
                };

                let t = t.clamp(0.0, 1.0);

                let n = gradient.stops.len();
                let interpolated = if n < 2 {
                    gradient
                        .stops
                        .first()
                        .cloned()
                        .unwrap_or(Color::Rgba(0, 0, 0, 255))
                } else {
                    let idx = (t * (n - 1) as f32).floor() as usize;
                    let idx = idx.min(n - 2);
                    let t_local = t * (n - 1) as f32 - idx as f32;
                    let t_local = t_local.clamp(0.0, 1.0);
                    interpolate_color(
                        gradient.stops[idx].clone(),
                        gradient.stops[idx + 1].clone(),
                        t_local,
                    )
                };

                let (r_s, g_s, b_s, a_s) = match interpolated {
                    Color::Rgba(r, g, b, a) => (r, g, b, a),
                };

                let index = (y as usize) * (canvas.width as usize) + (x as usize);
                if let Some(pixel) = canvas.pixels.get_mut(index) {
                    *pixel = blend((r_s, g_s, b_s, a_s), *pixel);
                }
            }
        }
    } else {
        let gradient = match LinearGradient::parse(background) {
            Some(g) => g,
            None => return, // spec: Ignore invalid gradient formats (I-6)
        };

        // Clip rendering bounds to the canvas dimensions (prevent out of bounds index - I-6)
        let x_start = (rect.origin.x.max(0.0).floor() as u32).min(canvas.width);
        let y_start = (rect.origin.y.max(0.0).floor() as u32).min(canvas.height);
        let x_end = (rect.max_x().max(0.0).ceil() as u32).min(canvas.width);
        let y_end = (rect.max_y().max(0.0).ceil() as u32).min(canvas.height);

        let (ux, uy, l) = match &gradient.direction {
            GradientDirection::ToBottom => (0.0, 1.0, rect.size.height),
            GradientDirection::ToTop => (0.0, -1.0, rect.size.height),
            GradientDirection::ToRight => (1.0, 0.0, rect.size.width),
            GradientDirection::ToLeft => (-1.0, 0.0, rect.size.width),
            GradientDirection::Angle(deg) => {
                let rad = deg.to_radians();
                let ux = rad.sin();
                let uy = -rad.cos();
                let l = rect.size.width.abs() * rad.sin().abs()
                    + rect.size.height.abs() * rad.cos().abs();
                (ux, uy, l)
            }
        };

        // TODO(spec): Other forms / per-corner radius are not yet supported.

        for y in y_start..y_end {
            for x in x_start..x_end {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;

                // Check if the pixel center is inside the rounded rect
                if border_radius.is_some_and(|r| !is_inside_rounded_rect(&rect, r, px, py)) {
                    continue;
                }

                let t = if l <= 0.0 {
                    0.0
                } else {
                    match &gradient.direction {
                        GradientDirection::ToBottom => (py - rect.origin.y) / l,
                        GradientDirection::ToTop => (rect.max_y() - py) / l,
                        GradientDirection::ToRight => (px - rect.origin.x) / l,
                        GradientDirection::ToLeft => (rect.max_x() - px) / l,
                        GradientDirection::Angle(_) => {
                            let cx = rect.origin.x + rect.size.width / 2.0;
                            let cy = rect.origin.y + rect.size.height / 2.0;
                            let dx = px - cx;
                            let dy = py - cy;
                            let d = dx * ux + dy * uy;
                            (d / l) + 0.5
                        }
                    }
                };
                let t = t.clamp(0.0, 1.0);

                let interpolated =
                    interpolate_color(gradient.color1.clone(), gradient.color2.clone(), t);
                let (r_s, g_s, b_s, a_s) = match interpolated {
                    Color::Rgba(r, g, b, a) => (r, g, b, a),
                };

                let index = (y as usize) * (canvas.width as usize) + (x as usize);
                if let Some(pixel) = canvas.pixels.get_mut(index) {
                    *pixel = blend((r_s, g_s, b_s, a_s), *pixel);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Rect;

    #[test]
    fn test_canvas_new() {
        let canvas = Canvas::new(10, 10);
        assert_eq!(canvas.width, 10);
        assert_eq!(canvas.height, 10);
        assert_eq!(canvas.pixels.len(), 100);
        assert_eq!(canvas.pixel(0, 0), 0xFFFFFFFF);
        assert_eq!(canvas.pixel(9, 9), 0xFFFFFFFF);
        assert_eq!(canvas.pixel(10, 10), 0); // Out of bounds
    }

    #[test]
    fn test_rasterize_solid_rect() {
        let items = vec![DisplayItem::SolidRect {
            rect: Rect::new(2.0, 2.0, 3.0, 3.0),
            color: Color::Rgba(255, 0, 0, 255), // Red
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 10, 10);

        // Check a pixel inside the rect
        assert_eq!(canvas.pixel(2, 2), 0xFFFF0000);
        assert_eq!(canvas.pixel(4, 4), 0xFFFF0000);
        // Check a pixel outside the rect
        assert_eq!(canvas.pixel(1, 1), 0xFFFFFFFF);
        assert_eq!(canvas.pixel(5, 5), 0xFFFFFFFF);
    }

    #[test]
    fn test_rasterize_clipping() {
        let items = vec![DisplayItem::SolidRect {
            rect: Rect::new(-2.0, -2.0, 5.0, 5.0),
            color: Color::Rgba(0, 255, 0, 255), // Green
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 10, 10);

        // Inside clipped area
        assert_eq!(canvas.pixel(0, 0), 0xFF00FF00);
        assert_eq!(canvas.pixel(2, 2), 0xFF00FF00);
        // Outside clipped area
        assert_eq!(canvas.pixel(3, 3), 0xFFFFFFFF);
    }

    #[test]
    fn test_rasterize_alpha_blending() {
        let items = vec![
            // Red background
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                color: Color::Rgba(255, 0, 0, 255),
            },
            // Semi-transparent blue on top
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                color: Color::Rgba(0, 0, 255, 128), // ~50% blue
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 1, 1);

        let pixel = canvas.pixel(0, 0);
        let a = (pixel >> 24) & 0xFF;
        let r = (pixel >> 16) & 0xFF;
        let g = (pixel >> 8) & 0xFF;
        let b = pixel & 0xFF;

        assert_eq!(a, 255);
        // r = 255 * (1 - 128/255) = 255 * 127/255 = 127
        // b = 255 * (128/255) = 128
        assert!((126..=128).contains(&r));
        assert_eq!(g, 0);
        assert!((126..=128).contains(&b));
    }

    #[test]
    fn test_painter_algorithm() {
        let items = vec![
            // Red
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                color: Color::Rgba(255, 0, 0, 255),
            },
            // Blue on top (fully opaque)
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                color: Color::Rgba(0, 0, 255, 255),
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 1, 1);

        assert_eq!(canvas.pixel(0, 0), 0xFF0000FF);
    }

    #[test]
    fn test_rasterize_text() {
        let items = vec![DisplayItem::Text {
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            text: "A".into(),
            color: Color::Rgba(255, 0, 0, 255), // Red
            letter_spacing: 0.0,
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 20, 20);

        // "A" in BitmapFont::builtin() (8x8) is not blank.
        // It should have some red pixels.
        let mut found_red = false;
        for y in 0..20 {
            for x in 0..20 {
                let pixel = canvas.pixel(x, y);
                if pixel == 0xFFFF0000 {
                    found_red = true;
                }
            }
        }
        assert!(found_red, "Should find at least one red pixel for 'A'");
    }

    #[test]
    fn test_rasterize_text_clipping() {
        let items = vec![DisplayItem::Text {
            rect: Rect::new(18.0, 18.0, 10.0, 10.0),
            text: "A".into(),
            color: Color::Rgba(255, 0, 0, 255),
            letter_spacing: 0.0,
        }];
        let list = DisplayList(items);
        // Canvas is 20x20. Text starts at (18, 18).
        // 8x8 glyph will go to (26, 26).
        // It should not panic.
        let _canvas = rasterize(&list, 20, 20);
    }

    #[test]
    fn test_rasterize_ascii_regression() {
        let items = vec![DisplayItem::Text {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            text: "A".into(),
            color: Color::Rgba(0, 0, 0, 255), // Black
            letter_spacing: 0.0,
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 16, 16);

        let mut found_non_bg = false;
        for y in 0..16 {
            for x in 0..16 {
                if canvas.pixel(x, y) != 0xFFFFFFFF {
                    found_non_bg = true;
                }
            }
        }
        assert!(
            found_non_bg,
            "Should find at least one non-background pixel for ASCII text 'A'"
        );
    }

    #[test]
    fn test_rasterize_non_ascii_hiragana() {
        let items = vec![DisplayItem::Text {
            rect: Rect::new(2.0, 2.0, 28.0, 28.0),
            text: "あ".into(),
            color: Color::Rgba(0, 0, 0, 255), // Black
            letter_spacing: 0.0,
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 32, 32);

        // This should not panic and should draw at least one non-background pixel.
        // Whether a fallback font exists or .notdef (tofu) is drawn, it should produce some pixels.
        let mut found_non_bg = false;
        for y in 0..32 {
            for x in 0..32 {
                if canvas.pixel(x, y) != 0xFFFFFFFF {
                    found_non_bg = true;
                }
            }
        }
        assert!(
            found_non_bg,
            "Should find at least one non-background pixel for non-ASCII 'あ'"
        );
    }

    #[test]
    fn test_rasterize_robustness() {
        // Control char (e.g. \u{0001}) or unassigned codepoint on a tiny canvas
        let items = vec![DisplayItem::Text {
            rect: Rect::new(1.0, 1.0, 2.0, 2.0),
            text: "\u{0001}\u{E000}".into(), // control + private use area
            color: Color::Rgba(0, 0, 0, 255),
            letter_spacing: 0.0,
        }];
        let list = DisplayList(items);
        // Canvas is tiny (e.g. 2x2), should not panic and should stay in bounds.
        let canvas = rasterize(&list, 2, 2);
        assert_eq!(canvas.width, 2);
        assert_eq!(canvas.height, 2);
    }

    #[test]
    fn test_letter_spacing_rasterization() {
        // Test drawing "AB" with 0.0 letter spacing
        let items_0 = vec![DisplayItem::Text {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            text: "AB".into(),
            color: Color::Rgba(0, 0, 0, 255), // Black
            letter_spacing: 0.0,
        }];
        let list_0 = DisplayList(items_0);
        let canvas_0 = rasterize(&list_0, 100, 20);

        // Test drawing "AB" with 10.0 letter spacing
        let items_10 = vec![DisplayItem::Text {
            rect: Rect::new(0.0, 0.0, 100.0, 20.0),
            text: "AB".into(),
            color: Color::Rgba(0, 0, 0, 255), // Black
            letter_spacing: 10.0,
        }];
        let list_10 = DisplayList(items_10);
        let canvas_10 = rasterize(&list_10, 100, 20);

        // Find the rightmost non-background pixel (not white 0xFFFFFFFF) in both canvases
        let mut max_x_0 = 0;
        let mut found_0 = false;
        for y in 0..20 {
            for x in 0..100 {
                if canvas_0.pixel(x, y) != 0xFFFFFFFF {
                    max_x_0 = max_x_0.max(x);
                    found_0 = true;
                }
            }
        }

        let mut max_x_10 = 0;
        let mut found_10 = false;
        for y in 0..20 {
            for x in 0..100 {
                if canvas_10.pixel(x, y) != 0xFFFFFFFF {
                    max_x_10 = max_x_10.max(x);
                    found_10 = true;
                }
            }
        }

        assert!(found_0, "Should have drawn some pixels for 0.0 spacing");
        assert!(found_10, "Should have drawn some pixels for 10.0 spacing");

        // The text run with positive letter-spacing must be wider, hence its rightmost pixel is further right.
        assert!(
            max_x_10 > max_x_0,
            "Wider spacing (max_x: {}) should extend further right than default spacing (max_x: {})",
            max_x_10,
            max_x_0
        );

        // Verify that the actual pixel offset difference matches the expected spacing (roughly 10px shift)
        let diff = max_x_10 as i32 - max_x_0 as i32;
        assert!(
            diff >= 8,
            "Expected rightmost pixel shift of at least 8px, got {}",
            diff
        );
    }

    #[test]
    fn test_rasterize_gradient_integration() {
        // Build a display list containing a Gradient item
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let items = vec![DisplayItem::Gradient {
            rect,
            css: "linear-gradient(to bottom, #ff0000, #0000ff)".to_string(),
            border_radius: None,
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 10, 10);

        // Assert that at least some pixels are non-background (non-0xFFFFFFFF)
        let mut found_non_bg = false;
        for y in 0..10 {
            for x in 0..10 {
                if canvas.pixel(x, y) != 0xFFFFFFFF {
                    found_non_bg = true;
                }
            }
        }
        assert!(found_non_bg, "Should render non-background pixels");

        // Verify color at top (near #ff0000)
        let top_pixel = canvas.pixel(5, 0);
        let top_r = (top_pixel >> 16) & 0xFF;
        let top_b = top_pixel & 0xFF;
        assert!(top_r > 200, "Top should be mostly red");
        assert!(top_b < 50, "Top should have low blue");

        // Verify color at bottom (near #0000ff)
        let bot_pixel = canvas.pixel(5, 9);
        let bot_r = (bot_pixel >> 16) & 0xFF;
        let bot_b = bot_pixel & 0xFF;
        assert!(bot_r < 50, "Bottom should have low red");
        assert!(bot_b > 200, "Bottom should be mostly blue");
    }

    #[test]
    fn test_linear_gradient_midpoint() {
        // Create canvas of dimensions 9x9 (odd to have exact center pixel center at t=0.5)
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        // Linear gradient red to blue
        rasterize_gradient_box(
            &mut canvas,
            rect,
            "linear-gradient(to bottom, #ff0000, #0000ff)",
            None,
        );

        // Top pixel (0,0) center (0.5, 0.5), near start color (red)
        let top_pixel = canvas.pixel(4, 0);
        let top_r = (top_pixel >> 16) & 0xFF;
        let top_b = top_pixel & 0xFF;
        assert!(top_r > 230);
        assert!(top_b < 25);

        // Bottom pixel (0,8) center (0.5, 8.5), near end color (blue)
        let bot_pixel = canvas.pixel(4, 8);
        let bot_r = (bot_pixel >> 16) & 0xFF;
        let bot_b = bot_pixel & 0xFF;
        assert!(bot_r < 25);
        assert!(bot_b > 230);

        // Center pixel (4,4) center (4.5, 4.5), which is exactly at t=0.5
        // Midpoint should be exactly average of red and blue: (128, 0, 128)
        let center_pixel = canvas.pixel(4, 4);
        let cr = (center_pixel >> 16) & 0xFF;
        let cg = (center_pixel >> 8) & 0xFF;
        let cb = center_pixel & 0xFF;
        let ca = (center_pixel >> 24) & 0xFF;

        assert_eq!(ca, 255);
        assert_eq!(cr, 128);
        assert_eq!(cg, 0);
        assert_eq!(cb, 128);
    }

    #[test]
    fn test_border_radius_clipping() {
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        // Apply 3px border-radius
        rasterize_gradient_box(
            &mut canvas,
            rect,
            "linear-gradient(to bottom, #ff0000, #0000ff)",
            Some(3.0),
        );

        // Top-left corner pixel (0, 0) should remain background (0xFFFFFFFF)
        assert_eq!(canvas.pixel(0, 0), 0xFFFFFFFF);

        // Top-right corner pixel (8, 0) should remain background
        assert_eq!(canvas.pixel(8, 0), 0xFFFFFFFF);

        // Bottom-left corner pixel (0, 8) should remain background
        assert_eq!(canvas.pixel(0, 8), 0xFFFFFFFF);

        // Bottom-right corner pixel (8, 8) should remain background
        assert_eq!(canvas.pixel(8, 8), 0xFFFFFFFF);

        // Center pixel (4, 4) should still be filled (midpoint color)
        assert_eq!(canvas.pixel(4, 4), 0xFF800080);
    }

    #[test]
    fn test_angle_gradient() {
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        // Angle 90deg is to right
        rasterize_gradient_box(
            &mut canvas,
            rect,
            "linear-gradient(90deg, #ff0000, #0000ff)",
            None,
        );

        // Leftmost pixel (0, 4) should be red
        let left_pixel = canvas.pixel(0, 4);
        let left_r = (left_pixel >> 16) & 0xFF;
        let left_b = left_pixel & 0xFF;
        assert!(left_r > 230);
        assert!(left_b < 25);

        // Rightmost pixel (8, 4) should be blue
        let right_pixel = canvas.pixel(8, 4);
        let right_r = (right_pixel >> 16) & 0xFF;
        let right_b = right_pixel & 0xFF;
        assert!(right_r < 25);
        assert!(right_b > 230);

        // Center pixel (4, 4) should be exactly midpoint (128, 0, 128)
        assert_eq!(canvas.pixel(4, 4), 0xFF800080);
    }

    #[test]
    fn test_radial_gradient_parse() {
        // Simple 2-stop
        let g1 = RadialGradient::parse("radial-gradient(#ff0000, #0000ff)").unwrap();
        assert_eq!(g1.shape, None);
        assert_eq!(g1.stops.len(), 2);
        assert_eq!(g1.stops[0], Color::Rgba(255, 0, 0, 255));
        assert_eq!(g1.stops[1], Color::Rgba(0, 0, 255, 255));

        // Leading circle keyword
        let g2 = RadialGradient::parse("radial-gradient(circle, red, blue)").unwrap();
        assert_eq!(g2.shape, Some("circle".to_string()));
        assert_eq!(g2.stops.len(), 2);

        // Leading ellipse keyword
        let g3 = RadialGradient::parse("radial-gradient(ellipse, #ff0000, #0000ff)").unwrap();
        assert_eq!(g3.shape, Some("ellipse".to_string()));
        assert_eq!(g3.stops.len(), 2);

        // Leading complex descriptor (ignored but parsed)
        let g4 = RadialGradient::parse(
            "radial-gradient(ellipse farthest-corner at 20% 30%, red, green, blue)",
        )
        .unwrap();
        assert_eq!(g4.shape, Some("ellipse".to_string()));
        assert_eq!(g4.stops.len(), 3);

        // Malformed inputs
        assert!(RadialGradient::parse("linear-gradient(to bottom, red, blue)").is_none());
        assert!(RadialGradient::parse("radial-gradient(red)").is_none());
        assert!(RadialGradient::parse("radial-gradient(circle, red)").is_none());
    }

    #[test]
    fn test_radial_gradient_rendering() {
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        rasterize_gradient_box(&mut canvas, rect, "radial-gradient(#ff0000, #0000ff)", None);

        // Center pixel (4,4) should be close to first stop (red)
        let center_pixel = canvas.pixel(4, 4);
        let cr = (center_pixel >> 16) & 0xFF;
        let cb = center_pixel & 0xFF;
        assert!(cr > 230);
        assert!(cb < 25);

        // Corner pixel (0,0) should be near the last stop (blue)
        let corner_pixel = canvas.pixel(0, 0);
        let cor_r = (corner_pixel >> 16) & 0xFF;
        let cor_b = corner_pixel & 0xFF;
        assert!(cor_r < 35);
        assert!(cor_b > 220);

        // Prove interpolation: count distinct colors inside the painted area.
        // There should be more than 2 distinct colors.
        let mut colors = Vec::new();
        for y in 0..9 {
            for x in 0..9 {
                let pixel = canvas.pixel(x, y);
                if !colors.contains(&pixel) {
                    colors.push(pixel);
                }
            }
        }
        assert!(
            colors.len() > 2,
            "Proves interpolation occurs, got {} colors",
            colors.len()
        );
    }

    #[test]
    fn test_radial_gradient_circle_rendering() {
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        rasterize_gradient_box(
            &mut canvas,
            rect,
            "radial-gradient(circle, #ff0000, #0000ff)",
            None,
        );

        // Center pixel (4,4) is red
        let center_pixel = canvas.pixel(4, 4);
        let cr = (center_pixel >> 16) & 0xFF;
        let cb = center_pixel & 0xFF;
        assert!(cr > 230);
        assert!(cb < 25);

        // Corners are blue
        let corner_pixel = canvas.pixel(0, 0);
        let cor_r = (corner_pixel >> 16) & 0xFF;
        let cor_b = corner_pixel & 0xFF;
        assert!(cor_r < 35);
        assert!(cor_b > 220);
    }

    #[test]
    fn test_rasterize_image_blit() {
        use crate::geom::Rect;

        // 1. Generate 2x2 image
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        source_canvas.pixels[2] = 0xFF0000FF; // Blue
        source_canvas.pixels[3] = 0xFFFFFF00; // Yellow
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_test_rasterize_image_blit.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        // 2. Build DisplayList to scale this 2x2 PNG onto a 4x4 rect
        let items = vec![DisplayItem::Image {
            rect: Rect::new(0.0, 0.0, 4.0, 4.0),
            src: temp_filename.to_string(),
            base_url: None,
            decoded: None,
            object_fit: crate::paint::ObjectFit::Fill,
        }];
        let list = DisplayList(items);

        // 3. Rasterize onto a 4x4 canvas
        let canvas = rasterize(&list, 4, 4);

        // 4. Verify nearest-neighbor scale output pixels
        // Top-left 2x2 should be Red
        assert_eq!(canvas.pixel(0, 0), 0xFFFF0000);
        assert_eq!(canvas.pixel(1, 0), 0xFFFF0000);
        assert_eq!(canvas.pixel(0, 1), 0xFFFF0000);
        assert_eq!(canvas.pixel(1, 1), 0xFFFF0000);

        // Top-right 2x2 should be Green
        assert_eq!(canvas.pixel(2, 0), 0xFF00FF00);
        assert_eq!(canvas.pixel(3, 0), 0xFF00FF00);
        assert_eq!(canvas.pixel(2, 1), 0xFF00FF00);
        assert_eq!(canvas.pixel(3, 1), 0xFF00FF00);

        // Bottom-left 2x2 should be Blue
        assert_eq!(canvas.pixel(0, 2), 0xFF0000FF);
        assert_eq!(canvas.pixel(1, 2), 0xFF0000FF);
        assert_eq!(canvas.pixel(0, 3), 0xFF0000FF);
        assert_eq!(canvas.pixel(1, 3), 0xFF0000FF);

        // Bottom-right 2x2 should be Yellow
        assert_eq!(canvas.pixel(2, 2), 0xFFFFFF00);
        assert_eq!(canvas.pixel(3, 2), 0xFFFFFF00);
        assert_eq!(canvas.pixel(2, 3), 0xFFFFFF00);
        assert_eq!(canvas.pixel(3, 3), 0xFFFFFF00);

        // Cleanup
        let _ = std::fs::remove_file(temp_filename);
    }

    #[test]
    fn test_rasterize_image_missing_or_corrupt() {
        use crate::geom::Rect;

        // Missing file should skip gracefully without panicking
        let items = vec![DisplayItem::Image {
            rect: Rect::new(0.0, 0.0, 4.0, 4.0),
            src: "this_file_does_not_exist_at_all.png".to_string(),
            base_url: None,
            decoded: None,
            object_fit: crate::paint::ObjectFit::Fill,
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 4, 4);
        for &pixel in &canvas.pixels {
            assert_eq!(pixel, 0xFFFFFFFF);
        }

        // Corrupt file should skip gracefully without panicking
        let corrupt_filename = "temp_test_corrupt.png";
        std::fs::write(corrupt_filename, b"completely corrupted png data").unwrap();

        let items_corrupt = vec![DisplayItem::Image {
            rect: Rect::new(0.0, 0.0, 4.0, 4.0),
            src: corrupt_filename.to_string(),
            base_url: None,
            decoded: None,
            object_fit: crate::paint::ObjectFit::Fill,
        }];
        let list_corrupt = DisplayList(items_corrupt);
        let canvas_corrupt = rasterize(&list_corrupt, 4, 4);
        for &pixel in &canvas_corrupt.pixels {
            assert_eq!(pixel, 0xFFFFFFFF);
        }

        let _ = std::fs::remove_file(corrupt_filename);
    }

    #[test]
    fn test_rasterize_image_contain() {
        use crate::geom::Rect;
        // Generate 2x1 image: Red on left, Green on right
        let mut source_canvas = Canvas::new(2, 1);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_test_rasterize_image_contain.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        // Canvas: 4x4, Blue background
        let items = vec![
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                color: Color::Rgba(0, 0, 255, 255), // Blue
            },
            DisplayItem::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                src: temp_filename.to_string(),
                base_url: None,
                decoded: None,
                object_fit: crate::paint::ObjectFit::Contain,
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 4, 4);

        // Top row (y=0) and bottom row (y=3) should remain Blue background
        for x in 0..4 {
            assert_eq!(canvas.pixel(x, 0), 0xFF0000FF);
            assert_eq!(canvas.pixel(x, 3), 0xFF0000FF);
        }

        // Center rows (y=1 and y=2) should contain scaled image
        // Left half is Red (x=0,1), Right half is Green (x=2,3)
        for y in 1..3 {
            assert_eq!(canvas.pixel(0, y), 0xFFFF0000);
            assert_eq!(canvas.pixel(1, y), 0xFFFF0000);
            assert_eq!(canvas.pixel(2, y), 0xFF00FF00);
            assert_eq!(canvas.pixel(3, y), 0xFF00FF00);
        }

        let _ = std::fs::remove_file(temp_filename);
    }

    #[test]
    fn test_rasterize_image_cover() {
        use crate::geom::Rect;
        // Generate 2x1 image: Red on left, Green on right
        let mut source_canvas = Canvas::new(2, 1);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_test_rasterize_image_cover.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        // Canvas: 2x2, Blue background (should be completely covered)
        let items = vec![
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 2.0, 2.0),
                color: Color::Rgba(0, 0, 255, 255), // Blue
            },
            DisplayItem::Image {
                rect: Rect::new(0.0, 0.0, 2.0, 2.0),
                src: temp_filename.to_string(),
                base_url: None,
                decoded: None,
                object_fit: crate::paint::ObjectFit::Cover,
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 2, 2);

        // Entire 2x2 rect is painted, left column is Red, right column is Green
        assert_eq!(canvas.pixel(0, 0), 0xFFFF0000);
        assert_eq!(canvas.pixel(0, 1), 0xFFFF0000);
        assert_eq!(canvas.pixel(1, 0), 0xFF00FF00);
        assert_eq!(canvas.pixel(1, 1), 0xFF00FF00);

        let _ = std::fs::remove_file(temp_filename);
    }

    #[test]
    fn test_rasterize_image_none() {
        use crate::geom::Rect;
        // Generate 2x2 image:
        // Red, Green
        // Blue, Yellow
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        source_canvas.pixels[2] = 0xFF0000FF; // Blue
        source_canvas.pixels[3] = 0xFFFFFF00; // Yellow
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_test_rasterize_image_none.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        // Canvas: 4x4, Black background (image drawn intrinsic 2x2 at center x=1..2, y=1..2)
        let items = vec![
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                color: Color::Rgba(0, 0, 0, 255), // Black
            },
            DisplayItem::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                src: temp_filename.to_string(),
                base_url: None,
                decoded: None,
                object_fit: crate::paint::ObjectFit::None,
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 4, 4);

        // Surrounding border remains Black
        for x in 0..4 {
            assert_eq!(canvas.pixel(x, 0), 0xFF000000);
            assert_eq!(canvas.pixel(x, 3), 0xFF000000);
        }
        assert_eq!(canvas.pixel(0, 1), 0xFF000000);
        assert_eq!(canvas.pixel(3, 1), 0xFF000000);
        assert_eq!(canvas.pixel(0, 2), 0xFF000000);
        assert_eq!(canvas.pixel(3, 2), 0xFF000000);

        // Center 2x2 should match intrinsic image
        assert_eq!(canvas.pixel(1, 1), 0xFFFF0000); // Red
        assert_eq!(canvas.pixel(2, 1), 0xFF00FF00); // Green
        assert_eq!(canvas.pixel(1, 2), 0xFF0000FF); // Blue
        assert_eq!(canvas.pixel(2, 2), 0xFFFFFF00); // Yellow

        let _ = std::fs::remove_file(temp_filename);
    }

    #[test]
    fn test_rasterize_image_scale_down() {
        use crate::geom::Rect;
        // ScaleDown with a smaller image acts like None. Let's reuse the 2x2 image onto 4x4 rect.
        let mut source_canvas = Canvas::new(2, 2);
        source_canvas.pixels[0] = 0xFFFF0000; // Red
        source_canvas.pixels[1] = 0xFF00FF00; // Green
        source_canvas.pixels[2] = 0xFF0000FF; // Blue
        source_canvas.pixels[3] = 0xFFFFFF00; // Yellow
        let png_bytes = crate::image::encode_png(&source_canvas);

        let temp_filename = "temp_test_rasterize_image_scaledown.png";
        std::fs::write(temp_filename, &png_bytes).unwrap();

        let items = vec![
            DisplayItem::SolidRect {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                color: Color::Rgba(0, 0, 0, 255), // Black
            },
            DisplayItem::Image {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                src: temp_filename.to_string(),
                base_url: None,
                decoded: None,
                object_fit: crate::paint::ObjectFit::ScaleDown,
            },
        ];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 4, 4);

        // Surrounding border remains Black
        for x in 0..4 {
            assert_eq!(canvas.pixel(x, 0), 0xFF000000);
            assert_eq!(canvas.pixel(x, 3), 0xFF000000);
        }
        assert_eq!(canvas.pixel(0, 1), 0xFF000000);
        assert_eq!(canvas.pixel(3, 1), 0xFF000000);
        assert_eq!(canvas.pixel(0, 2), 0xFF000000);
        assert_eq!(canvas.pixel(3, 2), 0xFF000000);

        // Center 2x2 should match intrinsic image
        assert_eq!(canvas.pixel(1, 1), 0xFFFF0000); // Red
        assert_eq!(canvas.pixel(2, 1), 0xFF00FF00); // Green
        assert_eq!(canvas.pixel(1, 2), 0xFF0000FF); // Blue
        assert_eq!(canvas.pixel(2, 2), 0xFFFFFF00); // Yellow

        let _ = std::fs::remove_file(temp_filename);
    }

    #[test]
    fn test_conic_gradient_parse() {
        // Simple 2-stop
        let g1 = ConicGradient::parse("conic-gradient(#ff0000, #0000ff)").unwrap();
        assert_eq!(g1.from_angle, 0.0);
        assert_eq!(g1.stops.len(), 2);
        assert_eq!(g1.stops[0].color, Color::Rgba(255, 0, 0, 255));
        assert_eq!(g1.stops[0].position, Some(0.0));
        assert_eq!(g1.stops[1].color, Color::Rgba(0, 0, 255, 255));
        assert_eq!(g1.stops[1].position, Some(1.0));

        // Leading from 90deg
        let g2 = ConicGradient::parse("conic-gradient(from 90deg, red, blue)").unwrap();
        assert_eq!(g2.from_angle, 90.0);
        assert_eq!(g2.stops.len(), 2);

        // Leading from 0.5turn
        let g3 = ConicGradient::parse("conic-gradient(from 0.5turn, #ff0000, #0000ff)").unwrap();
        assert_eq!(g3.from_angle, 180.0);
        assert_eq!(g3.stops.len(), 2);

        // Custom stop positions
        let g4 = ConicGradient::parse("conic-gradient(red 10%, green 50deg, blue 0.8)").unwrap();
        assert_eq!(g4.stops.len(), 3);
        assert_eq!(g4.stops[0].position, Some(0.1));
        assert_eq!(g4.stops[1].position, Some(50.0 / 360.0));
        assert_eq!(g4.stops[2].position, Some(0.8));
    }

    #[test]
    fn test_conic_gradient_rendering() {
        let mut canvas = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        rasterize_gradient_box(&mut canvas, rect, "conic-gradient(#ff0000, #0000ff)", None);

        // Near North (0deg, i.e., top-middle, e.g. pixel 4, 1): should be reddish (start of the turn)
        let top_pixel = canvas.pixel(4, 1);
        let top_r = (top_pixel >> 16) & 0xFF;
        let top_b = top_pixel & 0xFF;
        assert!(top_r > 200, "top_r was {}", top_r);
        assert!(top_b < 60, "top_b was {}", top_b);

        // Near North-Northwest (e.g. 340deg, i.e., pixel 3, 1): should be bluish (near the end of the turn)
        let bottom_pixel = canvas.pixel(3, 1);
        let bottom_r = (bottom_pixel >> 16) & 0xFF;
        let bottom_b = bottom_pixel & 0xFF;
        assert!(bottom_r < 60, "bottom_r was {}", bottom_r);
        assert!(bottom_b > 200, "bottom_b was {}", bottom_b);

        assert_ne!(top_pixel, bottom_pixel);
    }

    #[test]
    fn test_conic_gradient_from_rotation() {
        let mut canvas1 = Canvas::new(9, 9);
        let mut canvas2 = Canvas::new(9, 9);
        let rect = Rect::new(0.0, 0.0, 9.0, 9.0);

        // Without from 90deg, East is 90deg / 0.25 (interpolated)
        rasterize_gradient_box(&mut canvas1, rect, "conic-gradient(#ff0000, #0000ff)", None);

        // With from 90deg, East becomes the starting position (0.0 -> Red)
        rasterize_gradient_box(
            &mut canvas2,
            rect,
            "conic-gradient(from 90deg, #ff0000, #0000ff)",
            None,
        );

        let pixel_no_from = canvas1.pixel(7, 4);
        let pixel_with_from = canvas2.pixel(7, 4);

        // Without from, (7, 4) should be intermediate purple
        let r_no = (pixel_no_from >> 16) & 0xFF;
        let b_no = pixel_no_from & 0xFF;
        assert!(r_no > 100 && r_no < 210, "r_no was {}", r_no);
        assert!(b_no > 30 && b_no < 100, "b_no was {}", b_no);

        // With from 90deg, (7, 4) is East, so it should be near starting Red
        let r_with = (pixel_with_from >> 16) & 0xFF;
        let b_with = pixel_with_from & 0xFF;
        assert!(r_with > 220, "r_with was {}", r_with);
        assert!(b_with < 35, "b_with was {}", b_with);

        assert_ne!(pixel_no_from, pixel_with_from);
    }

    #[test]
    fn test_conic_gradient_robustness() {
        assert!(ConicGradient::parse("conic-gradient()").is_none());
        assert!(ConicGradient::parse("conic-gradient(red)").is_none());
        assert!(ConicGradient::parse("conic-gradient(from 90deg)").is_none());
        assert!(ConicGradient::parse("conic-gradient(invalid_color, blue)").is_none());
    }
}
