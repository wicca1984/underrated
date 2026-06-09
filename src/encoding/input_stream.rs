pub struct InputStream {
    data: Vec<char>,
    pos: usize,
    reconsume: bool,
}

impl InputStream {
    /// Decode UTF-8 (invalid sequences -> U+FFFD), then apply HTML input
    /// stream preprocessing: normalize "\r\n" and a lone "\r" to "\n".
    pub fn from_utf8(bytes: &[u8]) -> Self {
        let s = String::from_utf8_lossy(bytes);
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
        }
    }

    /// Next preprocessed code point, or None at EOF.
    // spec: "consume the next input character" (HTML §13.2.3.5). The domain term
    // is `next`; this is intentionally not `Iterator::next` (cf. `reconsume`/`peek`).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            if self.pos > 0 {
                return Some(self.data[self.pos - 1]);
            }
            // TODO(spec): Behavior of reconsume when pos is 0 is undefined in task,
            // but in practice next() should have been called at least once.
        }
        if self.pos < self.data.len() {
            let c = self.data[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    /// Re-yield the most recently consumed code point on the next `next()`.
    pub fn reconsume(&mut self) {
        self.reconsume = true;
    }

    /// Look at the next code point without consuming it.
    pub fn peek(&self) -> Option<char> {
        if self.reconsume && self.pos > 0 {
            return Some(self.data[self.pos - 1]);
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
}
