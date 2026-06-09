//! Built-in bitmap font for text glyphs.
//!
//! This module provides a simple, fixed-cell monospace bitmap font for printable ASCII.

/// A fixed-cell monospace bitmap font.
pub struct BitmapFont {
    width: u32,
    height: u32,
    /// Row-major coverage data (0..=255) for printable ASCII (0x20-0x7E).
    /// Size: 95 * width * height.
    data: &'static [u8],
}

impl BitmapFont {
    /// Returns the built-in bitmap font.
    pub fn builtin() -> Self {
        Self {
            width: 8,
            height: 8,
            data: &BUILTIN_GLYPHS,
        }
    }

    /// Returns the advance width of a character in pixels.
    pub fn glyph_width(&self, _c: char) -> u32 {
        self.width
    }

    /// Returns the line height of the font in pixels.
    pub fn line_height(&self) -> u32 {
        self.height
    }

    /// Returns the row-major coverage data for a character.
    ///
    /// The length of the returned slice is always `width * height`.
    /// Printable ASCII (0x20-0x7E) returns its respective glyph.
    /// Other characters return a blank glyph.
    pub fn glyph_coverage(&self, c: char) -> &[u8] {
        let index = if (0x20..=0x7E).contains(&(c as u32)) {
            (c as u32 - 0x20) as usize
        } else {
            // TODO(spec): Non-ASCII/non-printable chars return a blank or tofu box.
            // Using index for space (0x20) as a blank box for now.
            0
        };

        let size = (self.width * self.height) as usize;
        let start = index * size;
        &self.data[start..start + size]
    }

    /// Returns the (width, height) of a glyph cell in pixels.
    pub fn glyph_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Expands 1-bit bitmaps into 8-bit coverage data.
const fn expand_bitmaps(bitmaps: &[u64; 95]) -> [u8; 95 * 8 * 8] {
    let mut data = [0u8; 95 * 8 * 8];
    let mut i = 0;
    while i < 95 {
        let bitmap = bitmaps[i];
        let mut row = 0;
        while row < 8 {
            // Row 0 is bits 56-63, row 7 is bits 0-7.
            let row_val = (bitmap >> ((7 - row) * 8)) & 0xFF;
            let mut col = 0;
            while col < 8 {
                // Bit 7 is leftmost pixel, bit 0 is rightmost.
                let bit = (row_val >> (7 - col)) & 1;
                data[i * 64 + row * 8 + col] = if bit == 1 { 255 } else { 0 };
                col += 1;
            }
            row += 1;
        }
        i += 1;
    }
    data
}

// Simple 8x8 bitmaps for printable ASCII (0x20-0x7E).
// Each u64 contains 8 rows of 8 bits. Row 0 is the top row (most significant byte).
// Bit 7 of each byte is the leftmost pixel.
const BITMAPS: [u64; 95] = [
    0x0000000000000000, // 0x20 space
    0x1818181818001800, // 0x21 !
    0x6666660000000000, // 0x22 "
    0x66667E667E666600, // 0x23 #
    0x3C663C183C663C00, // 0x24 $
    0x62660C1830664600, // 0x25 %
    0x3C663C6E66663B00, // 0x26 &
    0x1818300000000000, // 0x27 '
    0x0C18303030180C00, // 0x28 (
    0x30180C0C0C183000, // 0x29 )
    0x00663CFF3C660000, // 0x2A *
    0x0018187E18180000, // 0x2B +
    0x0000000000181830, // 0x2C ,
    0x0000007E00000000, // 0x2D -
    0x0000000000181800, // 0x2E .
    0x02060C183060C000, // 0x2F /
    0x3C66666666663C00, // 0x30 0
    0x1838181818187E00, // 0x31 1
    0x3C66060C18307E00, // 0x32 2
    0x3C66061C06663C00, // 0x33 3
    0x060E1E367E060600, // 0x34 4
    0x7E607C0606663C00, // 0x35 5
    0x3C66607C66663C00, // 0x36 6
    0x7E060C1830303000, // 0x37 7
    0x3C66663C66663C00, // 0x38 8
    0x3C66663E06663C00, // 0x39 9
    0x0018180018180000, // 0x3A :
    0x0018180018183000, // 0x3B ;
    0x0C18306030180C00, // 0x3C <
    0x00007E007E000000, // 0x3D =
    0x30180C060C183000, // 0x3E >
    0x3C66060C18001800, // 0x3F ?
    0x3C666E6E60663C00, // 0x40 @
    0x3C66667E66666600, // 0x41 A
    0x7C66667C66667C00, // 0x42 B
    0x3C66606060663C00, // 0x43 C
    0x786C6666666C7800, // 0x44 D
    0x7E60607C60607E00, // 0x45 E
    0x7E60607C60606000, // 0x46 F
    0x3C66606E66663C00, // 0x47 G
    0x6666667E66666600, // 0x48 H
    0x3C18181818183C00, // 0x49 I
    0x1E0C0C0C0C6C3800, // 0x4A J
    0x666C7870786C6600, // 0x4B K
    0x6060606060607E00, // 0x4C L
    0x63777F6B63636300, // 0x4D M
    0x66767E7E76666600, // 0x4E N
    0x3C66666666663C00, // 0x4F O
    0x7C66667C60606000, // 0x50 P
    0x3C666666666C3E06, // 0x51 Q
    0x7C66667C786C6600, // 0x52 R
    0x3C66603C06663C00, // 0x53 S
    0x7E18181818181800, // 0x54 T
    0x6666666666663C00, // 0x55 U
    0x66666666663C1800, // 0x56 V
    0x6363636B7F776300, // 0x57 W
    0x66663C183C666600, // 0x58 X
    0x66663C1818181800, // 0x59 Y
    0x7E060C1830607E00, // 0x5A Z
    0x3C30303030303C00, // 0x5B [
    0x4030180C06030100, // 0x5C \
    0x3C0C0C0C0C0C3C00, // 0x5D ]
    0x183C660000000000, // 0x5E ^
    0x00000000000000FF, // 0x5F _
    0x30180C0000000000, // 0x60 `
    0x00003C063E663E00, // 0x61 a
    0x60607C6666667C00, // 0x62 b
    0x00003C6060603C00, // 0x63 c
    0x06063E6666663E00, // 0x64 d
    0x00003C667E603C00, // 0x65 e
    0x1C307C3030303000, // 0x66 f
    0x00003E66663E063C, // 0x67 g
    0x60607C6666666600, // 0x68 h
    0x1800181818181800, // 0x69 i
    0x0C000C0C0C0C6C38, // 0x6A j
    0x6060666C786C6600, // 0x6B k
    0x1818181818181800, // 0x6C l
    0x00006B7F7F6B6300, // 0x6D m
    0x00007C6666666600, // 0x6E n
    0x00003C6666663C00, // 0x6F o
    0x00007C66667C6060, // 0x70 p
    0x00003E66663E0606, // 0x71 q
    0x00007C6660606000, // 0x72 r
    0x00003E603C067C00, // 0x73 s
    0x30307C3030301C00, // 0x74 t
    0x0000666666663E00, // 0x75 u
    0x00006666663C1800, // 0x76 v
    0x0000636B7F7F6300, // 0x77 w
    0x0000663C183C6600, // 0x78 x
    0x00006666663E063C, // 0x79 y
    0x00007E0C18307E00, // 0x7A z
    0x0C18183018180C00, // 0x7B {
    0x1818181818181800, // 0x7C |
    0x3018180C18183000, // 0x7D }
    0x0000314A44000000, // 0x7E ~
];

const BUILTIN_GLYPHS: [u8; 95 * 8 * 8] = expand_bitmaps(&BITMAPS);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_properties() {
        let font = BitmapFont::builtin();
        let (w, h) = font.glyph_size();
        assert_eq!(w, font.glyph_width('A'));
        assert_eq!(h, font.line_height());
        assert_eq!(w * h, font.glyph_coverage('A').len() as u32);
    }

    #[test]
    fn test_space_is_blank() {
        let font = BitmapFont::builtin();
        let coverage = font.glyph_coverage(' ');
        assert!(coverage.iter().all(|&v| v == 0));
    }

    #[test]
    fn test_a_is_not_blank() {
        let font = BitmapFont::builtin();
        let coverage = font.glyph_coverage('A');
        assert!(coverage.iter().any(|&v| v > 0));
    }

    #[test]
    fn test_out_of_range_is_valid() {
        let font = BitmapFont::builtin();
        let coverage = font.glyph_coverage('\u{0000}');
        assert_eq!(coverage.len(), 64);
        let coverage_emoji = font.glyph_coverage('🦀');
        assert_eq!(coverage_emoji.len(), 64);
    }
}
