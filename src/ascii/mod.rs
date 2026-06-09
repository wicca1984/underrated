//! ASCII and HTML-whitespace utilities.

/// Returns true if two strings are equal ignoring ASCII case.
/// Non-ASCII characters are compared exactly and NOT case-folded.
pub fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Returns a new string with all ASCII uppercase characters converted to lowercase.
/// Non-ASCII characters are left unchanged.
pub fn to_ascii_lowercase(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// Returns true if the character is an HTML "ASCII whitespace".
// spec: https://infra.spec.whatwg.org/#ascii-whitespace
pub fn is_html_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\x0C' | '\r' | ' ')
}

/// Returns true if the character is an ASCII alphabetic character (a-z, A-Z).
pub fn is_ascii_alpha(c: char) -> bool {
    c.is_ascii_alphabetic()
}

/// Returns true if the character is an ASCII alphanumeric character (a-z, A-Z, 0-9).
pub fn is_ascii_alphanumeric(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_ignore_ascii_case() {
        assert!(eq_ignore_ascii_case("abc", "ABC"));
        assert!(eq_ignore_ascii_case("abc", "abc"));
        assert!(eq_ignore_ascii_case("a1B", "A1b"));
        assert!(!eq_ignore_ascii_case("abc", "abd"));
        // Non-ASCII should not be case-folded
        assert!(!eq_ignore_ascii_case("Σ", "σ"));
        assert!(eq_ignore_ascii_case("Σ", "Σ"));
    }

    #[test]
    fn test_to_ascii_lowercase() {
        assert_eq!(to_ascii_lowercase("ABC"), "abc");
        assert_eq!(to_ascii_lowercase("abc"), "abc");
        assert_eq!(to_ascii_lowercase("A1b"), "a1b");
        // Non-ASCII should be left unchanged
        assert_eq!(to_ascii_lowercase("Σ"), "Σ");
        assert_eq!(to_ascii_lowercase("Hello World! Σ"), "hello world! Σ");
    }

    #[test]
    fn test_is_html_whitespace() {
        assert!(is_html_whitespace('\t')); // U+0009 TAB
        assert!(is_html_whitespace('\n')); // U+000A LF
        assert!(is_html_whitespace('\x0C')); // U+000C FF
        assert!(is_html_whitespace('\r')); // U+000D CR
        assert!(is_html_whitespace(' ')); // U+0020 SPACE

        assert!(!is_html_whitespace('\x0B')); // Vertical tab
        assert!(!is_html_whitespace('\u{00A0}')); // Non-breaking space
        assert!(!is_html_whitespace('a'));
        assert!(!is_html_whitespace('\0'));
    }

    #[test]
    fn test_is_ascii_alpha() {
        assert!(is_ascii_alpha('a'));
        assert!(is_ascii_alpha('z'));
        assert!(is_ascii_alpha('A'));
        assert!(is_ascii_alpha('Z'));
        assert!(!is_ascii_alpha('0'));
        assert!(!is_ascii_alpha('9'));
        assert!(!is_ascii_alpha(' '));
        assert!(!is_ascii_alpha('Σ'));
    }

    #[test]
    fn test_is_ascii_alphanumeric() {
        assert!(is_ascii_alphanumeric('a'));
        assert!(is_ascii_alphanumeric('z'));
        assert!(is_ascii_alphanumeric('A'));
        assert!(is_ascii_alphanumeric('Z'));
        assert!(is_ascii_alphanumeric('0'));
        assert!(is_ascii_alphanumeric('9'));
        assert!(!is_ascii_alphanumeric(' '));
        assert!(!is_ascii_alphanumeric('Σ'));
    }
}
