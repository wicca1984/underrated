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
    Angle(f32), // In radians
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

#[derive(Debug, Clone)]
pub struct ResolveContext {
    pub root_font_size: f32,
    pub current_font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub root_line_height: Option<f32>,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub percentage_basis: Option<f32>,
    pub property_name: Option<String>,
    pub parent_value: Option<CssValue>,
    pub current_color: Option<CssValue>,
    pub revert_value: Option<CssValue>,
    pub revert_layer_value: Option<CssValue>,
}

impl Default for ResolveContext {
    fn default() -> Self {
        Self {
            root_font_size: 16.0,
            current_font_size: None,
            line_height: None,
            root_line_height: None,
            viewport_w: 1024.0,
            viewport_h: 768.0,
            percentage_basis: None,
            property_name: None,
            parent_value: None,
            current_color: None,
            revert_value: None,
            revert_layer_value: None,
        }
    }
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

/// Helper to parse a `env()` function's arguments.
/// format: `env( <custom-ident> [, <declaration-value>]? )`
fn parse_env_function(components: &[ComponentValue]) -> Option<(&str, Option<&[ComponentValue]>)> {
    let mut non_ws_components = Vec::new();
    for comp in components {
        if !matches!(comp, ComponentValue::Token(CssToken::Whitespace)) {
            non_ws_components.push(comp);
        }
    }

    if non_ws_components.is_empty() {
        return None;
    }

    let env_name = match non_ws_components[0] {
        ComponentValue::Token(CssToken::Ident(name)) => name.as_str(),
        _ => return None,
    };

    if non_ws_components.len() == 1 {
        return Some((env_name, None));
    }

    if matches!(non_ws_components[1], ComponentValue::Token(CssToken::Comma)) {
        let comma_idx = components
            .iter()
            .position(|c| matches!(c, ComponentValue::Token(CssToken::Comma)))?;
        let fallback = &components[comma_idx + 1..];
        Some((env_name, Some(fallback)))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SubstitutionError {
    Cycle,
    Missing,
}

fn substitute_variables_rec(
    components: &[ComponentValue],
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
    active_lookups: &mut HashSet<String>,
) -> Result<Vec<ComponentValue>, SubstitutionError> {
    let mut result = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("var") => {
                let (var_name, fallback) =
                    parse_var_function(value).ok_or(SubstitutionError::Missing)?;
                if active_lookups.contains(var_name) {
                    return Err(SubstitutionError::Cycle);
                }
                if let Some(sub_val) = custom_properties.get(var_name) {
                    active_lookups.insert(var_name.to_string());
                    let resolved_res =
                        substitute_variables_rec(sub_val, custom_properties, active_lookups);
                    active_lookups.remove(var_name);
                    match resolved_res {
                        Ok(resolved) => {
                            result.extend(resolved);
                        }
                        Err(SubstitutionError::Cycle) => {
                            if active_lookups.is_empty() {
                                if let Some(fb) = fallback {
                                    let resolved = substitute_variables_rec(
                                        fb,
                                        custom_properties,
                                        active_lookups,
                                    )?;
                                    result.extend(resolved);
                                } else {
                                    return Err(SubstitutionError::Cycle);
                                }
                            } else {
                                return Err(SubstitutionError::Cycle);
                            }
                        }
                        Err(SubstitutionError::Missing) => {
                            if let Some(fb) = fallback {
                                let resolved = substitute_variables_rec(
                                    fb,
                                    custom_properties,
                                    active_lookups,
                                )?;
                                result.extend(resolved);
                            } else {
                                return Err(SubstitutionError::Missing);
                            }
                        }
                    }
                } else if let Some(fb) = fallback {
                    let resolved = substitute_variables_rec(fb, custom_properties, active_lookups)?;
                    result.extend(resolved);
                } else {
                    return Err(SubstitutionError::Missing);
                }
            }
            ComponentValue::Function { name, value } if name.eq_ignore_ascii_case("env") => {
                let (env_name, fallback) =
                    parse_env_function(value).ok_or(SubstitutionError::Missing)?;
                let known_env_val = match env_name {
                    "safe-area-inset-top"
                    | "safe-area-inset-right"
                    | "safe-area-inset-bottom"
                    | "safe-area-inset-left" => {
                        Some(vec![ComponentValue::Token(CssToken::Dimension {
                            value: 0.0,
                            unit: "px".to_string(),
                        })])
                    }
                    _ => None,
                };

                if let Some(resolved_env) = known_env_val {
                    let resolved =
                        substitute_variables_rec(&resolved_env, custom_properties, active_lookups)?;
                    result.extend(resolved);
                } else if let Some(fb) = fallback {
                    let resolved = substitute_variables_rec(fb, custom_properties, active_lookups)?;
                    result.extend(resolved);
                } else {
                    return Err(SubstitutionError::Missing);
                }
            }
            ComponentValue::Function { name, value } => {
                let resolved_args =
                    substitute_variables_rec(value, custom_properties, active_lookups)?;
                result.push(ComponentValue::Function {
                    name: name.clone(),
                    value: resolved_args,
                });
            }
            ComponentValue::SimpleBlock { associated, value } => {
                let resolved_block =
                    substitute_variables_rec(value, custom_properties, active_lookups)?;
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
    Ok(result)
}

/// Recursively performs custom-property variable substitution.
/// Returns `None` if a cyclic reference or malformed var structure is detected.
fn substitute_variables(
    components: &[ComponentValue],
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
    active_lookups: &mut HashSet<String>,
) -> Option<Vec<ComponentValue>> {
    substitute_variables_rec(components, custom_properties, active_lookups).ok()
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
                    CalcValue::Angle(v) => CalcValue::Angle(-v),
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
                    (CalcValue::Angle(l), CalcValue::Angle(r)) => CalcValue::Angle(l + r),
                    _ => return None,
                },
                CalcToken::Minus => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Length(r)) => CalcValue::Length(l - r),
                    (CalcValue::Number(l), CalcValue::Number(r)) => CalcValue::Number(l - r),
                    (CalcValue::Angle(l), CalcValue::Angle(r)) => CalcValue::Angle(l - r),
                    _ => return None,
                },
                CalcToken::Mul => match (lhs, rhs) {
                    (CalcValue::Length(l), CalcValue::Number(r)) => CalcValue::Length(l * r),
                    (CalcValue::Number(l), CalcValue::Length(r)) => CalcValue::Length(l * r),
                    (CalcValue::Number(l), CalcValue::Number(r)) => CalcValue::Number(l * r),
                    (CalcValue::Angle(l), CalcValue::Number(r)) => CalcValue::Angle(l * r),
                    (CalcValue::Number(l), CalcValue::Angle(r)) => CalcValue::Angle(l * r),
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
                    (CalcValue::Angle(l), CalcValue::Number(r)) => {
                        if r == 0.0 {
                            return None;
                        }
                        CalcValue::Angle(l / r)
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
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
    tokens: &mut Vec<CalcToken>,
) -> bool {
    match comp {
        ComponentValue::Token(CssToken::Whitespace) => true,
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let lower_unit = unit.to_ascii_lowercase();
            match lower_unit.as_str() {
                "px" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32)));
                    true
                }
                "rem" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.root_font_size,
                    )));
                    true
                }
                "vw" | "svw" | "lvw" | "dvw" | "vi" | "svi" | "lvi" | "dvi" | "cqw" | "cqi" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.viewport_w / 100.0,
                    )));
                    true
                }
                "vh" | "svh" | "lvh" | "dvh" | "vb" | "svb" | "lvb" | "dvb" | "cqh" | "cqb" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.viewport_h / 100.0,
                    )));
                    true
                }
                "vmin" | "svmin" | "lvmin" | "dvmin" | "cqmin" => {
                    let vmin = context.viewport_w.min(context.viewport_h);
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * vmin / 100.0,
                    )));
                    true
                }
                "vmax" | "svmax" | "lvmax" | "dvmax" | "cqmax" => {
                    let vmax = context.viewport_w.max(context.viewport_h);
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * vmax / 100.0,
                    )));
                    true
                }
                "in" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * 96.0)));
                    true
                }
                "cm" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * 96.0 / 2.54,
                    )));
                    true
                }
                "mm" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * 9.6 / 2.54,
                    )));
                    true
                }
                "pc" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * 16.0)));
                    true
                }
                "pt" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * 96.0 / 72.0,
                    )));
                    true
                }
                "q" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * 2.4 / 2.54,
                    )));
                    true
                }
                "em" => {
                    let fs = context.current_font_size.unwrap_or(context.root_font_size);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * fs)));
                    true
                }
                "ex" => {
                    let fs = context.current_font_size.unwrap_or(context.root_font_size);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * fs * 0.5)));
                    true
                }
                "ch" => {
                    let fs = context.current_font_size.unwrap_or(context.root_font_size);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * fs * 0.5)));
                    true
                }
                "ic" => {
                    let fs = context.current_font_size.unwrap_or(context.root_font_size);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * fs)));
                    true
                }
                "cap" => {
                    let fs = context.current_font_size.unwrap_or(context.root_font_size);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * fs * 0.7)));
                    true
                }
                "lh" => {
                    let lh = context.line_height.unwrap_or_else(|| {
                        context.current_font_size.unwrap_or(context.root_font_size) * 1.2
                    });
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * lh)));
                    true
                }
                "rlh" => {
                    let rlh = context
                        .root_line_height
                        .unwrap_or(context.root_font_size * 1.2);
                    tokens.push(CalcToken::Val(CalcValue::Length(*value as f32 * rlh)));
                    true
                }
                "rex" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.root_font_size * 0.5,
                    )));
                    true
                }
                "rch" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.root_font_size * 0.5,
                    )));
                    true
                }
                "ric" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.root_font_size,
                    )));
                    true
                }
                "rcap" => {
                    tokens.push(CalcToken::Val(CalcValue::Length(
                        *value as f32 * context.root_font_size * 0.7,
                    )));
                    true
                }
                "deg" => {
                    let rad = (*value as f32).to_radians();
                    tokens.push(CalcToken::Val(CalcValue::Angle(rad)));
                    true
                }
                "rad" => {
                    tokens.push(CalcToken::Val(CalcValue::Angle(*value as f32)));
                    true
                }
                "grad" => {
                    let rad = (*value as f32) * std::f32::consts::PI / 200.0;
                    tokens.push(CalcToken::Val(CalcValue::Angle(rad)));
                    true
                }
                "turn" => {
                    let rad = (*value as f32) * 2.0 * std::f32::consts::PI;
                    tokens.push(CalcToken::Val(CalcValue::Angle(rad)));
                    true
                }
                _ => false,
            }
        }
        ComponentValue::Token(CssToken::Percentage(v)) => {
            if let Some(basis) = context.percentage_basis {
                tokens.push(CalcToken::Val(CalcValue::Length(*v as f32 * basis / 100.0)));
                true
            } else {
                false
            }
        }
        ComponentValue::Token(CssToken::Number(v)) => {
            tokens.push(CalcToken::Val(CalcValue::Number(*v as f32)));
            true
        }
        ComponentValue::Token(CssToken::Ident(name)) => {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "pi" => {
                    tokens.push(CalcToken::Val(CalcValue::Number(std::f32::consts::PI)));
                    true
                }
                "e" => {
                    tokens.push(CalcToken::Val(CalcValue::Number(std::f32::consts::E)));
                    true
                }
                "infinity" => {
                    tokens.push(CalcToken::Val(CalcValue::Number(f32::INFINITY)));
                    true
                }
                "-infinity" => {
                    tokens.push(CalcToken::Val(CalcValue::Number(f32::NEG_INFINITY)));
                    true
                }
                "nan" => {
                    tokens.push(CalcToken::Val(CalcValue::Number(f32::NAN)));
                    true
                }
                _ => false,
            }
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
                if !component_to_calc_tokens(v, context, custom_properties, tokens) {
                    return false;
                }
            }
            tokens.push(CalcToken::RightParen);
            true
        }
        ComponentValue::Function { name, value } => {
            let resolved = if name.eq_ignore_ascii_case("calc") {
                evaluate_calc_to_value_with_context(value, context, custom_properties)
            } else if name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
                || name.eq_ignore_ascii_case("sqrt")
                || name.eq_ignore_ascii_case("pow")
                || name.eq_ignore_ascii_case("hypot")
                || name.eq_ignore_ascii_case("log")
                || name.eq_ignore_ascii_case("exp")
                || name.eq_ignore_ascii_case("sin")
                || name.eq_ignore_ascii_case("cos")
                || name.eq_ignore_ascii_case("tan")
                || name.eq_ignore_ascii_case("asin")
                || name.eq_ignore_ascii_case("acos")
                || name.eq_ignore_ascii_case("atan")
                || name.eq_ignore_ascii_case("atan2")
            {
                evaluate_math_fn_to_value_with_context(name, value, context, custom_properties)
            } else {
                None
            };

            match resolved {
                Some(CalcValue::Length(px)) => {
                    tokens.push(CalcToken::Val(CalcValue::Length(px)));
                    true
                }
                Some(CalcValue::Number(num)) => {
                    tokens.push(CalcToken::Val(CalcValue::Number(num)));
                    true
                }
                Some(CalcValue::Angle(rad)) => {
                    tokens.push(CalcToken::Val(CalcValue::Angle(rad)));
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

fn evaluate_math_fn_to_value_with_context(
    kind: &str,
    components: &[ComponentValue],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CalcValue> {
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
        "abs" | "sign" | "sqrt" | "exp" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" => {
            if numeric_args_slices.len() != 1 {
                return None;
            }
        }
        "mod" | "rem" | "pow" | "atan2" => {
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
        "min" | "max" | "hypot" => {
            if numeric_args_slices.is_empty() {
                return None;
            }
        }
        "log" => {
            if numeric_args_slices.len() != 1 && numeric_args_slices.len() != 2 {
                return None;
            }
        }
        _ => return None,
    }

    // 3. Evaluate each argument group with the existing evaluate_calc_to_value.
    let mut evaluated_args = Vec::new();
    for arg_group in numeric_args_slices {
        let trimmed_group = trim_whitespace(arg_group);
        if trimmed_group.is_empty() {
            return None;
        }
        let evaluated =
            evaluate_calc_to_value_with_context(trimmed_group, context, custom_properties)?;
        evaluated_args.push(evaluated);
    }

    if evaluated_args.is_empty() {
        return None;
    }

    // 4. Validate types
    match kind_str {
        "sin" | "cos" | "tan" => {
            if !matches!(
                evaluated_args[0],
                CalcValue::Number(_) | CalcValue::Angle(_)
            ) {
                return None;
            }
        }
        "asin" | "acos" | "atan" => {
            if !matches!(evaluated_args[0], CalcValue::Number(_)) {
                return None;
            }
        }
        "atan2" => match (evaluated_args[0], evaluated_args[1]) {
            (CalcValue::Number(_), CalcValue::Number(_)) => {}
            (CalcValue::Length(_), CalcValue::Length(_)) => {}
            (CalcValue::Angle(_), CalcValue::Angle(_)) => {}
            _ => return None,
        },
        "sqrt" | "pow" | "log" | "exp" => {
            for arg in &evaluated_args {
                if !matches!(arg, CalcValue::Number(_)) {
                    return None;
                }
            }
        }
        _ => {
            let first_type_ok = match evaluated_args[0] {
                CalcValue::Length(_) => {
                    for arg in &evaluated_args {
                        if !matches!(arg, CalcValue::Length(_)) {
                            return None;
                        }
                    }
                    true
                }
                CalcValue::Number(_) => {
                    for arg in &evaluated_args {
                        if !matches!(arg, CalcValue::Number(_)) {
                            return None;
                        }
                    }
                    true
                }
                CalcValue::Angle(_) => {
                    for arg in &evaluated_args {
                        if !matches!(arg, CalcValue::Angle(_)) {
                            return None;
                        }
                    }
                    true
                }
            };
            if !first_type_ok {
                return None;
            }
        }
    }

    // 5. Extract f32 values
    let is_exponential = matches!(kind_str, "sqrt" | "pow" | "log" | "exp");
    let is_length = matches!(evaluated_args[0], CalcValue::Length(_));
    let is_angle = matches!(evaluated_args[0], CalcValue::Angle(_));

    let values: Vec<f32> = evaluated_args
        .iter()
        .map(|arg| match arg {
            CalcValue::Length(v) => *v,
            CalcValue::Number(v) => *v,
            CalcValue::Angle(v) => *v,
        })
        .collect();

    // Safeguard input values against NaN or infinite values
    for &v in &values {
        if v.is_nan() || v.is_infinite() {
            return None;
        }
    }

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
    } else if kind_str == "sqrt" {
        if values.len() != 1 {
            return None;
        }
        let val = values[0];
        if val < 0.0 {
            return None;
        }
        let res = val.sqrt();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "pow" {
        if values.len() != 2 {
            return None;
        }
        let a = values[0];
        let b = values[1];
        if a < 0.0 && b.fract() != 0.0 {
            return None;
        }
        let res = a.powf(b);
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "hypot" {
        if values.is_empty() {
            return None;
        }
        let mut sum_sq = 0.0;
        for &v in &values {
            sum_sq += v * v;
        }
        let res = sum_sq.sqrt();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "log" {
        if values.len() == 1 {
            let a = values[0];
            if a <= 0.0 {
                return None;
            }
            let res = a.ln();
            if res.is_nan() || res.is_infinite() {
                return None;
            }
            res
        } else if values.len() == 2 {
            let a = values[0];
            let b = values[1];
            if a <= 0.0 || b <= 0.0 || b == 1.0 {
                return None;
            }
            let res = a.ln() / b.ln();
            if res.is_nan() || res.is_infinite() {
                return None;
            }
            res
        } else {
            return None;
        }
    } else if kind_str == "exp" {
        if values.len() != 1 {
            return None;
        }
        let val = values[0];
        let res = val.exp();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "sin" {
        let res = values[0].sin();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "cos" {
        let res = values[0].cos();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "tan" {
        let res = values[0].tan();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "asin" {
        let v = values[0];
        if !(-1.0..=1.0).contains(&v) {
            return None;
        }
        let res = v.asin();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "acos" {
        let v = values[0];
        if !(-1.0..=1.0).contains(&v) {
            return None;
        }
        let res = v.acos();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "atan" {
        let res = values[0].atan();
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else if kind_str == "atan2" {
        let res = values[0].atan2(values[1]);
        if res.is_nan() || res.is_infinite() {
            return None;
        }
        res
    } else {
        return None;
    };

    // 7. Return the result in the corresponding CalcValue variant.
    let is_angle_res = matches!(kind_str, "asin" | "acos" | "atan" | "atan2");
    if is_angle_res {
        Some(CalcValue::Angle(result))
    } else if kind_str == "sign" || is_exponential || matches!(kind_str, "sin" | "cos" | "tan") {
        Some(CalcValue::Number(result))
    } else if is_length {
        Some(CalcValue::Length(result))
    } else if is_angle {
        Some(CalcValue::Angle(result))
    } else {
        Some(CalcValue::Number(result))
    }
}

fn evaluate_math_fn_to_value(
    kind: &str,
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CalcValue> {
    let context = ResolveContext {
        root_font_size,
        viewport_w,
        viewport_h,
        ..Default::default()
    };
    evaluate_math_fn_to_value_with_context(kind, components, &context, custom_properties)
}

/// Evaluates CSS math functions `min()`, `max()`, `clamp()`, `abs()`, `sign()`, `round()`, `mod()`, `rem()`, `sqrt()`, `pow()`, `hypot()`, `log()`, and `exp()`.
pub fn evaluate_math_fn(
    kind: &str,
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    let res = evaluate_math_fn_to_value(
        kind,
        components,
        root_font_size,
        viewport_w,
        viewport_h,
        custom_properties,
    )?;

    match res {
        CalcValue::Length(v) => Some(CssValue::Length(v, LengthUnit::Px)),
        CalcValue::Number(v) => Some(CssValue::Number(v)),
        CalcValue::Angle(_) => None,
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

fn evaluate_calc_to_value_with_context(
    components: &[ComponentValue],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CalcValue> {
    let substituted = substitute_variables(components, custom_properties, &mut HashSet::new())?;

    let mut tokens = Vec::new();
    for comp in &substituted {
        if !component_to_calc_tokens(comp, context, custom_properties, &mut tokens) {
            return None;
        }
    }

    let mut parser = CalcExprParser { tokens, pos: 0 };
    let res = parser.parse_expr(0)?;
    if parser.pos != parser.tokens.len() {
        return None;
    }

    Some(res)
}

fn evaluate_calc_to_value(
    components: &[ComponentValue],
    root_font_size: f32,
    viewport_w: f32,
    viewport_h: f32,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CalcValue> {
    let context = ResolveContext {
        root_font_size,
        viewport_w,
        viewport_h,
        ..Default::default()
    };
    evaluate_calc_to_value_with_context(components, &context, custom_properties)
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
    let res = evaluate_calc_to_value(
        components,
        root_font_size,
        viewport_w,
        viewport_h,
        custom_properties,
    )?;

    match res {
        CalcValue::Length(v) => Some(CssValue::Length(v, LengthUnit::Px)),
        CalcValue::Number(v) => Some(CssValue::Number(v)),
        CalcValue::Angle(_) => None,
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
    let context = ResolveContext {
        root_font_size,
        viewport_w,
        viewport_h,
        ..Default::default()
    };
    resolve_value_with_context(components, &context, custom_properties)
}

pub fn resolve_components(
    components: &[ComponentValue],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<Vec<ComponentValue>> {
    let mut result = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let lower_unit = unit.to_ascii_lowercase();
                match lower_unit.as_str() {
                    "px" => {
                        result.push(comp.clone());
                    }
                    "rem" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.root_font_size) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "vw" | "svw" | "lvw" | "dvw" | "vi" | "svi" | "lvi" | "dvi" | "cqw" | "cqi" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.viewport_w / 100.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "vh" | "svh" | "lvh" | "dvh" | "vb" | "svb" | "lvb" | "dvb" | "cqh" | "cqb" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.viewport_h / 100.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "vmin" | "svmin" | "lvmin" | "dvmin" | "cqmin" => {
                        let vmin = context.viewport_w.min(context.viewport_h);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * vmin / 100.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "vmax" | "svmax" | "lvmax" | "dvmax" | "cqmax" => {
                        let vmax = context.viewport_w.max(context.viewport_h);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * vmax / 100.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "in" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 96.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "cm" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 96.0 / 2.54) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "mm" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 9.6 / 2.54) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "pc" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 16.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "pt" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 96.0 / 72.0) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "q" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * 2.4 / 2.54) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "em" => {
                        if let Some(fs) = context.current_font_size {
                            result.push(ComponentValue::Token(CssToken::Dimension {
                                value: (*value as f32 * fs) as f64,
                                unit: "px".to_string(),
                            }));
                        } else {
                            result.push(comp.clone());
                        }
                    }
                    "ex" => {
                        let fs = context.current_font_size.unwrap_or(context.root_font_size);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * fs * 0.5) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "ch" => {
                        let fs = context.current_font_size.unwrap_or(context.root_font_size);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * fs * 0.5) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "ic" => {
                        let fs = context.current_font_size.unwrap_or(context.root_font_size);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * fs) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "cap" => {
                        let fs = context.current_font_size.unwrap_or(context.root_font_size);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * fs * 0.7) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "lh" => {
                        let lh = context.line_height.unwrap_or_else(|| {
                            context.current_font_size.unwrap_or(context.root_font_size) * 1.2
                        });
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * lh) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "rlh" => {
                        let rlh = context
                            .root_line_height
                            .unwrap_or(context.root_font_size * 1.2);
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * rlh) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "rex" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.root_font_size * 0.5) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "rch" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.root_font_size * 0.5) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "ric" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.root_font_size) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    "rcap" => {
                        result.push(ComponentValue::Token(CssToken::Dimension {
                            value: (*value as f32 * context.root_font_size * 0.7) as f64,
                            unit: "px".to_string(),
                        }));
                    }
                    _ => {
                        result.push(comp.clone());
                    }
                }
            }
            ComponentValue::Token(CssToken::Percentage(v)) => {
                if let Some(basis) = context.percentage_basis {
                    result.push(ComponentValue::Token(CssToken::Dimension {
                        value: (*v as f32 * basis / 100.0) as f64,
                        unit: "px".to_string(),
                    }));
                } else {
                    result.push(comp.clone());
                }
            }
            ComponentValue::Function { name, value } => {
                let name_lower = name.to_ascii_lowercase();
                if name_lower == "calc" {
                    let res =
                        evaluate_calc_to_value_with_context(value, context, custom_properties)?;
                    match res {
                        CalcValue::Length(v) => {
                            result.push(ComponentValue::Token(CssToken::Dimension {
                                value: v as f64,
                                unit: "px".to_string(),
                            }));
                        }
                        CalcValue::Number(v) => {
                            result.push(ComponentValue::Token(CssToken::Number(v as f64)));
                        }
                        CalcValue::Angle(v) => {
                            result.push(ComponentValue::Token(CssToken::Dimension {
                                value: v as f64,
                                unit: "rad".to_string(),
                            }));
                        }
                    }
                } else if name_lower == "min"
                    || name_lower == "max"
                    || name_lower == "clamp"
                    || name_lower == "abs"
                    || name_lower == "sign"
                    || name_lower == "round"
                    || name_lower == "mod"
                    || name_lower == "rem"
                    || name_lower == "sqrt"
                    || name_lower == "pow"
                    || name_lower == "hypot"
                    || name_lower == "log"
                    || name_lower == "exp"
                    || name_lower == "sin"
                    || name_lower == "cos"
                    || name_lower == "tan"
                    || name_lower == "asin"
                    || name_lower == "acos"
                    || name_lower == "atan"
                    || name_lower == "atan2"
                {
                    let res = evaluate_math_fn_to_value_with_context(
                        &name_lower,
                        value,
                        context,
                        custom_properties,
                    )?;
                    match res {
                        CalcValue::Length(v) => {
                            result.push(ComponentValue::Token(CssToken::Dimension {
                                value: v as f64,
                                unit: "px".to_string(),
                            }));
                        }
                        CalcValue::Number(v) => {
                            result.push(ComponentValue::Token(CssToken::Number(v as f64)));
                        }
                        CalcValue::Angle(v) => {
                            result.push(ComponentValue::Token(CssToken::Dimension {
                                value: v as f64,
                                unit: "rad".to_string(),
                            }));
                        }
                    }
                } else {
                    let resolved_args = resolve_components(value, context, custom_properties)?;
                    result.push(ComponentValue::Function {
                        name: name.clone(),
                        value: resolved_args,
                    });
                }
            }
            ComponentValue::SimpleBlock { associated, value } => {
                let resolved_block = resolve_components(value, context, custom_properties)?;
                result.push(ComponentValue::SimpleBlock {
                    associated: *associated,
                    value: resolved_block,
                });
            }
            ComponentValue::Token(CssToken::Ident(name)) => {
                let lower = name.to_ascii_lowercase();
                if lower == "currentcolor" {
                    let color_val = if let Some("color") = context.property_name.as_deref() {
                        if let Some(parent) = &context.parent_value {
                            Some(parent.clone())
                        } else if let Some(init_str) = crate::css::property::initial_value("color")
                        {
                            resolve_string_with_context(init_str, context)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let resolved_color = color_val
                        .or_else(|| context.current_color.clone())
                        .unwrap_or_else(|| {
                            if let Some(init_str) = crate::css::property::initial_value("color") {
                                resolve_string_with_context(init_str, context).unwrap_or(
                                    CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 255)),
                                )
                            } else {
                                CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 255))
                            }
                        });

                    if let CssValue::Color(crate::css::values::Color::Rgba(r, g, b, a)) =
                        resolved_color
                    {
                        let color_str = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a);
                        let comps = crate::css::parser::parse_component_values(&color_str);
                        result.extend(comps);
                    } else {
                        result.push(comp.clone());
                    }
                } else {
                    result.push(comp.clone());
                }
            }
            _ => {
                result.push(comp.clone());
            }
        }
    }
    Some(result)
}

fn has_variable_references(components: &[ComponentValue]) -> bool {
    for comp in components {
        match comp {
            ComponentValue::Function { name, value } => {
                let lower = name.to_ascii_lowercase();
                if lower == "var" || lower == "env" || has_variable_references(value) {
                    return true;
                }
            }
            ComponentValue::SimpleBlock { value, .. } if has_variable_references(value) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn resolve_unset_for_property(prop: &str, context: &ResolveContext) -> Option<CssValue> {
    let is_inh = crate::css::property::is_inherited(prop);
    if is_inh {
        if let Some(parent) = &context.parent_value {
            Some(parent.clone())
        } else if let Some(init_str) = crate::css::property::initial_value(prop) {
            let mut sub_context = context.clone();
            sub_context.property_name = None;
            resolve_string_with_context(init_str, &sub_context)
        } else {
            Some(CssValue::Keyword("unset".to_string()))
        }
    } else {
        if let Some(init_str) = crate::css::property::initial_value(prop) {
            let mut sub_context = context.clone();
            sub_context.property_name = None;
            resolve_string_with_context(init_str, &sub_context)
        } else {
            Some(CssValue::Keyword("unset".to_string()))
        }
    }
}

fn handle_invalid_computed_value(
    components: &[ComponentValue],
    context: &ResolveContext,
) -> Option<CssValue> {
    if !has_variable_references(components) {
        return None;
    }
    let prop = context.property_name.as_ref()?;
    if prop.starts_with("--") {
        return None;
    }
    resolve_unset_for_property(prop, context)
}

pub fn resolve_value_with_context(
    components: &[ComponentValue],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    let substituted = match substitute_variables(components, custom_properties, &mut HashSet::new())
    {
        Some(s) => s,
        None => return handle_invalid_computed_value(components, context),
    };

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
        let ident_name = match &trimmed[0] {
            ComponentValue::Token(CssToken::Ident(name)) => Some(name),
            _ => None,
        };
        if let Some(name) = ident_name {
            let lower = name.to_ascii_lowercase();
            match lower.as_str() {
                "inherit" => {
                    if let Some(parent) = &context.parent_value {
                        return Some(parent.clone());
                    } else if let Some(prop) = &context.property_name {
                        if let Some(init_str) = crate::css::property::initial_value(prop) {
                            return resolve_string_with_context(init_str, context);
                        } else {
                            return Some(CssValue::Keyword("inherit".to_string()));
                        }
                    } else {
                        return Some(CssValue::Keyword("inherit".to_string()));
                    }
                }
                "initial" => {
                    if let Some(prop) = &context.property_name {
                        if let Some(init_str) = crate::css::property::initial_value(prop) {
                            let mut sub_context = context.clone();
                            sub_context.property_name = None;
                            return resolve_string_with_context(init_str, &sub_context);
                        } else {
                            return Some(CssValue::Keyword("initial".to_string()));
                        }
                    } else {
                        return Some(CssValue::Keyword("initial".to_string()));
                    }
                }
                "unset" => {
                    if let Some(val) = context
                        .property_name
                        .as_ref()
                        .and_then(|prop| resolve_unset_for_property(prop, context))
                    {
                        return Some(val);
                    }
                    return Some(CssValue::Keyword("unset".to_string()));
                }
                "revert" => {
                    if let Some(revert_val) = &context.revert_value {
                        return Some(revert_val.clone());
                    } else if let Some(prop) = &context.property_name {
                        let is_inh = crate::css::property::is_inherited(prop);
                        if is_inh {
                            if let Some(parent) = &context.parent_value {
                                return Some(parent.clone());
                            } else if let Some(init_str) = crate::css::property::initial_value(prop)
                            {
                                let mut sub_context = context.clone();
                                sub_context.property_name = None;
                                return resolve_string_with_context(init_str, &sub_context);
                            } else {
                                return Some(CssValue::Keyword("revert".to_string()));
                            }
                        } else {
                            if let Some(init_str) = crate::css::property::initial_value(prop) {
                                let mut sub_context = context.clone();
                                sub_context.property_name = None;
                                return resolve_string_with_context(init_str, &sub_context);
                            } else {
                                return Some(CssValue::Keyword("revert".to_string()));
                            }
                        }
                    } else {
                        return Some(CssValue::Keyword("revert".to_string()));
                    }
                }
                "revert-layer" => {
                    if let Some(revert_layer_val) = &context.revert_layer_value {
                        return Some(revert_layer_val.clone());
                    } else if let Some(revert_val) = &context.revert_value {
                        return Some(revert_val.clone());
                    } else if let Some(prop) = &context.property_name {
                        let is_inh = crate::css::property::is_inherited(prop);
                        if is_inh {
                            if let Some(parent) = &context.parent_value {
                                return Some(parent.clone());
                            } else if let Some(init_str) = crate::css::property::initial_value(prop)
                            {
                                let mut sub_context = context.clone();
                                sub_context.property_name = None;
                                return resolve_string_with_context(init_str, &sub_context);
                            } else {
                                return Some(CssValue::Keyword("revert-layer".to_string()));
                            }
                        } else {
                            if let Some(init_str) = crate::css::property::initial_value(prop) {
                                let mut sub_context = context.clone();
                                sub_context.property_name = None;
                                return resolve_string_with_context(init_str, &sub_context);
                            } else {
                                return Some(CssValue::Keyword("revert-layer".to_string()));
                            }
                        }
                    } else {
                        return Some(CssValue::Keyword("revert-layer".to_string()));
                    }
                }
                "currentcolor" => {
                    if let Some("color") = context.property_name.as_deref() {
                        if let Some(parent) = &context.parent_value {
                            return Some(parent.clone());
                        } else if let Some(init_str) = crate::css::property::initial_value("color")
                        {
                            return resolve_string_with_context(init_str, context);
                        }
                    }
                    if let Some(current) = &context.current_color {
                        return Some(current.clone());
                    } else {
                        if let Some(init_str) = crate::css::property::initial_value("color") {
                            return resolve_string_with_context(init_str, context);
                        } else {
                            return Some(CssValue::Color(crate::css::values::Color::Rgba(
                                0, 0, 0, 255,
                            )));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Recursively resolve all components and then parse.
    let resolved_components = match resolve_components(trimmed, context, custom_properties) {
        Some(rc) => rc,
        None => return handle_invalid_computed_value(components, context),
    };

    let parsed = if let Some(val) = crate::css::values::parse_value(&resolved_components) {
        Some(val)
    } else {
        crate::css::values::parse_transform(&resolved_components)
    };

    match parsed {
        Some(p) => Some(p),
        None => handle_invalid_computed_value(components, context),
    }
}

pub fn resolve_string_with_context(input: &str, context: &ResolveContext) -> Option<CssValue> {
    let components = crate::css::parser::parse_component_values(input);
    let vars_map = HashMap::new();
    resolve_value_with_context(&components, context, &vars_map)
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

/// A declaration in the CSS cascade, used to sort and resolve layered styles.
#[derive(Debug, Clone, PartialEq)]
pub struct CascadeDeclaration {
    pub value: String,
    pub is_important: bool,
    pub layer: Option<String>,
    pub specificity: (u32, u32, u32, u32),
    pub source_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerGroup {
    NormalLayer(String),
    NormalUnlayered,
    ImportantUnlayered,
    ImportantLayer(String),
}

impl CascadeDeclaration {
    pub fn layer_group(&self) -> LayerGroup {
        if self.is_important {
            if let Some(ref l) = self.layer {
                LayerGroup::ImportantLayer(l.clone())
            } else {
                LayerGroup::ImportantUnlayered
            }
        } else {
            if let Some(ref l) = self.layer {
                LayerGroup::NormalLayer(l.clone())
            } else {
                LayerGroup::NormalUnlayered
            }
        }
    }
}

pub fn get_group_priority(group: &LayerGroup, layer_order: &[String]) -> usize {
    match group {
        LayerGroup::NormalLayer(name) => layer_order.iter().position(|l| l == name).unwrap_or(0),
        LayerGroup::NormalUnlayered => layer_order.len(),
        LayerGroup::ImportantUnlayered => layer_order.len() + 1,
        LayerGroup::ImportantLayer(name) => {
            let idx = layer_order.iter().position(|l| l == name).unwrap_or(0);
            layer_order.len() + 1 + (layer_order.len() - idx)
        }
    }
}

pub fn compare_declarations(
    a: &CascadeDeclaration,
    b: &CascadeDeclaration,
    layer_order: &[String],
) -> std::cmp::Ordering {
    // 1. Importance
    if a.is_important != b.is_important {
        return a.is_important.cmp(&b.is_important);
    }

    // 2. Layer priority
    if a.is_important {
        // Important: layered > unlayered. Earlier layer > later layer.
        match (&a.layer, &b.layer) {
            (None, None) => {} // Both unlayered, proceed to specificity
            (None, Some(_)) => return std::cmp::Ordering::Less, // b is layered, so b > a
            (Some(_), None) => return std::cmp::Ordering::Greater, // a is layered, so a > b
            (Some(la), Some(lb)) => {
                if la != lb {
                    let idx_a = layer_order
                        .iter()
                        .position(|l| l == la)
                        .unwrap_or(usize::MAX);
                    let idx_b = layer_order
                        .iter()
                        .position(|l| l == lb)
                        .unwrap_or(usize::MAX);
                    // Earlier layer is higher priority for !important
                    return idx_b.cmp(&idx_a);
                }
            }
        }
    } else {
        // Normal: unlayered > layered. Later layer > earlier layer.
        match (&a.layer, &b.layer) {
            (None, None) => {} // Both unlayered, proceed to specificity
            (None, Some(_)) => return std::cmp::Ordering::Greater, // a is unlayered, so b is lower
            (Some(_), None) => return std::cmp::Ordering::Less, // b is unlayered, so b is higher
            (Some(la), Some(lb)) => {
                if la != lb {
                    let idx_a = layer_order
                        .iter()
                        .position(|l| l == la)
                        .unwrap_or(usize::MAX);
                    let idx_b = layer_order
                        .iter()
                        .position(|l| l == lb)
                        .unwrap_or(usize::MAX);
                    // Later layer is higher priority for normal
                    return idx_a.cmp(&idx_b);
                }
            }
        }
    }

    // 3. Specificity
    if a.specificity != b.specificity {
        return a.specificity.cmp(&b.specificity);
    }

    // 4. Source order
    a.source_order.cmp(&b.source_order)
}

fn resolve_cascade_under_priority(
    max_priority: usize,
    decls: &[CascadeDeclaration],
    layer_order: &[String],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    for i in (0..decls.len()).rev() {
        let decl = &decls[i];
        let group = decl.layer_group();
        let prio = get_group_priority(&group, layer_order);
        if prio < max_priority {
            let mut sub_context = context.clone();
            sub_context.revert_value = context.revert_value.clone();
            sub_context.revert_layer_value = resolve_cascade_under_priority(
                prio,
                decls,
                layer_order,
                context,
                custom_properties,
            );

            let components = crate::css::parser::parse_component_values(&decl.value);
            if let Some(val) =
                resolve_value_with_context(&components, &sub_context, custom_properties)
            {
                return Some(val);
            }
        }
    }
    context.revert_value.clone()
}

/// Resolves the winning declaration value from a list of layered cascade declarations.
pub fn resolve_cascade(
    decls: &[CascadeDeclaration],
    layer_order: &[String],
    context: &ResolveContext,
    custom_properties: &HashMap<String, Vec<ComponentValue>>,
) -> Option<CssValue> {
    if decls.is_empty() {
        return None;
    }
    let mut sorted_decls = decls.to_vec();
    sorted_decls.sort_by(|a, b| compare_declarations(a, b, layer_order));

    let last_idx = sorted_decls.len() - 1;
    let decl = &sorted_decls[last_idx];
    let group = decl.layer_group();
    let prio = get_group_priority(&group, layer_order);

    let mut sub_context = context.clone();
    sub_context.revert_value = context.revert_value.clone();
    sub_context.revert_layer_value = resolve_cascade_under_priority(
        prio,
        &sorted_decls,
        layer_order,
        context,
        custom_properties,
    );

    let components = crate::css::parser::parse_component_values(&decl.value);
    resolve_value_with_context(&components, &sub_context, custom_properties)
}

/// Expands a shorthand property declaration if its value is a CSS-wide keyword.
pub fn expand_shorthand_declaration(
    name: &str,
    components: &[ComponentValue],
) -> Option<Vec<(String, Vec<ComponentValue>)>> {
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
    let trimmed = &components[start..end];
    if trimmed.len() != 1 {
        return None;
    }
    let ident = match &trimmed[0] {
        ComponentValue::Token(CssToken::Ident(id)) => id,
        _ => return None,
    };
    let lower = ident.to_ascii_lowercase();
    if !matches!(
        lower.as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    ) {
        return None;
    }
    let longhands = crate::css::property::shorthand_longhands(name)?;
    let mut expanded = Vec::new();
    for lh in longhands {
        expanded.push((lh.to_string(), trimmed.to_vec()));
    }
    Some(expanded)
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
        assert_eq!(
            resolve_string("50vmin", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(400.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50vmax", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("2in", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(192.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("127cm", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(4800.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("1270mm", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(4800.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("6pc", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(96.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_level4_units() {
        let vars = HashMap::new();
        // Viewport Level 4 units
        assert_eq!(
            resolve_string("50svw", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50lvw", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50dvw", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50vi", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10svh", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10lvh", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10dvh", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10vb", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50svmin", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(400.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50lvmin", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(400.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50dvmin", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(400.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50svmax", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50lvmax", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("50dvmax", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(500.0, LengthUnit::Px))
        );

        // Absolute Quarter-millimeter
        // 10q = 2.5mm = 0.25cm
        assert_eq!(
            resolve_string("10q", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(10.0 * 2.4 / 2.54, LengthUnit::Px))
        );

        // Root font-relative Level 4 units
        // rex, rch, ric, rcap
        assert_eq!(
            resolve_string("2rex", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(16.0, LengthUnit::Px)) // 2 * 16.0 * 0.5
        );
        assert_eq!(
            resolve_string("2rch", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(16.0, LengthUnit::Px)) // 2 * 16.0 * 0.5
        );
        assert_eq!(
            resolve_string("2ric", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(32.0, LengthUnit::Px)) // 2 * 16.0
        );
        assert_eq!(
            resolve_string("2rcap", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(2.0 * 16.0 * 0.7, LengthUnit::Px))
        );

        // Font-relative Level 4 units with current font context
        // ic, cap
        let context = ResolveContext {
            root_font_size: 16.0,
            current_font_size: Some(20.0),
            ..ResolveContext::default()
        };

        let ic_components = crate::css::parser::parse_component_values("2ic");
        assert_eq!(
            resolve_value_with_context(&ic_components, &context, &vars),
            Some(CssValue::Length(40.0, LengthUnit::Px)) // 2 * 20.0
        );

        let cap_components = crate::css::parser::parse_component_values("2cap");
        assert_eq!(
            resolve_value_with_context(&cap_components, &context, &vars),
            Some(CssValue::Length(28.0, LengthUnit::Px)) // 2 * 20.0 * 0.7
        );

        // Level 4 units inside calc()
        assert_eq!(
            resolve_string("calc(10svw + 10svh)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(180.0, LengthUnit::Px)) // 100px + 80px
        );
        assert_eq!(
            resolve_string("calc(10q + 10q)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0 * 2.4 / 2.54, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("calc(2rex + 2rcap)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(16.0 + 2.0 * 16.0 * 0.7, LengthUnit::Px))
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
        // calc(10vmin + 10vmax) -> 10% of 800 + 10% of 1000 -> 80.0 + 100.0 = 180.0
        assert_eq!(
            resolve_string("calc(10vmin + 10vmax)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(180.0, LengthUnit::Px))
        );
        // calc(1in + 6pc) -> 96px + 96px = 192px
        assert_eq!(
            resolve_string("calc(1in + 6pc)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(192.0, LengthUnit::Px))
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

    #[test]
    fn test_resolve_exponential_math_fns() {
        let vars = HashMap::new();

        // sqrt
        assert_eq!(
            resolve_string("sqrt(9)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(3.0))
        );
        assert_eq!(
            resolve_string("sqrt(0)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(0.0))
        );
        assert_eq!(resolve_string("sqrt(-4)", 16.0, 1000.0, 800.0, &vars), None);
        assert_eq!(
            resolve_string("sqrt(9px)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // pow
        assert_eq!(
            resolve_string("pow(2, 3)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(8.0))
        );
        assert_eq!(
            resolve_string("pow(-2, 3)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(-8.0))
        );
        assert_eq!(
            resolve_string("pow(-2, 0.5)", 16.0, 1000.0, 800.0, &vars),
            None
        );
        assert_eq!(
            resolve_string("pow(2px, 3)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // hypot
        assert_eq!(
            resolve_string("hypot(3, 4)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(5.0))
        );
        assert_eq!(
            resolve_string("hypot(5)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(5.0))
        );
        assert_eq!(
            resolve_string("hypot(1, 2, 2)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(3.0))
        );
        assert_eq!(
            resolve_string("hypot(3px, 4px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(5.0, LengthUnit::Px))
        );

        // log
        assert_eq!(
            resolve_string("log(1)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(0.0))
        );
        if let Some(CssValue::Number(val)) = resolve_string("log(8, 2)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 3.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        assert_eq!(resolve_string("log(0)", 16.0, 1000.0, 800.0, &vars), None);
        assert_eq!(resolve_string("log(-5)", 16.0, 1000.0, 800.0, &vars), None);
        assert_eq!(
            resolve_string("log(8, 1)", 16.0, 1000.0, 800.0, &vars),
            None
        );
        assert_eq!(
            resolve_string("log(8, -2)", 16.0, 1000.0, 800.0, &vars),
            None
        );

        // exp
        assert_eq!(
            resolve_string("exp(0)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(1.0))
        );
        if let Some(CssValue::Number(val)) = resolve_string("exp(1)", 16.0, 1000.0, 800.0, &vars) {
            assert!((val - std::f32::consts::E).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        assert_eq!(resolve_string("exp(1px)", 16.0, 1000.0, 800.0, &vars), None);

        // nested math functions
        assert_eq!(
            resolve_string("sqrt(pow(2, 4))", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(4.0))
        );
    }

    #[test]
    fn test_resolve_t0795_css_math_env() {
        let vars = HashMap::new();

        // 1. Trig functions: sin, cos, tan
        assert_eq!(
            resolve_string("sin(0)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(0.0))
        );
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(90deg)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(0.25turn)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(100grad)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(1.5707963rad)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }

        assert_eq!(
            resolve_string("cos(0deg)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(1.0))
        );
        if let Some(CssValue::Number(val)) =
            resolve_string("cos(180deg)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - -1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }

        assert_eq!(
            resolve_string("tan(0)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(0.0))
        );
        if let Some(CssValue::Number(val)) =
            resolve_string("tan(45deg)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }

        // 2. Inverse trig functions and nesting inside calc()
        if let Some(CssValue::Number(val)) =
            resolve_string("calc(sin(asin(0.5)))", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 0.5).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("calc(cos(acos(0.5)))", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 0.5).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("calc(tan(atan(1)))", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("calc(tan(atan2(1, 1)))", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 1.0).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }

        // 3. Multiplication/addition of length and sin/cos values
        if let Some(CssValue::Length(val, _)) =
            resolve_string("calc(10px * sin(30deg))", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 5.0).abs() < 1e-4);
        } else {
            panic!("Expected a length");
        }

        // 4. Constants: pi, e, infinity, nan
        if let Some(CssValue::Number(val)) = resolve_string("calc(pi)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - std::f32::consts::PI).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) = resolve_string("calc(e)", 16.0, 1000.0, 800.0, &vars) {
            assert!((val - std::f32::consts::E).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(pi / 6)", 16.0, 1000.0, 800.0, &vars)
        {
            assert!((val - 0.5).abs() < 1e-5);
        } else {
            panic!("Expected a number");
        }
        assert_eq!(
            resolve_string("calc(infinity)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(f32::INFINITY))
        );
        assert_eq!(
            resolve_string("calc(-infinity)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Number(f32::NEG_INFINITY))
        );

        // 5. env() variables
        assert_eq!(
            resolve_string("env(safe-area-inset-top)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(0.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("env(safe-area-inset-top, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(0.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("env(custom-unknown-env, 20px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string(
                "calc(env(safe-area-inset-left) + 10px)",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string(
                "calc(env(custom-unknown-env, 5rem) * 2)",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Length(160.0, LengthUnit::Px))
        );
    }

    #[test]
    fn test_resolve_extended_t0843() {
        // 1. Relative Units resolution (em, ex, ch, lh, rlh)
        let ctx_relative = ResolveContext {
            root_font_size: 16.0,
            current_font_size: Some(24.0),
            line_height: Some(40.0),
            root_line_height: Some(20.0),
            viewport_w: 1000.0,
            viewport_h: 800.0,
            ..Default::default()
        };

        // Direct resolving
        assert_eq!(
            resolve_string_with_context("2em", &ctx_relative),
            Some(CssValue::Length(48.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string_with_context("3ex", &ctx_relative),
            Some(CssValue::Length(36.0, LengthUnit::Px)) // 3 * 24.0 * 0.5 = 36
        );
        assert_eq!(
            resolve_string_with_context("4ch", &ctx_relative),
            Some(CssValue::Length(48.0, LengthUnit::Px)) // 4 * 24.0 * 0.5 = 48
        );
        assert_eq!(
            resolve_string_with_context("2lh", &ctx_relative),
            Some(CssValue::Length(80.0, LengthUnit::Px)) // 2 * 40.0 = 80
        );
        assert_eq!(
            resolve_string_with_context("3rlh", &ctx_relative),
            Some(CssValue::Length(60.0, LengthUnit::Px)) // 3 * 20.0 = 60
        );

        // Fallbacks when current/line_height are None
        let ctx_relative_fallbacks = ResolveContext {
            root_font_size: 16.0,
            current_font_size: None,
            line_height: None,
            root_line_height: None,
            viewport_w: 1000.0,
            viewport_h: 800.0,
            ..Default::default()
        };
        // em without current_font_size stays as Em
        assert_eq!(
            resolve_string_with_context("2em", &ctx_relative_fallbacks),
            Some(CssValue::Length(2.0, LengthUnit::Em))
        );
        // ex, ch without current_font_size use root_font_size
        assert_eq!(
            resolve_string_with_context("2ex", &ctx_relative_fallbacks),
            Some(CssValue::Length(16.0, LengthUnit::Px)) // 2 * 16 * 0.5 = 16
        );
        // lh without current_font_size and line_height uses 1.2 * root_font_size
        assert_eq!(
            resolve_string_with_context("2lh", &ctx_relative_fallbacks),
            Some(CssValue::Length(38.4, LengthUnit::Px)) // 2 * 16 * 1.2 = 38.4
        );
        // rlh without root_line_height uses 1.2 * root_font_size
        assert_eq!(
            resolve_string_with_context("2rlh", &ctx_relative_fallbacks),
            Some(CssValue::Length(38.4, LengthUnit::Px)) // 2 * 16 * 1.2 = 38.4
        );

        // Within calc
        assert_eq!(
            resolve_string_with_context("calc(1em + 10px)", &ctx_relative),
            Some(CssValue::Length(34.0, LengthUnit::Px)) // 24 + 10 = 34
        );
        assert_eq!(
            resolve_string_with_context("calc(2ex * 2)", &ctx_relative),
            Some(CssValue::Length(48.0, LengthUnit::Px)) // 2 * 12 * 2 = 48
        );
        assert_eq!(
            resolve_string_with_context("calc(1lh / 2)", &ctx_relative),
            Some(CssValue::Length(20.0, LengthUnit::Px)) // 40 / 2 = 20
        );

        // 2. inherit / initial / unset handling
        let parent_color = CssValue::Color(crate::css::values::Color::Rgba(255, 0, 0, 255));
        let ctx_keywords = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_color.clone()),
            ..Default::default()
        };

        // inherit resolves to parent's value
        assert_eq!(
            resolve_string_with_context("inherit", &ctx_keywords),
            Some(parent_color.clone())
        );

        // initial resolves to property's initial value ("black")
        assert_eq!(
            resolve_string_with_context("initial", &ctx_keywords),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // unset on color (inherited) resolves to parent's value
        assert_eq!(
            resolve_string_with_context("unset", &ctx_keywords),
            Some(parent_color.clone())
        );

        // unset on color (inherited) with no parent resolves to initial value
        let ctx_keywords_no_parent = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: None,
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("unset", &ctx_keywords_no_parent),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // unset on background-color (not inherited, initial: "transparent")
        let ctx_non_inherited = ResolveContext {
            property_name: Some("background-color".to_string()),
            parent_value: Some(parent_color.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("unset", &ctx_non_inherited),
            Some(CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 0)))
        );

        // 3. Percentage Resolution
        let ctx_percentage = ResolveContext {
            percentage_basis: Some(500.0),
            ..Default::default()
        };

        // Direct percentage resolving to Px
        assert_eq!(
            resolve_string_with_context("50%", &ctx_percentage),
            Some(CssValue::Length(250.0, LengthUnit::Px))
        );

        // Percentage within calc
        assert_eq!(
            resolve_string_with_context("calc(10% + 50px)", &ctx_percentage),
            Some(CssValue::Length(100.0, LengthUnit::Px)) // 50 + 50 = 100
        );
        assert_eq!(
            resolve_string_with_context("calc(10% + 20%)", &ctx_percentage),
            Some(CssValue::Length(150.0, LengthUnit::Px)) // 50 + 100 = 150
        );
    }

    #[test]
    fn test_resolve_t0895_additive() {
        let vars = HashMap::new();

        // 1. New viewport units
        assert_eq!(
            resolve_string("10svi", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10lvi", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10dvi", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10svb", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10lvb", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10dvb", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );

        // 2. Container query units falling back to viewport units
        assert_eq!(
            resolve_string("10cqw", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10cqh", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10cqi", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10cqb", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10cqmin", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );
        assert_eq!(
            resolve_string("10cqmax", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );

        // 3. hypot with dimensions/units inside calc and directly
        assert_eq!(
            resolve_string("hypot(3px, 4px)", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Length(5.0, LengthUnit::Px))
        );
        if let Some(CssValue::Number(val)) =
            resolve_string("sin(hypot(30deg, 40deg))", 16.0, 1000.0, 800.0, &vars)
        {
            let expected_sin = (50.0f32).to_radians().sin();
            assert!((val - expected_sin).abs() < 1e-5);
        } else {
            panic!("Expected a number for sin(hypot(30deg, 40deg))");
        }
    }

    #[test]
    fn test_resolve_multi_component_and_nested_fns() {
        let vars = HashMap::new();

        // 1. Resolve relative units and math functions in multi-component values (lists)
        assert_eq!(
            resolve_string("1rem 2rem", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Multiple(vec![
                CssValue::Length(16.0, LengthUnit::Px),
                CssValue::Length(32.0, LengthUnit::Px)
            ]))
        );
        assert_eq!(
            resolve_string("calc(10px + 5px) 20px", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Multiple(vec![
                CssValue::Length(15.0, LengthUnit::Px),
                CssValue::Length(20.0, LengthUnit::Px)
            ]))
        );

        // 2. Comma and Slash separation within multi-component values
        assert_eq!(
            resolve_string("1rem / 2rem", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Multiple(vec![
                CssValue::Length(16.0, LengthUnit::Px),
                CssValue::Keyword("/".to_string()),
                CssValue::Length(32.0, LengthUnit::Px)
            ]))
        );
        assert_eq!(
            resolve_string("calc(10px + 5px), 20px", 16.0, 1000.0, 800.0, &vars),
            Some(CssValue::Multiple(vec![
                CssValue::Length(15.0, LengthUnit::Px),
                CssValue::Keyword(",".to_string()),
                CssValue::Length(20.0, LengthUnit::Px)
            ]))
        );

        // 3. Resolve inside standard/custom function arguments recursively
        assert_eq!(
            resolve_string(
                "translate(2rem, calc(10px * 3))",
                16.0,
                1000.0,
                800.0,
                &vars
            ),
            Some(CssValue::Transform(vec![
                crate::css::values::TransformFn::Translate {
                    x: crate::css::values::LengthOrPercent {
                        value: 32.0,
                        unit: crate::css::values::LengthUnit::Px,
                    },
                    y: crate::css::values::LengthOrPercent {
                        value: 30.0,
                        unit: crate::css::values::LengthUnit::Px,
                    },
                }
            ]))
        );
    }

    #[test]
    fn test_currentcolor_propagation() {
        // Test basic currentColor propagation as a standalone value.
        let ctx_standalone = ResolveContext {
            current_color: Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 100, 50, 255,
            ))),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("currentcolor", &ctx_standalone),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 100, 50, 255
            )))
        );

        // Test basic currentColor fallback to initial/default (black) when not set.
        let ctx_no_current = ResolveContext {
            current_color: None,
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("currentcolor", &ctx_no_current),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // Test currentColor on the color property itself with parent color (behaves as inherit).
        let parent_color = CssValue::Color(crate::css::values::Color::Rgba(0, 255, 0, 255));
        let ctx_color_prop_inherit = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_color.clone()),
            current_color: Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255,
            ))), // Should ignore this and inherit
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("currentcolor", &ctx_color_prop_inherit),
            Some(parent_color)
        );

        // Test currentColor inside multi-component values (e.g. lists).
        let ctx_multi = ResolveContext {
            current_color: Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255,
            ))),
            ..Default::default()
        };
        let comps = crate::css::parser::parse_component_values("1px solid currentcolor");
        let resolved = resolve_components(&comps, &ctx_multi, &HashMap::new()).unwrap();
        // The resulting components should have currentcolor replaced by the hex formatted color.
        let parsed = crate::css::values::parse_value(&resolved);
        assert!(parsed.is_some());
    }

    #[test]
    fn test_resolve_revert_keywords() {
        // 1. revert keyword with explicit revert_value
        let explicit_revert = CssValue::Length(100.0, LengthUnit::Px);
        let ctx_revert = ResolveContext {
            revert_value: Some(explicit_revert.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("revert", &ctx_revert),
            Some(explicit_revert)
        );

        // 2. revert keyword fallback (behaves as unset)
        // For inherited property (e.g. "color"), fallback inherits from parent value
        let parent_color = CssValue::Color(crate::css::values::Color::Rgba(255, 0, 0, 255));
        let ctx_revert_inherited = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_color.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("revert", &ctx_revert_inherited),
            Some(parent_color.clone())
        );

        // For non-inherited property (e.g. "background-color"), fallback resolves to initial
        let ctx_revert_non_inherited = ResolveContext {
            property_name: Some("background-color".to_string()),
            parent_value: Some(parent_color.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("revert", &ctx_revert_non_inherited),
            Some(CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 0)))
        );

        // 3. revert-layer keyword with explicit revert_layer_value
        let explicit_revert_layer = CssValue::Length(50.0, LengthUnit::Px);
        let ctx_revert_layer = ResolveContext {
            revert_layer_value: Some(explicit_revert_layer.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("revert-layer", &ctx_revert_layer),
            Some(explicit_revert_layer.clone())
        );

        // 4. revert-layer falls back to revert_value
        let ctx_revert_layer_to_revert = ResolveContext {
            revert_value: Some(explicit_revert_layer.clone()),
            ..Default::default()
        };
        assert_eq!(
            resolve_string_with_context("revert-layer", &ctx_revert_layer_to_revert),
            Some(explicit_revert_layer.clone())
        );

        // 5. revert-layer falls back to unset (inherited/initial) when no rollback is given
        assert_eq!(
            resolve_string_with_context("revert-layer", &ctx_revert_inherited),
            Some(parent_color)
        );
        assert_eq!(
            resolve_string_with_context("revert-layer", &ctx_revert_non_inherited),
            Some(CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 0)))
        );
    }

    #[test]
    fn test_resolve_t0988_custom_property_correctness() {
        // 1. Cycle detection ignoring internal fallbacks, but resolving to top-level fallback
        let mut vars = HashMap::new();
        vars.insert(
            "--foo".to_string(),
            crate::css::parser::parse_component_values("var(--bar, 10px)"),
        );
        vars.insert(
            "--bar".to_string(),
            crate::css::parser::parse_component_values("var(--foo, 20px)"),
        );

        // A standard property color referencing --foo with a 30px fallback should resolve to 30px
        let ctx_top_fallback = ResolveContext {
            property_name: Some("color".to_string()),
            ..Default::default()
        };
        let components = crate::css::parser::parse_component_values("var(--foo, 30px)");
        assert_eq!(
            resolve_value_with_context(&components, &ctx_top_fallback, &vars),
            Some(CssValue::Length(30.0, LengthUnit::Px))
        );

        // 2. Custom property with missing reference falling back to its fallback
        let mut vars2 = HashMap::new();
        vars2.insert(
            "--baz".to_string(),
            crate::css::parser::parse_component_values("var(--missing, 10px)"),
        );
        let components2 = crate::css::parser::parse_component_values("var(--baz, 20px)");
        assert_eq!(
            resolve_value_with_context(&components2, &ctx_top_fallback, &vars2),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );

        // 3. Invalid at computed-value time fallback to unset: inherited property inherits parent color
        let parent_color = CssValue::Color(crate::css::values::Color::Rgba(0, 255, 0, 255));
        let ctx_inherited = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_color.clone()),
            ..Default::default()
        };
        // var(--missing) has no fallback, so the standard color property is invalid at computed-value time
        let components3 = crate::css::parser::parse_component_values("var(--missing)");
        assert_eq!(
            resolve_value_with_context(&components3, &ctx_inherited, &vars),
            Some(parent_color)
        );

        // 4. Invalid at computed-value time fallback to unset: non-inherited property resets to initial
        let ctx_non_inherited = ResolveContext {
            property_name: Some("background-color".to_string()),
            parent_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255,
            ))),
            ..Default::default()
        };
        assert_eq!(
            resolve_value_with_context(&components3, &ctx_non_inherited, &vars),
            Some(CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 0))) // initial background-color is transparent
        );

        // 5. Invalid syntax after substitution (invalid at computed-value time) resets to unset
        let mut vars_invalid_val = HashMap::new();
        vars_invalid_val.insert(
            "--bad-val".to_string(),
            crate::css::parser::parse_component_values("calc(10px +)"),
        );
        // color: var(--bad-val) -> color: calc(10px +) (syntactically invalid for color property)
        let components4 = crate::css::parser::parse_component_values("var(--bad-val)");
        assert_eq!(
            resolve_value_with_context(&components4, &ctx_inherited, &vars_invalid_val),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 255, 0, 255
            ))) // inherits parent color
        );
    }

    #[test]
    fn test_cascade_layer_ordering_and_revert() {
        let layers = vec![
            "base".to_string(),
            "theme".to_string(),
            "utilities".to_string(),
        ];
        let custom_props = HashMap::new();
        let default_ctx = ResolveContext::default();

        // 1. Normal unlayered declaration beats normal layered declaration
        let d_base = CascadeDeclaration {
            value: "red".to_string(),
            is_important: false,
            layer: Some("base".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 1,
        };
        let d_unlayered = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };

        let mut decls = [d_base.clone(), d_unlayered.clone()];
        decls.sort_by(|a, b| compare_declarations(a, b, &layers));
        // unlayered should be higher priority, thus sorted last
        assert_eq!(decls[1].layer, None);

        // 2. Important layered declaration beats important unlayered declaration
        let d_base_important = CascadeDeclaration {
            value: "red".to_string(),
            is_important: true,
            layer: Some("base".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 1,
        };
        let d_unlayered_important = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: true,
            layer: None,
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };

        let mut decls = [d_base_important.clone(), d_unlayered_important.clone()];
        decls.sort_by(|a, b| compare_declarations(a, b, &layers));
        // base important should be higher priority, thus sorted last
        assert_eq!(decls[1].layer, Some("base".to_string()));

        // 3. Normal layer order: later layer wins (utilities > theme)
        let d_theme = CascadeDeclaration {
            value: "green".to_string(),
            is_important: false,
            layer: Some("theme".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 1,
        };
        let d_utilities = CascadeDeclaration {
            value: "yellow".to_string(),
            is_important: false,
            layer: Some("utilities".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };

        let mut decls = [d_theme.clone(), d_utilities.clone()];
        decls.sort_by(|a, b| compare_declarations(a, b, &layers));
        assert_eq!(decls[1].layer, Some("utilities".to_string()));

        // 4. Important layer order: earlier layer wins (theme > utilities)
        let d_theme_important = CascadeDeclaration {
            value: "green".to_string(),
            is_important: true,
            layer: Some("theme".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 1,
        };
        let d_utilities_important = CascadeDeclaration {
            value: "yellow".to_string(),
            is_important: true,
            layer: Some("utilities".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };

        let mut decls = [d_theme_important.clone(), d_utilities_important.clone()];
        decls.sort_by(|a, b| compare_declarations(a, b, &layers));
        assert_eq!(decls[1].layer, Some("theme".to_string()));

        // 5. Revert-layer rolls back to the next lower-priority layer (utilities rolls back to theme)
        let d_utilities_revert = CascadeDeclaration {
            value: "revert-layer".to_string(),
            is_important: false,
            layer: Some("utilities".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };
        let decls = vec![d_theme.clone(), d_utilities_revert.clone()];
        let res = resolve_cascade(&decls, &layers, &default_ctx, &custom_props);
        assert_eq!(
            res,
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 128, 0, 255
            )))
        );

        // 6. Important revert-layer rolls back to next lower-priority important layer
        let d_theme_revert_important = CascadeDeclaration {
            value: "revert-layer".to_string(),
            is_important: true,
            layer: Some("theme".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };
        // For utilities important, let's use "blue" which is parsed as Rgba(0, 0, 255, 255)
        let d_utilities_blue_important = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: true,
            layer: Some("utilities".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };
        let decls = vec![
            d_theme_revert_important.clone(),
            d_utilities_blue_important.clone(),
        ];
        let res = resolve_cascade(&decls, &layers, &default_ctx, &custom_props);
        assert_eq!(
            res,
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255
            )))
        );

        // 7. Chained rollback (utilities rolls back to theme which rolls back to base)
        let d_theme_revert = CascadeDeclaration {
            value: "revert-layer".to_string(),
            is_important: false,
            layer: Some("theme".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 1,
        };
        let d_base_color = CascadeDeclaration {
            value: "red".to_string(),
            is_important: false,
            layer: Some("base".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 0,
        };
        let decls = vec![d_base_color, d_theme_revert, d_utilities_revert];
        let res = resolve_cascade(&decls, &layers, &default_ctx, &custom_props);
        assert_eq!(
            res,
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255
            )))
        );

        // 8. Revert-layer on lowest layer rolls back to revert_value
        let ctx_with_revert = ResolveContext {
            revert_value: Some(CssValue::Length(100.0, LengthUnit::Px)),
            ..Default::default()
        };
        let d_base_revert = CascadeDeclaration {
            value: "revert-layer".to_string(),
            is_important: false,
            layer: Some("base".to_string()),
            specificity: (0, 0, 1, 0),
            source_order: 0,
        };
        let decls = vec![d_base_revert];
        let res = resolve_cascade(&decls, &layers, &ctx_with_revert, &custom_props);
        assert_eq!(res, Some(CssValue::Length(100.0, LengthUnit::Px)));
    }

    #[test]
    fn test_shorthand_wide_keyword_expansion() {
        // 1. margin shorthand with CSS-wide keyword "inherit"
        let comps = crate::css::parser::parse_component_values("inherit");
        let expanded = expand_shorthand_declaration("margin", &comps).unwrap();
        assert_eq!(expanded.len(), 4);
        assert_eq!(expanded[0].0, "margin-top");
        assert_eq!(expanded[0].1, comps);
        assert_eq!(expanded[3].0, "margin-left");

        // 2. padding shorthand with CSS-wide keyword "initial"
        let comps_initial = crate::css::parser::parse_component_values("  initial  ");
        let expanded_padding = expand_shorthand_declaration("padding", &comps_initial).unwrap();
        assert_eq!(expanded_padding.len(), 4);
        assert_eq!(expanded_padding[0].0, "padding-top");

        // 3. Non-shorthand properties like "color" should return None
        let color_comps = crate::css::parser::parse_component_values("inherit");
        assert!(expand_shorthand_declaration("color", &color_comps).is_none());

        // 4. Non-keyword value should return None
        let normal_comps = crate::css::parser::parse_component_values("10px");
        assert!(expand_shorthand_declaration("margin", &normal_comps).is_none());
    }

    #[test]
    fn test_t1034_cascade_specificity_and_layers() {
        let layers = vec!["base".to_string(), "theme".to_string()];

        // Let's verify that important layered beats important unlayered
        let decl_unlayered_imp = CascadeDeclaration {
            value: "red".to_string(),
            is_important: true,
            layer: None,
            specificity: (0, 1, 0, 0), // Has high specificity
            source_order: 1,
        };
        let decl_layered_imp = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: true,
            layer: Some("base".to_string()),
            specificity: (0, 0, 0, 1), // Has low specificity
            source_order: 2,
        };
        // Specificity shouldn't matter; layer priority for important wins
        assert_eq!(
            compare_declarations(&decl_unlayered_imp, &decl_layered_imp, &layers),
            std::cmp::Ordering::Less
        );

        // Let's verify that normal unlayered beats normal layered
        let decl_unlayered_norm = CascadeDeclaration {
            value: "red".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 0, 0, 1), // Has low specificity
            source_order: 1,
        };
        let decl_layered_norm = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: false,
            layer: Some("base".to_string()),
            specificity: (0, 1, 0, 0), // Has high specificity
            source_order: 2,
        };
        // Layer priority for normal wins (unlayered beats layered)
        assert_eq!(
            compare_declarations(&decl_unlayered_norm, &decl_layered_norm, &layers),
            std::cmp::Ordering::Greater
        );

        // Specificity comparison for same layer (e.g., both unlayered)
        let decl_spec_high = CascadeDeclaration {
            value: "red".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 1, 0, 0),
            source_order: 1,
        };
        let decl_spec_low = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 0, 1, 0),
            source_order: 2,
        };
        assert_eq!(
            compare_declarations(&decl_spec_high, &decl_spec_low, &layers),
            std::cmp::Ordering::Greater
        );

        // Source order comparison for same specificity and layer
        let decl_source_early = CascadeDeclaration {
            value: "red".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 1, 0, 0),
            source_order: 1,
        };
        let decl_source_late = CascadeDeclaration {
            value: "blue".to_string(),
            is_important: false,
            layer: None,
            specificity: (0, 1, 0, 0),
            source_order: 2,
        };
        assert_eq!(
            compare_declarations(&decl_source_early, &decl_source_late, &layers),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_t1034_keyword_resolution_details() {
        let custom_props = HashMap::new();

        // 1. Initial keyword
        let ctx_initial = ResolveContext {
            property_name: Some("color".to_string()),
            ..Default::default()
        };
        let comps_initial = crate::css::parser::parse_component_values("initial");
        // Initial value of color is black (#000000)
        assert_eq!(
            resolve_value_with_context(&comps_initial, &ctx_initial, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // 2. Inherit keyword with parent value
        let parent_val = CssValue::Color(crate::css::values::Color::Rgba(0, 255, 0, 255));
        let ctx_inherit = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_val.clone()),
            ..Default::default()
        };
        let comps_inherit = crate::css::parser::parse_component_values("inherit");
        assert_eq!(
            resolve_value_with_context(&comps_inherit, &ctx_inherit, &custom_props),
            Some(parent_val)
        );

        // Inherit with no parent falls back to initial
        let ctx_inherit_no_parent = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: None,
            ..Default::default()
        };
        assert_eq!(
            resolve_value_with_context(&comps_inherit, &ctx_inherit_no_parent, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // 3. Unset keyword
        // On inherited properties (like color), unset acts as inherit
        let ctx_unset_inh = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255,
            ))),
            ..Default::default()
        };
        let comps_unset = crate::css::parser::parse_component_values("unset");
        assert_eq!(
            resolve_value_with_context(&comps_unset, &ctx_unset_inh, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255
            )))
        );

        // On non-inherited properties (like background-color), unset acts as initial
        let ctx_unset_non_inh = ResolveContext {
            property_name: Some("background-color".to_string()),
            parent_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 255, 255,
            ))),
            ..Default::default()
        };
        assert_eq!(
            resolve_value_with_context(&comps_unset, &ctx_unset_non_inh, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(0, 0, 0, 0))) // transparent
        );

        // 4. Revert keyword
        let ctx_revert = ResolveContext {
            property_name: Some("color".to_string()),
            revert_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 255, 0, 255,
            ))),
            ..Default::default()
        };
        let comps_revert = crate::css::parser::parse_component_values("revert");
        assert_eq!(
            resolve_value_with_context(&comps_revert, &ctx_revert, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 255, 0, 255
            )))
        );

        // Revert falls back to inherit if no revert_value is set
        let ctx_revert_fallback = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255,
            ))),
            revert_value: None,
            ..Default::default()
        };
        assert_eq!(
            resolve_value_with_context(&comps_revert, &ctx_revert_fallback, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                255, 0, 0, 255
            )))
        );
    }

    #[test]
    fn test_t1034_custom_property_substitution_correctness() {
        let mut custom_props = HashMap::new();
        custom_props.insert(
            "--nested-a".to_string(),
            crate::css::parser::parse_component_values("var(--nested-b)"),
        );
        custom_props.insert(
            "--nested-b".to_string(),
            crate::css::parser::parse_component_values("var(--nested-c)"),
        );
        custom_props.insert(
            "--nested-c".to_string(),
            crate::css::parser::parse_component_values("20px"),
        );

        // Check deep nested custom property
        assert_eq!(
            resolve_string("var(--nested-a)", 16.0, 1000.0, 800.0, &custom_props),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );

        // Check cycle with a fallback on the outer var reference
        // --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a);
        custom_props.insert(
            "--cycle-a".to_string(),
            crate::css::parser::parse_component_values("var(--cycle-b)"),
        );
        custom_props.insert(
            "--cycle-b".to_string(),
            crate::css::parser::parse_component_values("var(--cycle-a)"),
        );
        assert_eq!(
            resolve_string("var(--cycle-a, 50px)", 16.0, 1000.0, 800.0, &custom_props),
            Some(CssValue::Length(50.0, LengthUnit::Px))
        );

        // Check custom property containing a CSS-wide keyword triggering parent inheritance
        custom_props.insert(
            "--my-keyword".to_string(),
            crate::css::parser::parse_component_values("inherit"),
        );
        let parent_val = CssValue::Color(crate::css::values::Color::Rgba(128, 0, 128, 255));
        let ctx = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(parent_val.clone()),
            ..Default::default()
        };
        let comps = crate::css::parser::parse_component_values("var(--my-keyword)");
        assert_eq!(
            resolve_value_with_context(&comps, &ctx, &custom_props),
            Some(parent_val)
        );

        // Check custom property containing a CSS-wide keyword triggering initial value
        let ctx_initial = ResolveContext {
            property_name: Some("color".to_string()),
            ..Default::default()
        };
        custom_props.insert(
            "--my-initial-kw".to_string(),
            crate::css::parser::parse_component_values("initial"),
        );
        let comps_init = crate::css::parser::parse_component_values("var(--my-initial-kw)");
        assert_eq!(
            resolve_value_with_context(&comps_init, &ctx_initial, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 0, 0, 255
            )))
        );

        // Check empty custom property resulting in invalid-at-computed-value-time and falling back to unset
        custom_props.insert(
            "--empty-val".to_string(),
            crate::css::parser::parse_component_values("   "),
        );
        let comps_empty = crate::css::parser::parse_component_values("var(--empty-val)");
        let ctx_empty = ResolveContext {
            property_name: Some("color".to_string()),
            parent_value: Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 255, 0, 255,
            ))),
            ..Default::default()
        };
        assert_eq!(
            resolve_value_with_context(&comps_empty, &ctx_empty, &custom_props),
            Some(CssValue::Color(crate::css::values::Color::Rgba(
                0, 255, 0, 255
            ))) // inherits parent color
        );
    }
}
