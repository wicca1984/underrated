use super::CssToken;
use super::parser::ComponentValue;

#[derive(Debug, PartialEq, Clone)]
pub enum LengthUnit {
    Px,
    Em,
    Rem,
    Pt,
    Percent,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    Rgba(u8, u8, u8, u8),
}

#[derive(Debug, PartialEq, Clone)]
pub enum CssValue {
    Keyword(String),
    Length(f32, LengthUnit),
    Number(f32),
    Color(Color),
    Multiple(Vec<CssValue>),
}

/// Parses a list of component values into a typed CSS value.
/// spec: <https://www.w3.org/TR/css-values-4/>
pub fn parse_value(components: &[ComponentValue]) -> Option<CssValue> {
    let mut values = Vec::new();
    let mut current_group = Vec::new();

    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {
                if !current_group.is_empty() {
                    if let Some(val) = parse_single_value(&current_group) {
                        values.push(val);
                        current_group.clear();
                    } else {
                        return None;
                    }
                }
            }
            _ => {
                current_group.push(component);
            }
        }
    }

    if !current_group.is_empty() {
        if let Some(val) = parse_single_value(&current_group) {
            values.push(val);
        } else {
            return None;
        }
    }

    match values.len() {
        0 => None,
        1 => Some(values.remove(0)),
        _ => Some(CssValue::Multiple(values)),
    }
}

fn parse_single_value(components: &[&ComponentValue]) -> Option<CssValue> {
    if components.len() != 1 {
        // TODO(spec): Support complex single values (e.g. 1px/2px)
        return None;
    }

    match components[0] {
        ComponentValue::Token(CssToken::Ident(s)) => {
            if let Some(color) = parse_named_color(s) {
                Some(CssValue::Color(color))
            } else {
                Some(CssValue::Keyword(s.clone()))
            }
        }
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let unit_enum = match unit.to_ascii_lowercase().as_str() {
                "px" => LengthUnit::Px,
                "em" => LengthUnit::Em,
                "rem" => LengthUnit::Rem,
                "pt" => LengthUnit::Pt,
                _ => return None, // TODO(spec): other units
            };
            Some(CssValue::Length(*value as f32, unit_enum))
        }
        ComponentValue::Token(CssToken::Percentage(v)) => {
            Some(CssValue::Length(*v as f32, LengthUnit::Percent))
        }
        ComponentValue::Token(CssToken::Number(v)) => Some(CssValue::Number(*v as f32)),
        ComponentValue::Token(CssToken::Hash(s)) => parse_hex_color(s).map(CssValue::Color),
        ComponentValue::Function { name, value } => {
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") {
                return parse_rgb_function(value).map(CssValue::Color);
            }
            None // TODO(spec): other functions
        }
        _ => None,
    }
}

fn parse_named_color(name: &str) -> Option<Color> {
    match name.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Rgba(0, 0, 0, 255)),
        "white" => Some(Color::Rgba(255, 255, 255, 255)),
        "red" => Some(Color::Rgba(255, 0, 0, 255)),
        "green" => Some(Color::Rgba(0, 128, 0, 255)),
        "blue" => Some(Color::Rgba(0, 0, 255, 255)),
        "transparent" => Some(Color::Rgba(0, 0, 0, 0)),
        _ => None,
    }
}

fn parse_hex_color(s: &str) -> Option<Color> {
    if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()?;
        let g = u8::from_str_radix(&s[1..2], 16).ok()?;
        let b = u8::from_str_radix(&s[2..3], 16).ok()?;
        Some(Color::Rgba(r * 17, g * 17, b * 17, 255))
    } else if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgba(r, g, b, 255))
    } else {
        None
    }
}

fn parse_rgb_function(components: &[ComponentValue]) -> Option<Color> {
    // Basic support for rgb(r, g, b) or rgba(r, g, b, a)
    // Filter out whitespace and commas
    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(*v as f32),
            ComponentValue::Token(CssToken::Percentage(v)) => {
                args.push((*v as f32 / 100.0) * 255.0)
            }
            _ => return None,
        }
    }

    if args.len() == 3 {
        Some(Color::Rgba(
            args[0].clamp(0.0, 255.0) as u8,
            args[1].clamp(0.0, 255.0) as u8,
            args[2].clamp(0.0, 255.0) as u8,
            255,
        ))
    } else if args.len() == 4 {
        Some(Color::Rgba(
            args[0].clamp(0.0, 255.0) as u8,
            args[1].clamp(0.0, 255.0) as u8,
            args[2].clamp(0.0, 255.0) as u8,
            (args[3].clamp(0.0, 1.0) * 255.0) as u8,
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::CssToken;

    fn token(t: CssToken) -> ComponentValue {
        ComponentValue::Token(t)
    }

    #[test]
    fn test_parse_keyword() {
        let components = [token(CssToken::Ident("auto".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Keyword("auto".to_string()))
        );
    }

    #[test]
    fn test_parse_length() {
        let components = [token(CssToken::Dimension {
            value: 10.0,
            unit: "px".to_string(),
        })];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );

        let components = [token(CssToken::Dimension {
            value: 1.5,
            unit: "em".to_string(),
        })];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Length(1.5, LengthUnit::Em))
        );

        let components = [token(CssToken::Dimension {
            value: 2.0,
            unit: "rem".to_string(),
        })];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Length(2.0, LengthUnit::Rem))
        );

        let components = [token(CssToken::Dimension {
            value: 12.0,
            unit: "pt".to_string(),
        })];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Length(12.0, LengthUnit::Pt))
        );
    }

    #[test]
    fn test_parse_percentage() {
        let components = [token(CssToken::Percentage(50.0))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Length(50.0, LengthUnit::Percent))
        );
    }

    #[test]
    fn test_parse_number() {
        let components = [token(CssToken::Number(1.5))];
        assert_eq!(parse_value(&components), Some(CssValue::Number(1.5)));
    }

    #[test]
    fn test_parse_color_hex() {
        // #ff0000
        let components = [token(CssToken::Hash("ff0000".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // #f00
        let components = [token(CssToken::Hash("f00".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
    }

    #[test]
    fn test_parse_color_named() {
        let components = [token(CssToken::Ident("red".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        let components = [token(CssToken::Ident("transparent".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 0)))
        );
    }

    #[test]
    fn test_parse_color_rgb() {
        // rgb(0, 128, 0)
        let components = [ComponentValue::Function {
            name: "rgb".to_string(),
            value: vec![
                token(CssToken::Number(0.0)),
                token(CssToken::Comma),
                token(CssToken::Whitespace),
                token(CssToken::Number(128.0)),
                token(CssToken::Comma),
                token(CssToken::Whitespace),
                token(CssToken::Number(0.0)),
            ],
        }];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(0, 128, 0, 255)))
        );

        // rgba(0, 0, 255, 0.5)
        let components = [ComponentValue::Function {
            name: "rgba".to_string(),
            value: vec![
                token(CssToken::Number(0.0)),
                token(CssToken::Comma),
                token(CssToken::Number(0.0)),
                token(CssToken::Comma),
                token(CssToken::Number(255.0)),
                token(CssToken::Comma),
                token(CssToken::Number(0.5)),
            ],
        }];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 127)))
        );
    }

    #[test]
    fn test_parse_multiple() {
        // 10px 20px
        let components = [
            token(CssToken::Dimension {
                value: 10.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 20.0,
                unit: "px".to_string(),
            }),
        ];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Multiple(vec![
                CssValue::Length(10.0, LengthUnit::Px),
                CssValue::Length(20.0, LengthUnit::Px),
            ]))
        );
    }

    #[test]
    fn test_unrecognized() {
        // Unknown function
        let components = [ComponentValue::Function {
            name: "unknown".to_string(),
            value: vec![],
        }];
        assert_eq!(parse_value(&components), None);
    }
}
