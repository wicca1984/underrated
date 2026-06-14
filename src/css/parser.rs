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

// spec: https://www.w3.org/TR/css-syntax-3/#parse-list-of-rules
pub fn parse_list_of_rules(input: &str) -> Vec<Rule> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.consume_list_of_rules(true)
}

// spec: https://www.w3.org/TR/css-syntax-3/#parse-rule
pub fn parse_rule(input: &str) -> Option<Rule> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.parse_rule()
}

// spec: https://www.w3.org/TR/css-syntax-3/#parse-declaration
pub fn parse_declaration(input: &str) -> Option<Declaration> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.parse_declaration()
}

// spec: https://www.w3.org/TR/css-syntax-3/#parse-list-of-declarations
pub fn parse_list_of_declarations(input: &str) -> Vec<Declaration> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.consume_list_of_declarations(&[])
}

// spec: https://www.w3.org/TR/css-syntax-3/#parse-comma-separated-list-of-component-values
pub fn parse_comma_separated_list_of_component_values(input: &str) -> Vec<Vec<ComponentValue>> {
    let mut tokenizer = CssTokenizer::new(input);
    let mut parser = Parser::new(&mut tokenizer);
    parser.parse_comma_separated_list_of_component_values()
}

struct Parser<'a> {
    tokenizer: &'a mut CssTokenizer,
    next_token: Option<CssToken>,
    nested_rules: Vec<Rule>,
}

impl<'a> Parser<'a> {
    fn new(tokenizer: &'a mut CssTokenizer) -> Self {
        Self {
            tokenizer,
            next_token: None,
            nested_rules: Vec::new(),
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

    // spec: https://www.w3.org/TR/css-syntax-3/#parse-rule
    fn parse_rule(&mut self) -> Option<Rule> {
        let mut token = self.consume_token();
        while token == CssToken::Whitespace {
            token = self.consume_token();
        }
        if token == CssToken::Eof {
            return None;
        }
        let rule = if let CssToken::AtKeyword(_) = token {
            self.reconsume_token(token);
            Rule::At(self.consume_at_rule())
        } else {
            self.reconsume_token(token);
            if let Some(r) = self.consume_qualified_rule() {
                Rule::Qualified(r)
            } else {
                return None;
            }
        };
        let mut next_token = self.consume_token();
        while next_token == CssToken::Whitespace {
            next_token = self.consume_token();
        }
        if next_token == CssToken::Eof {
            Some(rule)
        } else {
            None
        }
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#parse-declaration
    fn parse_declaration(&mut self) -> Option<Declaration> {
        let mut token = self.consume_token();
        while token == CssToken::Whitespace {
            token = self.consume_token();
        }
        if !matches!(token, CssToken::Ident(_)) {
            return None;
        }
        let mut values = vec![ComponentValue::Token(token)];
        loop {
            let next = self.consume_token();
            if next == CssToken::Eof {
                break;
            }
            self.reconsume_token(next);
            values.push(self.consume_component_value());
        }
        self.consume_declaration_from_component_values(values)
    }

    // spec: https://www.w3.org/TR/css-syntax-3/#parse-comma-separated-list-of-component-values
    fn parse_comma_separated_list_of_component_values(&mut self) -> Vec<Vec<ComponentValue>> {
        // 2. Let list of cvs be an empty list of component values, containing an initially empty list.
        let mut list_of_cvs = vec![Vec::new()];
        loop {
            let token = self.consume_token();
            if token == CssToken::Eof {
                break;
            }
            if token == CssToken::Comma {
                list_of_cvs.push(Vec::new());
            } else {
                self.reconsume_token(token);
                let cv = self.consume_component_value();
                if let Some(last) = list_of_cvs.last_mut() {
                    last.push(cv);
                }
            }
        }
        list_of_cvs
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
                    let nested = self.nested_rules.drain(..).collect::<Vec<_>>();
                    rules.extend(nested);
                }
                // anything else: Reconsume the current input token.
                // Consume a qualified rule. If anything is returned, append it to the list of rules.
                _ => {
                    self.reconsume_token(token);
                    if let Some(rule) = self.consume_qualified_rule() {
                        rules.push(Rule::Qualified(rule));
                        let nested = self.nested_rules.drain(..).collect::<Vec<_>>();
                        rules.extend(nested);
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
                    let declarations = self.consume_list_of_declarations(&prelude);
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
    fn consume_list_of_declarations(
        &mut self,
        parent_prelude: &[ComponentValue],
    ) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        let mut collected: Vec<ComponentValue> = Vec::new();

        loop {
            let val = self.consume_component_value();
            match val {
                ComponentValue::Token(CssToken::Whitespace) => {
                    collected.push(val);
                }
                ComponentValue::Token(CssToken::Semicolon) => {
                    if !collected.is_empty() {
                        let mut decl_vals = collected;
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.first()
                        {
                            decl_vals.remove(0);
                        }
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.last()
                        {
                            decl_vals.pop();
                        }
                        if let Some(decl) =
                            self.consume_declaration_from_component_values(decl_vals)
                        {
                            declarations.push(decl);
                        }
                        collected = Vec::new();
                    }
                }
                ComponentValue::Token(CssToken::Eof)
                | ComponentValue::Token(CssToken::RightBrace) => {
                    if !collected.is_empty() {
                        let mut decl_vals = collected;
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.first()
                        {
                            decl_vals.remove(0);
                        }
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.last()
                        {
                            decl_vals.pop();
                        }
                        if let Some(decl) =
                            self.consume_declaration_from_component_values(decl_vals)
                        {
                            declarations.push(decl);
                        }
                    }
                    return declarations;
                }
                ComponentValue::Token(CssToken::AtKeyword(_)) => {
                    if !collected.is_empty() {
                        let mut decl_vals = collected;
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.first()
                        {
                            decl_vals.remove(0);
                        }
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            decl_vals.last()
                        {
                            decl_vals.pop();
                        }
                        if let Some(decl) =
                            self.consume_declaration_from_component_values(decl_vals)
                        {
                            declarations.push(decl);
                        }
                        collected = Vec::new();
                    }
                    if let ComponentValue::Token(token) = val {
                        self.reconsume_token(token);
                    }
                    let at_rule = self.consume_at_rule();
                    if !parent_prelude.is_empty() {
                        if let Some(block_vals) = at_rule.block {
                            let block_css = serialize_component_values(&block_vals);
                            let mut sub_tokenizer = CssTokenizer::new(&block_css);
                            let mut sub_parser = Parser::new(&mut sub_tokenizer);
                            let sub_decls = sub_parser.consume_list_of_declarations(parent_prelude);

                            let mut nested_css = String::new();
                            if !sub_decls.is_empty() {
                                let nested_qual = QualifiedRule {
                                    prelude: parent_prelude.to_vec(),
                                    declarations: sub_decls,
                                };
                                nested_css.push_str(&serialize_rule(&Rule::Qualified(nested_qual)));
                            }
                            for nested_rule in &sub_parser.nested_rules {
                                nested_css.push_str(&serialize_rule(nested_rule));
                            }

                            let mut token_stream = CssTokenizer::new(&nested_css);
                            let mut parser = Parser::new(&mut token_stream);
                            let mut block_cvs = Vec::new();
                            loop {
                                let cv = parser.consume_component_value();
                                if let ComponentValue::Token(CssToken::Eof) = cv {
                                    break;
                                }
                                block_cvs.push(cv);
                            }
                            let nested_at = AtRule {
                                name: at_rule.name,
                                prelude: at_rule.prelude,
                                block: Some(block_cvs),
                            };
                            self.nested_rules.push(Rule::At(nested_at));
                        } else {
                            self.nested_rules.push(Rule::At(at_rule));
                        }
                    } else {
                        self.nested_rules.push(Rule::At(at_rule));
                    }
                }
                ComponentValue::SimpleBlock {
                    associated: '{',
                    value: block_values,
                } => {
                    let is_decl_block = collected
                        .iter()
                        .rev()
                        .find(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
                        .is_some_and(|c| matches!(c, ComponentValue::Token(CssToken::Colon)));

                    if is_decl_block {
                        collected.push(ComponentValue::SimpleBlock {
                            associated: '{',
                            value: block_values,
                        });
                    } else {
                        let mut child_prelude = collected.clone();
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            child_prelude.first()
                        {
                            child_prelude.remove(0);
                        }
                        while let Some(ComponentValue::Token(CssToken::Whitespace)) =
                            child_prelude.last()
                        {
                            child_prelude.pop();
                        }

                        if !child_prelude.is_empty() {
                            let parent_sel = serialize_component_values(parent_prelude);
                            let child_sel = serialize_component_values(&child_prelude);

                            let combined_sel = combine_selectors(&parent_sel, &child_sel);

                            let mut combined_tokenizer = CssTokenizer::new(&combined_sel);
                            let mut combined_parser = Parser::new(&mut combined_tokenizer);
                            let mut combined_prelude = Vec::new();
                            loop {
                                let cv = combined_parser.consume_component_value();
                                if let ComponentValue::Token(CssToken::Eof) = cv {
                                    break;
                                }
                                combined_prelude.push(cv);
                            }

                            let block_css = serialize_component_values(&block_values);
                            let mut sub_tokenizer = CssTokenizer::new(&block_css);
                            let mut sub_parser = Parser::new(&mut sub_tokenizer);
                            let nested_decls =
                                sub_parser.consume_list_of_declarations(&combined_prelude);

                            self.nested_rules.extend(sub_parser.nested_rules);

                            let nested_rule = Rule::Qualified(QualifiedRule {
                                prelude: combined_prelude,
                                declarations: nested_decls,
                            });

                            self.nested_rules.push(nested_rule);
                        }

                        collected = Vec::new();
                    }
                }
                _ => {
                    collected.push(val);
                }
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

        // Trim trailing semicolon and any trailing whitespaces after it (e.g. from direct parse_declaration input)
        let mut semicolon_idx = None;
        for (i, v) in value_components.iter().enumerate().rev() {
            match v {
                ComponentValue::Token(CssToken::Semicolon) => {
                    semicolon_idx = Some(i);
                    break;
                }
                ComponentValue::Token(CssToken::Whitespace) => {
                    // continue looking back
                }
                _ => {
                    // any other token means no trailing semicolon is present
                    break;
                }
            }
        }
        if let Some(idx) = semicolon_idx {
            value_components.truncate(idx);
        }

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

        // spec: https://www.w3.org/TR/css-syntax-3/#consume-declaration
        // If any of the component values of the declaration is a <bad-string-token>, <bad-url-token>,
        // <right-paren-token>, <right-bracket-token>, or <right-curly-bracket-token>, the declaration is invalid.
        let is_invalid = value_components.iter().any(has_invalid_token);
        if is_invalid {
            return None;
        }

        let is_custom_property = name.starts_with("--");
        if !is_custom_property {
            let is_empty = !value_components
                .iter()
                .any(|v| !matches!(v, ComponentValue::Token(CssToken::Whitespace)));
            if is_empty {
                return None;
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

fn has_invalid_token(cv: &ComponentValue) -> bool {
    matches!(
        cv,
        ComponentValue::Token(CssToken::BadString)
            | ComponentValue::Token(CssToken::BadUrl)
            | ComponentValue::Token(CssToken::RightParen)
            | ComponentValue::Token(CssToken::RightBracket)
            | ComponentValue::Token(CssToken::RightBrace)
    )
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

fn escape_css_identifier(ident: &str) -> String {
    let chars: Vec<char> = ident.chars().collect();
    let len = chars.len();
    let mut result = String::new();

    for (i, &ch) in chars.iter().enumerate() {
        if ch == '\0' {
            result.push('\u{FFFD}');
        } else if ('\u{0001}'..='\u{001F}').contains(&ch) || ch == '\u{007F}' {
            result.push_str(&format!("\\{:x} ", ch as u32));
        } else if ch.is_ascii_digit() {
            if i == 0 || (i == 1 && chars[0] == '-') {
                result.push_str(&format!("\\{:x} ", ch as u32));
            } else {
                result.push(ch);
            }
        } else if ch == '-' {
            if len == 1 {
                result.push_str("\\-");
            } else {
                result.push(ch);
            }
        } else if (ch as u32) < 0x0080 && ch != '_' && ch != '-' && !ch.is_ascii_alphanumeric() {
            result.push('\\');
            result.push(ch);
        } else {
            result.push(ch);
        }
    }
    result
}

fn escape_hash_value(v: &str) -> String {
    let mut s = String::new();
    for c in v.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            s.push(c);
        } else {
            s.push('\\');
            s.push(c);
        }
    }
    s
}

fn serialize_component_values(values: &[ComponentValue]) -> String {
    let mut s = String::new();
    for val in values {
        match val {
            ComponentValue::Token(t) => match t {
                CssToken::Ident(v) => s.push_str(&escape_css_identifier(v)),
                CssToken::Function(v) => {
                    s.push_str(&escape_css_identifier(v));
                    s.push('(');
                }
                CssToken::AtKeyword(v) => {
                    s.push('@');
                    s.push_str(&escape_css_identifier(v));
                }
                CssToken::Hash(v) => {
                    s.push('#');
                    s.push_str(&escape_hash_value(v));
                }
                CssToken::String(v) => {
                    s.push('"');
                    s.push_str(v);
                    s.push('"');
                }
                CssToken::Number(v) => s.push_str(&v.to_string()),
                CssToken::Percentage(v) => {
                    s.push_str(&v.to_string());
                    s.push('%');
                }
                CssToken::Dimension { value, unit } => {
                    s.push_str(&value.to_string());
                    s.push_str(&escape_css_identifier(unit));
                }
                CssToken::Delim(c) => s.push(*c),
                CssToken::Whitespace => s.push(' '),
                CssToken::Colon => s.push(':'),
                CssToken::Semicolon => s.push(';'),
                CssToken::Comma => s.push(','),
                CssToken::LeftBrace => s.push('{'),
                CssToken::RightBrace => s.push('}'),
                CssToken::LeftParen => s.push('('),
                CssToken::RightParen => s.push(')'),
                CssToken::LeftBracket => s.push('['),
                CssToken::RightBracket => s.push(']'),
                CssToken::Cdo => s.push_str("<!--"),
                CssToken::Cdc => s.push_str("-->"),
                CssToken::Url(v) => {
                    s.push_str("url(");
                    s.push_str(v);
                    s.push(')');
                }
                _ => {}
            },
            ComponentValue::Function { name, value } => {
                s.push_str(&escape_css_identifier(name));
                s.push('(');
                s.push_str(&serialize_component_values(value));
                s.push(')');
            }
            ComponentValue::SimpleBlock { associated, value } => {
                s.push(*associated);
                s.push_str(&serialize_component_values(value));
                match associated {
                    '{' => s.push('}'),
                    '[' => s.push(']'),
                    '(' => s.push(')'),
                    _ => {}
                }
            }
        }
    }
    s
}

fn serialize_declaration(decl: &Declaration) -> String {
    let mut s = String::new();
    s.push_str(&escape_css_identifier(&decl.name));
    s.push(':');
    s.push_str(&serialize_component_values(&decl.value));
    if decl.important {
        s.push_str(" !important");
    }
    s.push(';');
    s
}

fn serialize_rule(rule: &Rule) -> String {
    let mut s = String::new();
    match rule {
        Rule::Qualified(q) => {
            s.push_str(&serialize_component_values(&q.prelude));
            s.push('{');
            for decl in &q.declarations {
                s.push_str(&serialize_declaration(decl));
            }
            s.push('}');
        }
        Rule::At(at) => {
            s.push('@');
            s.push_str(&escape_css_identifier(&at.name));
            if !at.prelude.is_empty() {
                s.push(' ');
                s.push_str(&serialize_component_values(&at.prelude));
            }
            if let Some(block) = &at.block {
                s.push('{');
                s.push_str(&serialize_component_values(block));
                s.push('}');
            } else {
                s.push(';');
            }
        }
    }
    s
}

fn split_selector_list(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth_paren = 0;
    let mut depth_bracket = 0;
    let mut depth_brace = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut chars = s.char_indices().peekable();

    while let Some((idx, c)) = chars.next() {
        if in_string {
            if c == '\\' {
                let _ = chars.next();
            } else if c == string_char {
                in_string = false;
            }
        } else {
            match c {
                '\'' | '"' => {
                    in_string = true;
                    string_char = c;
                }
                '\\' => {
                    let _ = chars.next();
                }
                '(' => depth_paren += 1,
                ')' => {
                    if depth_paren > 0 {
                        depth_paren -= 1;
                    }
                }
                '[' => depth_bracket += 1,
                ']' => {
                    if depth_bracket > 0 {
                        depth_bracket -= 1;
                    }
                }
                '{' => depth_brace += 1,
                '}' => {
                    if depth_brace > 0 {
                        depth_brace -= 1;
                    }
                }
                ',' if depth_paren == 0 && depth_bracket == 0 && depth_brace == 0 => {
                    parts.push(&s[start..idx]);
                    start = idx + 1;
                }
                _ => {}
            }
        }
    }
    parts.push(&s[start..]);
    parts
}

fn parse_single_urange(s: &str) -> Option<(u32, u32)> {
    let s = s.trim();
    if s.len() < 3 {
        return None;
    }
    if !s.starts_with('u') && !s.starts_with('U') {
        return None;
    }
    if s.as_bytes()[1] != b'+' {
        return None;
    }
    let suffix = &s[2..];
    if suffix.is_empty() {
        return None;
    }

    if let Some(dash_idx) = suffix.find('-') {
        let start_str = &suffix[..dash_idx];
        let end_str = &suffix[dash_idx + 1..];
        if start_str.is_empty() || start_str.len() > 6 {
            return None;
        }
        if end_str.is_empty() || end_str.len() > 6 {
            return None;
        }
        if !start_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        if !end_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let start = u32::from_str_radix(start_str, 16).ok()?;
        let end = u32::from_str_radix(end_str, 16).ok()?;
        if start > end || end > 0x10FFFF {
            return None;
        }
        Some((start, end))
    } else {
        if suffix.len() > 6 {
            return None;
        }
        let mut has_wildcard = false;
        let mut first_wildcard_idx = None;
        for (i, c) in suffix.chars().enumerate() {
            if c == '?' {
                has_wildcard = true;
                if first_wildcard_idx.is_none() {
                    first_wildcard_idx = Some(i);
                }
            } else {
                if has_wildcard {
                    return None;
                }
                if !c.is_ascii_hexdigit() {
                    return None;
                }
            }
        }

        if let Some(idx) = first_wildcard_idx {
            let hex_part = &suffix[..idx];
            let start_str = format!("{}{}", hex_part, "0".repeat(suffix.len() - idx));
            let end_str = format!("{}{}", hex_part, "f".repeat(suffix.len() - idx));
            let start = u32::from_str_radix(&start_str, 16).ok()?;
            let end = u32::from_str_radix(&end_str, 16).ok()?;
            if end > 0x10FFFF {
                return None;
            }
            Some((start, end))
        } else {
            let val = u32::from_str_radix(suffix, 16).ok()?;
            if val > 0x10FFFF {
                return None;
            }
            Some((val, val))
        }
    }
}

fn serialize_urange_components(values: &[ComponentValue]) -> String {
    let mut s = String::new();
    let mut it = values.iter().peekable();
    while let Some(val) = it.next() {
        match val {
            ComponentValue::Token(t) => match t {
                CssToken::Ident(v) if v.eq_ignore_ascii_case("u") => {
                    s.push_str(v);
                    if let Some(ComponentValue::Token(
                        CssToken::Number(_) | CssToken::Dimension { .. },
                    )) = it.peek()
                    {
                        s.push('+');
                    }
                }
                CssToken::Number(v) => {
                    s.push_str(&v.to_string());
                }
                CssToken::Dimension { value, unit } => {
                    s.push_str(&value.to_string());
                    s.push_str(unit);
                }
                CssToken::Delim(c) => s.push(*c),
                CssToken::Whitespace => s.push(' '),
                CssToken::Comma => s.push(','),
                _ => {
                    s.push_str(&serialize_component_values(std::slice::from_ref(val)));
                }
            },
            _ => {
                s.push_str(&serialize_component_values(std::slice::from_ref(val)));
            }
        }
    }
    s
}

/// Parses a slice of component values into a list of Unicode ranges.
/// spec: <https://drafts.csswg.org/css-fonts/#unicode-range-desc>
pub fn parse_unicode_range(values: &[ComponentValue]) -> Option<Vec<(u32, u32)>> {
    let mut ranges = Vec::new();
    let mut current_part = Vec::new();
    for cv in values {
        if let ComponentValue::Token(CssToken::Comma) = cv {
            if current_part.is_empty() {
                return None;
            }
            let s = serialize_urange_components(&current_part);
            let parsed = parse_single_urange(&s)?;
            ranges.push(parsed);
            current_part.clear();
        } else {
            current_part.push(cv.clone());
        }
    }
    if !current_part.is_empty() {
        let s = serialize_urange_components(&current_part);
        let parsed = parse_single_urange(&s)?;
        ranges.push(parsed);
    } else {
        return None;
    }
    Some(ranges)
}

fn has_unescaped_ampersand(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let _ = chars.next();
        } else if ch == '&' {
            return true;
        }
    }
    false
}

fn replace_unescaped_ampersand(c: &str, p: &str) -> String {
    let mut result = String::new();
    let mut chars = c.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            result.push(ch);
            if let Some(next_ch) = chars.next() {
                result.push(next_ch);
            }
        } else if ch == '&' {
            result.push_str(p);
        } else {
            result.push(ch);
        }
    }
    result
}

fn combine_selectors(parent: &str, child: &str) -> String {
    let parent = parent.trim();
    let child = child.trim();

    if parent.is_empty() {
        return child.to_string();
    }
    if child.is_empty() {
        return parent.to_string();
    }

    let parents: Vec<&str> = split_selector_list(parent)
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let children: Vec<&str> = split_selector_list(child)
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut combined = Vec::new();
    for p in &parents {
        for c in &children {
            if has_unescaped_ampersand(c) {
                combined.push(replace_unescaped_ampersand(c, p));
            } else {
                combined.push(format!("{} {}", p, c));
            }
        }
    }

    combined.join(", ")
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

    #[test]
    fn test_parse_empty_declaration_values_discarded() {
        // Standard properties with empty or whitespace-only values should be discarded.
        let input = "
            div {
                color: ;
                background:   ;
                font-size: !important;
                margin: !important   ;
                --custom-empty: ; /* custom properties are allowed to be empty */
                border: solid; /* normal valid property */
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            // Only --custom-empty and border should be kept.
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "--custom-empty");
            assert_eq!(rule.declarations[1].name, "border");
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_new_standard_parser_entry_points() {
        // 1. test parse_list_of_rules
        let rules_input = "a { color: red; } @media screen {}";
        let rules = parse_list_of_rules(rules_input);
        assert_eq!(rules.len(), 2);
        assert!(matches!(rules[0], Rule::Qualified(_)));
        assert!(matches!(rules[1], Rule::At(_)));

        // 2. test parse_rule
        let rule_ok = "p { margin: 10px; }";
        let parsed_rule_ok = parse_rule(rule_ok);
        assert!(parsed_rule_ok.is_some());
        if let Some(Rule::Qualified(r)) = parsed_rule_ok {
            assert_eq!(r.declarations.len(), 1);
            assert_eq!(r.declarations[0].name, "margin");
        } else {
            panic!("Expected qualified rule");
        }

        // Test whitespace padding on parse_rule
        let rule_ws = "  \n  @media print {}  \t  ";
        let parsed_rule_ws = parse_rule(rule_ws);
        assert!(parsed_rule_ws.is_some());
        if let Some(Rule::At(r)) = parsed_rule_ws {
            assert_eq!(r.name, "media");
        } else {
            panic!("Expected at rule");
        }

        // Test syntax error (extra tokens after rule) on parse_rule
        let rule_err = "p { color: blue; } extra";
        assert!(parse_rule(rule_err).is_none());

        // 3. test parse_declaration
        let decl_ok = "color: blue";
        let parsed_decl = parse_declaration(decl_ok);
        assert!(parsed_decl.is_some());
        if let Some(d) = parsed_decl {
            assert_eq!(d.name, "color");
            assert!(!d.important);
        }

        let decl_important = "margin: 20px !important";
        let parsed_decl_imp = parse_declaration(decl_important);
        assert!(parsed_decl_imp.is_some());
        if let Some(d) = parsed_decl_imp {
            assert_eq!(d.name, "margin");
            assert!(d.important);
        }

        let decl_err = "not-a-declaration";
        assert!(parse_declaration(decl_err).is_none());

        // 4. test parse_list_of_declarations
        let list_input = "color: red; margin: 10px; padding: 5px";
        let decls = parse_list_of_declarations(list_input);
        assert_eq!(decls.len(), 3);
        assert_eq!(decls[0].name, "color");
        assert_eq!(decls[1].name, "margin");
        assert_eq!(decls[2].name, "padding");

        // 5. test parse_comma_separated_list_of_component_values
        let comma_input = "a, b, c";
        let lists = parse_comma_separated_list_of_component_values(comma_input);
        assert_eq!(lists.len(), 3);
        // "a"
        assert_eq!(lists[0].len(), 1);
        // " b" (whitespace + "b")
        assert_eq!(lists[1].len(), 2);
        // " c" (whitespace + "c")
        assert_eq!(lists[2].len(), 2);

        let empty_comma = "";
        let empty_lists = parse_comma_separated_list_of_component_values(empty_comma);
        assert_eq!(empty_lists.len(), 1);
        assert_eq!(empty_lists[0].len(), 0);
    }

    #[test]
    fn test_parse_css_nesting_and_at_rule_nesting() {
        let input = "
            div {
                color: red;
                span {
                    color: blue;
                }
                &:hover {
                    color: green;
                }
                @media (min-width: 100px) {
                    color: yellow;
                }
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 4);

        if let Rule::Qualified(r) = &stylesheet.rules[0] {
            assert_eq!(serialize_component_values(&r.prelude).trim(), "div");
            assert_eq!(r.declarations.len(), 1);
            assert_eq!(r.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule 1");
        }

        if let Rule::Qualified(r) = &stylesheet.rules[1] {
            assert_eq!(serialize_component_values(&r.prelude).trim(), "div span");
            assert_eq!(r.declarations.len(), 1);
            assert_eq!(r.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule 2");
        }

        if let Rule::Qualified(r) = &stylesheet.rules[2] {
            assert_eq!(serialize_component_values(&r.prelude).trim(), "div:hover");
            assert_eq!(r.declarations.len(), 1);
            assert_eq!(r.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule 3");
        }

        if let Rule::At(r) = &stylesheet.rules[3] {
            assert_eq!(r.name, "media");
            assert_eq!(
                serialize_component_values(&r.prelude).trim(),
                "(min-width: 100px)"
            );
            assert!(r.block.is_some());
            let block_str = serialize_component_values(r.block.as_ref().unwrap());
            assert!(block_str.contains("div"));
            assert!(block_str.contains("color: yellow"));
        } else {
            panic!("Expected at-rule 4");
        }
    }

    #[test]
    fn test_selector_list_nesting_commas() {
        let input = "
            div {
                span:not(a, b) {
                    color: blue;
                }
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(stylesheet.rules.len(), 2);
        if let Rule::Qualified(r) = &stylesheet.rules[0] {
            assert_eq!(serialize_component_values(&r.prelude).trim(), "div");
            assert_eq!(r.declarations.len(), 0);
        } else {
            panic!("Expected qualified rule 1");
        }
        if let Rule::Qualified(r) = &stylesheet.rules[1] {
            assert_eq!(
                serialize_component_values(&r.prelude).trim(),
                "div span:not(a, b)"
            );
            assert_eq!(r.declarations.len(), 1);
            assert_eq!(r.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule 2");
        }
    }

    #[test]
    fn test_unicode_range_parsing() {
        let single = parse_component_values("U+26");
        let parsed_single = parse_unicode_range(&single).unwrap();
        assert_eq!(parsed_single, vec![(38, 38)]);

        let range = parse_component_values("U+0025-00FF");
        let parsed_range = parse_unicode_range(&range).unwrap();
        assert_eq!(parsed_range, vec![(37, 255)]);

        let wildcard = parse_component_values("U+4??");
        let parsed_wildcard = parse_unicode_range(&wildcard).unwrap();
        assert_eq!(parsed_wildcard, vec![(1024, 1279)]);

        let list = parse_component_values("U+26, U+0025-00FF, U+4??");
        let parsed_list = parse_unicode_range(&list).unwrap();
        assert_eq!(parsed_list, vec![(38, 38), (37, 255), (1024, 1279)]);

        let wildcards_only = parse_component_values("U+????");
        let parsed_wildcards_only = parse_unicode_range(&wildcards_only).unwrap();
        assert_eq!(parsed_wildcards_only, vec![(0, 65535)]);

        let invalid = parse_component_values("U+110000");
        assert!(parse_unicode_range(&invalid).is_none());

        let invalid_dash = parse_component_values("U+26-25-24");
        assert!(parse_unicode_range(&invalid_dash).is_none());
    }

    #[test]
    fn test_parse_nested_at_rule_with_nested_style_rule() {
        let input = "
            div {
                @media (min-width: 100px) {
                    color: yellow;
                    span {
                        color: blue;
                    }
                }
            }
        ";
        let stylesheet = parse_stylesheet(input);
        assert_eq!(
            stylesheet.rules.len(),
            2,
            "Should only have div and @media rule at the top level"
        );

        if let Rule::Qualified(r) = &stylesheet.rules[0] {
            assert_eq!(serialize_component_values(&r.prelude).trim(), "div");
            assert_eq!(r.declarations.len(), 0);
        } else {
            panic!("Expected qualified rule 1 to be div");
        }

        if let Rule::At(r) = &stylesheet.rules[1] {
            assert_eq!(r.name, "media");
            assert_eq!(
                serialize_component_values(&r.prelude).trim(),
                "(min-width: 100px)"
            );
            assert!(r.block.is_some());
            let block_str = serialize_component_values(r.block.as_ref().unwrap())
                .trim()
                .to_string();
            assert!(
                block_str.contains("div {color: yellow;}"),
                "Block should contain div's yellow declaration: {}",
                block_str
            );
            assert!(
                block_str.contains("div span{color: blue;}"),
                "Block should contain nested style rule div span: {}",
                block_str
            );
        } else {
            panic!("Expected at-rule @media");
        }
    }

    #[test]
    fn test_css_parser_robustness_t1008() {
        // 1. Error recovery on malformed declarations
        // Semicolon padding and garbage recovery
        let input_err = "
            div {
                color: red;;;
                garbage-no-colon;
                123: blue;
                background: blue;
            }
        ";
        let stylesheet = parse_stylesheet(input_err);
        assert_eq!(stylesheet.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(rule.declarations.len(), 2);
            assert_eq!(rule.declarations[0].name, "color");
            assert_eq!(rule.declarations[1].name, "background");
        } else {
            panic!("Expected qualified rule");
        }

        // Direct parse_declaration with trailing semicolon and whitespace
        let decl1 = parse_declaration("color: red;");
        assert!(decl1.is_some());
        assert_eq!(decl1.unwrap().name, "color");

        let decl2 = parse_declaration("margin: 10px !important ; ");
        assert!(decl2.is_some());
        let d2 = decl2.unwrap();
        assert_eq!(d2.name, "margin");
        assert!(d2.important);

        // 2. !important handling
        // Whitespace between ! and important
        let decl3 = parse_declaration("color: red !   important");
        assert!(decl3.is_some());
        let d3 = decl3.unwrap();
        assert_eq!(d3.name, "color");
        assert!(d3.important);

        // Custom property with !important
        let custom_decl = parse_declaration("--my-color: blue !important");
        assert!(custom_decl.is_some());
        let cd = custom_decl.unwrap();
        assert_eq!(cd.name, "--my-color");
        assert!(cd.important);
        // check that !important is stripped from custom property value
        let val_str = serialize_component_values(&cd.value);
        assert!(!val_str.contains("important"));
        assert!(val_str.contains("blue"));

        // 3. Comment/whitespace tokenization edge cases
        let comment_input = "
            div {
                color/* comment */:/* comment */red;
            }
            /* trailing unclosed comment
        ";
        let stylesheet_comment = parse_stylesheet(comment_input);
        assert_eq!(stylesheet_comment.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_comment.rules[0] {
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule");
        }

        // 4. Escaped identifiers and combine_selectors with escaped ampersand
        let escaped_input = "
            div {
                \\&:hover {
                    color: green;
                }
            }
        ";
        let stylesheet_escaped = parse_stylesheet(escaped_input);
        // The nested rule should combined into "div \&:hover", NOT "div:hover"
        assert_eq!(stylesheet_escaped.rules.len(), 2);
        if let Rule::Qualified(rule) = &stylesheet_escaped.rules[1] {
            let prelude_str = serialize_component_values(&rule.prelude);
            assert!(prelude_str.contains("\\&"));
            assert!(!prelude_str.contains("div:hover"));
        } else {
            panic!("Expected qualified rule");
        }

        // 5. At-rule prelude parsing
        let at_input = "@media (min-width: 100px), screen { div { color: red; } }";
        let stylesheet_at = parse_stylesheet(at_input);
        assert_eq!(stylesheet_at.rules.len(), 1);
        if let Rule::At(rule) = &stylesheet_at.rules[0] {
            assert_eq!(rule.name, "media");
            let prelude_str = serialize_component_values(&rule.prelude);
            assert_eq!(prelude_str.trim(), "(min-width: 100px), screen");
        } else {
            panic!("Expected at-rule");
        }

        // 6. Custom-property value preservation
        let custom_preserve = "
            div {
                --complex: var(--another, { color: red; });
            }
        ";
        let stylesheet_cp = parse_stylesheet(custom_preserve);
        assert_eq!(stylesheet_cp.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_cp.rules[0] {
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "--complex");
            let val_str = serialize_component_values(&rule.declarations[0].value);
            assert!(val_str.contains("var"));
            assert!(val_str.contains("color: red;"));
        } else {
            panic!("Expected qualified rule");
        }
    }

    #[test]
    fn test_css_parser_robustness_t1027() {
        // 1. Blockless nested at-rules (e.g. @import or @charset) inside a style rule
        let input_blockless_at = "
            div {
                @import url(\"foo.css\");
                color: red;
            }
        ";
        let stylesheet = parse_stylesheet(input_blockless_at);
        // Should produce two rules: the qualified rule (div { color: red; }) and the hoisted nested at-rule (@import)
        assert_eq!(stylesheet.rules.len(), 2);
        if let Rule::Qualified(rule) = &stylesheet.rules[0] {
            assert_eq!(serialize_component_values(&rule.prelude).trim(), "div");
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "color");
        } else {
            panic!("Expected qualified rule");
        }
        if let Rule::At(rule) = &stylesheet.rules[1] {
            assert_eq!(rule.name, "import");
            assert!(rule.block.is_none());
            let prelude_str = serialize_component_values(&rule.prelude);
            assert!(prelude_str.contains("url"));
        } else {
            panic!("Expected at-rule");
        }

        // 2. Bad-declaration recovery & stray brackets
        let input_stray_brackets = "
            div {
                color: rgb(255, 0, 0) );
                background: blue;
            }
        ";
        let stylesheet_stray = parse_stylesheet(input_stray_brackets);
        assert_eq!(stylesheet_stray.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_stray.rules[0] {
            // "color" has a stray top-level RightParen, so it is discarded as invalid. "background" is preserved.
            assert_eq!(rule.declarations.len(), 1);
            assert_eq!(rule.declarations[0].name, "background");
        } else {
            panic!("Expected qualified rule");
        }

        // 3. Balanced blocks containing various nested brackets inside custom properties
        let input_mismatched_inner = "
            div {
                --mismatched: ( [ ] );
                --another: { [ ] };
                color: green;
            }
        ";
        let stylesheet_mismatched = parse_stylesheet(input_mismatched_inner);
        assert_eq!(stylesheet_mismatched.rules.len(), 1);
        if let Rule::Qualified(rule) = &stylesheet_mismatched.rules[0] {
            // These nested brackets are balanced and considered valid components at top level
            assert_eq!(rule.declarations.len(), 3);
            assert_eq!(rule.declarations[0].name, "--mismatched");
            assert_eq!(rule.declarations[1].name, "--another");
            assert_eq!(rule.declarations[2].name, "color");
        } else {
            panic!("Expected qualified rule");
        }
    }
}
