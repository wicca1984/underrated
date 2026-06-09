use crate::css::values::Color;
use crate::paint::{DisplayItem, DisplayList};

/// A pixel buffer for software software rasterization.
/// Each pixel is stored as a u32 in 0xAARRGGBB format.
/// spec: S-14
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

impl Canvas {
    /// Creates a new canvas with the given dimensions, initialized to transparent black (0x00000000).
    /// spec: S-14
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width as usize).saturating_mul(height as usize);
        Self {
            width,
            height,
            pixels: vec![0; size],
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

/// Loads image bytes from a path or file:// URL.
fn load_image_bytes(src: &str) -> Option<Vec<u8>> {
    // 1. Try parsing as a URL
    if let Ok(url) = crate::url::Url::parse(src)
        && url.scheme == "file"
    {
        if let Ok(bytes) = std::fs::read(&url.path) {
            return Some(bytes);
        }
        // Try relative as fallback
        let relative_path = url.path.trim_start_matches('/');
        if let Ok(bytes) = std::fs::read(relative_path) {
            return Some(bytes);
        }
    }

    // 2. Fall back to reading src directly as a standard file path
    if let Ok(bytes) = std::fs::read(src) {
        return Some(bytes);
    }

    None
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
            DisplayItem::Text { rect, text, color } => {
                let font = crate::font::BitmapFont::builtin();
                let (r_f, g_f, b_f, a_f) = match color {
                    Color::Rgba(r, g, b, a) => (*r, *g, *b, *a),
                };
                if a_f == 0 {
                    continue;
                }

                let mut cursor_x = rect.origin.x;
                let cursor_y = rect.origin.y;

                for c in text.chars() {
                    let coverage = font.glyph_coverage(c);
                    let (gw, gh) = font.glyph_size();

                    for gy in 0..gh {
                        for gx in 0..gw {
                            let c_x = (cursor_x.floor() as i32) + gx as i32;
                            let c_y = (cursor_y.floor() as i32) + gy as i32;

                            // Clip to canvas bounds
                            if c_x >= 0 && c_x < width as i32 && c_y >= 0 && c_y < height as i32 {
                                let cov = coverage[(gy * gw + gx) as usize];
                                if cov > 0 {
                                    // Scale foreground alpha by glyph coverage
                                    let alpha = ((a_f as u32 * cov as u32 + 127) / 255) as u8;
                                    let index = (c_y as usize) * (width as usize) + (c_x as usize);
                                    if let Some(pixel) = canvas.pixels.get_mut(index) {
                                        *pixel = blend((r_f, g_f, b_f, alpha), *pixel);
                                    }
                                }
                            }
                        }
                    }
                    cursor_x += font.glyph_width(c) as f32;
                }
            }
            DisplayItem::Image { rect, src } => {
                let rect_w = rect.size.width;
                let rect_h = rect.size.height;
                if rect_w <= 0.0 || rect_h <= 0.0 {
                    continue;
                }

                if let Some(bytes) = load_image_bytes(src)
                    && let Some(decoded) = crate::image::decode_png(&bytes)
                {
                    if decoded.width == 0 || decoded.height == 0 {
                        continue;
                    }

                    // Clip to canvas bounds
                    let x_start = (rect.origin.x.max(0.0).floor() as u32).min(width);
                    let y_start = (rect.origin.y.max(0.0).floor() as u32).min(height);
                    let x_end = (rect.max_x().max(0.0).ceil() as u32).min(width);
                    let y_end = (rect.max_y().max(0.0).ceil() as u32).min(height);

                    for y in y_start..y_end {
                        let fy = ((y as f32 - rect.origin.y) / rect_h).clamp(0.0, 1.0);
                        let src_y =
                            ((fy * decoded.height as f32).floor() as u32).min(decoded.height - 1);
                        for x in x_start..x_end {
                            let fx = ((x as f32 - rect.origin.x) / rect_w).clamp(0.0, 1.0);
                            let src_x =
                                ((fx * decoded.width as f32).floor() as u32).min(decoded.width - 1);

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

/// Draws a box's computed linear-gradient background, with optional border-radius rounded-corner clipping.
/// spec: S-40
pub fn rasterize_gradient_box(
    canvas: &mut Canvas,
    rect: crate::geom::Rect,
    background: &str,
    border_radius: Option<f32>,
) {
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
            let l =
                rect.size.width.abs() * rad.sin().abs() + rect.size.height.abs() * rad.cos().abs();
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
        assert_eq!(canvas.pixel(0, 0), 0);
        assert_eq!(canvas.pixel(9, 9), 0);
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
        assert_eq!(canvas.pixel(1, 1), 0);
        assert_eq!(canvas.pixel(5, 5), 0);
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
        assert_eq!(canvas.pixel(3, 3), 0);
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
        }];
        let list = DisplayList(items);
        // Canvas is 20x20. Text starts at (18, 18).
        // 8x8 glyph will go to (26, 26).
        // It should not panic.
        let _canvas = rasterize(&list, 20, 20);
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

        // Top-left corner pixel (0, 0) should remain background (0x00000000)
        assert_eq!(canvas.pixel(0, 0), 0);

        // Top-right corner pixel (8, 0) should remain background
        assert_eq!(canvas.pixel(8, 0), 0);

        // Bottom-left corner pixel (0, 8) should remain background
        assert_eq!(canvas.pixel(0, 8), 0);

        // Bottom-right corner pixel (8, 8) should remain background
        assert_eq!(canvas.pixel(8, 8), 0);

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
        }];
        let list = DisplayList(items);
        let canvas = rasterize(&list, 4, 4);
        for &pixel in &canvas.pixels {
            assert_eq!(pixel, 0);
        }

        // Corrupt file should skip gracefully without panicking
        let corrupt_filename = "temp_test_corrupt.png";
        std::fs::write(corrupt_filename, b"completely corrupted png data").unwrap();

        let items_corrupt = vec![DisplayItem::Image {
            rect: Rect::new(0.0, 0.0, 4.0, 4.0),
            src: corrupt_filename.to_string(),
        }];
        let list_corrupt = DisplayList(items_corrupt);
        let canvas_corrupt = rasterize(&list_corrupt, 4, 4);
        for &pixel in &canvas_corrupt.pixels {
            assert_eq!(pixel, 0);
        }

        let _ = std::fs::remove_file(corrupt_filename);
    }
}
