#![forbid(unsafe_code)]
#![allow(dead_code)]

// spec: https://www.w3.org/TR/css-conditional-3/
// spec: https://www.w3.org/TR/mediaqueries-4/

use crate::css::parser::{ComponentValue, QualifiedRule, Rule, Stylesheet};
use crate::css::{CssToken, CssTokenizer};
use std::cell::Cell;

/// Represents the preferred color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

thread_local! {
    static PREFERRED_COLOR_SCHEME: Cell<ColorScheme> = const { Cell::new(ColorScheme::Light) };
}

/// Sets the preferred color scheme for the current thread.
pub fn set_preferred_color_scheme(scheme: ColorScheme) {
    PREFERRED_COLOR_SCHEME.with(|c| c.set(scheme));
}

/// Gets the preferred color scheme for the current thread.
pub fn preferred_color_scheme() -> ColorScheme {
    PREFERRED_COLOR_SCHEME.with(|c| c.get())
}

/// Serializes component values back to a CSS string.
pub fn serialize_component_values(values: &[ComponentValue]) -> String {
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

/// Helper to check if a token is a case-insensitive identifier matching `name`.
fn is_ident(token: &CssToken, name: &str) -> bool {
    if let CssToken::Ident(s) = token {
        s.eq_ignore_ascii_case(name)
    } else {
        false
    }
}

/// Helper to split a slice of tokens on top-level commas.
fn split_by_comma(tokens: &[CssToken]) -> Vec<Vec<CssToken>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0;
    for token in tokens {
        match token {
            CssToken::LeftParen | CssToken::LeftBrace | CssToken::LeftBracket => {
                depth += 1;
                current.push(token.clone());
            }
            CssToken::RightParen | CssToken::RightBrace | CssToken::RightBracket => {
                if depth > 0 {
                    depth -= 1;
                }
                current.push(token.clone());
            }
            CssToken::Comma if depth == 0 => {
                result.push(std::mem::take(&mut current));
            }
            CssToken::Comma => {
                current.push(token.clone());
            }
            _ => {
                current.push(token.clone());
            }
        }
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// Evaluates a single media query (a list of non-whitespace tokens) against the viewport width.
fn evaluate_single_query(tokens: &[CssToken], viewport_w: f32) -> bool {
    let mut idx = 0;
    let mut is_negated = false;

    if idx < tokens.len() && is_ident(&tokens[idx], "not") {
        is_negated = true;
        idx += 1;
    }

    if idx < tokens.len() && is_ident(&tokens[idx], "only") {
        idx += 1;
    }

    let mut matches = true;
    let mut expect_and = false;

    // Check if the current token is a media type or an expression
    if idx < tokens.len() {
        if let CssToken::LeftParen = &tokens[idx] {
            // No media type specified, defaults to "all" (which is true)
            expect_and = false;
        } else if let CssToken::Ident(name) = &tokens[idx] {
            let media_type = name.to_ascii_lowercase();
            if media_type == "screen" || media_type == "all" {
                // matches true
            } else if media_type == "print" || media_type == "speech" {
                matches = false;
            } else {
                // unrecognized media type
                matches = false;
            }
            idx += 1;
            expect_and = true;
        } else {
            // Invalid starting token
            matches = false;
        }
    }

    // Now loop through expressions
    while idx < tokens.len() {
        if expect_and {
            if is_ident(&tokens[idx], "and") {
                idx += 1;
            } else {
                // Expected "and" but got something else (invalid query)
                matches = false;
                break;
            }
        } else {
            // The first feature expression doesn't need "and"
            expect_and = true;
        }

        if idx < tokens.len() && matches!(tokens[idx], CssToken::LeftParen) {
            // Find matching RightParen
            let start_expr = idx + 1;
            let mut end_expr = start_expr;
            let mut depth = 1;
            while end_expr < tokens.len() && depth > 0 {
                match &tokens[end_expr] {
                    CssToken::LeftParen => depth += 1,
                    CssToken::RightParen => depth -= 1,
                    _ => {}
                }
                if depth > 0 {
                    end_expr += 1;
                }
            }

            if end_expr < tokens.len() && depth == 0 {
                let expr_tokens = &tokens[start_expr..end_expr];
                if !evaluate_feature(expr_tokens, viewport_w) {
                    matches = false;
                }
                idx = end_expr + 1;
            } else {
                // Mismatched parentheses
                matches = false;
                break;
            }
        } else {
            matches = false;
            break;
        }
    }

    if is_negated { !matches } else { matches }
}

/// Evaluates a single media feature, e.g., max-width: 600px.
fn evaluate_feature(tokens: &[CssToken], viewport_w: f32) -> bool {
    if tokens.is_empty() {
        return false;
    }

    let feature_name = if let CssToken::Ident(name) = &tokens[0] {
        name.to_ascii_lowercase()
    } else {
        return false;
    };

    if tokens.len() == 1 {
        match feature_name.as_str() {
            "prefers-color-scheme" => return true,
            _ => return false,
        }
    }

    if tokens.len() < 3 {
        return false;
    }

    if !matches!(tokens[1], CssToken::Colon) {
        return false;
    }

    if feature_name == "prefers-color-scheme" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = preferred_color_scheme();
            match (current, val_lower.as_str()) {
                (ColorScheme::Light, "light") => return true,
                (ColorScheme::Dark, "dark") => return true,
                _ => return false,
            }
        }
        return false;
    }

    let value_px = match &tokens[2] {
        CssToken::Dimension { value, unit } => {
            if unit.eq_ignore_ascii_case("px") {
                Some(*value as f32)
            } else if unit.eq_ignore_ascii_case("em") || unit.eq_ignore_ascii_case("rem") {
                Some((*value * 16.0) as f32)
            } else {
                None
            }
        }
        CssToken::Number(value) => Some(*value as f32),
        _ => None,
    };

    match feature_name.as_str() {
        "min-width" => value_px.is_some_and(|limit| viewport_w >= limit),
        "max-width" => value_px.is_some_and(|limit| viewport_w <= limit),
        "width" => value_px.is_some_and(|limit| (viewport_w - limit).abs() < 1e-5),
        _ => {
            // TODO(spec): other media features
            false
        }
    }
}

/// Evaluates a media query string against the given viewport width.
// spec: https://www.w3.org/TR/mediaqueries-4/#evaluation
pub fn media_matches(query: &str, viewport_w: f32) -> bool {
    let query_trimmed = query.trim();
    if query_trimmed.is_empty() {
        return true; // Default to true if empty
    }

    let mut tokenizer = CssTokenizer::new(query_trimmed);
    let mut tokens = Vec::new();
    loop {
        let token = tokenizer.next_token();
        if token == CssToken::Eof {
            break;
        }
        tokens.push(token);
    }

    if tokens.is_empty() {
        return true;
    }

    let sub_queries_tokens = split_by_comma(&tokens);
    for sq_tokens in sub_queries_tokens {
        // Filter out whitespace tokens
        let filtered_tokens: Vec<CssToken> = sq_tokens
            .into_iter()
            .filter(|t| !matches!(t, CssToken::Whitespace))
            .collect();

        if filtered_tokens.is_empty() {
            continue;
        }

        if evaluate_single_query(&filtered_tokens, viewport_w) {
            return true; // Comma acts as logical OR
        }
    }

    false
}

/// Helper to filter out top-level whitespace from a list of component values.
fn clean_values(values: &[ComponentValue]) -> Vec<&ComponentValue> {
    values
        .iter()
        .filter(|v| !matches!(v, ComponentValue::Token(CssToken::Whitespace)))
        .collect()
}

/// Helper to check if a component value is a case-insensitive identifier matching `keyword`.
fn is_keyword(cv: &ComponentValue, keyword: &str) -> bool {
    if let ComponentValue::Token(t) = cv {
        is_ident(t, keyword)
    } else {
        false
    }
}

/// Splits a slice of component values on a top-level keyword (like "and" or "or").
fn split_by_keyword<'a>(
    values: &[&'a ComponentValue],
    keyword: &str,
) -> Vec<Vec<&'a ComponentValue>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for &cv in values {
        if is_keyword(cv, keyword) {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(cv);
        }
    }
    parts.push(current);
    parts
}

/// Evaluates a CSS `@supports` condition (represented as a slice of `ComponentValue` pointers).
fn evaluate_supports_condition(values: &[&ComponentValue]) -> bool {
    if values.is_empty() {
        return false;
    }

    // 1. Check for 'and' and 'or' combinators at this level
    let mut has_and = false;
    let mut has_or = false;
    for &cv in values {
        if is_keyword(cv, "and") {
            has_and = true;
        }
        if is_keyword(cv, "or") {
            has_or = true;
        }
    }

    if has_and && has_or {
        // Mixing 'and' and 'or' without parentheses is invalid
        return false;
    }

    if has_and {
        let parts = split_by_keyword(values, "and");
        for part in &parts {
            if part.is_empty() {
                return false;
            }
        }
        return parts.iter().all(|part| evaluate_supports_condition(part));
    }

    if has_or {
        let parts = split_by_keyword(values, "or");
        for part in &parts {
            if part.is_empty() {
                return false;
            }
        }
        return parts.iter().any(|part| evaluate_supports_condition(part));
    }

    // 2. Check for negation
    if is_keyword(values[0], "not") {
        if values.len() < 2 {
            return false;
        }
        // Negate the evaluation of the rest of the condition
        return !evaluate_supports_condition(&values[1..]);
    }

    // 3. Single `<supports-in-parens>` operand
    if values.len() == 1 {
        if let ComponentValue::SimpleBlock {
            associated: '(',
            value: inner_values,
        } = values[0]
        {
            // Find the first top-level colon inside this parenthesis block.
            if let Some(colon_idx) = inner_values
                .iter()
                .position(|cv| matches!(cv, ComponentValue::Token(CssToken::Colon)))
            {
                // Before colon: property name (must be exactly one Ident, ignoring Whitespace)
                let before_colon = &inner_values[..colon_idx];
                let name_tokens: Vec<&ComponentValue> = before_colon
                    .iter()
                    .filter(|cv| !matches!(cv, ComponentValue::Token(CssToken::Whitespace)))
                    .collect();

                if name_tokens.len() == 1
                    && let ComponentValue::Token(CssToken::Ident(prop_name)) = name_tokens[0]
                {
                    let name = prop_name.trim();
                    if name.is_empty() {
                        return false;
                    }
                    let is_recognized = crate::css::property::lookup(name).is_some()
                        || crate::css::property::shorthand_longhands(name).is_some();
                    if !is_recognized {
                        return false;
                    }

                    // After colon: value (keep inner whitespace, trim leading/trailing whitespace)
                    let after_colon = &inner_values[colon_idx + 1..];
                    let mut start = 0;
                    while start < after_colon.len()
                        && matches!(
                            after_colon[start],
                            ComponentValue::Token(CssToken::Whitespace)
                        )
                    {
                        start += 1;
                    }
                    let mut end = after_colon.len();
                    while end > start
                        && matches!(
                            after_colon[end - 1],
                            ComponentValue::Token(CssToken::Whitespace)
                        )
                    {
                        end -= 1;
                    }

                    let val_components: Vec<ComponentValue> = after_colon[start..end].to_vec();
                    return crate::css::values::parse_property_value(name, &val_components)
                        .is_some();
                }
            }

            // Otherwise, it must be a nested `<supports-condition>`
            let cleaned_inner = clean_values(inner_values);
            return evaluate_supports_condition(&cleaned_inner);
        } else {
            // TODO(spec): selector(...) / font-tech(...) / font-format(...) / other general-enclosed
            return false;
        }
    }

    false
}

/// Evaluates a supports condition string.
pub fn supports_condition_matches(condition: &str) -> bool {
    let components = crate::css::parser::parse_component_values(condition);
    let cleaned = clean_values(&components);
    evaluate_supports_condition(&cleaned)
}

/// Hostile stylesheets can nest @media rules arbitrarily deep.
/// To prevent resource exhaustion and stack overflow, we restrict depth.
const MAX_MEDIA_NEST_DEPTH: usize = 32;

enum RulesSource<'a> {
    Borrowed(&'a [Rule]),
    Owned(Vec<Rule>),
}

impl<'a> RulesSource<'a> {
    fn as_slice(&self) -> &[Rule] {
        match self {
            RulesSource::Borrowed(s) => s,
            RulesSource::Owned(v) => v,
        }
    }
}

struct Frame<'a> {
    rules: RulesSource<'a>,
    index: usize,
    depth: usize,
}

/// Iteratively extracts all active qualified rules from a stylesheet under the given viewport width.
pub fn extract_matched_rules(stylesheet: &Stylesheet, viewport_w: f32) -> Vec<QualifiedRule> {
    let mut matched = Vec::new();
    let mut stack = vec![Frame {
        rules: RulesSource::Borrowed(&stylesheet.rules),
        index: 0,
        depth: 0,
    }];

    while let Some(frame) = stack.last_mut() {
        let rules_slice = frame.rules.as_slice();
        if frame.index >= rules_slice.len() {
            stack.pop();
            continue;
        }

        let rule = &rules_slice[frame.index];
        frame.index += 1;

        match rule {
            Rule::Qualified(qualified) => {
                matched.push(qualified.clone());
            }
            Rule::At(at_rule) if at_rule.name == "media" => {
                let query_str = serialize_component_values(&at_rule.prelude);
                if media_matches(&query_str, viewport_w)
                    && let Some(block) = &at_rule.block
                {
                    let next_depth = frame.depth + 1;
                    if next_depth > MAX_MEDIA_NEST_DEPTH {
                        eprintln!(
                            "css: @media nesting exceeded {MAX_MEDIA_NEST_DEPTH}, skipping deeper rules"
                        );
                        continue;
                    }

                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    stack.push(Frame {
                        rules: RulesSource::Owned(inner_stylesheet.rules),
                        index: 0,
                        depth: next_depth,
                    });
                }
            }
            Rule::At(at_rule) if at_rule.name == "supports" => {
                let cleaned_prelude = clean_values(&at_rule.prelude);
                if evaluate_supports_condition(&cleaned_prelude)
                    && let Some(block) = &at_rule.block
                {
                    let next_depth = frame.depth + 1;
                    if next_depth > MAX_MEDIA_NEST_DEPTH {
                        eprintln!(
                            "css: @supports nesting exceeded {MAX_MEDIA_NEST_DEPTH}, skipping deeper rules"
                        );
                        continue;
                    }

                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    stack.push(Frame {
                        rules: RulesSource::Owned(inner_stylesheet.rules),
                        index: 0,
                        depth: next_depth,
                    });
                }
            }
            _ => {
                // Ignore other at-rules
            }
        }
    }

    matched
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_matches_basic() {
        // min-width
        assert!(media_matches("(min-width: 600px)", 700.0));
        assert!(media_matches("(min-width: 600px)", 600.0));
        assert!(!media_matches("(min-width: 600px)", 500.0));

        // max-width
        assert!(media_matches("(max-width: 600px)", 500.0));
        assert!(media_matches("(max-width: 600px)", 600.0));
        assert!(!media_matches("(max-width: 600px)", 700.0));

        // width
        assert!(media_matches("(width: 600px)", 600.0));
        assert!(!media_matches("(width: 600px)", 601.0));
    }

    #[test]
    fn test_media_matches_media_type() {
        assert!(media_matches("screen and (max-width: 600px)", 500.0));
        assert!(!media_matches("screen and (max-width: 600px)", 700.0));
        assert!(media_matches("all and (max-width: 600px)", 500.0));
        assert!(!media_matches("print and (max-width: 600px)", 500.0));
    }

    #[test]
    fn test_media_matches_comma_or() {
        assert!(media_matches(
            "(max-width: 600px), (min-width: 1000px)",
            500.0
        ));
        assert!(!media_matches(
            "(max-width: 600px), (min-width: 1000px)",
            800.0
        ));
        assert!(media_matches(
            "(max-width: 600px), (min-width: 1000px)",
            1200.0
        ));
    }

    #[test]
    fn test_media_matches_and() {
        assert!(media_matches(
            "(min-width: 400px) and (max-width: 600px)",
            500.0
        ));
        assert!(!media_matches(
            "(min-width: 400px) and (max-width: 600px)",
            300.0
        ));
        assert!(!media_matches(
            "(min-width: 400px) and (max-width: 600px)",
            700.0
        ));
    }

    #[test]
    fn test_media_matches_negation() {
        assert!(!media_matches("not screen and (max-width: 600px)", 500.0));
        assert!(media_matches("not screen and (max-width: 600px)", 700.0));
    }

    #[test]
    fn test_extract_matched_rules_basic() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            div { color: blue; }
            @media (max-width: 600px) {
                span { color: red; }
            }
        ",
        );

        // At 500.0 width, both div and span rules match
        let matched_500 = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched_500.len(), 2);
        assert_eq!(serialize_component_values(&matched_500[0].prelude), "div ");
        assert_eq!(serialize_component_values(&matched_500[1].prelude), "span ");

        // At 700.0 width, only div rule matches
        let matched_700 = extract_matched_rules(&stylesheet, 700.0);
        assert_eq!(matched_700.len(), 1);
        assert_eq!(serialize_component_values(&matched_700[0].prelude), "div ");
    }

    #[test]
    fn test_extract_matched_rules_nested() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            @media (min-width: 300px) {
                @media (max-width: 600px) {
                    p { color: green; }
                }
            }
        ",
        );

        // At 500.0 width, the nested p rule matches
        let matched_500 = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched_500.len(), 1);
        assert_eq!(serialize_component_values(&matched_500[0].prelude), "p ");

        // At 200.0 width, nested p rule does not match
        let matched_200 = extract_matched_rules(&stylesheet, 200.0);
        assert!(matched_200.is_empty());

        // At 700.0 width, nested p rule does not match
        let matched_700 = extract_matched_rules(&stylesheet, 700.0);
        assert!(matched_700.is_empty());
    }

    #[test]
    fn test_extract_nested_media_preserves_order() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            .top-start { color: red; }
            @media (min-width: 1px) {
                .inner-1 { color: green; }
                .inner-2 { color: blue; }
            }
            .top-end { color: yellow; }
        ",
        );

        let matched = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched.len(), 4);
        assert_eq!(
            serialize_component_values(&matched[0].prelude),
            ".top-start "
        );
        assert_eq!(serialize_component_values(&matched[1].prelude), ".inner-1 ");
        assert_eq!(serialize_component_values(&matched[2].prelude), ".inner-2 ");
        assert_eq!(serialize_component_values(&matched[3].prelude), ".top-end ");
    }

    #[test]
    fn test_extract_unmatched_media_skipped() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            @media (min-width: 1px) {
                .outer { color: red; }
                @media (min-width: 1000px) {
                    .inner { color: green; }
                }
            }
        ",
        );

        let matched = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched.len(), 1);
        assert_eq!(serialize_component_values(&matched[0].prelude), ".outer ");
    }

    #[test]
    fn test_extract_deeply_nested_media_no_overflow() {
        let mut css = String::new();
        for _ in 0..2000 {
            css.push_str("@media (min-width: 1px) { ");
        }
        css.push_str(".deepest { color: red; }");
        for _ in 0..2000 {
            css.push('}');
        }

        let stylesheet = crate::css::parser::parse_stylesheet(&css);
        let matched = extract_matched_rules(&stylesheet, 500.0);
        // Assert it successfully executed without stack overflow.
        // It might be empty or some rules depending on depth guard, which is correct.
        let _ = matched;
    }

    #[test]
    fn test_prefers_color_scheme_default() {
        // Default is light
        assert!(media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(!media_matches("(prefers-color-scheme: dark)", 1000.0));
        // Boolean context
        assert!(media_matches("(prefers-color-scheme)", 1000.0));
    }

    #[test]
    fn test_prefers_color_scheme_configured() {
        // Set to dark
        set_preferred_color_scheme(ColorScheme::Dark);
        assert!(!media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(media_matches("(prefers-color-scheme: dark)", 1000.0));
        assert!(media_matches("(prefers-color-scheme)", 1000.0));

        // Set back to light
        set_preferred_color_scheme(ColorScheme::Light);
        assert!(media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(!media_matches("(prefers-color-scheme: dark)", 1000.0));
        assert!(media_matches("(prefers-color-scheme)", 1000.0));
    }

    #[test]
    fn test_prefers_color_scheme_case_insensitive() {
        set_preferred_color_scheme(ColorScheme::Dark);
        assert!(media_matches("(PREFERS-COLOR-SCHEME: DaRk)", 1000.0));
        assert!(!media_matches("(PREFERS-COLOR-SCHEME: LiGhT)", 1000.0));
        // Reset to default
        set_preferred_color_scheme(ColorScheme::Light);
    }

    #[test]
    fn test_supports_condition_matches_basic() {
        // Supported basic feature
        assert!(supports_condition_matches("(color: red)"));
        assert!(supports_condition_matches("(display: block)"));

        // Unsupported basic feature (unknown property name)
        assert!(!supports_condition_matches("(totally-not-a-prop: 5px)"));

        // Negated unsupported feature -> true
        assert!(supports_condition_matches("not (totally-not-a-prop: 5px)"));

        // Negated supported feature -> false
        assert!(!supports_condition_matches("not (color: red)"));

        // Conjunction (and):
        // true and true -> true
        assert!(supports_condition_matches(
            "(color: red) and (display: block)"
        ));
        // true and false -> false
        assert!(!supports_condition_matches(
            "(color: red) and (totally-not-a-prop: 5px)"
        ));
        // false and false -> false
        assert!(!supports_condition_matches(
            "(totally-not-a-prop: 5px) and (totally-not-another-prop: 10px)"
        ));

        // Disjunction (or):
        // true or false -> true
        assert!(supports_condition_matches(
            "(color: red) or (totally-not-a-prop: 5px)"
        ));
        // false or false -> false
        assert!(!supports_condition_matches(
            "(totally-not-a-prop: 5px) or (totally-not-another-prop: 10px)"
        ));

        // Nesting and complex combinations:
        assert!(supports_condition_matches(
            "((color: red) and (display: block))"
        ));
        assert!(supports_condition_matches(
            "not ((color: red) and (totally-not-a-prop: 5px))"
        ));
    }

    #[test]
    fn test_extract_matched_rules_supports() {
        // 1. @supports (color: red) { div { color: red; } } -> div rule is returned
        let stylesheet1 =
            crate::css::parser::parse_stylesheet("@supports (color: red) { div { color: red; } }");
        let matched1 = extract_matched_rules(&stylesheet1, 1000.0);
        assert_eq!(matched1.len(), 1);
        assert_eq!(serialize_component_values(&matched1[0].prelude), "div ");

        // 2. @supports (totally-not-a-prop: 5px) { div { color: red } } -> div rule is NOT returned
        let stylesheet2 = crate::css::parser::parse_stylesheet(
            "@supports (totally-not-a-prop: 5px) { div { color: red; } }",
        );
        let matched2 = extract_matched_rules(&stylesheet2, 1000.0);
        assert!(matched2.is_empty());

        // 3. @supports not (totally-not-a-prop: 5px) { div { color: red } } -> div rule IS returned
        let stylesheet3 = crate::css::parser::parse_stylesheet(
            "@supports not (totally-not-a-prop: 5px) { div { color: red; } }",
        );
        let matched3 = extract_matched_rules(&stylesheet3, 1000.0);
        assert_eq!(matched3.len(), 1);
        assert_eq!(serialize_component_values(&matched3[0].prelude), "div ");

        // 4. Conjunction and disjunction nested rules
        let stylesheet4 = crate::css::parser::parse_stylesheet(
            "
            @supports (color: red) and (totally-not-a-prop: 5px) {
                span { color: green; }
            }
            @supports (color: red) or (totally-not-a-prop: 5px) {
                p { color: blue; }
            }
            ",
        );
        let matched4 = extract_matched_rules(&stylesheet4, 1000.0);
        assert_eq!(matched4.len(), 1);
        assert_eq!(serialize_component_values(&matched4[0].prelude), "p ");
    }
}
