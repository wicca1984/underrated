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
    Sticky,
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
    Table,
    TableRow,
    TableCell,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WhiteSpaceValue {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
}

impl WhiteSpaceValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Nowrap => "nowrap",
            Self::Pre => "pre",
            Self::PreWrap => "pre-wrap",
            Self::PreLine => "pre-line",
        }
    }
}

impl std::str::FromStr for WhiteSpaceValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Ok(Self::Normal),
            "nowrap" => Ok(Self::Nowrap),
            "pre" => Ok(Self::Pre),
            "pre-wrap" => Ok(Self::PreWrap),
            "pre-line" => Ok(Self::PreLine),
            _ => Err(()),
        }
    }
}

impl TryFrom<&CssValue> for WhiteSpaceValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LengthOrPercent {
    pub value: f32,
    pub unit: LengthUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AngleDeg(pub f32);

#[derive(Debug, Clone, PartialEq)]
pub enum TransformFn {
    Translate {
        x: LengthOrPercent,
        y: LengthOrPercent,
    },
    TranslateX(LengthOrPercent),
    TranslateY(LengthOrPercent),
    Scale {
        x: f32,
        y: f32,
    },
    ScaleX(f32),
    ScaleY(f32),
    Rotate(AngleDeg),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZIndex {
    #[default]
    Auto,
    Index(i32),
}

impl ZIndex {
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        if trimmed.eq_ignore_ascii_case("auto") {
            ZIndex::Auto
        } else if let Ok(val) = trimmed.parse::<i32>() {
            ZIndex::Index(val)
        } else {
            ZIndex::Auto
        }
    }
}

impl std::str::FromStr for ZIndex {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Opacity(pub f32);

impl Default for Opacity {
    fn default() -> Self {
        Opacity(1.0)
    }
}

impl Opacity {
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Opacity(1.0);
        }

        // TODO(spec): Percentage support is optional; we support bare <number> here.
        if let Ok(val) = trimmed.parse::<f32>() {
            return Opacity(if val.is_finite() {
                val.clamp(0.0, 1.0)
            } else {
                1.0
            });
        }
        Opacity(1.0)
    }
}

impl std::str::FromStr for Opacity {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
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
    Transform(Vec<TransformFn>),
    ZIndex(ZIndex),
    Opacity(f32),
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
            // A `/` delimiter separates values even without surrounding
            // whitespace (e.g. `aspect-ratio:16/9`), so flush the current
            // group and emit the slash as its own keyword. This keeps the
            // tight `16/9` form consistent with the spaced `16 / 9` form.
            ComponentValue::Token(CssToken::Delim('/')) => {
                if !current_group.is_empty() {
                    if let Some(val) = parse_single_value(&current_group) {
                        values.push(val);
                        current_group.clear();
                    } else {
                        return None;
                    }
                }
                values.push(CssValue::Keyword("/".to_string()));
            }
            // A comma (`,`) separates values in multi-value declarations (e.g. box-shadow,
            // transition, font-family, gradients). Flush the current group and emit the
            // comma as its own keyword so downstream consumers can split on it.
            ComponentValue::Token(CssToken::Comma) => {
                if !current_group.is_empty() {
                    if let Some(val) = parse_single_value(&current_group) {
                        values.push(val);
                        current_group.clear();
                    } else {
                        return None;
                    }
                }
                values.push(CssValue::Keyword(",".to_string()));
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
            | "overflow-x"
            | "overflow-y"
            | "box-sizing"
            | "display"
            | "flex-direction"
            | "justify-content"
            | "align-items"
            | "white-space"
            | "text-overflow"
            | "flex-wrap"
            | "float"
            | "clear"
            | "table-layout"
            | "scroll-behavior"
            | "overscroll-behavior"
            | "overscroll-behavior-x"
            | "overscroll-behavior-y"
            | "user-select"
            | "visibility"
            | "direction"
            | "cursor"
            | "accent-color"
            | "caret-color"
            | "transition-timing-function"
            | "transition-delay"
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
                    "static" | "relative" | "absolute" | "fixed" | "sticky"
                )
            }
            CssValue::Position(_) => true,
            _ => false,
        },
        "overflow" | "overflow-x" | "overflow-y" => match value {
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
        "white-space" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal" | "nowrap" | "pre" | "pre-wrap" | "pre-line" | "initial" | "inherit"
                )
            }
            _ => false,
        },
        // TODO(spec): layout-time truncation/ellipsis rendering is out of scope (a separate future task)
        "text-overflow" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "clip" | "ellipsis" | "initial" | "inherit"
                )
            }
            _ => false,
        },
        "flex-wrap" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "nowrap" | "wrap" | "wrap-reverse"
                )
            }
            _ => false,
        },
        "float" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "none" | "left" | "right")
            }
            _ => false,
        },
        "clear" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "none" | "left" | "right" | "both"
                )
            }
            _ => false,
        },
        "table-layout" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "fixed")
            }
            _ => false,
        },
        "scroll-behavior" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "smooth")
            }
            _ => false,
        },
        "overscroll-behavior" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "contain" | "none"
                )
            }
            CssValue::Multiple(vals) if vals.len() == 2 => vals.iter().all(|val| match val {
                CssValue::Keyword(kw) => {
                    matches!(
                        kw.to_ascii_lowercase().as_str(),
                        "auto" | "contain" | "none"
                    )
                }
                _ => false,
            }),
            _ => false,
        },
        "overscroll-behavior-x" | "overscroll-behavior-y" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "contain" | "none"
                )
            }
            _ => false,
        },
        "user-select" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "text" | "none" | "contain" | "all"
                )
            }
            _ => false,
        },
        "visibility" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "visible" | "hidden" | "collapse"
                )
            }
            _ => false,
        },
        "direction" => match value {
            CssValue::Keyword(kw) => matches!(kw.to_ascii_lowercase().as_str(), "ltr" | "rtl"),
            _ => false,
        },
        // TODO(spec): Custom cursor images using `url(...)` and comma-separated fallback lists are currently out of scope.
        "cursor" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto"
                        | "default"
                        | "none"
                        | "context-menu"
                        | "help"
                        | "pointer"
                        | "progress"
                        | "wait"
                        | "cell"
                        | "crosshair"
                        | "text"
                        | "vertical-text"
                        | "alias"
                        | "copy"
                        | "move"
                        | "no-drop"
                        | "not-allowed"
                        | "grab"
                        | "grabbing"
                        | "e-resize"
                        | "n-resize"
                        | "ne-resize"
                        | "nw-resize"
                        | "s-resize"
                        | "se-resize"
                        | "sw-resize"
                        | "w-resize"
                        | "ew-resize"
                        | "ns-resize"
                        | "nesw-resize"
                        | "nwse-resize"
                        | "col-resize"
                        | "row-resize"
                        | "all-scroll"
                        | "zoom-in"
                        | "zoom-out"
                )
            }
            _ => false,
        },
        "accent-color" | "caret-color" => match value {
            CssValue::Color(_) => true,
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "currentcolor")
            }
            _ => false,
        },
        "transition-timing-function" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "ease"
                        | "linear"
                        | "ease-in"
                        | "ease-out"
                        | "ease-in-out"
                        | "step-start"
                        | "step-end"
                )
            }
            _ => false,
        },
        "transition-delay" => match value {
            CssValue::Keyword(kw) => {
                let kw_lower = kw.to_ascii_lowercase();
                if kw_lower.ends_with("ms") {
                    kw_lower[..kw_lower.len() - 2].parse::<f32>().is_ok()
                } else if kw_lower.ends_with('s') {
                    kw_lower[..kw_lower.len() - 1].parse::<f32>().is_ok()
                } else {
                    false
                }
            }
            CssValue::Number(v) => *v == 0.0,
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
                    "sticky" => PositionValue::Sticky,
                    _ => return None,
                };
                Some(CssValue::Position(typed))
            } else {
                None
            }
        }
        // TODO(spec): expand two-value overflow shorthand into overflow-x/overflow-y in style::expand
        "overflow" | "overflow-x" | "overflow-y" => {
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
                    "table" => DisplayValue::Table,
                    "table-row" => DisplayValue::TableRow,
                    "table-cell" => DisplayValue::TableCell,
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
        "white-space" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "normal" | "nowrap" | "pre" | "pre-wrap" | "pre-line" | "initial"
                    | "inherit" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        // TODO(spec): layout-time truncation/ellipsis rendering is out of scope (a separate future task)
        "text-overflow" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "clip" | "ellipsis" | "initial" | "inherit" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "flex-wrap" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "nowrap" | "wrap" | "wrap-reverse" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "float" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "none" | "left" | "right" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "clear" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "none" | "left" | "right" | "both" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "table-layout" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "fixed" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "scroll-behavior" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "smooth" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "overscroll-behavior" => {
            // TODO(spec): expand overscroll-behavior shorthand in style resolver if needed
            match &val {
                CssValue::Keyword(kw) => match kw.to_ascii_lowercase().as_str() {
                    "auto" | "contain" | "none" => Some(val),
                    _ => None,
                },
                CssValue::Multiple(vals) => {
                    if vals.len() == 2 {
                        let is_valid = vals.iter().all(|v| match v {
                            CssValue::Keyword(kw) => {
                                matches!(
                                    kw.to_ascii_lowercase().as_str(),
                                    "auto" | "contain" | "none"
                                )
                            }
                            _ => false,
                        });
                        if is_valid { Some(val) } else { None }
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        "overscroll-behavior-x" | "overscroll-behavior-y" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "contain" | "none" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "user-select" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "text" | "none" | "contain" | "all" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "visibility" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "visible" | "hidden" | "collapse" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "direction" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "ltr" | "rtl" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        // TODO(spec): Custom cursor images using `url(...)` and comma-separated fallback lists are currently out of scope.
        "cursor" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "default" | "none" | "context-menu" | "help" | "pointer"
                    | "progress" | "wait" | "cell" | "crosshair" | "text" | "vertical-text"
                    | "alias" | "copy" | "move" | "no-drop" | "not-allowed" | "grab"
                    | "grabbing" | "e-resize" | "n-resize" | "ne-resize" | "nw-resize"
                    | "s-resize" | "se-resize" | "sw-resize" | "w-resize" | "ew-resize"
                    | "ns-resize" | "nesw-resize" | "nwse-resize" | "col-resize" | "row-resize"
                    | "all-scroll" | "zoom-in" | "zoom-out" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "accent-color" | "caret-color" => match &val {
            CssValue::Color(_) => Some(val),
            CssValue::Keyword(kw) => match kw.to_ascii_lowercase().as_str() {
                "auto" | "currentcolor" => Some(val),
                _ => None,
            },
            _ => None,
        },
        "transition-timing-function" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out" | "step-start"
                    | "step-end" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "transition-delay" => match &val {
            CssValue::Keyword(kw) => {
                let kw_lower = kw.to_ascii_lowercase();
                if kw_lower.ends_with("ms") {
                    if kw_lower[..kw_lower.len() - 2].parse::<f32>().is_ok() {
                        Some(val)
                    } else {
                        None
                    }
                } else if kw_lower.ends_with('s') {
                    if kw_lower[..kw_lower.len() - 1].parse::<f32>().is_ok() {
                        Some(val)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            CssValue::Number(v) => {
                if *v == 0.0 {
                    Some(val)
                } else {
                    None
                }
            }
            _ => None,
        },
        "object-position" => {
            // // TODO(spec): full position resolution for object-position grammar.
            Some(val)
        }
        _ => Some(val),
    }
}

fn serialize_component_value(cv: &ComponentValue) -> String {
    match cv {
        ComponentValue::Token(token) => match token {
            CssToken::Ident(s) => s.clone(),
            CssToken::Function(s) => format!("{}(", s),
            CssToken::AtKeyword(s) => format!("@{}", s),
            CssToken::Hash(s) => format!("#{}", s),
            CssToken::String(s) => format!("\"{}\"", s),
            CssToken::Number(v) => v.to_string(),
            CssToken::Percentage(v) => format!("{}%", v),
            CssToken::Dimension { value, unit } => format!("{}{}", value, unit),
            CssToken::Delim(c) => c.to_string(),
            CssToken::Whitespace => " ".to_string(),
            CssToken::Colon => ":".to_string(),
            CssToken::Semicolon => ";".to_string(),
            CssToken::Comma => ",".to_string(),
            CssToken::LeftBrace => "{".to_string(),
            CssToken::RightBrace => "}".to_string(),
            CssToken::LeftParen => "(".to_string(),
            CssToken::RightParen => ")".to_string(),
            CssToken::LeftBracket => "[".to_string(),
            CssToken::RightBracket => "]".to_string(),
            CssToken::Cdo => "<!--".to_string(),
            CssToken::Cdc => "-->".to_string(),
            CssToken::BadString => "".to_string(),
            CssToken::BadUrl => "".to_string(),
            CssToken::Url(s) => format!("url({})", s),
            CssToken::Eof => "".to_string(),
        },
        ComponentValue::Function { name, value } => {
            let mut s = format!("{}(", name);
            for val in value {
                s.push_str(&serialize_component_value(val));
            }
            s.push(')');
            s
        }
        ComponentValue::SimpleBlock { associated, value } => {
            let (open, close) = match associated {
                '{' => ("{", "}"),
                '[' => ("[", "]"),
                '(' => ("(", ")"),
                _ => ("", ""),
            };
            let mut s = open.to_string();
            for val in value {
                s.push_str(&serialize_component_value(val));
            }
            s.push_str(close);
            s
        }
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
            let lower_unit = unit.to_ascii_lowercase();
            if lower_unit == "s" || lower_unit == "ms" {
                return Some(CssValue::Keyword(format!("{}{}", value, lower_unit)));
            }
            let unit_enum = match lower_unit.as_str() {
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
        ComponentValue::Token(CssToken::Delim('/')) => Some(CssValue::Keyword("/".to_string())),
        ComponentValue::Token(CssToken::Url(s)) => Some(CssValue::Keyword(format!("url({})", s))),
        ComponentValue::Function { name, value } => {
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") {
                return parse_rgb_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") {
                return parse_hsl_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("url") {
                let mut url_str = None;
                for val in value {
                    match val {
                        ComponentValue::Token(CssToken::String(s)) => {
                            url_str = Some(s.clone());
                            break;
                        }
                        ComponentValue::Token(CssToken::Ident(s)) => {
                            url_str = Some(s.clone());
                            break;
                        }
                        _ => {}
                    }
                }
                return url_str.map(|s| CssValue::Keyword(format!("url({})", s)));
            }
            if name.eq_ignore_ascii_case("linear-gradient")
                || name.eq_ignore_ascii_case("radial-gradient")
            {
                return Some(CssValue::Keyword(serialize_component_value(components[0])));
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

fn parse_hsl_function(components: &[ComponentValue]) -> Option<Color> {
    enum HslArg {
        Number(f64),
        Percentage(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(HslArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(HslArg::Percentage(*v)),
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // Parse Hue
    let h_val = match args[0] {
        HslArg::Number(v) => v,
        _ => return None,
    };
    let h = ((h_val % 360.0) + 360.0) % 360.0;

    // Parse Saturation
    let s_val = match args[1] {
        HslArg::Percentage(v) => v,
        _ => return None,
    };
    let s = (s_val / 100.0).clamp(0.0, 1.0);

    // Parse Lightness
    let l_val = match args[2] {
        HslArg::Percentage(v) => v,
        _ => return None,
    };
    let l = (l_val / 100.0).clamp(0.0, 1.0);

    // Parse Alpha
    let alpha = if args.len() == 4 {
        let a_val = match args[3] {
            HslArg::Number(v) => v,
            HslArg::Percentage(v) => v / 100.0,
        };
        (a_val.clamp(0.0, 1.0) * 255.0) as u8
    } else {
        255
    };

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hp as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x), // covers hp in [5,6)
    };
    let m = l - c / 2.0;
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;

    Some(Color::Rgba(r, g, b, alpha))
}

fn parse_args(components: &[ComponentValue]) -> Option<Vec<&ComponentValue>> {
    let mut args = Vec::new();
    let mut expect_comma = false;

    for comp in components {
        match comp {
            ComponentValue::Token(CssToken::Whitespace) => {
                continue;
            }
            ComponentValue::Token(CssToken::Comma) => {
                if !expect_comma {
                    return None;
                }
                expect_comma = false;
            }
            other => {
                if expect_comma {
                    return None;
                }
                args.push(other);
                expect_comma = true;
            }
        }
    }

    if !expect_comma && !components.is_empty() && !args.is_empty() {
        return None;
    }

    Some(args)
}

fn parse_length_or_percent(comp: &ComponentValue) -> Option<LengthOrPercent> {
    match comp {
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let unit_enum = match unit.to_ascii_lowercase().as_str() {
                "px" => LengthUnit::Px,
                "em" => LengthUnit::Em,
                "rem" => LengthUnit::Rem,
                "pt" => LengthUnit::Pt,
                "vw" => LengthUnit::Vw,
                "vh" => LengthUnit::Vh,
                _ => return None,
            };
            Some(LengthOrPercent {
                value: *value as f32,
                unit: unit_enum,
            })
        }
        ComponentValue::Token(CssToken::Percentage(v)) => Some(LengthOrPercent {
            value: *v as f32,
            unit: LengthUnit::Percent,
        }),
        ComponentValue::Token(CssToken::Number(v)) => {
            if *v == 0.0 {
                Some(LengthOrPercent {
                    value: 0.0,
                    unit: LengthUnit::Px,
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_number(comp: &ComponentValue) -> Option<f32> {
    match comp {
        ComponentValue::Token(CssToken::Number(v)) => Some(*v as f32),
        _ => None,
    }
}

fn parse_angle(comp: &ComponentValue) -> Option<AngleDeg> {
    match comp {
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let deg = match unit.to_ascii_lowercase().as_str() {
                "deg" => *value as f32,
                "rad" => (*value as f32) * 180.0 / std::f32::consts::PI,
                "grad" => (*value as f32) * 0.9,
                "turn" => (*value as f32) * 360.0,
                _ => return None,
            };
            Some(AngleDeg(deg))
        }
        ComponentValue::Token(CssToken::Number(v)) => {
            if *v == 0.0 {
                Some(AngleDeg(0.0))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_transform_function(name: &str, value: &[ComponentValue]) -> Option<TransformFn> {
    let args = parse_args(value)?;
    match name.to_ascii_lowercase().as_str() {
        "translate" => {
            if args.len() == 1 {
                let x = parse_length_or_percent(args[0])?;
                let y = LengthOrPercent {
                    value: 0.0,
                    unit: LengthUnit::Px,
                };
                Some(TransformFn::Translate { x, y })
            } else if args.len() == 2 {
                let x = parse_length_or_percent(args[0])?;
                let y = parse_length_or_percent(args[1])?;
                Some(TransformFn::Translate { x, y })
            } else {
                None
            }
        }
        "translatex" => {
            if args.len() == 1 {
                let x = parse_length_or_percent(args[0])?;
                Some(TransformFn::TranslateX(x))
            } else {
                None
            }
        }
        "translatey" => {
            if args.len() == 1 {
                let y = parse_length_or_percent(args[0])?;
                Some(TransformFn::TranslateY(y))
            } else {
                None
            }
        }
        "scale" => {
            if args.len() == 1 {
                let s = parse_number(args[0])?;
                Some(TransformFn::Scale { x: s, y: s })
            } else if args.len() == 2 {
                let x = parse_number(args[0])?;
                let y = parse_number(args[1])?;
                Some(TransformFn::Scale { x, y })
            } else {
                None
            }
        }
        "scalex" => {
            if args.len() == 1 {
                let x = parse_number(args[0])?;
                Some(TransformFn::ScaleX(x))
            } else {
                None
            }
        }
        "scaley" => {
            if args.len() == 1 {
                let y = parse_number(args[0])?;
                Some(TransformFn::ScaleY(y))
            } else {
                None
            }
        }
        "rotate" => {
            if args.len() == 1 {
                let angle = parse_angle(args[0])?;
                Some(TransformFn::Rotate(angle))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn parse_transform(components: &[ComponentValue]) -> Option<CssValue> {
    let mut fns = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Token(CssToken::Whitespace) => {
                continue;
            }
            ComponentValue::Function { name, value } => {
                if let Some(tf) = parse_transform_function(name, value) {
                    fns.push(tf);
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }

    if fns.is_empty() {
        None
    } else {
        Some(CssValue::Transform(fns))
    }
}

pub fn parse_z_index(components: &[ComponentValue]) -> Option<CssValue> {
    // Trim leading and trailing whitespace
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

    match &trimmed[0] {
        ComponentValue::Token(CssToken::Ident(s)) => {
            if s.eq_ignore_ascii_case("auto") {
                Some(CssValue::ZIndex(ZIndex::Auto))
            } else {
                None
            }
        }
        ComponentValue::Token(CssToken::Number(val)) => {
            if val.fract() == 0.0 && *val >= i32::MIN as f64 && *val <= i32::MAX as f64 {
                Some(CssValue::ZIndex(ZIndex::Index(*val as i32)))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn parse_opacity(value: &str) -> Option<CssValue> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let is_percentage = trimmed.ends_with('%');
    let parsed_val = if is_percentage {
        let num_part = &trimmed[..trimmed.len() - 1];
        if num_part.is_empty() {
            return None;
        }
        let val: f32 = num_part.parse::<f32>().ok()?;
        val / 100.0
    } else {
        trimmed.parse::<f32>().ok()?
    };

    if !parsed_val.is_finite() {
        return None;
    }

    let clamped = parsed_val.clamp(0.0, 1.0);

    Some(CssValue::Opacity(clamped))
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
    fn test_parse_color_hsl() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // hsl(0, 100%, 50%) -> pure red
        assert_eq!(
            parse("hsl(0, 100%, 50%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // HSL(0, 100%, 50%) -> case-insensitivity
        assert_eq!(
            parse("HSL(0, 100%, 50%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // hsl(120, 100%, 50%) -> green
        assert_eq!(
            parse("hsl(120, 100%, 50%)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // hsl(240, 100%, 50%) -> blue
        assert_eq!(
            parse("hsl(240, 100%, 50%)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );

        // hsl(0, 0%, 100%) -> white
        assert_eq!(
            parse("hsl(0, 0%, 100%)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // hsl(0, 0%, 0%) -> black
        assert_eq!(
            parse("hsl(0, 0%, 0%)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // hsla(0, 100%, 50%, 0.5) -> alpha within 1 of 127
        let alpha_color = parse("hsla(0, 100%, 50%, 0.5)");
        match alpha_color {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 127).abs() <= 1);
            }
            _ => panic!("Expected hsla(0, 100%, 50%, 0.5) to parse as a color"),
        }

        // Percentage alpha: hsla(0, 100%, 50%, 50%) -> alpha within 1 of 127
        let alpha_pct = parse("hsla(0, 100%, 50%, 50%)");
        match alpha_pct {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 127).abs() <= 1);
            }
            _ => panic!("Expected hsla(0, 100%, 50%, 50%) to parse as a color"),
        }

        // Negative hues wrapping: hsl(-240, 100%, 50%) wraps to 120 (green)
        assert_eq!(
            parse("hsl(-240, 100%, 50%)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // Saturation/Lightness clamping: hsl(0, 150%, 50%) -> red
        assert_eq!(
            parse("hsl(0, 150%, 50%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // Rejecting bare numbers for S/L: hsl(0, 100, 50) -> None
        assert_eq!(parse("hsl(0, 100, 50)"), None);

        // Rejecting invalid arguments count: hsl(0, 100%) -> None
        assert_eq!(parse("hsl(0, 100%)"), None);
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
    fn test_parse_multiple_with_slash() {
        // 16 / 9
        let components = [
            token(CssToken::Number(16.0)),
            token(CssToken::Whitespace),
            token(CssToken::Delim('/')),
            token(CssToken::Whitespace),
            token(CssToken::Number(9.0)),
        ];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Multiple(vec![
                CssValue::Number(16.0),
                CssValue::Keyword("/".to_string()),
                CssValue::Number(9.0),
            ]))
        );
    }

    #[test]
    fn test_parse_multiple_with_slash_no_whitespace() {
        // 16/9 (tight form, no surrounding whitespace) must parse the same
        // as the spaced `16 / 9` form so aspect-ratio reaches layout.
        let components = [
            token(CssToken::Number(16.0)),
            token(CssToken::Delim('/')),
            token(CssToken::Number(9.0)),
        ];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Multiple(vec![
                CssValue::Number(16.0),
                CssValue::Keyword("/".to_string()),
                CssValue::Number(9.0),
            ]))
        );
    }

    #[test]
    fn test_parse_multiple_with_comma_shadows() {
        // 5px 5px red , 10px 10px blue
        let components = [
            token(CssToken::Dimension {
                value: 5.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 5.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Ident("red".to_string())),
            token(CssToken::Whitespace),
            token(CssToken::Comma),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 10.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 10.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Ident("blue".to_string())),
        ];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Multiple(vec![
                CssValue::Length(5.0, LengthUnit::Px),
                CssValue::Length(5.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(255, 0, 0, 255)),
                CssValue::Keyword(",".to_string()),
                CssValue::Length(10.0, LengthUnit::Px),
                CssValue::Length(10.0, LengthUnit::Px),
                CssValue::Color(Color::Rgba(0, 0, 255, 255)),
            ]))
        );
    }

    #[test]
    fn test_parse_multiple_with_comma_simple() {
        // red , blue
        let components = [
            token(CssToken::Ident("red".to_string())),
            token(CssToken::Whitespace),
            token(CssToken::Comma),
            token(CssToken::Whitespace),
            token(CssToken::Ident("blue".to_string())),
        ];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Multiple(vec![
                CssValue::Color(Color::Rgba(255, 0, 0, 255)),
                CssValue::Keyword(",".to_string()),
                CssValue::Color(Color::Rgba(0, 0, 255, 255)),
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
            parse_property_value("position", &[token(CssToken::Ident("sticky".to_string()))]),
            Some(CssValue::Position(PositionValue::Sticky))
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

        // Test overflow-x and overflow-y
        assert!(is_known_layout_property("overflow-x"));
        assert!(is_known_layout_property("overflow-y"));

        assert!(is_valid_property_value(
            "position",
            &CssValue::Position(PositionValue::Sticky)
        ));
        assert!(is_valid_property_value(
            "position",
            &CssValue::Keyword("sticky".to_string())
        ));
        assert!(!is_valid_property_value(
            "position",
            &CssValue::Keyword("invalid-pos".to_string())
        ));

        assert!(is_valid_property_value(
            "overflow-x",
            &CssValue::Overflow(OverflowValue::Hidden)
        ));
        assert!(is_valid_property_value(
            "overflow-y",
            &CssValue::Overflow(OverflowValue::Scroll)
        ));
        assert!(!is_valid_property_value(
            "overflow-x",
            &CssValue::Keyword("banana".to_string())
        ));

        assert_eq!(
            parse_property_value(
                "overflow-x",
                &[token(CssToken::Ident("hidden".to_string()))]
            ),
            Some(CssValue::Overflow(OverflowValue::Hidden))
        );
        assert_eq!(
            parse_property_value(
                "overflow-y",
                &[token(CssToken::Ident("scroll".to_string()))]
            ),
            Some(CssValue::Overflow(OverflowValue::Scroll))
        );
        assert_eq!(
            parse_property_value("overflow-x", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::Overflow(OverflowValue::Auto))
        );
        assert_eq!(
            parse_property_value(
                "overflow-y",
                &[token(CssToken::Ident("visible".to_string()))]
            ),
            Some(CssValue::Overflow(OverflowValue::Visible))
        );
        assert_eq!(
            parse_property_value(
                "overflow-x",
                &[token(CssToken::Ident("banana".to_string()))]
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

        // Test white-space keywords
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("normal".to_string()))]
            ),
            Some(CssValue::Keyword("normal".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("nowrap".to_string()))]
            ),
            Some(CssValue::Keyword("nowrap".to_string()))
        );
        assert_eq!(
            parse_property_value("white-space", &[token(CssToken::Ident("pre".to_string()))]),
            Some(CssValue::Keyword("pre".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("pre-wrap".to_string()))]
            ),
            Some(CssValue::Keyword("pre-wrap".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("pre-line".to_string()))]
            ),
            Some(CssValue::Keyword("pre-line".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("initial".to_string()))]
            ),
            Some(CssValue::Keyword("initial".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("inherit".to_string()))]
            ),
            Some(CssValue::Keyword("inherit".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "white-space",
                &[token(CssToken::Ident("bogus".to_string()))]
            ),
            None
        );

        // Test WhiteSpaceValue parsing and conversion
        assert_eq!(
            "pre-wrap".parse::<WhiteSpaceValue>(),
            Ok(WhiteSpaceValue::PreWrap)
        );
        assert_eq!("BOGUS".parse::<WhiteSpaceValue>(), Err(()));
        assert_eq!(
            WhiteSpaceValue::try_from(&CssValue::Keyword("nowrap".to_string())),
            Ok(WhiteSpaceValue::Nowrap)
        );
        assert_eq!(WhiteSpaceValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test non-layout properties
        assert_eq!(
            parse_property_value("color", &[token(CssToken::Ident("red".to_string()))]),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
    }

    #[test]
    fn test_parse_transform() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_transform(&components)
        };

        // 1. translate(10px, 20px)
        let val1 = parse("translate(10px, 20px)").unwrap();
        assert_eq!(
            val1,
            CssValue::Transform(vec![TransformFn::Translate {
                x: LengthOrPercent {
                    value: 10.0,
                    unit: LengthUnit::Px
                },
                y: LengthOrPercent {
                    value: 20.0,
                    unit: LengthUnit::Px
                },
            }])
        );

        // 2. translate(10px) -> y defaults to 0
        let val2 = parse("translate(10px)").unwrap();
        assert_eq!(
            val2,
            CssValue::Transform(vec![TransformFn::Translate {
                x: LengthOrPercent {
                    value: 10.0,
                    unit: LengthUnit::Px
                },
                y: LengthOrPercent {
                    value: 0.0,
                    unit: LengthUnit::Px
                },
            }])
        );

        // 3. translateX(5px) and translateY(50%)
        let val3 = parse("translateX(5px)").unwrap();
        assert_eq!(
            val3,
            CssValue::Transform(vec![TransformFn::TranslateX(LengthOrPercent {
                value: 5.0,
                unit: LengthUnit::Px,
            })])
        );

        let val4 = parse("translateY(50%)").unwrap();
        assert_eq!(
            val4,
            CssValue::Transform(vec![TransformFn::TranslateY(LengthOrPercent {
                value: 50.0,
                unit: LengthUnit::Percent,
            })])
        );

        // 4. scale(2) -> uniform x=y=2
        let val5 = parse("scale(2)").unwrap();
        assert_eq!(
            val5,
            CssValue::Transform(vec![TransformFn::Scale { x: 2.0, y: 2.0 }])
        );

        // scale(2, 3) -> x=2, y=3
        let val6 = parse("scale(2, 3)").unwrap();
        assert_eq!(
            val6,
            CssValue::Transform(vec![TransformFn::Scale { x: 2.0, y: 3.0 }])
        );

        // scaleX(0.5)
        let val7 = parse("scaleX(0.5)").unwrap();
        assert_eq!(val7, CssValue::Transform(vec![TransformFn::ScaleX(0.5)]));

        // 5. rotate(45deg), rotate(0) (unitless zero), and rotate(1turn), rotate(100grad), rotate(3.14159265rad)
        let val8 = parse("rotate(45deg)").unwrap();
        if let CssValue::Transform(ref fns) = val8 {
            if let TransformFn::Rotate(AngleDeg(deg)) = fns[0] {
                assert_eq!(deg, 45.0);
            } else {
                panic!("Expected Rotate");
            }
        } else {
            panic!("Expected Transform");
        }

        let val9 = parse("rotate(0)").unwrap();
        if let CssValue::Transform(ref fns) = val9 {
            if let TransformFn::Rotate(AngleDeg(deg)) = fns[0] {
                assert_eq!(deg, 0.0);
            } else {
                panic!("Expected Rotate");
            }
        } else {
            panic!("Expected Transform");
        }

        let val10 = parse("rotate(1turn)").unwrap();
        if let CssValue::Transform(ref fns) = val10 {
            if let TransformFn::Rotate(AngleDeg(deg)) = fns[0] {
                assert_eq!(deg, 360.0);
            } else {
                panic!("Expected Rotate");
            }
        } else {
            panic!("Expected Transform");
        }

        let val11 = parse("rotate(100grad)").unwrap();
        if let CssValue::Transform(ref fns) = val11 {
            if let TransformFn::Rotate(AngleDeg(deg)) = fns[0] {
                assert_eq!(deg, 90.0);
            } else {
                panic!("Expected Rotate");
            }
        } else {
            panic!("Expected Transform");
        }

        let val12 = parse("rotate(3.141592653589793rad)").unwrap();
        if let CssValue::Transform(ref fns) = val12 {
            if let TransformFn::Rotate(AngleDeg(deg)) = fns[0] {
                assert!((deg - 180.0).abs() < 1e-4);
            } else {
                panic!("Expected Rotate");
            }
        } else {
            panic!("Expected Transform");
        }

        // 6. A chained list: translate(1px, 2px) rotate(45deg) scale(2)
        let val13 = parse("translate(1px, 2px) rotate(45deg) scale(2)").unwrap();
        assert_eq!(
            val13,
            CssValue::Transform(vec![
                TransformFn::Translate {
                    x: LengthOrPercent {
                        value: 1.0,
                        unit: LengthUnit::Px
                    },
                    y: LengthOrPercent {
                        value: 2.0,
                        unit: LengthUnit::Px
                    },
                },
                TransformFn::Rotate(AngleDeg(45.0)),
                TransformFn::Scale { x: 2.0, y: 2.0 },
            ])
        );

        // 7. Invalid inputs return None
        assert!(parse("skew(10deg)").is_none());
        assert!(parse("translate(1px, 2px, 3px)").is_none());
        assert!(parse("scale(10px)").is_none());
        assert!(parse("rotate(45)").is_none()); // unitless non-zero angle is invalid
        assert!(parse("translate(10)").is_none()); // unitless non-zero length is invalid
    }

    #[test]
    fn test_parse_z_index() {
        // Valid auto and case-insensitive
        assert_eq!(
            parse_z_index(&[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::ZIndex(ZIndex::Auto))
        );
        assert_eq!(
            parse_z_index(&[token(CssToken::Ident("AUTO".to_string()))]),
            Some(CssValue::ZIndex(ZIndex::Auto))
        );
        assert_eq!(
            parse_z_index(&[token(CssToken::Ident("aUtO".to_string()))]),
            Some(CssValue::ZIndex(ZIndex::Auto))
        );

        // Valid integers
        assert_eq!(
            parse_z_index(&[token(CssToken::Number(0.0))]),
            Some(CssValue::ZIndex(ZIndex::Index(0)))
        );
        assert_eq!(
            parse_z_index(&[token(CssToken::Number(5.0))]),
            Some(CssValue::ZIndex(ZIndex::Index(5)))
        );
        assert_eq!(
            parse_z_index(&[token(CssToken::Number(-1.0))]),
            Some(CssValue::ZIndex(ZIndex::Index(-1)))
        );
        assert_eq!(
            parse_z_index(&[token(CssToken::Number(123456.0))]),
            Some(CssValue::ZIndex(ZIndex::Index(123456)))
        );

        // Whitespace handling
        assert_eq!(
            parse_z_index(&[
                token(CssToken::Whitespace),
                token(CssToken::Ident("auto".to_string())),
                token(CssToken::Whitespace)
            ]),
            Some(CssValue::ZIndex(ZIndex::Auto))
        );
        assert_eq!(
            parse_z_index(&[
                token(CssToken::Whitespace),
                token(CssToken::Number(42.0)),
                token(CssToken::Whitespace)
            ]),
            Some(CssValue::ZIndex(ZIndex::Index(42)))
        );

        // Invalid inputs
        assert_eq!(parse_z_index(&[token(CssToken::Number(1.5))]), None);
        assert_eq!(
            parse_z_index(&[token(CssToken::Dimension {
                value: 5.0,
                unit: "px".to_string()
            })]),
            None
        );
        assert_eq!(parse_z_index(&[token(CssToken::Percentage(50.0))]), None);
        assert_eq!(
            parse_z_index(&[token(CssToken::Ident("foo".to_string()))]),
            None
        );
        assert_eq!(
            parse_z_index(&[
                token(CssToken::Number(1.0)),
                token(CssToken::Whitespace),
                token(CssToken::Number(2.0))
            ]),
            None
        );
    }

    #[test]
    fn test_parse_opacity() {
        // 1. Bare numbers
        assert_eq!(parse_opacity("0"), Some(CssValue::Opacity(0.0)));
        assert_eq!(parse_opacity("1"), Some(CssValue::Opacity(1.0)));
        assert_eq!(parse_opacity("0.5"), Some(CssValue::Opacity(0.5)));
        assert_eq!(parse_opacity(".25"), Some(CssValue::Opacity(0.25)));

        // 2. Percentages
        assert_eq!(parse_opacity("50%"), Some(CssValue::Opacity(0.5)));
        assert_eq!(parse_opacity("100%"), Some(CssValue::Opacity(1.0)));

        // 3. Clamping
        assert_eq!(parse_opacity("1.5"), Some(CssValue::Opacity(1.0)));
        assert_eq!(parse_opacity("-0.2"), Some(CssValue::Opacity(0.0)));
        assert_eq!(parse_opacity("150%"), Some(CssValue::Opacity(1.0)));
        assert_eq!(parse_opacity("-10%"), Some(CssValue::Opacity(0.0)));

        // 4. Whitespace
        assert_eq!(parse_opacity("  0.3  "), Some(CssValue::Opacity(0.3)));

        // 5. Invalid -> None
        assert_eq!(parse_opacity(""), None);
        assert_eq!(parse_opacity("abc"), None);
        assert_eq!(parse_opacity("%"), None);
    }

    #[test]
    fn test_typed_z_index_and_opacity() {
        // ZIndex
        assert_eq!(ZIndex::parse("auto"), ZIndex::Auto);
        assert_eq!(ZIndex::parse("AUTO"), ZIndex::Auto);
        assert_eq!(ZIndex::parse("aUtO"), ZIndex::Auto);
        assert_eq!(ZIndex::parse("0"), ZIndex::Index(0));
        assert_eq!(ZIndex::parse("5"), ZIndex::Index(5));
        assert_eq!(ZIndex::parse("-1"), ZIndex::Index(-1));
        assert_eq!(ZIndex::parse("bogus"), ZIndex::Auto);
        assert_eq!(ZIndex::parse(""), ZIndex::Auto);
        assert_eq!(ZIndex::parse("1.5"), ZIndex::Auto);

        // FromStr for ZIndex (also covers leading/trailing whitespace trimming)
        use std::str::FromStr;
        assert_eq!(ZIndex::from_str("5").unwrap(), ZIndex::Index(5));
        assert_eq!(ZIndex::parse(" -5 "), ZIndex::Index(-5));

        // Opacity
        assert_eq!(Opacity::parse("0"), Opacity(0.0));
        assert_eq!(Opacity::parse("0.5"), Opacity(0.5));
        assert_eq!(Opacity::parse("1"), Opacity(1.0));
        assert_eq!(Opacity::parse("1.5"), Opacity(1.0));
        assert_eq!(Opacity::parse("-0.2"), Opacity(0.0));
        assert_eq!(Opacity::parse("bogus"), Opacity(1.0));
        assert_eq!(Opacity::parse(""), Opacity(1.0));

        // FromStr for Opacity (also covers whitespace trimming + clamping)
        assert_eq!(Opacity::from_str("0.25").unwrap(), Opacity(0.25));
        assert_eq!(Opacity::parse(" -10.0 "), Opacity(0.0));
    }

    #[test]
    fn test_flex_wrap_and_float() {
        // Test known properties
        assert!(is_known_layout_property("flex-wrap"));
        assert!(is_known_layout_property("float"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "flex-wrap",
            &CssValue::Keyword("wrap".to_string())
        ));
        assert!(is_valid_property_value(
            "flex-wrap",
            &CssValue::Keyword("nowrap".to_string())
        ));
        assert!(is_valid_property_value(
            "flex-wrap",
            &CssValue::Keyword("wrap-reverse".to_string())
        ));
        assert!(!is_valid_property_value(
            "flex-wrap",
            &CssValue::Keyword("banana".to_string())
        ));

        assert!(is_valid_property_value(
            "float",
            &CssValue::Keyword("left".to_string())
        ));
        assert!(is_valid_property_value(
            "float",
            &CssValue::Keyword("right".to_string())
        ));
        assert!(is_valid_property_value(
            "float",
            &CssValue::Keyword("none".to_string())
        ));
        assert!(!is_valid_property_value(
            "float",
            &CssValue::Keyword("up".to_string())
        ));

        // Test parse_property_value for flex-wrap
        assert_eq!(
            parse_property_value("flex-wrap", &[token(CssToken::Ident("wrap".to_string()))]),
            Some(CssValue::Keyword("wrap".to_string()))
        );
        assert_eq!(
            parse_property_value("flex-wrap", &[token(CssToken::Ident("nowrap".to_string()))]),
            Some(CssValue::Keyword("nowrap".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "flex-wrap",
                &[token(CssToken::Ident("wrap-reverse".to_string()))]
            ),
            Some(CssValue::Keyword("wrap-reverse".to_string()))
        );
        assert_eq!(
            parse_property_value("flex-wrap", &[token(CssToken::Ident("banana".to_string()))]),
            None
        );

        // Test parse_property_value for float
        assert_eq!(
            parse_property_value("float", &[token(CssToken::Ident("left".to_string()))]),
            Some(CssValue::Keyword("left".to_string()))
        );
        assert_eq!(
            parse_property_value("float", &[token(CssToken::Ident("right".to_string()))]),
            Some(CssValue::Keyword("right".to_string()))
        );
        assert_eq!(
            parse_property_value("float", &[token(CssToken::Ident("none".to_string()))]),
            Some(CssValue::Keyword("none".to_string()))
        );
        assert_eq!(
            parse_property_value("float", &[token(CssToken::Ident("up".to_string()))]),
            None
        );
    }

    #[test]
    fn test_clear_property() {
        // Test known properties
        assert!(is_known_layout_property("clear"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "clear",
            &CssValue::Keyword("left".to_string())
        ));
        assert!(is_valid_property_value(
            "clear",
            &CssValue::Keyword("right".to_string())
        ));
        assert!(is_valid_property_value(
            "clear",
            &CssValue::Keyword("none".to_string())
        ));
        assert!(is_valid_property_value(
            "clear",
            &CssValue::Keyword("both".to_string())
        ));
        assert!(!is_valid_property_value(
            "clear",
            &CssValue::Keyword("up".to_string())
        ));

        // Test parse_property_value for clear
        assert_eq!(
            parse_property_value("clear", &[token(CssToken::Ident("left".to_string()))]),
            Some(CssValue::Keyword("left".to_string()))
        );
        assert_eq!(
            parse_property_value("clear", &[token(CssToken::Ident("right".to_string()))]),
            Some(CssValue::Keyword("right".to_string()))
        );
        assert_eq!(
            parse_property_value("clear", &[token(CssToken::Ident("none".to_string()))]),
            Some(CssValue::Keyword("none".to_string()))
        );
        assert_eq!(
            parse_property_value("clear", &[token(CssToken::Ident("both".to_string()))]),
            Some(CssValue::Keyword("both".to_string()))
        );
        assert_eq!(
            parse_property_value("clear", &[token(CssToken::Ident("up".to_string()))]),
            None
        );
    }

    #[test]
    fn test_table_layout_property() {
        // Test known properties
        assert!(is_known_layout_property("table-layout"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "table-layout",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(is_valid_property_value(
            "table-layout",
            &CssValue::Keyword("fixed".to_string())
        ));
        assert!(!is_valid_property_value(
            "table-layout",
            &CssValue::Keyword("bogus".to_string())
        ));
        assert!(!is_valid_property_value(
            "table-layout",
            &CssValue::Number(1.0)
        ));

        // Test parse_property_value for table-layout
        assert_eq!(
            parse_property_value(
                "table-layout",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::Keyword("auto".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "table-layout",
                &[token(CssToken::Ident("fixed".to_string()))]
            ),
            Some(CssValue::Keyword("fixed".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "table-layout",
                &[token(CssToken::Ident("bogus".to_string()))]
            ),
            None
        );
    }

    #[test]
    fn test_visibility_property() {
        // Test known properties
        assert!(is_known_layout_property("visibility"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "visibility",
            &CssValue::Keyword("visible".to_string())
        ));
        assert!(is_valid_property_value(
            "visibility",
            &CssValue::Keyword("hidden".to_string())
        ));
        assert!(is_valid_property_value(
            "visibility",
            &CssValue::Keyword("collapse".to_string())
        ));
        assert!(!is_valid_property_value(
            "visibility",
            &CssValue::Keyword("gone".to_string())
        ));

        // Test parse_property_value for visibility
        assert_eq!(
            parse_property_value(
                "visibility",
                &[token(CssToken::Ident("visible".to_string()))]
            ),
            Some(CssValue::Keyword("visible".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "visibility",
                &[token(CssToken::Ident("hidden".to_string()))]
            ),
            Some(CssValue::Keyword("hidden".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "visibility",
                &[token(CssToken::Ident("collapse".to_string()))]
            ),
            Some(CssValue::Keyword("collapse".to_string()))
        );
        assert_eq!(
            parse_property_value("visibility", &[token(CssToken::Ident("gone".to_string()))]),
            None
        );
    }

    #[test]
    fn test_direction_property() {
        // Test known properties
        assert!(is_known_layout_property("direction"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "direction",
            &CssValue::Keyword("ltr".to_string())
        ));
        assert!(is_valid_property_value(
            "direction",
            &CssValue::Keyword("rtl".to_string())
        ));
        assert!(!is_valid_property_value(
            "direction",
            &CssValue::Keyword("sideways".to_string())
        ));

        // Test parse_property_value for direction
        assert_eq!(
            parse_property_value("direction", &[token(CssToken::Ident("ltr".to_string()))]),
            Some(CssValue::Keyword("ltr".to_string()))
        );
        assert_eq!(
            parse_property_value("direction", &[token(CssToken::Ident("rtl".to_string()))]),
            Some(CssValue::Keyword("rtl".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "direction",
                &[token(CssToken::Ident("sideways".to_string()))]
            ),
            None
        );
    }

    #[test]
    fn test_cursor_property() {
        // Test known properties
        assert!(is_known_layout_property("cursor"));
        assert!(is_known_layout_property("Cursor"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "cursor",
            &CssValue::Keyword("pointer".to_string())
        ));
        assert!(is_valid_property_value(
            "cursor",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(is_valid_property_value(
            "cursor",
            &CssValue::Keyword("not-allowed".to_string())
        ));
        assert!(is_valid_property_value(
            "cursor",
            &CssValue::Keyword("grab".to_string())
        ));
        assert!(is_valid_property_value(
            "cursor",
            &CssValue::Keyword("Pointer".to_string()) // Case insensitivity
        ));
        assert!(!is_valid_property_value(
            "cursor",
            &CssValue::Keyword("bogus".to_string())
        ));
        assert!(!is_valid_property_value(
            "cursor",
            &CssValue::Overflow(OverflowValue::Visible) // Non-keyword value
        ));

        // Test parse_property_value for cursor
        assert_eq!(
            parse_property_value("cursor", &[token(CssToken::Ident("pointer".to_string()))]),
            Some(CssValue::Keyword("pointer".to_string()))
        );
        assert_eq!(
            parse_property_value("cursor", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::Keyword("auto".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "cursor",
                &[token(CssToken::Ident("not-allowed".to_string()))]
            ),
            Some(CssValue::Keyword("not-allowed".to_string()))
        );
        assert_eq!(
            parse_property_value("cursor", &[token(CssToken::Ident("grab".to_string()))]),
            Some(CssValue::Keyword("grab".to_string()))
        );
        assert_eq!(
            parse_property_value("cursor", &[token(CssToken::Ident("bogus".to_string()))]),
            None
        );
    }

    // Guards recognition, validation, and parsing of the text-overflow property
    #[test]
    fn test_text_overflow_property() {
        // Test known properties
        assert!(is_known_layout_property("text-overflow"));
        assert!(is_known_layout_property("Text-Overflow"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "text-overflow",
            &CssValue::Keyword("clip".to_string())
        ));
        assert!(is_valid_property_value(
            "text-overflow",
            &CssValue::Keyword("ellipsis".to_string())
        ));
        assert!(is_valid_property_value(
            "text-overflow",
            &CssValue::Keyword("initial".to_string())
        ));
        assert!(is_valid_property_value(
            "text-overflow",
            &CssValue::Keyword("inherit".to_string())
        ));
        assert!(!is_valid_property_value(
            "text-overflow",
            &CssValue::Keyword("bogus".to_string())
        ));
        assert!(!is_valid_property_value(
            "text-overflow",
            &CssValue::Overflow(OverflowValue::Visible)
        ));

        // Test parse_property_value for text-overflow
        assert_eq!(
            parse_property_value(
                "text-overflow",
                &[token(CssToken::Ident("clip".to_string()))]
            ),
            Some(CssValue::Keyword("clip".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "text-overflow",
                &[token(CssToken::Ident("ellipsis".to_string()))]
            ),
            Some(CssValue::Keyword("ellipsis".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "text-overflow",
                &[token(CssToken::Ident("initial".to_string()))]
            ),
            Some(CssValue::Keyword("initial".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "text-overflow",
                &[token(CssToken::Ident("inherit".to_string()))]
            ),
            Some(CssValue::Keyword("inherit".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "text-overflow",
                &[token(CssToken::Ident("bogus".to_string()))]
            ),
            None
        );

        // Test parse_property_value for object-position (t0471)
        assert_eq!(
            parse_property_value(
                "object-position",
                &[token(CssToken::Ident("center".to_string()))]
            ),
            Some(CssValue::Keyword("center".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "object-position",
                &[
                    token(CssToken::Percentage(50.0)),
                    token(CssToken::Whitespace),
                    token(CssToken::Percentage(50.0)),
                ]
            ),
            Some(CssValue::Multiple(vec![
                CssValue::Length(50.0, LengthUnit::Percent),
                CssValue::Length(50.0, LengthUnit::Percent),
            ]))
        );

        // Test parse_property_value for scroll-behavior (t0473)
        assert_eq!(
            parse_property_value(
                "scroll-behavior",
                &[token(CssToken::Ident("smooth".to_string()))]
            ),
            Some(CssValue::Keyword("smooth".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "scroll-behavior",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::Keyword("auto".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "scroll-behavior",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for scroll-behavior (t0473)
        assert!(is_valid_property_value(
            "scroll-behavior",
            &CssValue::Keyword("smooth".to_string())
        ));
        assert!(is_valid_property_value(
            "scroll-behavior",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(!is_valid_property_value(
            "scroll-behavior",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value for user-select (t0475)
        for val in &["auto", "text", "none", "contain", "all", "AUTO", "None"] {
            assert_eq!(
                parse_property_value("user-select", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "user-select",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for user-select (t0475)
        for val in &["auto", "text", "none", "contain", "all", "AUTO", "None"] {
            assert!(is_valid_property_value(
                "user-select",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "user-select",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value for accent-color and caret-color (t0477)
        assert_eq!(
            parse_property_value(
                "accent-color",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::Keyword("auto".to_string()))
        );
        assert_eq!(
            parse_property_value("accent-color", &[token(CssToken::Ident("red".to_string()))]),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
        assert_eq!(
            parse_property_value(
                "accent-color",
                &[token(CssToken::Hash("00ff00".to_string()))]
            ),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );
        assert_eq!(
            parse_property_value(
                "accent-color",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for accent-color and caret-color (t0477)
        assert!(is_valid_property_value(
            "accent-color",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(is_valid_property_value(
            "accent-color",
            &CssValue::Color(Color::Rgba(255, 0, 0, 255))
        ));
        assert!(!is_valid_property_value(
            "accent-color",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        assert_eq!(
            parse_property_value("caret-color", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::Keyword("auto".to_string()))
        );
        assert_eq!(
            parse_property_value("caret-color", &[token(CssToken::Ident("blue".to_string()))]),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );
        assert_eq!(
            parse_property_value(
                "caret-color",
                &[token(CssToken::Ident("currentcolor".to_string()))]
            ),
            Some(CssValue::Keyword("currentcolor".to_string()))
        );

        // Test parse_property_value for transition-timing-function and transition-delay (t0479)
        for val in &[
            "ease",
            "linear",
            "ease-in",
            "ease-out",
            "ease-in-out",
            "step-start",
            "step-end",
            "EASE",
            "Ease-In",
        ] {
            assert_eq!(
                parse_property_value(
                    "transition-timing-function",
                    &[token(CssToken::Ident(val.to_string()))]
                ),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "transition-timing-function",
                &[token(CssToken::Ident("invalid".to_string()))]
            ),
            None
        );

        assert_eq!(
            parse_property_value(
                "transition-delay",
                &[token(CssToken::Dimension {
                    value: 200.0,
                    unit: "ms".to_string()
                })]
            ),
            Some(CssValue::Keyword("200ms".to_string()))
        );
        assert_eq!(
            parse_property_value(
                "transition-delay",
                &[token(CssToken::Dimension {
                    value: 1.5,
                    unit: "s".to_string()
                })]
            ),
            Some(CssValue::Keyword("1.5s".to_string()))
        );
        assert_eq!(
            parse_property_value("transition-delay", &[token(CssToken::Number(0.0))]),
            Some(CssValue::Number(0.0))
        );
        assert_eq!(
            parse_property_value("transition-delay", &[token(CssToken::Number(5.0))]),
            None
        );
        assert_eq!(
            parse_property_value(
                "transition-delay",
                &[token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string()
                })]
            ),
            None
        );

        // Test overscroll properties (t0485)
        assert!(is_known_layout_property("overscroll-behavior"));
        assert!(is_known_layout_property("overscroll-behavior-x"));
        assert!(is_known_layout_property("overscroll-behavior-y"));

        // Test parse_property_value and is_valid_property_value for overscroll-behavior-x and overscroll-behavior-y
        for prop in &["overscroll-behavior-x", "overscroll-behavior-y"] {
            for val in &["auto", "contain", "none", "AUTO", "None"] {
                assert_eq!(
                    parse_property_value(prop, &[token(CssToken::Ident(val.to_string()))]),
                    Some(CssValue::Keyword(val.to_string()))
                );
                assert!(is_valid_property_value(
                    prop,
                    &CssValue::Keyword(val.to_string())
                ));
            }
            assert_eq!(
                parse_property_value(prop, &[token(CssToken::Ident("invalid".to_string()))]),
                None
            );
            assert!(!is_valid_property_value(
                prop,
                &CssValue::Keyword("invalid".to_string())
            ));
            // Multiple values are invalid for -x and -y longhands
            assert_eq!(
                parse_property_value(
                    prop,
                    &[
                        token(CssToken::Ident("contain".to_string())),
                        token(CssToken::Whitespace),
                        token(CssToken::Ident("none".to_string())),
                    ]
                ),
                None
            );
        }

        // Test parse_property_value and is_valid_property_value for overscroll-behavior
        for val in &["auto", "contain", "none", "AUTO", "None"] {
            assert_eq!(
                parse_property_value(
                    "overscroll-behavior",
                    &[token(CssToken::Ident(val.to_string()))]
                ),
                Some(CssValue::Keyword(val.to_string()))
            );
            assert!(is_valid_property_value(
                "overscroll-behavior",
                &CssValue::Keyword(val.to_string())
            ));
        }

        // Test overscroll-behavior with 2 values
        assert_eq!(
            parse_property_value(
                "overscroll-behavior",
                &[
                    token(CssToken::Ident("contain".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("none".to_string())),
                ]
            ),
            Some(CssValue::Multiple(vec![
                CssValue::Keyword("contain".to_string()),
                CssValue::Keyword("none".to_string()),
            ]))
        );
        assert!(is_valid_property_value(
            "overscroll-behavior",
            &CssValue::Multiple(vec![
                CssValue::Keyword("contain".to_string()),
                CssValue::Keyword("none".to_string()),
            ])
        ));

        // Test invalid 2-value overscroll-behavior
        assert_eq!(
            parse_property_value(
                "overscroll-behavior",
                &[
                    token(CssToken::Ident("contain".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("invalid".to_string())),
                ]
            ),
            None
        );
        assert!(!is_valid_property_value(
            "overscroll-behavior",
            &CssValue::Multiple(vec![
                CssValue::Keyword("contain".to_string()),
                CssValue::Keyword("invalid".to_string()),
            ])
        ));

        // Test invalid 3-value overscroll-behavior
        assert_eq!(
            parse_property_value(
                "overscroll-behavior",
                &[
                    token(CssToken::Ident("contain".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("none".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("auto".to_string())),
                ]
            ),
            None
        );
    }
}
