//! Charset detection and decoding for HTML documents.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#determining-the-character-encoding

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Utf8,
    Utf16Le,
    Utf16Be,
    Windows1252,
    Iso8859_15,
}

/// Sniff the charset from bytes and optional transport label.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#encoding-sniffing-algorithm
pub fn sniff_charset(bytes: &[u8], transport_label: Option<&str>) -> Charset {
    // 1. BOM sniffing (HTML §13.2.3.2)
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Charset::Utf8;
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Charset::Utf16Be;
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Charset::Utf16Le;
    }

    // 2. Transport label (e.g. Content-Type header)
    if let Some(label) = transport_label {
        let trimmed = label.trim_matches(|c: char| c.is_ascii_whitespace());
        match trimmed.to_ascii_lowercase().as_str() {
            "utf-8" | "utf8" | "unicode-1-1-utf-8" | "unicode11utf8" | "unicode20utf8"
            | "x-unicode20utf8" => return Charset::Utf8,
            "utf-16le" | "utf-16" | "csunicode" | "iso-10646-ucs-2" | "ucs-2" | "unicode"
            | "unicodefeff" => return Charset::Utf16Le,
            "utf-16be" | "unicodefffe" => return Charset::Utf16Be,
            "windows-1252" | "ansi_x3.4-1968" | "ascii" | "us-ascii" | "iso-8859-1"
            | "iso8859-1" | "iso_8859-1" | "latin1" | "l1" | "cp1252" | "cp819" | "ibm819"
            | "x-cp1252" => return Charset::Windows1252,
            "csisolatin9" | "iso-8859-15" | "iso8859-15" | "iso885915" | "iso_8859-15" | "l9" => {
                return Charset::Iso8859_15;
            }
            _ => {} // TODO(spec): Non-UTF/non-1252 legacy encodings (e.g. shift_jis, euc-jp, gbk) are decoded as windows-1252 because no dedicated decoder exists yet.
        }
    }

    // 3. Meta prescan (HTML §13.2.3.3) - simplified
    let limit = std::cmp::min(bytes.len(), 1024);
    let prescan_bytes = &bytes[..limit];
    // Look for <meta charset="...">
    if let Some(charset) = prescan_meta(prescan_bytes) {
        return charset;
    }

    // 4. Default
    Charset::Windows1252
}

fn prescan_meta(bytes: &[u8]) -> Option<Charset> {
    // This is a very simplified prescan.
    // Spec is much more complex, but we need to find charset in 1024 bytes.
    let s = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    if let Some(pos) = s.find("charset=") {
        let start = pos + "charset=".len();
        let bytes_s = s.as_bytes();
        // Skip leading quotes if any
        let mut actual_start = start;
        if actual_start < bytes_s.len()
            && (bytes_s[actual_start] == b'\"' || bytes_s[actual_start] == b'\'')
        {
            actual_start += 1;
        }
        let mut end = actual_start;
        while end < bytes_s.len() {
            let c = bytes_s[end];
            if c == b'\"' || c == b'\'' || c == b' ' || c == b'>' || c == b';' {
                break;
            }
            end += 1;
        }
        if actual_start < end {
            let label = &s[actual_start..end];
            match label {
                "utf-8" => return Some(Charset::Utf8),
                "utf-16le" => return Some(Charset::Utf16Le),
                "utf-16be" => return Some(Charset::Utf16Be),
                "windows-1252" => return Some(Charset::Windows1252),
                "iso-8859-15" => return Some(Charset::Iso8859_15),
                _ => {}
            }
        }
    }
    None
}

pub fn decode(bytes: &[u8], charset: Charset) -> String {
    match charset {
        Charset::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Charset::Utf16Le => decode_utf16(bytes, true),
        Charset::Utf16Be => decode_utf16(bytes, false),
        Charset::Windows1252 => decode_windows1252(bytes),
        Charset::Iso8859_15 => decode_iso8859_15(bytes),
    }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let mut result = String::new();
    let mut i = 0;
    let mut utf16_units = Vec::new();
    while i + 1 < bytes.len() {
        let unit = if little_endian {
            u16::from_le_bytes([bytes[i], bytes[i + 1]])
        } else {
            u16::from_be_bytes([bytes[i], bytes[i + 1]])
        };
        utf16_units.push(unit);
        i += 2;
    }

    let mut iter = utf16_units.into_iter().peekable();
    while let Some(u) = iter.next() {
        if (0xD800..=0xDBFF).contains(&u) {
            // High surrogate
            if let Some(&u2) = iter.peek() {
                if (0xDC00..=0xDFFF).contains(&u2) {
                    // Valid surrogate pair
                    iter.next(); // Consume u2
                    let code = 0x10000 + ((u as u32 - 0xD800) << 10) + (u2 as u32 - 0xDC00);
                    result.push(std::char::from_u32(code).unwrap_or('\u{FFFD}'));
                } else {
                    // Invalid: lone high surrogate
                    result.push('\u{FFFD}');
                }
            } else {
                // Lone high surrogate at end
                result.push('\u{FFFD}');
            }
        } else if (0xDC00..=0xDFFF).contains(&u) {
            // Lone low surrogate
            result.push('\u{FFFD}');
        } else {
            result.push(std::char::from_u32(u as u32).unwrap_or('\u{FFFD}'));
        }
    }
    result
}

fn decode_windows1252(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if (0x80..=0x9F).contains(&b) {
            let c = match b {
                0x80 => '€',
                0x82 => '‚',
                0x83 => 'ƒ',
                0x84 => '„',
                0x85 => '…',
                0x86 => '†',
                0x87 => '‡',
                0x88 => 'ˆ',
                0x89 => '‰',
                0x8A => 'Š',
                0x8B => '‹',
                0x8C => 'Œ',
                0x8E => 'Ž',
                0x91 => '‘',
                0x92 => '’',
                0x93 => '“',
                0x94 => '”',
                0x95 => '•',
                0x96 => '–',
                0x97 => '—',
                0x98 => '˜',
                0x99 => '™',
                0x9A => 'š',
                0x9B => '›',
                0x9C => 'œ',
                0x9E => 'ž',
                0x9F => 'Ÿ',
                _ => '\u{FFFD}', // Unassigned in Windows-1252: 0x81, 0x8D, 0x8F, 0x90, 0x9D
            };
            result.push(c);
        } else {
            result.push(b as char);
        }
    }
    result
}

fn decode_iso8859_15(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA4 => '\u{20AC}',
            0xA6 => '\u{0160}',
            0xA8 => '\u{0161}',
            0xB4 => '\u{017D}',
            0xB8 => '\u{017E}',
            0xBC => '\u{0152}',
            0xBD => '\u{0153}',
            0xBE => '\u{0178}',
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sniff_bom() {
        assert_eq!(
            sniff_charset(&[0xEF, 0xBB, 0xBF, b'a'], None),
            Charset::Utf8
        );
        assert_eq!(
            sniff_charset(&[0xFF, 0xFE, b'a', 0x00], None),
            Charset::Utf16Le
        );
        assert_eq!(
            sniff_charset(&[0xFE, 0xFF, 0x00, b'a'], None),
            Charset::Utf16Be
        );
    }

    #[test]
    fn test_sniff_transport() {
        assert_eq!(sniff_charset(b"abc", Some("utf-8")), Charset::Utf8);
        assert_eq!(sniff_charset(b"abc", Some("UTF-8")), Charset::Utf8);
        assert_eq!(sniff_charset(b"abc", Some("utf-16le")), Charset::Utf16Le);
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1252")),
            Charset::Windows1252
        );
    }

    #[test]
    fn test_sniff_meta() {
        let html = b"<html><head><meta charset=\"utf-8\"></head></html>";
        assert_eq!(sniff_charset(html, None), Charset::Utf8);

        let html_caps = b"<html><head><meta charset=\"UTF-8\"></head></html>";
        assert_eq!(sniff_charset(html_caps, None), Charset::Utf8);

        let html_attr = b"<html><head><meta http-equiv=\"Content-Type\" content=\"text/html; charset=windows-1252\"></head></html>";
        assert_eq!(sniff_charset(html_attr, None), Charset::Windows1252);
    }

    #[test]
    fn test_decode_utf8() {
        assert_eq!(decode(b"abc", Charset::Utf8), "abc");
        assert_eq!(decode(&[0xF0, 0x9F, 0x90, 0xA7], Charset::Utf8), "🐧");
        // Invalid UTF-8
        assert_eq!(decode(&[0xFF, b'a'], Charset::Utf8), "\u{FFFD}a");
    }

    #[test]
    fn test_decode_utf16() {
        // UTF-16LE: "abc"
        assert_eq!(
            decode(&[b'a', 0x00, b'b', 0x00, b'c', 0x00], Charset::Utf16Le),
            "abc"
        );
        // UTF-16BE: "abc"
        assert_eq!(
            decode(&[0x00, b'a', 0x00, b'b', 0x00, b'c'], Charset::Utf16Be),
            "abc"
        );

        // UTF-16LE: "🐧" (U+1F427 -> D83D DC27)
        assert_eq!(decode(&[0x3D, 0xD8, 0x27, 0xDC], Charset::Utf16Le), "🐧");
        // UTF-16BE: "🐧"
        assert_eq!(decode(&[0xD8, 0x3D, 0xDC, 0x27], Charset::Utf16Be), "🐧");

        // Lone surrogates
        assert_eq!(
            decode(&[0x3D, 0xD8, b'a', 0x00], Charset::Utf16Le),
            "\u{FFFD}a"
        );
    }

    #[test]
    fn test_decode_windows1252() {
        assert_eq!(decode(b"abc", Charset::Windows1252), "abc");
        // 0x80 in Windows-1252 is Euro sign € (U+20AC)
        assert_eq!(decode(&[0x80], Charset::Windows1252), "€");
        // 0xA3 is £ (U+00A3)
        assert_eq!(decode(&[0xA3], Charset::Windows1252), "£");
    }

    #[test]
    fn test_label_utf8_aliases() {
        assert_eq!(sniff_charset(b"abc", Some("UTF8")), Charset::Utf8);
        assert_eq!(
            sniff_charset(b"abc", Some("unicode-1-1-uTf-8")),
            Charset::Utf8
        );
    }

    #[test]
    fn test_label_latin1_alias() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-1")),
            Charset::Windows1252
        );
        assert_eq!(sniff_charset(b"abc", Some("latin1")), Charset::Windows1252);
    }

    #[test]
    fn test_label_ascii_alias() {
        assert_eq!(
            sniff_charset(b"abc", Some("us-ascii")),
            Charset::Windows1252
        );
    }

    #[test]
    fn test_label_utf16_bare() {
        assert_eq!(sniff_charset(b"abc", Some("utf-16")), Charset::Utf16Le);
    }

    #[test]
    fn test_label_whitespace_trimmed() {
        assert_eq!(sniff_charset(b"abc", Some("  utf-8  ")), Charset::Utf8);
        assert_eq!(sniff_charset(b"abc", Some("\tutf-8\r\n")), Charset::Utf8);
    }

    #[test]
    fn test_label_unknown_falls_through_to_default() {
        assert_eq!(
            sniff_charset(b"abc", Some("unknown-charset")),
            Charset::Windows1252
        );
    }

    #[test]
    fn test_iso8859_15_decode() {
        assert_eq!(decode(&[0xA4], Charset::Iso8859_15), "€");
        assert_eq!(decode(&[0xBD], Charset::Iso8859_15), "œ");
        assert_eq!(decode(&[0xBE], Charset::Iso8859_15), "Ÿ");
        assert_eq!(decode(&[0x41], Charset::Iso8859_15), "A");
        assert_eq!(decode(&[0xE9], Charset::Iso8859_15), "é");
    }

    #[test]
    fn test_iso8859_15_sniff() {
        assert_eq!(
            sniff_charset(b"<html></html>", Some("iso-8859-15")),
            Charset::Iso8859_15
        );
        assert_eq!(
            sniff_charset(b"<html></html>", Some("l9")),
            Charset::Iso8859_15
        );
    }
}
