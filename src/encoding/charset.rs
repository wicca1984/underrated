//! Charset detection and decoding for HTML documents.
// spec: https://html.spec.whatwg.org/multipage/parsing.html#determining-the-character-encoding

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Utf8,
    Utf16Le,
    Utf16Be,
    Windows1252,
    Windows1251,
    Windows1250,
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
            "windows-1251" | "cp1251" | "x-cp1251" => return Charset::Windows1251,
            "windows-1250" | "cp1250" | "x-cp1250" => return Charset::Windows1250,
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
                "windows-1251" => return Some(Charset::Windows1251),
                "windows-1250" => return Some(Charset::Windows1250),
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
        Charset::Windows1251 => decode_windows1251(bytes),
        Charset::Windows1250 => decode_windows1250(bytes),
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

const WINDOWS_1251_MAP: [char; 128] = [
    '\u{0402}', // 0x80
    '\u{0403}', // 0x81
    '\u{201A}', // 0x82
    '\u{0453}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{20AC}', // 0x88
    '\u{2030}', // 0x89
    '\u{0409}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{040A}', // 0x8C
    '\u{040C}', // 0x8D
    '\u{040B}', // 0x8E
    '\u{040F}', // 0x8F
    '\u{0452}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{FFFD}', // 0x98 (Undefined)
    '\u{2122}', // 0x99
    '\u{0459}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{045A}', // 0x9C
    '\u{045C}', // 0x9D
    '\u{045B}', // 0x9E
    '\u{045F}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{040E}', // 0xA1
    '\u{045E}', // 0xA2
    '\u{0408}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{0490}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{0401}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{0404}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{0407}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{0406}', // 0xB2
    '\u{0456}', // 0xB3
    '\u{0491}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{0451}', // 0xB8
    '\u{2116}', // 0xB9
    '\u{0454}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{0458}', // 0xBC
    '\u{0405}', // 0xBD
    '\u{0455}', // 0xBE
    '\u{0457}', // 0xBF
    '\u{0410}', // 0xC0
    '\u{0411}', // 0xC1
    '\u{0412}', // 0xC2
    '\u{0413}', // 0xC3
    '\u{0414}', // 0xC4
    '\u{0415}', // 0xC5
    '\u{0416}', // 0xC6
    '\u{0417}', // 0xC7
    '\u{0418}', // 0xC8
    '\u{0419}', // 0xC9
    '\u{041A}', // 0xCA
    '\u{041B}', // 0xCB
    '\u{041C}', // 0xCC
    '\u{041D}', // 0xCD
    '\u{041E}', // 0xCE
    '\u{041F}', // 0xCF
    '\u{0420}', // 0xD0
    '\u{0421}', // 0xD1
    '\u{0422}', // 0xD2
    '\u{0423}', // 0xD3
    '\u{0424}', // 0xD4
    '\u{0425}', // 0xD5
    '\u{0426}', // 0xD6
    '\u{0427}', // 0xD7
    '\u{0428}', // 0xD8
    '\u{0429}', // 0xD9
    '\u{042A}', // 0xDA
    '\u{042B}', // 0xDB
    '\u{042C}', // 0xDC
    '\u{042D}', // 0xDD
    '\u{042E}', // 0xDE
    '\u{042F}', // 0xDF
    '\u{0430}', // 0xE0
    '\u{0431}', // 0xE1
    '\u{0432}', // 0xE2
    '\u{0433}', // 0xE3
    '\u{0434}', // 0xE4
    '\u{0435}', // 0xE5
    '\u{0436}', // 0xE6
    '\u{0437}', // 0xE7
    '\u{0438}', // 0xE8
    '\u{0439}', // 0xE9
    '\u{043A}', // 0xEA
    '\u{043B}', // 0xEB
    '\u{043C}', // 0xEC
    '\u{043D}', // 0xED
    '\u{043E}', // 0xEE
    '\u{043F}', // 0xEF
    '\u{0440}', // 0xF0
    '\u{0441}', // 0xF1
    '\u{0442}', // 0xF2
    '\u{0443}', // 0xF3
    '\u{0444}', // 0xF4
    '\u{0445}', // 0xF5
    '\u{0446}', // 0xF6
    '\u{0447}', // 0xF7
    '\u{0448}', // 0xF8
    '\u{0449}', // 0xF9
    '\u{044A}', // 0xFA
    '\u{044B}', // 0xFB
    '\u{044C}', // 0xFC
    '\u{044D}', // 0xFD
    '\u{044E}', // 0xFE
    '\u{044F}', // 0xFF
];

fn decode_windows1251(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1251_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const WINDOWS_1250_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{FFFD}', // 0x81
    '\u{201A}', // 0x82
    '\u{FFFD}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{FFFD}', // 0x88
    '\u{2030}', // 0x89
    '\u{0160}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{015A}', // 0x8C
    '\u{0164}', // 0x8D
    '\u{017D}', // 0x8E
    '\u{0179}', // 0x8F
    '\u{FFFD}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{FFFD}', // 0x98
    '\u{2122}', // 0x99
    '\u{0161}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{015B}', // 0x9C
    '\u{0165}', // 0x9D
    '\u{017E}', // 0x9E
    '\u{017A}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{02C7}', // 0xA1
    '\u{02D8}', // 0xA2
    '\u{0141}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{0104}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{015E}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{017B}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{02DB}', // 0xB2
    '\u{0142}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00B8}', // 0xB8
    '\u{0105}', // 0xB9
    '\u{015F}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{013D}', // 0xBC
    '\u{02DD}', // 0xBD
    '\u{013E}', // 0xBE
    '\u{017C}', // 0xBF
    '\u{0154}', // 0xC0
    '\u{00C1}', // 0xC1
    '\u{00C2}', // 0xC2
    '\u{0102}', // 0xC3
    '\u{00C4}', // 0xC4
    '\u{0139}', // 0xC5
    '\u{0106}', // 0xC6
    '\u{00C7}', // 0xC7
    '\u{00C8}', // 0xC8
    '\u{00C9}', // 0xC9
    '\u{0118}', // 0xCA
    '\u{00CB}', // 0xCB
    '\u{011A}', // 0xCC
    '\u{00CD}', // 0xCD
    '\u{00CE}', // 0xCE
    '\u{010E}', // 0xCF
    '\u{0110}', // 0xD0
    '\u{0143}', // 0xD1
    '\u{0147}', // 0xD2
    '\u{00D3}', // 0xD3
    '\u{00D4}', // 0xD4
    '\u{0150}', // 0xD5
    '\u{00D6}', // 0xD6
    '\u{00D7}', // 0xD7
    '\u{0158}', // 0xD8
    '\u{016E}', // 0xD9
    '\u{00DA}', // 0xDA
    '\u{0170}', // 0xDB
    '\u{00DC}', // 0xDC
    '\u{00DD}', // 0xDD
    '\u{0162}', // 0xDE
    '\u{00DF}', // 0xDF
    '\u{0155}', // 0xE0
    '\u{00E1}', // 0xE1
    '\u{00E2}', // 0xE2
    '\u{0103}', // 0xE3
    '\u{00E4}', // 0xE4
    '\u{013A}', // 0xE5
    '\u{0107}', // 0xE6
    '\u{00E7}', // 0xE7
    '\u{00E8}', // 0xE8
    '\u{00E9}', // 0xE9
    '\u{0119}', // 0xEA
    '\u{00EB}', // 0xEB
    '\u{011B}', // 0xEC
    '\u{00ED}', // 0xED
    '\u{00EE}', // 0xEE
    '\u{010F}', // 0xEF
    '\u{0111}', // 0xF0
    '\u{0144}', // 0xF1
    '\u{0148}', // 0xF2
    '\u{00F3}', // 0xF3
    '\u{00F4}', // 0xF4
    '\u{0151}', // 0xF5
    '\u{00F6}', // 0xF6
    '\u{00F7}', // 0xF7
    '\u{0159}', // 0xF8
    '\u{016F}', // 0xF9
    '\u{00FA}', // 0xFA
    '\u{0171}', // 0xFB
    '\u{00FC}', // 0xFC
    '\u{00FD}', // 0xFD
    '\u{0163}', // 0xFE
    '\u{02D9}', // 0xFF
];

fn decode_windows1250(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1250_MAP[(b - 0x80) as usize]);
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

    #[test]
    fn test_windows1251_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1251")),
            Charset::Windows1251
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1251")), Charset::Windows1251);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1251")),
            Charset::Windows1251
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1251\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1251);
    }

    #[test]
    fn test_windows1251_decode() {
        // Pure-ASCII round-trip
        assert_eq!(decode(b"abc 123", Charset::Windows1251), "abc 123");

        // Cyrillic specific bytes:
        // 0xC0 -> "А" (U+0410)
        assert_eq!(decode(&[0xC0], Charset::Windows1251), "А");
        // 0xFF -> "я" (U+044F)
        assert_eq!(decode(&[0xFF], Charset::Windows1251), "я");
        // 0xA8 -> "Ё" (U+0401)
        assert_eq!(decode(&[0xA8], Charset::Windows1251), "Ё");
        // 0xB8 -> "ё" (U+0451)
        assert_eq!(decode(&[0xB8], Charset::Windows1251), "ё");

        // Check unmapped 0x98 -> U+FFFD (replacement char)
        assert_eq!(decode(&[0x98], Charset::Windows1251), "\u{FFFD}");

        // Check a full Cyrillic sentence: "Привет" in windows-1251 bytes:
        // П: 0xCF, р: 0xF0, i: 0xE8, в: 0xE2, е: 0xE5, т: 0xF2
        let bytes = &[0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2];
        assert_eq!(decode(bytes, Charset::Windows1251), "Привет");
    }

    #[test]
    fn test_windows1250_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1250")),
            Charset::Windows1250
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1250")), Charset::Windows1250);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1250")),
            Charset::Windows1250
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1250\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1250);
    }

    #[test]
    fn test_windows1250_decode() {
        // Pure-ASCII round-trip
        assert_eq!(decode(b"abc 123", Charset::Windows1250), "abc 123");

        // Authoritative windows-1250 anchors:
        // 0xC8 -> È (U+00C8)
        assert_eq!(decode(&[0xC8], Charset::Windows1250), "È");
        // 0xE8 -> è (U+00E8)
        assert_eq!(decode(&[0xE8], Charset::Windows1250), "è");
        // 0x8A -> Š (U+0160)
        assert_eq!(decode(&[0x8A], Charset::Windows1250), "Š");
        // 0x9A -> š (U+0161)
        assert_eq!(decode(&[0x9A], Charset::Windows1250), "š");
        // 0x8E -> Ž (U+017D)
        assert_eq!(decode(&[0x8E], Charset::Windows1250), "Ž");
        // 0x9E -> ž (U+017E)
        assert_eq!(decode(&[0x9E], Charset::Windows1250), "ž");
        // 0xA3 -> Ł (U+0141)
        assert_eq!(decode(&[0xA3], Charset::Windows1250), "Ł");
        // 0xB3 -> ł (U+0142)
        assert_eq!(decode(&[0xB3], Charset::Windows1250), "ł");
    }
}
