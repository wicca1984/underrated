use super::{CssToken, CssTokenizer};

// spec: https://www.w3.org/TR/css-syntax-3/#component-value
#[derive(Debug, PartialEq, Clone)]
pub enum ComponentValue {
    Token(CssToken),
    Function {
        name: String,
        value: Vec<ComponentValue>,
    },
    SimpleBlock {
        associated: char,
        value: Vec<ComponentValue>,
    },
}

// spec: https://www.w3.org/TR/css-syntax-3/#declaration
#[derive(Debug, PartialEq, Clone)]
pub struct Declaration {
    pub name: String,
    pub value: Vec<ComponentValue>,
    pub important: bool,
}

// spec: https://www.w3.org/TR/css-syntax-3/#qualified-rule
#[derive(Debug, PartialEq, Clone)]
pub struct QualifiedRule {
    pub prelude: Vec<ComponentValue>,
    pub declarations: Vec<Declaration>,
}

// spec: https://www.w3.org/TR/css-syntax-3/#at-rule
#[derive(Debug, PartialEq, Clone)]
pub struct AtRule {
    pub name: String,
    pub prelude: Vec<ComponentValue>,
    pub block: Option<Vec<ComponentValue>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Rule {
    Qualified(QualifiedRule),
    At(AtRule),
}

// spec: https://www.w3.org/TR/css-syntax-3/#stylesheet
#[derive(Debug, PartialEq, Clone)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

// spec: https://www.w3.org/TR/css-syntax-3/#parse-stylesheet
pub fn parse_stylesheet(input: &str) -> Stylesheet {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.parse_stylesheet()
}

struct Parser<'a> {
    tokenizer: &'a mut CssTokenizer,
    next_token: Option<CssToken>,
}

impl<'a> Parser<'a> {
    fn new(tokenizer: &'a mut CssTokenizer) -> Self {
        Self {
            tokenizer,
            next_token: None,
        }
    }

    fn consume_token(&mut self) -> CssToken {
        if let Some(token) = self.next_token.take() {
            token
        } else {
            self.tokenizer.next_token()
        }
    }

    fn peek_token(&mut self) -> &CssToken {
        self.next_token
            .get_or_insert_with(|| self.tokenizer.next_token())
    }

    fn reconsume_token(&mut self, token: CssToken) {
        debug_assert!(self.next_token.is_none());
        self.next_token = Some(token);
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#parse-stylesheet
    fn parse_stylesheet(&mut self) -> Stylesheet {
        // 1. Consume a list of rules with the top-level flag set.
        let rules = self.consume_list_of_rules(true);
        // 2. Return a new stylesheet with its value set to the consumed rules.
        Stylesheet { rules }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-list-of-rules
    fn consume_list_of_rules(&mut self, top_level: bool) -> Vec<Rule> {
        let mut rules = Vec::new();
        loop {
            let token = self.consume_token();
            match token {
                // <whitespace-token>: Do nothing.
                CssToken::Whitespace => {}
                // <EOF-token>: Return the list of rules.
                CssToken::Eof => return rules,
                // <CDO-token> / <CDC-token>:
                CssToken::Cdo | CssToken::Cdc => {
                    // If the top-level flag is set, do nothing.
                    // Otherwise, reconsume the current input token.
                    // Consume a qualified rule. If anything is returned, append it to the list of rules.
                    if !top_level {
                        self.reconsume_token(token);
                        if let Some(rule) = self.consume_qualified_rule() {
                            rules.push(Rule::Qualified(rule));
                        }
                    }
                }
                // <at-keyword-token>: Reconsume the current input token.
                // Consume an at-rule. Append the returned rule to the list of rules.
                CssToken::AtKeyword(_) => {
                    self.reconsume_token(token);
                    rules.push(Rule::At(self.consume_at_rule()));
                }
                // anything else: Reconsume the current input token.
                // Consume a qualified rule. If anything is returned, append it to the list of rules.
                _ => {
                    self.reconsume_token(token);
                    if let Some(rule) = self.consume_qualified_rule() {
                        rules.push(Rule::Qualified(rule));
                    }
                }
            }
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-at-rule
    fn consume_at_rule(&mut self) -> AtRule {
        // 1. Consume the next input token.
        let token = self.consume_token();
        let name = if let CssToken::AtKeyword(name) = token {
            name
        } else {
            // Should not happen if called correctly
            String::new()
        };

        // 2. Create a new at-rule with its name set to the value of the current input token,
        // its prelude initially set to an empty list, and its block initially set to null.
        let mut prelude = Vec::new();

        // 3. Consume the next input token.
        loop {
            let token = self.consume_token();
            match token {
                // <semicolon-token>: Return the at-rule.
                CssToken::Semicolon => {
                    return AtRule {
                        name,
                        prelude,
                        block: None,
                    };
                }
                // <EOF-token>: This is a parse error. Return the at-rule.
                CssToken::Eof => {
                    return AtRule {
                        name,
                        prelude,
                        block: None,
                    };
                }
                // <left-curly-bracket-token>: Consume a simple block and assign it to the at-rule’s block. Return the at-rule.
                CssToken::LeftBrace => {
                    let block = self.consume_simple_block('{');
                    return AtRule {
                        name,
                        prelude,
                        block: Some(block),
                    };
                }
                // anything else: Reconsume the current input token.
                // Consume a component value and append it to the at-rule’s prelude.
                _ => {
                    self.reconsume_token(token);
                    prelude.push(self.consume_component_value());
                }
            }
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-qualified-rule
    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        // 1. Create a new qualified rule with its prelude initially set to an empty list,
        // and its declarations initially set to an empty list.
        let mut prelude = Vec::new();

        // 2. Consume the next input token.
        loop {
            let token = self.consume_token();
            match token {
                // <EOF-token>: This is a parse error. Return nothing.
                CssToken::Eof => {
                    return None;
                }
                // <left-curly-bracket-token>: Consume a list of declarations.
                // Assign the returned list to the qualified rule’s declarations. Return the qualified rule.
                CssToken::LeftBrace => {
                    let declarations = self.consume_list_of_declarations();
                    return Some(QualifiedRule {
                        prelude,
                        declarations,
                    });
                }
                // anything else: Reconsume the current input token.
                // Consume a component value and append it to the qualified rule’s prelude.
                _ => {
                    self.reconsume_token(token);
                    prelude.push(self.consume_component_value());
                }
            }
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-list-of-declarations
    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        loop {
            let token = self.consume_token();
            match token {
                // <whitespace-token> / <semicolon-token>: Do nothing.
                CssToken::Whitespace | CssToken::Semicolon => {}
                // <EOF-token>: Return the list of declarations.
                CssToken::Eof => {
                    return declarations;
                }
                // <right-curly-bracket-token>: Return the list of declarations.
                CssToken::RightBrace => {
                    return declarations;
                }
                // <at-keyword-token>: Reconsume the current input token.
                // Consume an at-rule. Append the returned rule to the list of declarations.
                CssToken::AtKeyword(_) => {
                    self.reconsume_token(token);
                    // TODO(spec): Declaration struct only supports Declaration, not AtRule.
                    // SPEC S-6: pub struct QualifiedRule { ..., declarations: Vec<Declaration> }
                    self.consume_at_rule();
                }
                // <ident-token>: Create a list of tokens, initially containing the current input token.
                CssToken::Ident(_) => {
                    let mut tokens = vec![token];
                    // While the next input token is anything other than a <semicolon-token> or <EOF-token>,
                    // consume the next input token and append it to the list of tokens.
                    loop {
                        let next = self.consume_token();
                        // Note: RightBrace is also treated as terminator here to avoid infinite loop
                        // if a declaration is not properly closed with semicolon inside a block.
                        if next == CssToken::Semicolon
                            || next == CssToken::Eof
                            || next == CssToken::RightBrace
                        {
                            self.reconsume_token(next);
                            break;
                        }
                        tokens.push(next);
                    }
                    // Consume a declaration from the list of tokens. If anything is returned, append it to the list of declarations.
                    if let Some(decl) = self.consume_declaration_from_tokens(tokens) {
                        declarations.push(decl);
                    }
                }
                // anything else: This is a parse error. Reconsume the current input token.
                // While the next input token is anything other than a <semicolon-token> or <EOF-token>,
                // consume the next input token.
                _ => {
                    self.reconsume_token(token);
                    loop {
                        let next = self.consume_token();
                        if next == CssToken::Semicolon
                            || next == CssToken::Eof
                            || next == CssToken::RightBrace
                        {
                            self.reconsume_token(next);
                            break;
                        }
                    }
                }
            }
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-declaration
    fn consume_declaration_from_tokens(&mut self, tokens: Vec<CssToken>) -> Option<Declaration> {
        // 1. Consume the next input token. Create a new declaration with its name set to the
        // value of the current input token, and its value initially set to an empty list.
        let mut it = tokens.into_iter();
        let name = if let Some(CssToken::Ident(name)) = it.next() {
            name
        } else {
            return None;
        };

        // 2. While the next input token is a <whitespace-token>, consume the next input token.
        let mut next = it.next();
        while let Some(CssToken::Whitespace) = next {
            next = it.next();
        }

        // 3. If the next input token is anything other than a <colon-token>, this is a parse error. Return nothing.
        if next != Some(CssToken::Colon) {
            return None;
        }

        // 4. While the next input token is anything other than an <EOF-token>,
        // consume the next input token and append it to the declaration’s value.
        let mut tokens_for_value: Vec<CssToken> = it.collect();

        // 5. If the last two non-whitespace tokens in the declaration’s value are a <delim-token> with
        // the value "!" and an <ident-token> with a value that is an ASCII case-insensitive match for
        // "important", set the declaration’s important flag to true, and remove them from the declaration’s value.
        let mut important = false;
        let mut non_whitespace_indices = Vec::new();
        for (i, t) in tokens_for_value.iter().enumerate() {
            if !matches!(t, CssToken::Whitespace) {
                non_whitespace_indices.push(i);
            }
        }

        if non_whitespace_indices.len() >= 2 {
            let idx1 = non_whitespace_indices[non_whitespace_indices.len() - 2];
            let idx2 = non_whitespace_indices[non_whitespace_indices.len() - 1];
            match (&tokens_for_value[idx1], &tokens_for_value[idx2]) {
                (CssToken::Delim('!'), CssToken::Ident(ident))
                    if ident.eq_ignore_ascii_case("important") =>
                {
                    important = true;
                    tokens_for_value.truncate(idx1);
                }
                _ => {}
            }
        }

        // 6. Convert the declaration's value to a list of component values.
        let value = self.tokens_to_component_values(tokens_for_value);

        Some(Declaration {
            name,
            value,
            important,
        })
    }

    fn tokens_to_component_values(&mut self, tokens: Vec<CssToken>) -> Vec<ComponentValue> {
        let mut values = Vec::new();
        let mut it = tokens.into_iter().peekable();
        while let Some(token) = it.next() {
            match token {
                CssToken::LeftBrace | CssToken::LeftBracket | CssToken::LeftParen => {
                    let associated = match token {
                        CssToken::LeftBrace => '{',
                        CssToken::LeftBracket => '[',
                        CssToken::LeftParen => '(',
                        _ => unreachable!(),
                    };
                    let mut block_tokens = Vec::new();
                    let mut depth = 1;
                    let closing = match associated {
                        '{' => CssToken::RightBrace,
                        '[' => CssToken::RightBracket,
                        '(' => CssToken::RightParen,
                        _ => unreachable!(),
                    };
                    for t in it.by_ref() {
                        if t == token {
                            depth += 1;
                        } else if t == closing {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        block_tokens.push(t);
                    }
                    values.push(ComponentValue::SimpleBlock {
                        associated,
                        value: self.tokens_to_component_values(block_tokens),
                    });
                }
                CssToken::Function(name) => {
                    let mut func_tokens = Vec::new();
                    let mut depth = 1;
                    for t in it.by_ref() {
                        if let CssToken::Function(_) = t {
                            depth += 1;
                        } else if t == CssToken::LeftParen {
                            depth += 1;
                        } else if t == CssToken::RightParen {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        func_tokens.push(t);
                    }
                    values.push(ComponentValue::Function {
                        name,
                        value: self.tokens_to_component_values(func_tokens),
                    });
                }
                _ => {
                    values.push(ComponentValue::Token(token));
                }
            }
        }
        values
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-component-value
    fn consume_component_value(&mut self) -> ComponentValue {
        // 1. Consume the next input token.
        let token = self.consume_token();
        match token {
            // If the current input token is a <left-curly-bracket-token>, <left-square-bracket-token>,
            // or <left-paren-token>, consume a simple block and return it.
            CssToken::LeftBrace | CssToken::LeftBracket | CssToken::LeftParen => {
                let associated = match token {
                    CssToken::LeftBrace => '{',
                    CssToken::LeftBracket => '[',
                    CssToken::LeftParen => '(',
                    _ => unreachable!(),
                };
                ComponentValue::SimpleBlock {
                    associated,
                    value: self.consume_simple_block(associated),
                }
            }
            // If the current input token is a <function-token>, consume a function and return it.
            CssToken::Function(name) => ComponentValue::Function {
                name,
                value: self.consume_function(),
            },
            // Otherwise, return the current input token.
            _ => ComponentValue::Token(token),
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-simple-block
    fn consume_simple_block(&mut self, associated: char) -> Vec<ComponentValue> {
        let mut value = Vec::new();
        let closing = match associated {
            '{' => CssToken::RightBrace,
            '[' => CssToken::RightBracket,
            '(' => CssToken::RightParen,
            _ => unreachable!(),
        };

        loop {
            let token = self.consume_token();
            // <right-curly-bracket-token>, <right-square-bracket-token>, or <right-paren-token>,
            // matching the associated token: return the block.
            if token == closing {
                return value;
            }
            // <EOF-token>: This is a parse error. Return the block.
            if token == CssToken::Eof {
                return value;
            }
            // anything else: Reconsume the current input token.
            // Consume a component value and append it to the value of the block.
            self.reconsume_token(token);
            value.push(self.consume_component_value());
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-function
    fn consume_function(&mut self) -> Vec<ComponentValue> {
        let mut value = Vec::new();
        loop {
            let token = self.consume_token();
            // <right-paren-token>: Return the function.
            if token == CssToken::RightParen {
                return value;
            }
            // <EOF-token>: This is a parse error. Return the function.
            if token == CssToken::Eof {
                return value;
            }
            // anything else: Reconsume the current input token.
            // Consume a component value and append it to the function’s value.
            self.reconsume_token(token);
            value.push(self.consume_component_value());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_stylesheet() {
        let stylesheet = parse_stylesheet("");
        assert_eq!(stylesheet.rules.len(), 0);
    }

    #[test]
    fn test_parse_simple_rule() {
        let input = "a { color: red; }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            // "a "
            assert_eq!(rule.prelude.len(), 2);
            if let ComponentValue::Token(CssToken::Ident(name)) = &rule.prelude[0] {
                assert_eq!(name, "a");
            } else {
                panic!("Expected ident in prelude");
            }
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
            assert!(!rule.declarations[0].important);
            // Whitespace + "red"
            assert_eq!(rule.declarations[0].value.len(), 2);
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_parse_important_declaration() {
        let input = "a { color: red !important; }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
            assert!(rule.declarations[0].important);
            // Whitespace + "red" + Whitespace
            assert_eq!(rule.declarations[0].value.len(), 3);
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_parse_at_rule() {
        let input = "@media screen { }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::At(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.name, "media");
            // " screen "
            assert_eq!(rule.prelude.len(), 3);
            assert!(rule.block.is_some());
        } else {
            panic!("Expected at-rule");
        }
    }

    #[test]
    fn test_parse_nested_function() {
        let input = "a { color: rgb(255, 0, 0); }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 1);
            let decl = &rule.declarations[0];
            // Whitespace + rgb(255, 0, 0)
            assert_eq!(decl.value.len(), 2);
            if let ComponentValue::Function { name, value } = &decl.value[1] {
                assert_eq!(name, "rgb");
                // 255 + Comma + Whitespace + 0 + Comma + Whitespace + 0
                assert_eq!(value.len(), 7);
            } else {
                panic!("Expected function");
            }
        }
    }

    #[test]
    fn test_parse_multiple_declarations() {
        let input = "a { color: red; background: blue; ; }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "color");
            assert_eq!(rule.declarations[1].name, "background");
        }
    }
}
