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

#[cfg(test)]
mod tests {
    use super::*;

    // Each character is 10 wide, space included.
    fn fixed(_c: char) -> f32 {
        10.0
    }

    #[test]
    fn words_collapse_whitespace() {
        assert_eq!(words("  a\t b\n c  "), vec!["a", "b", "c"]);
        assert!(words("   ").is_empty());
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
        assert_eq!(break_lines("x wwwww y", 20.0, fixed), vec!["x", "wwwww", "y"]);
    }

    #[test]
    fn non_ascii_does_not_panic() {
        let _ = break_lines("こんにちは world テスト", 30.0, fixed);
        let _ = words("a　b"); // ideographic space is NOT ascii whitespace
    }
}
