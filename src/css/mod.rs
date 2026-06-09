#![forbid(unsafe_code)]
#![allow(dead_code)]

#[derive(Debug, PartialEq, Clone)]
pub enum CssToken {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash(String),
    String(String),
    Number(f64),
    Percentage(f64),
    Dimension { value: f64, unit: String },
    Delim(char),
    Whitespace,
    Colon,
    Semicolon,
    Comma,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Cdo,
    Cdc,
    BadString,
    BadUrl,
    Url(String),
    Eof,
}

pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
    reconsume: bool,
}

impl CssTokenizer {
    pub fn new(input: &str) -> Self {
        // § 3.3. Preprocessing the input stream
        let mut processed = Vec::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    processed.push('\n');
                }
                '\x0C' => {
                    // \f (U+000C FORM FEED) -> \n
                    processed.push('\n');
                }
                '\x00' => {
                    // U+0000 NULL -> U+FFFD REPLACEMENT CHARACTER
                    processed.push('\u{FFFD}');
                }
                _ => processed.push(c),
            }
        }

        Self {
            input: processed,
            pos: 0,
            reconsume: false,
        }
    }

    fn consume(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            return self.input.get(self.pos - 1).copied();
        }
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn reconsume(&mut self) {
        self.reconsume = true;
    }

    fn peek(&self, n: usize) -> Option<char> {
        let actual_pos = if self.reconsume {
            (self.pos as isize - 1 + n as isize) as usize
        } else {
            self.pos + n
        };
        self.input.get(actual_pos).copied()
    }

    pub fn next_token(&mut self) -> CssToken {
        // § 4.3.1. Consume a token
        self.consume_comments();

        match self.consume() {
            Some(c) if is_whitespace(c) => {
                while let Some(c) = self.peek(0) {
                    if is_whitespace(c) {
                        self.consume();
                    } else {
                        break;
                    }
                }
                CssToken::Whitespace
            }
            Some('"') => self.consume_string_token('"'),
            Some('#') => {
                let c1 = self.peek(0);
                let c2 = self.peek(1);
                if c1.is_some_and(is_name) || self.is_valid_escape(c1, c2) {
                    let is_id = self.starts_ident(Some(c1.unwrap_or('\0')), c2, self.peek(2));
                    let name = self.consume_name();
                    if is_id {
                        // TODO(spec): Hash token has a type "id" or "unrestricted".
                        // SPEC S-5 only has Hash(String).
                        CssToken::Hash(name)
                    } else {
                        CssToken::Hash(name)
                    }
                } else {
                    CssToken::Delim('#')
                }
            }
            Some('\'') => self.consume_string_token('\''),
            Some('(') => CssToken::LeftParen,
            Some(')') => CssToken::RightParen,
            Some('+') => {
                if self.starts_number(Some('+'), self.peek(0), self.peek(1)) {
                    self.reconsume();
                    self.consume_numeric_token()
                } else {
                    CssToken::Delim('+')
                }
            }
            Some(',') => CssToken::Comma,
            Some('-') => {
                if self.starts_number(Some('-'), self.peek(0), self.peek(1)) {
                    self.reconsume();
                    self.consume_numeric_token()
                } else if self.peek(0) == Some('-') && self.peek(1) == Some('>') {
                    self.consume();
                    self.consume();
                    CssToken::Cdc
                } else if self.starts_ident(Some('-'), self.peek(0), self.peek(1)) {
                    self.reconsume();
                    self.consume_ident_like_token()
                } else {
                    CssToken::Delim('-')
                }
            }
            Some('.') => {
                if self.starts_number(Some('.'), self.peek(0), self.peek(1)) {
                    self.reconsume();
                    self.consume_numeric_token()
                } else {
                    CssToken::Delim('.')
                }
            }
            Some(':') => CssToken::Colon,
            Some(';') => CssToken::Semicolon,
            Some('<') => {
                if self.peek(0) == Some('!')
                    && self.peek(1) == Some('-')
                    && self.peek(2) == Some('-')
                {
                    self.consume();
                    self.consume();
                    self.consume();
                    CssToken::Cdo
                } else {
                    CssToken::Delim('<')
                }
            }
            Some('@') => {
                if self.starts_ident(self.peek(0), self.peek(1), self.peek(2)) {
                    let name = self.consume_name();
                    CssToken::AtKeyword(name)
                } else {
                    CssToken::Delim('@')
                }
            }
            Some('[') => CssToken::LeftBracket,
            Some('\\') => {
                if self.is_valid_escape(Some('\\'), self.peek(0)) {
                    self.reconsume();
                    self.consume_ident_like_token()
                } else {
                    // parse error
                    CssToken::Delim('\\')
                }
            }
            Some(']') => CssToken::RightBracket,
            Some('{') => CssToken::LeftBrace,
            Some('}') => CssToken::RightBrace,
            Some(c) if is_digit(c) => {
                self.reconsume();
                self.consume_numeric_token()
            }
            Some(c) if is_name_start(c) => {
                self.reconsume();
                self.consume_ident_like_token()
            }
            Some(c) => CssToken::Delim(c),
            None => CssToken::Eof,
        }
    }

    fn consume_comments(&mut self) {
        // § 4.3.2. Consume comments
        while self.peek(0) == Some('/') && self.peek(1) == Some('*') {
            self.consume();
            self.consume();
            loop {
                match self.consume() {
                    Some('*') if self.peek(0) == Some('/') => {
                        self.consume();
                        break;
                    }
                    Some(_) => {}
                    None => break, // parse error
                }
            }
        }
    }

    fn consume_numeric_token(&mut self) -> CssToken {
        // § 4.3.3. Consume a numeric token
        let number = self.consume_number();
        if self.starts_ident(self.peek(0), self.peek(1), self.peek(2)) {
            let unit = self.consume_name();
            CssToken::Dimension {
                value: number,
                unit,
            }
        } else if self.peek(0) == Some('%') {
            self.consume();
            CssToken::Percentage(number)
        } else {
            CssToken::Number(number)
        }
    }

    fn consume_ident_like_token(&mut self) -> CssToken {
        // § 4.3.4. Consume an ident-like token
        let name = self.consume_name();
        if name.eq_ignore_ascii_case("url") && self.peek(0) == Some('(') {
            self.consume();
            // § 4.3.4 says:
            // "While the next two input code points are whitespace, consume the next input code point."
            while let Some(c) = self.peek(0) {
                if is_whitespace(c) {
                    if let Some(c2) = self.peek(1) {
                        if is_whitespace(c2) {
                            self.consume();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
            // "If the next input code point is U+0022 QUOTATION MARK (") or U+0027 APOSTROPHE ('), return the <function-token> with its value set to name."
            if self.peek(0) == Some('"') || self.peek(0) == Some('\'') {
                CssToken::Function(name)
            } else {
                self.consume_url_token()
            }
        } else if self.peek(0) == Some('(') {
            self.consume();
            CssToken::Function(name)
        } else {
            CssToken::Ident(name)
        }
    }

    fn consume_string_token(&mut self, ending: char) -> CssToken {
        // § 4.3.5. Consume a string token
        let mut string = String::new();
        loop {
            match self.consume() {
                Some(c) if c == ending => return CssToken::String(string),
                None => return CssToken::String(string), // parse error
                Some('\n') => {
                    self.reconsume();
                    return CssToken::BadString;
                }
                Some('\\') => {
                    if let Some(c) = self.peek(0) {
                        if c == '\n' {
                            self.consume();
                        } else {
                            string.push(self.consume_escaped_code_point());
                        }
                    } else {
                        // ignore backslash at EOF
                    }
                }
                Some(c) => string.push(c),
            }
        }
    }

    fn consume_url_token(&mut self) -> CssToken {
        // § 4.3.6. Consume a url-token
        let mut string = String::new();
        // Consume as much whitespace as possible.
        while let Some(c) = self.peek(0) {
            if is_whitespace(c) {
                self.consume();
            } else {
                break;
            }
        }
        loop {
            match self.consume() {
                Some(')') => return CssToken::Url(string),
                None => return CssToken::Url(string), // parse error
                Some(c) if is_whitespace(c) => {
                    while let Some(c) = self.peek(0) {
                        if is_whitespace(c) {
                            self.consume();
                        } else {
                            break;
                        }
                    }
                    if self.peek(0) == Some(')') || self.peek(0).is_none() {
                        if self.peek(0).is_some() {
                            self.consume();
                        }
                        return CssToken::Url(string);
                    } else {
                        self.consume_remnants_of_bad_url();
                        return CssToken::BadUrl;
                    }
                }
                Some('"') | Some('\'') | Some('(') => {
                    // parse error
                    self.consume_remnants_of_bad_url();
                    return CssToken::BadUrl;
                }
                Some(c) if is_non_printable(c) => {
                    // parse error
                    self.consume_remnants_of_bad_url();
                    return CssToken::BadUrl;
                }
                Some('\\') => {
                    if self.is_valid_escape(Some('\\'), self.peek(0)) {
                        string.push(self.consume_escaped_code_point());
                    } else {
                        // parse error
                        self.consume_remnants_of_bad_url();
                        return CssToken::BadUrl;
                    }
                }
                Some(c) => string.push(c),
            }
        }
    }

    fn consume_escaped_code_point(&mut self) -> char {
        // § 4.3.7. Consume an escaped code point
        match self.consume() {
            Some(c) if is_hex_digit(c) => {
                let mut hex = String::new();
                hex.push(c);
                for _ in 0..5 {
                    if let Some(next) = self.peek(0) {
                        if is_hex_digit(next) {
                            if let Some(consumed) = self.consume() {
                                hex.push(consumed);
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if self.peek(0).is_some_and(is_whitespace) {
                    self.consume();
                }
                let val = u32::from_str_radix(&hex, 16).unwrap_or(0xFFFD);
                if val == 0 || (0xD800..=0xDFFF).contains(&val) || val > 0x10FFFF {
                    '\u{FFFD}'
                } else {
                    char::from_u32(val).unwrap_or('\u{FFFD}')
                }
            }
            Some(c) => c,
            None => '\u{FFFD}',
        }
    }

    fn is_valid_escape(&self, c1: Option<char>, c2: Option<char>) -> bool {
        // § 4.3.8. Check if two code points are a valid escape
        if c1 != Some('\\') {
            return false;
        }
        if let Some(c2_val) = c2 {
            c2_val != '\n'
        } else {
            false
        }
    }

    fn starts_ident(&self, c1: Option<char>, c2: Option<char>, c3: Option<char>) -> bool {
        // § 4.3.9. Check if three code points would start an identifier
        match c1 {
            Some('-') => c2.is_some_and(|c| {
                is_name_start(c) || c == '-' || self.is_valid_escape(Some('-'), c3)
            }),
            Some(c) if is_name_start(c) => true,
            Some('\\') => self.is_valid_escape(Some('\\'), c2),
            _ => false,
        }
    }

    fn starts_number(&self, c1: Option<char>, c2: Option<char>, c3: Option<char>) -> bool {
        // § 4.3.10. Check if three code points would start a number
        match c1 {
            Some('+') | Some('-') => {
                if let Some(c2_val) = c2 {
                    if is_digit(c2_val) {
                        return true;
                    }
                    if c2_val == '.' && c3.is_some_and(is_digit) {
                        return true;
                    }
                }
                false
            }
            Some('.') => c2.is_some_and(is_digit),
            Some(c) if is_digit(c) => true,
            _ => false,
        }
    }

    fn consume_name(&mut self) -> String {
        // § 4.3.11. Consume a name
        let mut result = String::new();
        loop {
            let c1 = self.peek(0);
            let c2 = self.peek(1);
            match c1 {
                Some(c) if is_name(c) => {
                    if let Some(consumed) = self.consume() {
                        result.push(consumed);
                    }
                }
                Some('\\') if self.is_valid_escape(Some('\\'), c2) => {
                    self.consume();
                    result.push(self.consume_escaped_code_point());
                }
                _ => break,
            }
        }
        result
    }

    fn consume_number(&mut self) -> f64 {
        // § 4.3.12. Consume a number
        // This is a bit simplified but follows the spec logic
        let mut repr = String::new();
        if let Some(c) = self.peek(0).filter(|&c| c == '+' || c == '-') {
            self.consume();
            repr.push(c);
        }
        while let Some(c) = self.peek(0) {
            if is_digit(c) {
                if let Some(consumed) = self.consume() {
                    repr.push(consumed);
                }
            } else {
                break;
            }
        }
        if self.peek(0) == Some('.') && self.peek(1).is_some_and(is_digit) {
            if let Some(consumed) = self.consume() {
                repr.push(consumed);
            }
            if let Some(consumed) = self.consume() {
                repr.push(consumed);
            }
            while let Some(c) = self.peek(0) {
                if is_digit(c) {
                    if let Some(consumed) = self.consume() {
                        repr.push(consumed);
                    }
                } else {
                    break;
                }
            }
        }
        if self.peek(0) == Some('e') || self.peek(0) == Some('E') {
            let next1 = self.peek(1);
            let next2 = self.peek(2);
            let has_sign = next1 == Some('+') || next1 == Some('-');
            let exp_digit = if has_sign { next2 } else { next1 };
            if exp_digit.is_some_and(is_digit) {
                if let Some(consumed) = self.consume() {
                    repr.push(consumed);
                }
                if let Some(c) = self.peek(0).filter(|_| has_sign) {
                    self.consume();
                    repr.push(c);
                }
                while let Some(c) = self.peek(0) {
                    if is_digit(c) {
                        if let Some(consumed) = self.consume() {
                            repr.push(consumed);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        repr.parse().unwrap_or(0.0)
    }

    fn consume_remnants_of_bad_url(&mut self) {
        // § 4.3.13. Consume the remnants of a bad url
        loop {
            match self.consume() {
                Some(')') | None => break,
                Some('\\') if self.is_valid_escape(Some('\\'), self.peek(0)) => {
                    self.consume_escaped_code_point();
                }
                _ => {}
            }
        }
    }
}

fn is_whitespace(c: char) -> bool {
    c == '\n' || c == '\t' || c == ' '
}

fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_uppercase(c: char) -> bool {
    c.is_ascii_uppercase()
}

fn is_lowercase(c: char) -> bool {
    c.is_ascii_lowercase()
}

fn is_letter(c: char) -> bool {
    is_uppercase(c) || is_lowercase(c)
}

fn is_non_ascii(c: char) -> bool {
    c >= '\u{0080}'
}

fn is_name_start(c: char) -> bool {
    is_letter(c) || is_non_ascii(c) || c == '_'
}

fn is_name(c: char) -> bool {
    is_name_start(c) || is_digit(c) || c == '-'
}

fn is_non_printable(c: char) -> bool {
    ('\u{0000}'..='\u{0008}').contains(&c)
        || c == '\u{000B}'
        || ('\u{000E}'..='\u{001F}').contains(&c)
        || c == '\u{007F}'
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessing() {
        let t = CssTokenizer::new("a\r\nb\rc\u{000c}d\0e");
        assert_eq!(
            t.input,
            vec!['a', '\n', 'b', '\n', 'c', '\n', 'd', '\u{FFFD}', 'e']
        );
    }

    fn tokenize(input: &str) -> Vec<CssToken> {
        let mut t = CssTokenizer::new(input);
        let mut tokens = Vec::new();
        loop {
            let token = t.next_token();
            if token == CssToken::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    #[test]
    fn test_whitespace() {
        assert_eq!(tokenize("  \n\t "), vec![CssToken::Whitespace]);
    }

    #[test]
    fn test_string() {
        assert_eq!(
            tokenize("\"double\""),
            vec![CssToken::String("double".to_string())]
        );
        assert_eq!(
            tokenize("'single'"),
            vec![CssToken::String("single".to_string())]
        );
        assert_eq!(
            tokenize("\"with \\\" escape\""),
            vec![CssToken::String("with \" escape".to_string())]
        );
        // § 4.3.5: EOF in string returns the string
        assert_eq!(
            tokenize("\"eof string"),
            vec![CssToken::String("eof string".to_string())]
        );
        // Newline in string returns BadString
        assert_eq!(
            tokenize("\"newline\nstring\""),
            vec![
                CssToken::BadString,
                CssToken::Whitespace,
                CssToken::Ident("string".to_string()),
                CssToken::String("".to_string())
            ]
        );
    }

    #[test]
    fn test_hash() {
        assert_eq!(tokenize("#abc"), vec![CssToken::Hash("abc".to_string())]);
        assert_eq!(tokenize("#123"), vec![CssToken::Hash("123".to_string())]);
    }

    #[test]
    fn test_numeric() {
        assert_eq!(tokenize("123"), vec![CssToken::Number(123.0)]);
        assert_eq!(tokenize("123.456"), vec![CssToken::Number(123.456)]);
        assert_eq!(tokenize(".456"), vec![CssToken::Number(0.456)]);
        assert_eq!(tokenize("123e2"), vec![CssToken::Number(12300.0)]);
        assert_eq!(tokenize("+123"), vec![CssToken::Number(123.0)]);
        assert_eq!(tokenize("-123"), vec![CssToken::Number(-123.0)]);

        assert_eq!(tokenize("10%"), vec![CssToken::Percentage(10.0)]);
        assert_eq!(
            tokenize("12px"),
            vec![CssToken::Dimension {
                value: 12.0,
                unit: "px".to_string()
            }]
        );
    }

    #[test]
    fn test_ident_like() {
        assert_eq!(tokenize("auto"), vec![CssToken::Ident("auto".to_string())]);
        assert_eq!(
            tokenize("--variable"),
            vec![CssToken::Ident("--variable".to_string())]
        );
        assert_eq!(
            tokenize("rgb("),
            vec![CssToken::Function("rgb".to_string())]
        );
        assert_eq!(
            tokenize("@media"),
            vec![CssToken::AtKeyword("media".to_string())]
        );
    }

    #[test]
    fn test_url_no_quotes() {
        assert_eq!(
            tokenize("url(http://example.com)"),
            vec![CssToken::Url("http://example.com".to_string())]
        );
    }

    #[test]
    fn test_url_with_quotes() {
        // § 4.3.4: url("...") returns Function("url") + String("...")
        assert_eq!(
            tokenize("url(\"http://example.com\")"),
            vec![
                CssToken::Function("url".to_string()),
                CssToken::String("http://example.com".to_string()),
                CssToken::RightParen,
            ]
        );
    }

    #[test]
    fn test_comments() {
        assert_eq!(
            tokenize("/* comment */auto"),
            vec![CssToken::Ident("auto".to_string())]
        );
        assert_eq!(tokenize("/**/"), vec![]);
    }

    #[test]
    fn test_cdo_cdc() {
        assert_eq!(tokenize("<!--"), vec![CssToken::Cdo]);
        assert_eq!(tokenize("-->"), vec![CssToken::Cdc]);
    }

    #[test]
    fn test_single_chars() {
        assert_eq!(
            tokenize(":;,{}()[]"),
            vec![
                CssToken::Colon,
                CssToken::Semicolon,
                CssToken::Comma,
                CssToken::LeftBrace,
                CssToken::RightBrace,
                CssToken::LeftParen,
                CssToken::RightParen,
                CssToken::LeftBracket,
                CssToken::RightBracket,
            ]
        );
    }

    #[test]
    fn test_delims() {
        assert_eq!(
            tokenize("&|"),
            vec![CssToken::Delim('&'), CssToken::Delim('|')]
        );
    }
}
