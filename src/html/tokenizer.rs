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
    return_state: State,
    character_reference_code: u32,
    temporary_buffer: String,
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
    BogusComment,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThanSign,
    CommentLessThanSignBang,
    CommentLessThanSignBangDash,
    CommentLessThanSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypePublicKeyword,
    BeforeDoctypePublicIdentifier,
    DoctypePublicIdentifierDoubleQuoted,
    DoctypePublicIdentifierSingleQuoted,
    AfterDoctypePublicIdentifier,
    BetweenDoctypePublicAndSystemIdentifiers,
    AfterDoctypeSystemKeyword,
    BeforeDoctypeSystemIdentifier,
    DoctypeSystemIdentifierDoubleQuoted,
    DoctypeSystemIdentifierSingleQuoted,
    AfterDoctypeSystemIdentifier,
    BogusDoctype,
    // Character reference states
    CharacterReference,
    NamedCharacterReference,
    NumericCharacterReference,
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReferenceStart,
    HexadecimalCharacterReference,
    DecimalCharacterReference,
    NumericCharacterReferenceEnd,
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
            return_state: State::Data,
            character_reference_code: 0,
            temporary_buffer: String::new(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            if let Some(token) = self.token_buffer.pop_front() {
                return token;
            }

            let c = self.input.next();

            match self.state {
                State::Data => {
                    // // spec: §13.2.5.1 Data state
                    match c {
                        Some('&') => {
                            self.return_state = State::Data;
                            self.state = State::CharacterReference;
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
                State::Doctype => {
                    // // spec: §13.2.5.54 Doctype state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::BeforeDoctypeName;
                        }
                        Some('>') => {
                            self.input.reconsume();
                            self.state = State::BeforeDoctypeName;
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            self.current_token = Some(Token::Doctype {
                                name: None,
                                public_id: None,
                                system_id: None,
                                force_quirks: true,
                            });
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-whitespace-before-doctype-name");
                            self.input.reconsume();
                            self.state = State::BeforeDoctypeName;
                        }
                    }
                }
                State::BeforeDoctypeName => {
                    // // spec: §13.2.5.55 Before Doctype name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            self.current_token = Some(Token::Doctype {
                                name: Some(c_val.to_ascii_lowercase().to_string()),
                                public_id: None,
                                system_id: None,
                                force_quirks: false,
                            });
                            self.state = State::DoctypeName;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            self.current_token = Some(Token::Doctype {
                                name: Some("\u{FFFD}".to_string()),
                                public_id: None,
                                system_id: None,
                                force_quirks: false,
                            });
                            self.state = State::DoctypeName;
                        }
                        Some('>') => {
                            self.emit_error("missing-doctype-name");
                            self.current_token = Some(Token::Doctype {
                                name: None,
                                public_id: None,
                                system_id: None,
                                force_quirks: true,
                            });
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            self.current_token = Some(Token::Doctype {
                                name: None,
                                public_id: None,
                                system_id: None,
                                force_quirks: true,
                            });
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            self.current_token = Some(Token::Doctype {
                                name: Some(c_val.to_string()),
                                public_id: None,
                                system_id: None,
                                force_quirks: false,
                            });
                            self.state = State::DoctypeName;
                        }
                    }
                }
                State::DoctypeName => {
                    // // spec: §13.2.5.56 Doctype name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::AfterDoctypeName;
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::Doctype { name: Some(n), .. }) =
                                &mut self.current_token
                            {
                                n.push(c_val.to_ascii_lowercase());
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Doctype { name: Some(n), .. }) =
                                &mut self.current_token
                            {
                                n.push('\u{FFFD}');
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Doctype { name: Some(n), .. }) =
                                &mut self.current_token
                            {
                                n.push(c_val);
                            }
                        }
                    }
                }
                State::AfterDoctypeName => {
                    // // spec: §13.2.5.57 After Doctype name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_c_val) => {
                            self.input.reconsume();
                            if let Some('P' | 'p') = self.input.peek()
                                && self.match_keyword("PUBLIC")
                            {
                                self.state = State::AfterDoctypePublicKeyword;
                                continue;
                            }
                            if let Some('S' | 's') = self.input.peek()
                                && self.match_keyword("SYSTEM")
                            {
                                self.state = State::AfterDoctypeSystemKeyword;
                                continue;
                            }
                            self.emit_error("invalid-character-sequence-after-doctype-name");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::AfterDoctypePublicKeyword => {
                    // // spec: §13.2.5.58 After Doctype public keyword state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::BeforeDoctypePublicIdentifier;
                        }
                        Some('"') => {
                            self.emit_error("missing-whitespace-after-doctype-public-keyword");
                            if let Some(Token::Doctype { public_id, .. }) = &mut self.current_token
                            {
                                *public_id = Some(String::new());
                            }
                            self.state = State::DoctypePublicIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            self.emit_error("missing-whitespace-after-doctype-public-keyword");
                            if let Some(Token::Doctype { public_id, .. }) = &mut self.current_token
                            {
                                *public_id = Some(String::new());
                            }
                            self.state = State::DoctypePublicIdentifierSingleQuoted;
                        }
                        Some('>') => {
                            self.emit_error("missing-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::BeforeDoctypePublicIdentifier => {
                    // // spec: §13.2.5.59 Before Doctype public identifier state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('"') => {
                            if let Some(Token::Doctype { public_id, .. }) = &mut self.current_token
                            {
                                *public_id = Some(String::new());
                            }
                            self.state = State::DoctypePublicIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            if let Some(Token::Doctype { public_id, .. }) = &mut self.current_token
                            {
                                *public_id = Some(String::new());
                            }
                            self.state = State::DoctypePublicIdentifierSingleQuoted;
                        }
                        Some('>') => {
                            self.emit_error("missing-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::DoctypePublicIdentifierDoubleQuoted => {
                    // // spec: §13.2.5.60 Doctype public identifier (double-quoted) state
                    match c {
                        Some('"') => {
                            self.state = State::AfterDoctypePublicIdentifier;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Doctype {
                                public_id: Some(p), ..
                            }) = &mut self.current_token
                            {
                                p.push('\u{FFFD}');
                            }
                        }
                        Some('>') => {
                            self.emit_error("abrupt-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Doctype {
                                public_id: Some(p), ..
                            }) = &mut self.current_token
                            {
                                p.push(c_val);
                            }
                        }
                    }
                }
                State::DoctypePublicIdentifierSingleQuoted => {
                    // // spec: §13.2.5.61 Doctype public identifier (single-quoted) state
                    match c {
                        Some('\'') => {
                            self.state = State::AfterDoctypePublicIdentifier;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Doctype {
                                public_id: Some(p), ..
                            }) = &mut self.current_token
                            {
                                p.push('\u{FFFD}');
                            }
                        }
                        Some('>') => {
                            self.emit_error("abrupt-doctype-public-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Doctype {
                                public_id: Some(p), ..
                            }) = &mut self.current_token
                            {
                                p.push(c_val);
                            }
                        }
                    }
                }
                State::AfterDoctypePublicIdentifier => {
                    // // spec: §13.2.5.62 After Doctype public identifier state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::BetweenDoctypePublicAndSystemIdentifiers;
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some('"') => {
                            self.emit_error(
                                "missing-whitespace-between-doctype-public-and-system-identifiers",
                            );
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            self.emit_error(
                                "missing-whitespace-between-doctype-public-and-system-identifiers",
                            );
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierSingleQuoted;
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::BetweenDoctypePublicAndSystemIdentifiers => {
                    // // spec: §13.2.5.63 Between Doctype public and system identifiers state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some('"') => {
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierSingleQuoted;
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::AfterDoctypeSystemKeyword => {
                    // // spec: §13.2.5.64 After Doctype system keyword state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            self.state = State::BeforeDoctypeSystemIdentifier;
                        }
                        Some('"') => {
                            self.emit_error("missing-whitespace-after-doctype-system-keyword");
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            self.emit_error("missing-whitespace-after-doctype-system-keyword");
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierSingleQuoted;
                        }
                        Some('>') => {
                            self.emit_error("missing-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::BeforeDoctypeSystemIdentifier => {
                    // // spec: §13.2.5.65 Before Doctype system identifier state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('"') => {
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierDoubleQuoted;
                        }
                        Some('\'') => {
                            if let Some(Token::Doctype { system_id, .. }) = &mut self.current_token
                            {
                                *system_id = Some(String::new());
                            }
                            self.state = State::DoctypeSystemIdentifierSingleQuoted;
                        }
                        Some('>') => {
                            self.emit_error("missing-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("missing-quote-before-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::DoctypeSystemIdentifierDoubleQuoted => {
                    // // spec: §13.2.5.66 Doctype system identifier (double-quoted) state
                    match c {
                        Some('"') => {
                            self.state = State::AfterDoctypeSystemIdentifier;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Doctype {
                                system_id: Some(s), ..
                            }) = &mut self.current_token
                            {
                                s.push('\u{FFFD}');
                            }
                        }
                        Some('>') => {
                            self.emit_error("abrupt-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Doctype {
                                system_id: Some(s), ..
                            }) = &mut self.current_token
                            {
                                s.push(c_val);
                            }
                        }
                    }
                }
                State::DoctypeSystemIdentifierSingleQuoted => {
                    // // spec: §13.2.5.67 Doctype system identifier (single-quoted) state
                    match c {
                        Some('\'') => {
                            self.state = State::AfterDoctypeSystemIdentifier;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Doctype {
                                system_id: Some(s), ..
                            }) = &mut self.current_token
                            {
                                s.push('\u{FFFD}');
                            }
                        }
                        Some('>') => {
                            self.emit_error("abrupt-doctype-system-identifier");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Doctype {
                                system_id: Some(s), ..
                            }) = &mut self.current_token
                            {
                                s.push(c_val);
                            }
                        }
                    }
                }
                State::AfterDoctypeSystemIdentifier => {
                    // // spec: §13.2.5.68 After Doctype system identifier state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            // Ignore
                        }
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-doctype");
                            if let Some(Token::Doctype { force_quirks, .. }) =
                                &mut self.current_token
                            {
                                *force_quirks = true;
                            }
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.emit_error("unexpected-character-after-doctype-system-identifier");
                            self.state = State::BogusDoctype;
                        }
                    }
                }
                State::BogusDoctype => {
                    // // spec: §13.2.5.69 Bogus Doctype state
                    match c {
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                        }
                        None => {
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            // Ignore
                        }
                    }
                }
                State::MarkupDeclarationOpen => {
                    // // spec: §13.2.5.43 Markup declaration open state
                    match c {
                        Some('-') => {
                            if let Some('-') = self.input.peek() {
                                self.input.next();
                                self.current_token = Some(Token::Comment(String::new()));
                                self.state = State::CommentStart;
                            } else {
                                self.emit_error("incorrect-formatting-of-html-comment");
                                self.current_token = Some(Token::Comment("-".to_string()));
                                self.state = State::BogusComment;
                            }
                        }
                        Some(c_val @ 'D') | Some(c_val @ 'd') => {
                            let mut matched = true;
                            let mut data = String::from(c_val);
                            for k in "OCTYPE".chars() {
                                if let Some(in_c) = self.input.peek() {
                                    if in_c.to_ascii_uppercase() == k {
                                        data.push(self.input.next().unwrap_or('\0'));
                                    } else {
                                        matched = false;
                                        break;
                                    }
                                } else {
                                    matched = false;
                                    break;
                                }
                            }
                            if matched {
                                self.state = State::Doctype;
                            } else {
                                self.emit_error("mismatched-markup-declaration-open");
                                self.current_token = Some(Token::Comment(data));
                                self.state = State::BogusComment;
                            }
                        }
                        _ => {
                            self.emit_error("mismatched-markup-declaration-open");
                            self.current_token = Some(Token::Comment(String::new()));
                            self.state = State::BogusComment;
                            self.input.reconsume();
                        }
                    }
                }
                State::BogusComment => {
                    // // spec: §13.2.5.42 Bogus comment state
                    match c {
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('\u{FFFD}');
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push(c_val);
                            }
                        }
                    }
                }
                State::CommentStart => {
                    // // spec: §13.2.5.44 Comment start state
                    match c {
                        Some('-') => {
                            self.state = State::CommentStartDash;
                        }
                        Some('>') => {
                            self.emit_error("abrupt-closing-of-empty-comment");
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(_) => {
                            self.state = State::Comment;
                            self.input.reconsume();
                        }
                    }
                }
                State::CommentStartDash => {
                    // // spec: §13.2.5.45 Comment start dash state
                    match c {
                        Some('-') => {
                            self.state = State::CommentEnd;
                        }
                        Some('>') => {
                            self.emit_error("abrupt-closing-of-empty-comment");
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('\u{FFFD}');
                            }
                            self.state = State::Comment;
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push(c_val);
                            }
                            self.state = State::Comment;
                        }
                    }
                }
                State::Comment => {
                    // // spec: §13.2.5.46 Comment state
                    match c {
                        Some('<') => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('<');
                            }
                            self.state = State::CommentLessThanSign;
                        }
                        Some('-') => {
                            self.state = State::CommentEndDash;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('\u{FFFD}');
                            }
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push(c_val);
                            }
                        }
                    }
                }
                State::CommentLessThanSign => {
                    // // spec: §13.2.5.47 Comment less-than-sign state
                    match c {
                        Some('!') => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('!');
                            }
                            self.state = State::CommentLessThanSignBang;
                        }
                        Some('<') => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('<');
                            }
                        }
                        Some(_) => {
                            self.state = State::Comment;
                            self.input.reconsume();
                        }
                        None => {
                            self.state = State::Comment;
                            self.input.reconsume();
                        }
                    }
                }
                State::CommentLessThanSignBang => {
                    // // spec: §13.2.5.48 Comment less-than-sign bang state
                    match c {
                        Some('-') => {
                            self.state = State::CommentLessThanSignBangDash;
                        }
                        Some(_) => {
                            self.state = State::Comment;
                            self.input.reconsume();
                        }
                        None => {
                            self.state = State::Comment;
                            self.input.reconsume();
                        }
                    }
                }
                State::CommentLessThanSignBangDash => {
                    // // spec: §13.2.5.49 Comment less-than-sign bang dash state
                    match c {
                        Some('-') => {
                            self.state = State::CommentLessThanSignBangDashDash;
                        }
                        Some(_) => {
                            self.state = State::CommentEndDash;
                            self.input.reconsume();
                        }
                        None => {
                            self.state = State::CommentEndDash;
                            self.input.reconsume();
                        }
                    }
                }
                State::CommentLessThanSignBangDashDash => {
                    // // spec: §13.2.5.50 Comment less-than-sign bang dash dash state
                    match c {
                        Some(_) | None => {
                            self.state = State::CommentEnd;
                            self.input.reconsume();
                        }
                    }
                }
                State::CommentEndDash => {
                    // // spec: §13.2.5.51 Comment end dash state
                    match c {
                        Some('-') => {
                            self.state = State::CommentEnd;
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('\u{FFFD}');
                            }
                            self.state = State::Comment;
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push(c_val);
                            }
                            self.state = State::Comment;
                        }
                    }
                }
                State::CommentEnd => {
                    // // spec: §13.2.5.52 Comment end state
                    match c {
                        Some('>') => {
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        Some('!') => {
                            self.state = State::CommentEndBang;
                        }
                        Some('-') => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                            }
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('-');
                                data.push('\u{FFFD}');
                            }
                            self.state = State::Comment;
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('-');
                                data.push(c_val);
                            }
                            self.state = State::Comment;
                        }
                    }
                }
                State::CommentEndBang => {
                    // // spec: §13.2.5.53 Comment end bang state
                    match c {
                        Some('-') => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('-');
                                data.push('!');
                            }
                            self.state = State::CommentEndDash;
                        }
                        Some('>') => {
                            self.emit_error("incorrectly-closed-comment");
                            self.state = State::Data;
                            if let Some(token) = self.current_token.take() {
                                return token;
                            }
                        }
                        None => {
                            self.emit_error("eof-in-comment");
                            self.state = State::Data;
                            let token = self.current_token.take();
                            self.token_buffer.push_back(Token::Eof);
                            if let Some(t) = token {
                                return t;
                            }
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('-');
                                data.push('!');
                                data.push('\u{FFFD}');
                            }
                            self.state = State::Comment;
                        }
                        Some(c_val) => {
                            if let Some(Token::Comment(data)) = &mut self.current_token {
                                data.push('-');
                                data.push('-');
                                data.push('!');
                                data.push(c_val);
                            }
                            self.state = State::Comment;
                        }
                    }
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
                        Some('>') => {
                            self.emit_error("missing-tag-name");
                            self.state = State::Data;
                            self.token_buffer.push_back(Token::Character('>'));
                            return Token::Character('<');
                        }
                        Some('?') => {
                            self.emit_error("unexpected-question-mark-instead-of-tag-name");
                            self.current_token = Some(Token::Comment(String::new()));
                            self.state = State::BogusComment;
                            self.input.reconsume();
                        }
                        Some(_) => {
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
                            self.current_token = Some(Token::Comment(String::new()));
                            self.state = State::BogusComment;
                            self.input.reconsume();
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
                            self.return_state = State::AttributeValueDoubleQuoted;
                            self.state = State::CharacterReference;
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
                            self.return_state = State::AttributeValueSingleQuoted;
                            self.state = State::CharacterReference;
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
                            self.return_state = State::AttributeValueUnquoted;
                            self.state = State::CharacterReference;
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
                State::CharacterReference => {
                    // // spec: §13.2.5.72 Character reference state
                    self.temporary_buffer.clear();
                    self.temporary_buffer.push('&');
                    match c {
                        Some(c_val) if c_val.is_ascii_alphanumeric() => {
                            self.state = State::NamedCharacterReference;
                            self.input.reconsume();
                        }
                        Some('#') => {
                            self.temporary_buffer.push('#');
                            self.state = State::NumericCharacterReference;
                        }
                        _ => {
                            self.flush_string("&");
                            self.state = self.return_state;
                            self.input.reconsume();
                        }
                    }
                }
                State::NumericCharacterReference => {
                    // // spec: §13.2.5.75 Numeric character reference state
                    self.character_reference_code = 0;
                    match c {
                        Some(c_val @ 'x') | Some(c_val @ 'X') => {
                            self.temporary_buffer.push(c_val);
                            self.state = State::HexadecimalCharacterReferenceStart;
                        }
                        _ => {
                            self.state = State::DecimalCharacterReferenceStart;
                            self.input.reconsume();
                        }
                    }
                }
                State::HexadecimalCharacterReferenceStart => {
                    // // spec: §13.2.5.76 Hexadecimal character reference start state
                    match c {
                        Some(c_val) if c_val.is_ascii_hexdigit() => {
                            self.state = State::HexadecimalCharacterReference;
                            self.input.reconsume();
                        }
                        _ => {
                            self.emit_error("absence-of-digits-in-numeric-character-reference");
                            let buffer = self.temporary_buffer.clone();
                            self.flush_string(&buffer);
                            self.state = self.return_state;
                            self.input.reconsume();
                        }
                    }
                }
                State::DecimalCharacterReferenceStart => {
                    // // spec: §13.2.5.77 Decimal character reference start state
                    match c {
                        Some(c_val) if c_val.is_ascii_digit() => {
                            self.state = State::DecimalCharacterReference;
                            self.input.reconsume();
                        }
                        _ => {
                            self.emit_error("absence-of-digits-in-numeric-character-reference");
                            let buffer = self.temporary_buffer.clone();
                            self.flush_string(&buffer);
                            self.state = self.return_state;
                            self.input.reconsume();
                        }
                    }
                }
                State::HexadecimalCharacterReference => {
                    // // spec: §13.2.5.78 Hexadecimal character reference state
                    match c {
                        // spec: ASCII hex digit (0-9 / a-f / A-F) ONLY. Other letters
                        // (g-z, G-Z) must not be consumed here — they terminate the
                        // reference via the catch-all arm below (missing-semicolon).
                        Some(c_val) if c_val.is_ascii_hexdigit() => {
                            self.character_reference_code = self
                                .character_reference_code
                                .saturating_mul(16)
                                .saturating_add(c_val.to_digit(16).unwrap_or(0));
                        }
                        Some(';') => {
                            self.state = State::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            self.emit_error("missing-semicolon-after-character-reference");
                            self.state = State::NumericCharacterReferenceEnd;
                            self.input.reconsume();
                        }
                    }
                }
                State::DecimalCharacterReference => {
                    // // spec: §13.2.5.79 Decimal character reference state
                    match c {
                        Some(c_val) if c_val.is_ascii_digit() => {
                            self.character_reference_code = self
                                .character_reference_code
                                .saturating_mul(10)
                                .saturating_add(c_val.to_digit(10).unwrap_or(0));
                        }
                        Some(';') => {
                            self.state = State::NumericCharacterReferenceEnd;
                        }
                        _ => {
                            self.emit_error("missing-semicolon-after-character-reference");
                            self.state = State::NumericCharacterReferenceEnd;
                            self.input.reconsume();
                        }
                    }
                }
                State::NumericCharacterReferenceEnd => {
                    // // spec: §13.2.5.80 Numeric character reference end state
                    let mut code = self.character_reference_code;
                    // // spec: §13.2.5.80 replacement rules
                    if code == 0 {
                        self.emit_error("null-character-reference");
                        code = 0xFFFD;
                    } else if code > 0x10FFFF {
                        self.emit_error("character-reference-outside-unicode-range");
                        code = 0xFFFD;
                    } else if (0xD800..=0xDFFF).contains(&code) {
                        self.emit_error("surrogate-character-reference");
                        code = 0xFFFD;
                    } else if is_noncharacter(code) {
                        self.emit_error("noncharacter-character-reference");
                        // No replacement for noncharacters, just error.
                    } else if is_control_character(code) && !is_whitespace(code) {
                        self.emit_error("control-character-reference");
                        code = match code {
                            0x80 => 0x20AC, // EURO SIGN (€)
                            0x82 => 0x201A, // SINGLE LOW-9 QUOTATION MARK (‚)
                            0x83 => 0x0192, // LATIN SMALL LETTER F WITH HOOK (ƒ)
                            0x84 => 0x201E, // DOUBLE LOW-9 QUOTATION MARK („)
                            0x85 => 0x2026, // HORIZONTAL ELLIPSIS (…)
                            0x86 => 0x2020, // DAGGER (†)
                            0x87 => 0x2021, // DOUBLE DAGGER (‡)
                            0x88 => 0x02C6, // MODIFIER LETTER CIRCUMFLEX ACCENT (ˆ)
                            0x89 => 0x2030, // PER MILLE SIGN (‰)
                            0x8A => 0x0160, // LATIN CAPITAL LETTER S WITH CARON (Š)
                            0x8B => 0x2039, // SINGLE LEFT-POINTING ANGLE QUOTATION MARK (‹)
                            0x8C => 0x0152, // LATIN CAPITAL LIGATURE OE (Œ)
                            0x8E => 0x017D, // LATIN CAPITAL LETTER Z WITH CARON (Ž)
                            0x91 => 0x2018, // LEFT SINGLE QUOTATION MARK (‘)
                            0x92 => 0x2019, // RIGHT SINGLE QUOTATION MARK (’)
                            0x93 => 0x201C, // LEFT DOUBLE QUOTATION MARK (“)
                            0x94 => 0x201D, // RIGHT DOUBLE QUOTATION MARK (”)
                            0x95 => 0x2022, // BULLET (•)
                            0x96 => 0x2013, // EN DASH (–)
                            0x97 => 0x2014, // EM DASH (—)
                            0x98 => 0x02DC, // SMALL TILDE (˜)
                            0x99 => 0x2122, // TRADE MARK SIGN (™)
                            0x9A => 0x0161, // LATIN SMALL LETTER S WITH CARON (š)
                            0x9B => 0x203A, // SINGLE RIGHT-POINTING ANGLE QUOTATION MARK (›)
                            0x9C => 0x0153, // LATIN SMALL LIGATURE OE (œ)
                            0x9E => 0x017E, // LATIN SMALL LETTER Z WITH CARON (ž)
                            0x9F => 0x0178, // LATIN CAPITAL LETTER Y WITH DIAERESIS (Ÿ)
                            _ => code,
                        };
                    }

                    if let Some(rc) = std::char::from_u32(code) {
                        self.flush_character(rc);
                    } else {
                        // Should not happen if logic above is correct
                        self.flush_character('\u{FFFD}');
                    }

                    self.state = self.return_state;
                    self.input.reconsume();
                }
                State::NamedCharacterReference => {
                    // // spec: §13.2.5.73 Named character reference state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphanumeric() => {
                            self.temporary_buffer.push(c_val);
                            if !self.is_maybe_named_match() {
                                let buffer = self.temporary_buffer.clone();
                                self.flush_string(&buffer);
                                self.state = self.return_state;
                            }
                        }
                        Some(';') => {
                            self.temporary_buffer.push(';');
                            if let Some(replacement) = self.check_named_match(true) {
                                self.flush_string(replacement);
                                self.state = self.return_state;
                            } else {
                                let buffer = self.temporary_buffer.clone();
                                self.flush_string(&buffer);
                                self.state = self.return_state;
                            }
                        }
                        _ => {
                            self.input.reconsume();
                            if let Some(replacement) = self.check_named_match(false) {
                                self.flush_string(replacement);
                                self.state = self.return_state;
                            } else {
                                let buffer = self.temporary_buffer.clone();
                                self.flush_string(&buffer);
                                self.state = self.return_state;
                            }
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

    fn match_keyword(&mut self, keyword: &str) -> bool {
        let mut matched = true;
        for c_k in keyword.chars() {
            if let Some(c_in) = self.input.peek() {
                if c_in.to_ascii_uppercase() == c_k {
                    self.input.next();
                } else {
                    matched = false;
                    break;
                }
            } else {
                matched = false;
                break;
            }
        }
        matched
    }

    fn flush_character(&mut self, c: char) {
        match self.return_state {
            State::Data => {
                self.token_buffer.push_back(Token::Character(c));
            }
            State::AttributeValueDoubleQuoted
            | State::AttributeValueSingleQuoted
            | State::AttributeValueUnquoted => {
                if let Some(attr) = &mut self.current_attribute {
                    attr.1.push(c);
                }
            }
            _ => {
                // Should not happen according to spec
            }
        }
    }

    fn flush_string(&mut self, s: &str) {
        for c in s.chars() {
            self.flush_character(c);
        }
    }

    fn check_named_match(&self, has_semicolon: bool) -> Option<&'static str> {
        let core_entities = [
            ("&amp;", "&"),
            ("&lt;", "<"),
            ("&gt;", ">"),
            ("&quot;", "\""),
            ("&apos;", "'"),
            ("&nbsp;", "\u{00A0}"),
        ];

        for (name, replacement) in core_entities {
            if has_semicolon {
                if name == self.temporary_buffer {
                    return Some(replacement);
                }
            } else {
                // For non-semicolon matches, we check if the buffer is a prefix
                // but the spec says "match an entry in the table".
                // Since our core set all have semicolons, this will mostly return None.
                // However, we should handle cases where the buffer EQUALS an entry.
                if name == self.temporary_buffer {
                    return Some(replacement);
                }
            }
        }
        None
    }

    fn is_maybe_named_match(&self) -> bool {
        let core_entities = ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;", "&nbsp;"];
        for name in core_entities {
            if name.starts_with(&self.temporary_buffer) {
                return true;
            }
        }
        false
    }
}

fn is_noncharacter(code: u32) -> bool {
    (0xFDD0..=0xFDEF).contains(&code)
        || [
            0xFFFE, 0xFFFF, 0x1FFFE, 0x1FFFF, 0x2FFFE, 0x2FFFF, 0x3FFFE, 0x3FFFF, 0x4FFFE, 0x4FFFF,
            0x5FFFE, 0x5FFFF, 0x6FFFE, 0x6FFFF, 0x7FFFE, 0x7FFFF, 0x8FFFE, 0x8FFFF, 0x9FFFE,
            0x9FFFF, 0xAFFFE, 0xAFFFF, 0xBFFFE, 0xBFFFF, 0xCFFFE, 0xCFFFF, 0xDFFFE, 0xDFFFF,
            0xEFFFE, 0xEFFFF, 0xFFFFE, 0xFFFFF, 0x10FFFE, 0x10FFFF,
        ]
        .contains(&code)
}

fn is_control_character(code: u32) -> bool {
    (0x0000..=0x001F).contains(&code) || (0x007F..=0x009F).contains(&code)
}

fn is_whitespace(code: u32) -> bool {
    matches!(code, 0x0009 | 0x000A | 0x000C | 0x000D | 0x0020)
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

    #[test]
    fn test_comment() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"<!-- comment -->"));
        assert_eq!(t.next_token(), Token::Comment(" comment ".to_string()));
        assert_eq!(t.next_token(), Token::Eof);
    }

    #[test]
    fn test_doctype() {
        let mut t = Tokenizer::new(InputStream::from_utf8(b"<!Doctype html>"));
        assert_eq!(
            t.next_token(),
            Token::Doctype {
                name: Some("html".to_string()),
                public_id: None,
                system_id: None,
                force_quirks: false
            }
        );
    }
}
