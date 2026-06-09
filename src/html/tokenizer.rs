use crate::encoding::InputStream;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    StartTag {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
        attrs: Vec<(String, String)>,
        self_closing: bool,
    },
    Comment(String),
    Character(char),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub code: String,
}

pub struct Tokenizer {
    input: InputStream,
    errors: Vec<ParseError>,
    state: State,
    current_token: Option<Token>,
    current_attribute: Option<(String, String)>,
    token_buffer: std::collections::VecDeque<Token>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    // Non-scoped states
    MarkupDeclarationOpen,
}

impl Tokenizer {
    pub fn new(input: InputStream) -> Self {
        Self {
            input,
            errors: Vec::new(),
            state: State::Data,
            current_token: None,
            current_attribute: None,
            token_buffer: std::collections::VecDeque::new(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        if let Some(token) = self.token_buffer.pop_front() {
            return token;
        }

        loop {
            let c = self.input.next();

            match self.state {
                State::Data => {
                    // // spec: §13.2.5.1 Data state
                    match c {
                        Some('&') => {
                            // TODO(spec): Character reference
                            return Token::Character('&');
                        }
                        Some('<') => {
                            self.state = State::TagOpen;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\0');
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                        None => {
                            return Token::Eof;
                        }
                    }
                }
                State::MarkupDeclarationOpen => {
                    // // spec: §13.2.5.43 Markup declaration open state
                    // TODO(spec): Implement DOCTYPE and Comment properly.
                    while let Some(c_decl) = self.input.next() {
                        if c_decl == '>' {
                            break;
                        }
                    }
                    self.state = State::Data;
                    return self.next_token();
                }
                State::TagOpen => {
                    // // spec: §13.2.5.6 Tag open state
                    match c {
                        Some('!') => {
                            self.state = State::MarkupDeclarationOpen;
                        }
                        Some('/') => {
                            self.state = State::EndTagOpen;
                        }
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::StartTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::TagName;
                            self.input.reconsume();
                        }
                        Some('?') => {
                            self.emit_error("unexpected-question-mark-instead-of-tag-name");
                            // TODO(spec): Bogus comment
                            return Token::Character('<');
                        }
                        Some(_c) => {
                            self.emit_error("invalid-first-character-of-tag-name");
                            self.state = State::Data;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                        None => {
                            self.emit_error("eof-before-tag-name");
                            self.state = State::Data;
                            return Token::Character('<');
                        }
                    }
                }
                State::EndTagOpen => {
                    // // spec: §13.2.5.7 End tag open state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::EndTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::TagName;
                            self.input.reconsume();
                        }
                        Some('>') => {
                            self.emit_error("missing-end-tag-name");
                            self.state = State::Data;
                        }
                        None => {
                            self.emit_error("eof-before-tag-name");
                            self.state = State::Data;
                            self.token_buffer.push_back(Token::Character('/'));
                            return Token::Character('<');
                        }
                        Some(_c) => {
                            self.emit_error("invalid-first-character-of-tag-name");
                            // TODO(spec): Bogus comment
                            return Token::Character('<');
                        }
                    }
                }
                State::TagName => {
                    // // spec: §13.2.5.8 Tag name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::StartTag { name, .. } | Token::EndTag { name, .. }) =
                                &mut self.current_token
                            {
                                name.push(c_val.to_ascii_lowercase());
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::StartTag { name, .. } | Token::EndTag { name, .. }) =
                                &mut self.current_token
                            {
                                name.push('\u{FFFD}');
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::StartTag { name, .. } | Token::EndTag { name, .. }) =
                                &mut self.current_token
                            {
                                name.push(c_val);
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                    }
                }
                State::BeforeAttributeName => {
                    // // spec: §13.2.5.32 Before attribute name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('/') | Some('>') | None => {
                            self.state = State::AfterAttributeName;
                            self.input.reconsume();
                        }
                        Some('=') => {
                            self.emit_error("unexpected-equals-sign-before-attribute-name");
                            self.current_attribute = Some(("=".to_string(), String::new()));
                            self.state = State::AttributeName;
                        }
                        Some(_) => {
                            self.current_attribute = Some((String::new(), String::new()));
                            self.state = State::AttributeName;
                            self.input.reconsume();
                        }
                    }
                }
                State::AttributeName => {
                    // // spec: §13.2.5.33 Attribute name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') | Some('/')
                        | Some('>') | None => {
                            self.state = State::AfterAttributeName;
                            self.input.reconsume();
                        }
                        Some('=') => {
                            self.state = State::BeforeAttributeValue;
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(attr) = &mut self.current_attribute {
                                attr.0.push(c_val.to_ascii_lowercase());
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.0.push('\u{FFFD}');
                            }
                        }
                        Some(c_val @ '"') | Some(c_val @ '\'') | Some(c_val @ '<') => {
                            self.emit_error("unexpected-character-in-attribute-name");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.0.push(c_val);
                            }
                        }
                        Some(c_val) => {
                            if let Some(attr) = &mut self.current_attribute {
                                attr.0.push(c_val);
                            }
                        }
                    }
                }
                State::AfterAttributeName => {
                    // // spec: §13.2.5.34 After attribute name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('/') => {
                            self.emit_current_attribute();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('=') => {
                            self.state = State::BeforeAttributeValue;
                        }
                        Some('>') => {
                            self.emit_current_attribute();
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(_) => {
                            self.emit_current_attribute();
                            self.current_attribute = Some((String::new(), String::new()));
                            self.state = State::AttributeName;
                            self.input.reconsume();
                        }
                    }
                }
                State::BeforeAttributeValue => {
                    // // spec: §13.2.5.35 Before attribute value state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('"') => {
                            self.state = State::AttributeValueDoubleQuoted;
                        }
                        Some('\'') => {
                            self.state = State::AttributeValueSingleQuoted;
                        }
                        Some('>') => {
                            self.emit_error("missing-attribute-value");
                            self.emit_current_attribute();
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some(_) => {
                            self.state = State::AttributeValueUnquoted;
                            self.input.reconsume();
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                    }
                }
                State::AttributeValueDoubleQuoted => {
                    // // spec: §13.2.5.36 Attribute value (double-quoted) state
                    match c {
                        Some('"') => {
                            self.state = State::AfterAttributeValueQuoted;
                        }
                        Some('&') => {
                            // TODO(spec): Character reference
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('&');
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('\u{FFFD}');
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push(c_val);
                            }
                        }
                    }
                }
                State::AttributeValueSingleQuoted => {
                    // // spec: §13.2.5.37 Attribute value (single-quoted) state
                    match c {
                        Some('\'') => {
                            self.state = State::AfterAttributeValueQuoted;
                        }
                        Some('&') => {
                            // TODO(spec): Character reference
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('&');
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('\u{FFFD}');
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push(c_val);
                            }
                        }
                    }
                }
                State::AttributeValueUnquoted => {
                    // // spec: §13.2.5.38 Attribute value (unquoted) state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.emit_current_attribute();
                            self.state = State::BeforeAttributeName;
                        }
                        Some('&') => {
                            // TODO(spec): Character reference
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('&');
                            }
                        }
                        Some('>') => {
                            self.emit_current_attribute();
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push('\u{FFFD}');
                            }
                        }
                        Some(c_val @ '"') | Some(c_val @ '\'') | Some(c_val @ '<')
                        | Some(c_val @ '=') | Some(c_val @ '`') => {
                            self.emit_error("unexpected-character-in-unquoted-attribute-value");
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push(c_val);
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            if let Some(attr) = &mut self.current_attribute {
                                attr.1.push(c_val);
                            }
                        }
                    }
                }
                State::AfterAttributeValueQuoted => {
                    // // spec: §13.2.5.39 After attribute value (quoted) state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.emit_current_attribute();
                            self.state = State::BeforeAttributeName;
                        }
                        Some('/') => {
                            self.emit_current_attribute();
                            self.state = State::SelfClosingStartTag;
                        }
                        Some('>') => {
                            self.emit_current_attribute();
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(_) => {
                            self.emit_error("missing-whitespace-between-attributes");
                            self.emit_current_attribute();
                            self.state = State::BeforeAttributeName;
                            self.input.reconsume();
                        }
                    }
                }
                State::SelfClosingStartTag => {
                    // // spec: §13.2.5.40 Self-closing start tag state
                    match c {
                        Some('>') => {
                            if let Some(
                                Token::StartTag {
                                    ref mut self_closing,
                                    ..
                                }
                                | Token::EndTag {
                                    ref mut self_closing,
                                    ..
                                },
                            ) = self.current_token
                            {
                                *self_closing = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-tag");
                            self.state = State::Data;
                            return Token::Eof;
                        }
                        Some(_) => {
                            self.emit_error("unexpected-solidus-in-tag");
                            self.state = State::BeforeAttributeName;
                            self.input.reconsume();
                        }
                    }
                }
            }
        }
    }

    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    fn emit_error(&mut self, code: &str) {
        self.errors.push(ParseError {
            code: code.to_string(),
        });
    }

    fn emit_current_attribute(&mut self) {
        let Some(attr) = self.current_attribute.take() else {
            return;
        };
        if let Some(Token::StartTag { attrs, .. } | Token::EndTag { attrs, .. }) =
            &mut self.current_token
        {
            // Duplicate attribute check
            if attrs.iter().any(|(name, _)| name == &attr.0) {
                self.emit_error("duplicate-attribute");
            } else {
                attrs.push(attr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tag() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"<div>"));
        assert_eq!(
            t.next_token(),
            Token::StartTag {
                name: "div".to_string(),
                attrs: Vec::new(),
                self_closing: false
            }
        );
    }

    #[test]
    fn test_attributes() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"<div a='b' c=\"d\" e=f>"));
        let tok = t.next_token();
        if let Token::StartTag {
            name, mut attrs, ..
        } = tok
        {
            assert_eq!(name, "div");
            attrs.sort_by(|a, b| a.0.cmp(&b.0));
            assert_eq!(
                attrs,
                vec![
                    ("a".to_string(), "b".to_string()),
                    ("c".to_string(), "d".to_string()),
                    ("e".to_string(), "f".to_string())
                ]
            );
        } else {
            panic!("Expected StartTag, got {:?}", tok);
        }
    }

    #[test]
    fn test_self_closing() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"<img src='a' />"));
        assert_eq!(
            t.next_token(),
            Token::StartTag {
                name: "img".to_string(),
                attrs: vec![("src".to_string(), "a".to_string())],
                self_closing: true
            }
        );
    }

    #[test]
    fn test_end_tag() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"</div>"));
        assert_eq!(
            t.next_token(),
            Token::EndTag {
                name: "div".to_string(),
                attrs: Vec::new(),
                self_closing: false
            }
        );
    }
}
