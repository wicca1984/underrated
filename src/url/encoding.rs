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
    /// Special query percent-encode set.
    SpecialQuery,
    /// Path percent-encode set.
    Path,
    /// Userinfo percent-encode set.
    Userinfo,
    /// Component percent-encode set.
    Component,
    /// application/x-www-form-urlencoded percent-encode set.
    FormUrlencoded,
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

/// Parses a query string (application/x-www-form-urlencoded) into key-value pairs.
///
// spec: <https://url.spec.whatwg.org/#urlencoded-parsing>
pub fn parse_query(input: &str) -> Vec<(String, String)> {
    let input = input.strip_prefix('?').unwrap_or(input);
    if input.is_empty() {
        return Vec::new();
    }
    let mut pairs = Vec::new();
    for component in input.split('&') {
        if component.is_empty() {
            continue;
        }
        let (key, val) = if let Some(pos) = component.find('=') {
            (&component[..pos], &component[pos + 1..])
        } else {
            (component, "")
        };
        let decoded_key = percent_decode(&key.replace('+', " "));
        let decoded_val = percent_decode(&val.replace('+', " "));

        // I-6: Safe parsing, no panic. UTF-8 lossy decoding.
        let key_str = String::from_utf8_lossy(&decoded_key).into_owned();
        let val_str = String::from_utf8_lossy(&decoded_val).into_owned();
        pairs.push((key_str, val_str));
    }
    pairs
}

/// Serializes key-value pairs into a query string (application/x-www-form-urlencoded).
///
// spec: <https://url.spec.whatwg.org/#urlencoded-serializing>
pub fn encode_query(pairs: &[(String, String)]) -> String {
    let mut output = String::new();
    for (key, val) in pairs {
        if !output.is_empty() {
            output.push('&');
        }
        output.push_str(&encode_form_urlencoded(key));
        output.push('=');
        output.push_str(&encode_form_urlencoded(val));
    }
    output
}

fn encode_form_urlencoded(input: &str) -> String {
    let mut output = String::new();
    for b in input.as_bytes() {
        if *b == b' ' {
            output.push('+');
        } else if b.is_ascii_alphanumeric() || matches!(*b, b'*' | b'-' | b'.' | b'_') {
            output.push(*b as char);
        } else {
            output.push_str(&format!("%{:02X}", b));
        }
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
        PercentEncodeSet::SpecialQuery => {
            if should_percent_encode(byte, PercentEncodeSet::Query) {
                return true;
            }
            byte == b'\''
        }
        PercentEncodeSet::Path => {
            // query set + ?, ^, `, {, }
            if should_percent_encode(byte, PercentEncodeSet::Query) {
                return true;
            }
            matches!(byte, b'?' | b'^' | b'`' | b'{' | b'}')
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
        PercentEncodeSet::FormUrlencoded => {
            // spec: <https://url.spec.whatwg.org/#application-x-www-form-urlencoded-percent-encode-set>
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'*' | b'-' | b'.' | b'_'))
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

    #[test]
    fn test_query_parse_and_encode() {
        let original = vec![
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "two words".to_string()),
            ("c".to_string(), "foo&bar=baz".to_string()),
            ("d".to_string(), "é".to_string()),
        ];
        let encoded = encode_query(&original);
        assert_eq!(encoded, "a=1&b=two+words&c=foo%26bar%3Dbaz&d=%C3%A9");

        let parsed = parse_query(&encoded);
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_query_parse_leading_question_mark() {
        let parsed = parse_query("?a=1&b=2");
        assert_eq!(
            parsed,
            vec![
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string())
            ]
        );
    }

    #[test]
    fn test_query_parse_empty_and_malformed() {
        assert!(parse_query("").is_empty());
        assert!(parse_query("?").is_empty());
        assert_eq!(
            parse_query("a&b=2&&c"),
            vec![
                ("a".to_string(), "".to_string()),
                ("b".to_string(), "2".to_string()),
                ("c".to_string(), "".to_string()),
            ]
        );
    }

    #[test]
    fn test_special_query_and_form_urlencoded_sets() {
        assert_eq!(
            percent_encode("a'b", PercentEncodeSet::SpecialQuery),
            "a%27b"
        );
        assert_eq!(percent_encode("a'b", PercentEncodeSet::Query), "a'b");

        assert_eq!(
            percent_encode("a*b-c.d_e~f!g'h(i)j", PercentEncodeSet::FormUrlencoded),
            "a*b-c.d_e%7Ef%21g%27h%28i%29j"
        );
    }
}
