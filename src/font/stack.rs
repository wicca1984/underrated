use crate::font::BitmapFont;
use ab_glyph::{Font, ScaleFont};

/// One rasterized glyph (variable cell; full-width CJK fits as-is).
#[derive(Debug, Clone)]
pub struct Glyph {
    pub width: u32,        // coverage bitmap width (px)
    pub height: u32,       // coverage bitmap height (px)
    pub advance: u32,      // horizontal advance (px)
    pub bearing_x: i32,    // left offset from cell origin (px)
    pub bearing_y: i32,    // upward offset from baseline (px)
    pub coverage: Vec<u8>, // row-major width*height alpha 0..=255
}

/// Font stack: built-in ASCII bitmap + system fallback face for non-ASCII.
pub struct FontStack {
    bitmap_font: BitmapFont,
    fallback_font: Option<ab_glyph::FontVec>,
    px_size: u32,
}

impl FontStack {
    /// Build at the given px size. Loads fontdb system sources and picks ONE
    /// general fallback face that covers CJK. MUST NOT panic if none is found
    /// (I-6): in that case non-ASCII falls back to the .notdef/tofu cell.
    pub fn new(px_size: u32) -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let mut fallback_font = None;
        for face in db.faces() {
            let Some(data) = load_font_data(&face.source) else {
                continue;
            };
            let Ok(font) = ab_glyph::FontVec::try_from_vec_and_index(data, face.index) else {
                continue;
            };
            // Check if it supports 'あ' (Japanese Hiragana) or '一' (CJK Unified Ideograph)
            if font.glyph_id('あ').0 != 0 || font.glyph_id('一').0 != 0 {
                fallback_font = Some(font);
                break;
            }
        }

        Self {
            bitmap_font: BitmapFont::builtin(),
            fallback_font,
            px_size,
        }
    }

    /// Returns the line height of the font stack.
    pub fn line_height(&self) -> u32 {
        if let Some(ref font) = self.fallback_font {
            let scaled_font = font.as_scaled(ab_glyph::PxScale::from(self.px_size as f32));
            let raw_height = scaled_font.ascent() - scaled_font.descent() + scaled_font.line_gap();
            raw_height.ceil().max(1.0) as u32
        } else {
            self.px_size.max(8)
        }
    }

    /// Returns the horizontal advance of a character.
    pub fn advance(&self, c: char) -> u32 {
        if c.is_ascii() {
            self.bitmap_font.glyph_width(c)
        } else if let Some(ref font) = self.fallback_font {
            let scaled_font = font.as_scaled(ab_glyph::PxScale::from(self.px_size as f32));
            let glyph_id = scaled_font.glyph_id(c);
            let adv = if glyph_id.0 != 0 {
                scaled_font.h_advance(glyph_id)
            } else {
                scaled_font.h_advance(ab_glyph::GlyphId(0))
            };
            if adv > 0.0 {
                adv.ceil() as u32
            } else {
                self.px_size
            }
        } else {
            self.px_size.max(8)
        }
    }

    /// Returns the sum of advances for a string, saturating at u32::MAX.
    pub fn measure(&self, s: &str) -> u32 {
        let total = s.chars().map(|c| self.advance(c) as u64).sum::<u64>();
        if total > u32::MAX as u64 {
            u32::MAX
        } else {
            total as u32
        }
    }

    /// Rasterizes a single character.
    pub fn rasterize(&self, c: char) -> Glyph {
        if c.is_ascii() {
            let font = &self.bitmap_font;
            let width = 8;
            let height = 8;
            let advance = font.glyph_width(c);
            let coverage = font.glyph_coverage(c).to_vec();
            Glyph {
                width,
                height,
                advance,
                bearing_x: 0,
                bearing_y: 8,
                coverage,
            }
        } else if let Some(ref font) = self.fallback_font {
            let scaled_font = font.as_scaled(ab_glyph::PxScale::from(self.px_size as f32));
            let glyph_id = scaled_font.glyph_id(c);
            if glyph_id.0 == 0 {
                return make_tofu_glyph(self.px_size);
            }

            let ab_glyph_char = glyph_id.with_scale_and_position(
                ab_glyph::PxScale::from(self.px_size as f32),
                ab_glyph::point(0.0, 0.0),
            );

            if let Some(outlined) = scaled_font.outline_glyph(ab_glyph_char) {
                let bounds = outlined.px_bounds();
                let width = (bounds.max.x - bounds.min.x).ceil() as u32;
                let height = (bounds.max.y - bounds.min.y).ceil() as u32;
                let bearing_x = bounds.min.x.round() as i32;
                let bearing_y = (-bounds.min.y).round() as i32;

                if width > 0 && height > 0 {
                    let mut coverage = vec![0u8; (width * height) as usize];
                    outlined.draw(|x, y, cov| {
                        if x < width && y < height {
                            let idx = (y * width + x) as usize;
                            let val = (cov * 255.0).round().clamp(0.0, 255.0) as u8;
                            coverage[idx] = val;
                        }
                    });

                    let advance = scaled_font.h_advance(glyph_id).ceil().max(0.0) as u32;

                    Glyph {
                        width,
                        height,
                        advance,
                        bearing_x,
                        bearing_y,
                        coverage,
                    }
                } else {
                    let advance = scaled_font.h_advance(glyph_id).ceil().max(0.0) as u32;
                    Glyph {
                        width: 0,
                        height: 0,
                        advance,
                        bearing_x: 0,
                        bearing_y: 0,
                        coverage: Vec::new(),
                    }
                }
            } else {
                if c.is_whitespace() || c.is_control() {
                    let advance = scaled_font.h_advance(glyph_id).ceil().max(0.0) as u32;
                    Glyph {
                        width: 0,
                        height: 0,
                        advance: if advance > 0 {
                            advance
                        } else {
                            self.px_size / 2
                        },
                        bearing_x: 0,
                        bearing_y: 0,
                        coverage: Vec::new(),
                    }
                } else {
                    make_tofu_glyph(self.px_size)
                }
            }
        } else {
            make_tofu_glyph(self.px_size)
        }
    }
}

fn load_font_data(source: &fontdb::Source) -> Option<Vec<u8>> {
    match source {
        fontdb::Source::File(path) | fontdb::Source::SharedFile(path, _) => {
            std::fs::read(path).ok()
        }
        fontdb::Source::Binary(data) => Some(data.as_ref().as_ref().to_vec()),
    }
}

fn make_tofu_glyph(px_size: u32) -> Glyph {
    let w = px_size.max(8);
    let h = px_size.max(8);
    let mut coverage = vec![0u8; (w * h) as usize];

    for y in 0..h {
        for x in 0..w {
            let is_border = x == 0 || x == w - 1 || y == 0 || y == h - 1;
            if is_border {
                coverage[(y * w + x) as usize] = 255;
            }
        }
    }

    Glyph {
        width: w,
        height: h,
        advance: w,
        bearing_x: 0,
        bearing_y: h as i32,
        coverage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_stack_new_and_line_height() {
        let stack = FontStack::new(16);
        assert!(
            stack.line_height() > 0,
            "Line height should be greater than 0"
        );
    }

    #[test]
    fn test_font_stack_ascii_a() {
        let stack = FontStack::new(16);
        let glyph = stack.rasterize('A');
        assert!(glyph.advance > 0, "ASCII A advance should be > 0");
        assert_eq!(glyph.width, 8);
        assert_eq!(glyph.height, 8);
        let has_nonzero = glyph.coverage.iter().any(|&v| v > 0);
        assert!(has_nonzero, "ASCII A coverage should have non-zero pixels");

        let adv = stack.advance('A');
        assert_eq!(adv, 8);
    }

    #[test]
    fn test_font_stack_non_ascii_hiragana() {
        let stack = FontStack::new(16);
        let glyph = stack.rasterize('あ');
        let has_nonzero = glyph.coverage.iter().any(|&v| v > 0);

        if stack.fallback_font.is_some() {
            assert!(
                has_nonzero,
                "CJK character 'あ' should render with non-zero pixels if fallback font is found"
            );
            assert!(
                stack.advance('あ') > 0,
                "CJK character 'あ' advance should be > 0"
            );
        } else {
            assert!(
                has_nonzero,
                "Fallback tofu should still have visible outline border"
            );
            assert!(
                stack.advance('あ') > 0,
                "Fallback tofu advance should be > 0"
            );
        }
    }

    #[test]
    fn test_font_stack_control_and_unassigned() {
        let stack = FontStack::new(16);

        // Control character \u{0007}
        let glyph_ctrl = stack.rasterize('\u{0007}');
        let has_nonzero_ctrl = glyph_ctrl.coverage.iter().any(|&v| v > 0);
        assert!(
            has_nonzero_ctrl,
            "Control char rasterization should not be completely blank"
        );

        // Unassigned non-ASCII / extremely high codepoint
        let glyph_unassigned = stack.rasterize('\u{E000}');
        let has_nonzero_unassigned = glyph_unassigned.coverage.iter().any(|&v| v > 0);
        assert!(
            has_nonzero_unassigned,
            "Unassigned non-ASCII should return non-empty .notdef coverage"
        );
    }

    #[test]
    fn test_font_stack_measure() {
        let stack = FontStack::new(16);
        let width_empty = stack.measure("");
        assert_eq!(width_empty, 0);

        let width_ascii = stack.measure("ABC");
        assert_eq!(width_ascii, 24); // 8 * 3

        let width_mixed = stack.measure("AあB");
        assert!(
            width_mixed > 16,
            "Mixed string width should be computed and non-zero"
        );
    }
}
