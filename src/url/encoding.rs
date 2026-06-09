//! URL percent-encoding and decoding.
//!
// spec: <https://url.spec.whatwg.org/#percent-encoding>

/// Percent-encode sets as defined in the WHATWG URL standard.
///
// spec: <https://url.spec.whatwg.org/#percent-encoded-bytes>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PercentEncodeSet {
    /// Fragment percent-encode set.
    Fragment,
    /// Query percent-encode set.
    Query,
    /// Path percent-encode set.
    Path,
    /// Userinfo percent-encode set.
    Userinfo,
    /// Component percent-encode set.
    Component,
}

/// Percent-encodes the given input string using the specified encode set.
///
// spec: <https://url.spec.whatwg.org/#string-percent-encode-after-encoding>
pub fn percent_encode(input: &str, set: PercentEncodeSet) -> String {
    let mut output = String::new();
    for b in input.as_bytes() {
        if should_percent_encode(*b, set) {
            output.push_str(&format!("%{:02X}", b));
        } else {
            output.push(*b as char);
        }
    }
    output
}

/// Percent-decodes the given input string.
///
// spec: <https://url.spec.whatwg.org/#percent-decode>
pub fn percent_decode(input: &str) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut output = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let (Some(h1), Some(h2)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            )
        {
            output.push((h1 * 16 + h2) as u8);
            i += 3;
            continue;
        }
        output.push(bytes[i]);
        i += 1;
    }
    output
}

fn should_percent_encode(byte: u8, set: PercentEncodeSet) -> bool {
    // C0 control percent-encode set: U+0000 to U+001F and > U+007E
    if byte <= 0x1F || byte > 0x7E {
        return true;
    }

    match set {
        PercentEncodeSet::Fragment => {
            // C0 control + SPACE, ", <, >, `
            matches!(byte, b' ' | b'"' | b'<' | b'>' | b'`')
        }
        PercentEncodeSet::Query => {
            // C0 control + SPACE, ", #, <, >
            matches!(byte, b' ' | b'"' | b'#' | b'<' | b'>')
        }
        PercentEncodeSet::Path => {
            // query set + ?, {, }
            if should_percent_encode(byte, PercentEncodeSet::Query) {
                return true;
            }
            matches!(byte, b'?' | b'{' | b'}')
        }
        PercentEncodeSet::Userinfo => {
            // path set + /, :, ;, =, @, [, \, ], ^, |
            if should_percent_encode(byte, PercentEncodeSet::Path) {
                return true;
            }
            matches!(
                byte,
                b'/' | b':' | b';' | b'=' | b'@' | b'[' | b'\\' | b']' | b'^' | b'|'
            )
        }
        PercentEncodeSet::Component => {
            // userinfo set + $, %, &, +, ,
            if should_percent_encode(byte, PercentEncodeSet::Userinfo) {
                return true;
            }
            matches!(byte, b'$' | b'%' | b'&' | b'+' | b',')
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_basic() {
        assert_eq!(percent_encode("a b", PercentEncodeSet::Query), "a%20b");
        assert_eq!(percent_encode("a#b", PercentEncodeSet::Query), "a%23b");
        assert_eq!(percent_encode("a#b", PercentEncodeSet::Fragment), "a#b");
    }

    #[test]
    fn test_percent_encode_utf8() {
        // "é" is U+00E9, which is 0xC3 0xA9 in UTF-8
        assert_eq!(percent_encode("é", PercentEncodeSet::Query), "%C3%A9");
    }

    #[test]
    fn test_percent_decode_basic() {
        assert_eq!(percent_decode("a%20b"), b"a b");
        assert_eq!(percent_decode("a%23b"), b"a#b");
        assert_eq!(percent_decode("%C3%A9"), "é".as_bytes());
    }

    #[test]
    fn test_percent_decode_malformed() {
        assert_eq!(percent_decode("a%g1b"), b"a%g1b");
        assert_eq!(percent_decode("a%1"), b"a%1");
        assert_eq!(percent_decode("a%"), b"a%");
    }

    #[test]
    fn test_round_trip() {
        let input = "a b#c/d?e{f}é";
        let encoded = percent_encode(input, PercentEncodeSet::Component);
        let decoded = percent_decode(&encoded);
        assert_eq!(input.as_bytes(), decoded);
    }
}
