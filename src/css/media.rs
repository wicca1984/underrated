#![forbid(unsafe_code)]
#![allow(dead_code)]

// spec: https://www.w3.org/TR/css-conditional-3/
// spec: https://www.w3.org/TR/mediaqueries-4/

use crate::css::parser::{ComponentValue, QualifiedRule, Rule, Stylesheet};
use crate::css::{CssToken, CssTokenizer};

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
    if tokens.len() < 3 {
        return false;
    }

    let feature_name = if let CssToken::Ident(name) = &tokens[0] {
        name.to_ascii_lowercase()
    } else {
        return false;
    };

    if !matches!(tokens[1], CssToken::Colon) {
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

/// Recursively extracts all active qualified rules from a stylesheet under the given viewport width.
pub fn extract_matched_rules(stylesheet: &Stylesheet, viewport_w: f32) -> Vec<QualifiedRule> {
    let mut matched = Vec::new();
    extract_matched_rules_recursive(&stylesheet.rules, viewport_w, &mut matched);
    matched
}

fn extract_matched_rules_recursive(
    rules: &[Rule],
    viewport_w: f32,
    matched: &mut Vec<QualifiedRule>,
) {
    for rule in rules {
        match rule {
            Rule::Qualified(qualified) => {
                matched.push(qualified.clone());
            }
            Rule::At(at_rule) if at_rule.name == "media" => {
                let query_str = serialize_component_values(&at_rule.prelude);
                if media_matches(&query_str, viewport_w)
                    && let Some(block) = &at_rule.block
                {
                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    extract_matched_rules_recursive(&inner_stylesheet.rules, viewport_w, matched);
                }
            }
            _ => {
                // Ignore other at-rules
            }
        }
    }
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
}
