//! Built-in bitmap font for text glyphs.
//!
//! This module provides a simple, fixed-cell monospace bitmap font for printable ASCII.

pub mod stack;
pub use stack::{FontStack, Glyph};

/// A fixed-cell monospace bitmap font.
pub struct BitmapFont {
    width: u32,
    height: u32,
    /// Row-major coverage data (0..=255) for printable ASCII (0x20-0x7E) plus the .notdef glyph.
    /// Size: 99 * width * height.
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

    /// Returns the fixed-cell monospace advance width of a character in pixels (8px).
    pub fn glyph_width(&self, c: char) -> u32 {
        // spec: S-72
        // TODO(spec): true proportional advances (real S-72) deferred until a variable-width bitmap backend exists
        let index = if (0x20..=0x7E).contains(&(c as u32)) {
            (c as u32 - 0x20) as usize
        } else {
            match c {
                '\u{2022}' => 96,
                '\u{25E6}' => 97,
                '\u{25AA}' => 98,
                _ => 95,
            }
        };
        GLYPH_WIDTHS[index] as u32
    }

    /// Returns the line height of the font in pixels.
    pub fn line_height(&self) -> u32 {
        self.height
    }

    /// Returns the row-major coverage data for a character.
    ///
    /// The length of the returned slice is always `width * height`.
    /// Printable ASCII (0x20-0x7E) returns its respective glyph.
    /// Other characters return the visible `.notdef` glyph.
    pub fn glyph_coverage(&self, c: char) -> &[u8] {
        let index = if (0x20..=0x7E).contains(&(c as u32)) {
            (c as u32 - 0x20) as usize
        } else {
            // spec: Undefined chars return a visible .notdef box (not blank)
            match c {
                '\u{2022}' => 96,
                '\u{25E6}' => 97,
                '\u{25AA}' => 98,
                _ => 95,
            }
        };

        let size = (self.width * self.height) as usize;
        let start = index * size;
        // Checked slice so a future change to the glyph table can never panic
        // here (I-6); fall back to the .notdef cell at index 95.
        if let Some(slice) = self.data.get(start..start + size) {
            slice
        } else if let Some(slice) = self.data.get(95 * size..96 * size) {
            slice
        } else {
            &[]
        }
    }

    /// Returns the (width, height) of a glyph cell in pixels.
    pub fn glyph_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Returns the total pixel width of the given string.
    ///
    /// spec: measure(s) returns total pixel width for layout.
    pub fn measure(&self, s: &str) -> u32 {
        let total = s.chars().map(|c| self.glyph_width(c) as u64).sum::<u64>();
        if total > u32::MAX as u64 {
            u32::MAX
        } else {
            total as u32
        }
    }
}

/// Expands 1-bit bitmaps into 8-bit coverage data.
const fn expand_bitmaps(bitmaps: &[u64; 99]) -> [u8; 99 * 8 * 8] {
    let mut data = [0u8; 99 * 8 * 8];
    let mut i = 0;
    while i < 99 {
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

// Simple 8x8 bitmaps for printable ASCII (0x20-0x7E) plus the .notdef glyph at the end.
// Each u64 contains 8 rows of 8 bits. Row 0 is the top row (most significant byte).
// Bit 7 of each byte is the leftmost pixel.
const BITMAPS: [u64; 99] = [
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
    0xFF81A59999A581FF, // 0x7F / .notdef box
    0x00183C7E7E3C1800, // U+2022 • filled disc
    0x0018244242241800, // U+25E6 ◦ hollow circle
    0x00003C3C3C3C0000, // U+25AA ▪ filled square
];

// spec: S-72
/// Fixed-cell monospace advance widths (8px) for printable ASCII (0x20-0x7E) plus the .notdef glyph.
/// Index maps to (char as u32 - 0x20) with 95 as the .notdef fallback.
const GLYPH_WIDTHS: [u8; 99] = [
    8, // 0x20 ' '
    8, // 0x21 '!'
    8, // 0x22 '"'
    8, // 0x23 '#'
    8, // 0x24 '$'
    8, // 0x25 '%'
    8, // 0x26 '&'
    8, // 0x27 '\''
    8, // 0x28 '('
    8, // 0x29 ')'
    8, // 0x2A '*'
    8, // 0x2B '+'
    8, // 0x2C ','
    8, // 0x2D '-'
    8, // 0x2E '.'
    8, // 0x2F '/'
    8, // 0x30 '0'
    8, // 0x31 '1'
    8, // 0x32 '2'
    8, // 0x33 '3'
    8, // 0x34 '4'
    8, // 0x35 '5'
    8, // 0x36 '6'
    8, // 0x37 '7'
    8, // 0x38 '8'
    8, // 0x39 '9'
    8, // 0x3A ':'
    8, // 0x3B ';'
    8, // 0x3C '<'
    8, // 0x3D '='
    8, // 0x3E '>'
    8, // 0x3F '?'
    8, // 0x40 '@'
    8, // 0x41 'A'
    8, // 0x42 'B'
    8, // 0x43 'C'
    8, // 0x44 'D'
    8, // 0x45 'E'
    8, // 0x46 'F'
    8, // 0x47 'G'
    8, // 0x48 'H'
    8, // 0x49 'I'
    8, // 0x4A 'J'
    8, // 0x4B 'K'
    8, // 0x4C 'L'
    8, // 0x4D 'M'
    8, // 0x4E 'N'
    8, // 0x4F 'O'
    8, // 0x50 'P'
    8, // 0x51 'Q'
    8, // 0x52 'R'
    8, // 0x53 'S'
    8, // 0x54 'T'
    8, // 0x55 'U'
    8, // 0x56 'V'
    8, // 0x57 'W'
    8, // 0x58 'X'
    8, // 0x59 'Y'
    8, // 0x5A 'Z'
    8, // 0x5B '['
    8, // 0x5C '\\'
    8, // 0x5D ']'
    8, // 0x5E '^'
    8, // 0x5F '_'
    8, // 0x60 '`'
    8, // 0x61 'a'
    8, // 0x62 'b'
    8, // 0x63 'c'
    8, // 0x64 'd'
    8, // 0x65 'e'
    8, // 0x66 'f'
    8, // 0x67 'g'
    8, // 0x68 'h'
    8, // 0x69 'i'
    8, // 0x6A 'j'
    8, // 0x6B 'k'
    8, // 0x6C 'l'
    8, // 0x6D 'm'
    8, // 0x6E 'n'
    8, // 0x6F 'o'
    8, // 0x70 'p'
    8, // 0x71 'q'
    8, // 0x72 'r'
    8, // 0x73 's'
    8, // 0x74 't'
    8, // 0x75 'u'
    8, // 0x76 'v'
    8, // 0x77 'w'
    8, // 0x78 'x'
    8, // 0x79 'y'
    8, // 0x7A 'z'
    8, // 0x7B '{'
    8, // 0x7C '|'
    8, // 0x7D '}'
    8, // 0x7E '~'
    8, // 0x7F .notdef
    8, // U+2022
    8, // U+25E6
    8, // U+25AA
];

const BUILTIN_GLYPHS: [u8; 99 * 8 * 8] = expand_bitmaps(&BITMAPS);

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
    fn test_lowercase_coverage_not_blank() {
        let font = BitmapFont::builtin();
        for c in 'a'..='z' {
            let coverage = font.glyph_coverage(c);
            assert!(
                coverage.iter().any(|&v| v > 0),
                "Lowercase character '{}' should not be blank",
                c
            );
        }
    }

    #[test]
    fn test_numbers_coverage_not_blank() {
        let font = BitmapFont::builtin();
        for c in '0'..='9' {
            let coverage = font.glyph_coverage(c);
            assert!(
                coverage.iter().any(|&v| v > 0),
                "Number '{}' should not be blank",
                c
            );
        }
    }

    #[test]
    fn test_symbols_coverage_not_blank() {
        let font = BitmapFont::builtin();
        // Check a bunch of symbols
        for c in [
            '!', '#', '$', '%', '&', '*', '+', ',', '-', '.', '/', ':', ';', '<', '=', '>', '?',
            '@', '[', '\\', ']', '^', '_', '`', '{', '|', '}', '~',
        ] {
            let coverage = font.glyph_coverage(c);
            assert!(
                coverage.iter().any(|&v| v > 0),
                "Symbol '{}' should not be blank",
                c
            );
        }
    }

    #[test]
    fn test_out_of_range_returns_notdef() {
        let font = BitmapFont::builtin();
        // .notdef glyph is at index 95. Let's make sure that a character outside 0x20..=0x7E
        // yields the exact same coverage data as the explicit .notdef data.
        let notdef_expected = font.data[95 * 64..96 * 64].to_vec();

        let coverage_null = font.glyph_coverage('\u{0000}');
        assert_eq!(coverage_null.len(), 64);
        assert_eq!(coverage_null, &notdef_expected[..]);

        let coverage_emoji = font.glyph_coverage('🦀');
        assert_eq!(coverage_emoji.len(), 64);
        assert_eq!(coverage_emoji, &notdef_expected[..]);

        // .notdef box must be highly visible (must contain pixels)
        assert!(
            coverage_null.iter().any(|&v| v > 0),
            ".notdef glyph should not be blank"
        );
    }

    #[test]
    fn test_measure() {
        let font = BitmapFont::builtin();
        let w = font.glyph_width('a');
        assert_eq!(font.measure("ab"), w * 2);
        assert_eq!(font.measure(""), 0);
        assert_eq!(font.measure("Hello, World!"), w * 13);
        assert_eq!(font.measure("🦀"), w); // Even unknown emoji has glyph_width
    }

    #[test]
    fn test_monospace_widths() {
        let font = BitmapFont::builtin();
        let cell_width = font.glyph_size().0;
        assert_eq!(cell_width, 8);

        // spec: S-72 Acceptance criteria - Fixed-cell monospace
        // Every printable ASCII character and the .notdef fallback must have an advance width of 8px.
        // Representative set including previously-narrow and previously-wide chars.
        let test_chars = [
            'i', 'j', ':', ';', 'f', 'J', 'I', // previously narrow
            'M', 'm', // previously wide
            'A', 'a', '0', ' ', '🦀', // others and .notdef
        ];

        for &c in &test_chars {
            assert_eq!(
                font.glyph_width(c),
                cell_width,
                "Character '{}' glyph_width must be exactly equal to cell width ({})",
                c,
                cell_width
            );
        }

        // measure of an N-char ASCII string == 8 * N
        assert_eq!(font.measure(""), 0);
        assert_eq!(font.measure("i"), 8);
        assert_eq!(font.measure("m"), 8);
        assert_eq!(font.measure("im"), 16);
        assert_eq!(font.measure("ijfM"), 32);
        assert_eq!(font.measure("Hello, World!"), 8 * 13);
        assert_eq!(font.measure("🦀"), 8);
        assert_eq!(font.measure("🦀m"), 16);
    }

    #[test]
    fn test_bullet_glyphs_not_notdef() {
        let font = BitmapFont::builtin();
        let notdef = font.glyph_coverage('\u{FFFF}');

        let chars = ['\u{2022}', '\u{25E6}', '\u{25AA}'];
        for &c in &chars {
            let cov = font.glyph_coverage(c);
            assert!(
                !cov.is_empty(),
                "Glyph coverage for {:?} must be non-empty",
                c
            );
            assert_ne!(
                cov, notdef,
                "Glyph coverage for {:?} must not fall back to .notdef",
                c
            );
            assert!(
                cov.iter().any(|&b| b > 0),
                "Glyph coverage for {:?} must have at least one non-zero coverage byte",
                c
            );
        }
    }

    #[test]
    fn test_bullet_glyphs_mutually_distinct() {
        let font = BitmapFont::builtin();
        let disc = font.glyph_coverage('\u{2022}');
        let circle = font.glyph_coverage('\u{25E6}');
        let square = font.glyph_coverage('\u{25AA}');

        assert_ne!(
            disc, circle,
            "Filled disc and hollow circle glyphs must be distinct"
        );
        assert_ne!(
            disc, square,
            "Filled disc and filled square glyphs must be distinct"
        );
        assert_ne!(
            circle, square,
            "Hollow circle and filled square glyphs must be distinct"
        );
    }
}
