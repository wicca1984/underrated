pub struct InputStream {
    data: Vec<char>,
    pos: usize,
    reconsume: bool,
    /// True when the most recent `next()` returned `None` (EOF). Used so that
    /// `reconsume()` after EOF re-yields EOF, not the last real code point.
    last_was_eof: bool,
}

impl InputStream {
    /// Decode UTF-8 (invalid sequences -> U+FFFD), then apply HTML input
    /// stream preprocessing: normalize "\r\n" and a lone "\r" to "\n".
    pub fn from_utf8(bytes: &[u8]) -> Self {
        let s = String::from_utf8_lossy(bytes);
        Self::new(&s)
    }

    /// Sniff charset, decode, and apply HTML input stream preprocessing.
    pub fn from_bytes(bytes: &[u8], transport_label: Option<&str>) -> Self {
        let charset = super::sniff_charset(bytes, transport_label);
        let mut offset = 0;
        if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) && charset == super::Charset::Utf8 {
            offset = 3;
        } else if (bytes.starts_with(&[0xFE, 0xFF]) && charset == super::Charset::Utf16Be)
            || (bytes.starts_with(&[0xFF, 0xFE]) && charset == super::Charset::Utf16Le)
        {
            offset = 2;
        }
        let decoded = super::decode(&bytes[offset..], charset);
        Self::new(&decoded)
    }

    fn new(s: &str) -> Self {
        let mut data = Vec::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\r' {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                data.push('\n');
            } else {
                data.push(c);
            }
        }
        Self {
            data,
            pos: 0,
            reconsume: false,
            last_was_eof: false,
        }
    }

    /// Next preprocessed code point, or None at EOF.
    // spec: "consume the next input character" (HTML §13.2.3.5). The domain term
    // is `next`; this is intentionally not `Iterator::next` (cf. `reconsume`/`peek`).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            // After EOF, reconsume must re-yield EOF, not the last real character.
            if self.last_was_eof {
                return None;
            }
            if self.pos > 0 {
                return Some(self.data[self.pos - 1]);
            }
            // TODO(spec): reconsume before the first next() is undefined here; in
            // practice next() is always called at least once before reconsume().
        }
        if self.pos < self.data.len() {
            let c = self.data[self.pos];
            self.pos += 1;
            self.last_was_eof = false;
            Some(c)
        } else {
            self.last_was_eof = true;
            None
        }
    }

    /// Re-yield the most recently consumed code point on the next `next()`.
    pub fn reconsume(&mut self) {
        self.reconsume = true;
    }

    /// Look at the next code point without consuming it.
    pub fn peek(&self) -> Option<char> {
        if self.reconsume {
            if self.last_was_eof {
                return None;
            }
            if self.pos > 0 {
                return Some(self.data[self.pos - 1]);
            }
        }
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_utf8() {
        let mut stream = InputStream::from_utf8(b"abc");
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.next(), Some('c'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_newline_normalization() {
        // \r\n -> \n
        let mut stream = InputStream::from_utf8(b"a\r\nb");
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), Some('\n'));
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.next(), None);

        // \r -> \n
        let mut stream = InputStream::from_utf8(b"a\rb");
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), Some('\n'));
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.next(), None);

        // \n stays \n
        let mut stream = InputStream::from_utf8(b"a\nb");
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), Some('\n'));
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.next(), None);

        // Complex case: \r\r\n\n\r
        let mut stream = InputStream::from_utf8(b"\r\r\n\n\r");
        assert_eq!(stream.next(), Some('\n')); // \r
        assert_eq!(stream.next(), Some('\n')); // \r\n
        assert_eq!(stream.next(), Some('\n')); // \n
        assert_eq!(stream.next(), Some('\n')); // \r
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_reconsume() {
        let mut stream = InputStream::from_utf8(b"abc");
        assert_eq!(stream.next(), Some('a'));
        stream.reconsume();
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), Some('b'));
        stream.reconsume();
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.next(), Some('c'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_reconsume_after_eof() {
        // After EOF, reconsume must re-yield EOF (None), not the last character.
        let mut stream = InputStream::from_utf8(b"a");
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), None); // EOF
        stream.reconsume();
        assert_eq!(stream.peek(), None); // not Some('a')
        assert_eq!(stream.next(), None); // not Some('a')
    }

    #[test]
    fn test_peek() {
        let mut stream = InputStream::from_utf8(b"abc");
        assert_eq!(stream.peek(), Some('a'));
        assert_eq!(stream.peek(), Some('a')); // peek doesn't advance
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.peek(), Some('b'));
        assert_eq!(stream.next(), Some('b'));
        assert_eq!(stream.peek(), Some('c'));
        assert_eq!(stream.next(), Some('c'));
        assert_eq!(stream.peek(), None);
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_invalid_utf8() {
        // 0xFF is invalid UTF-8
        let mut stream = InputStream::from_utf8(&[0xFF, b'a']);
        assert_eq!(stream.next(), Some('\u{FFFD}'));
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_from_bytes() {
        // UTF-16BE BOM
        let mut stream = InputStream::from_bytes(&[0xFE, 0xFF, 0x00, b'a'], None);
        assert_eq!(stream.next(), Some('a'));
        assert_eq!(stream.next(), None);

        // Windows-1252 via transport label
        let mut stream = InputStream::from_bytes(&[0x80], Some("windows-1252"));
        assert_eq!(stream.next(), Some('€'));
        assert_eq!(stream.next(), None);

        // Meta prescan
        let html = b"<html><meta charset='utf-8'><body>\xF0\x9F\x90\xA7</body></html>";
        let mut stream = InputStream::from_bytes(html, None);
        let mut s = String::new();
        while let Some(c) = stream.next() {
            s.push(c);
        }
        assert!(s.contains('🐧'));
    }
}
