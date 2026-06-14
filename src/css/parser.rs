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
                            self.nested_rules.extend(sub_parser.nested_rules);

                            let nested_qual = QualifiedRule {
                                prelude: parent_prelude.to_vec(),
                                declarations: sub_decls,
                            };
                            let nested_css = serialize_rule(&Rule::Qualified(nested_qual));
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

fn serialize_component_values(values: &[ComponentValue]) -> String {
    let mut s = String::new();
    for val in values {
        match val {
            ComponentValue::Token(t) => match t {
                CssToken::Ident(v) => s.push_str(v),
                CssToken::Function(v) => {
                    s.push_str(v);
                    s.push('(');
                }
                CssToken::AtKeyword(v) => {
                    s.push('@');
                    s.push_str(v);
                }
                CssToken::Hash(v) => {
                    s.push('#');
                    s.push_str(v);
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
                    s.push_str(unit);
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
                s.push_str(name);
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
    s.push_str(&decl.name);
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
            s.push_str(&at.name);
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

fn combine_selectors(parent: &str, child: &str) -> String {
    let parent = parent.trim();
    let child = child.trim();

    if parent.is_empty() {
        return child.to_string();
    }
    if child.is_empty() {
        return parent.to_string();
    }

    let parents: Vec<&str> = parent
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let children: Vec<&str> = child
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut combined = Vec::new();
    for p in &parents {
        for c in &children {
            if c.contains('&') {
                combined.push(c.replace('&', p));
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
}
