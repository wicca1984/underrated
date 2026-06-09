//! CSS Resolution helpers for S-41 (calc, relative units, and custom variables).
//!
//! spec: <https://www.w3.org/TR/css-values-4/>

use crate::css::CssToken;
use crate::css::parser::ComponentValue;
use crate::css::values::{CssValue, LengthUnit};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CalcValue {
    Length(f32),
    Number(f32),
}

#[derive(Debug, Clone, PartialEq)]
enum CalcToken {
    Val(CalcValue),
    Plus,
    Minus,
    Mul,
    Div,
    LeftParen,
    RightParen,
}

/// Helper to parse a `var()` function's arguments.
/// format: `var( <custom-property-name> [, <declaration-value>]? )`
fn parse_var_function(components: &[ComponentValue]) -> Option<(&str, Option<&[ComponentValue]>)> {
    let mut non_ws_components = Vec::new();
    for comp in components {
        if !matches!(comp, ComponentValue::Token(CssToken::Whitespace)) {
            non_ws_components.push(comp);
        }
    }

    if non_ws_components.is_empty() {
        return None;
    }

    let var_name = match non_ws_components[0] {
        ComponentValue::Token(CssToken::Ident(name)) if name.starts_with("--") => name.as_str(),
        _ => return None,
    };

    if non_ws_components.len() == 1 {
        return Some((var_name, None));
    }

    if matches!(non_ws_components[1], ComponentValue::Token(CssToken::Comma)) {
        let comma_idx = components
            .iter()
            .position(|c| matches!(c, ComponentValue::Token(CssToken::Comma)))?;
        let fallback = &components[comma_idx + 1..];
        Some((var_name, Some(fallback)))
    } else {
        None
    }
}

/// Recursively performs custom-property variable substitution.
/// Returns `None` if a cyclic reference or malformed var structure is detected.
fn substitute_variables(
    components: &[ComponentValue],
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
    active_lookups: &mut HashSet<String>,
) -> Option<Vec<ComponentValue>> {
    let mut result = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("var") => {
                let (var_name, fallback) = parse_var_function(value)?;
                if active_lookups.contains(var_name) {
                    // TODO(spec): detect cycle and resolve to invalid/None
                    return None;
                }
                if let Some(sub_val) = custom_properties.get(var_name) {
                    active_lookups.insert(var_name.to_string());
                    let resolved =
                        substitute_variables(sub_val, custom_properties, active_lookups)?;
                    active_lookups.remove(var_name);
                    result.extend(resolved);
                } else if let Some(fb) = fallback {
                    let resolved = substitute_variables(fb, custom_properties, active_lookups)?;
                    result.extend(resolved);
                } else {
                    return None;
                }
            }
            ComponentValue::Function { name, value } => {
                let resolved_args = substitute_variables(value, custom_properties, active_lookups)?;
                result.push(ComponentValue::Function {
                    name: name.clone(),
                    value: resolved_args,
                });
            }
            ComponentValue::SimpleBlock { associated, value } => {
                let resolved_block =
                    substitute_variables(value, custom_properties, active_lookups)?;
                result.push(ComponentValue::SimpleBlock {
                    associated: *associated,
                    value: resolved_block,
                });
            }
            _ => {
                result.push(comp.clone());
            }
        }
    }
    Some(result)
}

struct CalcExprParser {
    tokens: Vec<CalcToken>,
    pos: usize,
}

impl CalcExprParser {
    fn peek(&self) -> Option<&CalcToken> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<&CalcToken> {
        let t = self.tokens.get(self.pos);
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse_expr(&mut self, min_bp: u8) -> Option<CalcValue> {
        let mut lhs = match self.consume()? {
            CalcToken::Val(val) => *val,
            CalcToken::LeftParen => {
                let val = self.parse_expr(0)?;
                if !matches!(self.consume()?, CalcToken::RightParen) {
                    return None;
                }
                val
            }
            CalcToken::Minus => {
                let val = self.parse_expr(10)?;
                match val {
                    CalcValue::Length(v) => CalcValue::Length(-v),
                    CalcValue::Number(v) => CalcValue::Number(-v),
                }
            }
            CalcToken::Plus => self.parse_expr(10)?,
            _ => return None,
        };

        while let Some(op) = self.peek() {
            let (left_bp, right_bp) = match op {
                CalcToken::Plus | CalcToken::Minus => (1, 2),
                CalcToken::Mul | CalcToken::Div => (3, 4),
                _ => break,
            };

            if left_bp < min_bp {
                break;
            }

            let op = self.consume().cloned()?;
            let rhs = self.parse_expr(right_bp)?;

            lhs = match op {
                CalcToken::Plus => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Length(r)) => CalcValue::Length(l + r),
                    (CalcValue::Number(l), CalcValue::Number(r)) => CalcValue::Number(l + r),
                    _ => return None,
                },
                CalcToken::Minus => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Length(r)) => CalcValue::Length(l - r),
                    (CalcValue::Number(l), CalcValue::Number(r)) => CalcValue::Number(l - r),
                    _ => return None,
                },
                CalcToken::Mul => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Number(r)) => CalcValue::Length(l * r),
                    (CalcValue::Number(l), CalcValue::Length(r)) => CalcValue::Length(l * r),
                    (CalcValue::Number(l), CalcValue::Number(r)) => CalcValue::Number(l * r),
                    _ => return None,
                },
                CalcToken::Div => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Number(r)) => {
                        if r == 0.0 {
                            return None;
                        }
                        CalcValue::Length(l / r)
                    }
                    (CalcValue::Number(l), CalcValue::Number(r)) => {
                        if r == 0.0 {
                            return None;
                        }
                        CalcValue::Number(l / r)
                    }
                    _ => return None,
                },
                _ => return None,
            };
        }

        Some(lhs)
    }
}

fn component_to_calc_tokens(
    comp: &ComponentValue,
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    tokens: &mut Vec<CalcToken>,
) -> bool {
    match comp {
        ComponentValue::Token(CssToken::Whitespace) => true,
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let lower_unit = unit.to_ascii_lowercase();
            let px_val = match lower_unit.as_str() {
                "px" => *value as f32,
                "rem" => *value as f32 * root_font_size,
                "vw" => *value as f32 * viewport_w / 100.0,
                "vh" => *value as f32 * viewport_h / 100.0,
                "pt" => *value as f32 * 96.0 / 72.0,
                _ => return false,
            };
            tokens.push(CalcToken::Val(CalcValue::Length(px_val)));
            true
        }
        ComponentValue::Token(CssToken::Number(v)) => {
            tokens.push(CalcToken::Val(CalcValue::Number(*v as f32)));
            true
        }
        ComponentValue::Token(CssToken::Delim('+')) => {
            tokens.push(CalcToken::Plus);
            true
        }
        ComponentValue::Token(CssToken::Delim('-')) => {
            tokens.push(CalcToken::Minus);
            true
        }
        ComponentValue::Token(CssToken::Delim('*')) => {
            tokens.push(CalcToken::Mul);
            true
        }
        ComponentValue::Token(CssToken::Delim('/')) => {
            tokens.push(CalcToken::Div);
            true
        }
        ComponentValue::Token(CssToken::LeftParen) => {
            tokens.push(CalcToken::LeftParen);
            true
        }
        ComponentValue::Token(CssToken::RightParen) => {
            tokens.push(CalcToken::RightParen);
            true
        }
        ComponentValue::SimpleBlock {
            associated: '(',
            value,
        } => {
            tokens.push(CalcToken::LeftParen);
            for v in value {
                if !component_to_calc_tokens(v, root_font_size, viewport_w, viewport_h, tokens) {
                    return false;
                }
            }
            tokens.push(CalcToken::RightParen);
            true
        }
        _ => false,
    }
}

/// Evaluates a CSS `calc()` expression.
/// Supports basic arithmetic (+ - * /) of lengths (resolved to px) and numbers.
pub fn evaluate_calc(
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    let substituted = substitute_variables(components, custom_properties, &mut HashSet::new())?;

    let mut tokens = Vec::new();
    for comp in &substituted {
        if !component_to_calc_tokens(comp, root_font_size, viewport_w, viewport_h, &mut tokens) {
            return None;
        }
    }

    let mut parser = CalcExprParser { tokens, pos: 0 };
    let res = parser.parse_expr(0)?;
    if parser.pos != parser.tokens.len() {
        return None;
    }

    match res {
        CalcValue::Length(v) => Some(CssValue::Length(v, LengthUnit::Px)),
        CalcValue::Number(v) => Some(CssValue::Number(v)),
    }
}

/// Resolves variables, relative units, and `calc()` expressions inside a list of component values.
pub fn resolve_value(
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    let substituted = substitute_variables(components, custom_properties, &mut HashSet::new())?;

    // Trim leading and trailing whitespace
    let mut start = 0;
    while start < substituted.len()
        && matches!(
            substituted[start],
            ComponentValue::Token(CssToken::Whitespace)
        )
    {
        start += 1;
    }
    let mut end = substituted.len();
    while end > start
        && matches!(
            substituted[end - 1],
            ComponentValue::Token(CssToken::Whitespace)
        )
    {
        end -= 1;
    }
    let trimmed = &substituted[start..end];

    if trimmed.len() == 1 {
        match &trimmed[0] {
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("calc") => {
                evaluate_calc(
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let lower_unit = unit.to_ascii_lowercase();
                match lower_unit.as_str() {
                    "px" => Some(CssValue::Length(*value as f32, LengthUnit::Px)),
                    "rem" => Some(CssValue::Length(
                        *value as f32 * root_font_size,
                        LengthUnit::Px,
                    )),
                    "vw" => Some(CssValue::Length(
                        *value as f32 * viewport_w / 100.0,
                        LengthUnit::Px,
                    )),
                    "vh" => Some(CssValue::Length(
                        *value as f32 * viewport_h / 100.0,
                        LengthUnit::Px,
                    )),
                    "em" => Some(CssValue::Length(*value as f32, LengthUnit::Em)),
                    "pt" => Some(CssValue::Length(
                        *value as f32 * 96.0 / 72.0,
                        LengthUnit::Px,
                    )),
                    _ => None,
                }
            }
            ComponentValue::Token(CssToken::Percentage(v)) => {
                Some(CssValue::Length(*v as f32, LengthUnit::Percent))
            }
            ComponentValue::Token(CssToken::Number(v)) => Some(CssValue::Number(*v as f32)),
            _ => crate::css::values::parse_value(trimmed),
        }
    } else {
        crate::css::values::parse_value(trimmed)
    }
}

/// Parses an input string and resolves variables, relative units, and `calc()` expressions inside it.
pub fn resolve_string(
    input: &str,
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    let components = crate::css::parser::parse_component_values(input);
    resolve_value(
        &components,
        root_font_size,
        viewport_w,
        viewport_h,
        custom_properties,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_calc_basic() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_string("calc(10px + 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(15.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("calc(100px - 20px*2)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(60.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("calc(10px + (5px * 4))", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(30.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("calc(100px / 4)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(25.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_relative_units() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_string("2rem", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(32.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50vw", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10vh", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_vars() {
        let mut vars = HashMap::new();
        vars.insert(
            "--main-size".to_string(),
            crate::css::parser::parse_component_values("10px"),
        );
        vars.insert(
            "--double-size".to_string(),
            crate::css::parser::parse_component_values("calc(var(--main-size) * 2)"),
        );

        assert_eq!(
            resolve_string("var(--main-size)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("var(--double-size)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("calc(var(--main-size) + 15px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(25.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_var_fallback() {
        let vars = HashMap::new();
        assert_eq!(
            resolve_string("var(--missing-var, 25px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(25.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string(
                "var(--missing-var, var(--another-missing, 3rem))",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Length(48.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_calc_relative_units() {
        let vars = HashMap::new();
        // 1rem + 10vw -> 16.0 + 100.0 = 116.0
        assert_eq!(
            resolve_string("calc(1rem + 10vw)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(116.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_malformed_and_cycles() {
        let mut vars = HashMap::new();
        // Self-cyclic
        vars.insert(
            "--cycle".to_string(),
            crate::css::parser::parse_component_values("var(--cycle)"),
        );
        assert_eq!(
            resolve_string("var(--cycle)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // Mutual cyclic
        vars.insert(
            "--a".to_string(),
            crate::css::parser::parse_component_values("var(--b)"),
        );
        vars.insert(
            "--b".to_string(),
            crate::css::parser::parse_component_values("var(--a)"),
        );
        assert_eq!(resolve_string("var(--a)", 16.0, 1000.0, 800.0, &vars), None);

        // Division by zero in calc
        assert_eq!(
            resolve_string("calc(10px / 0)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // Malformed operators
        assert_eq!(
            resolve_string("calc(10px + +)", 16.0, 1000.0, 800.0, &vars),
            None
        );
    }
}
