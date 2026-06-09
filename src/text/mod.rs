//! Text segmentation and line breaking.
//!
//! A simplified UAX#14: greedy line breaking at ASCII-whitespace boundaries.
//! Real Unicode line/word/grapheme segmentation is left as `// TODO(spec)`.
//! Layout's inline formatting uses this to wrap text into lines.

/// Splits `text` into words on runs of ASCII whitespace, dropping the
/// whitespace. Runs of whitespace are collapsed (no empty words).
// spec: https://infra.spec.whatwg.org/#ascii-whitespace
pub fn words(text: &str) -> Vec<&str> {
    text.split_ascii_whitespace().collect()
}

/// Collapses whitespace according to CSS `white-space: normal` rules.
///
/// Runs of collapsible whitespace are collapsed into a single space (U+0020),
/// and leading and trailing whitespaces are trimmed.
// spec: https://www.w3.org/TR/css-text-3/#white-space-collapsing
pub fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut pending_space = false;
    let mut has_non_whitespace = false;

    for c in s.chars() {
        if crate::ascii::is_html_whitespace(c) {
            if has_non_whitespace {
                pending_space = true;
            }
        } else {
            if pending_space {
                result.push(' ');
                pending_space = false;
            }
            result.push(c);
            has_non_whitespace = true;
        }
    }

    result
}

/// Greedily breaks `text` into lines no wider than `max_width`, measuring each
/// character with `measure`. Words are kept whole and separated by a single
/// space; a word that alone exceeds `max_width` is placed on its own line
/// (it overflows rather than being hard-split). Whitespace runs are collapsed.
// spec: https://www.w3.org/TR/css-text-3/#line-breaking (simplified, greedy)
pub fn break_lines(text: &str, max_width: f32, measure: impl Fn(char) -> f32) -> Vec<String> {
    let word_width = |w: &str| -> f32 { w.chars().map(&measure).sum() };
    let space_width = measure(' ');

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0.0_f32;

    for word in words(text) {
        let w = word_width(word);
        if current.is_empty() {
            // First word on the line always goes on (even if it overflows).
            current.push_str(word);
            current_width = w;
            continue;
        }
        // Width if we append " word" to the current line.
        let with_word = current_width + space_width + w;
        if with_word <= max_width {
            current.push(' ');
            current.push_str(word);
            current_width = with_word;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_width = w;
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Returns a list of byte offsets where a line break may occur (after spaces,
/// after hyphens, or between consecutive CJK characters).
///
/// Full UAX#14 tables are left as `// TODO(spec)`.
/// Never panics; offsets are guaranteed to be valid `char` boundaries.
// spec: https://www.unicode.org/reports/tr14/ (simplified, minimal class set)
pub fn break_opportunities(s: &str) -> Vec<usize> {
    // TODO(spec): Implement full UAX#14 class tables and state machine.
    let mut opportunities = Vec::new();
    let mut chars = s.char_indices().peekable();

    while let Some((_idx, c)) = chars.next() {
        let next_opt = chars.peek();

        if let Some(&(next_idx, next_c)) = next_opt {
            let break_after_space = is_space(c) && !is_space(next_c);
            let break_after_hyphen = is_hyphen(c) && !is_space(next_c) && !is_hyphen(next_c);
            let break_between_cjk = is_cjk(c) && is_cjk(next_c);

            if break_after_space || break_after_hyphen || break_between_cjk {
                opportunities.push(next_idx);
            }
        }
    }

    opportunities
}

fn is_space(c: char) -> bool {
    c.is_whitespace()
}

fn is_hyphen(c: char) -> bool {
    c == '-' || c == '\u{2010}'
}

fn is_cjk(c: char) -> bool {
    match c as u32 {
        0x4E00..=0x9FFF |    // CJK Unified Ideographs
        0x3400..=0x4DBF |    // CJK Unified Ideographs Extension A
        0x20000..=0x2A6DF |  // CJK Unified Ideographs Extension B
        0x2A700..=0x2B73F |  // CJK Unified Ideographs Extension C
        0x2B740..=0x2B81F |  // CJK Unified Ideographs Extension D
        0x2B820..=0x2CEAF |  // CJK Unified Ideographs Extension E
        0xF900..=0xFAFF |    // CJK Compatibility Ideographs
        0x2F800..=0x2FA1F |  // CJK Compatibility Ideographs Supplement
        0x3040..=0x309F |    // Hiragana
        0x30A0..=0x30FF |    // Katakana
        0xAC00..=0xD7AF |    // Hangul Syllables
        0x1100..=0x11FF |    // Hangul Jamo
        0x3130..=0x318F      // Hangul Compatibility Jamo
        => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each character is 10 wide, space included.
    fn fixed(_c: char) -> f32 {
        10.0
    }

    #[test]
    fn test_break_opportunities() {
        // "ab cd-ef" -> opportunities at index 3 (after ' ') and 6 (after '-')
        assert_eq!(break_opportunities("ab cd-ef"), vec![3, 6]);

        // Consecutive CJK characters ("日本語") -> opportunities between characters (3 and 6)
        assert_eq!(break_opportunities("日本語"), vec![3, 6]);

        // Empty string -> no opportunities
        assert_eq!(break_opportunities(""), Vec::<usize>::new());

        // Single character -> no opportunities
        assert_eq!(break_opportunities("a"), Vec::<usize>::new());
        assert_eq!(break_opportunities("日"), Vec::<usize>::new());

        // Multiple spaces -> break after the sequence of spaces
        assert_eq!(break_opportunities("ab   cd"), vec![5]);

        // Multiple hyphens -> break after the sequence of hyphens
        assert_eq!(break_opportunities("ab--cd"), vec![4]);

        // Space followed by hyphen -> break after space (at 3) and after hyphen (at 4)
        assert_eq!(break_opportunities("ab -cd"), vec![3, 4]);

        // CJK mixed with non-CJK (e.g. "日a") -> no opportunities between CJK and standard letters
        assert_eq!(break_opportunities("日a"), Vec::<usize>::new());
        assert_eq!(break_opportunities("a日"), Vec::<usize>::new());
    }

    #[test]
    fn words_collapse_whitespace() {
        assert_eq!(words("  a\t b\n c  "), vec!["a", "b", "c"]);
        assert!(words("   ").is_empty());
    }

    #[test]
    fn test_collapse_whitespace() {
        assert_eq!(collapse_whitespace("a  \n b "), "a b");
        assert_eq!(collapse_whitespace(""), "");
        assert_eq!(collapse_whitespace("   "), "");
        assert_eq!(collapse_whitespace("  abc  "), "abc");
        assert_eq!(
            collapse_whitespace(" \t\n\r\x0C a \t\n\r\x0C b \t\n\r\x0C "),
            "a b"
        );
        assert_eq!(collapse_whitespace("こんにちは  世界"), "こんにちは 世界");
        assert_eq!(collapse_whitespace("abc"), "abc");
    }

    #[test]
    fn short_text_stays_one_line() {
        // "ab cd" = 5 chars * 10 = 50 <= 100
        assert_eq!(break_lines("ab cd", 100.0, fixed), vec!["ab cd"]);
    }

    #[test]
    fn wraps_at_word_boundary() {
        // "aa" (20) + " " (10) + "bb" (20) = 50 fits in 50; adding " cc" (30) -> 80 > 50 -> wrap.
        // width 50: "aa bb" then "cc".
        assert_eq!(break_lines("aa bb cc", 50.0, fixed), vec!["aa bb", "cc"]);
    }

    #[test]
    fn one_word_per_line_when_narrow() {
        assert_eq!(break_lines("aa bb cc", 20.0, fixed), vec!["aa", "bb", "cc"]);
    }

    #[test]
    fn long_word_overflows_its_own_line() {
        // "wwwww" (50) alone exceeds 20 but is not split.
        assert_eq!(
            break_lines("x wwwww y", 20.0, fixed),
            vec!["x", "wwwww", "y"]
        );
    }

    #[test]
    fn non_ascii_does_not_panic() {
        let _ = break_lines("こんにちは world テスト", 30.0, fixed);
        let _ = words("a　b"); // ideographic space is NOT ascii whitespace
    }
}
