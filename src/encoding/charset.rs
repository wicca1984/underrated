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
    Windows1253,
    Windows1254,
    Windows1255,
    Windows1256,
    Windows1257,
    Windows1258,
    Iso8859_1,
    Iso8859_15,
    Iso8859_2,
    Iso8859_3,
    Iso8859_4,
    Iso8859_5,
    Iso8859_7,
    Iso8859_13,
    Iso8859_10,
    Iso8859_16,
    Iso8859_9,
    Iso8859_6,
    Koi8R,
    Koi8U,
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
            "windows-1252" | "ansi_x3.4-1968" | "ascii" | "us-ascii" | "cp1252" | "x-cp1252" => {
                return Charset::Windows1252;
            }
            "iso-8859-1" | "iso8859-1" | "iso_8859-1" | "latin1" | "l1" | "cp819" | "ibm819" => {
                return Charset::Iso8859_1;
            }
            "windows-1251" | "cp1251" | "x-cp1251" => return Charset::Windows1251,
            "windows-1250" | "cp1250" | "x-cp1250" => return Charset::Windows1250,
            "windows-1253" | "cp1253" | "x-cp1253" => return Charset::Windows1253,
            "windows-1254" | "cp1254" | "x-cp1254" => return Charset::Windows1254,
            "windows-1255" | "cp1255" | "x-cp1255" => return Charset::Windows1255,
            "windows-1256" | "cp1256" | "x-cp1256" => return Charset::Windows1256,
            "windows-1257" | "cp1257" | "x-cp1257" => return Charset::Windows1257,
            "windows-1258" | "cp1258" | "x-cp1258" => return Charset::Windows1258,
            "csisolatin9" | "iso-8859-15" | "iso8859-15" | "iso885915" | "iso_8859-15" | "l9" => {
                return Charset::Iso8859_15;
            }
            "iso-8859-2" | "iso8859-2" | "iso88592" | "iso_8859-2" | "iso-ir-101"
            | "csisolatin2" | "latin2" | "l2" => {
                return Charset::Iso8859_2;
            }
            "iso-8859-3" | "iso8859-3" | "iso88593" | "iso_8859-3" | "iso-ir-109"
            | "csisolatin3" | "latin3" | "l3" => {
                return Charset::Iso8859_3;
            }
            "iso-8859-4" | "iso8859-4" | "iso88594" | "iso_8859-4" | "iso-ir-110"
            | "iso_8859-4:1988" | "csisolatin4" | "latin4" | "l4" => {
                return Charset::Iso8859_4;
            }
            "iso-8859-5" | "iso8859-5" | "iso88595" | "iso_8859-5" | "iso-ir-144"
            | "iso_8859-5:1988" | "csisolatincyrillic" | "cyrillic" => {
                return Charset::Iso8859_5;
            }
            "iso-8859-7" | "iso8859-7" | "iso88597" | "iso_8859-7" | "iso_8859-7:1987"
            | "iso-ir-126" | "csisolatingreek" | "elot_928" | "ecma-118" | "greek" | "greek8"
            | "sun_eu_greek" => {
                return Charset::Iso8859_7;
            }
            "iso-8859-13" | "iso8859-13" | "iso885913" | "iso_8859-13" | "iso_8859_13" | "l7"
            | "latin7" => {
                return Charset::Iso8859_13;
            }
            "iso-8859-10" | "iso8859-10" | "iso885910" | "iso_8859-10" | "iso_8859_10" | "l6"
            | "latin6" | "iso-ir-157" | "csisolatin6" => {
                return Charset::Iso8859_10;
            }
            "iso-8859-16" | "iso8859-16" | "iso885916" | "iso_8859-16" | "iso_8859_16" | "l10"
            | "latin10" => {
                return Charset::Iso8859_16;
            }
            "iso-8859-9" | "iso8859-9" | "iso88599" | "iso_8859-9" | "iso_8859_9" | "l5"
            | "latin5" | "csisolatin5" => {
                return Charset::Iso8859_9;
            }
            "iso-8859-6" | "iso8859-6" | "iso88596" | "iso_8859-6" | "iso_8859_6" | "arabic"
            | "csisolatingrabic" | "iso-ir-127" | "iso_8859-6:1987" | "ecma-114" | "asmo-708" => {
                return Charset::Iso8859_6;
            }
            "koi8-r" | "koi8_r" | "cskoi8r" => {
                return Charset::Koi8R;
            }
            "koi8-u" | "koi8_u" | "koi8-ru" => {
                return Charset::Koi8U;
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
                "windows-1253" => return Some(Charset::Windows1253),
                "windows-1254" => return Some(Charset::Windows1254),
                "windows-1255" => return Some(Charset::Windows1255),
                "windows-1256" => return Some(Charset::Windows1256),
                "windows-1257" => return Some(Charset::Windows1257),
                "windows-1258" => return Some(Charset::Windows1258),
                "iso-8859-1" | "iso8859-1" | "iso_8859-1" | "latin1" | "l1" | "cp819"
                | "ibm819" => {
                    return Some(Charset::Iso8859_1);
                }
                "iso-8859-15" => return Some(Charset::Iso8859_15),
                "iso-8859-2" => return Some(Charset::Iso8859_2),
                "iso-8859-3" => return Some(Charset::Iso8859_3),
                "iso-8859-4" => return Some(Charset::Iso8859_4),
                "iso-8859-5" => return Some(Charset::Iso8859_5),
                "iso-8859-7" => return Some(Charset::Iso8859_7),
                "iso-8859-13" => return Some(Charset::Iso8859_13),
                "iso-8859-10" => return Some(Charset::Iso8859_10),
                "iso-8859-16" | "iso8859-16" | "iso885916" | "iso_8859-16" | "iso_8859_16"
                | "l10" | "latin10" => return Some(Charset::Iso8859_16),
                "iso-8859-9" | "iso8859-9" | "iso88599" | "iso_8859-9" | "iso_8859_9" | "l5"
                | "latin5" | "csisolatin5" => return Some(Charset::Iso8859_9),
                "iso-8859-6" | "iso8859-6" | "iso88596" | "iso_8859-6" | "iso_8859_6"
                | "arabic" | "csisolatingrabic" | "iso-ir-127" | "iso_8859-6:1987" | "ecma-114"
                | "asmo-708" => return Some(Charset::Iso8859_6),
                "koi8-r" => return Some(Charset::Koi8R),
                "koi8-u" => return Some(Charset::Koi8U),
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
        Charset::Windows1253 => decode_windows1253(bytes),
        Charset::Windows1254 => decode_windows1254(bytes),
        Charset::Windows1255 => decode_windows1255(bytes),
        Charset::Windows1256 => decode_windows1256(bytes),
        Charset::Windows1257 => decode_windows1257(bytes),
        Charset::Windows1258 => decode_windows1258(bytes),
        Charset::Iso8859_1 => decode_iso8859_1(bytes),
        Charset::Iso8859_15 => decode_iso8859_15(bytes),
        Charset::Iso8859_2 => decode_iso8859_2(bytes),
        Charset::Iso8859_3 => decode_iso8859_3(bytes),
        Charset::Iso8859_4 => decode_iso8859_4(bytes),
        Charset::Iso8859_5 => decode_iso8859_5(bytes),
        Charset::Iso8859_7 => decode_iso8859_7(bytes),
        Charset::Iso8859_13 => decode_iso8859_13(bytes),
        Charset::Iso8859_10 => decode_iso8859_10(bytes),
        Charset::Iso8859_16 => decode_iso8859_16(bytes),
        Charset::Iso8859_9 => decode_iso8859_9(bytes),
        Charset::Iso8859_6 => decode_iso8859_6(bytes),
        Charset::Koi8R => decode_koi8r(bytes),
        Charset::Koi8U => decode_koi8u(bytes),
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

fn decode_iso8859_1(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        result.push(char::from(b));
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

fn decode_iso8859_2(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA0 => '\u{00A0}',
            0xA1 => '\u{0104}',
            0xA2 => '\u{02D8}',
            0xA3 => '\u{0141}',
            0xA4 => '\u{00A4}',
            0xA5 => '\u{013D}',
            0xA6 => '\u{015A}',
            0xA7 => '\u{00A7}',
            0xA8 => '\u{00A8}',
            0xA9 => '\u{0160}',
            0xAA => '\u{015E}',
            0xAB => '\u{0164}',
            0xAC => '\u{0179}',
            0xAD => '\u{00AD}',
            0xAE => '\u{017D}',
            0xAF => '\u{017B}',
            0xB0 => '\u{00B0}',
            0xB1 => '\u{0105}',
            0xB2 => '\u{02DB}',
            0xB3 => '\u{0142}',
            0xB4 => '\u{00B4}',
            0xB5 => '\u{013E}',
            0xB6 => '\u{015B}',
            0xB7 => '\u{02C7}',
            0xB8 => '\u{00B8}',
            0xB9 => '\u{0161}',
            0xBA => '\u{015F}',
            0xBB => '\u{0165}',
            0xBC => '\u{017A}',
            0xBD => '\u{02DD}',
            0xBE => '\u{017E}',
            0xBF => '\u{017C}',
            0xC0 => '\u{0154}',
            0xC1 => '\u{00C1}',
            0xC2 => '\u{00C2}',
            0xC3 => '\u{0102}',
            0xC4 => '\u{00C4}',
            0xC5 => '\u{0139}',
            0xC6 => '\u{0106}',
            0xC7 => '\u{00C7}',
            0xC8 => '\u{010C}',
            0xC9 => '\u{00C9}',
            0xCA => '\u{0118}',
            0xCB => '\u{00CB}',
            0xCC => '\u{011A}',
            0xCD => '\u{00CD}',
            0xCE => '\u{00CE}',
            0xCF => '\u{010E}',
            0xD0 => '\u{0110}',
            0xD1 => '\u{0143}',
            0xD2 => '\u{0147}',
            0xD3 => '\u{00D3}',
            0xD4 => '\u{00D4}',
            0xD5 => '\u{0150}',
            0xD6 => '\u{00D6}',
            0xD7 => '\u{00D7}',
            0xD8 => '\u{0158}',
            0xD9 => '\u{016E}',
            0xDA => '\u{00DA}',
            0xDB => '\u{0170}',
            0xDC => '\u{00DC}',
            0xDD => '\u{00DD}',
            0xDE => '\u{0162}',
            0xDF => '\u{00DF}',
            0xE0 => '\u{0155}',
            0xE1 => '\u{00E1}',
            0xE2 => '\u{00E2}',
            0xE3 => '\u{0103}',
            0xE4 => '\u{00E4}',
            0xE5 => '\u{013A}',
            0xE6 => '\u{0107}',
            0xE7 => '\u{00E7}',
            0xE8 => '\u{010D}',
            0xE9 => '\u{00E9}',
            0xEA => '\u{0119}',
            0xEB => '\u{00EB}',
            0xEC => '\u{011B}',
            0xED => '\u{00ED}',
            0xEE => '\u{00EE}',
            0xEF => '\u{010F}',
            0xF0 => '\u{0111}',
            0xF1 => '\u{0144}',
            0xF2 => '\u{0148}',
            0xF3 => '\u{00F3}',
            0xF4 => '\u{00F4}',
            0xF5 => '\u{0151}',
            0xF6 => '\u{00F6}',
            0xF7 => '\u{00F7}',
            0xF8 => '\u{0159}',
            0xF9 => '\u{016F}',
            0xFA => '\u{00FA}',
            0xFB => '\u{0171}',
            0xFC => '\u{00FC}',
            0xFD => '\u{00FD}',
            0xFE => '\u{0163}',
            0xFF => '\u{02D9}',
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

fn decode_iso8859_3(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA0 => '\u{00A0}',
            0xA1 => '\u{0126}',
            0xA2 => '\u{02D8}',
            0xA3 => '\u{00A3}',
            0xA4 => '\u{00A4}',
            0xA5 => '\u{FFFD}',
            0xA6 => '\u{0124}',
            0xA7 => '\u{00A7}',
            0xA8 => '\u{00A8}',
            0xA9 => '\u{0130}',
            0xAA => '\u{015E}',
            0xAB => '\u{011E}',
            0xAC => '\u{0134}',
            0xAD => '\u{00AD}',
            0xAE => '\u{FFFD}',
            0xAF => '\u{017B}',
            0xB0 => '\u{00B0}',
            0xB1 => '\u{0127}',
            0xB2 => '\u{00B2}',
            0xB3 => '\u{00B3}',
            0xB4 => '\u{00B4}',
            0xB5 => '\u{00B5}',
            0xB6 => '\u{0125}',
            0xB7 => '\u{00B7}',
            0xB8 => '\u{00B8}',
            0xB9 => '\u{0131}',
            0xBA => '\u{015F}',
            0xBB => '\u{011F}',
            0xBC => '\u{0135}',
            0xBD => '\u{00BD}',
            0xBE => '\u{FFFD}',
            0xBF => '\u{017C}',
            0xC0 => '\u{00C0}',
            0xC1 => '\u{00C1}',
            0xC2 => '\u{00C2}',
            0xC3 => '\u{FFFD}',
            0xC4 => '\u{00C4}',
            0xC5 => '\u{010A}',
            0xC6 => '\u{0108}',
            0xC7 => '\u{00C7}',
            0xC8 => '\u{00C8}',
            0xC9 => '\u{00C9}',
            0xCA => '\u{00CA}',
            0xCB => '\u{00CB}',
            0xCC => '\u{00CC}',
            0xCD => '\u{00CD}',
            0xCE => '\u{00CE}',
            0xCF => '\u{00CF}',
            0xD0 => '\u{FFFD}',
            0xD1 => '\u{00D1}',
            0xD2 => '\u{00D2}',
            0xD3 => '\u{00D3}',
            0xD4 => '\u{00D4}',
            0xD5 => '\u{0120}',
            0xD6 => '\u{00D6}',
            0xD7 => '\u{00D7}',
            0xD8 => '\u{011C}',
            0xD9 => '\u{00D9}',
            0xDA => '\u{00DA}',
            0xDB => '\u{00DB}',
            0xDC => '\u{00DC}',
            0xDD => '\u{016C}',
            0xDE => '\u{015C}',
            0xDF => '\u{00DF}',
            0xE0 => '\u{00E0}',
            0xE1 => '\u{00E1}',
            0xE2 => '\u{00E2}',
            0xE3 => '\u{FFFD}',
            0xE4 => '\u{00E4}',
            0xE5 => '\u{010B}',
            0xE6 => '\u{0109}',
            0xE7 => '\u{00E7}',
            0xE8 => '\u{00E8}',
            0xE9 => '\u{00E9}',
            0xEA => '\u{00EA}',
            0xEB => '\u{00EB}',
            0xEC => '\u{00EC}',
            0xED => '\u{00ED}',
            0xEE => '\u{00EE}',
            0xEF => '\u{00EF}',
            0xF0 => '\u{FFFD}',
            0xF1 => '\u{00F1}',
            0xF2 => '\u{00F2}',
            0xF3 => '\u{00F3}',
            0xF4 => '\u{00F4}',
            0xF5 => '\u{0121}',
            0xF6 => '\u{00F6}',
            0xF7 => '\u{00F7}',
            0xF8 => '\u{011D}',
            0xF9 => '\u{00F9}',
            0xFA => '\u{00FA}',
            0xFB => '\u{00FB}',
            0xFC => '\u{00FC}',
            0xFD => '\u{016D}',
            0xFE => '\u{015D}',
            0xFF => '\u{02D9}',
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

fn decode_iso8859_4(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA0 => '\u{00A0}', // NO-BREAK SPACE
            0xA1 => '\u{0104}', // LATIN CAPITAL LETTER A WITH OGONEK
            0xA2 => '\u{0138}', // LATIN SMALL LETTER KRA
            0xA3 => '\u{0156}', // LATIN CAPITAL LETTER R WITH CEDILLA
            0xA4 => '\u{00A4}', // CURRENCY SIGN
            0xA5 => '\u{0128}', // LATIN CAPITAL LETTER I WITH TILDE
            0xA6 => '\u{013B}', // LATIN CAPITAL LETTER L WITH CEDILLA
            0xA7 => '\u{00A7}', // SECTION SIGN
            0xA8 => '\u{00A8}', // DIAERESIS
            0xA9 => '\u{0160}', // LATIN CAPITAL LETTER S WITH CARON
            0xAA => '\u{0112}', // LATIN CAPITAL LETTER E WITH MACRON
            0xAB => '\u{0122}', // LATIN CAPITAL LETTER G WITH CEDILLA
            0xAC => '\u{0166}', // LATIN CAPITAL LETTER T WITH STROKE
            0xAD => '\u{00AD}', // SOFT HYPHEN
            0xAE => '\u{017D}', // LATIN CAPITAL LETTER Z WITH CARON
            0xAF => '\u{00AF}', // MACRON
            0xB0 => '\u{00B0}', // DEGREE SIGN
            0xB1 => '\u{0105}', // LATIN SMALL LETTER A WITH OGONEK
            0xB2 => '\u{02DB}', // OGONEK
            0xB3 => '\u{0157}', // LATIN SMALL LETTER R WITH CEDILLA
            0xB4 => '\u{00B4}', // ACUTE ACCENT
            0xB5 => '\u{0129}', // LATIN SMALL LETTER I WITH TILDE
            0xB6 => '\u{013C}', // LATIN SMALL LETTER L WITH CEDILLA
            0xB7 => '\u{02C7}', // CARON
            0xB8 => '\u{00B8}', // CEDILLA
            0xB9 => '\u{0161}', // LATIN SMALL LETTER S WITH CARON
            0xBA => '\u{0113}', // LATIN SMALL LETTER E WITH MACRON
            0xBB => '\u{0123}', // LATIN SMALL LETTER G WITH CEDILLA
            0xBC => '\u{0167}', // LATIN SMALL LETTER T WITH STROKE
            0xBD => '\u{014A}', // LATIN CAPITAL LETTER ENG
            0xBE => '\u{017E}', // LATIN SMALL LETTER Z WITH CARON
            0xBF => '\u{014B}', // LATIN SMALL LETTER ENG
            0xC0 => '\u{0100}', // LATIN CAPITAL LETTER A WITH MACRON
            0xC1 => '\u{00C1}', // LATIN CAPITAL LETTER A WITH ACUTE
            0xC2 => '\u{00C2}', // LATIN CAPITAL LETTER A WITH CIRCUMFLEX
            0xC3 => '\u{00C3}', // LATIN CAPITAL LETTER A WITH TILDE
            0xC4 => '\u{00C4}', // LATIN CAPITAL LETTER A WITH DIAERESIS
            0xC5 => '\u{00C5}', // LATIN CAPITAL LETTER A WITH RING ABOVE
            0xC6 => '\u{00C6}', // LATIN CAPITAL LETTER AE
            0xC7 => '\u{012E}', // LATIN CAPITAL LETTER I WITH OGONEK
            0xC8 => '\u{010C}', // LATIN CAPITAL LETTER C WITH CARON
            0xC9 => '\u{00C9}', // LATIN CAPITAL LETTER E WITH ACUTE
            0xCA => '\u{0118}', // LATIN CAPITAL LETTER E WITH OGONEK
            0xCB => '\u{00CB}', // LATIN CAPITAL LETTER E WITH DIAERESIS
            0xCC => '\u{0116}', // LATIN CAPITAL LETTER E WITH DOT ABOVE
            0xCD => '\u{00CD}', // LATIN CAPITAL LETTER I WITH ACUTE
            0xCE => '\u{00CE}', // LATIN CAPITAL LETTER I WITH CIRCUMFLEX
            0xCF => '\u{012A}', // LATIN CAPITAL LETTER I WITH MACRON
            0xD0 => '\u{0110}', // LATIN CAPITAL LETTER D WITH STROKE
            0xD1 => '\u{0145}', // LATIN CAPITAL LETTER N WITH CEDILLA
            0xD2 => '\u{014C}', // LATIN CAPITAL LETTER O WITH MACRON
            0xD3 => '\u{0136}', // LATIN CAPITAL LETTER K WITH CEDILLA
            0xD4 => '\u{00D4}', // LATIN CAPITAL LETTER O WITH CIRCUMFLEX
            0xD5 => '\u{00D5}', // LATIN CAPITAL LETTER O WITH TILDE
            0xD6 => '\u{00D6}', // LATIN CAPITAL LETTER O WITH DIAERESIS
            0xD7 => '\u{00D7}', // MULTIPLICATION SIGN
            0xD8 => '\u{00D8}', // LATIN CAPITAL LETTER O WITH STROKE
            0xD9 => '\u{0172}', // LATIN CAPITAL LETTER U WITH OGONEK
            0xDA => '\u{00DA}', // LATIN CAPITAL LETTER U WITH ACUTE
            0xDB => '\u{00DB}', // LATIN CAPITAL LETTER U WITH CIRCUMFLEX
            0xDC => '\u{00DC}', // LATIN CAPITAL LETTER U WITH DIAERESIS
            0xDD => '\u{0168}', // LATIN CAPITAL LETTER U WITH TILDE
            0xDE => '\u{016A}', // LATIN CAPITAL LETTER U WITH MACRON
            0xDF => '\u{00DF}', // LATIN SMALL LETTER SHARP S
            0xE0 => '\u{0101}', // LATIN SMALL LETTER A WITH MACRON
            0xE1 => '\u{00E1}', // LATIN SMALL LETTER A WITH ACUTE
            0xE2 => '\u{00E2}', // LATIN SMALL LETTER A WITH CIRCUMFLEX
            0xE3 => '\u{00E3}', // LATIN SMALL LETTER A WITH TILDE
            0xE4 => '\u{00E4}', // LATIN SMALL LETTER A WITH DIAERESIS
            0xE5 => '\u{00E5}', // LATIN SMALL LETTER A WITH RING ABOVE
            0xE6 => '\u{00E6}', // LATIN SMALL LETTER AE
            0xE7 => '\u{012F}', // LATIN SMALL LETTER I WITH OGONEK
            0xE8 => '\u{010D}', // LATIN SMALL LETTER C WITH CARON
            0xE9 => '\u{00E9}', // LATIN SMALL LETTER E WITH ACUTE
            0xEA => '\u{0119}', // LATIN SMALL LETTER E WITH OGONEK
            0xEB => '\u{00EB}', // LATIN SMALL LETTER E WITH DIAERESIS
            0xEC => '\u{0117}', // LATIN SMALL LETTER E WITH DOT ABOVE
            0xED => '\u{00ED}', // LATIN SMALL LETTER I WITH ACUTE
            0xEE => '\u{00EE}', // LATIN SMALL LETTER I WITH CIRCUMFLEX
            0xEF => '\u{012B}', // LATIN SMALL LETTER I WITH MACRON
            0xF0 => '\u{0111}', // LATIN SMALL LETTER D WITH STROKE
            0xF1 => '\u{0146}', // LATIN SMALL LETTER N WITH CEDILLA
            0xF2 => '\u{014D}', // LATIN SMALL LETTER O WITH MACRON
            0xF3 => '\u{0137}', // LATIN SMALL LETTER K WITH CEDILLA
            0xF4 => '\u{00F4}', // LATIN SMALL LETTER O WITH CIRCUMFLEX
            0xF5 => '\u{00F5}', // LATIN SMALL LETTER O WITH TILDE
            0xF6 => '\u{00F6}', // LATIN SMALL LETTER O WITH DIAERESIS
            0xF7 => '\u{00F7}', // DIVISION SIGN
            0xF8 => '\u{00F8}', // LATIN SMALL LETTER O WITH STROKE
            0xF9 => '\u{0173}', // LATIN SMALL LETTER U WITH OGONEK
            0xFA => '\u{00FA}', // LATIN SMALL LETTER U WITH ACUTE
            0xFB => '\u{00FB}', // LATIN SMALL LETTER U WITH CIRCUMFLEX
            0xFC => '\u{00FC}', // LATIN SMALL LETTER U WITH DIAERESIS
            0xFD => '\u{0169}', // LATIN SMALL LETTER U WITH TILDE
            0xFE => '\u{016B}', // LATIN SMALL LETTER U WITH MACRON
            0xFF => '\u{02D9}', // DOT ABOVE
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

fn decode_iso8859_5(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA0 => '\u{00A0}', // NO-BREAK SPACE
            0xA1 => '\u{0401}', // CYRILLIC CAPITAL LETTER IO
            0xA2 => '\u{0402}', // CYRILLIC CAPITAL LETTER DJE
            0xA3 => '\u{0403}', // CYRILLIC CAPITAL LETTER GJE
            0xA4 => '\u{0404}', // CYRILLIC CAPITAL LETTER UKRAINIAN IE
            0xA5 => '\u{0405}', // CYRILLIC CAPITAL LETTER DZE
            0xA6 => '\u{0406}', // CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I
            0xA7 => '\u{0407}', // CYRILLIC CAPITAL LETTER YI
            0xA8 => '\u{0408}', // CYRILLIC CAPITAL LETTER JE
            0xA9 => '\u{0409}', // CYRILLIC CAPITAL LETTER LJE
            0xAA => '\u{040A}', // CYRILLIC CAPITAL LETTER NJE
            0xAB => '\u{040B}', // CYRILLIC CAPITAL LETTER TSHE
            0xAC => '\u{040C}', // CYRILLIC CAPITAL LETTER KJE
            0xAD => '\u{00AD}', // SOFT HYPHEN
            0xAE => '\u{040E}', // CYRILLIC CAPITAL LETTER SHORT U
            0xAF => '\u{040F}', // CYRILLIC CAPITAL LETTER DZHE
            0xB0 => '\u{0410}', // CYRILLIC CAPITAL LETTER A
            0xB1 => '\u{0411}', // CYRILLIC CAPITAL LETTER BE
            0xB2 => '\u{0412}', // CYRILLIC CAPITAL LETTER VE
            0xB3 => '\u{0413}', // CYRILLIC CAPITAL LETTER GHE
            0xB4 => '\u{0414}', // CYRILLIC CAPITAL LETTER DE
            0xB5 => '\u{0415}', // CYRILLIC CAPITAL LETTER IE
            0xB6 => '\u{0416}', // CYRILLIC CAPITAL LETTER ZHE
            0xB7 => '\u{0417}', // CYRILLIC CAPITAL LETTER ZE
            0xB8 => '\u{0418}', // CYRILLIC CAPITAL LETTER I
            0xB9 => '\u{0419}', // CYRILLIC CAPITAL LETTER SHORT I
            0xBA => '\u{041A}', // CYRILLIC CAPITAL LETTER KA
            0xBB => '\u{041B}', // CYRILLIC CAPITAL LETTER EL
            0xBC => '\u{041C}', // CYRILLIC CAPITAL LETTER EM
            0xBD => '\u{041D}', // CYRILLIC CAPITAL LETTER EN
            0xBE => '\u{041E}', // CYRILLIC CAPITAL LETTER O
            0xBF => '\u{041F}', // CYRILLIC CAPITAL LETTER PE
            0xC0 => '\u{0420}', // CYRILLIC CAPITAL LETTER ER
            0xC1 => '\u{0421}', // CYRILLIC CAPITAL LETTER ES
            0xC2 => '\u{0422}', // CYRILLIC CAPITAL LETTER TE
            0xC3 => '\u{0423}', // CYRILLIC CAPITAL LETTER U
            0xC4 => '\u{0424}', // CYRILLIC CAPITAL LETTER EF
            0xC5 => '\u{0425}', // CYRILLIC CAPITAL LETTER HA
            0xC6 => '\u{0426}', // CYRILLIC CAPITAL LETTER TSE
            0xC7 => '\u{0427}', // CYRILLIC CAPITAL LETTER CHE
            0xC8 => '\u{0428}', // CYRILLIC CAPITAL LETTER SHA
            0xC9 => '\u{0429}', // CYRILLIC CAPITAL LETTER SHCHA
            0xCA => '\u{042A}', // CYRILLIC CAPITAL LETTER HARD SIGN
            0xCB => '\u{042B}', // CYRILLIC CAPITAL LETTER YERU
            0xCC => '\u{042C}', // CYRILLIC CAPITAL LETTER SOFT SIGN
            0xCD => '\u{042D}', // CYRILLIC CAPITAL LETTER E
            0xCE => '\u{042E}', // CYRILLIC CAPITAL LETTER YU
            0xCF => '\u{042F}', // CYRILLIC CAPITAL LETTER YA
            0xD0 => '\u{0430}', // CYRILLIC SMALL LETTER A
            0xD1 => '\u{0431}', // CYRILLIC SMALL LETTER BE
            0xD2 => '\u{0432}', // CYRILLIC SMALL LETTER VE
            0xD3 => '\u{0433}', // CYRILLIC SMALL LETTER GHE
            0xD4 => '\u{0434}', // CYRILLIC SMALL LETTER DE
            0xD5 => '\u{0435}', // CYRILLIC SMALL LETTER IE
            0xD6 => '\u{0436}', // CYRILLIC SMALL LETTER ZHE
            0xD7 => '\u{0437}', // CYRILLIC SMALL LETTER ZE
            0xD8 => '\u{0438}', // CYRILLIC SMALL LETTER I
            0xD9 => '\u{0439}', // CYRILLIC SMALL LETTER SHORT I
            0xDA => '\u{043A}', // CYRILLIC SMALL LETTER KA
            0xDB => '\u{043B}', // CYRILLIC SMALL LETTER EL
            0xDC => '\u{043C}', // CYRILLIC SMALL LETTER EM
            0xDD => '\u{043D}', // CYRILLIC SMALL LETTER EN
            0xDE => '\u{043E}', // CYRILLIC SMALL LETTER O
            0xDF => '\u{043F}', // CYRILLIC SMALL LETTER PE
            0xE0 => '\u{0440}', // CYRILLIC SMALL LETTER ER
            0xE1 => '\u{0441}', // CYRILLIC SMALL LETTER ES
            0xE2 => '\u{0442}', // CYRILLIC SMALL LETTER TE
            0xE3 => '\u{0443}', // CYRILLIC SMALL LETTER U
            0xE4 => '\u{0444}', // CYRILLIC SMALL LETTER EF
            0xE5 => '\u{0445}', // CYRILLIC SMALL LETTER HA
            0xE6 => '\u{0446}', // CYRILLIC SMALL LETTER TSE
            0xE7 => '\u{0447}', // CYRILLIC SMALL LETTER CHE
            0xE8 => '\u{0448}', // CYRILLIC SMALL LETTER SHA
            0xE9 => '\u{0449}', // CYRILLIC SMALL LETTER SHCHA
            0xEA => '\u{044A}', // CYRILLIC SMALL LETTER HARD SIGN
            0xEB => '\u{044B}', // CYRILLIC SMALL LETTER YERU
            0xEC => '\u{044C}', // CYRILLIC SMALL LETTER SOFT SIGN
            0xED => '\u{044D}', // CYRILLIC SMALL LETTER E
            0xEE => '\u{044E}', // CYRILLIC SMALL LETTER YU
            0xEF => '\u{044F}', // CYRILLIC SMALL LETTER YA
            0xF0 => '\u{2116}', // NUMERO SIGN
            0xF1 => '\u{0451}', // CYRILLIC SMALL LETTER IO
            0xF2 => '\u{0452}', // CYRILLIC SMALL LETTER DJE
            0xF3 => '\u{0453}', // CYRILLIC SMALL LETTER GJE
            0xF4 => '\u{0454}', // CYRILLIC SMALL LETTER UKRAINIAN IE
            0xF5 => '\u{0455}', // CYRILLIC SMALL LETTER DZE
            0xF6 => '\u{0456}', // CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I
            0xF7 => '\u{0457}', // CYRILLIC SMALL LETTER YI
            0xF8 => '\u{0458}', // CYRILLIC SMALL LETTER JE
            0xF9 => '\u{0459}', // CYRILLIC SMALL LETTER LJE
            0xFA => '\u{045A}', // CYRILLIC SMALL LETTER NJE
            0xFB => '\u{045B}', // CYRILLIC SMALL LETTER TSHE
            0xFC => '\u{045C}', // CYRILLIC SMALL LETTER KJE
            0xFD => '\u{00A7}', // SECTION SIGN
            0xFE => '\u{045E}', // CYRILLIC SMALL LETTER SHORT U
            0xFF => '\u{045F}', // CYRILLIC SMALL LETTER DZHE
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

fn decode_iso8859_7(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        let c = match b {
            0xA0 => '\u{00A0}', // NO-BREAK SPACE
            0xA1 => '\u{2018}', // LEFT SINGLE QUOTATION MARK
            0xA2 => '\u{2019}', // RIGHT SINGLE QUOTATION MARK
            0xA3 => '\u{00A3}', // POUND SIGN
            0xA4 => '\u{20AC}', // EURO SIGN
            0xA5 => '\u{20AF}', // DRACHMA SIGN
            0xA6 => '\u{00A6}', // BROKEN BAR
            0xA7 => '\u{00A7}', // SECTION SIGN
            0xA8 => '\u{00A8}', // DIAERESIS
            0xA9 => '\u{00A9}', // COPYRIGHT SIGN
            0xAA => '\u{037A}', // GREEK YPOGEGRAMMENI
            0xAB => '\u{00AB}', // LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
            0xAC => '\u{00AC}', // NOT SIGN
            0xAD => '\u{00AD}', // SOFT HYPHEN
            0xAE => '\u{FFFD}', // UNDEFINED
            0xAF => '\u{2015}', // HORIZONTAL BAR
            0xB0 => '\u{00B0}', // DEGREE SIGN
            0xB1 => '\u{00B1}', // PLUS-MINUS SIGN
            0xB2 => '\u{00B2}', // SUPERSCRIPT TWO
            0xB3 => '\u{00B3}', // SUPERSCRIPT THREE
            0xB4 => '\u{0384}', // GREEK TONOS
            0xB5 => '\u{0385}', // GREEK DIALYTIKA TONOS
            0xB6 => '\u{0386}', // GREEK CAPITAL LETTER ALPHA WITH TONOS
            0xB7 => '\u{00B7}', // MIDDLE DOT
            0xB8 => '\u{0388}', // GREEK CAPITAL LETTER EPSILON WITH TONOS
            0xB9 => '\u{0389}', // GREEK CAPITAL LETTER ETA WITH TONOS
            0xBA => '\u{038A}', // GREEK CAPITAL LETTER IOTA WITH TONOS
            0xBB => '\u{00BB}', // RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
            0xBC => '\u{038C}', // GREEK CAPITAL LETTER OMICRON WITH TONOS
            0xBD => '\u{00BD}', // VULGAR FRACTION ONE HALF
            0xBE => '\u{038E}', // GREEK CAPITAL LETTER UPSILON WITH TONOS
            0xBF => '\u{038F}', // GREEK CAPITAL LETTER OMEGA WITH TONOS
            0xC0 => '\u{0390}', // GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS
            0xC1 => '\u{0391}', // GREEK CAPITAL LETTER ALPHA
            0xC2 => '\u{0392}', // GREEK CAPITAL LETTER BETA
            0xC3 => '\u{0393}', // GREEK CAPITAL LETTER GAMMA
            0xC4 => '\u{0394}', // GREEK CAPITAL LETTER DELTA
            0xC5 => '\u{0395}', // GREEK CAPITAL LETTER EPSILON
            0xC6 => '\u{0396}', // GREEK CAPITAL LETTER ZETA
            0xC7 => '\u{0397}', // GREEK CAPITAL LETTER ETA
            0xC8 => '\u{0398}', // GREEK CAPITAL LETTER THETA
            0xC9 => '\u{0399}', // GREEK CAPITAL LETTER IOTA
            0xCA => '\u{039A}', // GREEK CAPITAL LETTER KAPPA
            0xCB => '\u{039B}', // GREEK CAPITAL LETTER LAMDA
            0xCC => '\u{039C}', // GREEK CAPITAL LETTER MU
            0xCD => '\u{039D}', // GREEK CAPITAL LETTER NU
            0xCE => '\u{039E}', // GREEK CAPITAL LETTER XI
            0xCF => '\u{039F}', // GREEK CAPITAL LETTER OMICRON
            0xD0 => '\u{03A0}', // GREEK CAPITAL LETTER PI
            0xD1 => '\u{03A1}', // GREEK CAPITAL LETTER RHO
            0xD2 => '\u{FFFD}', // UNDEFINED
            0xD3 => '\u{03A3}', // GREEK CAPITAL LETTER SIGMA
            0xD4 => '\u{03A4}', // GREEK CAPITAL LETTER TAU
            0xD5 => '\u{03A5}', // GREEK CAPITAL LETTER UPSILON
            0xD6 => '\u{03A6}', // GREEK CAPITAL LETTER PHI
            0xD7 => '\u{03A7}', // GREEK CAPITAL LETTER CHI
            0xD8 => '\u{03A8}', // GREEK CAPITAL LETTER PSI
            0xD9 => '\u{03A9}', // GREEK CAPITAL LETTER OMEGA
            0xDA => '\u{03AA}', // GREEK CAPITAL LETTER IOTA WITH DIALYTIKA
            0xDB => '\u{03AB}', // GREEK CAPITAL LETTER UPSILON WITH DIALYTIKA
            0xDC => '\u{03AC}', // GREEK SMALL LETTER ALPHA WITH TONOS
            0xDD => '\u{03AD}', // GREEK SMALL LETTER EPSILON WITH TONOS
            0xDE => '\u{03AE}', // GREEK SMALL LETTER ETA WITH TONOS
            0xDF => '\u{03AF}', // GREEK SMALL LETTER IOTA WITH TONOS
            0xE0 => '\u{03B0}', // GREEK SMALL LETTER IOTA WITH DIALYTIKA AND TONOS
            0xE1 => '\u{03B1}', // GREEK SMALL LETTER ALPHA
            0xE2 => '\u{03B2}', // GREEK SMALL LETTER BETA
            0xE3 => '\u{03B3}', // GREEK SMALL LETTER GAMMA
            0xE4 => '\u{03B4}', // GREEK SMALL LETTER DELTA
            0xE5 => '\u{03B5}', // GREEK SMALL LETTER EPSILON
            0xE6 => '\u{03B6}', // GREEK SMALL LETTER ZETA
            0xE7 => '\u{03B7}', // GREEK SMALL LETTER ETA
            0xE8 => '\u{03B8}', // GREEK SMALL LETTER THETA
            0xE9 => '\u{03B9}', // GREEK SMALL LETTER IOTA
            0xEA => '\u{03BA}', // GREEK SMALL LETTER KAPPA
            0xEB => '\u{03BB}', // GREEK SMALL LETTER LAMDA
            0xEC => '\u{03BC}', // GREEK SMALL LETTER MU
            0xED => '\u{03BD}', // GREEK SMALL LETTER NU
            0xEE => '\u{03BE}', // GREEK SMALL LETTER XI
            0xEF => '\u{03BF}', // GREEK SMALL LETTER OMICRON
            0xF0 => '\u{03C0}', // GREEK SMALL LETTER PI
            0xF1 => '\u{03C1}', // GREEK SMALL LETTER RHO
            0xF2 => '\u{03C2}', // GREEK SMALL LETTER FINAL SIGMA
            0xF3 => '\u{03C3}', // GREEK SMALL LETTER SIGMA
            0xF4 => '\u{03C4}', // GREEK SMALL LETTER TAU
            0xF5 => '\u{03C5}', // GREEK SMALL LETTER UPSILON
            0xF6 => '\u{03C6}', // GREEK SMALL LETTER PHI
            0xF7 => '\u{03C7}', // GREEK SMALL LETTER CHI
            0xF8 => '\u{03C8}', // GREEK SMALL LETTER PSI
            0xF9 => '\u{03C9}', // GREEK SMALL LETTER OMEGA
            0xFA => '\u{03CA}', // GREEK SMALL LETTER IOTA WITH DIALYTIKA
            0xFB => '\u{03CB}', // GREEK SMALL LETTER UPSILON WITH DIALYTIKA
            0xFC => '\u{03CC}', // GREEK SMALL LETTER OMICRON WITH TONOS
            0xFD => '\u{03CD}', // GREEK SMALL LETTER UPSILON WITH TONOS
            0xFE => '\u{03CE}', // GREEK SMALL LETTER OMEGA WITH TONOS
            0xFF => '\u{FFFD}', // UNDEFINED
            _ => char::from(b),
        };
        result.push(c);
    }
    result
}

const WINDOWS_1253_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{0081}', // 0x81
    '\u{201A}', // 0x82
    '\u{0192}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{0088}', // 0x88
    '\u{2030}', // 0x89
    '\u{008A}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{008C}', // 0x8C
    '\u{008D}', // 0x8D
    '\u{008E}', // 0x8E
    '\u{008F}', // 0x8F
    '\u{0090}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{0098}', // 0x98
    '\u{2122}', // 0x99
    '\u{009A}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{009C}', // 0x9C
    '\u{009D}', // 0x9D
    '\u{009E}', // 0x9E
    '\u{009F}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{0385}', // 0xA1
    '\u{0386}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{00A5}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{FFFD}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{2015}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{0384}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{0388}', // 0xB8
    '\u{0389}', // 0xB9
    '\u{038A}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{038C}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{038E}', // 0xBE
    '\u{038F}', // 0xBF
    '\u{0390}', // 0xC0
    '\u{0391}', // 0xC1
    '\u{0392}', // 0xC2
    '\u{0393}', // 0xC3
    '\u{0394}', // 0xC4
    '\u{0395}', // 0xC5
    '\u{0396}', // 0xC6
    '\u{0397}', // 0xC7
    '\u{0398}', // 0xC8
    '\u{0399}', // 0xC9
    '\u{039A}', // 0xCA
    '\u{039B}', // 0xCB
    '\u{039C}', // 0xCC
    '\u{039D}', // 0xCD
    '\u{039E}', // 0xCE
    '\u{039F}', // 0xCF
    '\u{03A0}', // 0xD0
    '\u{03A1}', // 0xD1
    '\u{FFFD}', // 0xD2
    '\u{03A3}', // 0xD3
    '\u{03A4}', // 0xD4
    '\u{03A5}', // 0xD5
    '\u{03A6}', // 0xD6
    '\u{03A7}', // 0xD7
    '\u{03A8}', // 0xD8
    '\u{03A9}', // 0xD9
    '\u{03AA}', // 0xDA
    '\u{03AB}', // 0xDB
    '\u{03AC}', // 0xDC
    '\u{03AD}', // 0xDD
    '\u{03AE}', // 0xDE
    '\u{03AF}', // 0xDF
    '\u{03B0}', // 0xE0
    '\u{03B1}', // 0xE1
    '\u{03B2}', // 0xE2
    '\u{03B3}', // 0xE3
    '\u{03B4}', // 0xE4
    '\u{03B5}', // 0xE5
    '\u{03B6}', // 0xE6
    '\u{03B7}', // 0xE7
    '\u{03B8}', // 0xE8
    '\u{03B9}', // 0xE9
    '\u{03BA}', // 0xEA
    '\u{03BB}', // 0xEB
    '\u{03BC}', // 0xEC
    '\u{03BD}', // 0xED
    '\u{03BE}', // 0xEE
    '\u{03BF}', // 0xEF
    '\u{03C0}', // 0xF0
    '\u{03C1}', // 0xF1
    '\u{03C2}', // 0xF2
    '\u{03C3}', // 0xF3
    '\u{03C4}', // 0xF4
    '\u{03C5}', // 0xF5
    '\u{03C6}', // 0xF6
    '\u{03C7}', // 0xF7
    '\u{03C8}', // 0xF8
    '\u{03C9}', // 0xF9
    '\u{03CA}', // 0xFA
    '\u{03CB}', // 0xFB
    '\u{03CC}', // 0xFC
    '\u{03CD}', // 0xFD
    '\u{03CE}', // 0xFE
    '\u{FFFD}', // 0xFF
];

fn decode_windows1253(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1253_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const WINDOWS_1254_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{0081}', // 0x81
    '\u{201A}', // 0x82
    '\u{0192}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{02C6}', // 0x88
    '\u{2030}', // 0x89
    '\u{0160}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{0152}', // 0x8C
    '\u{008D}', // 0x8D
    '\u{008E}', // 0x8E
    '\u{008F}', // 0x8F
    '\u{0090}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{02DC}', // 0x98
    '\u{2122}', // 0x99
    '\u{0161}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{0153}', // 0x9C
    '\u{009D}', // 0x9D
    '\u{009E}', // 0x9E
    '\u{0178}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{00A1}', // 0xA1
    '\u{00A2}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{00A5}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{00AA}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{00AF}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00B8}', // 0xB8
    '\u{00B9}', // 0xB9
    '\u{00BA}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{00BC}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{00BE}', // 0xBE
    '\u{00BF}', // 0xBF
    '\u{00C0}', // 0xC0
    '\u{00C1}', // 0xC1
    '\u{00C2}', // 0xC2
    '\u{00C3}', // 0xC3
    '\u{00C4}', // 0xC4
    '\u{00C5}', // 0xC5
    '\u{00C6}', // 0xC6
    '\u{00C7}', // 0xC7
    '\u{00C8}', // 0xC8
    '\u{00C9}', // 0xC9
    '\u{00CA}', // 0xCA
    '\u{00CB}', // 0xCB
    '\u{00CC}', // 0xCC
    '\u{00CD}', // 0xCD
    '\u{00CE}', // 0xCE
    '\u{00CF}', // 0xCF
    '\u{011E}', // 0xD0
    '\u{00D1}', // 0xD1
    '\u{00D2}', // 0xD2
    '\u{00D3}', // 0xD3
    '\u{00D4}', // 0xD4
    '\u{00D5}', // 0xD5
    '\u{00D6}', // 0xD6
    '\u{00D7}', // 0xD7
    '\u{00D8}', // 0xD8
    '\u{00D9}', // 0xD9
    '\u{00DA}', // 0xDA
    '\u{00DB}', // 0xDB
    '\u{00DC}', // 0xDC
    '\u{0130}', // 0xDD
    '\u{015E}', // 0xDE
    '\u{00DF}', // 0xDF
    '\u{00E0}', // 0xE0
    '\u{00E1}', // 0xE1
    '\u{00E2}', // 0xE2
    '\u{00E3}', // 0xE3
    '\u{00E4}', // 0xE4
    '\u{00E5}', // 0xE5
    '\u{00E6}', // 0xE6
    '\u{00E7}', // 0xE7
    '\u{00E8}', // 0xE8
    '\u{00E9}', // 0xE9
    '\u{00EA}', // 0xEA
    '\u{00EB}', // 0xEB
    '\u{00EC}', // 0xEC
    '\u{00ED}', // 0xED
    '\u{00EE}', // 0xEE
    '\u{00EF}', // 0xEF
    '\u{011F}', // 0xF0
    '\u{00F1}', // 0xF1
    '\u{00F2}', // 0xF2
    '\u{00F3}', // 0xF3
    '\u{00F4}', // 0xF4
    '\u{00F5}', // 0xF5
    '\u{00F6}', // 0xF6
    '\u{00F7}', // 0xF7
    '\u{00F8}', // 0xF8
    '\u{00F9}', // 0xF9
    '\u{00FA}', // 0xFA
    '\u{00FB}', // 0xFB
    '\u{00FC}', // 0xFC
    '\u{0131}', // 0xFD
    '\u{015F}', // 0xFE
    '\u{00FF}', // 0xFF
];

fn decode_windows1254(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1254_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const WINDOWS_1255_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{0081}', // 0x81
    '\u{201A}', // 0x82
    '\u{0192}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{02C6}', // 0x88
    '\u{2030}', // 0x89
    '\u{008A}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{008C}', // 0x8C
    '\u{008D}', // 0x8D
    '\u{008E}', // 0x8E
    '\u{008F}', // 0x8F
    '\u{0090}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{02DC}', // 0x98
    '\u{2122}', // 0x99
    '\u{009A}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{009C}', // 0x9C
    '\u{009D}', // 0x9D
    '\u{009E}', // 0x9E
    '\u{009F}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{FFFD}', // 0xA1
    '\u{00A2}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{00A5}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{00D7}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{00AF}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00B8}', // 0xB8
    '\u{00B9}', // 0xB9
    '\u{00F7}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{00BC}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{00BE}', // 0xBE
    '\u{00BF}', // 0xBF
    '\u{05B0}', // 0xC0
    '\u{05B1}', // 0xC1
    '\u{05B2}', // 0xC2
    '\u{05B3}', // 0xC3
    '\u{05B4}', // 0xC4
    '\u{05B5}', // 0xC5
    '\u{05B6}', // 0xC6
    '\u{05B7}', // 0xC7
    '\u{05B8}', // 0xC8
    '\u{05B9}', // 0xC9
    '\u{FFFD}', // 0xCA
    '\u{05BB}', // 0xCB
    '\u{05BC}', // 0xCC
    '\u{05BD}', // 0xCD
    '\u{05BE}', // 0xCE
    '\u{05BF}', // 0xCF
    '\u{05C0}', // 0xD0
    '\u{05C1}', // 0xD1
    '\u{05C2}', // 0xD2
    '\u{05C3}', // 0xD3
    '\u{05C4}', // 0xD4
    '\u{FFFD}', // 0xD5
    '\u{FFFD}', // 0xD6
    '\u{FFFD}', // 0xD7
    '\u{FFFD}', // 0xD8
    '\u{FFFD}', // 0xD9
    '\u{FFFD}', // 0xDA
    '\u{FFFD}', // 0xDB
    '\u{FFFD}', // 0xDC
    '\u{FFFD}', // 0xDD
    '\u{FFFD}', // 0xDE
    '\u{FFFD}', // 0xDF
    '\u{05D0}', // 0xE0
    '\u{05D1}', // 0xE1
    '\u{05D2}', // 0xE2
    '\u{05D3}', // 0xE3
    '\u{05D4}', // 0xE4
    '\u{05D5}', // 0xE5
    '\u{05D6}', // 0xE6
    '\u{05D7}', // 0xE7
    '\u{05D8}', // 0xE8
    '\u{05D9}', // 0xE9
    '\u{05DA}', // 0xEA
    '\u{05DB}', // 0xEB
    '\u{05DC}', // 0xEC
    '\u{05DD}', // 0xED
    '\u{05DE}', // 0xEE
    '\u{05DF}', // 0xEF
    '\u{05E0}', // 0xF0
    '\u{05E1}', // 0xF1
    '\u{05E2}', // 0xF2
    '\u{05E3}', // 0xF3
    '\u{05E4}', // 0xF4
    '\u{05E5}', // 0xF5
    '\u{05E6}', // 0xF6
    '\u{05E7}', // 0xF7
    '\u{05E8}', // 0xF8
    '\u{05E9}', // 0xF9
    '\u{05EA}', // 0xFA
    '\u{FFFD}', // 0xFB
    '\u{FFFD}', // 0xFC
    '\u{200E}', // 0xFD
    '\u{200F}', // 0xFE
    '\u{FFFD}', // 0xFF
];

fn decode_windows1255(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1255_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const WINDOWS_1256_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{067E}', // 0x81
    '\u{201A}', // 0x82
    '\u{0192}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{02C6}', // 0x88
    '\u{2030}', // 0x89
    '\u{0679}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{0152}', // 0x8C
    '\u{0686}', // 0x8D
    '\u{0698}', // 0x8E
    '\u{0688}', // 0x8F
    '\u{06AF}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{06A9}', // 0x98
    '\u{2122}', // 0x99
    '\u{0691}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{0153}', // 0x9C
    '\u{200C}', // 0x9D
    '\u{200D}', // 0x9E
    '\u{06BA}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{060C}', // 0xA1
    '\u{00A2}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{00A5}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{06BE}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{00AF}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00B8}', // 0xB8
    '\u{00B9}', // 0xB9
    '\u{061B}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{00BC}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{00BE}', // 0xBE
    '\u{061F}', // 0xBF
    '\u{06C1}', // 0xC0
    '\u{0621}', // 0xC1
    '\u{0622}', // 0xC2
    '\u{0623}', // 0xC3
    '\u{0624}', // 0xC4
    '\u{0625}', // 0xC5
    '\u{0626}', // 0xC6
    '\u{0627}', // 0xC7
    '\u{0628}', // 0xC8
    '\u{0629}', // 0xC9
    '\u{062A}', // 0xCA
    '\u{062B}', // 0xCB
    '\u{062C}', // 0xCC
    '\u{062D}', // 0xCD
    '\u{062E}', // 0xCE
    '\u{062F}', // 0xCF
    '\u{0630}', // 0xD0
    '\u{0631}', // 0xD1
    '\u{0632}', // 0xD2
    '\u{0633}', // 0xD3
    '\u{0634}', // 0xD4
    '\u{0635}', // 0xD5
    '\u{0636}', // 0xD6
    '\u{00D7}', // 0xD7
    '\u{0637}', // 0xD8
    '\u{0638}', // 0xD9
    '\u{0639}', // 0xDA
    '\u{063A}', // 0xDB
    '\u{0640}', // 0xDC
    '\u{0641}', // 0xDD
    '\u{0642}', // 0xDE
    '\u{0643}', // 0xDF
    '\u{00E0}', // 0xE0
    '\u{0644}', // 0xE1
    '\u{00E2}', // 0xE2
    '\u{0645}', // 0xE3
    '\u{0646}', // 0xE4
    '\u{0647}', // 0xE5
    '\u{0648}', // 0xE6
    '\u{00E7}', // 0xE7
    '\u{00E8}', // 0xE8
    '\u{00E9}', // 0xE9
    '\u{00EA}', // 0xEA
    '\u{00EB}', // 0xEB
    '\u{0649}', // 0xEC
    '\u{064A}', // 0xED
    '\u{00EE}', // 0xEE
    '\u{00EF}', // 0xEF
    '\u{064B}', // 0xF0
    '\u{064C}', // 0xF1
    '\u{064D}', // 0xF2
    '\u{064E}', // 0xF3
    '\u{00F4}', // 0xF4
    '\u{064F}', // 0xF5
    '\u{0650}', // 0xF6
    '\u{00F7}', // 0xF7
    '\u{0651}', // 0xF8
    '\u{00F9}', // 0xF9
    '\u{0652}', // 0xFA
    '\u{00FB}', // 0xFB
    '\u{00FC}', // 0xFC
    '\u{200E}', // 0xFD
    '\u{200F}', // 0xFE
    '\u{06D2}', // 0xFF
];

fn decode_windows1256(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1256_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const WINDOWS_1257_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{0081}', // 0x81
    '\u{201A}', // 0x82
    '\u{0083}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{0088}', // 0x88
    '\u{2030}', // 0x89
    '\u{008A}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{008C}', // 0x8C
    '\u{00A8}', // 0x8D
    '\u{02C7}', // 0x8E
    '\u{00B8}', // 0x8F
    '\u{0090}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{0098}', // 0x98
    '\u{2122}', // 0x99
    '\u{009A}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{009C}', // 0x9C
    '\u{00AF}', // 0x9D
    '\u{02DB}', // 0x9E
    '\u{009F}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{FFFD}', // 0xA1
    '\u{00A2}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{FFFD}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00D8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{0156}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{00C6}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00F8}', // 0xB8
    '\u{00B9}', // 0xB9
    '\u{0157}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{00BC}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{00BE}', // 0xBE
    '\u{00E6}', // 0xBF
    '\u{0104}', // 0xC0
    '\u{012E}', // 0xC1
    '\u{0100}', // 0xC2
    '\u{0106}', // 0xC3
    '\u{00C4}', // 0xC4
    '\u{00C5}', // 0xC5
    '\u{0118}', // 0xC6
    '\u{0112}', // 0xC7
    '\u{010C}', // 0xC8
    '\u{00C9}', // 0xC9
    '\u{0179}', // 0xCA
    '\u{0116}', // 0xCB
    '\u{0122}', // 0xCC
    '\u{0136}', // 0xCD
    '\u{012A}', // 0xCE
    '\u{013B}', // 0xCF
    '\u{0160}', // 0xD0
    '\u{0143}', // 0xD1
    '\u{0145}', // 0xD2
    '\u{00D3}', // 0xD3
    '\u{014C}', // 0xD4
    '\u{00D5}', // 0xD5
    '\u{00D6}', // 0xD6
    '\u{00D7}', // 0xD7
    '\u{0172}', // 0xD8
    '\u{0141}', // 0xD9
    '\u{015A}', // 0xDA
    '\u{016A}', // 0xDB
    '\u{00DC}', // 0xDC
    '\u{017B}', // 0xDD
    '\u{017D}', // 0xDE
    '\u{00DF}', // 0xDF
    '\u{0105}', // 0xE0
    '\u{012F}', // 0xE1
    '\u{0101}', // 0xE2
    '\u{0107}', // 0xE3
    '\u{00E4}', // 0xE4
    '\u{00E5}', // 0xE5
    '\u{0119}', // 0xE6
    '\u{0113}', // 0xE7
    '\u{010D}', // 0xE8
    '\u{00E9}', // 0xE9
    '\u{017A}', // 0xEA
    '\u{0117}', // 0xEB
    '\u{0123}', // 0xEC
    '\u{0137}', // 0xED
    '\u{012B}', // 0xEE
    '\u{013C}', // 0xEF
    '\u{0161}', // 0xF0
    '\u{0144}', // 0xF1
    '\u{0146}', // 0xF2
    '\u{00F3}', // 0xF3
    '\u{014D}', // 0xF4
    '\u{00F5}', // 0xF5
    '\u{00F6}', // 0xF6
    '\u{00F7}', // 0xF7
    '\u{0173}', // 0xF8
    '\u{0142}', // 0xF9
    '\u{015B}', // 0xFA
    '\u{016B}', // 0xFB
    '\u{00FC}', // 0xFC
    '\u{017C}', // 0xFD
    '\u{017E}', // 0xFE
    '\u{02D9}', // 0xFF
];

fn decode_windows1257(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1257_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

// Windows-1258 (Vietnamese) 0x80-0xFF -> Unicode.
// Source: WHATWG Encoding Standard index-windows-1258 (all 128 positions defined; no gaps).
const WINDOWS_1258_MAP: [char; 128] = [
    '\u{20AC}', // 0x80
    '\u{0081}', // 0x81
    '\u{201A}', // 0x82
    '\u{0192}', // 0x83
    '\u{201E}', // 0x84
    '\u{2026}', // 0x85
    '\u{2020}', // 0x86
    '\u{2021}', // 0x87
    '\u{02C6}', // 0x88
    '\u{2030}', // 0x89
    '\u{008A}', // 0x8A
    '\u{2039}', // 0x8B
    '\u{0152}', // 0x8C
    '\u{008D}', // 0x8D
    '\u{008E}', // 0x8E
    '\u{008F}', // 0x8F
    '\u{0090}', // 0x90
    '\u{2018}', // 0x91
    '\u{2019}', // 0x92
    '\u{201C}', // 0x93
    '\u{201D}', // 0x94
    '\u{2022}', // 0x95
    '\u{2013}', // 0x96
    '\u{2014}', // 0x97
    '\u{02DC}', // 0x98
    '\u{2122}', // 0x99
    '\u{009A}', // 0x9A
    '\u{203A}', // 0x9B
    '\u{0153}', // 0x9C
    '\u{009D}', // 0x9D
    '\u{009E}', // 0x9E
    '\u{0178}', // 0x9F
    '\u{00A0}', // 0xA0
    '\u{00A1}', // 0xA1
    '\u{00A2}', // 0xA2
    '\u{00A3}', // 0xA3
    '\u{00A4}', // 0xA4
    '\u{00A5}', // 0xA5
    '\u{00A6}', // 0xA6
    '\u{00A7}', // 0xA7
    '\u{00A8}', // 0xA8
    '\u{00A9}', // 0xA9
    '\u{00AA}', // 0xAA
    '\u{00AB}', // 0xAB
    '\u{00AC}', // 0xAC
    '\u{00AD}', // 0xAD
    '\u{00AE}', // 0xAE
    '\u{00AF}', // 0xAF
    '\u{00B0}', // 0xB0
    '\u{00B1}', // 0xB1
    '\u{00B2}', // 0xB2
    '\u{00B3}', // 0xB3
    '\u{00B4}', // 0xB4
    '\u{00B5}', // 0xB5
    '\u{00B6}', // 0xB6
    '\u{00B7}', // 0xB7
    '\u{00B8}', // 0xB8
    '\u{00B9}', // 0xB9
    '\u{00BA}', // 0xBA
    '\u{00BB}', // 0xBB
    '\u{00BC}', // 0xBC
    '\u{00BD}', // 0xBD
    '\u{00BE}', // 0xBE
    '\u{00BF}', // 0xBF
    '\u{00C0}', // 0xC0
    '\u{00C1}', // 0xC1
    '\u{00C2}', // 0xC2
    '\u{0102}', // 0xC3  LATIN CAPITAL LETTER A WITH BREVE
    '\u{00C4}', // 0xC4
    '\u{00C5}', // 0xC5
    '\u{00C6}', // 0xC6
    '\u{00C7}', // 0xC7
    '\u{00C8}', // 0xC8
    '\u{00C9}', // 0xC9
    '\u{00CA}', // 0xCA
    '\u{00CB}', // 0xCB
    '\u{0300}', // 0xCC  COMBINING GRAVE ACCENT
    '\u{00CD}', // 0xCD
    '\u{00CE}', // 0xCE
    '\u{00CF}', // 0xCF
    '\u{0110}', // 0xD0  LATIN CAPITAL LETTER D WITH STROKE
    '\u{00D1}', // 0xD1
    '\u{0309}', // 0xD2  COMBINING HOOK ABOVE
    '\u{00D3}', // 0xD3
    '\u{00D4}', // 0xD4
    '\u{01A0}', // 0xD5  LATIN CAPITAL LETTER O WITH HORN
    '\u{00D6}', // 0xD6
    '\u{00D7}', // 0xD7
    '\u{00D8}', // 0xD8
    '\u{00D9}', // 0xD9
    '\u{00DA}', // 0xDA
    '\u{00DB}', // 0xDB
    '\u{00DC}', // 0xDC
    '\u{01AF}', // 0xDD  LATIN CAPITAL LETTER U WITH HORN
    '\u{0303}', // 0xDE  COMBINING TILDE
    '\u{00DF}', // 0xDF
    '\u{00E0}', // 0xE0
    '\u{00E1}', // 0xE1
    '\u{00E2}', // 0xE2
    '\u{0103}', // 0xE3  LATIN SMALL LETTER A WITH BREVE
    '\u{00E4}', // 0xE4
    '\u{00E5}', // 0xE5
    '\u{00E6}', // 0xE6
    '\u{00E7}', // 0xE7
    '\u{00E8}', // 0xE8
    '\u{00E9}', // 0xE9
    '\u{00EA}', // 0xEA
    '\u{00EB}', // 0xEB
    '\u{0301}', // 0xEC  COMBINING ACUTE ACCENT
    '\u{00ED}', // 0xED
    '\u{00EE}', // 0xEE
    '\u{00EF}', // 0xEF
    '\u{0111}', // 0xF0  LATIN SMALL LETTER D WITH STROKE
    '\u{00F1}', // 0xF1
    '\u{0323}', // 0xF2  COMBINING DOT BELOW
    '\u{00F3}', // 0xF3
    '\u{00F4}', // 0xF4
    '\u{01A1}', // 0xF5  LATIN SMALL LETTER O WITH HORN
    '\u{00F6}', // 0xF6
    '\u{00F7}', // 0xF7
    '\u{00F8}', // 0xF8
    '\u{00F9}', // 0xF9
    '\u{00FA}', // 0xFA
    '\u{00FB}', // 0xFB
    '\u{00FC}', // 0xFC
    '\u{01B0}', // 0xFD  LATIN SMALL LETTER U WITH HORN
    '\u{20AB}', // 0xFE  DONG SIGN
    '\u{00FF}', // 0xFF
];

fn decode_windows1258(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(WINDOWS_1258_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const KOI8_R_MAP: [char; 128] = [
    '\u{2500}', // 0x80
    '\u{2502}', // 0x81
    '\u{250C}', // 0x82
    '\u{2510}', // 0x83
    '\u{2514}', // 0x84
    '\u{2518}', // 0x85
    '\u{251C}', // 0x86
    '\u{2524}', // 0x87
    '\u{252C}', // 0x88
    '\u{2534}', // 0x89
    '\u{253C}', // 0x8A
    '\u{2580}', // 0x8B
    '\u{2584}', // 0x8C
    '\u{2588}', // 0x8D
    '\u{258C}', // 0x8E
    '\u{2590}', // 0x8F
    '\u{2591}', // 0x90
    '\u{2592}', // 0x91
    '\u{2593}', // 0x92
    '\u{2320}', // 0x93
    '\u{25A0}', // 0x94
    '\u{2219}', // 0x95
    '\u{221A}', // 0x96
    '\u{2248}', // 0x97
    '\u{2264}', // 0x98
    '\u{2265}', // 0x99
    '\u{00A0}', // 0x9A
    '\u{2321}', // 0x9B
    '\u{00B0}', // 0x9C
    '\u{00B2}', // 0x9D
    '\u{00B7}', // 0x9E
    '\u{00F7}', // 0x9F
    '\u{2550}', // 0xA0
    '\u{2551}', // 0xA1
    '\u{2552}', // 0xA2
    '\u{0451}', // 0xA3  CYRILLIC SMALL LETTER IO
    '\u{2553}', // 0xA4
    '\u{2554}', // 0xA5
    '\u{2555}', // 0xA6
    '\u{2556}', // 0xA7
    '\u{2557}', // 0xA8
    '\u{2558}', // 0xA9
    '\u{2559}', // 0xAA
    '\u{255A}', // 0xAB
    '\u{255B}', // 0xAC
    '\u{255C}', // 0xAD
    '\u{255D}', // 0xAE
    '\u{255E}', // 0xAF
    '\u{255F}', // 0xB0
    '\u{2560}', // 0xB1
    '\u{2561}', // 0xB2
    '\u{0401}', // 0xB3  CYRILLIC CAPITAL LETTER IO
    '\u{2562}', // 0xB4
    '\u{2563}', // 0xB5
    '\u{2564}', // 0xB6
    '\u{2565}', // 0xB7
    '\u{2566}', // 0xB8
    '\u{2567}', // 0xB9
    '\u{2568}', // 0xBA
    '\u{2569}', // 0xBB
    '\u{256A}', // 0xBC
    '\u{256B}', // 0xBD
    '\u{256C}', // 0xBE
    '\u{00A9}', // 0xBF  COPYRIGHT SIGN
    '\u{044E}', // 0xC0  CYRILLIC SMALL LETTER YU
    '\u{0430}', // 0xC1  CYRILLIC SMALL LETTER A
    '\u{0431}', // 0xC2  CYRILLIC SMALL LETTER BE
    '\u{0446}', // 0xC3  CYRILLIC SMALL LETTER TSE
    '\u{0434}', // 0xC4  CYRILLIC SMALL LETTER DE
    '\u{0435}', // 0xC5  CYRILLIC SMALL LETTER IE
    '\u{0444}', // 0xC6  CYRILLIC SMALL LETTER EF
    '\u{0433}', // 0xC7  CYRILLIC SMALL LETTER GHE
    '\u{0445}', // 0xC8  CYRILLIC SMALL LETTER HA
    '\u{0438}', // 0xC9  CYRILLIC SMALL LETTER I
    '\u{0439}', // 0xCA  CYRILLIC SMALL LETTER SHORT I
    '\u{043A}', // 0xCB  CYRILLIC SMALL LETTER KA
    '\u{043B}', // 0xCC  CYRILLIC SMALL LETTER EL
    '\u{043C}', // 0xCD  CYRILLIC SMALL LETTER EM
    '\u{043D}', // 0xCE  CYRILLIC SMALL LETTER EN
    '\u{043E}', // 0xCF  CYRILLIC SMALL LETTER O
    '\u{043F}', // 0xD0  CYRILLIC SMALL LETTER PE
    '\u{044F}', // 0xD1  CYRILLIC SMALL LETTER YA
    '\u{0440}', // 0xD2  CYRILLIC SMALL LETTER ER
    '\u{0441}', // 0xD3  CYRILLIC SMALL LETTER ES
    '\u{0442}', // 0xD4  CYRILLIC SMALL LETTER TE
    '\u{0443}', // 0xD5  CYRILLIC SMALL LETTER U
    '\u{0436}', // 0xD6  CYRILLIC SMALL LETTER ZHE
    '\u{0432}', // 0xD7  CYRILLIC SMALL LETTER VE
    '\u{044C}', // 0xD8  CYRILLIC SMALL LETTER SOFT SIGN
    '\u{044B}', // 0xD9  CYRILLIC SMALL LETTER YERU
    '\u{0437}', // 0xDA  CYRILLIC SMALL LETTER ZE
    '\u{0448}', // 0xDB  CYRILLIC SMALL LETTER SHA
    '\u{044D}', // 0xDC  CYRILLIC SMALL LETTER E
    '\u{0449}', // 0xDD  CYRILLIC SMALL LETTER SHCHA
    '\u{0447}', // 0xDE  CYRILLIC SMALL LETTER CHE
    '\u{044A}', // 0xDF  CYRILLIC SMALL LETTER HARD SIGN
    '\u{042E}', // 0xE0  CYRILLIC CAPITAL LETTER YU
    '\u{0410}', // 0xE1  CYRILLIC CAPITAL LETTER A
    '\u{0411}', // 0xE2  CYRILLIC CAPITAL LETTER BE
    '\u{0426}', // 0xE3  CYRILLIC CAPITAL LETTER TSE
    '\u{0414}', // 0xE4  CYRILLIC CAPITAL LETTER DE
    '\u{0415}', // 0xE5  CYRILLIC CAPITAL LETTER IE
    '\u{0424}', // 0xE6  CYRILLIC CAPITAL LETTER EF
    '\u{0413}', // 0xE7  CYRILLIC CAPITAL LETTER GHE
    '\u{0425}', // 0xE8  CYRILLIC CAPITAL LETTER HA
    '\u{0418}', // 0xE9  CYRILLIC CAPITAL LETTER I
    '\u{0419}', // 0xEA  CYRILLIC CAPITAL LETTER SHORT I
    '\u{041A}', // 0xEB  CYRILLIC CAPITAL LETTER KA
    '\u{041B}', // 0xEC  CYRILLIC CAPITAL LETTER EL
    '\u{041C}', // 0xED  CYRILLIC CAPITAL LETTER EM
    '\u{041D}', // 0xEE  CYRILLIC CAPITAL LETTER EN
    '\u{041E}', // 0xEF  CYRILLIC CAPITAL LETTER O
    '\u{041F}', // 0xF0  CYRILLIC CAPITAL LETTER PE
    '\u{042F}', // 0xF1  CYRILLIC CAPITAL LETTER YA
    '\u{0420}', // 0xF2  CYRILLIC CAPITAL LETTER ER
    '\u{0421}', // 0xF3  CYRILLIC CAPITAL LETTER ES
    '\u{0422}', // 0xF4  CYRILLIC CAPITAL LETTER TE
    '\u{0423}', // 0xF5  CYRILLIC CAPITAL LETTER U
    '\u{0416}', // 0xF6  CYRILLIC CAPITAL LETTER ZHE
    '\u{0412}', // 0xF7  CYRILLIC CAPITAL LETTER VE
    '\u{042C}', // 0xF8  CYRILLIC CAPITAL LETTER SOFT SIGN
    '\u{042B}', // 0xF9  CYRILLIC CAPITAL LETTER YERU
    '\u{0417}', // 0xFA  CYRILLIC CAPITAL LETTER ZE
    '\u{0428}', // 0xFB  CYRILLIC CAPITAL LETTER SHA
    '\u{042D}', // 0xFC  CYRILLIC CAPITAL LETTER E
    '\u{0429}', // 0xFD  CYRILLIC CAPITAL LETTER SHCHA
    '\u{0427}', // 0xFE  CYRILLIC CAPITAL LETTER CHE
    '\u{042A}', // 0xFF  CYRILLIC CAPITAL LETTER HARD SIGN
];

fn decode_koi8r(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(KOI8_R_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const KOI8_U_MAP: [char; 128] = [
    '\u{2500}', // 0x80
    '\u{2502}', // 0x81
    '\u{250C}', // 0x82
    '\u{2510}', // 0x83
    '\u{2514}', // 0x84
    '\u{2518}', // 0x85
    '\u{251C}', // 0x86
    '\u{2524}', // 0x87
    '\u{252C}', // 0x88
    '\u{2534}', // 0x89
    '\u{253C}', // 0x8A
    '\u{2580}', // 0x8B
    '\u{2584}', // 0x8C
    '\u{2588}', // 0x8D
    '\u{258C}', // 0x8E
    '\u{2590}', // 0x8F
    '\u{2591}', // 0x90
    '\u{2592}', // 0x91
    '\u{2593}', // 0x92
    '\u{2320}', // 0x93
    '\u{25A0}', // 0x94
    '\u{2219}', // 0x95
    '\u{221A}', // 0x96
    '\u{2248}', // 0x97
    '\u{2264}', // 0x98
    '\u{2265}', // 0x99
    '\u{00A0}', // 0x9A
    '\u{2321}', // 0x9B
    '\u{00B0}', // 0x9C
    '\u{00B2}', // 0x9D
    '\u{00B7}', // 0x9E
    '\u{00F7}', // 0x9F
    '\u{2550}', // 0xA0
    '\u{2551}', // 0xA1
    '\u{2552}', // 0xA2
    '\u{0451}', // 0xA3  CYRILLIC SMALL LETTER IO
    '\u{0454}', // 0xA4  CYRILLIC SMALL LETTER UKRAINIAN IE (KOI8-U override)
    '\u{2554}', // 0xA5
    '\u{0456}', // 0xA6  CYRILLIC SMALL LETTER BYELORUSSIAN-UKRAINIAN I (KOI8-U override)
    '\u{0457}', // 0xA7  CYRILLIC SMALL LETTER YI (KOI8-U override)
    '\u{2557}', // 0xA8
    '\u{2558}', // 0xA9
    '\u{2559}', // 0xAA
    '\u{255A}', // 0xAB
    '\u{255B}', // 0xAC
    '\u{0491}', // 0xAD  CYRILLIC SMALL LETTER GHE WITH UPTURN (KOI8-U override)
    '\u{255D}', // 0xAE
    '\u{255E}', // 0xAF
    '\u{255F}', // 0xB0
    '\u{2560}', // 0xB1
    '\u{2561}', // 0xB2
    '\u{0401}', // 0xB3  CYRILLIC CAPITAL LETTER IO
    '\u{0404}', // 0xB4  CYRILLIC CAPITAL LETTER UKRAINIAN IE (KOI8-U override)
    '\u{2563}', // 0xB5
    '\u{0406}', // 0xB6  CYRILLIC CAPITAL LETTER BYELORUSSIAN-UKRAINIAN I (KOI8-U override)
    '\u{0407}', // 0xB7  CYRILLIC CAPITAL LETTER YI (KOI8-U override)
    '\u{2566}', // 0xB8
    '\u{2567}', // 0xB9
    '\u{2568}', // 0xBA
    '\u{2569}', // 0xBB
    '\u{256A}', // 0xBC
    '\u{0490}', // 0xBD  CYRILLIC CAPITAL LETTER GHE WITH UPTURN (KOI8-U override)
    '\u{256C}', // 0xBE
    '\u{00A9}', // 0xBF  COPYRIGHT SIGN
    '\u{044E}', // 0xC0  CYRILLIC SMALL LETTER YU
    '\u{0430}', // 0xC1  CYRILLIC SMALL LETTER A
    '\u{0431}', // 0xC2  CYRILLIC SMALL LETTER BE
    '\u{0446}', // 0xC3  CYRILLIC SMALL LETTER TSE
    '\u{0434}', // 0xC4  CYRILLIC SMALL LETTER DE
    '\u{0435}', // 0xC5  CYRILLIC SMALL LETTER IE
    '\u{0444}', // 0xC6  CYRILLIC SMALL LETTER EF
    '\u{0433}', // 0xC7  CYRILLIC SMALL LETTER GHE
    '\u{0445}', // 0xC8  CYRILLIC SMALL LETTER HA
    '\u{0438}', // 0xC9  CYRILLIC SMALL LETTER I
    '\u{0439}', // 0xCA  CYRILLIC SMALL LETTER SHORT I
    '\u{043A}', // 0xCB  CYRILLIC SMALL LETTER KA
    '\u{043B}', // 0xCC  CYRILLIC SMALL LETTER EL
    '\u{043C}', // 0xCD  CYRILLIC SMALL LETTER EM
    '\u{043D}', // 0xCE  CYRILLIC SMALL LETTER EN
    '\u{043E}', // 0xCF  CYRILLIC SMALL LETTER O
    '\u{043F}', // 0xD0  CYRILLIC SMALL LETTER PE
    '\u{044F}', // 0xD1  CYRILLIC SMALL LETTER YA
    '\u{0440}', // 0xD2  CYRILLIC SMALL LETTER ER
    '\u{0441}', // 0xD3  CYRILLIC SMALL LETTER ES
    '\u{0442}', // 0xD4  CYRILLIC SMALL LETTER TE
    '\u{0443}', // 0xD5  CYRILLIC SMALL LETTER U
    '\u{0436}', // 0xD6  CYRILLIC SMALL LETTER ZHE
    '\u{0432}', // 0xD7  CYRILLIC SMALL LETTER VE
    '\u{044C}', // 0xD8  CYRILLIC SMALL LETTER SOFT SIGN
    '\u{044B}', // 0xD9  CYRILLIC SMALL LETTER YERU
    '\u{0437}', // 0xDA  CYRILLIC SMALL LETTER ZE
    '\u{0448}', // 0xDB  CYRILLIC SMALL LETTER SHA
    '\u{044D}', // 0xDC  CYRILLIC SMALL LETTER E
    '\u{0449}', // 0xDD  CYRILLIC SMALL LETTER SHCHA
    '\u{0447}', // 0xDE  CYRILLIC SMALL LETTER CHE
    '\u{044A}', // 0xDF  CYRILLIC SMALL LETTER HARD SIGN
    '\u{042E}', // 0xE0  CYRILLIC CAPITAL LETTER YU
    '\u{0410}', // 0xE1  CYRILLIC CAPITAL LETTER A
    '\u{0411}', // 0xE2  CYRILLIC CAPITAL LETTER BE
    '\u{0426}', // 0xE3  CYRILLIC CAPITAL LETTER TSE
    '\u{0414}', // 0xE4  CYRILLIC CAPITAL LETTER DE
    '\u{0415}', // 0xE5  CYRILLIC CAPITAL LETTER IE
    '\u{0424}', // 0xE6  CYRILLIC CAPITAL LETTER EF
    '\u{0413}', // 0xE7  CYRILLIC CAPITAL LETTER GHE
    '\u{0425}', // 0xE8  CYRILLIC CAPITAL LETTER HA
    '\u{0418}', // 0xE9  CYRILLIC CAPITAL LETTER I
    '\u{0419}', // 0xEA  CYRILLIC CAPITAL LETTER SHORT I
    '\u{041A}', // 0xEB  CYRILLIC CAPITAL LETTER KA
    '\u{041B}', // 0xEC  CYRILLIC CAPITAL LETTER EL
    '\u{041C}', // 0xED  CYRILLIC CAPITAL LETTER EM
    '\u{041D}', // 0xEE  CYRILLIC CAPITAL LETTER EN
    '\u{041E}', // 0xEF  CYRILLIC CAPITAL LETTER O
    '\u{041F}', // 0xF0  CYRILLIC CAPITAL LETTER PE
    '\u{042F}', // 0xF1  CYRILLIC CAPITAL LETTER YA
    '\u{0420}', // 0xF2  CYRILLIC CAPITAL LETTER ER
    '\u{0421}', // 0xF3  CYRILLIC CAPITAL LETTER ES
    '\u{0422}', // 0xF4  CYRILLIC CAPITAL LETTER TE
    '\u{0423}', // 0xF5  CYRILLIC CAPITAL LETTER U
    '\u{0416}', // 0xF6  CYRILLIC CAPITAL LETTER ZHE
    '\u{0412}', // 0xF7  CYRILLIC CAPITAL LETTER VE
    '\u{042C}', // 0xF8  CYRILLIC CAPITAL LETTER SOFT SIGN
    '\u{042B}', // 0xF9  CYRILLIC CAPITAL LETTER YERU
    '\u{0417}', // 0xFA  CYRILLIC CAPITAL LETTER ZE
    '\u{0428}', // 0xFB  CYRILLIC CAPITAL LETTER SHA
    '\u{042D}', // 0xFC  CYRILLIC CAPITAL LETTER E
    '\u{0429}', // 0xFD  CYRILLIC CAPITAL LETTER SHCHA
    '\u{0427}', // 0xFE  CYRILLIC CAPITAL LETTER CHE
    '\u{042A}', // 0xFF  CYRILLIC CAPITAL LETTER HARD SIGN
];

fn decode_koi8u(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(KOI8_U_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const ISO_8859_13_MAP: [char; 128] = [
    // 0x80..=0x9F (C1 control range, identity mapping)
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}', '\u{0087}',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}', '\u{008F}',
    '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\u{0097}',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}', '\u{009F}',
    // 0xA0..=0xAF
    '\u{00A0}', // 0xA0  NO-BREAK SPACE
    '\u{201D}', // 0xA1  RIGHT DOUBLE QUOTATION MARK
    '\u{00A2}', // 0xA2  CENT SIGN
    '\u{00A3}', // 0xA3  POUND SIGN
    '\u{00A4}', // 0xA4  CURRENCY SIGN
    '\u{201E}', // 0xA5  DOUBLE LOW-9 QUOTATION MARK
    '\u{00A6}', // 0xA6  BROKEN BAR
    '\u{00A7}', // 0xA7  SECTION SIGN
    '\u{00D8}', // 0xA8  LATIN CAPITAL LETTER O WITH STROKE
    '\u{00A9}', // 0xA9  COPYRIGHT SIGN
    '\u{0156}', // 0xAA  LATIN CAPITAL LETTER R WITH CEDILLA
    '\u{00AB}', // 0xAB  LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{00AC}', // 0xAC  NOT SIGN
    '\u{00AD}', // 0xAD  SOFT HYPHEN
    '\u{00AE}', // 0xAE  REGISTERED SIGN
    '\u{00C6}', // 0xAF  LATIN CAPITAL LETTER AE
    // 0xB0..=0xBF
    '\u{00B0}', // 0xB0  DEGREE SIGN
    '\u{00B1}', // 0xB1  PLUS-MINUS SIGN
    '\u{00B2}', // 0xB2  SUPERSCRIPT TWO
    '\u{00B3}', // 0xB3  SUPERSCRIPT THREE
    '\u{201C}', // 0xB4  LEFT DOUBLE QUOTATION MARK
    '\u{00B5}', // 0xB5  MICRO SIGN
    '\u{00B6}', // 0xB6  PILCROW SIGN
    '\u{00B7}', // 0xB7  MIDDLE DOT
    '\u{00F8}', // 0xB8  LATIN SMALL LETTER O WITH STROKE
    '\u{00B9}', // 0xB9  SUPERSCRIPT ONE
    '\u{0157}', // 0xBA  LATIN SMALL LETTER R WITH CEDILLA
    '\u{00BB}', // 0xBB  RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{00BC}', // 0xBC  VULGAR FRACTION ONE QUARTER
    '\u{00BD}', // 0xBD  VULGAR FRACTION ONE HALF
    '\u{00BE}', // 0xBE  VULGAR FRACTION THREE QUARTERS
    '\u{00E6}', // 0xBF  LATIN SMALL LETTER AE
    // 0xC0..=0xCF
    '\u{0104}', // 0xC0  LATIN CAPITAL LETTER A WITH OGONEK
    '\u{012E}', // 0xC1  LATIN CAPITAL LETTER I WITH OGONEK
    '\u{0100}', // 0xC2  LATIN CAPITAL LETTER A WITH MACRON
    '\u{0106}', // 0xC3  LATIN CAPITAL LETTER C WITH ACUTE
    '\u{00C4}', // 0xC4  LATIN CAPITAL LETTER A WITH DIAERESIS
    '\u{00C5}', // 0xC5  LATIN CAPITAL LETTER A WITH RING ABOVE
    '\u{0118}', // 0xC6  LATIN CAPITAL LETTER E WITH OGONEK
    '\u{0112}', // 0xC7  LATIN CAPITAL LETTER E WITH MACRON
    '\u{010C}', // 0xC8  LATIN CAPITAL LETTER C WITH CARON
    '\u{00C9}', // 0xC9  LATIN CAPITAL LETTER E WITH ACUTE
    '\u{0179}', // 0xCA  LATIN CAPITAL LETTER Z WITH ACUTE
    '\u{0116}', // 0xCB  LATIN CAPITAL LETTER E WITH DOT ABOVE
    '\u{0122}', // 0xCC  LATIN CAPITAL LETTER G WITH CEDILLA
    '\u{0136}', // 0xCD  LATIN CAPITAL LETTER K WITH CEDILLA
    '\u{012A}', // 0xCE  LATIN CAPITAL LETTER I WITH MACRON
    '\u{013B}', // 0xCF  LATIN CAPITAL LETTER L WITH CEDILLA
    // 0xD0..=0xDF
    '\u{0160}', // 0xD0  LATIN CAPITAL LETTER S WITH CARON
    '\u{0143}', // 0xD1  LATIN CAPITAL LETTER N WITH ACUTE
    '\u{0145}', // 0xD2  LATIN CAPITAL LETTER N WITH CEDILLA
    '\u{00D3}', // 0xD3  LATIN CAPITAL LETTER O WITH ACUTE
    '\u{014C}', // 0xD4  LATIN CAPITAL LETTER O WITH MACRON
    '\u{00D5}', // 0xD5  LATIN CAPITAL LETTER O WITH TILDE
    '\u{00D6}', // 0xD6  LATIN CAPITAL LETTER O WITH DIAERESIS
    '\u{00D7}', // 0xD7  MULTIPLICATION SIGN
    '\u{0172}', // 0xD8  LATIN CAPITAL LETTER U WITH OGONEK
    '\u{0141}', // 0xD9  LATIN CAPITAL LETTER L WITH STROKE
    '\u{015A}', // 0xDA  LATIN CAPITAL LETTER S WITH ACUTE
    '\u{016A}', // 0xDB  LATIN CAPITAL LETTER U WITH MACRON
    '\u{00DC}', // 0xDC  LATIN CAPITAL LETTER U WITH DIAERESIS
    '\u{017B}', // 0xDD  LATIN CAPITAL LETTER Z WITH DOT ABOVE
    '\u{017D}', // 0xDE  LATIN CAPITAL LETTER Z WITH CARON
    '\u{00DF}', // 0xDF  LATIN SMALL LETTER SHARP S
    // 0xE0..=0xEF
    '\u{0105}', // 0xE0  LATIN SMALL LETTER A WITH OGONEK
    '\u{012F}', // 0xE1  LATIN SMALL LETTER I WITH OGONEK
    '\u{0101}', // 0xE2  LATIN SMALL LETTER A WITH MACRON
    '\u{0107}', // 0xE3  LATIN SMALL LETTER C WITH ACUTE
    '\u{00E4}', // 0xE4  LATIN SMALL LETTER A WITH DIAERESIS
    '\u{00E5}', // 0xE5  LATIN SMALL LETTER A WITH RING ABOVE
    '\u{0119}', // 0xE6  LATIN SMALL LETTER E WITH OGONEK
    '\u{0113}', // 0xE7  LATIN SMALL LETTER E WITH MACRON
    '\u{010D}', // 0xE8  LATIN SMALL LETTER C WITH CARON
    '\u{00E9}', // 0xE9  LATIN SMALL LETTER E WITH ACUTE
    '\u{017A}', // 0xEA  LATIN SMALL LETTER Z WITH ACUTE
    '\u{0117}', // 0xEB  LATIN SMALL LETTER E WITH DOT ABOVE
    '\u{0123}', // 0xEC  LATIN SMALL LETTER G WITH CEDILLA
    '\u{0137}', // 0xED  LATIN SMALL LETTER K WITH CEDILLA
    '\u{012B}', // 0xEE  LATIN SMALL LETTER I WITH MACRON
    '\u{013C}', // 0xEF  LATIN SMALL LETTER L WITH CEDILLA
    // 0xF0..=0xFF
    '\u{0161}', // 0xF0  LATIN SMALL LETTER S WITH CARON
    '\u{0144}', // 0xF1  LATIN SMALL LETTER N WITH ACUTE
    '\u{0146}', // 0xF2  LATIN SMALL LETTER N WITH CEDILLA
    '\u{00F3}', // 0xF3  LATIN SMALL LETTER O WITH ACUTE
    '\u{014D}', // 0xF4  LATIN SMALL LETTER O WITH MACRON
    '\u{00F5}', // 0xF5  LATIN SMALL LETTER O WITH TILDE
    '\u{00F6}', // 0xF6  LATIN SMALL LETTER O WITH DIAERESIS
    '\u{00F7}', // 0xF7  DIVISION SIGN
    '\u{0173}', // 0xF8  LATIN SMALL LETTER U WITH OGONEK
    '\u{0142}', // 0xF9  LATIN SMALL LETTER L WITH STROKE
    '\u{015B}', // 0xFA  LATIN SMALL LETTER S WITH ACUTE
    '\u{016B}', // 0xFB  LATIN SMALL LETTER U WITH MACRON
    '\u{00FC}', // 0xFC  LATIN SMALL LETTER U WITH DIAERESIS
    '\u{017C}', // 0xFD  LATIN SMALL LETTER Z WITH DOT ABOVE
    '\u{017E}', // 0xFE  LATIN SMALL LETTER Z WITH CARON
    '\u{2019}', // 0xFF  RIGHT SINGLE QUOTATION MARK
];

fn decode_iso8859_13(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(ISO_8859_13_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const ISO_8859_10_MAP: [char; 128] = [
    // 0x80..=0x9F (C1 control range, identity mapping)
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}', '\u{0087}',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}', '\u{008F}',
    '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\u{0097}',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}', '\u{009F}',
    // 0xA0..=0xAF
    '\u{00A0}', // 0xA0  NO-BREAK SPACE
    '\u{0104}', // 0xA1  LATIN CAPITAL LETTER A WITH OGONEK
    '\u{0112}', // 0xA2  LATIN CAPITAL LETTER E WITH MACRON
    '\u{0122}', // 0xA3  LATIN CAPITAL LETTER G WITH CEDILLA
    '\u{012A}', // 0xA4  LATIN CAPITAL LETTER I WITH MACRON
    '\u{0128}', // 0xA5  LATIN CAPITAL LETTER I WITH TILDE
    '\u{0136}', // 0xA6  LATIN CAPITAL LETTER K WITH CEDILLA
    '\u{00A7}', // 0xA7  SECTION SIGN
    '\u{013B}', // 0xA8  LATIN CAPITAL LETTER L WITH CEDILLA
    '\u{0110}', // 0xA9  LATIN CAPITAL LETTER D WITH STROKE / LATIN CAPITAL LETTER ETH
    '\u{0160}', // 0xAA  LATIN CAPITAL LETTER S WITH CARON
    '\u{0166}', // 0xAB  LATIN CAPITAL LETTER T WITH STROKE
    '\u{017D}', // 0xAC  LATIN CAPITAL LETTER Z WITH CARON
    '\u{00AD}', // 0xAD  SOFT HYPHEN
    '\u{016A}', // 0xAE  LATIN CAPITAL LETTER U WITH MACRON
    '\u{014A}', // 0xAF  LATIN CAPITAL LETTER ENG
    // 0xB0..=0xBF
    '\u{00B0}', // 0xB0  DEGREE SIGN
    '\u{0105}', // 0xB1  LATIN SMALL LETTER A WITH OGONEK
    '\u{0113}', // 0xB2  LATIN SMALL LETTER E WITH MACRON
    '\u{0123}', // 0xB3  LATIN SMALL LETTER G WITH CEDILLA
    '\u{012B}', // 0xB4  LATIN SMALL LETTER I WITH MACRON
    '\u{0129}', // 0xB5  LATIN SMALL LETTER I WITH TILDE
    '\u{0137}', // 0xB6  LATIN SMALL LETTER K WITH CEDILLA
    '\u{00B7}', // 0xB7  MIDDLE DOT
    '\u{013C}', // 0xB8  LATIN SMALL LETTER L WITH CEDILLA
    '\u{0111}', // 0xB9  LATIN SMALL LETTER D WITH STROKE / LATIN SMALL LETTER ETH
    '\u{0161}', // 0xBA  LATIN SMALL LETTER S WITH CARON
    '\u{0167}', // 0xBB  LATIN SMALL LETTER T WITH STROKE
    '\u{017E}', // 0xBC  LATIN SMALL LETTER Z WITH CARON
    '\u{2015}', // 0xBD  HORIZONTAL BAR
    '\u{016B}', // 0xBE  LATIN SMALL LETTER U WITH MACRON
    '\u{014B}', // 0xBF  LATIN SMALL LETTER ENG
    // 0xC0..=0xCF
    '\u{0100}', // 0xC0  LATIN CAPITAL LETTER A WITH MACRON
    '\u{00C1}', // 0xC1  LATIN CAPITAL LETTER A WITH ACUTE
    '\u{00C2}', // 0xC2  LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    '\u{00C3}', // 0xC3  LATIN CAPITAL LETTER A WITH TILDE
    '\u{00C4}', // 0xC4  LATIN CAPITAL LETTER A WITH DIAERESIS
    '\u{00C5}', // 0xC5  LATIN CAPITAL LETTER A WITH RING ABOVE
    '\u{00C6}', // 0xC6  LATIN CAPITAL LETTER AE
    '\u{012E}', // 0xC7  LATIN CAPITAL LETTER I WITH OGONEK
    '\u{010C}', // 0xC8  LATIN CAPITAL LETTER C WITH CARON
    '\u{00C9}', // 0xC9  LATIN CAPITAL LETTER E WITH ACUTE
    '\u{0118}', // 0xCA  LATIN CAPITAL LETTER E WITH OGONEK
    '\u{00CB}', // 0xCB  LATIN CAPITAL LETTER E WITH DIAERESIS
    '\u{0116}', // 0xCC  LATIN CAPITAL LETTER E WITH DOT ABOVE
    '\u{00CD}', // 0xCD  LATIN CAPITAL LETTER I WITH ACUTE
    '\u{00CE}', // 0xCE  LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    '\u{01CF}', // 0xCF  LATIN CAPITAL LETTER I WITH CARON
    // 0xD0..=0xDF
    '\u{00D0}', // 0xD0  LATIN CAPITAL LETTER ETH
    '\u{0145}', // 0xD1  LATIN CAPITAL LETTER N WITH CEDILLA
    '\u{014C}', // 0xD2  LATIN CAPITAL LETTER O WITH MACRON
    '\u{00D3}', // 0xD3  LATIN CAPITAL LETTER O WITH ACUTE
    '\u{00D4}', // 0xD4  LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    '\u{00D5}', // 0xD5  LATIN CAPITAL LETTER O WITH TILDE
    '\u{00D6}', // 0xD6  LATIN CAPITAL LETTER O WITH DIAERESIS
    '\u{0168}', // 0xD7  LATIN CAPITAL LETTER U WITH TILDE
    '\u{00D8}', // 0xD8  LATIN CAPITAL LETTER O WITH STROKE
    '\u{0172}', // 0xD9  LATIN CAPITAL LETTER U WITH OGONEK
    '\u{00DA}', // 0xDA  LATIN CAPITAL LETTER U WITH ACUTE
    '\u{00DB}', // 0xDB  LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    '\u{00DC}', // 0xDC  LATIN CAPITAL LETTER U WITH DIAERESIS
    '\u{00DD}', // 0xDD  LATIN CAPITAL LETTER Y WITH ACUTE
    '\u{00DE}', // 0xDE  LATIN CAPITAL LETTER THORN
    '\u{00DF}', // 0xDF  LATIN SMALL LETTER SHARP S
    // 0xE0..=0xEF
    '\u{0101}', // 0xE0  LATIN SMALL LETTER A WITH MACRON
    '\u{00E1}', // 0xE1  LATIN SMALL LETTER A WITH ACUTE
    '\u{00E2}', // 0xE2  LATIN SMALL LETTER A WITH CIRCUMFLEX
    '\u{00E3}', // 0xE3  LATIN SMALL LETTER A WITH TILDE
    '\u{00E4}', // 0xE4  LATIN SMALL LETTER A WITH DIAERESIS
    '\u{00E5}', // 0xE5  LATIN SMALL LETTER A WITH RING ABOVE
    '\u{00E6}', // 0xE6  LATIN SMALL LETTER AE
    '\u{012F}', // 0xE7  LATIN SMALL LETTER I WITH OGONEK
    '\u{010D}', // 0xE8  LATIN SMALL LETTER C WITH CARON
    '\u{00E9}', // 0xE9  LATIN SMALL LETTER E WITH ACUTE
    '\u{0119}', // 0xEA  LATIN SMALL LETTER E WITH OGONEK
    '\u{00EB}', // 0xEB  LATIN SMALL LETTER E WITH DIAERESIS
    '\u{0117}', // 0xEC  LATIN SMALL LETTER E WITH DOT ABOVE
    '\u{00ED}', // 0xED  LATIN SMALL LETTER I WITH ACUTE
    '\u{00EE}', // 0xEE  LATIN SMALL LETTER I WITH CIRCUMFLEX
    '\u{01D0}', // 0xEF  LATIN SMALL LETTER I WITH CARON
    // 0xF0..=0xFF
    '\u{00F0}', // 0xF0  LATIN SMALL LETTER ETH
    '\u{0146}', // 0xF1  LATIN SMALL LETTER N WITH CEDILLA
    '\u{014D}', // 0xF2  LATIN SMALL LETTER O WITH MACRON
    '\u{00F3}', // 0xF3  LATIN SMALL LETTER O WITH ACUTE
    '\u{00F4}', // 0xF4  LATIN SMALL LETTER O WITH CIRCUMFLEX
    '\u{00F5}', // 0xF5  LATIN SMALL LETTER O WITH TILDE
    '\u{00F6}', // 0xF6  LATIN SMALL LETTER O WITH DIAERESIS
    '\u{0169}', // 0xF7  LATIN SMALL LETTER U WITH TILDE
    '\u{00F8}', // 0xF8  LATIN SMALL LETTER O WITH STROKE
    '\u{0173}', // 0xF9  LATIN SMALL LETTER U WITH OGONEK
    '\u{00FA}', // 0xFA  LATIN SMALL LETTER U WITH ACUTE
    '\u{00FB}', // 0xFB  LATIN SMALL LETTER U WITH CIRCUMFLEX
    '\u{00FC}', // 0xFC  LATIN SMALL LETTER U WITH DIAERESIS
    '\u{00FD}', // 0xFD  LATIN SMALL LETTER Y WITH ACUTE
    '\u{00FE}', // 0xFE  LATIN SMALL LETTER THORN
    '\u{0138}', // 0xFF  LATIN SMALL LETTER KRA
];

fn decode_iso8859_10(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(ISO_8859_10_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const ISO_8859_16_MAP: [char; 128] = [
    // 0x80..=0x9F (C1 control range, identity mapping)
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}', '\u{0087}',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}', '\u{008F}',
    '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\u{0097}',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}', '\u{009F}',
    // 0xA0..=0xAF
    '\u{00A0}', // 0xA0  NO-BREAK SPACE
    '\u{0104}', // 0xA1  LATIN CAPITAL LETTER A WITH OGONEK
    '\u{0105}', // 0xA2  LATIN SMALL LETTER A WITH OGONEK
    '\u{0141}', // 0xA3  LATIN CAPITAL LETTER L WITH STROKE
    '\u{20AC}', // 0xA4  EURO SIGN
    '\u{201E}', // 0xA5  DOUBLE LOW-9 QUOTATION MARK
    '\u{0160}', // 0xA6  LATIN CAPITAL LETTER S WITH CARON
    '\u{00A7}', // 0xA7  SECTION SIGN
    '\u{0161}', // 0xA8  LATIN SMALL LETTER S WITH CARON
    '\u{00A9}', // 0xA9  COPYRIGHT SIGN
    '\u{0218}', // 0xAA  LATIN CAPITAL LETTER S WITH COMMA BELOW
    '\u{00AB}', // 0xAB  LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{0179}', // 0xAC  LATIN CAPITAL LETTER Z WITH ACUTE
    '\u{00AD}', // 0xAD  SOFT HYPHEN
    '\u{017A}', // 0xAE  LATIN SMALL LETTER Z WITH ACUTE
    '\u{017B}', // 0xAF  LATIN CAPITAL LETTER Z WITH DOT ABOVE
    // 0xB0..=0xBF
    '\u{00B0}', // 0xB0  DEGREE SIGN
    '\u{00B1}', // 0xB1  PLUS-MINUS SIGN
    '\u{010C}', // 0xB2  LATIN CAPITAL LETTER C WITH CARON
    '\u{0142}', // 0xB3  LATIN SMALL LETTER L WITH STROKE
    '\u{017D}', // 0xB4  LATIN CAPITAL LETTER Z WITH CARON
    '\u{201D}', // 0xB5  RIGHT DOUBLE QUOTATION MARK
    '\u{00B6}', // 0xB6  PILCROW SIGN
    '\u{00B7}', // 0xB7  MIDDLE DOT
    '\u{017E}', // 0xB8  LATIN SMALL LETTER Z WITH CARON
    '\u{010D}', // 0xB9  LATIN SMALL LETTER C WITH CARON
    '\u{0219}', // 0xBA  LATIN SMALL LETTER S WITH COMMA BELOW
    '\u{00BB}', // 0xBB  RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{0152}', // 0xBC  LATIN CAPITAL LIGATURE OE
    '\u{0153}', // 0xBD  LATIN SMALL LIGATURE OE
    '\u{0178}', // 0xBE  LATIN CAPITAL LETTER Y WITH DIAERESIS
    '\u{017C}', // 0xBF  LATIN SMALL LETTER Z WITH DOT ABOVE
    // 0xC0..=0xCF
    '\u{00C0}', // 0xC0  LATIN CAPITAL LETTER A WITH GRAVE
    '\u{00C1}', // 0xC1  LATIN CAPITAL LETTER A WITH ACUTE
    '\u{00C2}', // 0xC2  LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    '\u{0102}', // 0xC3  LATIN CAPITAL LETTER A WITH BREVE
    '\u{00C4}', // 0xC4  LATIN CAPITAL LETTER A WITH DIAERESIS
    '\u{0106}', // 0xC5  LATIN CAPITAL LETTER C WITH ACUTE
    '\u{00C6}', // 0xC6  LATIN CAPITAL LETTER AE
    '\u{00C7}', // 0xC7  LATIN CAPITAL LETTER C WITH CEDILLA
    '\u{00C8}', // 0xC8  LATIN CAPITAL LETTER E WITH GRAVE
    '\u{00C9}', // 0xC9  LATIN CAPITAL LETTER E WITH ACUTE
    '\u{00CA}', // 0xCA  LATIN CAPITAL LETTER E WITH CIRCUMFLEX
    '\u{00CB}', // 0xCB  LATIN CAPITAL LETTER E WITH DIAERESIS
    '\u{00CC}', // 0xCC  LATIN CAPITAL LETTER I WITH GRAVE
    '\u{00CD}', // 0xCD  LATIN CAPITAL LETTER I WITH ACUTE
    '\u{00CE}', // 0xCE  LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    '\u{00CF}', // 0xCF  LATIN CAPITAL LETTER I WITH DIAERESIS
    // 0xD0..=0xDF
    '\u{0110}', // 0xD0  LATIN CAPITAL LETTER D WITH STROKE
    '\u{0143}', // 0xD1  LATIN CAPITAL LETTER N WITH ACUTE
    '\u{00D2}', // 0xD2  LATIN CAPITAL LETTER O WITH GRAVE
    '\u{00D3}', // 0xD3  LATIN CAPITAL LETTER O WITH ACUTE
    '\u{00D4}', // 0xD4  LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    '\u{0150}', // 0xD5  LATIN CAPITAL LETTER O WITH DOUBLE ACUTE
    '\u{00D6}', // 0xD6  LATIN CAPITAL LETTER O WITH DIAERESIS
    '\u{015A}', // 0xD7  LATIN CAPITAL LETTER S WITH ACUTE
    '\u{0170}', // 0xD8  LATIN CAPITAL LETTER U WITH DOUBLE ACUTE
    '\u{00D9}', // 0xD9  LATIN CAPITAL LETTER U WITH GRAVE
    '\u{00DA}', // 0xDA  LATIN CAPITAL LETTER U WITH ACUTE
    '\u{00DB}', // 0xDB  LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    '\u{00DC}', // 0xDC  LATIN CAPITAL LETTER U WITH DIAERESIS
    '\u{0118}', // 0xDD  LATIN CAPITAL LETTER E WITH OGONEK
    '\u{021A}', // 0xDE  LATIN CAPITAL LETTER T WITH COMMA BELOW
    '\u{00DF}', // 0xDF  LATIN SMALL LETTER SHARP S
    // 0xE0..=0xEF
    '\u{00E0}', // 0xE0  LATIN SMALL LETTER A WITH GRAVE
    '\u{00E1}', // 0xE1  LATIN SMALL LETTER A WITH ACUTE
    '\u{00E2}', // 0xE2  LATIN SMALL LETTER A WITH CIRCUMFLEX
    '\u{0103}', // 0xE3  LATIN SMALL LETTER A WITH BREVE
    '\u{00E4}', // 0xE4  LATIN SMALL LETTER A WITH DIAERESIS
    '\u{0107}', // 0xE5  LATIN SMALL LETTER C WITH ACUTE
    '\u{00E6}', // 0xE6  LATIN SMALL LETTER AE
    '\u{00E7}', // 0xE7  LATIN SMALL LETTER C WITH CEDILLA
    '\u{00E8}', // 0xE8  LATIN SMALL LETTER E WITH GRAVE
    '\u{00E9}', // 0xE9  LATIN SMALL LETTER E WITH ACUTE
    '\u{00EA}', // 0xEA  LATIN SMALL LETTER E WITH CIRCUMFLEX
    '\u{00EB}', // 0xEB  LATIN SMALL LETTER E WITH DIAERESIS
    '\u{00EC}', // 0xEC  LATIN SMALL LETTER I WITH GRAVE
    '\u{00ED}', // 0xED  LATIN SMALL LETTER I WITH ACUTE
    '\u{00EE}', // 0xEE  LATIN SMALL LETTER I WITH CIRCUMFLEX
    '\u{00EF}', // 0xEF  LATIN SMALL LETTER I WITH DIAERESIS
    // 0xF0..=0xFF
    '\u{0111}', // 0xF0  LATIN SMALL LETTER D WITH STROKE
    '\u{0144}', // 0xF1  LATIN SMALL LETTER N WITH ACUTE
    '\u{00F2}', // 0xF2  LATIN SMALL LETTER O WITH GRAVE
    '\u{00F3}', // 0xF3  LATIN SMALL LETTER O WITH ACUTE
    '\u{00F4}', // 0xF4  LATIN SMALL LETTER O WITH CIRCUMFLEX
    '\u{0151}', // 0xF5  LATIN SMALL LETTER O WITH DOUBLE ACUTE
    '\u{00F6}', // 0xF6  LATIN SMALL LETTER O WITH DIAERESIS
    '\u{015B}', // 0xF7  LATIN SMALL LETTER S WITH ACUTE
    '\u{0171}', // 0xF8  LATIN SMALL LETTER U WITH DOUBLE ACUTE
    '\u{00F9}', // 0xF9  LATIN SMALL LETTER U WITH GRAVE
    '\u{00FA}', // 0xFA  LATIN SMALL LETTER U WITH ACUTE
    '\u{00FB}', // 0xFB  LATIN SMALL LETTER U WITH CIRCUMFLEX
    '\u{00FC}', // 0xFC  LATIN SMALL LETTER U WITH DIAERESIS
    '\u{0119}', // 0xFD  LATIN SMALL LETTER E WITH OGONEK
    '\u{021B}', // 0xFE  LATIN SMALL LETTER T WITH COMMA BELOW
    '\u{00FF}', // 0xFF  LATIN SMALL LETTER Y WITH DIAERESIS
];

fn decode_iso8859_16(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(ISO_8859_16_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const ISO_8859_9_MAP: [char; 128] = [
    // 0x80..=0x9F (C1 control range, identity mapping)
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}', '\u{0087}',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}', '\u{008F}',
    '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\u{0097}',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}', '\u{009F}',
    // 0xA0..=0xAF
    '\u{00A0}', // 0xA0  NO-BREAK SPACE
    '\u{00A1}', // 0xA1  INVERTED EXCLAMATION MARK
    '\u{00A2}', // 0xA2  CENT SIGN
    '\u{00A3}', // 0xA3  POUND SIGN
    '\u{00A4}', // 0xA4  CURRENCY SIGN
    '\u{00A5}', // 0xA5  YEN SIGN
    '\u{00A6}', // 0xA6  BROKEN BAR
    '\u{00A7}', // 0xA7  SECTION SIGN
    '\u{00A8}', // 0xA8  DIAERESIS
    '\u{00A9}', // 0xA9  COPYRIGHT SIGN
    '\u{00AA}', // 0xAA  FEMININE ORDINAL INDICATOR
    '\u{00AB}', // 0xAB  LEFT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{00AC}', // 0xAC  NOT SIGN
    '\u{00AD}', // 0xAD  SOFT HYPHEN
    '\u{00AE}', // 0xAE  REGISTERED SIGN
    '\u{00AF}', // 0xAF  MACRON
    // 0xB0..=0xBF
    '\u{00B0}', // 0xB0  DEGREE SIGN
    '\u{00B1}', // 0xB1  PLUS-MINUS SIGN
    '\u{00B2}', // 0xB2  SUPERSCRIPT TWO
    '\u{00B3}', // 0xB3  SUPERSCRIPT THREE
    '\u{00B4}', // 0xB4  ACUTE ACCENT
    '\u{00B5}', // 0xB5  MICRO SIGN
    '\u{00B6}', // 0xB6  PILCROW SIGN
    '\u{00B7}', // 0xB7  MIDDLE DOT
    '\u{00B8}', // 0xB8  CEDILLA
    '\u{00B9}', // 0xB9  SUPERSCRIPT ONE
    '\u{00BA}', // 0xBA  MASCULINE ORDINAL INDICATOR
    '\u{00BB}', // 0xBB  RIGHT-POINTING DOUBLE ANGLE QUOTATION MARK
    '\u{00BC}', // 0xBC  VULGAR FRACTION ONE QUARTER
    '\u{00BD}', // 0xBD  VULGAR FRACTION ONE HALF
    '\u{00BE}', // 0xBE  VULGAR FRACTION THREE QUARTERS
    '\u{00BF}', // 0xBF  INVERTED QUESTION MARK
    // 0xC0..=0xCF
    '\u{00C0}', // 0xC0  LATIN CAPITAL LETTER A WITH GRAVE
    '\u{00C1}', // 0xC1  LATIN CAPITAL LETTER A WITH ACUTE
    '\u{00C2}', // 0xC2  LATIN CAPITAL LETTER A WITH CIRCUMFLEX
    '\u{00C3}', // 0xC3  LATIN CAPITAL LETTER A WITH TILDE
    '\u{00C4}', // 0xC4  LATIN CAPITAL LETTER A WITH DIAERESIS
    '\u{00C5}', // 0xC5  LATIN CAPITAL LETTER A WITH RING ABOVE
    '\u{00C6}', // 0xC6  LATIN CAPITAL LETTER AE
    '\u{00C7}', // 0xC7  LATIN CAPITAL LETTER C WITH CEDILLA
    '\u{00C8}', // 0xC8  LATIN CAPITAL LETTER E WITH GRAVE
    '\u{00C9}', // 0xC9  LATIN CAPITAL LETTER E WITH ACUTE
    '\u{00CA}', // 0xCA  LATIN CAPITAL LETTER E WITH CIRCUMFLEX
    '\u{00CB}', // 0xCB  LATIN CAPITAL LETTER E WITH DIAERESIS
    '\u{00CC}', // 0xCC  LATIN CAPITAL LETTER I WITH GRAVE
    '\u{00CD}', // 0xCD  LATIN CAPITAL LETTER I WITH ACUTE
    '\u{00CE}', // 0xCE  LATIN CAPITAL LETTER I WITH CIRCUMFLEX
    '\u{00CF}', // 0xCF  LATIN CAPITAL LETTER I WITH DIAERESIS
    // 0xD0..=0xDF
    '\u{011E}', // 0xD0  LATIN CAPITAL LETTER G WITH BREVE
    '\u{00D1}', // 0xD1  LATIN CAPITAL LETTER N WITH TILDE
    '\u{00D2}', // 0xD2  LATIN CAPITAL LETTER O WITH GRAVE
    '\u{00D3}', // 0xD3  LATIN CAPITAL LETTER O WITH ACUTE
    '\u{00D4}', // 0xD4  LATIN CAPITAL LETTER O WITH CIRCUMFLEX
    '\u{00D5}', // 0xD5  LATIN CAPITAL LETTER O WITH TILDE
    '\u{00D6}', // 0xD6  LATIN CAPITAL LETTER O WITH DIAERESIS
    '\u{00D7}', // 0xD7  MULTIPLICATION SIGN
    '\u{00D8}', // 0xD8  LATIN CAPITAL LETTER O WITH STROKE
    '\u{00D9}', // 0xD9  LATIN CAPITAL LETTER U WITH GRAVE
    '\u{00DA}', // 0xDA  LATIN CAPITAL LETTER U WITH ACUTE
    '\u{00DB}', // 0xDB  LATIN CAPITAL LETTER U WITH CIRCUMFLEX
    '\u{00DC}', // 0xDC  LATIN CAPITAL LETTER U WITH DIAERESIS
    '\u{0130}', // 0xDD  LATIN CAPITAL LETTER I WITH DOT ABOVE
    '\u{015E}', // 0xDE  LATIN CAPITAL LETTER S WITH CEDILLA
    '\u{00DF}', // 0xDF  LATIN SMALL LETTER SHARP S
    // 0xE0..=0xEF
    '\u{00E0}', // 0xE0  LATIN SMALL LETTER A WITH GRAVE
    '\u{00E1}', // 0xE1  LATIN SMALL LETTER A WITH ACUTE
    '\u{00E2}', // 0xE2  LATIN SMALL LETTER A WITH CIRCUMFLEX
    '\u{00E3}', // 0xE3  LATIN SMALL LETTER A WITH TILDE
    '\u{00E4}', // 0xE4  LATIN SMALL LETTER A WITH DIAERESIS
    '\u{00E5}', // 0xE5  LATIN SMALL LETTER A WITH RING ABOVE
    '\u{00E6}', // 0xE6  LATIN SMALL LETTER AE
    '\u{00E7}', // 0xE7  LATIN SMALL LETTER C WITH CEDILLA
    '\u{00E8}', // 0xE8  LATIN SMALL LETTER E WITH GRAVE
    '\u{00E9}', // 0xE9  LATIN SMALL LETTER E WITH ACUTE
    '\u{00EA}', // 0xEA  LATIN SMALL LETTER E WITH CIRCUMFLEX
    '\u{00EB}', // 0xEB  LATIN SMALL LETTER E WITH DIAERESIS
    '\u{00EC}', // 0xEC  LATIN SMALL LETTER I WITH GRAVE
    '\u{00ED}', // 0xED  LATIN SMALL LETTER I WITH ACUTE
    '\u{00EE}', // 0xEE  LATIN SMALL LETTER I WITH CIRCUMFLEX
    '\u{00EF}', // 0xEF  LATIN SMALL LETTER I WITH DIAERESIS
    // 0xF0..=0xFF
    '\u{011F}', // 0xF0  LATIN SMALL LETTER G WITH BREVE
    '\u{00F1}', // 0xF1  LATIN SMALL LETTER N WITH TILDE
    '\u{00F2}', // 0xF2  LATIN SMALL LETTER O WITH GRAVE
    '\u{00F3}', // 0xF3  LATIN SMALL LETTER O WITH ACUTE
    '\u{00F4}', // 0xF4  LATIN SMALL LETTER O WITH CIRCUMFLEX
    '\u{00F5}', // 0xF5  LATIN SMALL LETTER O WITH TILDE
    '\u{00F6}', // 0xF6  LATIN SMALL LETTER O WITH DIAERESIS
    '\u{00F7}', // 0xF7  DIVISION SIGN
    '\u{00F8}', // 0xF8  LATIN SMALL LETTER O WITH STROKE
    '\u{00F9}', // 0xF9  LATIN SMALL LETTER U WITH GRAVE
    '\u{00FA}', // 0xFA  LATIN SMALL LETTER U WITH ACUTE
    '\u{00FB}', // 0xFB  LATIN SMALL LETTER U WITH CIRCUMFLEX
    '\u{00FC}', // 0xFC  LATIN SMALL LETTER U WITH DIAERESIS
    '\u{0131}', // 0xFD  LATIN SMALL LETTER DOTLESS I
    '\u{015F}', // 0xFE  LATIN SMALL LETTER S WITH CEDILLA
    '\u{00FF}', // 0xFF  LATIN SMALL LETTER Y WITH DIAERESIS
];

fn decode_iso8859_9(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(ISO_8859_9_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
    }
    result
}

const ISO_8859_6_MAP: [char; 128] = [
    // 0x80..=0x9F (C1 control range, identity mapping)
    '\u{0080}', '\u{0081}', '\u{0082}', '\u{0083}', '\u{0084}', '\u{0085}', '\u{0086}', '\u{0087}',
    '\u{0088}', '\u{0089}', '\u{008A}', '\u{008B}', '\u{008C}', '\u{008D}', '\u{008E}', '\u{008F}',
    '\u{0090}', '\u{0091}', '\u{0092}', '\u{0093}', '\u{0094}', '\u{0095}', '\u{0096}', '\u{0097}',
    '\u{0098}', '\u{0099}', '\u{009A}', '\u{009B}', '\u{009C}', '\u{009D}', '\u{009E}', '\u{009F}',
    // 0xA0..=0xAF
    '\u{00A0}', // 0xA0  NO-BREAK SPACE
    '\u{FFFD}', // 0xA1  UNDEFINED
    '\u{FFFD}', // 0xA2  UNDEFINED
    '\u{FFFD}', // 0xA3  UNDEFINED
    '\u{00A4}', // 0xA4  CURRENCY SIGN
    '\u{FFFD}', // 0xA5  UNDEFINED
    '\u{FFFD}', // 0xA6  UNDEFINED
    '\u{FFFD}', // 0xA7  UNDEFINED
    '\u{FFFD}', // 0xA8  UNDEFINED
    '\u{FFFD}', // 0xA9  UNDEFINED
    '\u{FFFD}', // 0xAA  UNDEFINED
    '\u{FFFD}', // 0xAB  UNDEFINED
    '\u{060C}', // 0xAC  ARABIC COMMA
    '\u{00AD}', // 0xAD  SOFT HYPHEN
    '\u{FFFD}', // 0xAE  UNDEFINED
    '\u{FFFD}', // 0xAF  UNDEFINED
    // 0xB0..=0xBF
    '\u{FFFD}', // 0xB0  UNDEFINED
    '\u{FFFD}', // 0xB1  UNDEFINED
    '\u{FFFD}', // 0xB2  UNDEFINED
    '\u{FFFD}', // 0xB3  UNDEFINED
    '\u{FFFD}', // 0xB4  UNDEFINED
    '\u{FFFD}', // 0xB5  UNDEFINED
    '\u{FFFD}', // 0xB6  UNDEFINED
    '\u{FFFD}', // 0xB7  UNDEFINED
    '\u{FFFD}', // 0xB8  UNDEFINED
    '\u{FFFD}', // 0xB9  UNDEFINED
    '\u{FFFD}', // 0xBA  UNDEFINED
    '\u{061B}', // 0xBB  ARABIC SEMICOLON
    '\u{FFFD}', // 0xBC  UNDEFINED
    '\u{FFFD}', // 0xBD  UNDEFINED
    '\u{FFFD}', // 0xBE  UNDEFINED
    '\u{061F}', // 0xBF  ARABIC QUESTION MARK
    // 0xC0..=0xCF
    '\u{FFFD}', // 0xC0  UNDEFINED
    '\u{0621}', // 0xC1  ARABIC LETTER HAMZA
    '\u{0622}', // 0xC2  ARABIC LETTER ALEF WITH MADDA ABOVE
    '\u{0623}', // 0xC3  ARABIC LETTER ALEF WITH HAMZA ABOVE
    '\u{0624}', // 0xC4  ARABIC LETTER WAW WITH HAMZA ABOVE
    '\u{0625}', // 0xC5  ARABIC LETTER ALEF WITH HAMZA BELOW
    '\u{0626}', // 0xC6  ARABIC LETTER YEH WITH HAMZA ABOVE
    '\u{0627}', // 0xC7  ARABIC LETTER ALEF
    '\u{0628}', // 0xC8  ARABIC LETTER BEH
    '\u{0629}', // 0xC9  ARABIC LETTER TEH MARBUTA
    '\u{062A}', // 0xCA  ARABIC LETTER TEH
    '\u{062B}', // 0xCB  ARABIC LETTER THEH
    '\u{062C}', // 0xCC  ARABIC LETTER JEEM
    '\u{062D}', // 0xCD  ARABIC LETTER HAH
    '\u{062E}', // 0xCE  ARABIC LETTER KHAH
    '\u{062F}', // 0xCF  ARABIC LETTER DAL
    // 0xD0..=0xDF
    '\u{0630}', // 0xD0  ARABIC LETTER THAL
    '\u{0631}', // 0xD1  ARABIC LETTER REH
    '\u{0632}', // 0xD2  ARABIC LETTER ZAIN
    '\u{0633}', // 0xD3  ARABIC LETTER SEEN
    '\u{0634}', // 0xD4  ARABIC LETTER SHEEN
    '\u{0635}', // 0xD5  ARABIC LETTER SAD
    '\u{0636}', // 0xD6  ARABIC LETTER DAD
    '\u{0637}', // 0xD7  ARABIC LETTER TAH
    '\u{0638}', // 0xD8  ARABIC LETTER ZAH
    '\u{0639}', // 0xD9  ARABIC LETTER AIN
    '\u{063A}', // 0xDA  ARABIC LETTER GHAIN
    '\u{FFFD}', // 0xDB  UNDEFINED
    '\u{FFFD}', // 0xDC  UNDEFINED
    '\u{FFFD}', // 0xDD  UNDEFINED
    '\u{FFFD}', // 0xDE  UNDEFINED
    '\u{FFFD}', // 0xDF  UNDEFINED
    // 0xE0..=0xEF
    '\u{0640}', // 0xE0  ARABIC TATWEEL
    '\u{0641}', // 0xE1  ARABIC LETTER FEH
    '\u{0642}', // 0xE2  ARABIC LETTER QAF
    '\u{0643}', // 0xE3  ARABIC LETTER KAF
    '\u{0644}', // 0xE4  ARABIC LETTER LAM
    '\u{0645}', // 0xE5  ARABIC LETTER MEEM
    '\u{0646}', // 0xE6  ARABIC LETTER NOON
    '\u{0647}', // 0xE7  ARABIC LETTER HEH
    '\u{0648}', // 0xE8  ARABIC LETTER WAW
    '\u{0649}', // 0xE9  ARABIC LETTER ALEF MAKSURA
    '\u{064A}', // 0xEA  ARABIC LETTER YEH
    '\u{064B}', // 0xEB  ARABIC FATHATAN
    '\u{064C}', // 0xEC  ARABIC DAMMATAN
    '\u{064D}', // 0xED  ARABIC KASRATAN
    '\u{064E}', // 0xEE  ARABIC FATHA
    '\u{064F}', // 0xEF  ARABIC DAMMA
    // 0xF0..=0xFF
    '\u{0650}', // 0xF0  ARABIC KASRA
    '\u{0651}', // 0xF1  ARABIC SHADDA
    '\u{0652}', // 0xF2  ARABIC SUKUN
    '\u{FFFD}', // 0xF3  UNDEFINED
    '\u{FFFD}', // 0xF4  UNDEFINED
    '\u{FFFD}', // 0xF5  UNDEFINED
    '\u{FFFD}', // 0xF6  UNDEFINED
    '\u{FFFD}', // 0xF7  UNDEFINED
    '\u{FFFD}', // 0xF8  UNDEFINED
    '\u{FFFD}', // 0xF9  UNDEFINED
    '\u{FFFD}', // 0xFA  UNDEFINED
    '\u{FFFD}', // 0xFB  UNDEFINED
    '\u{FFFD}', // 0xFC  UNDEFINED
    '\u{FFFD}', // 0xFD  UNDEFINED
    '\u{FFFD}', // 0xFE  UNDEFINED
    '\u{FFFD}', // 0xFF  UNDEFINED
];

fn decode_iso8859_6(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    for &b in bytes {
        if b >= 0x80 {
            result.push(ISO_8859_6_MAP[(b - 0x80) as usize]);
        } else {
            result.push(b as char);
        }
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
    fn test_decode_iso8859_1() {
        assert_eq!(decode(&[0x80], Charset::Iso8859_1), "\u{0080}");
        assert_eq!(decode(&[0xA9], Charset::Iso8859_1), "\u{00A9}");
        assert_eq!(decode(&[0xE9], Charset::Iso8859_1), "\u{00E9}");
        assert_eq!(decode(&[0xFF], Charset::Iso8859_1), "\u{00FF}");
        assert_eq!(decode(b"abc", Charset::Iso8859_1), "abc");
    }

    #[test]
    fn test_label_latin1_alias() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-1")),
            Charset::Iso8859_1
        );
        assert_eq!(sniff_charset(b"abc", Some("latin1")), Charset::Iso8859_1);
        assert_eq!(sniff_charset(b"abc", Some("l1")), Charset::Iso8859_1);
        assert_eq!(sniff_charset(b"abc", Some("cp819")), Charset::Iso8859_1);
        assert_eq!(sniff_charset(b"abc", Some("ibm819")), Charset::Iso8859_1);
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
    fn test_iso8859_2_decode() {
        assert_eq!(decode(&[0xE1, 0xE9, 0xED], Charset::Iso8859_2), "áéí");
        assert_eq!(decode(&[0xB9], Charset::Iso8859_2), "š");
        assert_eq!(decode(&[0xE8], Charset::Iso8859_2), "č");
        assert_eq!(decode(&[0xA9], Charset::Iso8859_2), "Š");
    }

    #[test]
    fn test_iso8859_2_sniff() {
        assert_eq!(
            sniff_charset(b"<html></html>", Some("iso-8859-2")),
            Charset::Iso8859_2
        );
        assert_eq!(
            sniff_charset(b"<html></html>", Some("l2")),
            Charset::Iso8859_2
        );
    }

    #[test]
    fn test_iso8859_3_decode() {
        assert_eq!(decode(&[0x41], Charset::Iso8859_3), "A");
        assert_eq!(decode(&[0xA1], Charset::Iso8859_3), "\u{0126}");
        assert_eq!(decode(&[0xB1], Charset::Iso8859_3), "\u{0127}");
        assert_eq!(decode(&[0xA5], Charset::Iso8859_3), "\u{FFFD}");
    }

    #[test]
    fn test_iso8859_3_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-3")),
            Charset::Iso8859_3
        );
        assert_eq!(sniff_charset(b"abc", Some("latin3")), Charset::Iso8859_3);
    }

    #[test]
    fn test_iso8859_4_decode() {
        // ASCII passthrough
        assert_eq!(decode(b"abc 123", Charset::Iso8859_4), "abc 123");

        // Representative high-byte mappings
        assert_eq!(decode(&[0xA1], Charset::Iso8859_4), "\u{0104}"); // Ą
        assert_eq!(decode(&[0xB1], Charset::Iso8859_4), "\u{0105}"); // ą
        assert_eq!(decode(&[0xB9], Charset::Iso8859_4), "\u{0161}"); // š
        assert_eq!(decode(&[0xC0], Charset::Iso8859_4), "\u{0100}"); // Ā

        // Check a few more for thoroughness
        assert_eq!(decode(&[0xBD], Charset::Iso8859_4), "\u{014A}"); // Ŋ
        assert_eq!(decode(&[0xBF], Charset::Iso8859_4), "\u{014B}"); // ŋ
        assert_eq!(decode(&[0xFF], Charset::Iso8859_4), "\u{02D9}"); // ˙

        // Round-trip check
        let sample = "ĄąšĀŊŋ˙";
        let bytes = &[0xA1, 0xB1, 0xB9, 0xC0, 0xBD, 0xBF, 0xFF];
        assert_eq!(decode(bytes, Charset::Iso8859_4), sample);
    }

    #[test]
    fn test_iso8859_4_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-4")),
            Charset::Iso8859_4
        );
        assert_eq!(sniff_charset(b"abc", Some("latin4")), Charset::Iso8859_4);
        assert_eq!(sniff_charset(b"abc", Some("l4")), Charset::Iso8859_4);
        assert_eq!(sniff_charset(b"abc", Some("iso8859-4")), Charset::Iso8859_4);
        assert_eq!(
            sniff_charset(b"abc", Some("csisolatin4")),
            Charset::Iso8859_4
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-4\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_4);
    }

    #[test]
    fn test_iso8859_5_decode() {
        // ASCII passthrough
        assert_eq!(decode(b"abc 123", Charset::Iso8859_5), "abc 123");

        // Representative high-byte mappings
        assert_eq!(decode(&[0xA0], Charset::Iso8859_5), "\u{00A0}"); // NBSP
        assert_eq!(decode(&[0xA1], Charset::Iso8859_5), "\u{0401}"); // Ё
        assert_eq!(decode(&[0xB0], Charset::Iso8859_5), "\u{0410}"); // А
        assert_eq!(decode(&[0xDF], Charset::Iso8859_5), "\u{043F}"); // п
        assert_eq!(decode(&[0xF0], Charset::Iso8859_5), "\u{2116}"); // №
        assert_eq!(decode(&[0xFD], Charset::Iso8859_5), "\u{00A7}"); // §
        assert_eq!(decode(&[0xFF], Charset::Iso8859_5), "\u{045F}"); // џ

        // Check a full Cyrillic sentence: "Привет" (using ISO-8859-5 mappings)
        // П: 0xBF (U+041F)
        // р: 0xE0 (U+0440)
        // и: 0xD8 (U+0438)
        // в: 0xD2 (U+0432)
        // е: 0xD5 (U+0435)
        // т: 0xE2 (U+0442)
        let bytes = &[0xBF, 0xE0, 0xD8, 0xD2, 0xD5, 0xE2];
        assert_eq!(decode(bytes, Charset::Iso8859_5), "Привет");
    }

    #[test]
    fn test_iso8859_5_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-5")),
            Charset::Iso8859_5
        );
        assert_eq!(sniff_charset(b"abc", Some("cyrillic")), Charset::Iso8859_5);
        assert_eq!(sniff_charset(b"abc", Some("iso8859-5")), Charset::Iso8859_5);
        assert_eq!(
            sniff_charset(b"abc", Some("csisolatincyrillic")),
            Charset::Iso8859_5
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-5\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_5);
    }

    #[test]
    fn test_iso8859_7_decode() {
        // ASCII passthrough
        assert_eq!(decode(b"abc 123", Charset::Iso8859_7), "abc 123");

        // Representative high-byte mappings
        assert_eq!(decode(&[0xA0], Charset::Iso8859_7), "\u{00A0}"); // NBSP
        assert_eq!(decode(&[0xA4], Charset::Iso8859_7), "\u{20AC}"); // Euro
        assert_eq!(decode(&[0xA5], Charset::Iso8859_7), "\u{20AF}"); // Drachma
        assert_eq!(decode(&[0xC1], Charset::Iso8859_7), "\u{0391}"); // Capital Alpha
        assert_eq!(decode(&[0xD1], Charset::Iso8859_7), "\u{03A1}"); // Capital Rho

        // Undefined byte positions decoding to U+FFFD
        assert_eq!(decode(&[0xAE], Charset::Iso8859_7), "\u{FFFD}");
        assert_eq!(decode(&[0xD2], Charset::Iso8859_7), "\u{FFFD}");
        assert_eq!(decode(&[0xFF], Charset::Iso8859_7), "\u{FFFD}");

        // Greek word "Ελλάδα" (Greece)
        let bytes = &[0xC5, 0xEB, 0xEB, 0xDC, 0xE4, 0xE1];
        assert_eq!(decode(bytes, Charset::Iso8859_7), "Ελλάδα");
    }

    #[test]
    fn test_iso8859_7_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-7")),
            Charset::Iso8859_7
        );
        assert_eq!(sniff_charset(b"abc", Some("greek")), Charset::Iso8859_7);
        assert_eq!(sniff_charset(b"abc", Some("iso8859-7")), Charset::Iso8859_7);
        assert_eq!(sniff_charset(b"abc", Some("greek8")), Charset::Iso8859_7);
        assert_eq!(
            sniff_charset(b"abc", Some("csisolatingreek")),
            Charset::Iso8859_7
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-7\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_7);
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

    #[test]
    fn test_windows1253_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1253")),
            Charset::Windows1253
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1253")), Charset::Windows1253);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1253")),
            Charset::Windows1253
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1253\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1253);
    }

    #[test]
    fn test_windows1253_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1253), "abc 123");

        // Greek specific bytes:
        // 0xC1 -> "Α" (U+0391, Greek Capital Letter Alpha)
        assert_eq!(decode(&[0xC1], Charset::Windows1253), "Α");
        // 0xE1 -> "α" (U+03B1, Greek Small Letter Alpha)
        assert_eq!(decode(&[0xE1], Charset::Windows1253), "α");
        // 0xDC -> "ά" (U+03AC, Greek Small Letter Alpha with Tonos)
        assert_eq!(decode(&[0xDC], Charset::Windows1253), "ά");
        // 0x80 -> "€" (U+20AC, Euro Sign)
        assert_eq!(decode(&[0x80], Charset::Windows1253), "€");

        // Undefined bytes (map to replacement character U+FFFD)
        assert_eq!(decode(&[0xAA], Charset::Windows1253), "\u{FFFD}");
        assert_eq!(decode(&[0xD2], Charset::Windows1253), "\u{FFFD}");
        assert_eq!(decode(&[0xFF], Charset::Windows1253), "\u{FFFD}");

        // Control characters checking:
        // 0x81 -> control char U+0081
        assert_eq!(decode(&[0x81], Charset::Windows1253), "\u{0081}");
        // 0x88 -> control char U+0088
        assert_eq!(decode(&[0x88], Charset::Windows1253), "\u{0088}");
    }

    #[test]
    fn test_windows1254_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1254")),
            Charset::Windows1254
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1254")), Charset::Windows1254);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1254")),
            Charset::Windows1254
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1254\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1254);
    }

    #[test]
    fn test_windows1254_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1254), "abc 123");

        // Turkish specific bytes:
        // 0xD0 -> "Ğ" (U+011E)
        assert_eq!(decode(&[0xD0], Charset::Windows1254), "Ğ");
        // 0xDD -> "İ" (U+0130)
        assert_eq!(decode(&[0xDD], Charset::Windows1254), "İ");
        // 0xDE -> "Ş" (U+015E)
        assert_eq!(decode(&[0xDE], Charset::Windows1254), "Ş");
        // 0xF0 -> "ğ" (U+011F)
        assert_eq!(decode(&[0xF0], Charset::Windows1254), "ğ");
        // 0xFD -> "ı" (U+0131)
        assert_eq!(decode(&[0xFD], Charset::Windows1254), "ı");
        // 0xFE -> "ş" (U+015F)
        assert_eq!(decode(&[0xFE], Charset::Windows1254), "ş");

        // Euro sign U+20AC
        assert_eq!(decode(&[0x80], Charset::Windows1254), "€");

        // Control / Undefined bytes (map to raw control codepoints as in 1253)
        assert_eq!(decode(&[0x81], Charset::Windows1254), "\u{0081}");
        assert_eq!(decode(&[0x8D], Charset::Windows1254), "\u{008D}");
        assert_eq!(decode(&[0x9E], Charset::Windows1254), "\u{009E}");
    }

    #[test]
    fn test_windows1255_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1255")),
            Charset::Windows1255
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1255")), Charset::Windows1255);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1255")),
            Charset::Windows1255
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1255\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1255);
    }

    #[test]
    fn test_windows1255_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1255), "abc 123");

        // Hebrew specific bytes:
        // 0xE0 -> "א" (U+05D0, Hebrew Letter Alef)
        assert_eq!(decode(&[0xE0], Charset::Windows1255), "א");
        // 0xFA -> "ת" (U+05EA, Hebrew Letter Tav)
        assert_eq!(decode(&[0xFA], Charset::Windows1255), "ת");
        // 0xC0 -> "ְ" (U+05B0, Hebrew Point Sheva)
        assert_eq!(decode(&[0xC0], Charset::Windows1255), "\u{05B0}");

        // Multiplication sign (0xAA) -> U+00D7 (×)
        assert_eq!(decode(&[0xAA], Charset::Windows1255), "×");
        // Division sign (0xBA) -> U+00F7 (÷)
        assert_eq!(decode(&[0xBA], Charset::Windows1255), "÷");

        // LRM (0xFD) -> U+200E
        assert_eq!(decode(&[0xFD], Charset::Windows1255), "\u{200E}");
        // RLM (0xFE) -> U+200F
        assert_eq!(decode(&[0xFE], Charset::Windows1255), "\u{200F}");

        // Undefined bytes map to replacement character U+FFFD
        assert_eq!(decode(&[0xA1], Charset::Windows1255), "\u{FFFD}");
        assert_eq!(decode(&[0xD5], Charset::Windows1255), "\u{FFFD}");
        assert_eq!(decode(&[0xFF], Charset::Windows1255), "\u{FFFD}");

        // Control characters:
        // 0x81 -> control char U+0081
        assert_eq!(decode(&[0x81], Charset::Windows1255), "\u{0081}");
        // 0x8D -> control char U+008D
        assert_eq!(decode(&[0x8D], Charset::Windows1255), "\u{008D}");
    }

    #[test]
    fn test_windows1256_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1256")),
            Charset::Windows1256
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1256")), Charset::Windows1256);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1256")),
            Charset::Windows1256
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1256\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1256);
    }

    #[test]
    fn test_windows1256_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1256), "abc 123");

        // Arabic and French specific bytes:
        // 0x80 -> "€" (U+20AC)
        assert_eq!(decode(&[0x80], Charset::Windows1256), "€");
        // 0x81 -> "پ" (U+067E, Arabic Letter Peh)
        assert_eq!(decode(&[0x81], Charset::Windows1256), "پ");
        // 0xE6 -> "و" (U+0648, Arabic Letter Waw)
        assert_eq!(decode(&[0xE6], Charset::Windows1256), "و");
        // 0xE0 -> "à" (U+00E0, French Small A with grave)
        assert_eq!(decode(&[0xE0], Charset::Windows1256), "à");
        // 0xFF -> "ے" (U+06D2, Arabic Letter Yeh Barree)
        assert_eq!(decode(&[0xFF], Charset::Windows1256), "ے");

        // ZWNJ (0x9D) -> U+200C
        assert_eq!(decode(&[0x9D], Charset::Windows1256), "\u{200C}");
        // ZWJ (0x9E) -> U+200D
        assert_eq!(decode(&[0x9E], Charset::Windows1256), "\u{200D}");
        // LRM (0xFD) -> U+200E
        assert_eq!(decode(&[0xFD], Charset::Windows1256), "\u{200E}");
        // RLM (0xFE) -> U+200F
        assert_eq!(decode(&[0xFE], Charset::Windows1256), "\u{200F}");
    }

    #[test]
    fn test_windows1257_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1257")),
            Charset::Windows1257
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1257")), Charset::Windows1257);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1257")),
            Charset::Windows1257
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1257\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1257);
    }

    #[test]
    fn test_windows1257_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1257), "abc 123");

        // Baltic specific bytes:
        // 0x80 -> "€" (U+20AC)
        assert_eq!(decode(&[0x80], Charset::Windows1257), "€");
        // 0xC0 -> "Ą" (U+0104)
        assert_eq!(decode(&[0xC0], Charset::Windows1257), "Ą");
        // 0xE0 -> "ą" (U+0105)
        assert_eq!(decode(&[0xE0], Charset::Windows1257), "ą");
        // 0xCA -> "Ź" (U+0179)
        assert_eq!(decode(&[0xCA], Charset::Windows1257), "Ź");
        // 0xEA -> "ź" (U+017A)
        assert_eq!(decode(&[0xEA], Charset::Windows1257), "ź");
        // 0xFF -> "˙" (U+02D9)
        assert_eq!(decode(&[0xFF], Charset::Windows1257), "˙");
        // 0xD5 -> "Õ" (U+00D5)
        assert_eq!(decode(&[0xD5], Charset::Windows1257), "Õ");
        // 0xF5 -> "õ" (U+00F5)
        assert_eq!(decode(&[0xF5], Charset::Windows1257), "õ");

        // Undefined bytes map to replacement character U+FFFD
        assert_eq!(decode(&[0xA1], Charset::Windows1257), "\u{FFFD}");
        assert_eq!(decode(&[0xA5], Charset::Windows1257), "\u{FFFD}");

        // Control characters:
        // 0x81 -> control char U+0081
        assert_eq!(decode(&[0x81], Charset::Windows1257), "\u{0081}");
        // 0x9F -> control char U+009F
        assert_eq!(decode(&[0x9F], Charset::Windows1257), "\u{009F}");
    }

    #[test]
    fn test_windows1258_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("windows-1258")),
            Charset::Windows1258
        );
        assert_eq!(sniff_charset(b"abc", Some("cp1258")), Charset::Windows1258);
        assert_eq!(
            sniff_charset(b"abc", Some("x-cp1258")),
            Charset::Windows1258
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"windows-1258\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Windows1258);
    }

    #[test]
    fn test_windows1258_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Windows1258), "abc 123");

        // Windows-1258 specific bytes:
        // 0x80 -> "€" (U+20AC)
        assert_eq!(decode(&[0x80], Charset::Windows1258), "€");
        // 0xC3 -> "Ă" (U+0102)
        assert_eq!(decode(&[0xC3], Charset::Windows1258), "Ă");
        // 0xCC -> combining grave (U+0300)
        assert_eq!(decode(&[0xCC], Charset::Windows1258), "\u{0300}");
        // 0xFE -> dong sign (U+20AB)
        assert_eq!(decode(&[0xFE], Charset::Windows1258), "\u{20AB}");

        // Sample Vietnamese byte sequence using combining accents
        let bytes = &[0xCA, 0xEC, b'n', b'g', b' ', b'V', b'i', 0xEA, 0xF2, b't'];
        assert_eq!(
            decode(bytes, Charset::Windows1258),
            "Ê\u{0301}ng Viê\u{0323}t"
        );
    }

    #[test]
    fn test_koi8r_sniff() {
        assert_eq!(sniff_charset(b"abc", Some("koi8-r")), Charset::Koi8R);
        assert_eq!(sniff_charset(b"abc", Some("koi8_r")), Charset::Koi8R);
        assert_eq!(sniff_charset(b"abc", Some("cskoi8r")), Charset::Koi8R);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"koi8-r\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Koi8R);
    }

    #[test]
    fn test_koi8r_decode() {
        // ASCII passthrough (as required: "an ASCII byte decodes to itself")
        assert_eq!(decode(b"abc 123", Charset::Koi8R), "abc 123");

        // Representative high-byte mappings
        // The prompt asks us to assert/verify specific mappings, but also to verify the indexing
        // against the 128 elements provided in order, which match the standard.
        // Let's assert both standard KOI8-R and standard anchors:
        // Byte 0xC1 maps to U+0430 (CYRILLIC SMALL LETTER A) in standard KOI8-R (Index 0x41)
        assert_eq!(decode(&[0xC1], Charset::Koi8R), "\u{0430}");
        // Byte 0xE1 maps to U+0410 (CYRILLIC CAPITAL LETTER A) in standard KOI8-R (Index 0x61)
        assert_eq!(decode(&[0xE1], Charset::Koi8R), "\u{0410}");
        // Byte 0xF2 maps to U+0420 (CYRILLIC CAPITAL LETTER ER) in standard KOI8-R (Index 0x72)
        assert_eq!(decode(&[0xF2], Charset::Koi8R), "\u{0420}");
        // Byte 0xF0 maps to U+041F (CYRILLIC CAPITAL LETTER PE) in standard KOI8-R (Index 0x70)
        assert_eq!(decode(&[0xF0], Charset::Koi8R), "\u{041F}");

        // Verification of anchors per the prompt:
        // - Byte 0xE1 corresponds to index 0x61. In the canonical 128-entry list, index 0x61 is U+0410 (CYRILLIC CAPITAL LETTER A), while U+0430 (CYRILLIC SMALL LETTER A) is at index 0x41 (byte 0xC1).
        // - Byte 0xF0 corresponds to index 0x70. In the canonical 128-entry list, index 0x70 is U+041F (CYRILLIC CAPITAL LETTER PE), while U+0420 (CYRILLIC CAPITAL LETTER ER) is at index 0x72 (byte 0xF2).

        // Extra check for Cyrillic small letter IO (0xA3) -> U+0451
        assert_eq!(decode(&[0xA3], Charset::Koi8R), "\u{0451}");
        // Extra check for Cyrillic capital letter IO (0xB3) -> U+0401
        assert_eq!(decode(&[0xB3], Charset::Koi8R), "\u{0401}");
        // Extra check for Copyright sign (0xBF) -> U+00A9
        assert_eq!(decode(&[0xBF], Charset::Koi8R), "\u{00A9}");
    }

    #[test]
    fn test_koi8u_sniff() {
        assert_eq!(sniff_charset(b"abc", Some("koi8-u")), Charset::Koi8U);
        assert_eq!(sniff_charset(b"abc", Some("koi8_u")), Charset::Koi8U);
        assert_eq!(sniff_charset(b"abc", Some("koi8-ru")), Charset::Koi8U);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"koi8-u\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Koi8U);
    }

    #[test]
    fn test_koi8u_decode() {
        // ASCII passthrough
        assert_eq!(decode(b"abc 123", Charset::Koi8U), "abc 123");

        // Verified Ukrainian/Belarusian specific overrides
        assert_eq!(decode(&[0xA4], Charset::Koi8U), "\u{0454}"); // є
        assert_eq!(decode(&[0xA6], Charset::Koi8U), "\u{0456}"); // і
        assert_eq!(decode(&[0xA7], Charset::Koi8U), "\u{0457}"); // ї
        assert_eq!(decode(&[0xAD], Charset::Koi8U), "\u{0491}"); // ґ
        assert_eq!(decode(&[0xB4], Charset::Koi8U), "\u{0404}"); // Є
        assert_eq!(decode(&[0xB6], Charset::Koi8U), "\u{0406}"); // І
        assert_eq!(decode(&[0xB7], Charset::Koi8U), "\u{0407}"); // Ї
        assert_eq!(decode(&[0xBD], Charset::Koi8U), "\u{0490}"); // Ґ

        // Verified standard mappings identical to KOI8-R are preserved
        assert_eq!(
            decode(&[0xC1], Charset::Koi8U),
            decode(&[0xC1], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xE1], Charset::Koi8U),
            decode(&[0xE1], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xF2], Charset::Koi8U),
            decode(&[0xF2], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xF0], Charset::Koi8U),
            decode(&[0xF0], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xA3], Charset::Koi8U),
            decode(&[0xA3], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xB3], Charset::Koi8U),
            decode(&[0xB3], Charset::Koi8R)
        );
        assert_eq!(
            decode(&[0xBF], Charset::Koi8U),
            decode(&[0xBF], Charset::Koi8R)
        );
    }

    #[test]
    fn test_iso8859_13_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-13")),
            Charset::Iso8859_13
        );
        assert_eq!(
            sniff_charset(b"abc", Some("iso_8859_13")),
            Charset::Iso8859_13
        );
        assert_eq!(sniff_charset(b"abc", Some("l7")), Charset::Iso8859_13);
        assert_eq!(sniff_charset(b"abc", Some("latin7")), Charset::Iso8859_13);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-13\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_13);
    }

    #[test]
    fn test_iso8859_13_decode() {
        // ASCII passthrough
        assert_eq!(decode(b"abc 123", Charset::Iso8859_13), "abc 123");

        // Verified ISO-8859-13 / Latin-7 high half checkpoints
        assert_eq!(decode(&[0xA0], Charset::Iso8859_13), "\u{00A0}");
        assert_eq!(decode(&[0xA1], Charset::Iso8859_13), "\u{201D}");
        assert_eq!(decode(&[0xA5], Charset::Iso8859_13), "\u{201E}");
        assert_eq!(decode(&[0xA8], Charset::Iso8859_13), "\u{00D8}"); // Ø
        assert_eq!(decode(&[0xAA], Charset::Iso8859_13), "\u{0156}"); // Ŗ
        assert_eq!(decode(&[0xAF], Charset::Iso8859_13), "\u{00C6}"); // Æ
        assert_eq!(decode(&[0xB4], Charset::Iso8859_13), "\u{201C}");
        assert_eq!(decode(&[0xB8], Charset::Iso8859_13), "\u{00F8}"); // ø
        assert_eq!(decode(&[0xBA], Charset::Iso8859_13), "\u{0157}"); // ŗ
        assert_eq!(decode(&[0xBF], Charset::Iso8859_13), "\u{00E6}"); // æ
        assert_eq!(decode(&[0xC0], Charset::Iso8859_13), "\u{0104}"); // Ą
        assert_eq!(decode(&[0xC2], Charset::Iso8859_13), "\u{0100}"); // Ā
        assert_eq!(decode(&[0xD0], Charset::Iso8859_13), "\u{0160}"); // Š
        assert_eq!(decode(&[0xD9], Charset::Iso8859_13), "\u{0141}"); // Ł
        assert_eq!(decode(&[0xDE], Charset::Iso8859_13), "\u{017D}"); // Ž
        assert_eq!(decode(&[0xDF], Charset::Iso8859_13), "\u{00DF}"); // ß
        assert_eq!(decode(&[0xE0], Charset::Iso8859_13), "\u{0105}"); // ą
        assert_eq!(decode(&[0xF0], Charset::Iso8859_13), "\u{0161}"); // š
        assert_eq!(decode(&[0xFD], Charset::Iso8859_13), "\u{017C}"); // ż
        assert_eq!(decode(&[0xFE], Charset::Iso8859_13), "\u{017E}"); // ž
        assert_eq!(decode(&[0xFF], Charset::Iso8859_13), "\u{2019}"); // ’

        // Control characters map to their identity code points (C1 range)
        assert_eq!(decode(&[0x80], Charset::Iso8859_13), "\u{0080}");
        assert_eq!(decode(&[0x9F], Charset::Iso8859_13), "\u{009F}");
    }

    #[test]
    fn test_iso8859_10_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-10")),
            Charset::Iso8859_10
        );
        assert_eq!(
            sniff_charset(b"abc", Some("iso_8859_10")),
            Charset::Iso8859_10
        );
        assert_eq!(sniff_charset(b"abc", Some("l6")), Charset::Iso8859_10);
        assert_eq!(sniff_charset(b"abc", Some("latin6")), Charset::Iso8859_10);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-10\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_10);
    }

    #[test]
    fn test_iso8859_10_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Iso8859_10), "abc 123");

        // Verified ISO-8859-10 / Latin-6 high half checkpoints (anchors from prompt)
        assert_eq!(decode(&[0xA0], Charset::Iso8859_10), "\u{00A0}"); // NBSP (U+00A0)
        assert_eq!(decode(&[0xA1], Charset::Iso8859_10), "\u{0104}"); // Ą (U+0104)
        assert_eq!(decode(&[0xA2], Charset::Iso8859_10), "\u{0112}"); // Ē (U+0112)
        assert_eq!(decode(&[0xA3], Charset::Iso8859_10), "\u{0122}"); // Ģ (U+0122)
        assert_eq!(decode(&[0xA4], Charset::Iso8859_10), "\u{012A}"); // Ī (U+012A)
        assert_eq!(decode(&[0xC5], Charset::Iso8859_10), "\u{00C5}"); // Å (U+00C5)
        assert_eq!(decode(&[0xF0], Charset::Iso8859_10), "\u{00F0}"); // ð (U+00F0)
        assert_eq!(decode(&[0xFF], Charset::Iso8859_10), "\u{0138}"); // ĸ (U+0138)

        // Control characters map to their identity code points (C1 range)
        assert_eq!(decode(&[0x80], Charset::Iso8859_10), "\u{0080}");
        assert_eq!(decode(&[0x9F], Charset::Iso8859_10), "\u{009F}");
    }

    #[test]
    fn test_iso8859_16_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-16")),
            Charset::Iso8859_16
        );
        assert_eq!(
            sniff_charset(b"abc", Some("iso_8859_16")),
            Charset::Iso8859_16
        );
        assert_eq!(sniff_charset(b"abc", Some("l10")), Charset::Iso8859_16);
        assert_eq!(sniff_charset(b"abc", Some("latin10")), Charset::Iso8859_16);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-16\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_16);
    }

    #[test]
    fn test_iso8859_16_decode() {
        // Pure-ASCII round-trip (ASCII passthrough)
        assert_eq!(decode(b"abc 123", Charset::Iso8859_16), "abc 123");

        // Verified ISO-8859-16 / Latin-10 high half checkpoints (anchors from prompt)
        assert_eq!(decode(&[0xA0], Charset::Iso8859_16), "\u{00A0}"); // NBSP (U+00A0)
        assert_eq!(decode(&[0xA1], Charset::Iso8859_16), "\u{0104}"); // Ą (U+0104)
        assert_eq!(decode(&[0xA2], Charset::Iso8859_16), "\u{0105}"); // ą (U+0105)
        assert_eq!(decode(&[0xA3], Charset::Iso8859_16), "\u{0141}"); // Ł (U+0141)
        assert_eq!(decode(&[0xA4], Charset::Iso8859_16), "\u{20AC}"); // € (U+20AC)
        assert_eq!(decode(&[0xA5], Charset::Iso8859_16), "\u{201E}"); // „ (U+201E)
        assert_eq!(decode(&[0xA6], Charset::Iso8859_16), "\u{0160}"); // Š (U+0160)
        assert_eq!(decode(&[0xA8], Charset::Iso8859_16), "\u{0161}"); // š (U+0161)
        assert_eq!(decode(&[0xAA], Charset::Iso8859_16), "\u{0218}"); // Ș (U+0218)
        assert_eq!(decode(&[0xB5], Charset::Iso8859_16), "\u{201D}"); // ” (U+201D)
        assert_eq!(decode(&[0xBC], Charset::Iso8859_16), "\u{0152}"); // Œ (U+0152)

        // Control characters map to their identity code points (C1 range)
        assert_eq!(decode(&[0x80], Charset::Iso8859_16), "\u{0080}");
        assert_eq!(decode(&[0x9F], Charset::Iso8859_16), "\u{009F}");
    }

    #[test]
    fn test_iso8859_9_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-9")),
            Charset::Iso8859_9
        );
        assert_eq!(
            sniff_charset(b"abc", Some("ISO-8859-9")),
            Charset::Iso8859_9
        );
        assert_eq!(sniff_charset(b"abc", Some("latin5")), Charset::Iso8859_9);
        assert_eq!(sniff_charset(b"abc", Some("l5")), Charset::Iso8859_9);

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-9\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_9);
    }

    #[test]
    fn test_iso8859_9_decode() {
        // Pure-ASCII round-trip
        assert_eq!(decode(b"abc 123", Charset::Iso8859_9), "abc 123");

        // Verified ISO-8859-9 (Latin-5, Turkish) high half checkpoints (anchors from prompt)
        assert_eq!(
            decode(&[0xD0, 0xDD, 0xDE, 0xF0, 0xFD, 0xFE], Charset::Iso8859_9),
            "ĞİŞğış"
        );

        // identity or same-as-latin1 check
        assert_eq!(decode(&[0xE9], Charset::Iso8859_9), "é"); // é (U+00E9)
        assert_eq!(decode(&[0x80], Charset::Iso8859_9), "\u{0080}");
        assert_eq!(decode(&[0x9F], Charset::Iso8859_9), "\u{009F}");
    }

    #[test]
    fn test_iso8859_6_sniff() {
        assert_eq!(
            sniff_charset(b"abc", Some("iso-8859-6")),
            Charset::Iso8859_6
        );
        assert_eq!(sniff_charset(b"abc", Some("arabic")), Charset::Iso8859_6);
        assert_eq!(sniff_charset(b"abc", Some("asmo-708")), Charset::Iso8859_6);
        assert_eq!(sniff_charset(b"abc", Some("iso8859-6")), Charset::Iso8859_6);
        assert_eq!(sniff_charset(b"abc", Some("iso88596")), Charset::Iso8859_6);
        assert_eq!(
            sniff_charset(b"abc", Some("iso_8859-6")),
            Charset::Iso8859_6
        );
        assert_eq!(
            sniff_charset(b"abc", Some("iso_8859_6")),
            Charset::Iso8859_6
        );

        // Meta prescan check
        let html_meta = b"<html><head><meta charset=\"iso-8859-6\"></head></html>";
        assert_eq!(sniff_charset(html_meta, None), Charset::Iso8859_6);
    }

    #[test]
    fn test_iso8859_6_decode() {
        // Pure-ASCII round-trip
        assert_eq!(decode(b"abc 123", Charset::Iso8859_6), "abc 123");

        // Verified ISO-8859-6 (Latin/Arabic) defined codepoints
        assert_eq!(decode(&[0xA0], Charset::Iso8859_6), "\u{00A0}"); // NBSP
        assert_eq!(decode(&[0xA4], Charset::Iso8859_6), "\u{00A4}"); // CURRENCY
        assert_eq!(decode(&[0xAC], Charset::Iso8859_6), "\u{060C}"); // ARABIC COMMA
        assert_eq!(decode(&[0xAD], Charset::Iso8859_6), "\u{00AD}"); // SOFT HYPHEN
        assert_eq!(decode(&[0xBB], Charset::Iso8859_6), "\u{061B}"); // ARABIC SEMICOLON
        assert_eq!(decode(&[0xBF], Charset::Iso8859_6), "\u{061F}"); // ARABIC QUESTION MARK
        assert_eq!(decode(&[0xC1], Charset::Iso8859_6), "\u{0621}"); // ARABIC LETTER HAMZA
        assert_eq!(decode(&[0xC7], Charset::Iso8859_6), "\u{0627}"); // ARABIC LETTER ALEF
        assert_eq!(decode(&[0xE0], Charset::Iso8859_6), "\u{0640}"); // ARABIC TATWEEL
        assert_eq!(decode(&[0xF2], Charset::Iso8859_6), "\u{0652}"); // ARABIC SUKUN

        // Undefined codepoints map to replacement char U+FFFD
        assert_eq!(decode(&[0xA1], Charset::Iso8859_6), "\u{FFFD}");
        assert_eq!(decode(&[0xDB], Charset::Iso8859_6), "\u{FFFD}");
        assert_eq!(decode(&[0xFF], Charset::Iso8859_6), "\u{FFFD}");

        // C1 control characters map to their identity code points (C1 range)
        assert_eq!(decode(&[0x80], Charset::Iso8859_6), "\u{0080}");
        assert_eq!(decode(&[0x9F], Charset::Iso8859_6), "\u{009F}");
    }
}
