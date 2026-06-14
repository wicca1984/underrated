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

/// Parses an input string into a list of component values.
/// spec: <https://www.w3.org/TR/css-syntax-3/#consume-component-value>
pub fn parse_component_values(input: &str) -> Vec<ComponentValue> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    let mut values = Vec::new();
    loop {
        let token = parser.consume_token();
        if token == CssToken::Eof {
            break;
        }
        parser.reconsume_token(token);
        values.push(parser.consume_component_value());
    }
    values
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
            let val = self.consume_component_value();
            match val {
                ComponentValue::Token(CssToken::Whitespace)
                | ComponentValue::Token(CssToken::Semicolon) => {}
                ComponentValue::Token(CssToken::Eof) => {
                    return declarations;
                }
                ComponentValue::Token(CssToken::RightBrace) => {
                    return declarations;
                }
                ComponentValue::Token(CssToken::AtKeyword(_)) => {
                    if let ComponentValue::Token(token) = val {
                        self.reconsume_token(token);
                    }
                    self.consume_at_rule();
                }
                ComponentValue::Token(CssToken::Ident(_)) => {
                    let mut decl_values = vec![val];
                    loop {
                        let next = self.consume_component_value();
                        match next {
                            ComponentValue::Token(CssToken::Semicolon) => {
                                break;
                            }
                            ComponentValue::Token(CssToken::Eof)
                            | ComponentValue::Token(CssToken::RightBrace) => {
                                if let ComponentValue::Token(token) = next {
                                    self.reconsume_token(token);
                                }
                                break;
                            }
                            _ => {
                                decl_values.push(next);
                            }
                        }
                    }
                    if let Some(decl) = self.consume_declaration_from_component_values(decl_values)
                    {
                        declarations.push(decl);
                    }
                }
                _ => loop {
                    let next = self.consume_component_value();
                    match next {
                        ComponentValue::Token(CssToken::Semicolon) => break,
                        ComponentValue::Token(CssToken::Eof)
                        | ComponentValue::Token(CssToken::RightBrace) => {
                            if let ComponentValue::Token(token) = next {
                                self.reconsume_token(token);
                            }
                            break;
                        }
                        _ => {}
                    }
                },
            }
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#consume-declaration
    fn consume_declaration_from_component_values(
        &mut self,
        values: Vec<ComponentValue>,
    ) -> Option<Declaration> {
        let mut it = values.into_iter();
        let name = if let Some(ComponentValue::Token(CssToken::Ident(name))) = it.next() {
            name
        } else {
            return None;
        };

        let mut next = it.next();
        while let Some(ComponentValue::Token(CssToken::Whitespace)) = next {
            next = it.next();
        }

        if next != Some(ComponentValue::Token(CssToken::Colon)) {
            return None;
        }

        let mut value_components: Vec<ComponentValue> = it.collect();

        let mut important = false;
        let mut non_whitespace_indices = Vec::new();
        for (i, v) in value_components.iter().enumerate() {
            if !matches!(v, ComponentValue::Token(CssToken::Whitespace)) {
                non_whitespace_indices.push(i);
            }
        }

        if non_whitespace_indices.len() >= 2 {
            let idx1 = non_whitespace_indices[non_whitespace_indices.len() - 2];
            let idx2 = non_whitespace_indices[non_whitespace_indices.len() - 1];
            match (&value_components[idx1], &value_components[idx2]) {
                (
                    ComponentValue::Token(CssToken::Delim('!')),
                    ComponentValue::Token(CssToken::Ident(ident)),
                ) if ident.eq_ignore_ascii_case("important") => {
                    important = true;
                    value_components.truncate(idx1);
                }
                _ => {}
            }
        }

        if crate::css::values::is_known_layout_property(&name)
            && !has_var_or_calc(&value_components)
        {
            if let Some(parsed_val) = crate::css::values::parse_value(&value_components) {
                if !crate::css::values::is_valid_property_value(&name, &parsed_val) {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some(Declaration {
            name,
            value: value_components,
            important,
        })
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

fn has_var_or_calc(components: &[ComponentValue]) -> bool {
    for comp in components {
        match comp {
            ComponentValue::Function { name, value } => {
                if name.eq_ignore_ascii_case("var") || name.eq_ignore_ascii_case("calc") {
                    return true;
                }
                if has_var_or_calc(value) {
                    return true;
                }
            }
            ComponentValue::SimpleBlock { value, .. } if has_var_or_calc(value) => {
                return true;
            }
            _ => {}
        }
    }
    false
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

    #[test]
    fn test_parse_layout_properties_validation() {
        // 1. Valid properties & values (mixed case)
        let input = "
            div {
                position: ABSOLUTE;
                overflow: hidden;
                box-sizing: border-box;
                display: flex;
                flex-direction: COLUMN;
                justify-content: space-between;
                align-items: center;
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 7);
            assert_eq!(rule.declarations[0].name, "position");
            assert_eq!(rule.declarations[1].name, "overflow");
            assert_eq!(rule.declarations[2].name, "box-sizing");
            assert_eq!(rule.declarations[3].name, "display");
            assert_eq!(rule.declarations[4].name, "flex-direction");
            assert_eq!(rule.declarations[5].name, "justify-content");
            assert_eq!(rule.declarations[6].name, "align-items");
        } else {
            panic!("Expected qualified rule");
        }

        // 2. Invalid/Unknown values are ignored/discarded
        let input_invalid = "
            div {
                position: invalid-position;
                overflow: invalid-overflow;
                box-sizing: invalid-box-sizing;
                display: invalid-display;
                flex-direction: invalid-direction;
                justify-content: invalid-justify;
                align-items: invalid-align;
                color: red; /* non-layout property stays valid */
            }
        ";
        let stylesheet_invalid = parse_stylesheet(input_invalid);
        assert_eq!(stylesheet_invalid.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_invalid.rules[0] {
            // All invalid layout declarations must be discarded, only color is left
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule");
        }

        // 3. Properties containing var() or calc() are preserved and not validated during parsing
        let input_vars = "
            div {
                position: var(--pos);
                flex-direction: calc(1);
            }
        ";
        let stylesheet_vars = parse_stylesheet(input_vars);
        assert_eq!(stylesheet_vars.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_vars.rules[0] {
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "position");
            assert_eq!(rule.declarations[1].name, "flex-direction");
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_parse_nested_blocks_in_declarations() {
        let input = "
            div {
                --custom: { color: red; background: blue; };
                color: green;
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "--custom");
            assert_eq!(rule.declarations[1].name, "color");

            // Check that the inner block was parsed as a simple block `{ ... }`
            let custom_val = &rule.declarations[0].value;
            // Whitespace + SimpleBlock
            assert_eq!(custom_val.len(), 2);
            if let ComponentValue::SimpleBlock { associated, value } = &custom_val[1] {
                assert_eq!(*associated, '{');
                // The inner block should contain all component values including semicolons!
                // { color: red; background: blue; }
                // Let's verify that "background" is present inside
                let has_background = value.iter().any(|val| {
                    if let ComponentValue::Token(CssToken::Ident(name)) = val {
                        name == "background"
                    } else {
                        false
                    }
                });
                assert!(
                    has_background,
                    "Expected 'background' inside custom property block"
                );
            } else {
                panic!("Expected SimpleBlock for --custom value");
            }
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_parse_unbalanced_braces_balancing() {
        let input = "div { val1: [ { ]; val2: ( { } ; }";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            // Due to unmatched bracket/braces:
            // val1's value contains unmatched LeftBracket '[' and LeftBrace '{'
            // The rest of the stream is consumed inside the unbalanced block, yielding exactly 1 declaration.
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "val1");

            // For val1: [ { ] ...
            let val1 = &rule.declarations[0].value;
            if let ComponentValue::SimpleBlock { associated, value } = &val1[1] {
                assert_eq!(*associated, '[');
                if let ComponentValue::SimpleBlock {
                    associated: inner_assoc,
                    value: _inner_val,
                } = &value[1]
                {
                    assert_eq!(*inner_assoc, '{');
                } else {
                    panic!("Expected inner SimpleBlock");
                }
            } else {
                panic!("Expected SimpleBlock for val1");
            }
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_important_declaration_whitespace_robustness() {
        let input = "
            a {
                color: red ! important ;
                background: blue !important   ;
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "color");
            assert!(rule.declarations[0].important);
            assert_eq!(rule.declarations[1].name, "background");
            assert!(rule.declarations[1].important);
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_parse_error_recovery_with_nested_blocks() {
        let input = "
            div {
                position: { color: red; };
                color: red;
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            // "position" has an invalid value (a simple block), so it is discarded, but "color" is kept!
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule");
        }
    }
}
