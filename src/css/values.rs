use super::CssToken;
use super::parser::ComponentValue;

#[derive(Debug, PartialEq, Clone)]
pub enum LengthUnit {
    Px,
    Em,
    Rem,
    Pt,
    Percent,
    Vw,
    Vh,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Color {
    Rgba(u8, u8, u8, u8),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PositionValue {
    Static,
    Relative,
    Absolute,
    Fixed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OverflowValue {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BoxSizingValue {
    ContentBox,
    BorderBox,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DisplayValue {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FlexDirectionValue {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JustifyContentValue {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AlignItemsValue {
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CssValue {
    Keyword(String),
    Length(f32, LengthUnit),
    Number(f32),
    Color(Color),
    Multiple(Vec<CssValue>),
    Position(PositionValue),
    Overflow(OverflowValue),
    BoxSizing(BoxSizingValue),
    Display(DisplayValue),
    FlexDirection(FlexDirectionValue),
    JustifyContent(JustifyContentValue),
    AlignItems(AlignItemsValue),
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

/// Helper to check if a property name is a layout-related property.
pub fn is_known_layout_property(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "position"
            | "overflow"
            | "box-sizing"
            | "display"
            | "flex-direction"
            | "justify-content"
            | "align-items"
    )
}

/// Validates that a CSS value is valid for a layout-related property.
pub fn is_valid_property_value(name: &str, value: &CssValue) -> bool {
    let name_lower = name.to_ascii_lowercase();
    match name_lower.as_str() {
        "position" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "static" | "relative" | "absolute" | "fixed"
                )
            }
            CssValue::Position(_) => true,
            _ => false,
        },
        "overflow" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "visible" | "hidden" | "scroll" | "auto"
                )
            }
            CssValue::Overflow(_) => true,
            _ => false,
        },
        "box-sizing" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "content-box" | "border-box"
                )
            }
            CssValue::BoxSizing(_) => true,
            _ => false,
        },
        "display" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "block" | "inline" | "inline-block" | "none" | "flex"
                )
            }
            CssValue::Display(_) => true,
            _ => false,
        },
        "flex-direction" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "row" | "row-reverse" | "column" | "column-reverse"
                )
            }
            CssValue::FlexDirection(_) => true,
            _ => false,
        },
        "justify-content" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "flex-start"
                        | "flex-end"
                        | "center"
                        | "space-between"
                        | "space-around"
                        | "space-evenly"
                )
            }
            CssValue::JustifyContent(_) => true,
            _ => false,
        },
        "align-items" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "flex-start" | "flex-end" | "center" | "baseline" | "stretch"
                )
            }
            CssValue::AlignItems(_) => true,
            _ => false,
        },
        _ => true,
    }
}

/// Parses a list of component values for a specific property, returning a typed CSS value if it matches a known layout property.
pub fn parse_property_value(
    property_name: &str,
    components: &[ComponentValue],
) -> Option<CssValue> {
    let val = parse_value(components)?;
    let name_lower = property_name.to_ascii_lowercase();
    match name_lower.as_str() {
        "position" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "static" => PositionValue::Static,
                    "relative" => PositionValue::Relative,
                    "absolute" => PositionValue::Absolute,
                    "fixed" => PositionValue::Fixed,
                    _ => return None,
                };
                Some(CssValue::Position(typed))
            } else {
                None
            }
        }
        "overflow" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "visible" => OverflowValue::Visible,
                    "hidden" => OverflowValue::Hidden,
                    "scroll" => OverflowValue::Scroll,
                    "auto" => OverflowValue::Auto,
                    _ => return None,
                };
                Some(CssValue::Overflow(typed))
            } else {
                None
            }
        }
        "box-sizing" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "content-box" => BoxSizingValue::ContentBox,
                    "border-box" => BoxSizingValue::BorderBox,
                    _ => return None,
                };
                Some(CssValue::BoxSizing(typed))
            } else {
                None
            }
        }
        "display" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "block" => DisplayValue::Block,
                    "inline" => DisplayValue::Inline,
                    "inline-block" => DisplayValue::InlineBlock,
                    "none" => DisplayValue::None,
                    "flex" => DisplayValue::Flex,
                    _ => return None,
                };
                Some(CssValue::Display(typed))
            } else {
                None
            }
        }
        "flex-direction" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "row" => FlexDirectionValue::Row,
                    "row-reverse" => FlexDirectionValue::RowReverse,
                    "column" => FlexDirectionValue::Column,
                    "column-reverse" => FlexDirectionValue::ColumnReverse,
                    _ => return None,
                };
                Some(CssValue::FlexDirection(typed))
            } else {
                None
            }
        }
        "justify-content" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "flex-start" => JustifyContentValue::FlexStart,
                    "flex-end" => JustifyContentValue::FlexEnd,
                    "center" => JustifyContentValue::Center,
                    "space-between" => JustifyContentValue::SpaceBetween,
                    "space-around" => JustifyContentValue::SpaceAround,
                    "space-evenly" => JustifyContentValue::SpaceEvenly,
                    _ => return None,
                };
                Some(CssValue::JustifyContent(typed))
            } else {
                None
            }
        }
        "align-items" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "stretch" => AlignItemsValue::Stretch,
                    "flex-start" => AlignItemsValue::FlexStart,
                    "flex-end" => AlignItemsValue::FlexEnd,
                    "center" => AlignItemsValue::Center,
                    "baseline" => AlignItemsValue::Baseline,
                    _ => return None,
                };
                Some(CssValue::AlignItems(typed))
            } else {
                None
            }
        }
        _ => Some(val),
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
                "vw" => LengthUnit::Vw,
                "vh" => LengthUnit::Vh,
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
    // A hex color is ASCII only; bail before any byte slicing so that a
    // non-ASCII `Hash` value (the tokenizer permits those) cannot panic on a
    // char boundary (I-6).
    if !s.is_ascii() {
        return None;
    }
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
    fn test_parse_color_hex_non_ascii_does_not_panic() {
        // A non-ASCII Hash must not panic on a char boundary (I-6); it is simply
        // not a valid hex color.
        let components = [token(CssToken::Hash("日本語".to_string()))];
        assert_eq!(parse_value(&components), None);
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

    #[test]
    fn test_parse_property_value_layout() {
        // Test position
        assert_eq!(
            parse_property_value("position", &[token(CssToken::Ident("static".to_string()))]),
            Some(CssValue::Position(PositionValue::Static))
        );
        assert_eq!(
            parse_property_value(
                "position",
                &[token(CssToken::Ident("relative".to_string()))]
            ),
            Some(CssValue::Position(PositionValue::Relative))
        );
        assert_eq!(
            parse_property_value(
                "position",
                &[token(CssToken::Ident("absolute".to_string()))]
            ),
            Some(CssValue::Position(PositionValue::Absolute))
        );
        assert_eq!(
            parse_property_value("position", &[token(CssToken::Ident("fixed".to_string()))]),
            Some(CssValue::Position(PositionValue::Fixed))
        );
        assert_eq!(
            parse_property_value(
                "position",
                &[token(CssToken::Ident("invalid-pos".to_string()))]
            ),
            None
        );

        // Test overflow
        assert_eq!(
            parse_property_value("overflow", &[token(CssToken::Ident("visible".to_string()))]),
            Some(CssValue::Overflow(OverflowValue::Visible))
        );
        assert_eq!(
            parse_property_value("overflow", &[token(CssToken::Ident("hidden".to_string()))]),
            Some(CssValue::Overflow(OverflowValue::Hidden))
        );
        assert_eq!(
            parse_property_value("overflow", &[token(CssToken::Ident("scroll".to_string()))]),
            Some(CssValue::Overflow(OverflowValue::Scroll))
        );
        assert_eq!(
            parse_property_value("overflow", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::Overflow(OverflowValue::Auto))
        );
        assert_eq!(
            parse_property_value(
                "overflow",
                &[token(CssToken::Ident("invalid-overflow".to_string()))]
            ),
            None
        );

        // Test box-sizing
        assert_eq!(
            parse_property_value(
                "box-sizing",
                &[token(CssToken::Ident("content-box".to_string()))]
            ),
            Some(CssValue::BoxSizing(BoxSizingValue::ContentBox))
        );
        assert_eq!(
            parse_property_value(
                "box-sizing",
                &[token(CssToken::Ident("border-box".to_string()))]
            ),
            Some(CssValue::BoxSizing(BoxSizingValue::BorderBox))
        );
        assert_eq!(
            parse_property_value(
                "box-sizing",
                &[token(CssToken::Ident("invalid-box-sizing".to_string()))]
            ),
            None
        );

        // Test display (with flex)
        assert_eq!(
            parse_property_value("display", &[token(CssToken::Ident("block".to_string()))]),
            Some(CssValue::Display(DisplayValue::Block))
        );
        assert_eq!(
            parse_property_value("display", &[token(CssToken::Ident("flex".to_string()))]),
            Some(CssValue::Display(DisplayValue::Flex))
        );
        assert_eq!(
            parse_property_value(
                "display",
                &[token(CssToken::Ident("invalid-display".to_string()))]
            ),
            None
        );

        // Test flex-direction
        assert_eq!(
            parse_property_value(
                "flex-direction",
                &[token(CssToken::Ident("row".to_string()))]
            ),
            Some(CssValue::FlexDirection(FlexDirectionValue::Row))
        );
        assert_eq!(
            parse_property_value(
                "flex-direction",
                &[token(CssToken::Ident("column-reverse".to_string()))]
            ),
            Some(CssValue::FlexDirection(FlexDirectionValue::ColumnReverse))
        );
        assert_eq!(
            parse_property_value(
                "flex-direction",
                &[token(CssToken::Ident("invalid-dir".to_string()))]
            ),
            None
        );

        // Test justify-content
        assert_eq!(
            parse_property_value(
                "justify-content",
                &[token(CssToken::Ident("flex-start".to_string()))]
            ),
            Some(CssValue::JustifyContent(JustifyContentValue::FlexStart))
        );
        assert_eq!(
            parse_property_value(
                "justify-content",
                &[token(CssToken::Ident("space-between".to_string()))]
            ),
            Some(CssValue::JustifyContent(JustifyContentValue::SpaceBetween))
        );
        assert_eq!(
            parse_property_value(
                "justify-content",
                &[token(CssToken::Ident("invalid-justify".to_string()))]
            ),
            None
        );

        // Test align-items
        assert_eq!(
            parse_property_value(
                "align-items",
                &[token(CssToken::Ident("stretch".to_string()))]
            ),
            Some(CssValue::AlignItems(AlignItemsValue::Stretch))
        );
        assert_eq!(
            parse_property_value(
                "align-items",
                &[token(CssToken::Ident("baseline".to_string()))]
            ),
            Some(CssValue::AlignItems(AlignItemsValue::Baseline))
        );
        assert_eq!(
            parse_property_value(
                "align-items",
                &[token(CssToken::Ident("invalid-align".to_string()))]
            ),
            None
        );

        // Test non-layout properties
        assert_eq!(
            parse_property_value("color", &[token(CssToken::Ident("red".to_string()))]),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
    }
}
