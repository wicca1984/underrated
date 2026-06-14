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
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
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
                if !component_to_calc_tokens(
                    v,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                    tokens,
                ) {
                    return false;
                }
            }
            tokens.push(CalcToken::RightParen);
            true
        }
        ComponentValue::Function { name, value } => {
            let resolved = if name.eq_ignore_ascii_case("calc") {
                evaluate_calc(
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            } else if name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
            {
                evaluate_math_fn(
                    name,
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            } else {
                None
            };

            match resolved {
                Some(CssValue::Length(px, _)) => {
                    tokens.push(CalcToken::Val(CalcValue::Length(px)));
                    true
                }
                Some(CssValue::Number(num)) => {
                    tokens.push(CalcToken::Val(CalcValue::Number(num)));
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Evaluates CSS math functions `min()`, `max()`, `clamp()`, `abs()`, `sign()`, `round()`, `mod()`, and `rem()`.
pub fn evaluate_math_fn(
    kind: &str,
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    // 1. Split by top-level Comma tokens.
    let mut args = Vec::new();
    let mut start = 0;
    for (i, comp) in components.iter().enumerate() {
        if matches!(comp, ComponentValue::Token(CssToken::Comma)) {
            args.push(&components[start..i]);
            start = i + 1;
        }
    }
    args.push(&components[start..]);

    let kind_lower = kind.to_ascii_lowercase();
    let kind_str = kind_lower.as_str();

    // 2. Extract strategy keyword if this is a round() function.
    let mut strategy = None;
    let numeric_args_slices = if kind_str == "round" {
        if args.len() == 3 {
            let first_trimmed = trim_whitespace(args[0]);
            if first_trimmed.len() != 1 {
                return None;
            }
            let strat = match &first_trimmed[0] {
                ComponentValue::Token(CssToken::Ident(s)) => {
                    let s_low = s.to_ascii_lowercase();
                    match s_low.as_str() {
                        "nearest" => "nearest",
                        "up" => "up",
                        "down" => "down",
                        "to-zero" => "to-zero",
                        _ => return None,
                    }
                }
                _ => return None,
            };
            strategy = Some(strat);
            vec![args[1], args[2]]
        } else if args.len() == 2 {
            strategy = Some("nearest");
            vec![args[0], args[1]]
        } else {
            return None;
        }
    } else {
        args.clone()
    };

    // Arity validation
    match kind_str {
        "abs" | "sign" => {
            if numeric_args_slices.len() != 1 {
                return None;
            }
        }
        "mod" | "rem" => {
            if numeric_args_slices.len() != 2 {
                return None;
            }
        }
        "round" => {
            if numeric_args_slices.len() != 2 {
                return None;
            }
        }
        "clamp" => {
            if numeric_args_slices.len() != 3 {
                return None;
            }
        }
        "min" | "max" => {
            if numeric_args_slices.is_empty() {
                return None;
            }
        }
        _ => return None,
    }

    // 3. Evaluate each argument group with the existing evaluate_calc.
    let mut evaluated_args = Vec::new();
    for arg_group in numeric_args_slices {
        let trimmed_group = trim_whitespace(arg_group);
        if trimmed_group.is_empty() {
            return None;
        }
        let evaluated = evaluate_calc(
            trimmed_group,
            root_font_size,
            viewport_w,
            viewport_h,
            custom_properties,
        )?;
        evaluated_args.push(evaluated);
    }

    if evaluated_args.is_empty() {
        return None;
    }

    // 4. Ensure all arguments are of the same kind: all Lengths in Px or all Numbers.
    let is_length = match evaluated_args[0] {
        CssValue::Length(_, LengthUnit::Px) => true,
        CssValue::Number(_) => false,
        _ => return None,
    };

    for arg in &evaluated_args {
        match arg {
            CssValue::Length(_, LengthUnit::Px) if is_length => {}
            CssValue::Number(_) if !is_length => {}
            _ => return None, // Mismatched kinds
        }
    }

    // 5. Extract f32 values.
    let values: Vec<f32> = if is_length {
        evaluated_args
            .iter()
            .map(|arg| match arg {
                CssValue::Length(v, _) => *v,
                _ => 0.0,
            })
            .collect()
    } else {
        evaluated_args
            .iter()
            .map(|arg| match arg {
                CssValue::Number(v) => *v,
                _ => 0.0,
            })
            .collect()
    };

    // 6. Combine values based on math function kind.
    let result = if kind_str == "min" {
        let &first = values.first()?;
        let mut min_val = first;
        for &v in &values[1..] {
            min_val = min_val.min(v);
        }
        min_val
    } else if kind_str == "max" {
        let &first = values.first()?;
        let mut max_val = first;
        for &v in &values[1..] {
            max_val = max_val.max(v);
        }
        max_val
    } else if kind_str == "clamp" {
        if values.len() != 3 {
            return None;
        }
        let min_val = values[0];
        let val_val = values[1];
        let max_val = values[2];
        min_val.max(val_val.min(max_val))
    } else if kind_str == "abs" {
        if values.len() != 1 {
            return None;
        }
        values[0].abs()
    } else if kind_str == "sign" {
        if values.len() != 1 {
            return None;
        }
        let val = values[0];
        if val > 0.0 {
            1.0
        } else if val < 0.0 {
            -1.0
        } else {
            0.0
        }
    } else if kind_str == "round" {
        if values.len() != 2 {
            return None;
        }
        let a = values[0];
        let b = values[1];
        if b == 0.0 {
            return None;
        }
        let b_abs = b.abs();
        let frac = a / b_abs;
        let strat = strategy.unwrap_or("nearest");
        let frac_rounded = match strat {
            "nearest" => (frac + 0.5).floor(),
            "up" => frac.ceil(),
            "down" => frac.floor(),
            "to-zero" => frac.trunc(),
            _ => return None,
        };
        frac_rounded * b_abs
    } else if kind_str == "mod" {
        if values.len() != 2 {
            return None;
        }
        let a = values[0];
        let b = values[1];
        if b == 0.0 {
            return None;
        }
        a - b * (a / b).floor()
    } else if kind_str == "rem" {
        if values.len() != 2 {
            return None;
        }
        let a = values[0];
        let b = values[1];
        if b == 0.0 {
            return None;
        }
        a - b * (a / b).trunc()
    } else {
        return None;
    };

    // 7. Return the result in the corresponding variant.
    if kind_str == "sign" {
        Some(CssValue::Number(result))
    } else if is_length {
        Some(CssValue::Length(result, LengthUnit::Px))
    } else {
        Some(CssValue::Number(result))
    }
}

/// Helper function to trim whitespace from a component slice.
fn trim_whitespace(components: &[ComponentValue]) -> &[ComponentValue] {
    let mut start = 0;
    while start < components.len()
        && matches!(
            components[start],
            ComponentValue::Token(CssToken::Whitespace)
        )
    {
        start += 1;
    }
    let mut end = components.len();
    while end > start
        && matches!(
            components[end - 1],
            ComponentValue::Token(CssToken::Whitespace)
        )
    {
        end -= 1;
    }
    &components[start..end]
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
        if !component_to_calc_tokens(
            comp,
            root_font_size,
            viewport_w,
            viewport_h,
            custom_properties,
            &mut tokens,
        ) {
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
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("min") => {
                evaluate_math_fn(
                    "min",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("max") => {
                evaluate_math_fn(
                    "max",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("clamp") => {
                evaluate_math_fn(
                    "clamp",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("abs") => {
                evaluate_math_fn(
                    "abs",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("sign") => {
                evaluate_math_fn(
                    "sign",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("round") => {
                evaluate_math_fn(
                    "round",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("mod") => {
                evaluate_math_fn(
                    "mod",
                    value,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                    custom_properties,
                )
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("rem") => {
                evaluate_math_fn(
                    "rem",
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

    #[test]
    fn test_resolve_math_fns() {
        let vars = HashMap::new();

        // 1. Basic min/max/clamp resolving to lengths in Px
        assert_eq!(
            resolve_string("min(10px, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("max(10px, 20px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("clamp(10px, 5px, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("clamp(10px, 30px, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("clamp(10px, 15px, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(15.0, LengthUnit::Px))
        );

        // 2. Bare numbers support
        assert_eq!(
            resolve_string("min(4, 2, 8)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(2.0))
        );
        assert_eq!(
            resolve_string("max(4, 2, 8)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(8.0))
        );
        assert_eq!(
            resolve_string("clamp(10, 5, 20)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(10.0))
        );

        // 3. Nested math/calc functions
        assert_eq!(
            resolve_string("max(10px, calc(5px + 8px))", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(13.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string(
                "calc(min(10px, 20px) + max(5px, 15px))",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Length(25.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string(
                "clamp(min(5px, 10px), 15px, max(20px, 30px))",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Length(15.0, LengthUnit::Px))
        );

        // 4. Mixed types and invalid clamp cases should return None
        assert_eq!(
            resolve_string("min(10px, 5)", 16.0, 1000.0, 800.0, &vars),
            None
        );
        assert_eq!(
            resolve_string("clamp(10px, 15px)", 16.0, 1000.0, 800.0, &vars),
            None
        );
        assert_eq!(
            resolve_string("clamp(10px, 15px, 20px, 25px)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // 5. Variables inside min/max/clamp
        let mut vars_with_custom = HashMap::new();
        vars_with_custom.insert(
            "--limit".to_string(),
            crate::css::parser::parse_component_values("15px"),
        );
        assert_eq!(
            resolve_string(
                "min(10px, var(--limit))",
                16.0,
                1000.0,
                800.0,
                &vars_with_custom
            ),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );

        // 6. clamp MIN > MAX case
        assert_eq!(
            resolve_string("clamp(20px, 15px, 10px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_new_math_fns() {
        let vars = HashMap::new();

        // abs
        assert_eq!(
            resolve_string("abs(-5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(5.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("abs(5)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(5.0))
        );
        assert_eq!(
            resolve_string("abs(-42)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(42.0))
        );

        // sign
        assert_eq!(
            resolve_string("sign(-3px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(-1.0))
        );
        assert_eq!(
            resolve_string("sign(0)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(0.0))
        );
        assert_eq!(
            resolve_string("sign(7)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(1.0))
        );
        assert_eq!(
            resolve_string("sign(10px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(1.0))
        );

        // round
        assert_eq!(
            resolve_string("round(11px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("round(up, 11px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(15.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("round(down, 14px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("round(to-zero, -14px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(-10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("round(13, 5)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(15.0))
        );
        assert_eq!(
            resolve_string("round(13, 0)", 16.0, 1000.0, 800.0, &vars),
            None
        );
        assert_eq!(
            resolve_string("round(up, 13, 0)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // mod
        assert_eq!(
            resolve_string("mod(18px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(3.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("mod(-18px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(2.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("mod(18px, 0px)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // rem
        assert_eq!(
            resolve_string("rem(18px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(3.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("rem(-18px, 5px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(-3.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("rem(18px, 0px)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // nested math functions inside round/mod/rem/abs/sign
        assert_eq!(
            resolve_string("abs(min(-10px, -20px))", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("sign(max(5px, 10px))", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(1.0))
        );
    }
}
