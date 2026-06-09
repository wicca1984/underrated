use crate::css::values::Color;
use crate::paint::{DisplayItem, DisplayList};

/// A pixel buffer for software rasterization.
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
            DisplayItem::Text { rect, .. } => {
                // // TODO(spec): real font rasterization (rustybuzz/fonts later)
                // Draw a faint rect placeholder (faint gray)
                let color = (200, 200, 200, 64); // Faint gray

                let x_start = (rect.origin.x.max(0.0).floor() as u32).min(width);
                let y_start = (rect.origin.y.max(0.0).floor() as u32).min(height);
                let x_end = (rect.max_x().max(0.0).ceil() as u32).min(width);
                let y_end = (rect.max_y().max(0.0).ceil() as u32).min(height);

                for y in y_start..y_end {
                    for x in x_start..x_end {
                        let index = (y as usize) * (width as usize) + (x as usize);
                        if let Some(pixel) = canvas.pixels.get_mut(index) {
                            *pixel = blend(color, *pixel);
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
}
