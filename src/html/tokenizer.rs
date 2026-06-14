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
    last_start_tag_name: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Data,
    Rcdata,
    Rawtext,
    ScriptData,
    Plaintext,
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
    // Special content states
    RcdataLessThanSign,
    RcdataEndTagOpen,
    RcdataEndTagName,
    RawtextLessThanSign,
    RawtextEndTagOpen,
    RawtextEndTagName,
    ScriptDataLessThanSign,
    ScriptDataEndTagOpen,
    ScriptDataEndTagName,
    ScriptDataEscapeStart,
    ScriptDataEscapeStartDash,
    ScriptDataEscaped,
    ScriptDataEscapedDash,
    ScriptDataEscapedDashDash,
    ScriptDataEscapedLessThanSign,
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeStart,
    ScriptDataDoubleEscaped,
    ScriptDataDoubleEscapedDash,
    ScriptDataDoubleEscapedDashDash,
    ScriptDataDoubleEscapedLessThanSign,
    ScriptDataDoubleEscapeEnd,
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
    // CDATA section states
    CdataSection,
    CdataSectionBracket,
    CdataSectionEnd,
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
            last_start_tag_name: None,
        }
    }

    pub fn set_initial_state(&mut self, state_name: &str) {
        match state_name {
            "Data state" => self.state = State::Data,
            "RCDATA state" => self.state = State::Rcdata,
            "RAWTEXT state" => self.state = State::Rawtext,
            "Script data state" => self.state = State::ScriptData,
            "PLAINTEXT state" => self.state = State::Plaintext,
            "CDATA section state" => self.state = State::CdataSection,
            _ => {}
        }
    }

    pub fn set_last_start_tag(&mut self, tag_name: &str) {
        self.last_start_tag_name = Some(tag_name.to_string());
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
                State::Rcdata => {
                    // // spec: §13.2.5.3 Rcdata state
                    match c {
                        Some('&') => {
                            self.return_state = State::Rcdata;
                            self.state = State::CharacterReference;
                        }
                        Some('<') => {
                            self.state = State::RcdataLessThanSign;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::Rawtext => {
                    // // spec: §13.2.5.4 Rawtext state
                    match c {
                        Some('<') => {
                            self.state = State::RawtextLessThanSign;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptData => {
                    // // spec: §13.2.5.5 Script data state
                    match c {
                        Some('<') => {
                            self.state = State::ScriptDataLessThanSign;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::Plaintext => {
                    // // spec: §13.2.5.6 Plaintext state
                    match c {
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                            return self.emit_current_tag();
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
                State::RcdataLessThanSign => {
                    // // spec: §13.2.5.12 Rcdata less-than-sign state
                    match c {
                        Some('/') => {
                            self.temporary_buffer.clear();
                            self.state = State::RcdataEndTagOpen;
                        }
                        _ => {
                            self.state = State::Rcdata;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::RcdataEndTagOpen => {
                    // // spec: §13.2.5.13 Rcdata end tag open state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::EndTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::RcdataEndTagName;
                            self.input.reconsume();
                        }
                        _ => {
                            self.state = State::Rcdata;
                            self.token_buffer.push_back(Token::Character('/'));
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::RcdataEndTagName => {
                    // // spec: §13.2.5.14 Rcdata end tag name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::BeforeAttributeName;
                            } else {
                                self.anything_else_in_rcdata_end_tag_name()
                            }
                        }
                        Some('/') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::SelfClosingStartTag;
                            } else {
                                self.anything_else_in_rcdata_end_tag_name()
                            }
                        }
                        Some('>') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::Data;
                                if let Some(token) = self.current_token.take() {
                                    return token;
                                }
                            } else {
                                self.anything_else_in_rcdata_end_tag_name()
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val.to_ascii_lowercase());
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val);
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        _ => self.anything_else_in_rcdata_end_tag_name(),
                    }
                }
                State::RawtextLessThanSign => {
                    // // spec: §13.2.5.15 Rawtext less-than-sign state
                    match c {
                        Some('/') => {
                            self.temporary_buffer.clear();
                            self.state = State::RawtextEndTagOpen;
                        }
                        _ => {
                            self.state = State::Rawtext;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::RawtextEndTagOpen => {
                    // // spec: §13.2.5.16 Rawtext end tag open state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::EndTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::RawtextEndTagName;
                            self.input.reconsume();
                        }
                        _ => {
                            self.state = State::Rawtext;
                            self.token_buffer.push_back(Token::Character('/'));
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::RawtextEndTagName => {
                    // // spec: §13.2.5.17 Rawtext end tag name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::BeforeAttributeName;
                            } else {
                                self.anything_else_in_rawtext_end_tag_name()
                            }
                        }
                        Some('/') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::SelfClosingStartTag;
                            } else {
                                self.anything_else_in_rawtext_end_tag_name()
                            }
                        }
                        Some('>') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::Data;
                                if let Some(token) = self.current_token.take() {
                                    return token;
                                }
                            } else {
                                self.anything_else_in_rawtext_end_tag_name()
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val.to_ascii_lowercase());
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val);
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        _ => self.anything_else_in_rawtext_end_tag_name(),
                    }
                }
                State::ScriptDataLessThanSign => {
                    // // spec: §13.2.5.18 Script data less-than-sign state
                    match c {
                        Some('/') => {
                            self.temporary_buffer.clear();
                            self.state = State::ScriptDataEndTagOpen;
                        }
                        Some('!') => {
                            self.state = State::ScriptDataEscapeStart;
                            self.token_buffer.push_back(Token::Character('!'));
                            return Token::Character('<');
                        }
                        _ => {
                            self.state = State::ScriptData;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::ScriptDataEndTagOpen => {
                    // // spec: §13.2.5.19 Script data end tag open state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::EndTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::ScriptDataEndTagName;
                            self.input.reconsume();
                        }
                        _ => {
                            self.state = State::ScriptData;
                            self.token_buffer.push_back(Token::Character('/'));
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::ScriptDataEndTagName => {
                    // // spec: §13.2.5.20 Script data end tag name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::BeforeAttributeName;
                            } else {
                                self.anything_else_in_script_data_end_tag_name()
                            }
                        }
                        Some('/') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::SelfClosingStartTag;
                            } else {
                                self.anything_else_in_script_data_end_tag_name()
                            }
                        }
                        Some('>') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::Data;
                                if let Some(token) = self.current_token.take() {
                                    return token;
                                }
                            } else {
                                self.anything_else_in_script_data_end_tag_name()
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val.to_ascii_lowercase());
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val);
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        _ => self.anything_else_in_script_data_end_tag_name(),
                    }
                }
                State::ScriptDataEscapeStart => {
                    // // spec: §13.2.5.21 Script data escape start state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataEscapeStartDash;
                            return Token::Character('-');
                        }
                        _ => {
                            self.state = State::ScriptData;
                            self.input.reconsume();
                        }
                    }
                }
                State::ScriptDataEscapeStartDash => {
                    // // spec: §13.2.5.22 Script data escape start dash state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataEscapedDashDash;
                            return Token::Character('-');
                        }
                        _ => {
                            self.state = State::ScriptData;
                            self.input.reconsume();
                        }
                    }
                }
                State::ScriptDataEscaped => {
                    // // spec: §13.2.5.23 Script data escaped state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataEscapedDash;
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataEscapedLessThanSign;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataEscapedDash => {
                    // // spec: §13.2.5.24 Script data escaped dash state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataEscapedDashDash;
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataEscapedLessThanSign;
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            self.state = State::ScriptDataEscaped;
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            self.state = State::ScriptDataEscaped;
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataEscapedDashDash => {
                    // // spec: §13.2.5.25 Script data escaped dash dash state
                    match c {
                        Some('-') => {
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataEscapedLessThanSign;
                        }
                        Some('>') => {
                            self.state = State::ScriptData;
                            return Token::Character('>');
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            self.state = State::ScriptDataEscaped;
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            self.state = State::ScriptDataEscaped;
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataEscapedLessThanSign => {
                    // // spec: §13.2.5.26 Script data escaped less-than-sign state
                    match c {
                        Some('/') => {
                            self.temporary_buffer.clear();
                            self.state = State::ScriptDataEscapedEndTagOpen;
                        }
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.temporary_buffer.clear();
                            self.state = State::ScriptDataDoubleEscapeStart;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                        _ => {
                            self.state = State::ScriptDataEscaped;
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::ScriptDataEscapedEndTagOpen => {
                    // // spec: §13.2.5.27 Script data escaped end tag open state
                    match c {
                        Some(c_val) if c_val.is_ascii_alphabetic() => {
                            self.current_token = Some(Token::EndTag {
                                name: String::new(),
                                attrs: Vec::new(),
                                self_closing: false,
                            });
                            self.state = State::ScriptDataEscapedEndTagName;
                            self.input.reconsume();
                        }
                        _ => {
                            self.state = State::ScriptDataEscaped;
                            self.token_buffer.push_back(Token::Character('/'));
                            self.input.reconsume();
                            return Token::Character('<');
                        }
                    }
                }
                State::ScriptDataEscapedEndTagName => {
                    // // spec: §13.2.5.28 Script data escaped end tag name state
                    match c {
                        Some('\t') | Some('\n') | Some('\u{000C}') | Some(' ') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::BeforeAttributeName;
                            } else {
                                self.anything_else_in_script_data_escaped_end_tag_name()
                            }
                        }
                        Some('/') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::SelfClosingStartTag;
                            } else {
                                self.anything_else_in_script_data_escaped_end_tag_name()
                            }
                        }
                        Some('>') => {
                            if self.is_appropriate_end_tag() {
                                self.state = State::Data;
                                if let Some(token) = self.current_token.take() {
                                    return token;
                                }
                            } else {
                                self.anything_else_in_script_data_escaped_end_tag_name()
                            }
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val.to_ascii_lowercase());
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            if let Some(Token::EndTag { name, .. }) = &mut self.current_token {
                                name.push(c_val);
                            }
                            self.temporary_buffer.push(c_val);
                        }
                        _ => self.anything_else_in_script_data_escaped_end_tag_name(),
                    }
                }
                State::ScriptDataDoubleEscapeStart => {
                    // // spec: §13.2.5.29 Script data double escape start state
                    match c {
                        Some(c_val @ '\t')
                        | Some(c_val @ '\n')
                        | Some(c_val @ '\u{000C}')
                        | Some(c_val @ ' ')
                        | Some(c_val @ '/')
                        | Some(c_val @ '>') => {
                            if self.temporary_buffer == "script" {
                                self.state = State::ScriptDataDoubleEscaped;
                            } else {
                                self.state = State::ScriptDataEscaped;
                            }
                            return Token::Character(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            self.temporary_buffer.push(c_val.to_ascii_lowercase());
                            return Token::Character(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            self.temporary_buffer.push(c_val);
                            return Token::Character(c_val);
                        }
                        _ => {
                            self.state = State::ScriptDataEscaped;
                            self.input.reconsume();
                        }
                    }
                }
                State::ScriptDataDoubleEscaped => {
                    // // spec: §13.2.5.30 Script data double escaped state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataDoubleEscapedDash;
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataDoubleEscapedLessThanSign;
                            return Token::Character('<');
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataDoubleEscapedDash => {
                    // // spec: §13.2.5.31 Script data double escaped dash state
                    match c {
                        Some('-') => {
                            self.state = State::ScriptDataDoubleEscapedDashDash;
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataDoubleEscapedLessThanSign;
                            return Token::Character('<');
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            self.state = State::ScriptDataDoubleEscaped;
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            self.state = State::ScriptDataDoubleEscaped;
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataDoubleEscapedDashDash => {
                    // // spec: §13.2.5.32 Script data double escaped dash dash state
                    match c {
                        Some('-') => {
                            return Token::Character('-');
                        }
                        Some('<') => {
                            self.state = State::ScriptDataDoubleEscapedLessThanSign;
                            return Token::Character('<');
                        }
                        Some('>') => {
                            self.state = State::ScriptData;
                            return Token::Character('>');
                        }
                        Some('\0') => {
                            self.emit_error("unexpected-null-character");
                            self.state = State::ScriptDataDoubleEscaped;
                            return Token::Character('\u{FFFD}');
                        }
                        None => {
                            self.emit_error("eof-in-script-html-comment-like-text");
                            return Token::Eof;
                        }
                        Some(c_val) => {
                            self.state = State::ScriptDataDoubleEscaped;
                            return Token::Character(c_val);
                        }
                    }
                }
                State::ScriptDataDoubleEscapedLessThanSign => {
                    // // spec: §13.2.5.33 Script data double escaped less-than-sign state
                    match c {
                        Some('/') => {
                            self.temporary_buffer.clear();
                            self.state = State::ScriptDataDoubleEscapeEnd;
                            return Token::Character('/');
                        }
                        _ => {
                            self.state = State::ScriptDataDoubleEscaped;
                            self.input.reconsume();
                        }
                    }
                }
                State::ScriptDataDoubleEscapeEnd => {
                    // // spec: §13.2.5.34 Script data double escape end state
                    match c {
                        Some(c_val @ '\t')
                        | Some(c_val @ '\n')
                        | Some(c_val @ '\u{000C}')
                        | Some(c_val @ ' ')
                        | Some(c_val @ '/')
                        | Some(c_val @ '>') => {
                            if self.temporary_buffer == "script" {
                                self.state = State::ScriptDataEscaped;
                            } else {
                                self.state = State::ScriptDataDoubleEscaped;
                            }
                            return Token::Character(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_uppercase() => {
                            self.temporary_buffer.push(c_val.to_ascii_lowercase());
                            return Token::Character(c_val);
                        }
                        Some(c_val) if c_val.is_ascii_lowercase() => {
                            self.temporary_buffer.push(c_val);
                            return Token::Character(c_val);
                        }
                        _ => {
                            self.state = State::ScriptDataDoubleEscaped;
                            self.input.reconsume();
                        }
                    }
                }
                State::CdataSection => {
                    // // spec: §13.2.5.68 CDATA section state
                    match c {
                        Some(']') => {
                            self.state = State::CdataSectionBracket;
                        }
                        None => {
                            self.input.reconsume();
                            self.state = State::Data;
                        }
                        Some(c_val) => {
                            return Token::Character(c_val);
                        }
                    }
                }
                State::CdataSectionBracket => {
                    // // spec: §13.2.5.69 CDATA section bracket state
                    match c {
                        Some(']') => {
                            self.state = State::CdataSectionEnd;
                        }
                        _ => {
                            self.token_buffer.push_back(Token::Character(']'));
                            self.input.reconsume();
                            self.state = State::CdataSection;
                        }
                    }
                }
                State::CdataSectionEnd => {
                    // // spec: §13.2.5.70 CDATA section end state
                    match c {
                        Some('>') => {
                            self.state = State::Data;
                        }
                        Some(']') => {
                            self.token_buffer.push_back(Token::Character(']'));
                        }
                        _ => {
                            self.token_buffer.push_back(Token::Character(']'));
                            self.token_buffer.push_back(Token::Character(']'));
                            self.input.reconsume();
                            self.state = State::CdataSection;
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
                                self.temporary_buffer.pop();
                                self.input.reconsume();
                                self.perform_named_character_reference_match();
                            }
                        }
                        Some(';') => {
                            self.temporary_buffer.push(';');
                            self.perform_named_character_reference_match();
                        }
                        _ => {
                            self.input.reconsume();
                            self.perform_named_character_reference_match();
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

    fn is_appropriate_end_tag(&self) -> bool {
        match (&self.current_token, &self.last_start_tag_name) {
            (Some(Token::EndTag { name, .. }), Some(last_name)) => name == last_name,
            _ => false,
        }
    }

    fn anything_else_in_rcdata_end_tag_name(&mut self) {
        self.state = State::Rcdata;
        self.token_buffer.push_back(Token::Character('<'));
        self.token_buffer.push_back(Token::Character('/'));
        let buffer = self.temporary_buffer.clone();
        for c in buffer.chars() {
            self.token_buffer.push_back(Token::Character(c));
        }
        self.input.reconsume();
    }

    fn anything_else_in_rawtext_end_tag_name(&mut self) {
        self.state = State::Rawtext;
        self.token_buffer.push_back(Token::Character('<'));
        self.token_buffer.push_back(Token::Character('/'));
        let buffer = self.temporary_buffer.clone();
        for c in buffer.chars() {
            self.token_buffer.push_back(Token::Character(c));
        }
        self.input.reconsume();
    }

    fn anything_else_in_script_data_end_tag_name(&mut self) {
        self.state = State::ScriptData;
        self.token_buffer.push_back(Token::Character('<'));
        self.token_buffer.push_back(Token::Character('/'));
        let buffer = self.temporary_buffer.clone();
        for c in buffer.chars() {
            self.token_buffer.push_back(Token::Character(c));
        }
        self.input.reconsume();
    }

    fn anything_else_in_script_data_escaped_end_tag_name(&mut self) {
        self.state = State::ScriptDataEscaped;
        self.token_buffer.push_back(Token::Character('<'));
        self.token_buffer.push_back(Token::Character('/'));
        let buffer = self.temporary_buffer.clone();
        for c in buffer.chars() {
            self.token_buffer.push_back(Token::Character(c));
        }
        self.input.reconsume();
    }

    fn emit_current_tag(&mut self) -> Token {
        if let Some(token) = self.current_token.take() {
            match &token {
                Token::StartTag {
                    name, self_closing, ..
                } => {
                    self.last_start_tag_name = Some(name.clone());
                    if *self_closing {
                        self.state = State::Data;
                    } else {
                        match name.as_str() {
                            "title" | "textarea" => self.state = State::Rcdata,
                            "style" | "xmp" | "iframe" | "noembed" | "noframes" => {
                                self.state = State::Rawtext
                            }
                            "script" => self.state = State::ScriptData,
                            "plaintext" => self.state = State::Plaintext,
                            _ => self.state = State::Data,
                        }
                    }
                }
                Token::EndTag { .. } | Token::Doctype { .. } | Token::Comment(_) => {
                    self.state = State::Data;
                }
                _ => {}
            }
            return token;
        }
        Token::Eof
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
            State::Data | State::Rcdata => {
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

    fn perform_named_character_reference_match(&mut self) {
        let mut longest_match: Option<(&'static str, &'static str)> = None;
        for (name, replacement) in NAMED_ENTITIES {
            if self.temporary_buffer.starts_with(name) {
                if let Some((prev_name, _)) = longest_match {
                    if name.len() > prev_name.len() {
                        longest_match = Some((*name, *replacement));
                    }
                } else {
                    longest_match = Some((*name, *replacement));
                }
            }
        }

        if let Some((name, replacement)) = longest_match {
            let is_in_attribute = matches!(
                self.return_state,
                State::AttributeValueDoubleQuoted
                    | State::AttributeValueSingleQuoted
                    | State::AttributeValueUnquoted
            );

            let next_char = if name.len() < self.temporary_buffer.len() {
                self.temporary_buffer.chars().nth(name.len())
            } else {
                self.input.peek()
            };

            let ignore_match = is_in_attribute
                && matches!(next_char, Some(nc) if nc.is_ascii_alphanumeric() || nc == '=');

            if ignore_match {
                let buffer = self.temporary_buffer.clone();
                self.flush_string(&buffer);
            } else {
                if !name.ends_with(';') {
                    self.emit_error("missing-semicolon-after-character-reference");
                }
                self.flush_string(replacement);
                if name.len() < self.temporary_buffer.len() {
                    let suffix: String = self.temporary_buffer.chars().skip(name.len()).collect();
                    self.flush_string(&suffix);
                }
            }
        } else {
            if self.temporary_buffer.ends_with(';') {
                self.emit_error("unknown-named-character-reference");
            }
            let buffer = self.temporary_buffer.clone();
            self.flush_string(&buffer);
        }
        self.state = self.return_state;
    }

    fn is_maybe_named_match(&self) -> bool {
        for (name, _) in NAMED_ENTITIES {
            if name.starts_with(&self.temporary_buffer) {
                return true;
            }
        }
        false
    }
}

const NAMED_ENTITIES: &[(&str, &str)] = &[
    ("&AElig;", "\u{00C6}"),
    ("&AElig", "\u{00C6}"),
    ("&Aacute;", "\u{00C1}"),
    ("&Aacute", "\u{00C1}"),
    ("&Acirc;", "\u{00C2}"),
    ("&Acirc", "\u{00C2}"),
    ("&Agrave;", "\u{00C0}"),
    ("&Agrave", "\u{00C0}"),
    ("&Alpha;", "\u{0391}"),
    ("&AMP;", "&"),
    ("&AMP", "&"),
    ("&Aring;", "\u{00C5}"),
    ("&Aring", "\u{00C5}"),
    ("&Atilde;", "\u{00C3}"),
    ("&Atilde", "\u{00C3}"),
    ("&Auml;", "\u{00C4}"),
    ("&Auml", "\u{00C4}"),
    ("&Beta;", "\u{0392}"),
    ("&Ccedil;", "\u{00C7}"),
    ("&Ccedil", "\u{00C7}"),
    ("&Chi;", "\u{03A7}"),
    ("&COPY;", "\u{00A9}"),
    ("&COPY", "\u{00A9}"),
    ("&Delta;", "\u{0394}"),
    ("&ETH;", "\u{00D0}"),
    ("&ETH", "\u{00D0}"),
    ("&Eacute;", "\u{00C9}"),
    ("&Eacute", "\u{00C9}"),
    ("&Ecirc;", "\u{00CA}"),
    ("&Ecirc", "\u{00CA}"),
    ("&Egrave;", "\u{00C8}"),
    ("&Egrave", "\u{00C8}"),
    ("&Epsilon;", "\u{0395}"),
    ("&Eta;", "\u{0397}"),
    ("&Euml;", "\u{00CB}"),
    ("&Euml", "\u{00CB}"),
    ("&Gamma;", "\u{0393}"),
    ("&GT;", ">"),
    ("&GT", ">"),
    ("&Iacute;", "\u{00CD}"),
    ("&Iacute", "\u{00CD}"),
    ("&Icirc;", "\u{00CE}"),
    ("&Icirc", "\u{00CE}"),
    ("&Igrave;", "\u{00CC}"),
    ("&Igrave", "\u{00CC}"),
    ("&Iota;", "\u{0399}"),
    ("&Iuml;", "\u{00CF}"),
    ("&Iuml", "\u{00CF}"),
    ("&Kappa;", "\u{039A}"),
    ("&Lambda;", "\u{039B}"),
    ("&LT;", "<"),
    ("&LT", "<"),
    ("&Mu;", "\u{039C}"),
    ("&Ntilde;", "\u{00D1}"),
    ("&Ntilde", "\u{00D1}"),
    ("&Nu;", "\u{039D}"),
    ("&NotEqualTilde;", "\u{2242}\u{0338}"),
    ("&Oacute;", "\u{00D3}"),
    ("&Oacute", "\u{00D3}"),
    ("&Ocirc;", "\u{00D4}"),
    ("&Ocirc", "\u{00D4}"),
    ("&OElig;", "\u{0152}"),
    ("&Ograve;", "\u{00D2}"),
    ("&Ograve", "\u{00D2}"),
    ("&Omega;", "\u{03A9}"),
    ("&Omicron;", "\u{039F}"),
    ("&Oslash;", "\u{00D8}"),
    ("&Oslash", "\u{00D8}"),
    ("&Otilde;", "\u{00D5}"),
    ("&Otilde", "\u{00D5}"),
    ("&Ouml;", "\u{00D6}"),
    ("&Ouml", "\u{00D6}"),
    ("&Phi;", "\u{03A6}"),
    ("&Pi;", "\u{03A0}"),
    ("&Psi;", "\u{03A8}"),
    ("&QUOT;", "\""),
    ("&QUOT", "\""),
    ("&REG;", "\u{00AE}"),
    ("&REG", "\u{00AE}"),
    ("&Rho;", "\u{03A1}"),
    ("&Scaron;", "\u{0160}"),
    ("&Sigma;", "\u{03A3}"),
    ("&THORN;", "\u{00DE}"),
    ("&THORN", "\u{00DE}"),
    ("&Tau;", "\u{03A4}"),
    ("&Theta;", "\u{0398}"),
    ("&Uacute;", "\u{00DA}"),
    ("&Uacute", "\u{00DA}"),
    ("&Ucirc;", "\u{00DB}"),
    ("&Ucirc", "\u{00DB}"),
    ("&Ugrave;", "\u{00D9}"),
    ("&Ugrave", "\u{00D9}"),
    ("&Upsilon;", "\u{03A5}"),
    ("&Uuml;", "\u{00DC}"),
    ("&Uuml", "\u{00DC}"),
    ("&Xi;", "\u{039E}"),
    ("&Yacute;", "\u{00DD}"),
    ("&Yacute", "\u{00DD}"),
    ("&Yuml;", "\u{0178}"),
    ("&Zeta;", "\u{0396}"),
    ("&aacute;", "\u{00E1}"),
    ("&aacute", "\u{00E1}"),
    ("&acirc;", "\u{00E2}"),
    ("&acirc", "\u{00E2}"),
    ("&acute;", "\u{00B4}"),
    ("&acute", "\u{00B4}"),
    ("&aelig;", "\u{00E6}"),
    ("&aelig", "\u{00E6}"),
    ("&agrave;", "\u{00E0}"),
    ("&agrave", "\u{00E0}"),
    ("&alefsym;", "\u{2135}"),
    ("&alpha;", "\u{03B1}"),
    ("&aleph;", "\u{2135}"),
    ("&amp;", "&"),
    ("&amp", "&"),
    ("&and;", "\u{2227}"),
    ("&ang;", "\u{2220}"),
    ("&apos;", "'"),
    ("&apos", "'"),
    ("&approx;", "\u{2248}"),
    ("&aring;", "\u{00E5}"),
    ("&aring", "\u{00E5}"),
    ("&ast;", "*"),
    ("&asymp;", "\u{2248}"),
    ("&atilde;", "\u{00E3}"),
    ("&atilde", "\u{00E3}"),
    ("&auml;", "\u{00E4}"),
    ("&auml", "\u{00E4}"),
    ("&bdquo;", "\u{201E}"),
    ("&beta;", "\u{03B2}"),
    ("&brvbar;", "\u{00A6}"),
    ("&brvbar", "\u{00A6}"),
    ("&bsol;", "\\"),
    ("&bull;", "\u{2022}"),
    ("&cap;", "\u{2229}"),
    ("&ccedil;", "\u{00E7}"),
    ("&ccedil", "\u{00E7}"),
    ("&cedil;", "\u{00B8}"),
    ("&cedil", "\u{00B8}"),
    ("&cent;", "\u{00A2}"),
    ("&cent", "\u{00A2}"),
    ("&checkmark;", "\u{2713}"),
    ("&chi;", "\u{03C7}"),
    ("&circ;", "\u{02C6}"),
    ("&clubs;", "\u{2663}"),
    ("&colon;", ":"),
    ("&comma;", ","),
    ("&commat;", "@"),
    ("&cong;", "\u{2245}"),
    ("&copy;", "\u{00A9}"),
    ("&copy", "\u{00A9}"),
    ("&crarr;", "\u{21B5}"),
    ("&cross;", "\u{2717}"),
    ("&cup;", "\u{222A}"),
    ("&curren;", "\u{00A4}"),
    ("&curren", "\u{00A4}"),
    ("&dArr;", "\u{21D3}"),
    ("&dagger;", "\u{2020}"),
    ("&dagger", "\u{2020}"),
    ("&Dagger;", "\u{2021}"),
    ("&darr;", "\u{2193}"),
    ("&deg;", "\u{00B0}"),
    ("&deg", "\u{00B0}"),
    ("&delta;", "\u{03B4}"),
    ("&diams;", "\u{2666}"),
    ("&divide;", "\u{00F7}"),
    ("&divide", "\u{00F7}"),
    ("&dollar;", "$"),
    ("&eacute;", "\u{00E9}"),
    ("&eacute", "\u{00E9}"),
    ("&ecirc;", "\u{00EA}"),
    ("&ecirc", "\u{00EA}"),
    ("&egrave;", "\u{00E8}"),
    ("&egrave", "\u{00E8}"),
    ("&empty;", "\u{2205}"),
    ("&emsp;", "\u{2003}"),
    ("&ensp;", "\u{2002}"),
    ("&epsilon;", "\u{03B5}"),
    ("&equals;", "="),
    ("&equiv;", "\u{2261}"),
    ("&eta;", "\u{03B7}"),
    ("&eth;", "\u{00F0}"),
    ("&eth", "\u{00F0}"),
    ("&euml;", "\u{00EB}"),
    ("&euml", "\u{00EB}"),
    ("&euro;", "\u{20AC}"),
    ("&excl;", "!"),
    ("&exist;", "\u{2203}"),
    ("&flat;", "\u{266D}"),
    ("&fnof;", "\u{0192}"),
    ("&forall;", "\u{2200}"),
    ("&frac12;", "\u{00BD}"),
    ("&frac12", "\u{00BD}"),
    ("&frac14;", "\u{00BC}"),
    ("&frac14", "\u{00BC}"),
    ("&frac34;", "\u{00BE}"),
    ("&frac34", "\u{00BE}"),
    ("&frasl;", "\u{2044}"),
    ("&gamma;", "\u{03B3}"),
    ("&ge;", "\u{2265}"),
    ("&ge", "\u{2265}"),
    ("&gt;", ">"),
    ("&gt", ">"),
    ("&hArr;", "\u{21D4}"),
    ("&hairsp;", "\u{200A}"),
    ("&harr;", "\u{2194}"),
    ("&hat;", "^"),
    ("&hearts;", "\u{2665}"),
    ("&hellip;", "\u{2026}"),
    ("&horbar;", "\u{2015}"),
    ("&iacute;", "\u{00ED}"),
    ("&iacute", "\u{00ED}"),
    ("&icirc;", "\u{00EE}"),
    ("&icirc", "\u{00EE}"),
    ("&iexcl;", "\u{00A1}"),
    ("&iexcl", "\u{00A1}"),
    ("&igrave;", "\u{00EC}"),
    ("&igrave", "\u{00EC}"),
    ("&image;", "\u{2111}"),
    ("&infin;", "\u{221E}"),
    ("&int;", "\u{222B}"),
    ("&iota;", "\u{03B9}"),
    ("&iquest;", "\u{00BF}"),
    ("&iquest", "\u{00BF}"),
    ("&isin;", "\u{2208}"),
    ("&iuml;", "\u{00EF}"),
    ("&iuml", "\u{00EF}"),
    ("&kappa;", "\u{03BA}"),
    ("&lArr;", "\u{21D0}"),
    ("&lambda;", "\u{03BB}"),
    ("&lang;", "\u{2329}"),
    ("&laquo;", "\u{00AB}"),
    ("&laquo", "\u{00AB}"),
    ("&larr;", "\u{2190}"),
    ("&lceil;", "\u{2308}"),
    ("&lcub;", "{"),
    ("&ldquo;", "\u{201C}"),
    ("&le;", "\u{2264}"),
    ("&le", "\u{2264}"),
    ("&lfloor;", "\u{230A}"),
    ("&lowast;", "\u{2217}"),
    ("&loz;", "\u{25CA}"),
    ("&lpar;", "("),
    ("&lrm;", "\u{200E}"),
    ("&lsaquo;", "\u{2039}"),
    ("&lsqb;", "["),
    ("&lsquo;", "\u{2018}"),
    ("&lt;", "<"),
    ("&lt", "<"),
    ("&macr;", "\u{00AF}"),
    ("&macr", "\u{00AF}"),
    ("&mdash;", "\u{2014}"),
    ("&micro;", "\u{00B5}"),
    ("&micro", "\u{00B5}"),
    ("&middot;", "\u{00B7}"),
    ("&middot", "\u{00B7}"),
    ("&minus;", "\u{2212}"),
    ("&mu;", "\u{03BC}"),
    ("&nabla;", "\u{2207}"),
    ("&nbsp;", "\u{00A0}"),
    ("&nbsp", "\u{00A0}"),
    ("&ndash;", "\u{2013}"),
    ("&ne;", "\u{2260}"),
    ("&ni;", "\u{220B}"),
    ("&not;", "\u{00AC}"),
    ("&not", "\u{00AC}"),
    ("&notin;", "\u{2209}"),
    ("&nsub;", "\u{2284}"),
    ("&nsup;", "\u{2285}"),
    ("&ntilde;", "\u{00F1}"),
    ("&ntilde", "\u{00F1}"),
    ("&num;", "#"),
    ("&nu;", "\u{03BD}"),
    ("&oacute;", "\u{00F3}"),
    ("&oacute", "\u{00F3}"),
    ("&ocirc;", "\u{00F4}"),
    ("&ocirc", "\u{00F4}"),
    ("&oelig;", "\u{0153}"),
    ("&ograve;", "\u{00F2}"),
    ("&ograve", "\u{00F2}"),
    ("&oline;", "\u{203E}"),
    ("&omega;", "\u{03C9}"),
    ("&omicron;", "\u{03BF}"),
    ("&oplus;", "\u{2295}"),
    ("&or;", "\u{2228}"),
    ("&ordf;", "\u{00AA}"),
    ("&ordf", "\u{00AA}"),
    ("&ordm;", "\u{00BA}"),
    ("&ordm", "\u{00BA}"),
    ("&oslash;", "\u{00F8}"),
    ("&oslash", "\u{00F8}"),
    ("&otilde;", "\u{00F5}"),
    ("&otilde", "\u{00F5}"),
    ("&otimes;", "\u{2297}"),
    ("&ouml;", "\u{00F6}"),
    ("&ouml", "\u{00F6}"),
    ("&para;", "\u{00B6}"),
    ("&para", "\u{00B6}"),
    ("&part;", "\u{2202}"),
    ("&percnt;", "%"),
    ("&period;", "."),
    ("&permil;", "\u{2030}"),
    ("&perp;", "\u{22A5}"),
    ("&phi;", "\u{03C6}"),
    ("&pi;", "\u{03C0}"),
    ("&piv;", "\u{03D6}"),
    ("&plus;", "+"),
    ("&plusmn;", "\u{00B1}"),
    ("&plusmn", "\u{00B1}"),
    ("&pound;", "\u{00A3}"),
    ("&pound", "\u{00A3}"),
    ("&prime;", "\u{2032}"),
    ("&Prime;", "\u{2033}"),
    ("&prod;", "\u{220F}"),
    ("&prop;", "\u{221D}"),
    ("&psi;", "\u{03C8}"),
    ("&quest;", "?"),
    ("&quot;", "\""),
    ("&quot", "\""),
    ("&rArr;", "\u{21D2}"),
    ("&radic;", "\u{221A}"),
    ("&rang;", "\u{232A}"),
    ("&raquo;", "\u{00BB}"),
    ("&raquo", "\u{00BB}"),
    ("&rarr;", "\u{2192}"),
    ("&rceil;", "\u{2309}"),
    ("&rcub;", "}"),
    ("&rdquo;", "\u{201D}"),
    ("&real;", "\u{211C}"),
    ("&reg;", "\u{00AE}"),
    ("&reg", "\u{00AE}"),
    ("&rfloor;", "\u{230B}"),
    ("&rho;", "\u{03C1}"),
    ("&rlm;", "\u{200F}"),
    ("&rpar;", ")"),
    ("&rsaquo;", "\u{203A}"),
    ("&rsqb;", "]"),
    ("&rsquo;", "\u{2019}"),
    ("&sbquo;", "\u{201A}"),
    ("&scaron;", "\u{0161}"),
    ("&scasp;", "\u{2005}"),
    ("&sdot;", "\u{22C5}"),
    ("&sect;", "\u{00A7}"),
    ("&sect", "\u{00A7}"),
    ("&semi;", ";"),
    ("&shy;", "\u{00AD}"),
    ("&shy", "\u{00AD}"),
    ("&sigma;", "\u{03C3}"),
    ("&sigmaf;", "\u{03C2}"),
    ("&sim;", "\u{223C}"),
    ("&sol;", "/"),
    ("&spades;", "\u{2660}"),
    ("&star;", "\u{2606}"),
    ("&starf;", "\u{2605}"),
    ("&sub;", "\u{2282}"),
    ("&sube;", "\u{2286}"),
    ("&sum;", "\u{2211}"),
    ("&sup1;", "\u{00B9}"),
    ("&sup1", "\u{00B9}"),
    ("&sup2;", "\u{00B2}"),
    ("&sup2", "\u{00B2}"),
    ("&sup3;", "\u{00B3}"),
    ("&sup3", "\u{00B3}"),
    ("&sup;", "\u{2283}"),
    ("&supe;", "\u{2287}"),
    ("&szlig;", "\u{00DF}"),
    ("&szlig", "\u{00DF}"),
    ("&tau;", "\u{03C4}"),
    ("&there4;", "\u{2234}"),
    ("&theta;", "\u{03B8}"),
    ("&thetasym;", "\u{03D1}"),
    ("&thinsp;", "\u{2009}"),
    ("&thorn;", "\u{00FE}"),
    ("&thorn", "\u{00FE}"),
    ("&tilde;", "\u{02DC}"),
    ("&times;", "\u{00D7}"),
    ("&times", "\u{00D7}"),
    ("&trade;", "\u{2122}"),
    ("&trade", "\u{2122}"),
    ("&uArr;", "\u{21D1}"),
    ("&uacute;", "\u{00FA}"),
    ("&uacute", "\u{00FA}"),
    ("&uarr;", "\u{2191}"),
    ("&ucirc;", "\u{00FB}"),
    ("&ucirc", "\u{00FB}"),
    ("&ugrave;", "\u{00F9}"),
    ("&ugrave", "\u{00F9}"),
    ("&uml;", "\u{00A8}"),
    ("&uml", "\u{00A8}"),
    ("&upsih;", "\u{03D2}"),
    ("&upsilon;", "\u{03C5}"),
    ("&uuml;", "\u{00FC}"),
    ("&uuml", "\u{00FC}"),
    ("&verbar;", "|"),
    ("&weierp;", "\u{2118}"),
    ("&xi;", "\u{03BE}"),
    ("&yacute;", "\u{00FD}"),
    ("&yacute", "\u{00FD}"),
    ("&yen;", "\u{00A5}"),
    ("&yen", "\u{00A5}"),
    ("&yuml;", "\u{00FF}"),
    ("&yuml", "\u{00FF}"),
    ("&zeta;", "\u{03B6}"),
    ("&zwj;", "\u{200D}"),
    ("&zwnj;", "\u{200C}"),
];

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

    #[test]
    fn test_extended_named_entities() {
        let test_cases = [
            ("&Alpha;", "\u{0391}"),
            ("&Omega;", "\u{03A9}"),
            ("&checkmark;", "\u{2713}"),
            ("&cross;", "\u{2717}"),
            ("&approx;", "\u{2248}"),
            ("&laquo;", "\u{00AB}"),
            ("&laquo", "\u{00AB}"),
            ("&raquo;", "\u{00BB}"),
            ("&raquo", "\u{00BB}"),
            ("&star;", "\u{2606}"),
            ("&starf;", "\u{2605}"),
            // Newly-added test cases:
            ("&AMP;", "&"),
            ("&AMP", "&"),
            ("&LT;", "<"),
            ("&LT", "<"),
            ("&GT;", ">"),
            ("&GT", ">"),
            ("&QUOT;", "\""),
            ("&QUOT", "\""),
            ("&COPY;", "\u{00A9}"),
            ("&COPY", "\u{00A9}"),
            ("&REG;", "\u{00AE}"),
            ("&REG", "\u{00AE}"),
            ("&OElig;", "\u{0152}"),
            ("&oelig;", "\u{0153}"),
            ("&Yuml;", "\u{0178}"),
            ("&lfloor;", "\u{230A}"),
            ("&rfloor;", "\u{230B}"),
            ("&le", "\u{2264}"),
            ("&ge", "\u{2265}"),
            ("&excl;", "!"),
            ("&dollar;", "$"),
            ("&percnt;", "%"),
            ("&sol;", "/"),
            ("&semi;", ";"),
            ("&colon;", ":"),
            ("&comma;", ","),
            ("&commat;", "@"),
            ("&lpar;", "("),
            ("&rpar;", ")"),
            ("&lsqb;", "["),
            ("&rsqb;", "]"),
            ("&lcub;", "{"),
            ("&rcub;", "}"),
            ("&verbar;", "|"),
            ("&hairsp;", "\u{200A}"),
            ("&aleph;", "\u{2135}"),
            ("&ast;", "*"),
            ("&bsol;", "\\"),
            ("&flat;", "\u{266D}"),
            ("&horbar;", "\u{2015}"),
        ];

        for (input, expected) in test_cases {
            let stream = InputStream::from_utf8(input.as_bytes());
            let mut tokenizer = Tokenizer::new(stream);
            let mut decoded = String::new();
            loop {
                match tokenizer.next_token() {
                    Token::Character(c) => decoded.push(c),
                    Token::Eof => break,
                    other => panic!("Unexpected token: {:?}", other),
                }
            }
            assert_eq!(decoded, expected, "Failed for {}", input);
        }
    }
}
