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
pub enum ColumnSpanValue {
    None,
    All,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnFillValue {
    Auto,
    Balance,
    BalanceAll,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DisplayValue {
    Block,
    Inline,
    InlineBlock,
    None,
    Flex,
    Grid,
    InlineGrid,
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
pub enum ScrollSnapAxis {
    X,
    Y,
    Block,
    Inline,
    Both,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScrollSnapStrictness {
    Mandatory,
    Proximity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScrollSnapTypeValue {
    None,
    Axis(ScrollSnapAxis, ScrollSnapStrictness),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScrollSnapAlignKeyword {
    None,
    Start,
    End,
    Center,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScrollSnapAlignValue {
    pub block: ScrollSnapAlignKeyword,
    pub inline: ScrollSnapAlignKeyword,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MixBlendModeValue {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BackgroundBlendModeValue {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl BackgroundBlendModeValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "multiply" => Some(Self::Multiply),
            "screen" => Some(Self::Screen),
            "overlay" => Some(Self::Overlay),
            "darken" => Some(Self::Darken),
            "lighten" => Some(Self::Lighten),
            "color-dodge" => Some(Self::ColorDodge),
            "color-burn" => Some(Self::ColorBurn),
            "hard-light" => Some(Self::HardLight),
            "soft-light" => Some(Self::SoftLight),
            "difference" => Some(Self::Difference),
            "exclusion" => Some(Self::Exclusion),
            "hue" => Some(Self::Hue),
            "saturation" => Some(Self::Saturation),
            "color" => Some(Self::Color),
            "luminosity" => Some(Self::Luminosity),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Multiply => "multiply",
            Self::Screen => "screen",
            Self::Overlay => "overlay",
            Self::Darken => "darken",
            Self::Lighten => "lighten",
            Self::ColorDodge => "color-dodge",
            Self::ColorBurn => "color-burn",
            Self::HardLight => "hard-light",
            Self::SoftLight => "soft-light",
            Self::Difference => "difference",
            Self::Exclusion => "exclusion",
            Self::Hue => "hue",
            Self::Saturation => "saturation",
            Self::Color => "color",
            Self::Luminosity => "luminosity",
        }
    }
}

impl std::str::FromStr for BackgroundBlendModeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IsolationValue {
    Auto,
    Isolate,
}

impl IsolationValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "isolate" => Some(Self::Isolate),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Isolate => "isolate",
        }
    }
}

impl std::str::FromStr for IsolationValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BackfaceVisibilityValue {
    Visible,
    Hidden,
}

impl BackfaceVisibilityValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "visible" => Some(Self::Visible),
            "hidden" => Some(Self::Hidden),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Hidden => "hidden",
        }
    }
}

impl std::str::FromStr for BackfaceVisibilityValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResizeValue {
    None,
    Both,
    Horizontal,
    Vertical,
}

impl ResizeValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "both" => Some(Self::Both),
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Both => "both",
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

impl std::str::FromStr for ResizeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ObjectFitValue {
    Fill,
    Contain,
    Cover,
    None,
    ScaleDown,
}

impl ObjectFitValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "fill" => Some(Self::Fill),
            "contain" => Some(Self::Contain),
            "cover" => Some(Self::Cover),
            "none" => Some(Self::None),
            "scale-down" => Some(Self::ScaleDown),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fill => "fill",
            Self::Contain => "contain",
            Self::Cover => "cover",
            Self::None => "none",
            Self::ScaleDown => "scale-down",
        }
    }
}

impl std::str::FromStr for ObjectFitValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ObjectFitValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WritingModeValue {
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl WritingModeValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "horizontal-tb" => Some(Self::HorizontalTb),
            "vertical-rl" => Some(Self::VerticalRl),
            "vertical-lr" => Some(Self::VerticalLr),
            "sideways-rl" => Some(Self::SidewaysRl),
            "sideways-lr" => Some(Self::SidewaysLr),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
            Self::SidewaysRl => "sideways-rl",
            Self::SidewaysLr => "sideways-lr",
        }
    }
}

impl std::str::FromStr for WritingModeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for WritingModeValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TextOrientationValue {
    #[default]
    Mixed,
    Upright,
    Sideways,
}

impl TextOrientationValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mixed" => Some(Self::Mixed),
            "upright" => Some(Self::Upright),
            "sideways" => Some(Self::Sideways),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::Upright => "upright",
            Self::Sideways => "sideways",
        }
    }
}

impl std::str::FromStr for TextOrientationValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextOrientationValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::TextOrientation(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum BoxDecorationBreakValue {
    #[default]
    Slice,
    Clone,
}

impl BoxDecorationBreakValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "slice" => Some(Self::Slice),
            "clone" => Some(Self::Clone),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Slice => "slice",
            Self::Clone => "clone",
        }
    }
}

impl std::str::FromStr for BoxDecorationBreakValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for BoxDecorationBreakValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::BoxDecorationBreak(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum MaskTypeValue {
    #[default]
    Luminance,
    Alpha,
}

impl MaskTypeValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "luminance" => Some(Self::Luminance),
            "alpha" => Some(Self::Alpha),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Luminance => "luminance",
            Self::Alpha => "alpha",
        }
    }
}

impl std::str::FromStr for MaskTypeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for MaskTypeValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::MaskType(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ScrollBehaviorValue {
    #[default]
    Auto,
    Smooth,
}

impl ScrollBehaviorValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "smooth" => Some(Self::Smooth),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth => "smooth",
        }
    }
}

impl std::str::FromStr for ScrollBehaviorValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ScrollBehaviorValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::ScrollBehavior(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum PrintColorAdjustValue {
    #[default]
    Economy,
    Exact,
}

impl PrintColorAdjustValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "economy" => Some(Self::Economy),
            "exact" => Some(Self::Exact),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Economy => "economy",
            Self::Exact => "exact",
        }
    }
}

impl std::str::FromStr for PrintColorAdjustValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for PrintColorAdjustValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::PrintColorAdjust(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ColorSchemeValue {
    #[default]
    Normal,
    Light,
    Dark,
}

impl ColorSchemeValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

impl std::str::FromStr for ColorSchemeValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ColorSchemeValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::ColorScheme(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ForcedColorAdjustValue {
    #[default]
    Auto,
    None,
}

impl ForcedColorAdjustValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
        }
    }
}

impl std::str::FromStr for ForcedColorAdjustValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ForcedColorAdjustValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::ForcedColorAdjust(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FontVariantPositionValue {
    #[default]
    Normal,
    Sub,
    Super,
}

impl FontVariantPositionValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "sub" => Some(Self::Sub),
            "super" => Some(Self::Super),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Sub => "sub",
            Self::Super => "super",
        }
    }
}

impl std::str::FromStr for FontVariantPositionValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for FontVariantPositionValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::FontVariantPosition(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FontOpticalSizingValue {
    #[default]
    Auto,
    None,
}

impl FontOpticalSizingValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
        }
    }
}

impl std::str::FromStr for FontOpticalSizingValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for FontOpticalSizingValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::FontOpticalSizing(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CaptionSideValue {
    Top,
    Bottom,
}

impl CaptionSideValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
}

impl std::str::FromStr for CaptionSideValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for CaptionSideValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ColorInterpolationValue {
    Auto,
    #[default]
    Srgb,
    LinearRgb,
}

impl ColorInterpolationValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "srgb" => Some(Self::Srgb),
            "linearrgb" => Some(Self::LinearRgb),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Srgb => "sRGB",
            Self::LinearRgb => "linearRGB",
        }
    }
}

impl std::str::FromStr for ColorInterpolationValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ColorInterpolationValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum UserSelectValue {
    #[default]
    Auto,
    Text,
    None,
    Contain,
    All,
}

impl UserSelectValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "text" => Some(Self::Text),
            "none" => Some(Self::None),
            "contain" => Some(Self::Contain),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Text => "text",
            Self::None => "none",
            Self::Contain => "contain",
            Self::All => "all",
        }
    }
}

impl std::str::FromStr for UserSelectValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for UserSelectValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TransformStyleValue {
    #[default]
    Flat,
    Preserve3d,
}

impl TransformStyleValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "flat" => Some(Self::Flat),
            "preserve-3d" => Some(Self::Preserve3d),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Preserve3d => "preserve-3d",
        }
    }
}

impl std::str::FromStr for TransformStyleValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TransformStyleValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum BreakInsideValue {
    #[default]
    Auto,
    Avoid,
    AvoidPage,
    AvoidColumn,
    AvoidRegion,
}

impl BreakInsideValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "avoid" => Some(Self::Avoid),
            "avoid-page" => Some(Self::AvoidPage),
            "avoid-column" => Some(Self::AvoidColumn),
            "avoid-region" => Some(Self::AvoidRegion),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Avoid => "avoid",
            Self::AvoidPage => "avoid-page",
            Self::AvoidColumn => "avoid-column",
            Self::AvoidRegion => "avoid-region",
        }
    }
}

impl std::str::FromStr for BreakInsideValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for BreakInsideValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmptyCellsValue {
    Show,
    Hide,
}

impl EmptyCellsValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "show" => Some(Self::Show),
            "hide" => Some(Self::Hide),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Hide => "hide",
        }
    }
}

impl std::str::FromStr for EmptyCellsValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for EmptyCellsValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum BackgroundAttachmentValue {
    #[default]
    Scroll,
    Fixed,
    Local,
}

impl BackgroundAttachmentValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "scroll" => Some(Self::Scroll),
            "fixed" => Some(Self::Fixed),
            "local" => Some(Self::Local),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Scroll => "scroll",
            Self::Fixed => "fixed",
            Self::Local => "local",
        }
    }
}

impl std::str::FromStr for BackgroundAttachmentValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for BackgroundAttachmentValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TextWrapValue {
    #[default]
    Wrap,
    Nowrap,
    Balance,
    Pretty,
    Stable,
}

impl TextWrapValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "wrap" => Some(Self::Wrap),
            "nowrap" => Some(Self::Nowrap),
            "balance" => Some(Self::Balance),
            "pretty" => Some(Self::Pretty),
            "stable" => Some(Self::Stable),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wrap => "wrap",
            Self::Nowrap => "nowrap",
            Self::Balance => "balance",
            Self::Pretty => "pretty",
            Self::Stable => "stable",
        }
    }
}

impl std::str::FromStr for TextWrapValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextWrapValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum BorderCollapseValue {
    #[default]
    Separate,
    Collapse,
}

impl BorderCollapseValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "separate" => Some(Self::Separate),
            "collapse" => Some(Self::Collapse),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Separate => "separate",
            Self::Collapse => "collapse",
        }
    }
}

impl std::str::FromStr for BorderCollapseValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for BorderCollapseValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum ClearValue {
    #[default]
    None,
    Left,
    Right,
    Both,
    InlineStart,
    InlineEnd,
}

impl ClearValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "both" => Some(Self::Both),
            "inline-start" => Some(Self::InlineStart),
            "inline-end" => Some(Self::InlineEnd),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Left => "left",
            Self::Right => "right",
            Self::Both => "both",
            Self::InlineStart => "inline-start",
            Self::InlineEnd => "inline-end",
        }
    }
}

impl std::str::FromStr for ClearValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ClearValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Default)]
pub enum TextAlignLastValue {
    #[default]
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

impl TextAlignLastValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "start" => Some(Self::Start),
            "end" => Some(Self::End),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "center" => Some(Self::Center),
            "justify" => Some(Self::Justify),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Start => "start",
            Self::End => "end",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::Justify => "justify",
        }
    }
}

impl std::str::FromStr for TextAlignLastValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextAlignLastValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::TextAlignLast(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Default)]
pub enum UnicodeBidiValue {
    #[default]
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

impl UnicodeBidiValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "embed" => Some(Self::Embed),
            "isolate" => Some(Self::Isolate),
            "bidi-override" => Some(Self::BidiOverride),
            "isolate-override" => Some(Self::IsolateOverride),
            "plaintext" => Some(Self::Plaintext),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Embed => "embed",
            Self::Isolate => "isolate",
            Self::BidiOverride => "bidi-override",
            Self::IsolateOverride => "isolate-override",
            Self::Plaintext => "plaintext",
        }
    }
}

impl std::str::FromStr for UnicodeBidiValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for UnicodeBidiValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::UnicodeBidi(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HyphensValue {
    None,
    Manual,
    Auto,
}

impl HyphensValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "manual" => Some(Self::Manual),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }
}

impl std::str::FromStr for HyphensValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for HyphensValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextRenderingValue {
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

impl TextRenderingValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "optimizeSpeed" => Some(Self::OptimizeSpeed),
            "optimizeLegibility" => Some(Self::OptimizeLegibility),
            "geometricPrecision" => Some(Self::GeometricPrecision),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::OptimizeSpeed => "optimizeSpeed",
            Self::OptimizeLegibility => "optimizeLegibility",
            Self::GeometricPrecision => "geometricPrecision",
        }
    }
}

impl std::str::FromStr for TextRenderingValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextRenderingValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ImageRenderingValue {
    Auto,
    CrispEdges,
    Pixelated,
    Smooth,
    HighQuality,
}

impl ImageRenderingValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "crisp-edges" => Some(Self::CrispEdges),
            "pixelated" => Some(Self::Pixelated),
            "smooth" => Some(Self::Smooth),
            "high-quality" => Some(Self::HighQuality),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::CrispEdges => "crisp-edges",
            Self::Pixelated => "pixelated",
            Self::Smooth => "smooth",
            Self::HighQuality => "high-quality",
        }
    }
}

impl std::str::FromStr for ImageRenderingValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ImageRenderingValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::ImageRendering(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FontVariantCapsValue {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

impl FontVariantCapsValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "small-caps" => Some(Self::SmallCaps),
            "all-small-caps" => Some(Self::AllSmallCaps),
            "petite-caps" => Some(Self::PetiteCaps),
            "all-petite-caps" => Some(Self::AllPetiteCaps),
            "unicase" => Some(Self::Unicase),
            "titling-caps" => Some(Self::TitlingCaps),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::SmallCaps => "small-caps",
            Self::AllSmallCaps => "all-small-caps",
            Self::PetiteCaps => "petite-caps",
            Self::AllPetiteCaps => "all-petite-caps",
            Self::Unicase => "unicase",
            Self::TitlingCaps => "titling-caps",
        }
    }
}

impl std::str::FromStr for FontVariantCapsValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for FontVariantCapsValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::FontVariantCaps(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FontStretchValue {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

impl FontStretchValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "ultra-condensed" => Some(Self::UltraCondensed),
            "extra-condensed" => Some(Self::ExtraCondensed),
            "condensed" => Some(Self::Condensed),
            "semi-condensed" => Some(Self::SemiCondensed),
            "normal" => Some(Self::Normal),
            "semi-expanded" => Some(Self::SemiExpanded),
            "expanded" => Some(Self::Expanded),
            "extra-expanded" => Some(Self::ExtraExpanded),
            "ultra-expanded" => Some(Self::UltraExpanded),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UltraCondensed => "ultra-condensed",
            Self::ExtraCondensed => "extra-condensed",
            Self::Condensed => "condensed",
            Self::SemiCondensed => "semi-condensed",
            Self::Normal => "normal",
            Self::SemiExpanded => "semi-expanded",
            Self::Expanded => "expanded",
            Self::ExtraExpanded => "extra-expanded",
            Self::UltraExpanded => "ultra-expanded",
        }
    }
}

impl std::str::FromStr for FontStretchValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for FontStretchValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::FontStretch(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FontKerningValue {
    Auto,
    Normal,
    None,
}

impl FontKerningValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "normal" => Some(Self::Normal),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Normal => "normal",
            Self::None => "none",
        }
    }
}

impl std::str::FromStr for FontKerningValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for FontKerningValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextJustifyValue {
    Auto,
    InterWord,
    InterCharacter,
    None,
}

impl TextJustifyValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "inter-word" => Some(Self::InterWord),
            "inter-character" => Some(Self::InterCharacter),
            "none" => Some(Self::None),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::InterWord => "inter-word",
            Self::InterCharacter => "inter-character",
            Self::None => "none",
        }
    }
}

impl std::str::FromStr for TextJustifyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextJustifyValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum WordBreakValue {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

impl WordBreakValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "break-all" => Some(Self::BreakAll),
            "keep-all" => Some(Self::KeepAll),
            "break-word" => Some(Self::BreakWord),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::BreakAll => "break-all",
            Self::KeepAll => "keep-all",
            Self::BreakWord => "break-word",
        }
    }
}

impl std::str::FromStr for WordBreakValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for WordBreakValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum LineBreakValue {
    #[default]
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

impl LineBreakValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "loose" => Some(Self::Loose),
            "normal" => Some(Self::Normal),
            "strict" => Some(Self::Strict),
            "anywhere" => Some(Self::Anywhere),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Loose => "loose",
            Self::Normal => "normal",
            Self::Strict => "strict",
            Self::Anywhere => "anywhere",
        }
    }
}

impl std::str::FromStr for LineBreakValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for LineBreakValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            CssValue::LineBreak(v) => Ok(*v),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum OverflowWrapValue {
    #[default]
    Normal,
    BreakWord,
    Anywhere,
}

impl OverflowWrapValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "normal" => Some(Self::Normal),
            "break-word" => Some(Self::BreakWord),
            "anywhere" => Some(Self::Anywhere),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::BreakWord => "break-word",
            Self::Anywhere => "anywhere",
        }
    }
}

impl std::str::FromStr for OverflowWrapValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for OverflowWrapValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ImageRendering {
    Auto,
    Smooth,
    HighQuality,
    CrispEdges,
    Pixelated,
}

impl ImageRendering {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "smooth" => Some(Self::Smooth),
            "high-quality" => Some(Self::HighQuality),
            "crisp-edges" => Some(Self::CrispEdges),
            "pixelated" => Some(Self::Pixelated),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth => "smooth",
            Self::HighQuality => "high-quality",
            Self::CrispEdges => "crisp-edges",
            Self::Pixelated => "pixelated",
        }
    }
}

impl std::str::FromStr for ImageRendering {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for ImageRendering {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PointerEventsValue {
    Auto,
    None,
    VisiblePainted,
    VisibleFill,
    VisibleStroke,
    Visible,
    Painted,
    Fill,
    Stroke,
    All,
}

impl PointerEventsValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "none" => Some(Self::None),
            "visiblepainted" => Some(Self::VisiblePainted),
            "visiblefill" => Some(Self::VisibleFill),
            "visiblestroke" => Some(Self::VisibleStroke),
            "visible" => Some(Self::Visible),
            "painted" => Some(Self::Painted),
            "fill" => Some(Self::Fill),
            "stroke" => Some(Self::Stroke),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::None => "none",
            Self::VisiblePainted => "visiblePainted",
            Self::VisibleFill => "visibleFill",
            Self::VisibleStroke => "visibleStroke",
            Self::Visible => "visible",
            Self::Painted => "painted",
            Self::Fill => "fill",
            Self::Stroke => "stroke",
            Self::All => "all",
        }
    }
}

impl std::str::FromStr for PointerEventsValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for PointerEventsValue {
    type Error = ();

    fn try_from(value: &CssValue) -> Result<Self, Self::Error> {
        match value {
            CssValue::Keyword(s) => s.parse(),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum TextDecorationStyleValue {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

impl TextDecorationStyleValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "solid" => Some(Self::Solid),
            "double" => Some(Self::Double),
            "dotted" => Some(Self::Dotted),
            "dashed" => Some(Self::Dashed),
            "wavy" => Some(Self::Wavy),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Double => "double",
            Self::Dotted => "dotted",
            Self::Dashed => "dashed",
            Self::Wavy => "wavy",
        }
    }
}

impl std::str::FromStr for TextDecorationStyleValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<&CssValue> for TextDecorationStyleValue {
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

impl LengthOrPercent {
    pub fn resolve(
        &self,
        percent_basis: f32,
        font_size: f32,
        root_font_size: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> f32 {
        match self.unit {
            LengthUnit::Px => self.value,
            LengthUnit::Em => self.value * font_size,
            LengthUnit::Rem => self.value * root_font_size,
            LengthUnit::Pt => self.value * 96.0 / 72.0,
            LengthUnit::Percent => self.value / 100.0 * percent_basis,
            LengthUnit::Vw => self.value * viewport_w / 100.0,
            LengthUnit::Vh => self.value * viewport_h / 100.0,
        }
    }
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
    Matrix([f32; 6]),
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
pub enum GridTrackSize {
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
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
    ColumnSpan(ColumnSpanValue),
    ColumnFill(ColumnFillValue),
    Display(DisplayValue),
    FlexDirection(FlexDirectionValue),
    JustifyContent(JustifyContentValue),
    AlignItems(AlignItemsValue),
    Transform(Vec<TransformFn>),
    ZIndex(ZIndex),
    Opacity(f32),
    GridTemplate(Vec<GridTrackSize>),
    ScrollSnapType(ScrollSnapTypeValue),
    ScrollSnapAlign(ScrollSnapAlignValue),
    MixBlendMode(MixBlendModeValue),
    BackgroundBlendMode(BackgroundBlendModeValue),
    Isolation(IsolationValue),
    Resize(ResizeValue),
    BackfaceVisibility(BackfaceVisibilityValue),
    EmptyCells(EmptyCellsValue),
    TextAlignLast(TextAlignLastValue),
    UnicodeBidi(UnicodeBidiValue),
    Hyphens(HyphensValue),
    LineBreak(LineBreakValue),
    TextOrientation(TextOrientationValue),
    TextRendering(TextRenderingValue),
    ImageRendering(ImageRenderingValue),
    FontVariantCaps(FontVariantCapsValue),
    FontVariantPosition(FontVariantPositionValue),
    FontStretch(FontStretchValue),
    FontOpticalSizing(FontOpticalSizingValue),
    BoxDecorationBreak(BoxDecorationBreakValue),
    MaskType(MaskTypeValue),
    ScrollBehavior(ScrollBehaviorValue),
    PrintColorAdjust(PrintColorAdjustValue),
    ForcedColorAdjust(ForcedColorAdjustValue),
    ColorScheme(ColorSchemeValue),
}

impl CssValue {
    pub fn resolve_to_px(
        &self,
        percent_basis: f32,
        font_size: f32,
        root_font_size: f32,
        viewport_w: f32,
        viewport_h: f32,
    ) -> Option<f32> {
        match self {
            CssValue::Length(v, u) => {
                let lp = LengthOrPercent {
                    value: *v,
                    unit: u.clone(),
                };
                Some(lp.resolve(
                    percent_basis,
                    font_size,
                    root_font_size,
                    viewport_w,
                    viewport_h,
                ))
            }
            CssValue::Number(v) => Some(*v),
            _ => None,
        }
    }
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
            | "column-span"
            | "column-fill"
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
            | "print-color-adjust"
            | "forced-color-adjust"
            | "color-scheme"
            | "scroll-snap-type"
            | "scroll-snap-align"
            | "mix-blend-mode"
            | "background-blend-mode"
            | "isolation"
            | "resize"
            | "backface-visibility"
            | "empty-cells"
            | "text-align-last"
            | "hyphens"
            | "text-rendering"
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
            | "animation-timing-function"
            | "transition-delay"
            | "grid-template-columns"
            | "grid-template-rows"
            | "image-rendering"
            | "font-variant-caps"
            | "font-variant-position"
            | "font-stretch"
            | "font-kerning"
            | "font-optical-sizing"
            | "text-justify"
            | "word-break"
            | "line-break"
            | "text-orientation"
            | "box-decoration-break"
            | "mask-type"
            | "overflow-wrap"
            | "word-wrap"
            | "object-fit"
            | "caption-side"
            | "border-collapse"
            | "break-inside"
            | "pointer-events"
            | "unicode-bidi"
    )
}

/// Validates that a CSS value is valid for a layout-related property.
pub fn is_valid_property_value(name: &str, value: &CssValue) -> bool {
    if let CssValue::Keyword(kw) = value {
        let kw_lower = kw.to_ascii_lowercase();
        if kw_lower == "inherit"
            || kw_lower == "initial"
            || kw_lower == "unset"
            || kw_lower == "revert"
            || kw_lower == "revert-layer"
        {
            return true;
        }
    }

    let name_lower = name.to_ascii_lowercase();
    match name_lower.as_str() {
        "grid-template-columns" | "grid-template-rows" => true,
        "scroll-snap-type" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "none" | "x" | "y" | "block" | "inline" | "both"
                )
            }
            CssValue::ScrollSnapType(_) => true,
            _ => false,
        },
        "scroll-snap-align" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "none" | "start" | "end" | "center"
                )
            }
            CssValue::ScrollSnapAlign(_) => true,
            _ => false,
        },
        "mix-blend-mode" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal"
                        | "multiply"
                        | "screen"
                        | "overlay"
                        | "darken"
                        | "lighten"
                        | "color-dodge"
                        | "color-burn"
                        | "hard-light"
                        | "soft-light"
                        | "difference"
                        | "exclusion"
                        | "hue"
                        | "saturation"
                        | "color"
                        | "luminosity"
                )
            }
            CssValue::MixBlendMode(_) => true,
            _ => false,
        },
        "background-blend-mode" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal"
                        | "multiply"
                        | "screen"
                        | "overlay"
                        | "darken"
                        | "lighten"
                        | "color-dodge"
                        | "color-burn"
                        | "hard-light"
                        | "soft-light"
                        | "difference"
                        | "exclusion"
                        | "hue"
                        | "saturation"
                        | "color"
                        | "luminosity"
                )
            }
            CssValue::BackgroundBlendMode(_) => true,
            _ => false,
        },
        "isolation" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "isolate")
            }
            CssValue::Isolation(_) => true,
            _ => false,
        },
        "resize" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "none" | "both" | "horizontal" | "vertical"
                )
            }
            CssValue::Resize(_) => true,
            _ => false,
        },
        "backface-visibility" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "visible" | "hidden")
            }
            CssValue::BackfaceVisibility(_) => true,
            _ => false,
        },
        "empty-cells" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "show" | "hide")
            }
            CssValue::EmptyCells(_) => true,
            _ => false,
        },
        "text-align-last" => match value {
            CssValue::Keyword(kw) => TextAlignLastValue::parse(kw).is_some(),
            CssValue::TextAlignLast(_) => true,
            _ => false,
        },
        "unicode-bidi" => match value {
            CssValue::Keyword(kw) => UnicodeBidiValue::parse(kw).is_some(),
            CssValue::UnicodeBidi(_) => true,
            _ => false,
        },
        "hyphens" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "none" | "manual" | "auto")
            }
            CssValue::Hyphens(_) => true,
            _ => false,
        },
        "line-break" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "loose" | "normal" | "strict" | "anywhere"
                )
            }
            CssValue::LineBreak(_) => true,
            _ => false,
        },
        "text-orientation" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "mixed" | "upright" | "sideways"
                )
            }
            CssValue::TextOrientation(_) => true,
            _ => false,
        },
        "box-decoration-break" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "slice" | "clone")
            }
            CssValue::BoxDecorationBreak(_) => true,
            _ => false,
        },
        "mask-type" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "luminance" | "alpha")
            }
            CssValue::MaskType(_) => true,
            _ => false,
        },
        "text-rendering" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.as_str(),
                    "auto" | "optimizeSpeed" | "optimizeLegibility" | "geometricPrecision"
                )
            }
            CssValue::TextRendering(_) => true,
            _ => false,
        },
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
        "column-span" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "none" | "all")
            }
            CssValue::ColumnSpan(_) => true,
            _ => false,
        },
        "column-fill" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "balance" | "balance-all"
                )
            }
            CssValue::ColumnFill(_) => true,
            _ => false,
        },
        "display" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "block" | "inline" | "inline-block" | "none" | "flex" | "grid" | "inline-grid"
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
            CssValue::ScrollBehavior(_) => true,
            _ => false,
        },
        "print-color-adjust" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "economy" | "exact")
            }
            CssValue::PrintColorAdjust(_) => true,
            _ => false,
        },
        "forced-color-adjust" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "none")
            }
            CssValue::ForcedColorAdjust(_) => true,
            _ => false,
        },
        "color-scheme" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal" | "light" | "dark"
                )
            }
            CssValue::ColorScheme(_) => true,
            _ => false,
        },
        "image-rendering" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "smooth" | "high-quality" | "crisp-edges" | "pixelated"
                )
            }
            CssValue::ImageRendering(_) => true,
            _ => false,
        },
        "font-variant-caps" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal"
                        | "small-caps"
                        | "all-small-caps"
                        | "petite-caps"
                        | "all-petite-caps"
                        | "unicase"
                        | "titling-caps"
                )
            }
            CssValue::FontVariantCaps(_) => true,
            _ => false,
        },
        "font-variant-position" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "normal" | "sub" | "super")
            }
            CssValue::FontVariantPosition(_) => true,
            _ => false,
        },
        "font-stretch" => match value {
            CssValue::Keyword(kw) => FontStretchValue::parse(kw).is_some(),
            CssValue::FontStretch(_) => true,
            _ => false,
        },
        "font-optical-sizing" => match value {
            CssValue::Keyword(kw) => FontOpticalSizingValue::parse(kw).is_some(),
            CssValue::FontOpticalSizing(_) => true,
            _ => false,
        },
        "font-kerning" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "auto" | "normal" | "none")
            }
            _ => false,
        },
        "text-justify" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "inter-word" | "inter-character" | "none"
                )
            }
            _ => false,
        },
        "word-break" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal" | "break-all" | "keep-all" | "break-word"
                )
            }
            _ => false,
        },
        "overflow-wrap" | "word-wrap" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "normal" | "break-word" | "anywhere"
                )
            }
            _ => false,
        },
        "caption-side" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "top" | "bottom")
            }
            _ => false,
        },
        "border-collapse" => match value {
            CssValue::Keyword(kw) => {
                matches!(kw.to_ascii_lowercase().as_str(), "separate" | "collapse")
            }
            _ => false,
        },
        "break-inside" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "avoid" | "avoid-page" | "avoid-column" | "avoid-region"
                )
            }
            _ => false,
        },
        "pointer-events" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto"
                        | "none"
                        | "visiblepainted"
                        | "visiblefill"
                        | "visiblestroke"
                        | "visible"
                        | "painted"
                        | "fill"
                        | "stroke"
                        | "all"
                )
            }
            _ => false,
        },
        "object-fit" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "fill" | "contain" | "cover" | "none" | "scale-down"
                )
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
        "transition-timing-function" | "animation-timing-function" => match value {
            CssValue::Keyword(kw) => {
                let kw_lower = kw.to_ascii_lowercase();
                matches!(
                    kw_lower.as_str(),
                    "ease"
                        | "linear"
                        | "ease-in"
                        | "ease-out"
                        | "ease-in-out"
                        | "step-start"
                        | "step-end"
                ) || kw_lower.starts_with("cubic-bezier(")
                    || kw_lower.starts_with("steps(")
                    || kw_lower.starts_with("linear(")
            }
            _ => false,
        },
        "transition-delay" | "animation-delay" => match value {
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
        "transition-duration" | "animation-duration" => match value {
            CssValue::Keyword(kw) => {
                let kw_lower = kw.to_ascii_lowercase();
                if kw_lower.ends_with("ms") {
                    if let Ok(v) = kw_lower[..kw_lower.len() - 2].parse::<f32>() {
                        v >= 0.0
                    } else {
                        false
                    }
                } else if kw_lower.ends_with('s') {
                    if let Ok(v) = kw_lower[..kw_lower.len() - 1].parse::<f32>() {
                        v >= 0.0
                    } else {
                        false
                    }
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

#[derive(Debug, PartialEq, Clone)]
pub struct AttrValue {
    pub name: String,
    pub type_or_unit: Option<String>,
    pub fallback: Option<Box<AttrFallback>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AttrFallback {
    Value(CssValue),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ToggleValue {
    pub values: Vec<CssValue>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ScrollValue {
    pub scroller: Option<String>,
    pub axis: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ViewValue {
    pub axis: Option<String>,
    pub inset: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RepeatCount {
    Number(i32),
    AutoFill,
    AutoFit,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AdditionalTrackSize {
    MinMax(Box<AdditionalTrackSize>, Box<AdditionalTrackSize>),
    FitContent(Box<AdditionalTrackSize>),
    Repeat(RepeatCount, Vec<AdditionalTrackSize>),
    Px(f32),
    Percent(f32),
    Fr(f32),
    Auto,
    MinContent,
    MaxContent,
}

impl AdditionalTrackSize {
    pub fn format(&self) -> String {
        match self {
            AdditionalTrackSize::Px(v) => format!("{}px", v),
            AdditionalTrackSize::Percent(v) => format!("{}%", v),
            AdditionalTrackSize::Fr(v) => format!("{}fr", v),
            AdditionalTrackSize::Auto => "auto".to_string(),
            AdditionalTrackSize::MinContent => "min-content".to_string(),
            AdditionalTrackSize::MaxContent => "max-content".to_string(),
            AdditionalTrackSize::MinMax(min, max) => {
                format!("minmax({}, {})", min.format(), max.format())
            }
            AdditionalTrackSize::FitContent(limit) => {
                format!("fit-content({})", limit.format())
            }
            AdditionalTrackSize::Repeat(count, sub_tracks) => {
                let count_str = match count {
                    RepeatCount::Number(n) => n.to_string(),
                    RepeatCount::AutoFill => "auto-fill".to_string(),
                    RepeatCount::AutoFit => "auto-fit".to_string(),
                };
                let tracks_str = sub_tracks
                    .iter()
                    .map(|t| t.format())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("repeat({}, {})", count_str, tracks_str)
            }
        }
    }
}

fn parse_additional_track_size_single(comp: &ComponentValue) -> Option<AdditionalTrackSize> {
    match comp {
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let lower_unit = unit.to_ascii_lowercase();
            match lower_unit.as_str() {
                "px" => Some(AdditionalTrackSize::Px(*value as f32)),
                "em" | "rem" | "pt" | "vw" | "vh" => Some(AdditionalTrackSize::Px(*value as f32)),
                "fr" => Some(AdditionalTrackSize::Fr(*value as f32)),
                _ => None,
            }
        }
        ComponentValue::Token(CssToken::Percentage(v)) => {
            Some(AdditionalTrackSize::Percent(*v as f32))
        }
        ComponentValue::Token(CssToken::Number(v)) if *v == 0.0 => {
            Some(AdditionalTrackSize::Px(0.0))
        }
        ComponentValue::Token(CssToken::Ident(s)) => {
            let s_lower = s.to_ascii_lowercase();
            if s_lower == "auto" {
                Some(AdditionalTrackSize::Auto)
            } else if s_lower == "min-content" {
                Some(AdditionalTrackSize::MinContent)
            } else if s_lower == "max-content" {
                Some(AdditionalTrackSize::MaxContent)
            } else {
                None
            }
        }
        ComponentValue::Function { name, value } => {
            if name.eq_ignore_ascii_case("minmax") {
                let first_comma_idx = value
                    .iter()
                    .position(|c| matches!(c, ComponentValue::Token(CssToken::Comma)));
                if let Some(idx) = first_comma_idx {
                    let min_comps: Vec<&ComponentValue> = value[..idx]
                        .iter()
                        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
                        .collect();
                    let max_comps: Vec<&ComponentValue> = value[idx + 1..]
                        .iter()
                        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
                        .collect();
                    if min_comps.len() == 1 && max_comps.len() == 1 {
                        let min_size = parse_additional_track_size_single(min_comps[0])?;
                        let max_size = parse_additional_track_size_single(max_comps[0])?;
                        return Some(AdditionalTrackSize::MinMax(
                            Box::new(min_size),
                            Box::new(max_size),
                        ));
                    }
                }
                None
            } else if name.eq_ignore_ascii_case("fit-content") {
                let limit_comps: Vec<&ComponentValue> = value
                    .iter()
                    .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
                    .collect();
                if limit_comps.len() == 1 {
                    let limit_size = parse_additional_track_size_single(limit_comps[0])?;
                    if matches!(
                        limit_size,
                        AdditionalTrackSize::Px(_) | AdditionalTrackSize::Percent(_)
                    ) {
                        return Some(AdditionalTrackSize::FitContent(Box::new(limit_size)));
                    }
                }
                None
            } else if name.eq_ignore_ascii_case("repeat") {
                let first_comma_idx = value
                    .iter()
                    .position(|c| matches!(c, ComponentValue::Token(CssToken::Comma)));
                if let Some(idx) = first_comma_idx {
                    let count_comps: Vec<&ComponentValue> = value[..idx]
                        .iter()
                        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
                        .collect();
                    let track_comps = &value[idx + 1..];
                    if count_comps.len() == 1 {
                        let count = match count_comps[0] {
                            ComponentValue::Token(CssToken::Number(v)) if *v >= 1.0 => {
                                Some(RepeatCount::Number((*v).round() as i32))
                            }
                            ComponentValue::Token(CssToken::Ident(s))
                                if s.eq_ignore_ascii_case("auto-fill") =>
                            {
                                Some(RepeatCount::AutoFill)
                            }
                            ComponentValue::Token(CssToken::Ident(s))
                                if s.eq_ignore_ascii_case("auto-fit") =>
                            {
                                Some(RepeatCount::AutoFit)
                            }
                            _ => None,
                        }?;

                        let mut sub_tracks = Vec::new();
                        for comp in track_comps {
                            if matches!(comp, ComponentValue::Token(CssToken::Whitespace)) {
                                continue;
                            }
                            let sub_size = parse_additional_track_size_single(comp)?;
                            sub_tracks.push(sub_size);
                        }
                        if !sub_tracks.is_empty() {
                            return Some(AdditionalTrackSize::Repeat(count, sub_tracks));
                        }
                    }
                }
                None
            } else {
                None
            }
        }
        _ => None,
    }
}

fn format_additional_track_sizes(tracks: &[AdditionalTrackSize]) -> String {
    tracks
        .iter()
        .map(|t| t.format())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn parse_attr_function(components: &[ComponentValue]) -> Option<AttrValue> {
    let first_comma_idx = components
        .iter()
        .position(|comp| matches!(comp, ComponentValue::Token(CssToken::Comma)));

    let (main_part, fallback_part) = match first_comma_idx {
        Some(idx) => (&components[..idx], &components[idx + 1..]),
        None => (components, &[][..]),
    };

    let main_non_ws: Vec<&ComponentValue> = main_part
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();
    if main_non_ws.is_empty() || main_non_ws.len() > 2 {
        return None;
    }

    let name = match main_non_ws[0] {
        ComponentValue::Token(CssToken::Ident(s)) => s.clone(),
        _ => return None,
    };

    let type_or_unit = if main_non_ws.len() == 2 {
        match main_non_ws[1] {
            ComponentValue::Token(CssToken::Ident(s)) => Some(s.clone()),
            ComponentValue::Token(CssToken::Percentage(_)) => Some("%".to_string()),
            _ => None,
        }
    } else {
        None
    };

    let fallback = if !fallback_part.is_empty() {
        let has_non_ws = fallback_part
            .iter()
            .any(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)));
        if has_non_ws {
            Some(Box::new(AttrFallback::Value(parse_value(fallback_part)?)))
        } else {
            None
        }
    } else {
        None
    };

    Some(AttrValue {
        name,
        type_or_unit,
        fallback,
    })
}

pub fn parse_toggle_function(components: &[ComponentValue]) -> Option<ToggleValue> {
    let parts = split_components_by_comma(components);
    let mut values = Vec::new();
    for part in parts {
        let has_non_ws = part
            .iter()
            .any(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)));
        if !has_non_ws {
            return None;
        }
        let parsed = parse_value(&part)?;
        values.push(parsed);
    }
    if values.is_empty() {
        return None;
    }
    Some(ToggleValue { values })
}

fn split_components_by_comma(components: &[ComponentValue]) -> Vec<Vec<ComponentValue>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for comp in components {
        if matches!(comp, ComponentValue::Token(CssToken::Comma)) {
            parts.push(current);
            current = Vec::new();
        } else {
            current.push(comp.clone());
        }
    }
    parts.push(current);
    parts
}

pub fn parse_scroll_function(components: &[ComponentValue]) -> Option<ScrollValue> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();
    if non_ws.len() > 2 {
        return None;
    }

    let mut scroller = None;
    let mut axis = None;

    if non_ws.len() == 1 {
        match non_ws[0] {
            ComponentValue::Token(CssToken::Ident(s)) => {
                let s_lower = s.to_ascii_lowercase();
                if matches!(s_lower.as_str(), "root" | "nearest" | "self") {
                    scroller = Some(s_lower);
                } else if matches!(s_lower.as_str(), "block" | "inline" | "x" | "y") {
                    axis = Some(s_lower);
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    } else if non_ws.len() == 2 {
        let first_s = match non_ws[0] {
            ComponentValue::Token(CssToken::Ident(s)) => s.to_ascii_lowercase(),
            _ => return None,
        };
        let second_s = match non_ws[1] {
            ComponentValue::Token(CssToken::Ident(s)) => s.to_ascii_lowercase(),
            _ => return None,
        };

        if matches!(first_s.as_str(), "root" | "nearest" | "self") {
            scroller = Some(first_s);
        } else {
            return None;
        }

        if matches!(second_s.as_str(), "block" | "inline" | "x" | "y") {
            axis = Some(second_s);
        } else {
            return None;
        }
    }

    Some(ScrollValue { scroller, axis })
}

pub fn parse_view_function(components: &[ComponentValue]) -> Option<ViewValue> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();
    if non_ws.len() > 3 {
        return None;
    }

    let mut axis = None;
    let mut inset = None;

    if non_ws.len() == 1 {
        match non_ws[0] {
            ComponentValue::Token(CssToken::Ident(s)) => {
                let s_lower = s.to_ascii_lowercase();
                if matches!(s_lower.as_str(), "block" | "inline" | "x" | "y") {
                    axis = Some(s_lower);
                } else {
                    inset = Some(serialize_component_value(non_ws[0]));
                }
            }
            _ => {
                inset = Some(serialize_component_value(non_ws[0]));
            }
        }
    } else if non_ws.len() == 2 {
        let first_is_axis = match non_ws[0] {
            ComponentValue::Token(CssToken::Ident(s)) => {
                matches!(
                    s.to_ascii_lowercase().as_str(),
                    "block" | "inline" | "x" | "y"
                )
            }
            _ => false,
        };

        if first_is_axis {
            if let ComponentValue::Token(CssToken::Ident(s)) = non_ws[0] {
                axis = Some(s.to_ascii_lowercase());
            }
            inset = Some(serialize_component_value(non_ws[1]));
        } else {
            let combined = format!(
                "{} {}",
                serialize_component_value(non_ws[0]),
                serialize_component_value(non_ws[1])
            );
            inset = Some(combined);
        }
    } else if non_ws.len() == 3 {
        let first_is_axis = match non_ws[0] {
            ComponentValue::Token(CssToken::Ident(s)) => {
                matches!(
                    s.to_ascii_lowercase().as_str(),
                    "block" | "inline" | "x" | "y"
                )
            }
            _ => false,
        };
        if !first_is_axis {
            return None;
        }
        if let ComponentValue::Token(CssToken::Ident(s)) = non_ws[0] {
            axis = Some(s.to_ascii_lowercase());
        }
        let combined = format!(
            "{} {}",
            serialize_component_value(non_ws[1]),
            serialize_component_value(non_ws[2])
        );
        inset = Some(combined);
    }

    Some(ViewValue { axis, inset })
}

fn parse_grid_template(components: &[ComponentValue]) -> Option<CssValue> {
    let mut tracks = Vec::new();
    let mut has_complex = false;

    for component in components {
        if matches!(component, ComponentValue::Token(CssToken::Whitespace)) {
            continue;
        }
        if let Some(track) = parse_additional_track_size_single(component) {
            match &track {
                AdditionalTrackSize::MinMax(_, _)
                | AdditionalTrackSize::FitContent(_)
                | AdditionalTrackSize::Repeat(_, _)
                | AdditionalTrackSize::MinContent
                | AdditionalTrackSize::MaxContent => {
                    has_complex = true;
                }
                _ => {}
            }
            tracks.push(track);
        } else {
            return None;
        }
    }

    if has_complex {
        let serialized = format_additional_track_sizes(&tracks);
        Some(CssValue::Keyword(serialized))
    } else {
        let mut simple_tracks = Vec::new();
        for t in tracks {
            let simple = match t {
                AdditionalTrackSize::Px(v) => GridTrackSize::Px(v),
                AdditionalTrackSize::Percent(v) => GridTrackSize::Percent(v),
                AdditionalTrackSize::Fr(v) => GridTrackSize::Fr(v),
                AdditionalTrackSize::Auto => GridTrackSize::Auto,
                _ => unreachable!(),
            };
            simple_tracks.push(simple);
        }
        Some(CssValue::GridTemplate(simple_tracks))
    }
}

fn parse_scroll_snap_type(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for scroll-snap-type recognition
        }
    }

    if idents.is_empty() {
        return None;
    }

    if idents.len() == 1 {
        let first = idents[0].as_str();
        if first == "none" {
            return Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::None));
        }
        let axis = match first {
            "x" => ScrollSnapAxis::X,
            "y" => ScrollSnapAxis::Y,
            "block" => ScrollSnapAxis::Block,
            "inline" => ScrollSnapAxis::Inline,
            "both" => ScrollSnapAxis::Both,
            _ => return None,
        };
        // default strictness when axis is present is proximity
        return Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::Axis(
            axis,
            ScrollSnapStrictness::Proximity,
        )));
    } else if idents.len() == 2 {
        let first = idents[0].as_str();
        let second = idents[1].as_str();

        let axis = match first {
            "x" => ScrollSnapAxis::X,
            "y" => ScrollSnapAxis::Y,
            "block" => ScrollSnapAxis::Block,
            "inline" => ScrollSnapAxis::Inline,
            "both" => ScrollSnapAxis::Both,
            _ => return None,
        };

        let strictness = match second {
            "mandatory" => ScrollSnapStrictness::Mandatory,
            "proximity" => ScrollSnapStrictness::Proximity,
            _ => return None,
        };

        return Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::Axis(
            axis, strictness,
        )));
    }

    // TODO(spec): Support multi-value or global keywords like inherit/initial if required in future
    None
}

fn parse_scroll_snap_align(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for scroll-snap-align recognition
        }
    }

    if idents.is_empty() {
        return None;
    }

    let parse_kw = |s: &str| -> Option<ScrollSnapAlignKeyword> {
        match s {
            "none" => Some(ScrollSnapAlignKeyword::None),
            "start" => Some(ScrollSnapAlignKeyword::Start),
            "end" => Some(ScrollSnapAlignKeyword::End),
            "center" => Some(ScrollSnapAlignKeyword::Center),
            _ => None,
        }
    };

    if idents.len() == 1 {
        let kw = parse_kw(idents[0].as_str())?;
        // a single value applies to both axes
        return Some(CssValue::ScrollSnapAlign(ScrollSnapAlignValue {
            block: kw,
            inline: kw,
        }));
    } else if idents.len() == 2 {
        let block = parse_kw(idents[0].as_str())?;
        let inline = parse_kw(idents[1].as_str())?;
        return Some(CssValue::ScrollSnapAlign(ScrollSnapAlignValue {
            block,
            inline,
        }));
    }

    // TODO(spec): Support global keywords like inherit/initial if required in future
    None
}

fn parse_mix_blend_mode(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for mix-blend-mode recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = match idents[0].as_str() {
        "normal" => MixBlendModeValue::Normal,
        "multiply" => MixBlendModeValue::Multiply,
        "screen" => MixBlendModeValue::Screen,
        "overlay" => MixBlendModeValue::Overlay,
        "darken" => MixBlendModeValue::Darken,
        "lighten" => MixBlendModeValue::Lighten,
        "color-dodge" => MixBlendModeValue::ColorDodge,
        "color-burn" => MixBlendModeValue::ColorBurn,
        "hard-light" => MixBlendModeValue::HardLight,
        "soft-light" => MixBlendModeValue::SoftLight,
        "difference" => MixBlendModeValue::Difference,
        "exclusion" => MixBlendModeValue::Exclusion,
        "hue" => MixBlendModeValue::Hue,
        "saturation" => MixBlendModeValue::Saturation,
        "color" => MixBlendModeValue::Color,
        "luminosity" => MixBlendModeValue::Luminosity,
        _ => return None,
    };

    Some(CssValue::MixBlendMode(kw))
}

fn parse_background_blend_mode(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for background-blend-mode recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = BackgroundBlendModeValue::parse(&idents[0])?;
    Some(CssValue::BackgroundBlendMode(kw))
}

fn parse_isolation(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for isolation recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = IsolationValue::parse(&idents[0])?;
    Some(CssValue::Isolation(kw))
}

fn parse_resize(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for resize recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = ResizeValue::parse(&idents[0])?;
    Some(CssValue::Resize(kw))
}

fn parse_backface_visibility(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for backface-visibility recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = BackfaceVisibilityValue::parse(&idents[0])?;
    Some(CssValue::BackfaceVisibility(kw))
}

fn parse_empty_cells(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for empty-cells recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = EmptyCellsValue::parse(&idents[0])?;
    Some(CssValue::EmptyCells(kw))
}

fn parse_text_align_last(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for text-align-last recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = TextAlignLastValue::parse(&idents[0])?;
    Some(CssValue::TextAlignLast(kw))
}

fn parse_unicode_bidi(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for unicode-bidi recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = UnicodeBidiValue::parse(&idents[0])?;
    Some(CssValue::UnicodeBidi(kw))
}

fn parse_hyphens(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for hyphens recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = HyphensValue::parse(&idents[0])?;
    Some(CssValue::Hyphens(kw))
}

fn parse_line_break(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for line-break recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = LineBreakValue::parse(&idents[0])?;
    Some(CssValue::LineBreak(kw))
}

fn parse_text_orientation(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for text-orientation recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = TextOrientationValue::parse(&idents[0])?;
    Some(CssValue::TextOrientation(kw))
}

fn parse_box_decoration_break(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for box-decoration-break recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = BoxDecorationBreakValue::parse(&idents[0])?;
    Some(CssValue::BoxDecorationBreak(kw))
}

fn parse_mask_type(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for mask-type recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = MaskTypeValue::parse(&idents[0])?;
    Some(CssValue::MaskType(kw))
}

fn parse_scroll_behavior(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for scroll-behavior recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = ScrollBehaviorValue::parse(&idents[0])?;
    Some(CssValue::ScrollBehavior(kw))
}

fn parse_print_color_adjust(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for print-color-adjust recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = PrintColorAdjustValue::parse(&idents[0])?;
    Some(CssValue::PrintColorAdjust(kw))
}

fn parse_forced_color_adjust(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for forced-color-adjust recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = ForcedColorAdjustValue::parse(&idents[0])?;
    Some(CssValue::ForcedColorAdjust(kw))
}

fn parse_color_scheme(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for color-scheme recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = ColorSchemeValue::parse(&idents[0])?;
    Some(CssValue::ColorScheme(kw))
}

fn parse_text_rendering(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_string());
            }
            _ => return None, // invalid token for text-rendering recognition
        }
    }

    if idents.len() != 1 {
        // TODO(spec): Support global keywords like inherit/initial/unset/revert if required in future
        return None;
    }

    let kw = TextRenderingValue::parse(&idents[0])?;
    Some(CssValue::TextRendering(kw))
}

fn parse_image_rendering(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_string());
            }
            _ => return None, // invalid token for image-rendering recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = ImageRenderingValue::parse(&idents[0])?;
    Some(CssValue::ImageRendering(kw))
}

fn parse_font_variant_caps(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_string());
            }
            _ => return None, // invalid token for font-variant-caps recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = FontVariantCapsValue::parse(&idents[0])?;
    Some(CssValue::FontVariantCaps(kw))
}

fn parse_font_variant_position(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_ascii_lowercase());
            }
            _ => return None, // invalid token for font-variant-position recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = FontVariantPositionValue::parse(&idents[0])?;
    Some(CssValue::FontVariantPosition(kw))
}

fn parse_font_optical_sizing(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_string());
            }
            _ => return None, // invalid token for font-optical-sizing recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = FontOpticalSizingValue::parse(&idents[0])?;
    Some(CssValue::FontOpticalSizing(kw))
}

fn parse_font_stretch(components: &[ComponentValue]) -> Option<CssValue> {
    let mut idents = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                idents.push(s.to_string());
            }
            _ => return None, // invalid token for font-stretch recognition
        }
    }

    if idents.len() != 1 {
        return None;
    }

    let kw = FontStretchValue::parse(&idents[0])?;
    Some(CssValue::FontStretch(kw))
}

/// Parses a list of component values for a specific property, returning a typed CSS value if it matches a known layout property.
pub fn parse_property_value(
    property_name: &str,
    components: &[ComponentValue],
) -> Option<CssValue> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();
    if let [ComponentValue::Token(CssToken::Ident(s))] = non_ws.as_slice() {
        let lower = s.to_ascii_lowercase();
        if lower == "inherit"
            || lower == "initial"
            || lower == "unset"
            || lower == "revert"
            || lower == "revert-layer"
        {
            return Some(CssValue::Keyword(s.clone()));
        }
    }

    let name_lower = property_name.to_ascii_lowercase();
    if name_lower == "grid-template-columns" || name_lower == "grid-template-rows" {
        return parse_grid_template(components);
    }
    if name_lower == "scroll-snap-type" {
        return parse_scroll_snap_type(components);
    }
    if name_lower == "scroll-snap-align" {
        return parse_scroll_snap_align(components);
    }
    if name_lower == "mix-blend-mode" {
        return parse_mix_blend_mode(components);
    }
    if name_lower == "background-blend-mode" {
        return parse_background_blend_mode(components);
    }
    if name_lower == "isolation" {
        return parse_isolation(components);
    }
    if name_lower == "resize" {
        return parse_resize(components);
    }
    if name_lower == "backface-visibility" {
        return parse_backface_visibility(components);
    }
    if name_lower == "empty-cells" {
        return parse_empty_cells(components);
    }
    if name_lower == "text-align-last" {
        return parse_text_align_last(components);
    }
    if name_lower == "unicode-bidi" {
        return parse_unicode_bidi(components);
    }
    if name_lower == "hyphens" {
        return parse_hyphens(components);
    }
    if name_lower == "line-break" {
        return parse_line_break(components);
    }
    if name_lower == "text-orientation" {
        return parse_text_orientation(components);
    }
    if name_lower == "box-decoration-break" {
        return parse_box_decoration_break(components);
    }
    if name_lower == "mask-type" {
        return parse_mask_type(components);
    }
    if name_lower == "scroll-behavior" {
        return parse_scroll_behavior(components);
    }
    if name_lower == "print-color-adjust" {
        return parse_print_color_adjust(components);
    }
    if name_lower == "forced-color-adjust" {
        return parse_forced_color_adjust(components);
    }
    if name_lower == "color-scheme" {
        return parse_color_scheme(components);
    }
    if name_lower == "text-rendering" {
        return parse_text_rendering(components);
    }
    if name_lower == "image-rendering" {
        return parse_image_rendering(components);
    }
    if name_lower == "font-variant-caps" {
        return parse_font_variant_caps(components);
    }
    if name_lower == "font-variant-position" {
        return parse_font_variant_position(components);
    }
    if name_lower == "font-optical-sizing" {
        return parse_font_optical_sizing(components);
    }
    if name_lower == "font-stretch" {
        return parse_font_stretch(components);
    }
    let val = parse_value(components)?;
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
        "column-span" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "none" => ColumnSpanValue::None,
                    "all" => ColumnSpanValue::All,
                    _ => return None,
                };
                Some(CssValue::ColumnSpan(typed))
            } else {
                None
            }
        }
        "column-fill" => {
            if let CssValue::Keyword(kw) = &val {
                let typed = match kw.to_ascii_lowercase().as_str() {
                    "auto" => ColumnFillValue::Auto,
                    "balance" => ColumnFillValue::Balance,
                    "balance-all" => ColumnFillValue::BalanceAll,
                    _ => return None,
                };
                Some(CssValue::ColumnFill(typed))
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
                    "grid" => DisplayValue::Grid,
                    "inline-grid" => DisplayValue::InlineGrid,
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
        "font-kerning" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "normal" | "none" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "text-justify" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "inter-word" | "inter-character" | "none" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "word-break" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "normal" | "break-all" | "keep-all" | "break-word" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "overflow-wrap" | "word-wrap" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "normal" | "break-word" | "anywhere" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "caption-side" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "top" | "bottom" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "border-collapse" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "separate" | "collapse" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "break-inside" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "avoid" | "avoid-page" | "avoid-column" | "avoid-region" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "pointer-events" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "none" | "visiblepainted" | "visiblefill" | "visiblestroke"
                    | "visible" | "painted" | "fill" | "stroke" | "all" => Some(val),
                    _ => None,
                }
            } else {
                None
            }
        }
        "object-fit" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "fill" | "contain" | "cover" | "none" | "scale-down" => Some(val),
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
        "transition-timing-function" | "animation-timing-function" => {
            if let CssValue::Keyword(kw) = &val {
                let kw_lower = kw.to_ascii_lowercase();
                match kw_lower.as_str() {
                    "ease" | "linear" | "ease-in" | "ease-out" | "ease-in-out" | "step-start"
                    | "step-end" => Some(val),
                    _ if kw_lower.starts_with("cubic-bezier(")
                        || kw_lower.starts_with("steps(")
                        || kw_lower.starts_with("linear(") =>
                    {
                        Some(val)
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
        "transition-delay" | "animation-delay" => match &val {
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
        "transition-duration" | "animation-duration" => match &val {
            CssValue::Keyword(kw) => {
                let kw_lower = kw.to_ascii_lowercase();
                if kw_lower.ends_with("ms") {
                    if let Ok(v) = kw_lower[..kw_lower.len() - 2].parse::<f32>() {
                        if v >= 0.0 { Some(val) } else { None }
                    } else {
                        None
                    }
                } else if kw_lower.ends_with('s') {
                    if let Ok(v) = kw_lower[..kw_lower.len() - 1].parse::<f32>() {
                        if v >= 0.0 { Some(val) } else { None }
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

fn validate_math_expression(components: &[ComponentValue]) -> bool {
    for comp in components {
        match comp {
            ComponentValue::Token(token) => match token {
                CssToken::Number(_)
                | CssToken::Percentage(_)
                | CssToken::Dimension { .. }
                | CssToken::Ident(_)
                | CssToken::Whitespace
                | CssToken::Delim(_)
                | CssToken::Comma
                | CssToken::LeftParen
                | CssToken::RightParen => {}
                _ => return false,
            },
            ComponentValue::SimpleBlock { associated, value } => {
                if *associated != '(' {
                    return false;
                }
                if !validate_math_expression(value) {
                    return false;
                }
            }
            ComponentValue::Function { name, value } => {
                let name_lower = name.to_ascii_lowercase();
                if name_lower == "calc"
                    || name_lower == "min"
                    || name_lower == "max"
                    || name_lower == "clamp"
                    || name_lower == "round"
                    || name_lower == "mod"
                    || name_lower == "rem"
                    || name_lower == "abs"
                    || name_lower == "sign"
                    || name_lower == "sin"
                    || name_lower == "cos"
                    || name_lower == "tan"
                    || name_lower == "asin"
                    || name_lower == "acos"
                    || name_lower == "atan"
                    || name_lower == "atan2"
                    || name_lower == "pow"
                    || name_lower == "sqrt"
                    || name_lower == "hypot"
                    || name_lower == "log"
                    || name_lower == "exp"
                    || name_lower == "var"
                    || name_lower == "env"
                    || name_lower == "calc-size"
                {
                    if !validate_math_expression(value) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
    true
}

#[derive(Debug, Clone, PartialEq)]
enum RoundStrategy {
    Nearest,
    Up,
    Down,
    ToZero,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MathFunc {
    Calc,
    Min,
    Max,
    Clamp,
    Abs,
    Sign,
    Mod,
    Rem,
    Sqrt,
    Pow,
    Hypot,
    Log,
    Exp,
}

#[derive(Debug, Clone, PartialEq)]
enum MathExpr {
    Length(f32, LengthUnit),
    Number(f32),
    Op(Box<MathExpr>, MathOp, Box<MathExpr>),
    Round(RoundStrategy, Box<MathExpr>, Box<MathExpr>),
    Func(MathFunc, Vec<MathExpr>),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
enum MathToken {
    Expr(MathExpr),
    Op(MathOp),
    LeftParen,
    RightParen,
    Comma,
}

struct MathParser {
    tokens: Vec<MathToken>,
    pos: usize,
}

impl MathParser {
    fn peek(&self) -> Option<&MathToken> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<&MathToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn parse_expr(&mut self, min_bp: u8) -> Option<MathExpr> {
        let mut lhs = match self.consume()? {
            MathToken::Expr(expr) => expr.clone(),
            MathToken::LeftParen => {
                let sub = self.parse_expr(0)?;
                if !matches!(self.consume()?, MathToken::RightParen) {
                    return None;
                }
                sub
            }
            _ => return None,
        };

        while let Some(tok) = self.peek() {
            let op = match tok {
                MathToken::Op(op) => *op,
                _ => break,
            };

            let (l_bp, r_bp) = match op {
                MathOp::Add | MathOp::Sub => (1, 2),
                MathOp::Mul | MathOp::Div => (3, 4),
            };

            if l_bp < min_bp {
                break;
            }

            self.consume(); // consume op
            let rhs = self.parse_expr(r_bp)?;
            lhs = MathExpr::Op(Box::new(lhs), op, Box::new(rhs));
        }

        Some(lhs)
    }
}

fn parse_comma_separated_components(
    components: &[ComponentValue],
) -> Option<Vec<Vec<ComponentValue>>> {
    let mut args = Vec::new();
    let mut current = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Token(CssToken::Comma) => {
                if current.is_empty() {
                    return None;
                }
                args.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(comp.clone());
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() { None } else { Some(args) }
}

fn parse_round_arguments(components: &[ComponentValue]) -> Option<MathExpr> {
    let args_raw = parse_comma_separated_components(components)?;
    if args_raw.len() == 3 {
        let strategy = match &args_raw[0][..] {
            [ComponentValue::Token(CssToken::Ident(s))] => match s.to_ascii_lowercase().as_str() {
                "nearest" => RoundStrategy::Nearest,
                "up" => RoundStrategy::Up,
                "down" => RoundStrategy::Down,
                "to-zero" => RoundStrategy::ToZero,
                _ => return None,
            },
            _ => return None,
        };
        let a = parse_math_expr(&args_raw[1])?;
        let b = parse_math_expr(&args_raw[2])?;
        Some(MathExpr::Round(strategy, Box::new(a), Box::new(b)))
    } else if args_raw.len() == 2 {
        let a = parse_math_expr(&args_raw[0])?;
        let b = parse_math_expr(&args_raw[1])?;
        Some(MathExpr::Round(
            RoundStrategy::Nearest,
            Box::new(a),
            Box::new(b),
        ))
    } else {
        None
    }
}

fn parse_function_arguments(components: &[ComponentValue]) -> Option<Vec<MathExpr>> {
    let args_raw = parse_comma_separated_components(components)?;
    let mut args = Vec::new();
    for arg_comp in args_raw {
        args.push(parse_math_expr(&arg_comp)?);
    }
    Some(args)
}

fn parse_math_tokens(components: &[ComponentValue]) -> Option<Vec<MathToken>> {
    let mut tokens = Vec::new();
    for comp in components {
        match comp {
            ComponentValue::Token(token) => match token {
                CssToken::Whitespace => {}
                CssToken::Number(v) => {
                    tokens.push(MathToken::Expr(MathExpr::Number(*v as f32)));
                }
                CssToken::Percentage(v) => {
                    tokens.push(MathToken::Expr(MathExpr::Length(
                        *v as f32,
                        LengthUnit::Percent,
                    )));
                }
                CssToken::Dimension { value, unit } => {
                    let lower_unit = unit.to_ascii_lowercase();
                    let (val, unit_enum) = match lower_unit.as_str() {
                        "px" => (*value as f32, LengthUnit::Px),
                        "em" => (*value as f32, LengthUnit::Em),
                        "rem" => (*value as f32, LengthUnit::Rem),
                        "pt" => (*value as f32 * 96.0 / 72.0, LengthUnit::Px),
                        "vw" => (*value as f32, LengthUnit::Vw),
                        "vh" => (*value as f32, LengthUnit::Vh),
                        "in" => (*value as f32 * 96.0, LengthUnit::Px),
                        "cm" => (*value as f32 * 96.0 / 2.54, LengthUnit::Px),
                        "mm" => (*value as f32 * 9.6 / 2.54, LengthUnit::Px),
                        "pc" => (*value as f32 * 16.0, LengthUnit::Px),
                        "q" => (*value as f32 * 96.0 / 101.6, LengthUnit::Px),
                        _ => return None,
                    };
                    tokens.push(MathToken::Expr(MathExpr::Length(val, unit_enum)));
                }
                CssToken::Delim('+') => tokens.push(MathToken::Op(MathOp::Add)),
                CssToken::Delim('-') => tokens.push(MathToken::Op(MathOp::Sub)),
                CssToken::Delim('*') => tokens.push(MathToken::Op(MathOp::Mul)),
                CssToken::Delim('/') => tokens.push(MathToken::Op(MathOp::Div)),
                CssToken::Comma => tokens.push(MathToken::Comma),
                CssToken::LeftParen => tokens.push(MathToken::LeftParen),
                CssToken::RightParen => tokens.push(MathToken::RightParen),
                _ => {
                    tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                        comp,
                    ))));
                }
            },
            ComponentValue::SimpleBlock { associated, value } => {
                if *associated == '(' {
                    tokens.push(MathToken::LeftParen);
                    let sub_tokens = parse_math_tokens(value)?;
                    tokens.extend(sub_tokens);
                    tokens.push(MathToken::RightParen);
                } else {
                    tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                        comp,
                    ))));
                }
            }
            ComponentValue::Function { name, value } => {
                let name_lower = name.to_ascii_lowercase();
                if name_lower == "calc" {
                    if let Some(inner) = parse_math_expr(value) {
                        tokens.push(MathToken::Expr(inner));
                    } else {
                        tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                            comp,
                        ))));
                    }
                } else if name_lower == "round" {
                    if let Some(inner) = parse_round_arguments(value) {
                        tokens.push(MathToken::Expr(inner));
                    } else {
                        tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                            comp,
                        ))));
                    }
                } else if let Some(func) = match name_lower.as_str() {
                    "min" => Some(MathFunc::Min),
                    "max" => Some(MathFunc::Max),
                    "clamp" => Some(MathFunc::Clamp),
                    "abs" => Some(MathFunc::Abs),
                    "sign" => Some(MathFunc::Sign),
                    "mod" => Some(MathFunc::Mod),
                    "rem" => Some(MathFunc::Rem),
                    "sqrt" => Some(MathFunc::Sqrt),
                    "pow" => Some(MathFunc::Pow),
                    "hypot" => Some(MathFunc::Hypot),
                    "log" => Some(MathFunc::Log),
                    "exp" => Some(MathFunc::Exp),
                    _ => None,
                } {
                    if let Some(args) = parse_function_arguments(value) {
                        let ok = match func {
                            MathFunc::Min | MathFunc::Max => !args.is_empty(),
                            MathFunc::Clamp => args.len() == 3,
                            MathFunc::Abs | MathFunc::Sign | MathFunc::Sqrt | MathFunc::Exp => {
                                args.len() == 1
                            }
                            MathFunc::Mod | MathFunc::Rem | MathFunc::Pow => args.len() == 2,
                            MathFunc::Hypot => !args.is_empty(),
                            MathFunc::Log => args.len() == 1 || args.len() == 2,
                            _ => false,
                        };
                        if ok {
                            tokens.push(MathToken::Expr(MathExpr::Func(func, args)));
                        } else {
                            tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                                comp,
                            ))));
                        }
                    } else {
                        tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                            comp,
                        ))));
                    }
                } else {
                    tokens.push(MathToken::Expr(MathExpr::Raw(serialize_component_value(
                        comp,
                    ))));
                }
            }
        }
    }
    Some(tokens)
}

fn parse_math_expr(components: &[ComponentValue]) -> Option<MathExpr> {
    let tokens = parse_math_tokens(components)?;
    let mut parser = MathParser { tokens, pos: 0 };
    let expr = parser.parse_expr(0)?;
    if parser.pos != parser.tokens.len() {
        None
    } else {
        Some(expr)
    }
}

fn flatten_sum(expr: &MathExpr, multiplier: f32, flat_terms: &mut Vec<(MathExpr, f32)>) {
    match expr {
        MathExpr::Op(lhs, MathOp::Add, rhs) => {
            flatten_sum(lhs, multiplier, flat_terms);
            flatten_sum(rhs, multiplier, flat_terms);
        }
        MathExpr::Op(lhs, MathOp::Sub, rhs) => {
            flatten_sum(lhs, multiplier, flat_terms);
            flatten_sum(rhs, -multiplier, flat_terms);
        }
        _ => {
            let simp = expr.simplify();
            match simp {
                MathExpr::Op(lhs, MathOp::Add, rhs) => {
                    flatten_sum(&lhs, multiplier, flat_terms);
                    flatten_sum(&rhs, multiplier, flat_terms);
                }
                MathExpr::Op(lhs, MathOp::Sub, rhs) => {
                    flatten_sum(&lhs, multiplier, flat_terms);
                    flatten_sum(&rhs, -multiplier, flat_terms);
                }
                other => {
                    flat_terms.push((other, multiplier));
                }
            }
        }
    }
}

fn simplify_sum_expr(lhs: &MathExpr, op: MathOp, rhs: &MathExpr) -> MathExpr {
    let mut flat_terms = Vec::new();
    flatten_sum(lhs, 1.0, &mut flat_terms);
    let right_mult = match op {
        MathOp::Add => 1.0,
        MathOp::Sub => -1.0,
        _ => unreachable!(),
    };
    flatten_sum(rhs, right_mult, &mut flat_terms);

    let mut lengths: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
    let mut unit_order: Vec<String> = Vec::new();
    let mut number_sum: f32 = 0.0;
    let mut unresolvable: Vec<(MathExpr, f32)> = Vec::new();
    let mut has_length = false;

    for (term, mult) in flat_terms {
        match term {
            MathExpr::Length(v, u) => {
                has_length = true;
                let unit_name = match u {
                    LengthUnit::Px => "px",
                    LengthUnit::Em => "em",
                    LengthUnit::Rem => "rem",
                    LengthUnit::Pt => "pt",
                    LengthUnit::Percent => "%",
                    LengthUnit::Vw => "vw",
                    LengthUnit::Vh => "vh",
                }
                .to_string();
                if !lengths.contains_key(&unit_name) {
                    unit_order.push(unit_name.clone());
                }
                *lengths.entry(unit_name).or_insert(0.0) += v * mult;
            }
            MathExpr::Number(n) => {
                number_sum += n * mult;
            }
            other => {
                unresolvable.push((other, mult));
            }
        }
    }

    let mut recon_terms: Vec<(MathExpr, f32)> = Vec::new();

    for unit_str in unit_order {
        if let Some(&sum) = lengths.get(&unit_str).filter(|&&sum| sum != 0.0) {
            let u = match unit_str.as_str() {
                "px" => LengthUnit::Px,
                "em" => LengthUnit::Em,
                "rem" => LengthUnit::Rem,
                "pt" => LengthUnit::Pt,
                "%" => LengthUnit::Percent,
                "vw" => LengthUnit::Vw,
                "vh" => LengthUnit::Vh,
                _ => continue,
            };
            if sum < 0.0 {
                recon_terms.push((MathExpr::Length(-sum, u), -1.0));
            } else {
                recon_terms.push((MathExpr::Length(sum, u), 1.0));
            }
        }
    }

    if number_sum != 0.0 {
        if number_sum < 0.0 {
            recon_terms.push((MathExpr::Number(-number_sum), -1.0));
        } else {
            recon_terms.push((MathExpr::Number(number_sum), 1.0));
        }
    }

    for (expr, mult) in unresolvable {
        if mult < 0.0 {
            recon_terms.push((expr, -1.0));
        } else {
            recon_terms.push((expr, 1.0));
        }
    }

    if recon_terms.is_empty() {
        if has_length {
            return MathExpr::Length(0.0, LengthUnit::Px);
        } else {
            return MathExpr::Number(0.0);
        }
    }

    // Sort to put a positive term first if possible
    if let Some(pos_idx) = recon_terms
        .iter()
        .position(|(_, mult)| *mult > 0.0)
        .filter(|&idx| idx > 0)
    {
        let item = recon_terms.remove(pos_idx);
        recon_terms.insert(0, item);
    }

    let (mut current_expr, first_mult) = recon_terms.remove(0);
    if first_mult < 0.0 {
        match current_expr {
            MathExpr::Length(v, u) => {
                current_expr = MathExpr::Length(-v, u);
            }
            MathExpr::Number(v) => {
                current_expr = MathExpr::Number(-v);
            }
            _ => {
                current_expr = MathExpr::Op(
                    Box::new(MathExpr::Number(0.0)),
                    MathOp::Sub,
                    Box::new(current_expr),
                );
            }
        }
    }

    for (expr, mult) in recon_terms {
        if mult < 0.0 {
            current_expr = MathExpr::Op(Box::new(current_expr), MathOp::Sub, Box::new(expr));
        } else {
            current_expr = MathExpr::Op(Box::new(current_expr), MathOp::Add, Box::new(expr));
        }
    }

    current_expr
}

impl MathExpr {
    fn simplify(&self) -> MathExpr {
        match self {
            MathExpr::Length(v, u) => MathExpr::Length(*v, u.clone()),
            MathExpr::Number(v) => MathExpr::Number(*v),
            MathExpr::Raw(s) => MathExpr::Raw(s.clone()),
            MathExpr::Op(lhs, op, rhs) => {
                let s_lhs = lhs.simplify();
                let s_rhs = rhs.simplify();
                match op {
                    MathOp::Add | MathOp::Sub => simplify_sum_expr(&s_lhs, *op, &s_rhs),
                    MathOp::Mul => match (&s_lhs, &s_rhs) {
                        (MathExpr::Length(v, u), MathExpr::Number(n))
                        | (MathExpr::Number(n), MathExpr::Length(v, u)) => {
                            MathExpr::Length(v * n, u.clone())
                        }
                        (MathExpr::Number(n1), MathExpr::Number(n2)) => MathExpr::Number(n1 * n2),
                        _ => MathExpr::Op(Box::new(s_lhs), *op, Box::new(s_rhs)),
                    },
                    MathOp::Div => match (&s_lhs, &s_rhs) {
                        (MathExpr::Length(v, u), MathExpr::Number(n)) => {
                            if *n != 0.0 {
                                MathExpr::Length(v / n, u.clone())
                            } else {
                                MathExpr::Op(Box::new(s_lhs), *op, Box::new(s_rhs))
                            }
                        }
                        (MathExpr::Number(n1), MathExpr::Number(n2)) => {
                            if *n2 != 0.0 {
                                MathExpr::Number(n1 / n2)
                            } else {
                                MathExpr::Op(Box::new(s_lhs), *op, Box::new(s_rhs))
                            }
                        }
                        _ => MathExpr::Op(Box::new(s_lhs), *op, Box::new(s_rhs)),
                    },
                }
            }
            MathExpr::Round(strategy, a, b) => {
                let s_a = a.simplify();
                let s_b = b.simplify();
                match (&s_a, &s_b) {
                    (MathExpr::Number(va), MathExpr::Number(vb)) => {
                        if *vb != 0.0 {
                            let ratio = va / vb;
                            let rounded = match strategy {
                                RoundStrategy::Nearest => ratio.round(),
                                RoundStrategy::Up => ratio.ceil(),
                                RoundStrategy::Down => ratio.floor(),
                                RoundStrategy::ToZero => ratio.trunc(),
                            };
                            MathExpr::Number(rounded * vb)
                        } else {
                            MathExpr::Round(strategy.clone(), Box::new(s_a), Box::new(s_b))
                        }
                    }
                    (MathExpr::Length(va, ua), MathExpr::Length(vb, ub)) if ua == ub => {
                        if *vb != 0.0 {
                            let ratio = va / vb;
                            let rounded = match strategy {
                                RoundStrategy::Nearest => ratio.round(),
                                RoundStrategy::Up => ratio.ceil(),
                                RoundStrategy::Down => ratio.floor(),
                                RoundStrategy::ToZero => ratio.trunc(),
                            };
                            MathExpr::Length(rounded * vb, ua.clone())
                        } else {
                            MathExpr::Round(strategy.clone(), Box::new(s_a), Box::new(s_b))
                        }
                    }
                    _ => MathExpr::Round(strategy.clone(), Box::new(s_a), Box::new(s_b)),
                }
            }
            MathExpr::Func(func, args) => {
                let s_args: Vec<MathExpr> = args.iter().map(|arg| arg.simplify()).collect();
                match func {
                    MathFunc::Calc => {
                        if s_args.len() == 1 {
                            s_args[0].clone()
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Min => {
                        if s_args.iter().all(|a| matches!(a, MathExpr::Number(_))) {
                            let mut min_val = f32::INFINITY;
                            for arg in &s_args {
                                if let MathExpr::Number(v) = arg {
                                    min_val = min_val.min(*v);
                                }
                            }
                            MathExpr::Number(min_val)
                        } else if s_args.is_empty() {
                            MathExpr::Func(*func, s_args)
                        } else {
                            let first_unit = match &s_args[0] {
                                MathExpr::Length(_, u) => Some(u.clone()),
                                _ => None,
                            };
                            if let Some(ref u) = first_unit {
                                let all_same = s_args.iter().all(|a| match a {
                                    MathExpr::Length(_, unit) => unit == u,
                                    _ => false,
                                });
                                if all_same {
                                    let mut min_val = f32::INFINITY;
                                    for arg in &s_args {
                                        if let MathExpr::Length(v, _) = arg {
                                            min_val = min_val.min(*v);
                                        }
                                    }
                                    MathExpr::Length(min_val, u.clone())
                                } else {
                                    MathExpr::Func(*func, s_args)
                                }
                            } else {
                                MathExpr::Func(*func, s_args)
                            }
                        }
                    }
                    MathFunc::Max => {
                        if s_args.iter().all(|a| matches!(a, MathExpr::Number(_))) {
                            let mut max_val = f32::NEG_INFINITY;
                            for arg in &s_args {
                                if let MathExpr::Number(v) = arg {
                                    max_val = max_val.max(*v);
                                }
                            }
                            MathExpr::Number(max_val)
                        } else if s_args.is_empty() {
                            MathExpr::Func(*func, s_args)
                        } else {
                            let first_unit = match &s_args[0] {
                                MathExpr::Length(_, u) => Some(u.clone()),
                                _ => None,
                            };
                            if let Some(ref u) = first_unit {
                                let all_same = s_args.iter().all(|a| match a {
                                    MathExpr::Length(_, unit) => unit == u,
                                    _ => false,
                                });
                                if all_same {
                                    let mut max_val = f32::NEG_INFINITY;
                                    for arg in &s_args {
                                        if let MathExpr::Length(v, _) = arg {
                                            max_val = max_val.max(*v);
                                        }
                                    }
                                    MathExpr::Length(max_val, u.clone())
                                } else {
                                    MathExpr::Func(*func, s_args)
                                }
                            } else {
                                MathExpr::Func(*func, s_args)
                            }
                        }
                    }
                    MathFunc::Clamp => {
                        if s_args.len() == 3 {
                            match (&s_args[0], &s_args[1], &s_args[2]) {
                                (
                                    MathExpr::Number(min),
                                    MathExpr::Number(val),
                                    MathExpr::Number(max),
                                ) => {
                                    let clamped = min.max(val.min(*max));
                                    MathExpr::Number(clamped)
                                }
                                (
                                    MathExpr::Length(min, u1),
                                    MathExpr::Length(val, u2),
                                    MathExpr::Length(max, u3),
                                ) if u1 == u2 && u2 == u3 => {
                                    let clamped = min.max(val.min(*max));
                                    MathExpr::Length(clamped, u1.clone())
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Abs => {
                        if s_args.len() == 1 {
                            match &s_args[0] {
                                MathExpr::Number(v) => MathExpr::Number(v.abs()),
                                MathExpr::Length(v, u) => MathExpr::Length(v.abs(), u.clone()),
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Sign => {
                        if s_args.len() == 1 {
                            match &s_args[0] {
                                MathExpr::Number(v) => {
                                    let s = if *v > 0.0 {
                                        1.0
                                    } else if *v < 0.0 {
                                        -1.0
                                    } else {
                                        0.0
                                    };
                                    MathExpr::Number(s)
                                }
                                MathExpr::Length(v, _) => {
                                    let s = if *v > 0.0 {
                                        1.0
                                    } else if *v < 0.0 {
                                        -1.0
                                    } else {
                                        0.0
                                    };
                                    MathExpr::Number(s)
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Mod => {
                        if s_args.len() == 2 {
                            match (&s_args[0], &s_args[1]) {
                                (MathExpr::Number(va), MathExpr::Number(vb)) => {
                                    if *vb != 0.0 {
                                        MathExpr::Number(va - vb * (va / vb).floor())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                (MathExpr::Length(va, ua), MathExpr::Length(vb, ub))
                                    if ua == ub =>
                                {
                                    if *vb != 0.0 {
                                        MathExpr::Length(va - vb * (va / vb).floor(), ua.clone())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Rem => {
                        if s_args.len() == 2 {
                            match (&s_args[0], &s_args[1]) {
                                (MathExpr::Number(va), MathExpr::Number(vb)) => {
                                    if *vb != 0.0 {
                                        MathExpr::Number(va - vb * (va / vb).trunc())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                (MathExpr::Length(va, ua), MathExpr::Length(vb, ub))
                                    if ua == ub =>
                                {
                                    if *vb != 0.0 {
                                        MathExpr::Length(va - vb * (va / vb).trunc(), ua.clone())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Sqrt => {
                        if s_args.len() == 1 {
                            match &s_args[0] {
                                MathExpr::Number(v) => {
                                    if *v >= 0.0 {
                                        MathExpr::Number(v.sqrt())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Pow => {
                        if s_args.len() == 2 {
                            match (&s_args[0], &s_args[1]) {
                                (MathExpr::Number(va), MathExpr::Number(vb)) => {
                                    MathExpr::Number(va.powf(*vb))
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Hypot => {
                        if s_args.iter().all(|a| matches!(a, MathExpr::Number(_))) {
                            let mut sum_sq = 0.0;
                            for arg in &s_args {
                                if let MathExpr::Number(v) = arg {
                                    sum_sq += v * v;
                                }
                            }
                            MathExpr::Number(sum_sq.sqrt())
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Log => {
                        if s_args.len() == 1 {
                            match &s_args[0] {
                                MathExpr::Number(v) => {
                                    if *v > 0.0 {
                                        MathExpr::Number(v.ln())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else if s_args.len() == 2 {
                            match (&s_args[0], &s_args[1]) {
                                (MathExpr::Number(va), MathExpr::Number(vb)) => {
                                    if *va > 0.0 && *vb > 0.0 && *vb != 1.0 {
                                        MathExpr::Number(va.ln() / vb.ln())
                                    } else {
                                        MathExpr::Func(*func, s_args)
                                    }
                                }
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                    MathFunc::Exp => {
                        if s_args.len() == 1 {
                            match &s_args[0] {
                                MathExpr::Number(v) => MathExpr::Number(v.exp()),
                                _ => MathExpr::Func(*func, s_args),
                            }
                        } else {
                            MathExpr::Func(*func, s_args)
                        }
                    }
                }
            }
        }
    }

    fn serialize(&self) -> String {
        match self {
            MathExpr::Length(v, u) => {
                let unit_str = match u {
                    LengthUnit::Px => "px",
                    LengthUnit::Em => "em",
                    LengthUnit::Rem => "rem",
                    LengthUnit::Pt => "pt",
                    LengthUnit::Percent => "%",
                    LengthUnit::Vw => "vw",
                    LengthUnit::Vh => "vh",
                };
                format!("{}{}", v, unit_str)
            }
            MathExpr::Number(v) => v.to_string(),
            MathExpr::Raw(s) => s.clone(),
            MathExpr::Op(lhs, op, rhs) => {
                let op_str = match op {
                    MathOp::Add => " + ",
                    MathOp::Sub => " - ",
                    MathOp::Mul => " * ",
                    MathOp::Div => " / ",
                };
                format!("({}{}{})", lhs.serialize(), op_str, rhs.serialize())
            }
            MathExpr::Round(strategy, a, b) => {
                let strat_str = match strategy {
                    RoundStrategy::Nearest => "nearest",
                    RoundStrategy::Up => "up",
                    RoundStrategy::Down => "down",
                    RoundStrategy::ToZero => "to-zero",
                };
                format!("round({},{},{})", strat_str, a.serialize(), b.serialize())
            }
            MathExpr::Func(func, args) => {
                let func_name = match func {
                    MathFunc::Calc => "calc",
                    MathFunc::Min => "min",
                    MathFunc::Max => "max",
                    MathFunc::Clamp => "clamp",
                    MathFunc::Abs => "abs",
                    MathFunc::Sign => "sign",
                    MathFunc::Mod => "mod",
                    MathFunc::Rem => "rem",
                    MathFunc::Sqrt => "sqrt",
                    MathFunc::Pow => "pow",
                    MathFunc::Hypot => "hypot",
                    MathFunc::Log => "log",
                    MathFunc::Exp => "exp",
                };
                let serialized_args: Vec<String> = args.iter().map(|a| a.serialize()).collect();
                format!("{}({})", func_name, serialized_args.join(","))
            }
        }
    }
}

fn serialize_top_level_math(expr: &MathExpr) -> String {
    match expr {
        MathExpr::Op(lhs, op, rhs) => {
            let op_str = match op {
                MathOp::Add => " + ",
                MathOp::Sub => " - ",
                MathOp::Mul => " * ",
                MathOp::Div => " / ",
            };
            format!("calc({}{}{})", lhs.serialize(), op_str, rhs.serialize())
        }
        MathExpr::Round(strategy, a, b) => {
            let strat_str = match strategy {
                RoundStrategy::Nearest => "nearest",
                RoundStrategy::Up => "up",
                RoundStrategy::Down => "down",
                RoundStrategy::ToZero => "to-zero",
            };
            format!("round({},{},{})", strat_str, a.serialize(), b.serialize())
        }
        MathExpr::Func(func, args) => {
            let func_name = match func {
                MathFunc::Calc => "calc",
                MathFunc::Min => "min",
                MathFunc::Max => "max",
                MathFunc::Clamp => "clamp",
                MathFunc::Abs => "abs",
                MathFunc::Sign => "sign",
                MathFunc::Mod => "mod",
                MathFunc::Rem => "rem",
                MathFunc::Sqrt => "sqrt",
                MathFunc::Pow => "pow",
                MathFunc::Hypot => "hypot",
                MathFunc::Log => "log",
                MathFunc::Exp => "exp",
            };
            let serialized_args: Vec<String> = args.iter().map(|a| a.serialize()).collect();
            format!("{}({})", func_name, serialized_args.join(","))
        }
        _ => expr.serialize(),
    }
}

fn parse_calc_function(
    _name: &str,
    value: &[ComponentValue],
    orig_comp: &ComponentValue,
) -> Option<CssValue> {
    if !validate_math_expression(value) {
        return None;
    }
    if let Some(expr) = parse_math_expr(std::slice::from_ref(orig_comp)) {
        let simplified = expr.simplify();
        match simplified {
            MathExpr::Length(v, u) => Some(CssValue::Length(v, u)),
            MathExpr::Number(v) => Some(CssValue::Number(v)),
            _ => {
                let serialized = serialize_top_level_math(&simplified);
                Some(CssValue::Keyword(serialized))
            }
        }
    } else {
        Some(CssValue::Keyword(serialize_component_value(orig_comp)))
    }
}

fn parse_env_function(components: &[ComponentValue]) -> Option<CssValue> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    if non_ws.is_empty() {
        return None;
    }

    match non_ws[0] {
        ComponentValue::Token(CssToken::Ident(_)) => {}
        _ => return None,
    }

    if non_ws.len() > 1 {
        match non_ws[1] {
            ComponentValue::Token(CssToken::Comma) => {}
            _ => return None,
        }

        if non_ws.len() < 3 {
            return None;
        }
    }

    let mut serialized = "env(".to_string();
    for comp in components {
        serialized.push_str(&serialize_component_value(comp));
    }
    serialized.push(')');
    Some(CssValue::Keyword(serialized))
}

fn parse_var_function(components: &[ComponentValue]) -> Option<CssValue> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    if non_ws.is_empty() {
        return None;
    }

    let prop_name = match non_ws[0] {
        ComponentValue::Token(CssToken::Ident(s)) => s,
        _ => return None,
    };

    if !prop_name.starts_with("--") {
        return None;
    }

    if non_ws.len() > 1 {
        match non_ws[1] {
            ComponentValue::Token(CssToken::Comma) => {}
            _ => return None,
        }
    }

    let mut serialized = "var(".to_string();
    for comp in components {
        serialized.push_str(&serialize_component_value(comp));
    }
    serialized.push(')');
    Some(CssValue::Keyword(serialized))
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
            if lower_unit == "s"
                || lower_unit == "ms"
                || lower_unit == "fr"
                || lower_unit == "ex"
                || lower_unit == "ch"
                || lower_unit == "vmin"
                || lower_unit == "vmax"
                || lower_unit == "svw"
                || lower_unit == "svh"
                || lower_unit == "lvw"
                || lower_unit == "lvh"
                || lower_unit == "dvw"
                || lower_unit == "dvh"
                || lower_unit == "svmin"
                || lower_unit == "svmax"
                || lower_unit == "lvmin"
                || lower_unit == "lvmax"
                || lower_unit == "dvmin"
                || lower_unit == "dvmax"
                || lower_unit == "vi"
                || lower_unit == "svi"
                || lower_unit == "lvi"
                || lower_unit == "dvi"
                || lower_unit == "vb"
                || lower_unit == "svb"
                || lower_unit == "lvb"
                || lower_unit == "dvb"
                || lower_unit == "rex"
                || lower_unit == "rch"
                || lower_unit == "ric"
                || lower_unit == "rcap"
                || lower_unit == "ic"
                || lower_unit == "cap"
                || lower_unit == "lh"
                || lower_unit == "rlh"
                || lower_unit == "deg"
                || lower_unit == "rad"
                || lower_unit == "grad"
                || lower_unit == "turn"
                || lower_unit == "dpi"
                || lower_unit == "dpcm"
                || lower_unit == "dppx"
                || lower_unit == "x"
                || lower_unit == "cqw"
                || lower_unit == "cqh"
                || lower_unit == "cqi"
                || lower_unit == "cqb"
                || lower_unit == "cqmin"
                || lower_unit == "cqmax"
            {
                return Some(CssValue::Keyword(format!("{}{}", value, lower_unit)));
            }
            let (val, unit_enum) = match lower_unit.as_str() {
                "px" => (*value as f32, LengthUnit::Px),
                "em" => (*value as f32, LengthUnit::Em),
                "rem" => (*value as f32, LengthUnit::Rem),
                "pt" => (*value as f32, LengthUnit::Pt),
                "vw" => (*value as f32, LengthUnit::Vw),
                "vh" => (*value as f32, LengthUnit::Vh),
                "in" => (*value as f32 * 96.0, LengthUnit::Px),
                "cm" => (*value as f32 * 96.0 / 2.54, LengthUnit::Px),
                "mm" => (*value as f32 * 9.6 / 2.54, LengthUnit::Px),
                "pc" => (*value as f32 * 16.0, LengthUnit::Px),
                "q" => (*value as f32 * 96.0 / 101.6, LengthUnit::Px),
                _ => return None, // TODO(spec): other units
            };
            Some(CssValue::Length(val, unit_enum))
        }
        ComponentValue::Token(CssToken::Percentage(v)) => {
            Some(CssValue::Length(*v as f32, LengthUnit::Percent))
        }
        ComponentValue::Token(CssToken::Number(v)) => Some(CssValue::Number(*v as f32)),
        ComponentValue::Token(CssToken::Hash(s)) => parse_hex_color(s).map(CssValue::Color),
        ComponentValue::Token(CssToken::Delim('/')) => Some(CssValue::Keyword("/".to_string())),
        ComponentValue::Token(CssToken::Url(s)) => Some(CssValue::Keyword(format!("url({})", s))),
        ComponentValue::Function { name, value } => {
            if name.eq_ignore_ascii_case("cubic-bezier") {
                return parse_cubic_bezier_function(value);
            }
            if name.eq_ignore_ascii_case("steps") {
                return parse_steps_function(value);
            }
            if name.eq_ignore_ascii_case("linear") {
                return parse_linear_function(value);
            }
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") {
                return parse_rgb_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") {
                return parse_hsl_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("hwb") {
                return parse_hwb_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("oklab") {
                return parse_oklab_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("oklch") {
                return parse_oklch_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("lab") {
                return parse_lab_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("lch") {
                return parse_lch_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("color") {
                return parse_color_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("device-cmyk") {
                return parse_device_cmyk_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("color-mix") {
                return parse_color_mix_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("light-dark") {
                return parse_light_dark_function(value).map(CssValue::Color);
            }
            if name.eq_ignore_ascii_case("cross-fade") {
                return parse_cross_fade_function(value);
            }
            if name.eq_ignore_ascii_case("color-contrast") {
                return parse_color_contrast_function(value).map(CssValue::Color);
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
            if name.eq_ignore_ascii_case("image-set")
                || name.eq_ignore_ascii_case("-webkit-image-set")
            {
                return parse_image_set_function(value);
            }
            if name.eq_ignore_ascii_case("linear-gradient")
                || name.eq_ignore_ascii_case("radial-gradient")
                || name.eq_ignore_ascii_case("conic-gradient")
            {
                return Some(CssValue::Keyword(serialize_component_value(components[0])));
            }
            if name.eq_ignore_ascii_case("anchor") {
                return parse_anchor_function(value);
            }
            if name.eq_ignore_ascii_case("anchor-size") {
                return parse_anchor_size_function(value);
            }
            if name.eq_ignore_ascii_case("attr") {
                if parse_attr_function(value).is_some() {
                    return Some(CssValue::Keyword(serialize_component_value(components[0])));
                }
                return None;
            }
            if name.eq_ignore_ascii_case("toggle") {
                if parse_toggle_function(value).is_some() {
                    return Some(CssValue::Keyword(serialize_component_value(components[0])));
                }
                return None;
            }
            if name.eq_ignore_ascii_case("scroll") {
                if parse_scroll_function(value).is_some() {
                    return Some(CssValue::Keyword(serialize_component_value(components[0])));
                }
                return None;
            }
            if name.eq_ignore_ascii_case("view") {
                if parse_view_function(value).is_some() {
                    return Some(CssValue::Keyword(serialize_component_value(components[0])));
                }
                return None;
            }
            if name.eq_ignore_ascii_case("env") {
                return parse_env_function(value);
            }
            if name.eq_ignore_ascii_case("var") {
                return parse_var_function(value);
            }
            if name.eq_ignore_ascii_case("calc")
                || name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("sin")
                || name.eq_ignore_ascii_case("cos")
                || name.eq_ignore_ascii_case("tan")
                || name.eq_ignore_ascii_case("asin")
                || name.eq_ignore_ascii_case("acos")
                || name.eq_ignore_ascii_case("atan")
                || name.eq_ignore_ascii_case("atan2")
                || name.eq_ignore_ascii_case("pow")
                || name.eq_ignore_ascii_case("sqrt")
                || name.eq_ignore_ascii_case("hypot")
                || name.eq_ignore_ascii_case("log")
                || name.eq_ignore_ascii_case("exp")
                || name.eq_ignore_ascii_case("calc-size")
            {
                return parse_calc_function(name, value, components[0]);
            }
            if name.eq_ignore_ascii_case("repeating-linear-gradient")
                || name.eq_ignore_ascii_case("repeating-radial-gradient")
                || name.eq_ignore_ascii_case("repeating-conic-gradient")
                || name.eq_ignore_ascii_case("blur")
                || name.eq_ignore_ascii_case("brightness")
                || name.eq_ignore_ascii_case("contrast")
                || name.eq_ignore_ascii_case("drop-shadow")
                || name.eq_ignore_ascii_case("grayscale")
                || name.eq_ignore_ascii_case("hue-rotate")
                || name.eq_ignore_ascii_case("invert")
                || name.eq_ignore_ascii_case("opacity")
                || name.eq_ignore_ascii_case("saturate")
                || name.eq_ignore_ascii_case("sepia")
                || name.eq_ignore_ascii_case("circle")
                || name.eq_ignore_ascii_case("ellipse")
                || name.eq_ignore_ascii_case("inset")
                || name.eq_ignore_ascii_case("polygon")
                || name.eq_ignore_ascii_case("path")
                || name.eq_ignore_ascii_case("rect")
                || name.eq_ignore_ascii_case("xywh")
                || name.eq_ignore_ascii_case("container-progress")
                || name.eq_ignore_ascii_case("scroll-progress")
                || name.eq_ignore_ascii_case("view-progress")
                || name.eq_ignore_ascii_case("image")
                || name.eq_ignore_ascii_case("element")
                || name.eq_ignore_ascii_case("paint")
                || name.eq_ignore_ascii_case("src")
                || name.eq_ignore_ascii_case("shape")
                || name.eq_ignore_ascii_case("ray")
            {
                return Some(CssValue::Keyword(serialize_component_value(components[0])));
            }
            None // TODO(spec): other functions
        }
        _ => None,
    }
}

fn extract_url_from_url_function(value: &[ComponentValue]) -> Option<String> {
    for val in value {
        match val {
            ComponentValue::Token(CssToken::String(s)) => {
                return Some(s.clone());
            }
            ComponentValue::Token(CssToken::Ident(s)) => {
                return Some(s.clone());
            }
            _ => {}
        }
    }
    None
}

fn parse_image_set_option(option_components: &[&ComponentValue]) -> Option<String> {
    let mut image_url: Option<String> = None;
    let mut resolution_seen = false;
    let mut type_seen = false;

    for comp in option_components {
        match comp {
            ComponentValue::Token(CssToken::String(s)) => {
                if image_url.is_some() {
                    return None;
                }
                image_url = Some(s.clone());
            }
            ComponentValue::Token(CssToken::Url(s)) => {
                if image_url.is_some() {
                    return None;
                }
                image_url = Some(s.clone());
            }
            ComponentValue::Function { name, value } => {
                if name.eq_ignore_ascii_case("url") {
                    if image_url.is_some() {
                        return None;
                    }
                    let u = extract_url_from_url_function(value)?;
                    image_url = Some(u);
                } else if name.eq_ignore_ascii_case("type") {
                    if type_seen {
                        return None;
                    }
                    let type_args: Vec<&ComponentValue> = value
                        .iter()
                        .filter(|v| !matches!(v, ComponentValue::Token(CssToken::Whitespace)))
                        .collect();
                    if type_args.len() != 1 {
                        return None;
                    }
                    match type_args[0] {
                        ComponentValue::Token(CssToken::String(_)) => {}
                        _ => return None,
                    }
                    type_seen = true;
                } else {
                    return None;
                }
            }
            ComponentValue::Token(CssToken::Dimension { value: _, unit }) => {
                if resolution_seen {
                    return None;
                }
                let unit_lower = unit.to_ascii_lowercase();
                if unit_lower != "x"
                    && unit_lower != "dpi"
                    && unit_lower != "dpcm"
                    && unit_lower != "dppx"
                {
                    return None;
                }
                resolution_seen = true;
            }
            _ => {
                return None;
            }
        }
    }

    image_url
}

fn parse_image_set_function(components: &[ComponentValue]) -> Option<CssValue> {
    let mut options_components: Vec<Vec<&ComponentValue>> = Vec::new();
    let mut current_option: Vec<&ComponentValue> = Vec::new();

    for comp in components {
        if matches!(comp, ComponentValue::Token(CssToken::Whitespace)) {
            continue;
        }
        if matches!(comp, ComponentValue::Token(CssToken::Comma)) {
            if current_option.is_empty() {
                return None;
            }
            options_components.push(current_option);
            current_option = Vec::new();
        } else {
            current_option.push(comp);
        }
    }
    if current_option.is_empty() {
        return None;
    }
    options_components.push(current_option);

    if options_components.is_empty() {
        return None;
    }

    let mut first_resolved_url: Option<String> = None;

    for opt_comps in options_components {
        let url_str = parse_image_set_option(&opt_comps)?;
        if first_resolved_url.is_none() {
            first_resolved_url = Some(url_str);
        }
    }

    // TODO(spec): true resolution-based selection needs DPR plumbing.
    first_resolved_url.map(|url_str| CssValue::Keyword(format!("url({})", url_str)))
}

fn parse_cubic_bezier_function(components: &[ComponentValue]) -> Option<CssValue> {
    let tokens: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    if tokens.len() != 7 {
        return None;
    }

    let x1 = match tokens[0] {
        ComponentValue::Token(CssToken::Number(v)) => *v as f32,
        _ => return None,
    };
    if !matches!(tokens[1], ComponentValue::Token(CssToken::Comma)) {
        return None;
    }
    let y1 = match tokens[2] {
        ComponentValue::Token(CssToken::Number(v)) => *v as f32,
        _ => return None,
    };
    if !matches!(tokens[3], ComponentValue::Token(CssToken::Comma)) {
        return None;
    }
    let x2 = match tokens[4] {
        ComponentValue::Token(CssToken::Number(v)) => *v as f32,
        _ => return None,
    };
    if !matches!(tokens[5], ComponentValue::Token(CssToken::Comma)) {
        return None;
    }
    let y2 = match tokens[6] {
        ComponentValue::Token(CssToken::Number(v)) => *v as f32,
        _ => return None,
    };

    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return None;
    }

    if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
        return None;
    }

    Some(CssValue::Keyword(format!(
        "cubic-bezier({}, {}, {}, {})",
        x1, y1, x2, y2
    )))
}

fn parse_steps_function(components: &[ComponentValue]) -> Option<CssValue> {
    let tokens: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    if tokens.is_empty() {
        return None;
    }

    let n_float = match tokens[0] {
        ComponentValue::Token(CssToken::Number(v)) => *v,
        _ => return None,
    };
    if n_float != n_float.round() || n_float < 1.0 || !n_float.is_finite() {
        return None;
    }
    let n = n_float as i32;

    let position = if tokens.len() == 1 {
        "end".to_string()
    } else if tokens.len() == 3 {
        if !matches!(tokens[1], ComponentValue::Token(CssToken::Comma)) {
            return None;
        }
        match tokens[2] {
            ComponentValue::Token(CssToken::Ident(s)) => {
                let s_lower = s.to_ascii_lowercase();
                match s_lower.as_str() {
                    "jump-start" | "jump-end" | "jump-none" | "jump-both" | "start" | "end" => {}
                    _ => return None,
                }
                s_lower
            }
            _ => return None,
        }
    } else {
        return None;
    };

    if tokens.len() == 1 {
        Some(CssValue::Keyword(format!("steps({})", n)))
    } else {
        Some(CssValue::Keyword(format!("steps({}, {})", n, position)))
    }
}

fn parse_linear_function(components: &[ComponentValue]) -> Option<CssValue> {
    let mut stops_components: Vec<Vec<&ComponentValue>> = Vec::new();
    let mut current_stop: Vec<&ComponentValue> = Vec::new();
    for comp in components {
        if matches!(comp, ComponentValue::Token(CssToken::Comma)) {
            stops_components.push(current_stop);
            current_stop = Vec::new();
        } else {
            current_stop.push(comp);
        }
    }
    if !current_stop.is_empty() {
        stops_components.push(current_stop);
    }

    if stops_components.len() < 2 {
        return None;
    }

    let mut parsed_stops = Vec::new();
    for stop_comps in stops_components {
        let non_ws: Vec<&ComponentValue> = stop_comps
            .iter()
            .copied()
            .filter(|c| !matches!(c, ComponentValue::Token(CssToken::Whitespace)))
            .collect();

        if non_ws.is_empty() {
            return None;
        }

        let val = match non_ws[0] {
            ComponentValue::Token(CssToken::Number(v)) => *v as f32,
            _ => return None,
        };
        if !val.is_finite() {
            return None;
        }

        let p1 = if non_ws.len() > 1 {
            match non_ws[1] {
                ComponentValue::Token(CssToken::Percentage(p)) => {
                    let pf = *p as f32;
                    if !pf.is_finite() {
                        return None;
                    }
                    Some(pf)
                }
                _ => return None,
            }
        } else {
            None
        };

        let p2 = if non_ws.len() > 2 {
            match non_ws[2] {
                ComponentValue::Token(CssToken::Percentage(p)) => {
                    let pf = *p as f32;
                    if !pf.is_finite() {
                        return None;
                    }
                    Some(pf)
                }
                _ => return None,
            }
        } else {
            None
        };

        if non_ws.len() > 3 {
            return None;
        }

        parsed_stops.push((val, p1, p2));
    }

    // We implement the full linear() stop grammar: linear(<number> [<percentage> <percentage>?]? ...)
    // per the CSS Easing Functions Level 1 spec.
    let mut parts = Vec::new();
    for (val, p1, p2) in parsed_stops {
        match (p1, p2) {
            (None, None) => parts.push(format!("{}", val)),
            (Some(pct1), None) => parts.push(format!("{} {}%", val, pct1)),
            (Some(pct1), Some(pct2)) => parts.push(format!("{} {}% {}%", val, pct1, pct2)),
            _ => unreachable!(),
        }
    }
    Some(CssValue::Keyword(format!("linear({})", parts.join(", "))))
}

fn is_length_percentage(cv: &ComponentValue) -> bool {
    match cv {
        ComponentValue::Token(CssToken::Percentage(_)) => true,
        ComponentValue::Token(CssToken::Dimension { value: _, unit }) => {
            let lower_unit = unit.to_ascii_lowercase();
            matches!(
                lower_unit.as_str(),
                "px" | "em"
                    | "rem"
                    | "pt"
                    | "vw"
                    | "vh"
                    | "in"
                    | "cm"
                    | "mm"
                    | "pc"
                    | "q"
                    | "ex"
                    | "ch"
                    | "vmin"
                    | "vmax"
            )
        }
        ComponentValue::Token(CssToken::Number(v)) => *v == 0.0,
        ComponentValue::Function { name: _, value: _ } => {
            // E.g. calc(), anchor(), anchor-size(), etc.
            true
        }
        _ => false,
    }
}

fn parse_anchor_side(cv: &ComponentValue) -> Option<String> {
    match cv {
        ComponentValue::Token(CssToken::Ident(s)) => {
            let s_lower = s.to_ascii_lowercase();
            if matches!(
                s_lower.as_str(),
                "top"
                    | "left"
                    | "right"
                    | "bottom"
                    | "start"
                    | "end"
                    | "self-start"
                    | "self-end"
                    | "center"
                    | "inside"
                    | "outside"
            ) {
                Some(s_lower)
            } else {
                None
            }
        }
        ComponentValue::Token(CssToken::Percentage(v)) => Some(format!("{}%", v)),
        _ => None,
    }
}

fn parse_anchor_function(components: &[ComponentValue]) -> Option<CssValue> {
    // Filter out whitespace
    let args: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    // Find the first top-level comma to separate the anchor parts and the optional fallback
    let mut comma_index = None;
    for (idx, arg) in args.iter().enumerate() {
        if matches!(arg, ComponentValue::Token(CssToken::Comma)) {
            comma_index = Some(idx);
            break;
        }
    }

    let (anchor_parts, fallback_part) = match comma_index {
        Some(idx) => {
            let first = &args[..idx];
            let second = &args[idx + 1..];
            (first, Some(second))
        }
        None => (&args[..], None),
    };

    // Validate anchor_parts:
    // [ <anchor-name> ]? <anchor-side>
    // Can be 1 or 2 items
    let mut anchor_name: Option<String> = None;
    let anchor_side_comp: &ComponentValue;

    match anchor_parts.len() {
        1 => {
            anchor_side_comp = anchor_parts[0];
        }
        2 => {
            let first = anchor_parts[0];
            if let ComponentValue::Token(CssToken::Ident(s)) = first {
                if s.starts_with("--") {
                    anchor_name = Some(s.clone());
                } else {
                    return None;
                }
            } else {
                return None;
            }
            anchor_side_comp = anchor_parts[1];
        }
        _ => return None,
    }

    // Validate <anchor-side>
    let anchor_side_str = parse_anchor_side(anchor_side_comp)?;

    // Validate optional fallback
    let fallback_str = match fallback_part {
        Some(fallback_args) => {
            // Must have exactly 1 component for <length-percentage>
            if fallback_args.len() != 1 {
                return None;
            }
            let fb = fallback_args[0];
            if is_length_percentage(fb) {
                Some(serialize_component_value(fb))
            } else {
                return None;
            }
        }
        None => None,
    };

    // Construct the serialized representation
    let mut res = String::from("anchor(");
    if let Some(name) = anchor_name {
        res.push_str(&name);
        res.push(' ');
    }
    res.push_str(&anchor_side_str);
    if let Some(fb) = fallback_str {
        res.push_str(", ");
        res.push_str(&fb);
    }
    res.push(')');

    // TODO(spec): layout resolution against an actual anchor element would plug in here.
    Some(CssValue::Keyword(res))
}

fn parse_anchor_size_function(components: &[ComponentValue]) -> Option<CssValue> {
    // Filter out whitespace
    let args: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    // Find the first top-level comma to separate the anchor parts and the optional fallback
    let mut comma_index = None;
    for (idx, arg) in args.iter().enumerate() {
        if matches!(arg, ComponentValue::Token(CssToken::Comma)) {
            comma_index = Some(idx);
            break;
        }
    }

    let (anchor_parts, fallback_part) = match comma_index {
        Some(idx) => {
            let first = &args[..idx];
            let second = &args[idx + 1..];
            (first, Some(second))
        }
        None => (&args[..], None),
    };

    // Validate anchor_parts:
    // [ <anchor-name> ]? <anchor-size>
    // Can be 1 or 2 items
    let mut anchor_name: Option<String> = None;
    let anchor_size_comp: &ComponentValue;

    match anchor_parts.len() {
        1 => {
            anchor_size_comp = anchor_parts[0];
        }
        2 => {
            let first = anchor_parts[0];
            if let ComponentValue::Token(CssToken::Ident(s)) = first {
                if s.starts_with("--") {
                    anchor_name = Some(s.clone());
                } else {
                    return None;
                }
            } else {
                return None;
            }
            anchor_size_comp = anchor_parts[1];
        }
        _ => return None,
    }

    // Validate <anchor-size>
    // where <anchor-size> is one of: width, height, block, inline, self-block, self-inline.
    let anchor_size_str = match anchor_size_comp {
        ComponentValue::Token(CssToken::Ident(s)) => {
            let s_lower = s.to_ascii_lowercase();
            if matches!(
                s_lower.as_str(),
                "width" | "height" | "block" | "inline" | "self-block" | "self-inline"
            ) {
                s_lower
            } else {
                return None;
            }
        }
        _ => return None,
    };

    // Validate optional fallback
    let fallback_str = match fallback_part {
        Some(fallback_args) => {
            // Must have exactly 1 component for <length-percentage>
            if fallback_args.len() != 1 {
                return None;
            }
            let fb = fallback_args[0];
            if is_length_percentage(fb) {
                Some(serialize_component_value(fb))
            } else {
                return None;
            }
        }
        None => None,
    };

    // Construct the serialized representation
    let mut res = String::from("anchor-size(");
    if let Some(name) = anchor_name {
        res.push_str(&name);
        res.push(' ');
    }
    res.push_str(&anchor_size_str);
    if let Some(fb) = fallback_str {
        res.push_str(", ");
        res.push_str(&fb);
    }
    res.push(')');

    // TODO(spec): layout resolution against an actual anchor element would plug in here.
    Some(CssValue::Keyword(res))
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
    } else if s.len() == 4 {
        let r = u8::from_str_radix(&s[0..1], 16).ok()?;
        let g = u8::from_str_radix(&s[1..2], 16).ok()?;
        let b = u8::from_str_radix(&s[2..3], 16).ok()?;
        let a = u8::from_str_radix(&s[3..4], 16).ok()?;
        Some(Color::Rgba(r * 17, g * 17, b * 17, a * 17))
    } else if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgba(r, g, b, 255))
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        let a = u8::from_str_radix(&s[6..8], 16).ok()?;
        Some(Color::Rgba(r, g, b, a))
    } else {
        None
    }
}

fn srgb_to_linear_srgb(c_u: u8) -> f64 {
    let c = c_u as f64 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn srgb_to_xyz_d50(r_lin: f64, g_lin: f64, b_lin: f64) -> (f64, f64, f64) {
    let x = 0.4360747 * r_lin + 0.3850649 * g_lin + 0.1430804 * b_lin;
    let y = 0.2225045 * r_lin + 0.7168786 * g_lin + 0.0606169 * b_lin;
    let z = 0.0139322 * r_lin + 0.0971045 * g_lin + 0.7141733 * b_lin;
    (x, y, z)
}

fn linear_srgb_to_lms(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
    (l, m, s)
}

fn lms_to_oklab(l_: f64, m_: f64, s_: f64) -> (f64, f64, f64) {
    let l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;
    (l, a, b)
}

fn color_to_lab(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r_lin = srgb_to_linear_srgb(r_u);
    let g_lin = srgb_to_linear_srgb(g_u);
    let b_lin = srgb_to_linear_srgb(b_u);
    let (x, y, z) = srgb_to_xyz_d50(r_lin, g_lin, b_lin);

    let f = |t: f64| {
        let d = 6.0 / 29.0;
        if t > d * d * d {
            t.powf(1.0 / 3.0)
        } else {
            t / (3.0 * d * d) + 4.0 / 29.0
        }
    };

    let xr = x / 0.96422;
    let yr = y / 1.0;
    let zr = z / 0.82521;

    let fx = f(xr);
    let fy = f(yr);
    let fz = f(zr);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    let alpha = a_u as f32 / 255.0;

    (l as f32, a as f32, b as f32, alpha)
}

fn color_to_lch(color: Color) -> (f32, f32, f32, f32) {
    let (l, a, b, alpha) = color_to_lab(color);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h, alpha)
}

fn color_to_oklab(color: Color) -> (f32, f32, f32, f32) {
    let Color::Rgba(r_u, g_u, b_u, a_u) = color;
    let r_lin = srgb_to_linear_srgb(r_u);
    let g_lin = srgb_to_linear_srgb(g_u);
    let b_lin = srgb_to_linear_srgb(b_u);
    let (l_lms, m_lms, s_lms) = linear_srgb_to_lms(r_lin, g_lin, b_lin);

    let l_ = l_lms.cbrt();
    let m_ = m_lms.cbrt();
    let s_ = s_lms.cbrt();

    let (l, a, b) = lms_to_oklab(l_, m_, s_);
    let alpha = a_u as f32 / 255.0;

    (l as f32, a as f32, b as f32, alpha)
}

fn color_to_oklch(color: Color) -> (f32, f32, f32, f32) {
    let (l, a, b, alpha) = color_to_oklab(color);
    let c = (a * a + b * b).sqrt();
    let mut h = b.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    (l, c, h, alpha)
}

fn substitute_color_variables(
    components: &[ComponentValue],
    variables: &[(&str, f32)],
) -> Vec<ComponentValue> {
    components
        .iter()
        .map(|comp| match comp {
            ComponentValue::Token(CssToken::Ident(s)) => {
                if let Some((_, val)) = variables
                    .iter()
                    .find(|(name, _)| s.eq_ignore_ascii_case(name))
                {
                    ComponentValue::Token(CssToken::Number(*val as f64))
                } else if s.eq_ignore_ascii_case("none") {
                    ComponentValue::Token(CssToken::Number(0.0))
                } else {
                    comp.clone()
                }
            }
            ComponentValue::SimpleBlock { associated, value } => ComponentValue::SimpleBlock {
                associated: *associated,
                value: substitute_color_variables(value, variables),
            },
            ComponentValue::Function { name, value } => ComponentValue::Function {
                name: name.clone(),
                value: substitute_color_variables(value, variables),
            },
            _ => comp.clone(),
        })
        .collect()
}

fn evaluate_color_component(
    comp: &ComponentValue,
    variables: &[(&str, f32)],
    pct_scale: f32,
) -> Option<f32> {
    match comp {
        ComponentValue::Token(CssToken::Number(v)) => Some(*v as f32),
        ComponentValue::Token(CssToken::Percentage(v)) => Some((*v as f32 / 100.0) * pct_scale),
        ComponentValue::Token(CssToken::Dimension { value, unit }) => {
            let deg = match unit.to_ascii_lowercase().as_str() {
                "deg" => *value,
                "rad" => *value * 180.0 / std::f64::consts::PI,
                "grad" => *value * 0.9,
                "turn" => *value * 360.0,
                _ => return None,
            };
            Some(deg as f32)
        }
        ComponentValue::Token(CssToken::Ident(s)) => {
            if let Some((_, val)) = variables
                .iter()
                .find(|(name, _)| s.eq_ignore_ascii_case(name))
            {
                Some(*val)
            } else if s.eq_ignore_ascii_case("none") {
                Some(0.0)
            } else {
                None
            }
        }
        ComponentValue::Function { name, value }
            if name.eq_ignore_ascii_case("calc")
                || name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("sin")
                || name.eq_ignore_ascii_case("cos")
                || name.eq_ignore_ascii_case("tan")
                || name.eq_ignore_ascii_case("asin")
                || name.eq_ignore_ascii_case("acos")
                || name.eq_ignore_ascii_case("atan")
                || name.eq_ignore_ascii_case("atan2")
                || name.eq_ignore_ascii_case("pow")
                || name.eq_ignore_ascii_case("sqrt")
                || name.eq_ignore_ascii_case("hypot")
                || name.eq_ignore_ascii_case("log")
                || name.eq_ignore_ascii_case("exp") =>
        {
            let substituted = substitute_color_variables(value, variables);
            let vars_map = std::collections::HashMap::new();
            let css_val = if name.eq_ignore_ascii_case("calc") {
                crate::css::resolve::evaluate_calc(&substituted, 16.0, 1000.0, 1000.0, &vars_map)
            } else {
                crate::css::resolve::evaluate_math_fn(
                    &name.to_ascii_lowercase(),
                    &substituted,
                    16.0,
                    1000.0,
                    1000.0,
                    &vars_map,
                )
            };
            if let Some(val) = css_val {
                match val {
                    CssValue::Number(num) => Some(num),
                    CssValue::Length(px, _) => Some(px),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn evaluate_channel_expression(
    comp: &ComponentValue,
    br_f: f32,
    bg_f: f32,
    bb_f: f32,
    ba_f: f32,
    is_alpha: bool,
) -> Option<f32> {
    match comp {
        ComponentValue::Token(CssToken::Number(v)) => Some(*v as f32),
        ComponentValue::Token(CssToken::Percentage(v)) => {
            let pct = *v as f32;
            if is_alpha {
                Some(pct / 100.0)
            } else {
                Some((pct / 100.0) * 255.0)
            }
        }
        ComponentValue::Token(CssToken::Ident(s)) => {
            if s.eq_ignore_ascii_case("r") {
                Some(br_f)
            } else if s.eq_ignore_ascii_case("g") {
                Some(bg_f)
            } else if s.eq_ignore_ascii_case("b") {
                Some(bb_f)
            } else if s.eq_ignore_ascii_case("alpha") {
                Some(ba_f)
            } else if s.eq_ignore_ascii_case("none") {
                Some(0.0)
            } else {
                None
            }
        }
        ComponentValue::Function { name, value }
            if name.eq_ignore_ascii_case("calc")
                || name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("sin")
                || name.eq_ignore_ascii_case("cos")
                || name.eq_ignore_ascii_case("tan")
                || name.eq_ignore_ascii_case("asin")
                || name.eq_ignore_ascii_case("acos")
                || name.eq_ignore_ascii_case("atan")
                || name.eq_ignore_ascii_case("atan2")
                || name.eq_ignore_ascii_case("pow")
                || name.eq_ignore_ascii_case("sqrt")
                || name.eq_ignore_ascii_case("hypot")
                || name.eq_ignore_ascii_case("log")
                || name.eq_ignore_ascii_case("exp") =>
        {
            let variables = [("r", br_f), ("g", bg_f), ("b", bb_f), ("alpha", ba_f)];
            let substituted = substitute_color_variables(value, &variables);
            let vars_map = std::collections::HashMap::new();
            let css_val = if name.eq_ignore_ascii_case("calc") {
                crate::css::resolve::evaluate_calc(&substituted, 16.0, 1000.0, 1000.0, &vars_map)
            } else {
                crate::css::resolve::evaluate_math_fn(
                    &name.to_ascii_lowercase(),
                    &substituted,
                    16.0,
                    1000.0,
                    1000.0,
                    &vars_map,
                )
            };
            if let Some(val) = css_val {
                match val {
                    CssValue::Number(num) => Some(num),
                    CssValue::Length(px, _) => Some(px),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

enum HslChannel {
    Hue,
    Saturation,
    Lightness,
    Alpha,
}

fn rgba_to_hsla(r_u: u8, g_u: u8, b_u: u8, a_u: u8) -> (f32, f32, f32, f32) {
    let r = r_u as f32 / 255.0;
    let g = g_u as f32 / 255.0;
    let b = b_u as f32 / 255.0;
    let a = a_u as f32 / 255.0;

    let mut max = r;
    if g > max {
        max = g;
    }
    if b > max {
        max = b;
    }

    let mut min = r;
    if g < min {
        min = g;
    }
    if b < min {
        min = b;
    }

    let l = (max + min) / 2.0;

    let (h, s) = if (max - min).abs() < f32::EPSILON {
        (0.0, 0.0)
    } else {
        let diff = max - min;
        let s_val = if l <= 0.5 {
            diff / (max + min)
        } else {
            diff / (2.0 - max - min)
        };

        let h_val = if (max - r).abs() < f32::EPSILON {
            60.0 * (g - b) / diff
        } else if (max - g).abs() < f32::EPSILON {
            60.0 * (b - r) / diff + 120.0
        } else {
            60.0 * (r - g) / diff + 240.0
        };

        let h_final = (h_val + 360.0) % 360.0;
        (h_final, s_val)
    };

    (h, s * 100.0, l * 100.0, a)
}

fn evaluate_hsl_channel_expression(
    comp: &ComponentValue,
    base_h: f32,
    base_s: f32,
    base_l: f32,
    base_alpha: f32,
    channel_type: HslChannel,
) -> Option<f32> {
    match comp {
        ComponentValue::Token(CssToken::Number(v)) => Some(*v as f32),
        ComponentValue::Token(CssToken::Percentage(v)) => {
            let pct = *v as f32;
            match channel_type {
                HslChannel::Alpha => Some(pct / 100.0),
                _ => Some(pct),
            }
        }
        ComponentValue::Token(CssToken::Ident(s)) => {
            if s.eq_ignore_ascii_case("h") {
                Some(base_h)
            } else if s.eq_ignore_ascii_case("s") {
                Some(base_s)
            } else if s.eq_ignore_ascii_case("l") {
                Some(base_l)
            } else if s.eq_ignore_ascii_case("alpha") {
                Some(base_alpha)
            } else if s.eq_ignore_ascii_case("none") {
                Some(0.0)
            } else {
                None
            }
        }
        ComponentValue::Function { name, value }
            if name.eq_ignore_ascii_case("calc")
                || name.eq_ignore_ascii_case("min")
                || name.eq_ignore_ascii_case("max")
                || name.eq_ignore_ascii_case("clamp")
                || name.eq_ignore_ascii_case("round")
                || name.eq_ignore_ascii_case("mod")
                || name.eq_ignore_ascii_case("rem")
                || name.eq_ignore_ascii_case("abs")
                || name.eq_ignore_ascii_case("sign")
                || name.eq_ignore_ascii_case("sin")
                || name.eq_ignore_ascii_case("cos")
                || name.eq_ignore_ascii_case("tan")
                || name.eq_ignore_ascii_case("asin")
                || name.eq_ignore_ascii_case("acos")
                || name.eq_ignore_ascii_case("atan")
                || name.eq_ignore_ascii_case("atan2")
                || name.eq_ignore_ascii_case("pow")
                || name.eq_ignore_ascii_case("sqrt")
                || name.eq_ignore_ascii_case("hypot")
                || name.eq_ignore_ascii_case("log")
                || name.eq_ignore_ascii_case("exp") =>
        {
            let variables = [
                ("h", base_h),
                ("s", base_s),
                ("l", base_l),
                ("alpha", base_alpha),
            ];
            let substituted = substitute_color_variables(value, &variables);
            let vars_map = std::collections::HashMap::new();
            let css_val = if name.eq_ignore_ascii_case("calc") {
                crate::css::resolve::evaluate_calc(&substituted, 16.0, 1000.0, 1000.0, &vars_map)
            } else {
                crate::css::resolve::evaluate_math_fn(
                    &name.to_ascii_lowercase(),
                    &substituted,
                    16.0,
                    1000.0,
                    1000.0,
                    &vars_map,
                )
            };
            if let Some(val) = css_val {
                match val {
                    CssValue::Number(num) => Some(num),
                    CssValue::Length(px, _) => Some(px),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn parse_rgb_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        // base color is at non_ws[1]
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let Color::Rgba(br, bg, bb, ba) = base_color;
        let br_f = br as f32;
        let bg_f = bg as f32;
        let bb_f = bb as f32;
        let ba_f = ba as f32 / 255.0;

        let r_val = evaluate_channel_expression(non_ws[2], br_f, bg_f, bb_f, ba_f, false)?;
        let g_val = evaluate_channel_expression(non_ws[3], br_f, bg_f, bb_f, ba_f, false)?;
        let b_val = evaluate_channel_expression(non_ws[4], br_f, bg_f, bb_f, ba_f, false)?;

        let a_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_channel_expression(non_ws[6], br_f, bg_f, bb_f, ba_f, true)?
        } else {
            1.0
        };

        // Note: other color spaces' relative form is implemented below.

        return Some(Color::Rgba(
            r_val.clamp(0.0, 255.0) as u8,
            g_val.clamp(0.0, 255.0) as u8,
            b_val.clamp(0.0, 255.0) as u8,
            (a_val.clamp(0.0, 1.0) * 255.0) as u8,
        ));
    }

    // Basic support for rgb(r, g, b) or rgba(r, g, b, a)
    // Filter out whitespace, commas, and slashes
    enum RgbArg {
        Number(f32),
        Percentage(f32),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(RgbArg::Number(*v as f32)),
            ComponentValue::Token(CssToken::Percentage(v)) => {
                args.push(RgbArg::Percentage(*v as f32))
            }
            _ => return None,
        }
    }

    if args.len() == 3 {
        let r = match args[0] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        let g = match args[1] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        let b = match args[2] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        Some(Color::Rgba(
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
            255,
        ))
    } else if args.len() == 4 {
        let r = match args[0] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        let g = match args[1] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        let b = match args[2] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => (v / 100.0) * 255.0,
        };
        let a_val = match args[3] {
            RgbArg::Number(v) => v,
            RgbArg::Percentage(v) => v / 100.0,
        };
        Some(Color::Rgba(
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
            (a_val.clamp(0.0, 1.0) * 255.0) as u8,
        ))
    } else {
        None
    }
}

fn parse_hsl_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        // base color is at non_ws[1]
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let Color::Rgba(br, bg, bb, ba) = base_color;
        let (base_h, base_s, base_l, base_alpha) = rgba_to_hsla(br, bg, bb, ba);

        let h_val = evaluate_hsl_channel_expression(
            non_ws[2],
            base_h,
            base_s,
            base_l,
            base_alpha,
            HslChannel::Hue,
        )?;
        let s_val = evaluate_hsl_channel_expression(
            non_ws[3],
            base_h,
            base_s,
            base_l,
            base_alpha,
            HslChannel::Saturation,
        )?;
        let l_val = evaluate_hsl_channel_expression(
            non_ws[4],
            base_h,
            base_s,
            base_l,
            base_alpha,
            HslChannel::Lightness,
        )?;

        let a_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_hsl_channel_expression(
                non_ws[6],
                base_h,
                base_s,
                base_l,
                base_alpha,
                HslChannel::Alpha,
            )?
        } else {
            1.0
        };

        let h = ((h_val % 360.0) + 360.0) % 360.0;
        let s = (s_val / 100.0).clamp(0.0, 1.0);
        let l = (l_val / 100.0).clamp(0.0, 1.0);
        let alpha = (a_val.clamp(0.0, 1.0) * 255.0) as u8;

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

        return Some(Color::Rgba(r, g, b, alpha));
    }

    enum HslArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(HslArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(HslArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(HslArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // Parse Hue
    let h_val = match args[0] {
        HslArg::Number(v) => v,
        HslArg::Angle(v) => v,
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
            _ => return None,
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

fn parse_hwb_function(components: &[ComponentValue]) -> Option<Color> {
    enum HwbArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(HwbArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(HwbArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(HwbArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // Parse Hue
    let h_val = match args[0] {
        HwbArg::Number(v) => v,
        HwbArg::Angle(v) => v,
        _ => return None,
    };
    let h = ((h_val % 360.0) + 360.0) % 360.0;

    // Parse Whiteness
    let w_val = match args[1] {
        HwbArg::Percentage(v) => v,
        _ => return None,
    };
    let w = (w_val / 100.0).clamp(0.0, 1.0);

    // Parse Blackness
    let b_val = match args[2] {
        HwbArg::Percentage(v) => v,
        _ => return None,
    };
    let b = (b_val / 100.0).clamp(0.0, 1.0);

    // Parse Alpha
    let alpha = if args.len() == 4 {
        let a_val = match args[3] {
            HwbArg::Number(v) => v,
            HwbArg::Percentage(v) => v / 100.0,
            _ => return None,
        };
        (a_val.clamp(0.0, 1.0) * 255.0) as u8
    } else {
        255
    };

    let c = 1.0;
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

    let (r_chan, g_chan, b_chan) = if w + b >= 1.0 {
        let gray = w / (w + b);
        (gray, gray, gray)
    } else {
        let factor = 1.0 - w - b;
        (r1 * factor + w, g1 * factor + w, b1 * factor + w)
    };

    let r = (r_chan * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (g_chan * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (b_chan * 255.0).round().clamp(0.0, 255.0) as u8;

    Some(Color::Rgba(r, g, b, alpha))
}

fn parse_oklab_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let (base_l, base_a, base_b, base_alpha) = color_to_oklab(base_color);

        let variables = [
            ("l", base_l),
            ("a", base_a),
            ("b", base_b),
            ("alpha", base_alpha),
        ];

        let l_val = evaluate_color_component(non_ws[2], &variables, 1.0)?;
        let a_val = evaluate_color_component(non_ws[3], &variables, 0.4)?;
        let b_val = evaluate_color_component(non_ws[4], &variables, 0.4)?;

        let alpha_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_color_component(non_ws[6], &variables, 1.0)?
        } else {
            base_alpha
        };

        let l_val = (l_val.max(0.0)) as f64;
        let a_val = a_val as f64;
        let b_val = b_val as f64;
        let alpha = (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8;

        return Some(oklab_to_color(l_val, a_val, b_val, alpha));
    }

    enum OklArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(OklArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(OklArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(OklArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // L (lightness): Number [0,1] or Percentage. Clamp L >= 0.
    let l_val = match args[0] {
        OklArg::Number(v) => v,
        OklArg::Percentage(v) => v / 100.0,
        _ => return None,
    };
    let l_val = l_val.max(0.0);

    // a: Number (roughly [-0.4, 0.4]) or Percentage (100% = 0.4, -100% = -0.4).
    let a_val = match args[1] {
        OklArg::Number(v) => v,
        OklArg::Percentage(v) => (v / 100.0) * 0.4,
        _ => return None,
    };

    // b: Number (roughly [-0.4, 0.4]) or Percentage (100% = 0.4, -100% = -0.4).
    let b_val = match args[2] {
        OklArg::Number(v) => v,
        OklArg::Percentage(v) => (v / 100.0) * 0.4,
        _ => return None,
    };

    // alpha: Number [0,1] or Percentage. Default 1.0.
    let alpha = if args.len() == 4 {
        let a_val = match args[3] {
            OklArg::Number(v) => v,
            OklArg::Percentage(v) => v / 100.0,
            _ => return None,
        };
        (a_val.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };

    let color = oklab_to_color(l_val, a_val, b_val, alpha);
    Some(color)
}

fn parse_oklch_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let (base_l, base_c, base_h, base_alpha) = color_to_oklch(base_color);

        let variables = [
            ("l", base_l),
            ("c", base_c),
            ("h", base_h),
            ("alpha", base_alpha),
        ];

        let l_val = evaluate_color_component(non_ws[2], &variables, 1.0)?;
        let c_val = evaluate_color_component(non_ws[3], &variables, 0.4)?;
        let h_val = evaluate_color_component(non_ws[4], &variables, 360.0)?;

        let alpha_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_color_component(non_ws[6], &variables, 1.0)?
        } else {
            base_alpha
        };

        let l_val = (l_val.max(0.0)) as f64;
        let c_val = (c_val.max(0.0)) as f64;
        let h_deg = h_val as f64;
        let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;
        let alpha = (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8;

        let h_rad = h_deg.to_radians();
        let a_val = c_val * h_rad.cos();
        let b_val = c_val * h_rad.sin();

        return Some(oklab_to_color(l_val, a_val, b_val, alpha));
    }

    enum OklArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(OklArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(OklArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(OklArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // L (lightness): Number [0,1] or Percentage. Clamp L >= 0.
    let l_val = match args[0] {
        OklArg::Number(v) => v,
        OklArg::Percentage(v) => v / 100.0,
        _ => return None,
    };
    let l_val = l_val.max(0.0);

    // C (chroma): Number >= 0 or Percentage (100% = 0.4). Clamp C >= 0.
    let c_val = match args[1] {
        OklArg::Number(v) => v,
        OklArg::Percentage(v) => (v / 100.0) * 0.4,
        _ => return None,
    };
    let c_val = c_val.max(0.0);

    // H (hue): Number (degrees) or Angle. Normalize ((H % 360) + 360) % 360.
    let h_deg = match args[2] {
        OklArg::Number(v) => v,
        OklArg::Angle(v) => v,
        _ => return None,
    };
    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;

    // alpha: Number [0,1] or Percentage. Default 1.0.
    let alpha = if args.len() == 4 {
        let a_val = match args[3] {
            OklArg::Number(v) => v,
            OklArg::Percentage(v) => v / 100.0,
            _ => return None,
        };
        (a_val.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };

    let h_rad = h_deg.to_radians();
    let a_val = c_val * h_rad.cos();
    let b_val = c_val * h_rad.sin();

    let color = oklab_to_color(l_val, a_val, b_val, alpha);
    Some(color)
}

fn oklab_to_color(l_val: f64, a_val: f64, b_val: f64, alpha: u8) -> Color {
    let l_ = l_val + 0.3963377774 * a_val + 0.2158037573 * b_val;
    let m_ = l_val - 0.1055613458 * a_val - 0.0638541728 * b_val;
    let s_ = l_val - 0.0894841775 * a_val - 1.2914855480 * b_val;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r_lin = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g_lin = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b_lin = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    let r = linear_srgb_to_srgb(r_lin);
    let g = linear_srgb_to_srgb(g_lin);
    let b = linear_srgb_to_srgb(b_lin);

    Color::Rgba(r, g, b, alpha)
}

fn linear_srgb_to_srgb(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

fn rec2020_to_linear(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    if v < 0.081243736875153 {
        v / 4.5
    } else {
        ((v + 0.099360459341054) / 1.099360459341054).powf(1.0 / 0.45)
    }
}

fn a98_to_linear(v: f64) -> f64 {
    v.clamp(0.0, 1.0).powf(2.19921875)
}

fn prophoto_to_linear(v: f64) -> f64 {
    let v = v.clamp(0.0, 1.0);
    if v < 0.03125 { v / 16.0 } else { v.powf(1.8) }
}

fn parse_lab_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let (base_l, base_a, base_b, base_alpha) = color_to_lab(base_color);

        let variables = [
            ("l", base_l),
            ("a", base_a),
            ("b", base_b),
            ("alpha", base_alpha),
        ];

        let l_val = evaluate_color_component(non_ws[2], &variables, 100.0)?;
        let a_val = evaluate_color_component(non_ws[3], &variables, 125.0)?;
        let b_val = evaluate_color_component(non_ws[4], &variables, 125.0)?;

        let alpha_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_color_component(non_ws[6], &variables, 1.0)?
        } else {
            base_alpha
        };

        let l_val = (l_val.max(0.0)) as f64;
        let a_val = a_val as f64;
        let b_val = b_val as f64;
        let alpha = (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8;

        return Some(lab_to_color(l_val, a_val, b_val, alpha));
    }

    enum LabArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(LabArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(LabArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(LabArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // L (lightness): Number [0,100] or Percentage (100% = 100). Clamp L >= 0.
    let l_val = match args[0] {
        LabArg::Number(v) => v,
        LabArg::Percentage(v) => v,
        _ => return None,
    };
    let l_val = l_val.max(0.0);

    // a: Number roughly [-125, 125] or Percentage (100% = 125, -100% = -125 -> (v / 100.0) * 125.0).
    let a_val = match args[1] {
        LabArg::Number(v) => v,
        LabArg::Percentage(v) => (v / 100.0) * 125.0,
        _ => return None,
    };

    // b: Number roughly [-125, 125] or Percentage (100% = 125, -100% = -125 -> (v / 100.0) * 125.0).
    let b_val = match args[2] {
        LabArg::Number(v) => v,
        LabArg::Percentage(v) => (v / 100.0) * 125.0,
        _ => return None,
    };

    // alpha: Number [0,1] or Percentage. Default 1.0.
    let alpha = if args.len() == 4 {
        let alpha_val = match args[3] {
            LabArg::Number(v) => v,
            LabArg::Percentage(v) => v / 100.0,
            _ => return None,
        };
        (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };

    let color = lab_to_color(l_val, a_val, b_val, alpha);
    Some(color)
}

fn parse_lch_function(components: &[ComponentValue]) -> Option<Color> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    let is_relative = match non_ws.first() {
        Some(ComponentValue::Token(CssToken::Ident(s))) => s.eq_ignore_ascii_case("from"),
        _ => false,
    };

    if is_relative {
        if non_ws.len() != 5 && non_ws.len() != 7 {
            return None;
        }
        let base_color_components = vec![non_ws[1].clone()];
        let base_color = parse_color_argument(&base_color_components)?;

        let (base_l, base_c, base_h, base_alpha) = color_to_lch(base_color);

        let variables = [
            ("l", base_l),
            ("c", base_c),
            ("h", base_h),
            ("alpha", base_alpha),
        ];

        let l_val = evaluate_color_component(non_ws[2], &variables, 100.0)?;
        let c_val = evaluate_color_component(non_ws[3], &variables, 150.0)?;
        let h_val = evaluate_color_component(non_ws[4], &variables, 360.0)?;

        let alpha_val = if non_ws.len() == 7 {
            if !matches!(non_ws[5], ComponentValue::Token(CssToken::Delim('/'))) {
                return None;
            }
            evaluate_color_component(non_ws[6], &variables, 1.0)?
        } else {
            base_alpha
        };

        let l_val = (l_val.max(0.0)) as f64;
        let c_val = (c_val.max(0.0)) as f64;
        let h_deg = h_val as f64;
        let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;
        let alpha = (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8;

        let h_rad = h_deg.to_radians();
        let a_val = c_val * h_rad.cos();
        let b_val = c_val * h_rad.sin();

        return Some(lab_to_color(l_val, a_val, b_val, alpha));
    }

    enum LchArg {
        Number(f64),
        Percentage(f64),
        Angle(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(LchArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(LchArg::Percentage(*v)),
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let deg = match unit.to_ascii_lowercase().as_str() {
                    "deg" => *value,
                    "rad" => *value * 180.0 / std::f64::consts::PI,
                    "grad" => *value * 0.9,
                    "turn" => *value * 360.0,
                    _ => return None,
                };
                args.push(LchArg::Angle(deg));
            }
            _ => return None,
        }
    }

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    // L (lightness): Number [0,100] or Percentage (100% = 100). Clamp L >= 0.
    let l_val = match args[0] {
        LchArg::Number(v) => v,
        LchArg::Percentage(v) => v,
        _ => return None,
    };
    let l_val = l_val.max(0.0);

    // C (chroma): Number >= 0 or Percentage (100% = 150 -> percent/100*150); clamp C >= 0.
    let c_val = match args[1] {
        LchArg::Number(v) => v,
        LchArg::Percentage(v) => (v / 100.0) * 150.0,
        _ => return None,
    };
    let c_val = c_val.max(0.0);

    // H (hue): Number (degrees) or Angle. Normalize ((H % 360) + 360) % 360.
    let h_deg = match args[2] {
        LchArg::Number(v) => v,
        LchArg::Angle(v) => v,
        _ => return None,
    };
    let h_deg = ((h_deg % 360.0) + 360.0) % 360.0;

    // alpha: Number [0,1] or Percentage. Default 1.0.
    let alpha = if args.len() == 4 {
        let alpha_val = match args[3] {
            LchArg::Number(v) => v,
            LchArg::Percentage(v) => v / 100.0,
            _ => return None,
        };
        (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };

    let h_rad = h_deg.to_radians();
    let a_val = c_val * h_rad.cos();
    let b_val = c_val * h_rad.sin();

    let color = lab_to_color(l_val, a_val, b_val, alpha);
    Some(color)
}

fn lab_to_color(l_val: f64, a_val: f64, b_val: f64, alpha: u8) -> Color {
    let fy = (l_val + 16.0) / 116.0;
    let fx = fy + a_val / 500.0;
    let fz = fy - b_val / 200.0;

    let finv = |t: f64| {
        let d = 6.0 / 29.0;
        if t > d {
            t * t * t
        } else {
            3.0 * d * d * (t - 4.0 / 29.0)
        }
    };

    let xr = finv(fx);
    let yr = finv(fy);
    let zr = finv(fz);

    let x = xr * 0.96422;
    let y = yr * 1.0;
    let z = zr * 0.82521;

    // Bradford-adapted D50 XYZ -> linear sRGB:
    let r_lin = 3.1338561 * x - 1.6168667 * y - 0.4906146 * z;
    let g_lin = -0.9787684 * x + 1.9161415 * y + 0.0334540 * z;
    let b_lin = 0.0719453 * x - 0.2289914 * y + 1.4052427 * z;

    let gamma_encode = |c: f64| {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };

    let r = gamma_encode(r_lin);
    let g = gamma_encode(g_lin);
    let b = gamma_encode(b_lin);

    Color::Rgba(r, g, b, alpha)
}

fn parse_color_function(components: &[ComponentValue]) -> Option<Color> {
    enum ColorArg {
        Number(f64),
        Percentage(f64),
    }

    let mut colorspace_ident: Option<String> = None;
    let mut args = Vec::new();

    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Ident(s)) => {
                if colorspace_ident.is_none() {
                    colorspace_ident = Some(s.clone());
                } else {
                    return None;
                }
            }
            ComponentValue::Token(CssToken::Number(v)) => {
                colorspace_ident.as_ref()?;
                args.push(ColorArg::Number(*v));
            }
            ComponentValue::Token(CssToken::Percentage(v)) => {
                colorspace_ident.as_ref()?;
                args.push(ColorArg::Percentage(*v));
            }
            _ => return None,
        }
    }

    let colorspace = colorspace_ident?;

    if args.len() != 3 && args.len() != 4 {
        return None;
    }

    let to_f64_channel = |arg: &ColorArg| match arg {
        ColorArg::Number(v) => *v,
        ColorArg::Percentage(v) => *v / 100.0,
    };

    let c1 = to_f64_channel(&args[0]);
    let c2 = to_f64_channel(&args[1]);
    let c3 = to_f64_channel(&args[2]);

    let alpha_val = if args.len() == 4 {
        match args[3] {
            ColorArg::Number(v) => v,
            ColorArg::Percentage(v) => v / 100.0,
        }
    } else {
        1.0
    };
    let alpha = (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8;

    let (r, g, b) = match colorspace.to_ascii_lowercase().as_str() {
        "srgb" => {
            let r = (c1 * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (c2 * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (c3 * 255.0).round().clamp(0.0, 255.0) as u8;
            (r, g, b)
        }
        "srgb-linear" => {
            let r = linear_srgb_to_srgb(c1);
            let g = linear_srgb_to_srgb(c2);
            let b = linear_srgb_to_srgb(c3);
            (r, g, b)
        }
        "display-p3" => {
            let r_lin = 1.2249401 * c1 - 0.2249404 * c2 + 0.0000000 * c3;
            let g_lin = -0.0420569 * c1 + 1.0420571 * c2 + 0.0000000 * c3;
            let b_lin = -0.0197376 * c1 - 0.0786361 * c2 + 1.0983735 * c3;
            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        "xyz" | "xyz-d65" => {
            let r_lin = 3.24096994 * c1 - 1.53738318 * c2 - 0.49861076 * c3;
            let g_lin = -0.96924364 * c1 + 1.87596750 * c2 + 0.04155506 * c3;
            let b_lin = 0.05563008 * c1 - 0.20397696 * c2 + 1.05697151 * c3;
            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        "xyz-d50" => {
            let r_lin = 3.1338561 * c1 - 1.6168667 * c2 - 0.4906146 * c3;
            let g_lin = -0.9787684 * c1 + 1.9161415 * c2 + 0.0334540 * c3;
            let b_lin = 0.0719453 * c1 - 0.2289914 * c2 + 1.4052427 * c3;
            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        "rec2020" => {
            let r_lin_rec = rec2020_to_linear(c1);
            let g_lin_rec = rec2020_to_linear(c2);
            let b_lin_rec = rec2020_to_linear(c3);

            let x = 0.6369580 * r_lin_rec + 0.1446169 * g_lin_rec + 0.1688810 * b_lin_rec;
            let y = 0.2627002 * r_lin_rec + 0.6779981 * g_lin_rec + 0.0593017 * b_lin_rec;
            let z = 0.0000000 * r_lin_rec + 0.0280727 * g_lin_rec + 1.0609851 * b_lin_rec;

            let r_lin = 3.24096994 * x - 1.53738318 * y - 0.49861076 * z;
            let g_lin = -0.96924364 * x + 1.87596750 * y + 0.04155506 * z;
            let b_lin = 0.05563008 * x - 0.20397696 * y + 1.05697151 * z;

            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        "a98-rgb" => {
            let r_lin_a98 = a98_to_linear(c1);
            let g_lin_a98 = a98_to_linear(c2);
            let b_lin_a98 = a98_to_linear(c3);

            let x = 0.5767309 * r_lin_a98 + 0.1855540 * g_lin_a98 + 0.1881852 * b_lin_a98;
            let y = 0.2973769 * r_lin_a98 + 0.6273491 * g_lin_a98 + 0.0752741 * b_lin_a98;
            let z = 0.0270343 * r_lin_a98 + 0.0706872 * g_lin_a98 + 0.9911085 * b_lin_a98;

            let r_lin = 3.24096994 * x - 1.53738318 * y - 0.49861076 * z;
            let g_lin = -0.96924364 * x + 1.87596750 * y + 0.04155506 * z;
            let b_lin = 0.05563008 * x - 0.20397696 * y + 1.05697151 * z;

            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        "prophoto-rgb" => {
            let r_lin_pro = prophoto_to_linear(c1);
            let g_lin_pro = prophoto_to_linear(c2);
            let b_lin_pro = prophoto_to_linear(c3);

            let x_d50 = 0.7976749 * r_lin_pro + 0.1351917 * g_lin_pro + 0.0313534 * b_lin_pro;
            let y_d50 = 0.2880402 * r_lin_pro + 0.7118741 * g_lin_pro + 0.0000857 * b_lin_pro;
            let z_d50 = 0.0000000 * r_lin_pro + 0.0000000 * g_lin_pro + 0.8252100 * b_lin_pro;

            let r_lin = 3.1338561 * x_d50 - 1.6168667 * y_d50 - 0.4906146 * z_d50;
            let g_lin = -0.9787684 * x_d50 + 1.9161415 * y_d50 + 0.0334540 * z_d50;
            let b_lin = 0.0719453 * x_d50 - 0.2289914 * y_d50 + 1.4052427 * z_d50;

            let r = linear_srgb_to_srgb(r_lin);
            let g = linear_srgb_to_srgb(g_lin);
            let b = linear_srgb_to_srgb(b_lin);
            (r, g, b)
        }
        _ => {
            return None;
        }
    };

    Some(Color::Rgba(r, g, b, alpha))
}

fn split_by_comma(components: &[ComponentValue]) -> Vec<&[ComponentValue]> {
    let mut args = Vec::new();
    let mut start = 0;
    for (i, comp) in components.iter().enumerate() {
        if matches!(comp, ComponentValue::Token(CssToken::Comma)) {
            args.push(&components[start..i]);
            start = i + 1;
        }
    }
    args.push(&components[start..]);
    args
}

fn parse_colorspace(components: &[ComponentValue]) -> Option<String> {
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    if non_ws.len() != 2 {
        return None;
    }

    let is_in = match non_ws[0] {
        ComponentValue::Token(CssToken::Ident(s)) => s.eq_ignore_ascii_case("in"),
        _ => false,
    };

    if !is_in {
        return None;
    }

    match non_ws[1] {
        ComponentValue::Token(CssToken::Ident(s)) => Some(s.clone()),
        _ => None,
    }
}

fn parse_color_with_optional_percentage(
    components: &[ComponentValue],
) -> Option<(Color, Option<f64>)> {
    // Filter out whitespace
    let non_ws: Vec<&ComponentValue> = components
        .iter()
        .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
        .collect();

    match non_ws.len() {
        1 => {
            // It must be a color
            let css_val = parse_single_value(&[non_ws[0]])?;
            if let CssValue::Color(color) = css_val {
                Some((color, None))
            } else {
                None
            }
        }
        2 => {
            // One must be a color, one must be a percentage
            let p0 = match non_ws[0] {
                ComponentValue::Token(CssToken::Percentage(p)) => Some(*p),
                _ => None,
            };
            let p1 = match non_ws[1] {
                ComponentValue::Token(CssToken::Percentage(p)) => Some(*p),
                _ => None,
            };

            match (p0, p1) {
                (Some(p), None) => {
                    let css_val = parse_single_value(&[non_ws[1]])?;
                    if let CssValue::Color(color) = css_val {
                        Some((color, Some(p)))
                    } else {
                        None
                    }
                }
                (None, Some(p)) => {
                    let css_val = parse_single_value(&[non_ws[0]])?;
                    if let CssValue::Color(color) = css_val {
                        Some((color, Some(p)))
                    } else {
                        None
                    }
                }
                _ => None, // Invalid: either both or neither are percentages
            }
        }
        _ => None,
    }
}

fn parse_color_mix_function(components: &[ComponentValue]) -> Option<Color> {
    let args = split_by_comma(components);
    if args.len() != 3 {
        return None;
    }

    let colorspace = parse_colorspace(args[0])?;
    let is_linear = if colorspace.eq_ignore_ascii_case("srgb") {
        false
    } else if colorspace.eq_ignore_ascii_case("srgb-linear") {
        true
    } else {
        // TODO(spec): Support non-srgb interpolation colorspaces in color-mix()
        return None;
    };

    let (color1, p1) = parse_color_with_optional_percentage(args[1])?;
    let (color2, p2) = parse_color_with_optional_percentage(args[2])?;

    // Determine weights
    let (w1, w2) = match (p1, p2) {
        (None, None) => (50.0, 50.0),
        (Some(p), None) => (p, 100.0 - p),
        (None, Some(p)) => (100.0 - p, p),
        (Some(p1_val), Some(p2_val)) => (p1_val, p2_val),
    };

    let sum = w1 + w2;
    if sum <= 0.0 {
        return None;
    }

    let weight1 = w1 / sum;
    let weight2 = w2 / sum;

    let alpha_multiplier = if sum < 100.0 { sum / 100.0 } else { 1.0 };

    // Convert colors to sRGB float channels
    let Color::Rgba(r1_u8, g1_u8, b1_u8, a1_u8) = color1;
    let Color::Rgba(r2_u8, g2_u8, b2_u8, a2_u8) = color2;

    let (r1, g1, b1) = if is_linear {
        (
            srgb_to_linear_srgb(r1_u8),
            srgb_to_linear_srgb(g1_u8),
            srgb_to_linear_srgb(b1_u8),
        )
    } else {
        (
            r1_u8 as f64 / 255.0,
            g1_u8 as f64 / 255.0,
            b1_u8 as f64 / 255.0,
        )
    };
    let a1 = a1_u8 as f64 / 255.0;

    let (r2, g2, b2) = if is_linear {
        (
            srgb_to_linear_srgb(r2_u8),
            srgb_to_linear_srgb(g2_u8),
            srgb_to_linear_srgb(b2_u8),
        )
    } else {
        (
            r2_u8 as f64 / 255.0,
            g2_u8 as f64 / 255.0,
            b2_u8 as f64 / 255.0,
        )
    };
    let a2 = a2_u8 as f64 / 255.0;

    // Premultiply
    let pr1 = r1 * a1;
    let pg1 = g1 * a1;
    let pb1 = b1 * a1;

    let pr2 = r2 * a2;
    let pg2 = g2 * a2;
    let pb2 = b2 * a2;

    // Linearly interpolate
    let mixed_pr = pr1 * weight1 + pr2 * weight2;
    let mixed_pg = pg1 * weight1 + pg2 * weight2;
    let mixed_pb = pb1 * weight1 + pb2 * weight2;
    let mixed_a = a1 * weight1 + a2 * weight2;

    // Un-premultiply
    let (mixed_r, mixed_g, mixed_b) = if mixed_a > 0.0 {
        (mixed_pr / mixed_a, mixed_pg / mixed_a, mixed_pb / mixed_a)
    } else {
        (0.0, 0.0, 0.0)
    };

    let final_a = mixed_a * alpha_multiplier;

    // Convert back to u8 RGBA
    let (r_out, g_out, b_out) = if is_linear {
        (
            linear_srgb_to_srgb(mixed_r),
            linear_srgb_to_srgb(mixed_g),
            linear_srgb_to_srgb(mixed_b),
        )
    } else {
        (
            (mixed_r * 255.0).round().clamp(0.0, 255.0) as u8,
            (mixed_g * 255.0).round().clamp(0.0, 255.0) as u8,
            (mixed_b * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    };
    let a_out = (final_a * 255.0).round().clamp(0.0, 255.0) as u8;

    Some(Color::Rgba(r_out, g_out, b_out, a_out))
}

fn parse_color_argument(components: &[ComponentValue]) -> Option<Color> {
    let (color, p) = parse_color_with_optional_percentage(components)?;
    if p.is_some() {
        return None;
    }
    Some(color)
}

fn parse_light_dark_function(components: &[ComponentValue]) -> Option<Color> {
    let args = split_by_comma(components);
    if args.len() != 2 {
        return None;
    }

    let first_color = parse_color_argument(args[0])?;
    let _second_color = parse_color_argument(args[1])?;

    Some(first_color)
}

fn parse_device_cmyk_function(components: &[ComponentValue]) -> Option<Color> {
    enum CmykArg {
        Number(f64),
        Percentage(f64),
    }

    let mut args = Vec::new();
    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace)
            | ComponentValue::Token(CssToken::Comma)
            | ComponentValue::Token(CssToken::Delim('/')) => {}
            ComponentValue::Token(CssToken::Number(v)) => args.push(CmykArg::Number(*v)),
            ComponentValue::Token(CssToken::Percentage(v)) => args.push(CmykArg::Percentage(*v)),
            _ => return None,
        }
    }

    if args.len() != 4 && args.len() != 5 {
        return None;
    }

    // Parse Cyan (C)
    let c_val = match args[0] {
        CmykArg::Number(v) => v,
        CmykArg::Percentage(v) => v / 100.0,
    };
    let c = c_val.clamp(0.0, 1.0);

    // Parse Magenta (M)
    let m_val = match args[1] {
        CmykArg::Number(v) => v,
        CmykArg::Percentage(v) => v / 100.0,
    };
    let m = m_val.clamp(0.0, 1.0);

    // Parse Yellow (Y)
    let y_val = match args[2] {
        CmykArg::Number(v) => v,
        CmykArg::Percentage(v) => v / 100.0,
    };
    let y = y_val.clamp(0.0, 1.0);

    // Parse Key/Black (K)
    let k_val = match args[3] {
        CmykArg::Number(v) => v,
        CmykArg::Percentage(v) => v / 100.0,
    };
    let k = k_val.clamp(0.0, 1.0);

    // Parse Alpha (A)
    let alpha = if args.len() == 5 {
        let alpha_val = match args[4] {
            CmykArg::Number(v) => v,
            CmykArg::Percentage(v) => v / 100.0,
        };
        (alpha_val.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };

    // Naive sRGB conversion (device-cmyk has no colorimetric definition without a profile;
    // use the standard naive fallback, which is what browsers use without an ICC profile):
    // TODO(spec): Naive profile-less device-cmyk fallback
    let r = (255.0 * (1.0 - c) * (1.0 - k)).round().clamp(0.0, 255.0) as u8;
    let g = (255.0 * (1.0 - m) * (1.0 - k)).round().clamp(0.0, 255.0) as u8;
    let b = (255.0 * (1.0 - y) * (1.0 - k)).round().clamp(0.0, 255.0) as u8;

    Some(Color::Rgba(r, g, b, alpha))
}

fn is_css_image(comp: &ComponentValue) -> bool {
    match comp {
        ComponentValue::Token(CssToken::Url(_)) | ComponentValue::Token(CssToken::String(_)) => {
            true
        }
        ComponentValue::Function { name, .. } => {
            let name_lower = name.to_ascii_lowercase();
            name_lower == "url"
                || name_lower == "image-set"
                || name_lower == "-webkit-image-set"
                || name_lower == "cross-fade"
                || name_lower == "linear-gradient"
                || name_lower == "radial-gradient"
                || name_lower == "conic-gradient"
                || name_lower == "repeating-linear-gradient"
                || name_lower == "repeating-radial-gradient"
                || name_lower == "repeating-conic-gradient"
        }
        _ => false,
    }
}

fn parse_cross_fade_arg(components: &[&ComponentValue]) -> Option<String> {
    if components.is_empty() || components.len() > 2 {
        return None;
    }
    let mut has_image = false;
    let mut has_percentage = false;
    for comp in components {
        match comp {
            ComponentValue::Token(CssToken::Percentage(_)) => {
                if has_percentage {
                    return None;
                }
                has_percentage = true;
            }
            comp if is_css_image(comp) => {
                if has_image {
                    return None;
                }
                has_image = true;
            }
            _ => return None,
        }
    }
    if !has_image && !has_percentage {
        return None;
    }
    let mut s = String::new();
    for (i, comp) in components.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&serialize_component_value(comp));
    }
    Some(s)
}

fn parse_cross_fade_function(components: &[ComponentValue]) -> Option<CssValue> {
    let args = split_by_comma(components);
    if args.is_empty() {
        return None;
    }
    let mut serialized_args = Vec::new();
    for arg in args {
        let non_ws: Vec<&ComponentValue> = arg
            .iter()
            .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
            .collect();
        if non_ws.is_empty() {
            return None;
        }
        let parsed_arg = parse_cross_fade_arg(&non_ws)?;
        serialized_args.push(parsed_arg);
    }
    Some(CssValue::Keyword(format!(
        "cross-fade({})",
        serialized_args.join(", ")
    )))
}

fn relative_luminance(color: &Color) -> f64 {
    let Color::Rgba(r_u8, g_u8, b_u8, _) = color;
    let r_srgb = *r_u8 as f64 / 255.0;
    let g_srgb = *g_u8 as f64 / 255.0;
    let b_srgb = *b_u8 as f64 / 255.0;

    let r = if r_srgb <= 0.03928 {
        r_srgb / 12.92
    } else {
        ((r_srgb + 0.055) / 1.055).powf(2.4)
    };
    let g = if g_srgb <= 0.03928 {
        g_srgb / 12.92
    } else {
        ((g_srgb + 0.055) / 1.055).powf(2.4)
    };
    let b = if b_srgb <= 0.03928 {
        b_srgb / 12.92
    } else {
        ((b_srgb + 0.055) / 1.055).powf(2.4)
    };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(lum1: f64, lum2: f64) -> f64 {
    let l1 = lum1.max(lum2);
    let l2 = lum1.min(lum2);
    (l1 + 0.05) / (l2 + 0.05)
}

fn parse_color_contrast_function(components: &[ComponentValue]) -> Option<Color> {
    let args = split_by_comma(components);
    if args.is_empty() {
        return None;
    }

    let first_arg_comps = args[0];
    let vs_index = first_arg_comps.iter().position(|comp| {
        if let ComponentValue::Token(CssToken::Ident(s)) = comp {
            s.eq_ignore_ascii_case("vs")
        } else {
            false
        }
    })?;

    let base_color_comps = &first_arg_comps[..vs_index];
    let first_candidate_comps = &first_arg_comps[vs_index + 1..];

    let base_color = parse_color_argument(base_color_comps)?;
    let first_candidate = parse_color_argument(first_candidate_comps)?;

    let mut candidates = vec![first_candidate];
    let mut target_contrast: Option<String> = None;

    let num_args = args.len();
    for (i, arg_comps) in args.iter().skip(1).enumerate() {
        let is_last = i + 2 == num_args;
        if is_last {
            let to_index = arg_comps.iter().position(|comp| {
                if let ComponentValue::Token(CssToken::Ident(s)) = comp {
                    s.eq_ignore_ascii_case("to")
                } else {
                    false
                }
            });

            if let Some(to_idx) = to_index {
                let candidate_comps = &arg_comps[..to_idx];
                let target_comps = &arg_comps[to_idx + 1..];

                let target_non_ws: Vec<&ComponentValue> = target_comps
                    .iter()
                    .filter(|comp| !matches!(comp, ComponentValue::Token(CssToken::Whitespace)))
                    .collect();

                if target_non_ws.len() != 1 {
                    return None;
                }
                match target_non_ws[0] {
                    ComponentValue::Token(CssToken::Number(v)) => {
                        target_contrast = Some(v.to_string());
                    }
                    ComponentValue::Token(CssToken::Ident(s)) => {
                        let lower = s.to_ascii_lowercase();
                        if lower == "aa"
                            || lower == "aaa"
                            || lower == "aalarge"
                            || lower == "aaalarge"
                        {
                            target_contrast = Some(lower);
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                }

                let candidate_color = parse_color_argument(candidate_comps)?;
                candidates.push(candidate_color);
            } else {
                let candidate_color = parse_color_argument(arg_comps)?;
                candidates.push(candidate_color);
            }
        } else {
            let candidate_color = parse_color_argument(arg_comps)?;
            candidates.push(candidate_color);
        }
    }

    let target_val = target_contrast.as_ref().and_then(|t| {
        if let Ok(num) = t.parse::<f64>() {
            Some(num)
        } else {
            match t.as_str() {
                "aa" => Some(4.5),
                "aalarge" => Some(3.0),
                "aaa" => Some(7.0),
                "aaalarge" => Some(4.5),
                _ => None,
            }
        }
    });

    let base_lum = relative_luminance(&base_color);

    if let Some(target) = target_val {
        for candidate in &candidates {
            let cand_lum = relative_luminance(candidate);
            let contrast = contrast_ratio(base_lum, cand_lum);
            if contrast >= target {
                return Some(candidate.clone());
            }
        }
    }

    let mut best_candidate = candidates[0].clone();
    let mut max_contrast = -1.0;

    for candidate in candidates {
        let cand_lum = relative_luminance(&candidate);
        let contrast = contrast_ratio(base_lum, cand_lum);
        if contrast > max_contrast {
            max_contrast = contrast;
            best_candidate = candidate;
        }
    }

    Some(best_candidate)
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
            let lower_unit = unit.to_ascii_lowercase();
            let (val, unit_enum) = match lower_unit.as_str() {
                "px" => (*value as f32, LengthUnit::Px),
                "em" => (*value as f32, LengthUnit::Em),
                "rem" => (*value as f32, LengthUnit::Rem),
                "pt" => (*value as f32, LengthUnit::Pt),
                "vw" => (*value as f32, LengthUnit::Vw),
                "vh" => (*value as f32, LengthUnit::Vh),
                "in" => (*value as f32 * 96.0, LengthUnit::Px),
                "cm" => (*value as f32 * 96.0 / 2.54, LengthUnit::Px),
                "mm" => (*value as f32 * 9.6 / 2.54, LengthUnit::Px),
                "pc" => (*value as f32 * 16.0, LengthUnit::Px),
                "q" => (*value as f32 * 96.0 / 101.6, LengthUnit::Px),
                _ => return None,
            };
            Some(LengthOrPercent {
                value: val,
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
        "matrix" => {
            if args.len() == 6 {
                let a = parse_number(args[0])?;
                let b = parse_number(args[1])?;
                let c = parse_number(args[2])?;
                let d = parse_number(args[3])?;
                let e = parse_number(args[4])?;
                let f = parse_number(args[5])?;
                Some(TransformFn::Matrix([a, b, c, d, e, f]))
            } else {
                None
            }
        }
        "skew" => {
            if args.len() == 1 {
                let ax = parse_angle(args[0])?;
                let rad_x = ax.0 * std::f32::consts::PI / 180.0;
                let tan_x = rad_x.tan();
                Some(TransformFn::Matrix([1.0, 0.0, tan_x, 1.0, 0.0, 0.0]))
            } else if args.len() == 2 {
                let ax = parse_angle(args[0])?;
                let ay = parse_angle(args[1])?;
                let rad_x = ax.0 * std::f32::consts::PI / 180.0;
                let rad_y = ay.0 * std::f32::consts::PI / 180.0;
                let tan_x = rad_x.tan();
                let tan_y = rad_y.tan();
                Some(TransformFn::Matrix([1.0, tan_y, tan_x, 1.0, 0.0, 0.0]))
            } else {
                None
            }
        }
        "skewx" => {
            if args.len() == 1 {
                let ax = parse_angle(args[0])?;
                let rad_x = ax.0 * std::f32::consts::PI / 180.0;
                let tan_x = rad_x.tan();
                Some(TransformFn::Matrix([1.0, 0.0, tan_x, 1.0, 0.0, 0.0]))
            } else {
                None
            }
        }
        "skewy" => {
            if args.len() == 1 {
                let ay = parse_angle(args[0])?;
                let rad_y = ay.0 * std::f32::consts::PI / 180.0;
                let tan_y = rad_y.tan();
                Some(TransformFn::Matrix([1.0, tan_y, 0.0, 1.0, 0.0, 0.0]))
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
    fn test_parse_easing_functions() {
        // Test cubic-bezier
        // Valid
        let cb_valid = ComponentValue::Function {
            name: "cubic-bezier".to_string(),
            value: vec![
                token(CssToken::Number(0.25)),
                token(CssToken::Comma),
                token(CssToken::Number(0.1)),
                token(CssToken::Comma),
                token(CssToken::Number(0.25)),
                token(CssToken::Comma),
                token(CssToken::Number(1.0)),
            ],
        };
        assert_eq!(
            parse_value(&[cb_valid]),
            Some(CssValue::Keyword(
                "cubic-bezier(0.25, 0.1, 0.25, 1)".to_string()
            ))
        );

        // Invalid: x coordinate out of range [0, 1]
        let cb_invalid_x1 = ComponentValue::Function {
            name: "cubic-bezier".to_string(),
            value: vec![
                token(CssToken::Number(-0.1)),
                token(CssToken::Comma),
                token(CssToken::Number(0.1)),
                token(CssToken::Comma),
                token(CssToken::Number(0.25)),
                token(CssToken::Comma),
                token(CssToken::Number(1.0)),
            ],
        };
        assert_eq!(parse_value(&[cb_invalid_x1]), None);

        let cb_invalid_x2 = ComponentValue::Function {
            name: "cubic-bezier".to_string(),
            value: vec![
                token(CssToken::Number(0.25)),
                token(CssToken::Comma),
                token(CssToken::Number(0.1)),
                token(CssToken::Comma),
                token(CssToken::Number(1.5)),
                token(CssToken::Comma),
                token(CssToken::Number(1.0)),
            ],
        };
        assert_eq!(parse_value(&[cb_invalid_x2]), None);

        // Test steps
        // steps(4, end)
        let steps_two_args = ComponentValue::Function {
            name: "steps".to_string(),
            value: vec![
                token(CssToken::Number(4.0)),
                token(CssToken::Comma),
                token(CssToken::Ident("end".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[steps_two_args]),
            Some(CssValue::Keyword("steps(4, end)".to_string()))
        );

        // steps(4) (implicit position)
        let steps_one_arg = ComponentValue::Function {
            name: "steps".to_string(),
            value: vec![token(CssToken::Number(4.0))],
        };
        assert_eq!(
            parse_value(&[steps_one_arg]),
            Some(CssValue::Keyword("steps(4)".to_string()))
        );

        // steps(4, jump-none)
        let steps_jump_none = ComponentValue::Function {
            name: "steps".to_string(),
            value: vec![
                token(CssToken::Number(4.0)),
                token(CssToken::Comma),
                token(CssToken::Ident("jump-none".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[steps_jump_none]),
            Some(CssValue::Keyword("steps(4, jump-none)".to_string()))
        );

        // steps invalid: non-positive integer
        let steps_invalid_n = ComponentValue::Function {
            name: "steps".to_string(),
            value: vec![
                token(CssToken::Number(0.0)),
                token(CssToken::Comma),
                token(CssToken::Ident("end".to_string())),
            ],
        };
        assert_eq!(parse_value(&[steps_invalid_n]), None);

        let steps_invalid_fraction = ComponentValue::Function {
            name: "steps".to_string(),
            value: vec![
                token(CssToken::Number(4.5)),
                token(CssToken::Comma),
                token(CssToken::Ident("end".to_string())),
            ],
        };
        assert_eq!(parse_value(&[steps_invalid_fraction]), None);

        // Test linear
        // linear(0, 0.25, 1)
        let linear_valid = ComponentValue::Function {
            name: "linear".to_string(),
            value: vec![
                token(CssToken::Number(0.0)),
                token(CssToken::Comma),
                token(CssToken::Number(0.25)),
                token(CssToken::Comma),
                token(CssToken::Number(1.0)),
            ],
        };
        assert_eq!(
            parse_value(&[linear_valid]),
            Some(CssValue::Keyword("linear(0, 0.25, 1)".to_string()))
        );

        // linear stops with percentages
        let linear_pct = ComponentValue::Function {
            name: "linear".to_string(),
            value: vec![
                token(CssToken::Number(0.0)),
                token(CssToken::Percentage(0.0)),
                token(CssToken::Comma),
                token(CssToken::Number(0.25)),
                token(CssToken::Percentage(25.0)),
                token(CssToken::Percentage(50.0)),
                token(CssToken::Comma),
                token(CssToken::Number(1.0)),
                token(CssToken::Percentage(100.0)),
            ],
        };
        assert_eq!(
            parse_value(&[linear_pct]),
            Some(CssValue::Keyword(
                "linear(0 0%, 0.25 25% 50%, 1 100%)".to_string()
            ))
        );

        // invalid linear: only one stop
        let linear_invalid_len = ComponentValue::Function {
            name: "linear".to_string(),
            value: vec![token(CssToken::Number(0.0))],
        };
        assert_eq!(parse_value(&[linear_invalid_len]), None);
    }

    #[test]
    fn test_parse_anchor_functions() {
        // anchor(top)
        let a_top = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![token(CssToken::Ident("top".to_string()))],
        };
        assert_eq!(
            parse_value(&[a_top]),
            Some(CssValue::Keyword("anchor(top)".to_string()))
        );

        // anchor(--my-anchor left)
        let a_with_name = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("--my-anchor".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("left".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[a_with_name]),
            Some(CssValue::Keyword("anchor(--my-anchor left)".to_string()))
        );

        // anchor(right, 10px)
        let a_with_fallback = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("right".to_string())),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[a_with_fallback]),
            Some(CssValue::Keyword("anchor(right, 10px)".to_string()))
        );

        // anchor(--my-anchor bottom, 50%)
        let a_all = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("--my-anchor".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("bottom".to_string())),
                token(CssToken::Comma),
                token(CssToken::Percentage(50.0)),
            ],
        };
        assert_eq!(
            parse_value(&[a_all]),
            Some(CssValue::Keyword(
                "anchor(--my-anchor bottom, 50%)".to_string()
            ))
        );

        // anchor(--my-anchor 10%)
        let a_pct = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("--my-anchor".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(10.0)),
            ],
        };
        assert_eq!(
            parse_value(&[a_pct]),
            Some(CssValue::Keyword("anchor(--my-anchor 10%)".to_string()))
        );

        // anchor(top, 0)
        let a_zero = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("top".to_string())),
                token(CssToken::Comma),
                token(CssToken::Number(0.0)),
            ],
        };
        assert_eq!(
            parse_value(&[a_zero]),
            Some(CssValue::Keyword("anchor(top, 0)".to_string()))
        );

        // anchor(top, calc(10% + 5px))
        let a_calc = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("top".to_string())),
                token(CssToken::Comma),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Percentage(10.0)),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('+')),
                        token(CssToken::Whitespace),
                        token(CssToken::Dimension {
                            value: 5.0,
                            unit: "px".to_string(),
                        }),
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[a_calc]),
            Some(CssValue::Keyword(
                "anchor(top, calc(10% + 5px))".to_string()
            ))
        );

        // Malformed anchor
        let a_empty = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![],
        };
        assert_eq!(parse_value(&[a_empty]), None);

        let a_missing_side = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![token(CssToken::Ident("--my-anchor".to_string()))],
        };
        assert_eq!(parse_value(&[a_missing_side]), None);

        let a_invalid_two_sides = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("top".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("left".to_string())),
            ],
        };
        assert_eq!(parse_value(&[a_invalid_two_sides]), None);

        let a_multiple_fallbacks = ComponentValue::Function {
            name: "anchor".to_string(),
            value: vec![
                token(CssToken::Ident("top".to_string())),
                token(CssToken::Comma),
                token(CssToken::Percentage(10.0)),
                token(CssToken::Comma),
                token(CssToken::Percentage(20.0)),
            ],
        };
        assert_eq!(parse_value(&[a_multiple_fallbacks]), None);

        // anchor-size(width)
        let as_width = ComponentValue::Function {
            name: "anchor-size".to_string(),
            value: vec![token(CssToken::Ident("width".to_string()))],
        };
        assert_eq!(
            parse_value(&[as_width]),
            Some(CssValue::Keyword("anchor-size(width)".to_string()))
        );

        // anchor-size(--my-anchor height)
        let as_with_name = ComponentValue::Function {
            name: "anchor-size".to_string(),
            value: vec![
                token(CssToken::Ident("--my-anchor".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("height".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[as_with_name]),
            Some(CssValue::Keyword(
                "anchor-size(--my-anchor height)".to_string()
            ))
        );

        // anchor-size(block, 20px)
        let as_with_fallback = ComponentValue::Function {
            name: "anchor-size".to_string(),
            value: vec![
                token(CssToken::Ident("block".to_string())),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[as_with_fallback]),
            Some(CssValue::Keyword("anchor-size(block, 20px)".to_string()))
        );

        // Malformed anchor-size
        let as_invalid_side = ComponentValue::Function {
            name: "anchor-size".to_string(),
            value: vec![token(CssToken::Ident("top".to_string()))],
        };
        assert_eq!(parse_value(&[as_invalid_side]), None);
    }

    #[test]
    fn test_parse_image_set_functions() {
        // image-set("a.png" 1x, "b.png" 2x)
        let is_valid1 = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![
                token(CssToken::String("a.png".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "x".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Whitespace),
                token(CssToken::String("b.png".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 2.0,
                    unit: "x".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[is_valid1]),
            Some(CssValue::Keyword("url(a.png)".to_string()))
        );

        // image-set(url(a.png) 1x, url(b.png) 2x)
        let is_valid2 = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![
                ComponentValue::Function {
                    name: "url".to_string(),
                    value: vec![token(CssToken::Ident("a.png".to_string()))],
                },
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "x".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "url".to_string(),
                    value: vec![token(CssToken::Ident("b.png".to_string()))],
                },
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 2.0,
                    unit: "x".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[is_valid2]),
            Some(CssValue::Keyword("url(a.png)".to_string()))
        );

        // -webkit-image-set(url(a.png) 1x)
        let is_valid3 = ComponentValue::Function {
            name: "-webkit-image-set".to_string(),
            value: vec![
                ComponentValue::Function {
                    name: "url".to_string(),
                    value: vec![token(CssToken::Ident("a.png".to_string()))],
                },
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "x".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[is_valid3]),
            Some(CssValue::Keyword("url(a.png)".to_string()))
        );

        // image-set(url(a.png) type("image/png"), url(b.png))
        let is_valid4 = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![
                ComponentValue::Function {
                    name: "url".to_string(),
                    value: vec![token(CssToken::Ident("a.png".to_string()))],
                },
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "type".to_string(),
                    value: vec![token(CssToken::String("image/png".to_string()))],
                },
                token(CssToken::Comma),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "url".to_string(),
                    value: vec![token(CssToken::Ident("b.png".to_string()))],
                },
            ],
        };
        assert_eq!(
            parse_value(&[is_valid4]),
            Some(CssValue::Keyword("url(a.png)".to_string()))
        );

        // malformed: empty
        let is_invalid_empty = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![],
        };
        assert_eq!(parse_value(&[is_invalid_empty]), None);

        // malformed: no image source
        let is_invalid_no_source = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![token(CssToken::Dimension {
                value: 1.0,
                unit: "x".to_string(),
            })],
        };
        assert_eq!(parse_value(&[is_invalid_no_source]), None);

        // malformed: multiple resolutions in one option
        let is_invalid_multiple_res = ComponentValue::Function {
            name: "image-set".to_string(),
            value: vec![
                token(CssToken::String("a.png".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "x".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 2.0,
                    unit: "x".to_string(),
                }),
            ],
        };
        assert_eq!(parse_value(&[is_invalid_multiple_res]), None);
    }

    #[test]
    fn test_property_validation_timing_functions() {
        // Validate transitions and animations with timing functions
        for prop in &["transition-timing-function", "animation-timing-function"] {
            // cubic-bezier
            let components = [ComponentValue::Function {
                name: "cubic-bezier".to_string(),
                value: vec![
                    token(CssToken::Number(0.25)),
                    token(CssToken::Comma),
                    token(CssToken::Number(0.1)),
                    token(CssToken::Comma),
                    token(CssToken::Number(0.25)),
                    token(CssToken::Comma),
                    token(CssToken::Number(1.0)),
                ],
            }];
            let val = parse_property_value(prop, &components);
            assert_eq!(
                val,
                Some(CssValue::Keyword(
                    "cubic-bezier(0.25, 0.1, 0.25, 1)".to_string()
                ))
            );
            assert!(is_valid_property_value(prop, &val.unwrap()));

            // steps
            let components_steps = [ComponentValue::Function {
                name: "steps".to_string(),
                value: vec![
                    token(CssToken::Number(4.0)),
                    token(CssToken::Comma),
                    token(CssToken::Ident("jump-none".to_string())),
                ],
            }];
            let val_steps = parse_property_value(prop, &components_steps);
            assert_eq!(
                val_steps,
                Some(CssValue::Keyword("steps(4, jump-none)".to_string()))
            );
            assert!(is_valid_property_value(prop, &val_steps.unwrap()));

            // linear()
            let components_linear = [ComponentValue::Function {
                name: "linear".to_string(),
                value: vec![
                    token(CssToken::Number(0.0)),
                    token(CssToken::Comma),
                    token(CssToken::Number(0.25)),
                    token(CssToken::Comma),
                    token(CssToken::Number(1.0)),
                ],
            }];
            let val_linear = parse_property_value(prop, &components_linear);
            assert_eq!(
                val_linear,
                Some(CssValue::Keyword("linear(0, 0.25, 1)".to_string()))
            );
            assert!(is_valid_property_value(prop, &val_linear.unwrap()));
        }
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
    fn test_text_rendering() {
        let cases = [
            ("auto", Some(TextRenderingValue::Auto)),
            ("optimizeSpeed", Some(TextRenderingValue::OptimizeSpeed)),
            (
                "optimizeLegibility",
                Some(TextRenderingValue::OptimizeLegibility),
            ),
            (
                "geometricPrecision",
                Some(TextRenderingValue::GeometricPrecision),
            ),
            ("OPTIMIZESPEED", None), // Case-sensitive check
            ("nonsense", None),
        ];

        for (input, expected) in cases {
            let parsed = TextRenderingValue::parse(input);
            assert_eq!(parsed, expected);
            if let Some(val) = expected {
                assert_eq!(val.as_str(), input);
                let components = [token(CssToken::Ident(input.to_string()))];
                assert_eq!(
                    parse_property_value("text-rendering", &components),
                    Some(CssValue::TextRendering(val))
                );
            } else {
                let components = [token(CssToken::Ident(input.to_string()))];
                assert_eq!(parse_property_value("text-rendering", &components), None);
            }
        }
    }

    #[test]
    fn test_image_rendering() {
        use std::str::FromStr;

        // Test parsing and roundtrip via parse and FromStr
        assert_eq!(
            ImageRenderingValue::parse("crisp-edges"),
            Some(ImageRenderingValue::CrispEdges)
        );
        assert_eq!(
            ImageRenderingValue::from_str("crisp-edges"),
            Ok(ImageRenderingValue::CrispEdges)
        );
        assert_eq!(ImageRenderingValue::CrispEdges.as_str(), "crisp-edges");

        assert_eq!(
            ImageRenderingValue::parse("pixelated"),
            Some(ImageRenderingValue::Pixelated)
        );
        assert_eq!(
            ImageRenderingValue::from_str("pixelated"),
            Ok(ImageRenderingValue::Pixelated)
        );
        assert_eq!(ImageRenderingValue::Pixelated.as_str(), "pixelated");

        assert_eq!(
            ImageRenderingValue::parse("smooth"),
            Some(ImageRenderingValue::Smooth)
        );
        assert_eq!(
            ImageRenderingValue::from_str("smooth"),
            Ok(ImageRenderingValue::Smooth)
        );
        assert_eq!(ImageRenderingValue::Smooth.as_str(), "smooth");

        assert_eq!(
            ImageRenderingValue::parse("high-quality"),
            Some(ImageRenderingValue::HighQuality)
        );
        assert_eq!(
            ImageRenderingValue::from_str("high-quality"),
            Ok(ImageRenderingValue::HighQuality)
        );
        assert_eq!(ImageRenderingValue::HighQuality.as_str(), "high-quality");

        assert_eq!(
            ImageRenderingValue::parse("auto"),
            Some(ImageRenderingValue::Auto)
        );
        assert_eq!(
            ImageRenderingValue::from_str("auto"),
            Ok(ImageRenderingValue::Auto)
        );
        assert_eq!(ImageRenderingValue::Auto.as_str(), "auto");

        // Case-insensitivity check
        assert_eq!(
            ImageRenderingValue::parse("CRISP-EDGES"),
            Some(ImageRenderingValue::CrispEdges)
        );
        assert_eq!(
            ImageRenderingValue::from_str("CRISP-EDGES"),
            Ok(ImageRenderingValue::CrispEdges)
        );

        // Unknown keywords are rejected
        assert_eq!(ImageRenderingValue::parse("bogus"), None);
        assert_eq!(ImageRenderingValue::from_str("bogus"), Err(()));

        // Test TryFrom<&CssValue>
        let css_val_keyword = CssValue::Keyword("crisp-edges".to_string());
        assert_eq!(
            ImageRenderingValue::try_from(&css_val_keyword),
            Ok(ImageRenderingValue::CrispEdges)
        );

        let css_val_typed = CssValue::ImageRendering(ImageRenderingValue::Pixelated);
        assert_eq!(
            ImageRenderingValue::try_from(&css_val_typed),
            Ok(ImageRenderingValue::Pixelated)
        );

        let css_val_invalid = CssValue::Keyword("bogus".to_string());
        assert_eq!(ImageRenderingValue::try_from(&css_val_invalid), Err(()));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "image-rendering",
            &CssValue::ImageRendering(ImageRenderingValue::CrispEdges)
        ));
        assert!(is_valid_property_value(
            "image-rendering",
            &CssValue::Keyword("pixelated".to_string())
        ));
        assert!(!is_valid_property_value(
            "image-rendering",
            &CssValue::Keyword("bogus".to_string())
        ));
    }

    #[test]
    fn test_font_variant_caps() {
        use std::str::FromStr;

        // Test parsing and roundtrip via parse and FromStr
        assert_eq!(
            FontVariantCapsValue::parse("normal"),
            Some(FontVariantCapsValue::Normal)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("normal"),
            Ok(FontVariantCapsValue::Normal)
        );
        assert_eq!(FontVariantCapsValue::Normal.as_str(), "normal");

        assert_eq!(
            FontVariantCapsValue::parse("small-caps"),
            Some(FontVariantCapsValue::SmallCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("small-caps"),
            Ok(FontVariantCapsValue::SmallCaps)
        );
        assert_eq!(FontVariantCapsValue::SmallCaps.as_str(), "small-caps");

        assert_eq!(
            FontVariantCapsValue::parse("all-small-caps"),
            Some(FontVariantCapsValue::AllSmallCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("all-small-caps"),
            Ok(FontVariantCapsValue::AllSmallCaps)
        );
        assert_eq!(
            FontVariantCapsValue::AllSmallCaps.as_str(),
            "all-small-caps"
        );

        assert_eq!(
            FontVariantCapsValue::parse("petite-caps"),
            Some(FontVariantCapsValue::PetiteCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("petite-caps"),
            Ok(FontVariantCapsValue::PetiteCaps)
        );
        assert_eq!(FontVariantCapsValue::PetiteCaps.as_str(), "petite-caps");

        assert_eq!(
            FontVariantCapsValue::parse("all-petite-caps"),
            Some(FontVariantCapsValue::AllPetiteCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("all-petite-caps"),
            Ok(FontVariantCapsValue::AllPetiteCaps)
        );
        assert_eq!(
            FontVariantCapsValue::AllPetiteCaps.as_str(),
            "all-petite-caps"
        );

        assert_eq!(
            FontVariantCapsValue::parse("unicase"),
            Some(FontVariantCapsValue::Unicase)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("unicase"),
            Ok(FontVariantCapsValue::Unicase)
        );
        assert_eq!(FontVariantCapsValue::Unicase.as_str(), "unicase");

        assert_eq!(
            FontVariantCapsValue::parse("titling-caps"),
            Some(FontVariantCapsValue::TitlingCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("titling-caps"),
            Ok(FontVariantCapsValue::TitlingCaps)
        );
        assert_eq!(FontVariantCapsValue::TitlingCaps.as_str(), "titling-caps");

        // Case-insensitivity check
        assert_eq!(
            FontVariantCapsValue::parse("SMALL-CAPS"),
            Some(FontVariantCapsValue::SmallCaps)
        );
        assert_eq!(
            FontVariantCapsValue::from_str("SMALL-CAPS"),
            Ok(FontVariantCapsValue::SmallCaps)
        );

        // Unknown keywords are rejected
        assert_eq!(FontVariantCapsValue::parse("bogus"), None);
        assert_eq!(FontVariantCapsValue::from_str("bogus"), Err(()));

        // Test TryFrom<&CssValue>
        let css_val_keyword = CssValue::Keyword("small-caps".to_string());
        assert_eq!(
            FontVariantCapsValue::try_from(&css_val_keyword),
            Ok(FontVariantCapsValue::SmallCaps)
        );

        let css_val_typed = CssValue::FontVariantCaps(FontVariantCapsValue::Unicase);
        assert_eq!(
            FontVariantCapsValue::try_from(&css_val_typed),
            Ok(FontVariantCapsValue::Unicase)
        );

        let css_val_invalid = CssValue::Keyword("bogus".to_string());
        assert_eq!(FontVariantCapsValue::try_from(&css_val_invalid), Err(()));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "font-variant-caps",
            &CssValue::FontVariantCaps(FontVariantCapsValue::SmallCaps)
        ));
        assert!(is_valid_property_value(
            "font-variant-caps",
            &CssValue::Keyword("titling-caps".to_string())
        ));
        assert!(!is_valid_property_value(
            "font-variant-caps",
            &CssValue::Keyword("bogus".to_string())
        ));
    }

    #[test]
    fn test_font_stretch() {
        use std::str::FromStr;

        // Test parsing and roundtrip via parse and FromStr
        let keywords = [
            ("ultra-condensed", FontStretchValue::UltraCondensed),
            ("extra-condensed", FontStretchValue::ExtraCondensed),
            ("condensed", FontStretchValue::Condensed),
            ("semi-condensed", FontStretchValue::SemiCondensed),
            ("normal", FontStretchValue::Normal),
            ("semi-expanded", FontStretchValue::SemiExpanded),
            ("expanded", FontStretchValue::Expanded),
            ("extra-expanded", FontStretchValue::ExtraExpanded),
            ("ultra-expanded", FontStretchValue::UltraExpanded),
        ];

        for (name, variant) in keywords {
            assert_eq!(FontStretchValue::parse(name), Some(variant));
            assert_eq!(FontStretchValue::from_str(name), Ok(variant));
            assert_eq!(variant.as_str(), name);

            // Case insensitivity
            let uppercase = name.to_uppercase();
            assert_eq!(FontStretchValue::parse(&uppercase), Some(variant));
            assert_eq!(FontStretchValue::from_str(&uppercase), Ok(variant));
        }

        // Unknown keywords are rejected
        assert_eq!(FontStretchValue::parse("bogus"), None);
        assert_eq!(FontStretchValue::from_str("bogus"), Err(()));

        // Test TryFrom<&CssValue>
        let css_val_keyword = CssValue::Keyword("condensed".to_string());
        assert_eq!(
            FontStretchValue::try_from(&css_val_keyword),
            Ok(FontStretchValue::Condensed)
        );

        let css_val_typed = CssValue::FontStretch(FontStretchValue::SemiExpanded);
        assert_eq!(
            FontStretchValue::try_from(&css_val_typed),
            Ok(FontStretchValue::SemiExpanded)
        );

        let css_val_invalid = CssValue::Keyword("bogus".to_string());
        assert_eq!(FontStretchValue::try_from(&css_val_invalid), Err(()));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "font-stretch",
            &CssValue::FontStretch(FontStretchValue::Condensed)
        ));
        assert!(is_valid_property_value(
            "font-stretch",
            &CssValue::Keyword("expanded".to_string())
        ));
        assert!(!is_valid_property_value(
            "font-stretch",
            &CssValue::Keyword("bogus".to_string())
        ));
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

        // #f00f (4-digit hex: red, full alpha)
        let components = [token(CssToken::Hash("f00f".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // #f008 (4-digit hex: red, half alpha)
        let components = [token(CssToken::Hash("f008".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 136)))
        );

        // #ff0000ff (8-digit hex: red, full alpha)
        let components = [token(CssToken::Hash("ff0000ff".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // #ff000080 (8-digit hex: red, half alpha)
        let components = [token(CssToken::Hash("ff000080".to_string()))];
        assert_eq!(
            parse_value(&components),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 128)))
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
    fn test_parse_color_hwb() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // hwb(0 0% 0%) -> red
        assert_eq!(
            parse("hwb(0 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // HWB(0 0% 0%) -> case-insensitivity
        assert_eq!(
            parse("HWB(0 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // hwb(0 100% 0%) -> white
        assert_eq!(
            parse("hwb(0 100% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // hwb(0 0% 100%) -> black
        assert_eq!(
            parse("hwb(0 0% 100%)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // hwb(120 0% 0%) -> green
        assert_eq!(
            parse("hwb(120 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // hwb(0 50% 50%) -> gray (approx 128, 128, 128)
        let gray_color = parse("hwb(0 50% 50%)");
        match gray_color {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert!((r as i32 - 128).abs() <= 1);
                assert!((g as i32 - 128).abs() <= 1);
                assert!((b as i32 - 128).abs() <= 1);
                assert_eq!(alpha, 255);
            }
            _ => panic!("Expected hwb(0 50% 50%) to parse as gray"),
        }

        // hwb(0 0% 0% / 0.5) -> alpha approx 128
        let alpha_color = parse("hwb(0 0% 0% / 0.5)");
        match alpha_color {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 127).abs() <= 1);
            }
            _ => panic!("Expected hwb(0 0% 0% / 0.5) to parse as alpha color"),
        }

        // hwb(0 0% 0% / 50%) -> percentage alpha
        let alpha_pct = parse("hwb(0 0% 0% / 50%)");
        match alpha_pct {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 127).abs() <= 1);
            }
            _ => panic!("Expected hwb(0 0% 0% / 50%) to parse as percentage alpha"),
        }

        // Negative hues wrapping: hwb(-240 0% 0%) wraps to 120 (green)
        assert_eq!(
            parse("hwb(-240 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // Clamp behavior: white + black >= 100% (e.g. hwb(0 40% 70%)) -> 40/110 white
        let clamp_color = parse("hwb(0 40% 70%)");
        match clamp_color {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                let expected = (0.4_f64 / 1.1_f64 * 255.0_f64).round() as i32; // ~93
                assert!((r as i32 - expected).abs() <= 1);
                assert!((g as i32 - expected).abs() <= 1);
                assert!((b as i32 - expected).abs() <= 1);
                assert_eq!(alpha, 255);
            }
            _ => panic!("Expected clamp_color to parse"),
        }

        // Comma-separated: hwb(0, 0%, 0%) -> red
        assert_eq!(
            parse("hwb(0, 0%, 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // Rejecting bare numbers for W/B: hwb(0 0 0) -> None
        assert_eq!(parse("hwb(0 0 0)"), None);

        // Rejecting invalid argument counts: hwb(0 0%) -> None
        assert_eq!(parse("hwb(0 0%)"), None);
    }

    #[test]
    fn test_parse_color_oklab_and_oklch() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // oklab(0 0 0) -> black
        assert_eq!(
            parse("oklab(0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // oklab(1 0 0) -> white
        assert_eq!(
            parse("oklab(1 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // oklab(100% 0 0) -> white
        assert_eq!(
            parse("oklab(100% 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // oklab(0.5 0 0) -> mid gray ~99 +/-2
        let gray_oklab = parse("oklab(0.5 0 0)");
        match gray_oklab {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert!((r as i32 - 99).abs() <= 2);
                assert!((g as i32 - 99).abs() <= 2);
                assert!((b as i32 - 99).abs() <= 2);
                assert_eq!(alpha, 255);
            }
            _ => panic!("Expected oklab(0.5 0 0) to parse as gray"),
        }

        // oklch(0 0 0) -> black
        assert_eq!(
            parse("oklch(0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // oklch(1 0 0) -> white
        assert_eq!(
            parse("oklch(1 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // oklch(0.5 0 0) -> same gray ~99 +/-2
        let gray_oklch = parse("oklch(0.5 0 0)");
        match gray_oklch {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert!((r as i32 - 99).abs() <= 2);
                assert!((g as i32 - 99).abs() <= 2);
                assert!((b as i32 - 99).abs() <= 2);
                assert_eq!(alpha, 255);
            }
            _ => panic!("Expected oklch(0.5 0 0) to parse as gray"),
        }

        // oklab(1 0 0 / 0.5) -> alpha within +/-1 of 128 (round(0.5*255) = 128)
        let alpha_oklab = parse("oklab(1 0 0 / 0.5)");
        match alpha_oklab {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 255);
                assert_eq!(b, 255);
                assert!((alpha as i32 - 128).abs() <= 1);
            }
            _ => panic!("Expected oklab(1 0 0 / 0.5) to parse with alpha"),
        }

        // Chromatic input: oklch(0.628 0.2577 29.23) -> all channels in [0, 255]
        let chromatic_oklch = parse("oklch(0.628 0.2577 29.23)");
        match chromatic_oklch {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(alpha, 255);
                let _ = (r, g, b);
            }
            _ => panic!("Expected oklch(0.628 0.2577 29.23) to parse as a color"),
        }

        // Comma-separated: oklab(1, 0, 0)
        assert_eq!(
            parse("oklab(1, 0, 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // Angle unit for hue in oklch: oklch(1 0 180deg)
        assert_eq!(
            parse("oklch(1 0 180deg)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
    }

    #[test]
    fn test_parse_color_lab_and_lch() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // lab(0 0 0) -> black
        assert_eq!(
            parse("lab(0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // lab(100 0 0) -> white (255,255,255,255)
        assert_eq!(
            parse("lab(100 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // lab(100% 0 0) -> white
        assert_eq!(
            parse("lab(100% 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // lch(0 0 0) -> black
        assert_eq!(
            parse("lch(0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // lch(100 0 0) -> white
        assert_eq!(
            parse("lch(100 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // lab(100 0 0 / 0.5) -> alpha within +/-1 of 128 (round(0.5*255) = 128)
        let alpha_lab = parse("lab(100 0 0 / 0.5)");
        match alpha_lab {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 255);
                assert_eq!(b, 255);
                assert!((alpha as i32 - 128).abs() <= 1);
            }
            _ => panic!("Expected lab(100 0 0 / 0.5) to parse with alpha"),
        }

        // Chromatic input: lch(52.2 72.2 50.0) -> all channels in [0, 255]
        let chromatic_lch = parse("lch(52.2 72.2 50.0)");
        match chromatic_lch {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(alpha, 255);
                let _ = (r, g, b);
            }
            _ => panic!("Expected lch(52.2 72.2 50.0) to parse as a color"),
        }

        // Percentage for chroma and a/b
        assert_eq!(
            parse("lab(100% 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
        assert_eq!(
            parse("lch(100% 0% 0deg)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
    }

    #[test]
    fn test_parse_color_function() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // srgb: color(srgb 1 0 0) -> red
        assert_eq!(
            parse("color(srgb 1 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // srgb: color(srgb 0 0 0) -> black
        assert_eq!(
            parse("color(srgb 0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // srgb: color(srgb 1 1 1) -> white
        assert_eq!(
            parse("color(srgb 1 1 1)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // srgb with alpha: color(srgb 1 0 0 / 0.5)
        let alpha_srgb = parse("color(srgb 1 0 0 / 0.5)");
        match alpha_srgb {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 128).abs() <= 1);
            }
            _ => panic!("Expected color(srgb 1 0 0 / 0.5) to parse with alpha"),
        }

        // srgb percentage: color(srgb 100% 0% 0%) -> red
        assert_eq!(
            parse("color(srgb 100% 0% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // xyz-d65: color(xyz-d65 0 0 0) -> black
        assert_eq!(
            parse("color(xyz-d65 0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // xyz: color(xyz 0 0 0) -> black
        assert_eq!(
            parse("color(xyz 0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // xyz-d50: color(xyz-d50 0 0 0) -> black
        assert_eq!(
            parse("color(xyz-d50 0 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // srgb-linear: color(srgb-linear 1 1 1) -> white
        assert_eq!(
            parse("color(srgb-linear 1 1 1)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // display-p3: color(display-p3 1 1 1) -> white
        assert_eq!(
            parse("color(display-p3 1 1 1)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // rec2020: color(rec2020 1 0 0) -> pure red (clipped/clamped)
        assert_eq!(
            parse("color(rec2020 1 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // a98-rgb: color(a98-rgb 1 1 1) -> white
        assert_eq!(
            parse("color(a98-rgb 1 1 1)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // prophoto-rgb: color(prophoto-rgb 1 1 1) -> white
        assert_eq!(
            parse("color(prophoto-rgb 1 1 1)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // Unknown/Unsupported colorspace: color(unsupported-space 1 0 0) -> None
        assert_eq!(parse("color(unsupported-space 1 0 0)"), None);
    }

    #[test]
    fn test_parse_color_device_cmyk() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // 1. device-cmyk(0 0 0 0) -> white (255,255,255)
        assert_eq!(
            parse("device-cmyk(0 0 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // 2. device-cmyk(0 0 0 1) -> black (0,0,0)
        assert_eq!(
            parse("device-cmyk(0 0 0 1)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 3. device-cmyk(0% 100% 100% 0%) -> red (255,0,0)
        assert_eq!(
            parse("device-cmyk(0% 100% 100% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // 4. device-cmyk(0 1 1 0 / 0.5) -> red with alpha ~0.5 (round(0.5*255) = 128)
        let alpha_cmyk = parse("device-cmyk(0 1 1 0 / 0.5)");
        match alpha_cmyk {
            Some(CssValue::Color(Color::Rgba(r, g, b, alpha))) => {
                assert_eq!(r, 255);
                assert_eq!(g, 0);
                assert_eq!(b, 0);
                assert!((alpha as i32 - 128).abs() <= 1);
            }
            _ => panic!("Expected device-cmyk(0 1 1 0 / 0.5) to parse with alpha"),
        }

        // 5. a malformed case (e.g. wrong arg count) -> None / not recognized
        assert_eq!(parse("device-cmyk(0 1 1)"), None);
        assert_eq!(parse("device-cmyk(0 1 1 0 1 2)"), None);
    }

    #[test]
    fn test_parse_color_mix_function() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // Standard srgb interpolation: color-mix(in srgb, red, blue)
        // Red is rgb(255, 0, 0), Blue is rgb(0, 0, 255). 50/50 mix should yield rgb(128, 0, 128)
        assert_eq!(
            parse("color-mix(in srgb, red, blue)"),
            Some(CssValue::Color(Color::Rgba(128, 0, 128, 255)))
        );

        // Weighted mix: color-mix(in srgb, white 25%, black)
        // White is rgb(255, 255, 255), Black is rgb(0, 0, 0). 25% white / 75% black yields rgb(64, 64, 64)
        assert_eq!(
            parse("color-mix(in srgb, white 25%, black)"),
            Some(CssValue::Color(Color::Rgba(64, 64, 64, 255)))
        );

        // Weighted mix with percentage specified first: color-mix(in srgb, 25% white, black)
        assert_eq!(
            parse("color-mix(in srgb, 25% white, black)"),
            Some(CssValue::Color(Color::Rgba(64, 64, 64, 255)))
        );

        // Non-srgb interpolation colorspace should return None (and log a todo)
        assert_eq!(parse("color-mix(in oklch, red, blue)"), None);

        // Edge case: sum of percentages is 0% -> None
        assert_eq!(parse("color-mix(in srgb, red 0%, blue 0%)"), None);

        // Edge case: percentages sum to S < 100% -> alpha scaling
        // red 20%, blue 30% -> weights are 0.4 and 0.6. alpha scale is 0.5.
        // red (255,0,0,255) and blue (0,0,255,255)
        // pr = 255 * 0.4 = 102, pg = 0, pb = 255 * 0.6 = 153, mixed_a = 1.0.
        // un-premultiply is 102, 0, 153.
        // final_a = 0.5 * 255 = 128 (127.5 rounded up to 128)
        assert_eq!(
            parse("color-mix(in srgb, red 20%, blue 30%)"),
            Some(CssValue::Color(Color::Rgba(102, 0, 153, 128)))
        );
    }

    #[test]
    fn test_parse_color_light_dark() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // a. light-dark(white, black) resolves to the light color (white / rgb 255,255,255)
        assert_eq!(
            parse("light-dark(white, black)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );

        // b. light-dark with a different first color (e.g. light-dark(#ff0000, #0000ff)) resolves to the first (red)
        assert_eq!(
            parse("light-dark(#ff0000, #0000ff)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // c. a malformed call with the wrong number of arguments (e.g. one or three colors) returns None
        assert_eq!(parse("light-dark(white)"), None);
        assert_eq!(parse("light-dark(white, black, blue)"), None);
        assert_eq!(parse("light-dark(white, black, )"), None);
        assert_eq!(parse("light-dark(white,)"), None);
        assert_eq!(parse("light-dark(,black)"), None);
        assert_eq!(parse("light-dark()"), None);

        // Nested cases
        assert_eq!(
            parse("light-dark(light-dark(red, blue), black)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
        assert_eq!(
            parse("light-dark(rgb(0, 255, 0), light-dark(white, black))"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // Case insensitivity
        assert_eq!(
            parse("LIGHT-DARK(White, Black)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
    }

    #[test]
    fn test_parse_cross_fade() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // 1. Basic cross-fade with images
        assert_eq!(
            parse("cross-fade(url(\"a.png\"), url(\"b.png\"))"),
            Some(CssValue::Keyword(
                "cross-fade(url(\"a.png\"), url(\"b.png\"))".to_string()
            ))
        );

        // 2. Cross-fade with percentages
        assert_eq!(
            parse("cross-fade(url(\"a.png\") 25%, url(\"b.png\") 75%)"),
            Some(CssValue::Keyword(
                "cross-fade(url(\"a.png\") 25%, url(\"b.png\") 75%)".to_string()
            ))
        );

        // 3. Nested cross-fade and other image types (gradients)
        assert_eq!(
            parse("cross-fade(linear-gradient(red, blue), cross-fade(url(\"b.png\")))"),
            Some(CssValue::Keyword(
                "cross-fade(linear-gradient(red, blue), cross-fade(url(\"b.png\")))".to_string()
            ))
        );

        // 4. Case insensitivity
        assert_eq!(
            parse("CROSS-FADE(url(\"a.png\"))"),
            Some(CssValue::Keyword("cross-fade(url(\"a.png\"))".to_string()))
        );

        // 5. Malformed/invalid cases
        assert_eq!(parse("cross-fade()"), None);
        assert_eq!(parse("cross-fade(10px)"), None); // not an image or percentage
    }

    #[test]
    fn test_parse_color_contrast() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // 1. Simple contrast matching: red vs black
        assert_eq!(
            parse("color-contrast(red vs black)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 2. Select highest contrast candidate: white vs red, black
        assert_eq!(
            parse("color-contrast(white vs red, black)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 3. Select first candidate meeting target contrast: white vs red, blue to AA
        // AA is 4.5 contrast.
        // Contrast of white vs red is 3.99 (< 4.5)
        // Contrast of white vs blue is 8.59 (>= 4.5)
        assert_eq!(
            parse("color-contrast(white vs red, blue to AA)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );

        // 4. Case insensitivity
        assert_eq!(
            parse("COLOR-CONTRAST(White VS Red, Blue to aa)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );

        // 5. Malformed/invalid cases
        assert_eq!(parse("color-contrast(white vs)"), None);
        assert_eq!(parse("color-contrast(white)"), None);
        assert_eq!(parse("color-contrast(white vs red to)"), None);
    }

    #[test]
    fn test_parse_relative_color_rgb() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // a. parse("rgb(from red 0 0 255)") == Some(CssValue::Color(Color::Rgba(0,0,255,255)))
        assert_eq!(
            parse("rgb(from red 0 0 255)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );

        // b. parse("rgb(from red r g b)") == Some(CssValue::Color(Color::Rgba(255,0,0,255)))
        assert_eq!(
            parse("rgb(from red r g b)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // c. parse("rgb(from red g b r)") == Some(CssValue::Color(Color::Rgba(0,0,255,255)))
        assert_eq!(
            parse("rgb(from red g b r)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 255)))
        );

        // d. parse("rgb(from red 0 255 0 / 0.5)") == Some(CssValue::Color(Color::Rgba(0,255,0,127)))
        assert_eq!(
            parse("rgb(from red 0 255 0 / 0.5)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 127)))
        );

        // e. parse("rgb(from #00ff00 r g b)") == Some(CssValue::Color(Color::Rgba(0,255,0,255)))
        assert_eq!(
            parse("rgb(from #00ff00 r g b)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // f. malformed tests
        assert_eq!(parse("rgb(from red 0 0)"), None);
        assert_eq!(parse("rgb(from 0 0 0)"), None);
        assert_eq!(parse("rgb(from red 0 0 0 0)"), None);

        // Additional relative color cases (none, percentages, alpha keyword)
        assert_eq!(
            parse("rgb(from red none none none)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );
        assert_eq!(
            parse("rgb(from red 100% 50% 0%)"),
            Some(CssValue::Color(Color::Rgba(255, 127, 0, 255)))
        );
        assert_eq!(
            parse("rgb(from red 0 0 255 / 50%)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 255, 127)))
        );
        assert_eq!(
            parse("rgb(from red r g b / alpha)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
    }

    #[test]
    fn test_parse_relative_color_hsl() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // 1. Copy through: hsl(from red h s l) -> resolves to red (255,0,0,255)
        assert_eq!(
            parse("hsl(from red h s l)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // 2. Modify Hue: hsl(from red 120 s l) -> rotates hue to green (0,255,0,255)
        assert_eq!(
            parse("hsl(from red 120 s l)"),
            Some(CssValue::Color(Color::Rgba(0, 255, 0, 255)))
        );

        // 3. Modify alpha: hsl(from red h s l / 0.5) -> sets alpha to 127
        assert_eq!(
            parse("hsl(from red h s l / 0.5)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 127)))
        );

        // 4. Modify alpha as percentage: hsl(from red h s l / 50%) -> sets alpha to 127
        assert_eq!(
            parse("hsl(from red h s l / 50%)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 127)))
        );

        // 5. Using alpha keyword: hsl(from red h s l / alpha) -> sets alpha to 255 (same as red's alpha)
        assert_eq!(
            parse("hsl(from red h s l / alpha)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // 6. Keywords 'none': hsl(from red none none none) -> hue=0, s=0, l=0 -> black (0,0,0,255)
        assert_eq!(
            parse("hsl(from red none none none)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 7. Case insensitivity
        assert_eq!(
            parse("HSL(from Red H S L / ALPHA)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // 8. Arity errors return None
        assert_eq!(parse("hsl(from red h s)"), None);
        assert_eq!(parse("hsl(from red)"), None);
        assert_eq!(parse("hsl(from red h s l /)"), None);
        assert_eq!(parse("hsl(from red h s l / 0.5 0.5)"), None);
    }

    #[test]
    fn test_parse_relative_color_lab_lch_oklab_oklch() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_value(&components)
        };

        // 1. lab relative color
        // lab(from red l a b) should roundtrip to red (represented in lab)
        // lab(from red 100 0 0) should resolve to white (255, 255, 255, 255)
        assert_eq!(
            parse("lab(from red 100 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
        // calc test for lab: lab(from red calc(l - 100) 0 0) should resolve to black (0, 0, 0, 255)
        assert_eq!(
            parse("lab(from red calc(l - 100) 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 2. lch relative color
        // lch(from red 100 0 h) should resolve to white (255, 255, 255, 255)
        assert_eq!(
            parse("lch(from red 100 0 h)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
        // calc test for lch: lch(from red l calc(c * 0) h) should resolve to gray
        // Let's verify with copy-through: lch(from red l c h)
        // Red is Color::Rgba(255, 0, 0, 255)
        assert_eq!(
            parse("lch(from red l c h)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );

        // 3. oklab relative color
        // oklab(from red 1 0 0) should resolve to white (255, 255, 255, 255)
        assert_eq!(
            parse("oklab(from red 1 0 0)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
        // oklab(from red l a b) should resolve to red
        assert_eq!(
            parse("oklab(from red l a b)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
        // calc test for oklab: oklab(from red calc(l * 0) 0 0) should resolve to black
        assert_eq!(
            parse("oklab(from red calc(l * 0) 0 0)"),
            Some(CssValue::Color(Color::Rgba(0, 0, 0, 255)))
        );

        // 4. oklch relative color
        // oklch(from red 1 0 h) should resolve to white (255, 255, 255, 255)
        assert_eq!(
            parse("oklch(from red 1 0 h)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
        );
        // oklch(from red l c h) should resolve to red
        assert_eq!(
            parse("oklch(from red l c h)"),
            Some(CssValue::Color(Color::Rgba(255, 0, 0, 255)))
        );
        // calc test for oklch: oklch(from red l calc(c * 0) h) should resolve to gray (neutral chroma)
        // Red has L ~ 0.627
        // So L=0.627, C=0 -> Oklab(0.627, 0, 0)
        // Let's test if L=1, C=0 resolves to white
        assert_eq!(
            parse("oklch(from red 1 calc(c * 0) h)"),
            Some(CssValue::Color(Color::Rgba(255, 255, 255, 255)))
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

        // Test column-span
        assert!(is_known_layout_property("column-span"));
        assert!(is_known_layout_property("Column-Span"));
        assert!(is_valid_property_value(
            "column-span",
            &CssValue::ColumnSpan(ColumnSpanValue::None)
        ));
        assert!(is_valid_property_value(
            "column-span",
            &CssValue::ColumnSpan(ColumnSpanValue::All)
        ));
        assert!(is_valid_property_value(
            "column-span",
            &CssValue::Keyword("none".to_string())
        ));
        assert!(is_valid_property_value(
            "column-span",
            &CssValue::Keyword("all".to_string())
        ));
        assert!(!is_valid_property_value(
            "column-span",
            &CssValue::Keyword("invalid-column-span".to_string())
        ));
        assert_eq!(
            parse_property_value("column-span", &[token(CssToken::Ident("none".to_string()))]),
            Some(CssValue::ColumnSpan(ColumnSpanValue::None))
        );
        assert_eq!(
            parse_property_value("column-span", &[token(CssToken::Ident("all".to_string()))]),
            Some(CssValue::ColumnSpan(ColumnSpanValue::All))
        );
        assert_eq!(
            parse_property_value(
                "column-span",
                &[token(CssToken::Ident("invalid-column-span".to_string()))]
            ),
            None
        );

        // Test column-fill
        assert!(is_known_layout_property("column-fill"));
        assert!(is_known_layout_property("Column-Fill"));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::ColumnFill(ColumnFillValue::Auto)
        ));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::ColumnFill(ColumnFillValue::Balance)
        ));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::ColumnFill(ColumnFillValue::BalanceAll)
        ));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::Keyword("balance".to_string())
        ));
        assert!(is_valid_property_value(
            "column-fill",
            &CssValue::Keyword("balance-all".to_string())
        ));
        assert!(!is_valid_property_value(
            "column-fill",
            &CssValue::Keyword("invalid-column-fill".to_string())
        ));
        assert_eq!(
            parse_property_value("column-fill", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::ColumnFill(ColumnFillValue::Auto))
        );
        assert_eq!(
            parse_property_value(
                "column-fill",
                &[token(CssToken::Ident("balance".to_string()))]
            ),
            Some(CssValue::ColumnFill(ColumnFillValue::Balance))
        );
        assert_eq!(
            parse_property_value(
                "column-fill",
                &[token(CssToken::Ident("balance-all".to_string()))]
            ),
            Some(CssValue::ColumnFill(ColumnFillValue::BalanceAll))
        );
        assert_eq!(
            parse_property_value(
                "column-fill",
                &[token(CssToken::Ident("invalid-column-fill".to_string()))]
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
    fn test_object_fit_value() {
        // Test parsing keyword strings to ObjectFitValue
        assert_eq!(ObjectFitValue::parse("fill"), Some(ObjectFitValue::Fill));
        assert_eq!(
            ObjectFitValue::parse("contain"),
            Some(ObjectFitValue::Contain)
        );
        assert_eq!(ObjectFitValue::parse("cover"), Some(ObjectFitValue::Cover));
        assert_eq!(ObjectFitValue::parse("none"), Some(ObjectFitValue::None));
        assert_eq!(
            ObjectFitValue::parse("scale-down"),
            Some(ObjectFitValue::ScaleDown)
        );
        assert_eq!(ObjectFitValue::parse("FILL"), Some(ObjectFitValue::Fill));
        assert_eq!(
            ObjectFitValue::parse("Scale-Down"),
            Some(ObjectFitValue::ScaleDown)
        );
        assert_eq!(ObjectFitValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("fill".parse::<ObjectFitValue>(), Ok(ObjectFitValue::Fill));
        assert_eq!(
            "contain".parse::<ObjectFitValue>(),
            Ok(ObjectFitValue::Contain)
        );
        assert_eq!("cover".parse::<ObjectFitValue>(), Ok(ObjectFitValue::Cover));
        assert_eq!("none".parse::<ObjectFitValue>(), Ok(ObjectFitValue::None));
        assert_eq!(
            "scale-down".parse::<ObjectFitValue>(),
            Ok(ObjectFitValue::ScaleDown)
        );
        assert_eq!("BOGUS".parse::<ObjectFitValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(ObjectFitValue::Fill.as_str(), "fill");
        assert_eq!(ObjectFitValue::Contain.as_str(), "contain");
        assert_eq!(ObjectFitValue::Cover.as_str(), "cover");
        assert_eq!(ObjectFitValue::None.as_str(), "none");
        assert_eq!(ObjectFitValue::ScaleDown.as_str(), "scale-down");

        assert_eq!(ObjectFitValue::Fill.as_str(), "fill");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            ObjectFitValue::try_from(&CssValue::Keyword("contain".to_string())),
            Ok(ObjectFitValue::Contain)
        );
        assert_eq!(
            ObjectFitValue::try_from(&CssValue::Keyword("SCALE-DOWN".to_string())),
            Ok(ObjectFitValue::ScaleDown)
        );
        assert_eq!(ObjectFitValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_writing_mode_value() {
        // Test parsing keyword strings to WritingModeValue
        assert_eq!(
            WritingModeValue::parse("horizontal-tb"),
            Some(WritingModeValue::HorizontalTb)
        );
        assert_eq!(
            WritingModeValue::parse("vertical-rl"),
            Some(WritingModeValue::VerticalRl)
        );
        assert_eq!(
            WritingModeValue::parse("vertical-lr"),
            Some(WritingModeValue::VerticalLr)
        );
        assert_eq!(
            WritingModeValue::parse("sideways-rl"),
            Some(WritingModeValue::SidewaysRl)
        );
        assert_eq!(
            WritingModeValue::parse("sideways-lr"),
            Some(WritingModeValue::SidewaysLr)
        );
        assert_eq!(
            WritingModeValue::parse("HORIZONTAL-TB"),
            Some(WritingModeValue::HorizontalTb)
        );
        assert_eq!(
            WritingModeValue::parse("Vertical-Rl"),
            Some(WritingModeValue::VerticalRl)
        );
        assert_eq!(WritingModeValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "horizontal-tb".parse::<WritingModeValue>(),
            Ok(WritingModeValue::HorizontalTb)
        );
        assert_eq!(
            "vertical-rl".parse::<WritingModeValue>(),
            Ok(WritingModeValue::VerticalRl)
        );
        assert_eq!(
            "vertical-lr".parse::<WritingModeValue>(),
            Ok(WritingModeValue::VerticalLr)
        );
        assert_eq!(
            "sideways-rl".parse::<WritingModeValue>(),
            Ok(WritingModeValue::SidewaysRl)
        );
        assert_eq!(
            "sideways-lr".parse::<WritingModeValue>(),
            Ok(WritingModeValue::SidewaysLr)
        );
        assert_eq!("BOGUS".parse::<WritingModeValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(WritingModeValue::HorizontalTb.as_str(), "horizontal-tb");
        assert_eq!(WritingModeValue::VerticalRl.as_str(), "vertical-rl");
        assert_eq!(WritingModeValue::VerticalLr.as_str(), "vertical-lr");
        assert_eq!(WritingModeValue::SidewaysRl.as_str(), "sideways-rl");
        assert_eq!(WritingModeValue::SidewaysLr.as_str(), "sideways-lr");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            WritingModeValue::try_from(&CssValue::Keyword("vertical-rl".to_string())),
            Ok(WritingModeValue::VerticalRl)
        );
        assert_eq!(
            WritingModeValue::try_from(&CssValue::Keyword("HORIZONTAL-TB".to_string())),
            Ok(WritingModeValue::HorizontalTb)
        );
        assert_eq!(WritingModeValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_text_orientation_value() {
        // Test parsing keyword strings to TextOrientationValue
        assert_eq!(
            TextOrientationValue::parse("mixed"),
            Some(TextOrientationValue::Mixed)
        );
        assert_eq!(
            TextOrientationValue::parse("upright"),
            Some(TextOrientationValue::Upright)
        );
        assert_eq!(
            TextOrientationValue::parse("sideways"),
            Some(TextOrientationValue::Sideways)
        );
        assert_eq!(
            TextOrientationValue::parse("MIXED"),
            Some(TextOrientationValue::Mixed)
        );
        assert_eq!(
            TextOrientationValue::parse("Upright"),
            Some(TextOrientationValue::Upright)
        );
        assert_eq!(TextOrientationValue::parse("sideways-right"), None);
        assert_eq!(TextOrientationValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "mixed".parse::<TextOrientationValue>(),
            Ok(TextOrientationValue::Mixed)
        );
        assert_eq!(
            "upright".parse::<TextOrientationValue>(),
            Ok(TextOrientationValue::Upright)
        );
        assert_eq!(
            "sideways".parse::<TextOrientationValue>(),
            Ok(TextOrientationValue::Sideways)
        );
        assert_eq!("sideways-right".parse::<TextOrientationValue>(), Err(()));
        assert_eq!("BOGUS".parse::<TextOrientationValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TextOrientationValue::Mixed.as_str(), "mixed");
        assert_eq!(TextOrientationValue::Upright.as_str(), "upright");
        assert_eq!(TextOrientationValue::Sideways.as_str(), "sideways");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::Keyword("upright".to_string())),
            Ok(TextOrientationValue::Upright)
        );
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::Keyword("MIXED".to_string())),
            Ok(TextOrientationValue::Mixed)
        );
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::TextOrientation(
                TextOrientationValue::Sideways
            )),
            Ok(TextOrientationValue::Sideways)
        );
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::Keyword("sideways-right".to_string())),
            Err(())
        );
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(TextOrientationValue::default(), TextOrientationValue::Mixed);
    }

    #[test]
    fn test_box_decoration_break_value() {
        // Test parsing keyword strings to BoxDecorationBreakValue
        assert_eq!(
            BoxDecorationBreakValue::parse("slice"),
            Some(BoxDecorationBreakValue::Slice)
        );
        assert_eq!(
            BoxDecorationBreakValue::parse("clone"),
            Some(BoxDecorationBreakValue::Clone)
        );
        assert_eq!(
            BoxDecorationBreakValue::parse("SLICE"),
            Some(BoxDecorationBreakValue::Slice)
        );
        assert_eq!(
            BoxDecorationBreakValue::parse("Clone"),
            Some(BoxDecorationBreakValue::Clone)
        );
        assert_eq!(BoxDecorationBreakValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "slice".parse::<BoxDecorationBreakValue>(),
            Ok(BoxDecorationBreakValue::Slice)
        );
        assert_eq!(
            "clone".parse::<BoxDecorationBreakValue>(),
            Ok(BoxDecorationBreakValue::Clone)
        );
        assert_eq!("invalid".parse::<BoxDecorationBreakValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(BoxDecorationBreakValue::Slice.as_str(), "slice");
        assert_eq!(BoxDecorationBreakValue::Clone.as_str(), "clone");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            BoxDecorationBreakValue::try_from(&CssValue::Keyword("clone".to_string())),
            Ok(BoxDecorationBreakValue::Clone)
        );
        assert_eq!(
            BoxDecorationBreakValue::try_from(&CssValue::Keyword("SLICE".to_string())),
            Ok(BoxDecorationBreakValue::Slice)
        );
        assert_eq!(
            BoxDecorationBreakValue::try_from(&CssValue::BoxDecorationBreak(
                BoxDecorationBreakValue::Clone
            )),
            Ok(BoxDecorationBreakValue::Clone)
        );
        assert_eq!(
            BoxDecorationBreakValue::try_from(&CssValue::Keyword("invalid".to_string())),
            Err(())
        );
        assert_eq!(
            BoxDecorationBreakValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(
            BoxDecorationBreakValue::default(),
            BoxDecorationBreakValue::Slice
        );
    }

    #[test]
    fn test_mask_type_value() {
        // Test parsing keyword strings to MaskTypeValue
        assert_eq!(
            MaskTypeValue::parse("luminance"),
            Some(MaskTypeValue::Luminance)
        );
        assert_eq!(MaskTypeValue::parse("alpha"), Some(MaskTypeValue::Alpha));
        assert_eq!(
            MaskTypeValue::parse("LUMINANCE"),
            Some(MaskTypeValue::Luminance)
        );
        assert_eq!(MaskTypeValue::parse("Alpha"), Some(MaskTypeValue::Alpha));
        assert_eq!(MaskTypeValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "luminance".parse::<MaskTypeValue>(),
            Ok(MaskTypeValue::Luminance)
        );
        assert_eq!("alpha".parse::<MaskTypeValue>(), Ok(MaskTypeValue::Alpha));
        assert_eq!("invalid".parse::<MaskTypeValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(MaskTypeValue::Luminance.as_str(), "luminance");
        assert_eq!(MaskTypeValue::Alpha.as_str(), "alpha");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            MaskTypeValue::try_from(&CssValue::Keyword("alpha".to_string())),
            Ok(MaskTypeValue::Alpha)
        );
        assert_eq!(
            MaskTypeValue::try_from(&CssValue::Keyword("LUMINANCE".to_string())),
            Ok(MaskTypeValue::Luminance)
        );
        assert_eq!(
            MaskTypeValue::try_from(&CssValue::MaskType(MaskTypeValue::Alpha)),
            Ok(MaskTypeValue::Alpha)
        );
        assert_eq!(
            MaskTypeValue::try_from(&CssValue::Keyword("invalid".to_string())),
            Err(())
        );
        assert_eq!(MaskTypeValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(MaskTypeValue::default(), MaskTypeValue::Luminance);
    }

    #[test]
    fn test_font_variant_position_value() {
        // Test parsing keyword strings to FontVariantPositionValue
        assert_eq!(
            FontVariantPositionValue::parse("normal"),
            Some(FontVariantPositionValue::Normal)
        );
        assert_eq!(
            FontVariantPositionValue::parse("sub"),
            Some(FontVariantPositionValue::Sub)
        );
        assert_eq!(
            FontVariantPositionValue::parse("super"),
            Some(FontVariantPositionValue::Super)
        );
        assert_eq!(
            FontVariantPositionValue::parse("NORMAL"),
            Some(FontVariantPositionValue::Normal)
        );
        assert_eq!(
            FontVariantPositionValue::parse("Sub"),
            Some(FontVariantPositionValue::Sub)
        );
        assert_eq!(FontVariantPositionValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "normal".parse::<FontVariantPositionValue>(),
            Ok(FontVariantPositionValue::Normal)
        );
        assert_eq!(
            "sub".parse::<FontVariantPositionValue>(),
            Ok(FontVariantPositionValue::Sub)
        );
        assert_eq!(
            "super".parse::<FontVariantPositionValue>(),
            Ok(FontVariantPositionValue::Super)
        );
        assert_eq!("BOGUS".parse::<FontVariantPositionValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(FontVariantPositionValue::Normal.as_str(), "normal");
        assert_eq!(FontVariantPositionValue::Sub.as_str(), "sub");
        assert_eq!(FontVariantPositionValue::Super.as_str(), "super");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            FontVariantPositionValue::try_from(&CssValue::Keyword("sub".to_string())),
            Ok(FontVariantPositionValue::Sub)
        );
        assert_eq!(
            FontVariantPositionValue::try_from(&CssValue::Keyword("NORMAL".to_string())),
            Ok(FontVariantPositionValue::Normal)
        );
        assert_eq!(
            FontVariantPositionValue::try_from(&CssValue::FontVariantPosition(
                FontVariantPositionValue::Super
            )),
            Ok(FontVariantPositionValue::Super)
        );
        assert_eq!(
            FontVariantPositionValue::try_from(&CssValue::Keyword("invalid-kw".to_string())),
            Err(())
        );
        assert_eq!(
            FontVariantPositionValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(
            FontVariantPositionValue::default(),
            FontVariantPositionValue::Normal
        );
    }

    #[test]
    fn test_font_optical_sizing_value() {
        // Test parsing keyword strings to FontOpticalSizingValue
        assert_eq!(
            FontOpticalSizingValue::parse("auto"),
            Some(FontOpticalSizingValue::Auto)
        );
        assert_eq!(
            FontOpticalSizingValue::parse("none"),
            Some(FontOpticalSizingValue::None)
        );
        assert_eq!(
            FontOpticalSizingValue::parse("AUTO"),
            Some(FontOpticalSizingValue::Auto)
        );
        assert_eq!(
            FontOpticalSizingValue::parse("None"),
            Some(FontOpticalSizingValue::None)
        );
        assert_eq!(FontOpticalSizingValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<FontOpticalSizingValue>(),
            Ok(FontOpticalSizingValue::Auto)
        );
        assert_eq!(
            "none".parse::<FontOpticalSizingValue>(),
            Ok(FontOpticalSizingValue::None)
        );
        assert_eq!("BOGUS".parse::<FontOpticalSizingValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(FontOpticalSizingValue::Auto.as_str(), "auto");
        assert_eq!(FontOpticalSizingValue::None.as_str(), "none");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            FontOpticalSizingValue::try_from(&CssValue::Keyword("none".to_string())),
            Ok(FontOpticalSizingValue::None)
        );
        assert_eq!(
            FontOpticalSizingValue::try_from(&CssValue::Keyword("AUTO".to_string())),
            Ok(FontOpticalSizingValue::Auto)
        );
        assert_eq!(
            FontOpticalSizingValue::try_from(&CssValue::FontOpticalSizing(
                FontOpticalSizingValue::None
            )),
            Ok(FontOpticalSizingValue::None)
        );
        assert_eq!(
            FontOpticalSizingValue::try_from(&CssValue::Keyword("invalid-kw".to_string())),
            Err(())
        );
        assert_eq!(
            FontOpticalSizingValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(
            FontOpticalSizingValue::default(),
            FontOpticalSizingValue::Auto
        );
    }

    #[test]
    fn test_caption_side_value() {
        // Test parsing keyword strings to CaptionSideValue
        assert_eq!(CaptionSideValue::parse("top"), Some(CaptionSideValue::Top));
        assert_eq!(
            CaptionSideValue::parse("bottom"),
            Some(CaptionSideValue::Bottom)
        );
        assert_eq!(CaptionSideValue::parse("TOP"), Some(CaptionSideValue::Top));
        assert_eq!(
            CaptionSideValue::parse("Bottom"),
            Some(CaptionSideValue::Bottom)
        );
        assert_eq!(CaptionSideValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("top".parse::<CaptionSideValue>(), Ok(CaptionSideValue::Top));
        assert_eq!(
            "bottom".parse::<CaptionSideValue>(),
            Ok(CaptionSideValue::Bottom)
        );
        assert_eq!("BOGUS".parse::<CaptionSideValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(CaptionSideValue::Top.as_str(), "top");
        assert_eq!(CaptionSideValue::Bottom.as_str(), "bottom");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            CaptionSideValue::try_from(&CssValue::Keyword("top".to_string())),
            Ok(CaptionSideValue::Top)
        );
        assert_eq!(
            CaptionSideValue::try_from(&CssValue::Keyword("BOTTOM".to_string())),
            Ok(CaptionSideValue::Bottom)
        );
        assert_eq!(CaptionSideValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_color_interpolation_value() {
        // Test parsing keyword strings to ColorInterpolationValue
        assert_eq!(
            ColorInterpolationValue::parse("auto"),
            Some(ColorInterpolationValue::Auto)
        );
        assert_eq!(
            ColorInterpolationValue::parse("srgb"),
            Some(ColorInterpolationValue::Srgb)
        );
        assert_eq!(
            ColorInterpolationValue::parse("linearrgb"),
            Some(ColorInterpolationValue::LinearRgb)
        );
        assert_eq!(
            ColorInterpolationValue::parse("AUTO"),
            Some(ColorInterpolationValue::Auto)
        );
        assert_eq!(
            ColorInterpolationValue::parse("sRGB"),
            Some(ColorInterpolationValue::Srgb)
        );
        assert_eq!(
            ColorInterpolationValue::parse("linearRGB"),
            Some(ColorInterpolationValue::LinearRgb)
        );
        assert_eq!(ColorInterpolationValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<ColorInterpolationValue>(),
            Ok(ColorInterpolationValue::Auto)
        );
        assert_eq!(
            "sRGB".parse::<ColorInterpolationValue>(),
            Ok(ColorInterpolationValue::Srgb)
        );
        assert_eq!(
            "linearrgb".parse::<ColorInterpolationValue>(),
            Ok(ColorInterpolationValue::LinearRgb)
        );
        assert_eq!("BOGUS".parse::<ColorInterpolationValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(ColorInterpolationValue::Auto.as_str(), "auto");
        assert_eq!(ColorInterpolationValue::Srgb.as_str(), "sRGB");
        assert_eq!(ColorInterpolationValue::LinearRgb.as_str(), "linearRGB");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            ColorInterpolationValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(ColorInterpolationValue::Auto)
        );
        assert_eq!(
            ColorInterpolationValue::try_from(&CssValue::Keyword("sRGB".to_string())),
            Ok(ColorInterpolationValue::Srgb)
        );
        assert_eq!(
            ColorInterpolationValue::try_from(&CssValue::Keyword("linearrgb".to_string())),
            Ok(ColorInterpolationValue::LinearRgb)
        );
        assert_eq!(
            ColorInterpolationValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(
            ColorInterpolationValue::default(),
            ColorInterpolationValue::Srgb
        );
    }

    #[test]
    fn test_user_select_value() {
        // Test parsing keyword strings to UserSelectValue
        assert_eq!(UserSelectValue::parse("auto"), Some(UserSelectValue::Auto));
        assert_eq!(UserSelectValue::parse("text"), Some(UserSelectValue::Text));
        assert_eq!(UserSelectValue::parse("none"), Some(UserSelectValue::None));
        assert_eq!(
            UserSelectValue::parse("contain"),
            Some(UserSelectValue::Contain)
        );
        assert_eq!(UserSelectValue::parse("all"), Some(UserSelectValue::All));
        assert_eq!(UserSelectValue::parse("AUTO"), Some(UserSelectValue::Auto));
        assert_eq!(UserSelectValue::parse("tExT"), Some(UserSelectValue::Text));
        assert_eq!(UserSelectValue::parse("nOnE"), Some(UserSelectValue::None));
        assert_eq!(
            UserSelectValue::parse("cOnTaIn"),
            Some(UserSelectValue::Contain)
        );
        assert_eq!(UserSelectValue::parse("AlL"), Some(UserSelectValue::All));
        assert_eq!(UserSelectValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("auto".parse::<UserSelectValue>(), Ok(UserSelectValue::Auto));
        assert_eq!("text".parse::<UserSelectValue>(), Ok(UserSelectValue::Text));
        assert_eq!("none".parse::<UserSelectValue>(), Ok(UserSelectValue::None));
        assert_eq!(
            "contain".parse::<UserSelectValue>(),
            Ok(UserSelectValue::Contain)
        );
        assert_eq!("all".parse::<UserSelectValue>(), Ok(UserSelectValue::All));
        assert_eq!("BOGUS".parse::<UserSelectValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(UserSelectValue::Auto.as_str(), "auto");
        assert_eq!(UserSelectValue::Text.as_str(), "text");
        assert_eq!(UserSelectValue::None.as_str(), "none");
        assert_eq!(UserSelectValue::Contain.as_str(), "contain");
        assert_eq!(UserSelectValue::All.as_str(), "all");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            UserSelectValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(UserSelectValue::Auto)
        );
        assert_eq!(
            UserSelectValue::try_from(&CssValue::Keyword("text".to_string())),
            Ok(UserSelectValue::Text)
        );
        assert_eq!(
            UserSelectValue::try_from(&CssValue::Keyword("none".to_string())),
            Ok(UserSelectValue::None)
        );
        assert_eq!(
            UserSelectValue::try_from(&CssValue::Keyword("contain".to_string())),
            Ok(UserSelectValue::Contain)
        );
        assert_eq!(
            UserSelectValue::try_from(&CssValue::Keyword("all".to_string())),
            Ok(UserSelectValue::All)
        );
        assert_eq!(UserSelectValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(UserSelectValue::default(), UserSelectValue::Auto);
    }

    #[test]
    fn test_transform_style_value() {
        // Test parsing keyword strings to TransformStyleValue
        assert_eq!(
            TransformStyleValue::parse("flat"),
            Some(TransformStyleValue::Flat)
        );
        assert_eq!(
            TransformStyleValue::parse("preserve-3d"),
            Some(TransformStyleValue::Preserve3d)
        );
        assert_eq!(
            TransformStyleValue::parse("FLAT"),
            Some(TransformStyleValue::Flat)
        );
        assert_eq!(
            TransformStyleValue::parse("pReSeRvE-3d"),
            Some(TransformStyleValue::Preserve3d)
        );
        assert_eq!(TransformStyleValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "flat".parse::<TransformStyleValue>(),
            Ok(TransformStyleValue::Flat)
        );
        assert_eq!(
            "preserve-3d".parse::<TransformStyleValue>(),
            Ok(TransformStyleValue::Preserve3d)
        );
        assert_eq!("BOGUS".parse::<TransformStyleValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TransformStyleValue::Flat.as_str(), "flat");
        assert_eq!(TransformStyleValue::Preserve3d.as_str(), "preserve-3d");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TransformStyleValue::try_from(&CssValue::Keyword("flat".to_string())),
            Ok(TransformStyleValue::Flat)
        );
        assert_eq!(
            TransformStyleValue::try_from(&CssValue::Keyword("preserve-3d".to_string())),
            Ok(TransformStyleValue::Preserve3d)
        );
        assert_eq!(
            TransformStyleValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation
        assert_eq!(TransformStyleValue::default(), TransformStyleValue::Flat);
    }

    #[test]
    fn test_break_inside_value() {
        // Test parsing keyword strings to BreakInsideValue
        assert_eq!(
            BreakInsideValue::parse("auto"),
            Some(BreakInsideValue::Auto)
        );
        assert_eq!(
            BreakInsideValue::parse("avoid"),
            Some(BreakInsideValue::Avoid)
        );
        assert_eq!(
            BreakInsideValue::parse("avoid-page"),
            Some(BreakInsideValue::AvoidPage)
        );
        assert_eq!(
            BreakInsideValue::parse("avoid-column"),
            Some(BreakInsideValue::AvoidColumn)
        );
        assert_eq!(
            BreakInsideValue::parse("avoid-region"),
            Some(BreakInsideValue::AvoidRegion)
        );

        assert_eq!(
            BreakInsideValue::parse("AUTO"),
            Some(BreakInsideValue::Auto)
        );
        assert_eq!(
            BreakInsideValue::parse("Avoid-Page"),
            Some(BreakInsideValue::AvoidPage)
        );
        assert_eq!(BreakInsideValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<BreakInsideValue>(),
            Ok(BreakInsideValue::Auto)
        );
        assert_eq!(
            "avoid-column".parse::<BreakInsideValue>(),
            Ok(BreakInsideValue::AvoidColumn)
        );
        assert_eq!("BOGUS".parse::<BreakInsideValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(BreakInsideValue::Auto.as_str(), "auto");
        assert_eq!(BreakInsideValue::Avoid.as_str(), "avoid");
        assert_eq!(BreakInsideValue::AvoidPage.as_str(), "avoid-page");
        assert_eq!(BreakInsideValue::AvoidColumn.as_str(), "avoid-column");
        assert_eq!(BreakInsideValue::AvoidRegion.as_str(), "avoid-region");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            BreakInsideValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(BreakInsideValue::Auto)
        );
        assert_eq!(
            BreakInsideValue::try_from(&CssValue::Keyword("AVOID-COLUMN".to_string())),
            Ok(BreakInsideValue::AvoidColumn)
        );
        assert_eq!(BreakInsideValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(BreakInsideValue::default(), BreakInsideValue::Auto);
    }

    #[test]
    fn test_empty_cells_value() {
        // Test parsing keyword strings to EmptyCellsValue
        assert_eq!(EmptyCellsValue::parse("show"), Some(EmptyCellsValue::Show));
        assert_eq!(EmptyCellsValue::parse("hide"), Some(EmptyCellsValue::Hide));
        assert_eq!(EmptyCellsValue::parse("SHOW"), Some(EmptyCellsValue::Show));
        assert_eq!(EmptyCellsValue::parse("Hide"), Some(EmptyCellsValue::Hide));
        assert_eq!(EmptyCellsValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("show".parse::<EmptyCellsValue>(), Ok(EmptyCellsValue::Show));
        assert_eq!("hide".parse::<EmptyCellsValue>(), Ok(EmptyCellsValue::Hide));
        assert_eq!("BOGUS".parse::<EmptyCellsValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(EmptyCellsValue::Show.as_str(), "show");
        assert_eq!(EmptyCellsValue::Hide.as_str(), "hide");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            EmptyCellsValue::try_from(&CssValue::Keyword("show".to_string())),
            Ok(EmptyCellsValue::Show)
        );
        assert_eq!(
            EmptyCellsValue::try_from(&CssValue::Keyword("HIDE".to_string())),
            Ok(EmptyCellsValue::Hide)
        );
        assert_eq!(EmptyCellsValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_border_collapse_value() {
        // Test parsing keyword strings to BorderCollapseValue
        assert_eq!(
            BorderCollapseValue::parse("separate"),
            Some(BorderCollapseValue::Separate)
        );
        assert_eq!(
            BorderCollapseValue::parse("collapse"),
            Some(BorderCollapseValue::Collapse)
        );
        assert_eq!(
            BorderCollapseValue::parse("SEPARATE"),
            Some(BorderCollapseValue::Separate)
        );
        assert_eq!(
            BorderCollapseValue::parse("Collapse"),
            Some(BorderCollapseValue::Collapse)
        );
        assert_eq!(BorderCollapseValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "separate".parse::<BorderCollapseValue>(),
            Ok(BorderCollapseValue::Separate)
        );
        assert_eq!(
            "collapse".parse::<BorderCollapseValue>(),
            Ok(BorderCollapseValue::Collapse)
        );
        assert_eq!("BOGUS".parse::<BorderCollapseValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(BorderCollapseValue::Separate.as_str(), "separate");
        assert_eq!(BorderCollapseValue::Collapse.as_str(), "collapse");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            BorderCollapseValue::try_from(&CssValue::Keyword("separate".to_string())),
            Ok(BorderCollapseValue::Separate)
        );
        assert_eq!(
            BorderCollapseValue::try_from(&CssValue::Keyword("COLLAPSE".to_string())),
            Ok(BorderCollapseValue::Collapse)
        );
        assert_eq!(
            BorderCollapseValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation (initial value)
        assert_eq!(
            BorderCollapseValue::default(),
            BorderCollapseValue::Separate
        );
    }

    #[test]
    fn test_background_attachment_value() {
        // Test parsing keyword strings to BackgroundAttachmentValue
        assert_eq!(
            BackgroundAttachmentValue::parse("scroll"),
            Some(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            BackgroundAttachmentValue::parse("fixed"),
            Some(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            BackgroundAttachmentValue::parse("local"),
            Some(BackgroundAttachmentValue::Local)
        );

        // Case insensitivity
        assert_eq!(
            BackgroundAttachmentValue::parse("SCROLL"),
            Some(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            BackgroundAttachmentValue::parse("Fixed"),
            Some(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            BackgroundAttachmentValue::parse("lOcAl"),
            Some(BackgroundAttachmentValue::Local)
        );

        assert_eq!(BackgroundAttachmentValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "scroll".parse::<BackgroundAttachmentValue>(),
            Ok(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            "fixed".parse::<BackgroundAttachmentValue>(),
            Ok(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            "local".parse::<BackgroundAttachmentValue>(),
            Ok(BackgroundAttachmentValue::Local)
        );
        assert_eq!("BOGUS".parse::<BackgroundAttachmentValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(BackgroundAttachmentValue::Scroll.as_str(), "scroll");
        assert_eq!(BackgroundAttachmentValue::Fixed.as_str(), "fixed");
        assert_eq!(BackgroundAttachmentValue::Local.as_str(), "local");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            BackgroundAttachmentValue::try_from(&CssValue::Keyword("scroll".to_string())),
            Ok(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            BackgroundAttachmentValue::try_from(&CssValue::Keyword("FIXED".to_string())),
            Ok(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            BackgroundAttachmentValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );

        // Test Default implementation (initial value is scroll)
        assert_eq!(
            BackgroundAttachmentValue::default(),
            BackgroundAttachmentValue::Scroll
        );
    }

    #[test]
    fn test_text_wrap_value() {
        // Test parsing keyword strings to TextWrapValue
        assert_eq!(TextWrapValue::parse("wrap"), Some(TextWrapValue::Wrap));
        assert_eq!(TextWrapValue::parse("nowrap"), Some(TextWrapValue::Nowrap));
        assert_eq!(
            TextWrapValue::parse("balance"),
            Some(TextWrapValue::Balance)
        );
        assert_eq!(TextWrapValue::parse("pretty"), Some(TextWrapValue::Pretty));
        assert_eq!(TextWrapValue::parse("stable"), Some(TextWrapValue::Stable));

        // Case insensitivity
        assert_eq!(TextWrapValue::parse("WRAP"), Some(TextWrapValue::Wrap));
        assert_eq!(TextWrapValue::parse("NoWrAp"), Some(TextWrapValue::Nowrap));
        assert_eq!(
            TextWrapValue::parse("Balance"),
            Some(TextWrapValue::Balance)
        );
        assert_eq!(TextWrapValue::parse("PrEtTy"), Some(TextWrapValue::Pretty));
        assert_eq!(TextWrapValue::parse("Stable"), Some(TextWrapValue::Stable));

        assert_eq!(TextWrapValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("wrap".parse::<TextWrapValue>(), Ok(TextWrapValue::Wrap));
        assert_eq!("nowrap".parse::<TextWrapValue>(), Ok(TextWrapValue::Nowrap));
        assert_eq!(
            "balance".parse::<TextWrapValue>(),
            Ok(TextWrapValue::Balance)
        );
        assert_eq!("pretty".parse::<TextWrapValue>(), Ok(TextWrapValue::Pretty));
        assert_eq!("stable".parse::<TextWrapValue>(), Ok(TextWrapValue::Stable));
        assert_eq!("BOGUS".parse::<TextWrapValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TextWrapValue::Wrap.as_str(), "wrap");
        assert_eq!(TextWrapValue::Nowrap.as_str(), "nowrap");
        assert_eq!(TextWrapValue::Balance.as_str(), "balance");
        assert_eq!(TextWrapValue::Pretty.as_str(), "pretty");
        assert_eq!(TextWrapValue::Stable.as_str(), "stable");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TextWrapValue::try_from(&CssValue::Keyword("wrap".to_string())),
            Ok(TextWrapValue::Wrap)
        );
        assert_eq!(
            TextWrapValue::try_from(&CssValue::Keyword("NOWRAP".to_string())),
            Ok(TextWrapValue::Nowrap)
        );
        assert_eq!(TextWrapValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation (initial value is wrap)
        assert_eq!(TextWrapValue::default(), TextWrapValue::Wrap);
    }

    #[test]
    fn test_clear_value() {
        // Test parsing keyword strings to ClearValue
        assert_eq!(ClearValue::parse("none"), Some(ClearValue::None));
        assert_eq!(ClearValue::parse("left"), Some(ClearValue::Left));
        assert_eq!(ClearValue::parse("right"), Some(ClearValue::Right));
        assert_eq!(ClearValue::parse("both"), Some(ClearValue::Both));
        assert_eq!(
            ClearValue::parse("inline-start"),
            Some(ClearValue::InlineStart)
        );
        assert_eq!(ClearValue::parse("inline-end"), Some(ClearValue::InlineEnd));

        // Case insensitivity
        assert_eq!(ClearValue::parse("NONE"), Some(ClearValue::None));
        assert_eq!(ClearValue::parse("Left"), Some(ClearValue::Left));
        assert_eq!(ClearValue::parse("rIgHt"), Some(ClearValue::Right));
        assert_eq!(ClearValue::parse("BoTh"), Some(ClearValue::Both));
        assert_eq!(
            ClearValue::parse("InLiNe-StArT"),
            Some(ClearValue::InlineStart)
        );
        assert_eq!(ClearValue::parse("iNlInE-eNd"), Some(ClearValue::InlineEnd));

        assert_eq!(ClearValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("none".parse::<ClearValue>(), Ok(ClearValue::None));
        assert_eq!("left".parse::<ClearValue>(), Ok(ClearValue::Left));
        assert_eq!("right".parse::<ClearValue>(), Ok(ClearValue::Right));
        assert_eq!("both".parse::<ClearValue>(), Ok(ClearValue::Both));
        assert_eq!(
            "inline-start".parse::<ClearValue>(),
            Ok(ClearValue::InlineStart)
        );
        assert_eq!(
            "inline-end".parse::<ClearValue>(),
            Ok(ClearValue::InlineEnd)
        );
        assert_eq!("BOGUS".parse::<ClearValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(ClearValue::None.as_str(), "none");
        assert_eq!(ClearValue::Left.as_str(), "left");
        assert_eq!(ClearValue::Right.as_str(), "right");
        assert_eq!(ClearValue::Both.as_str(), "both");
        assert_eq!(ClearValue::InlineStart.as_str(), "inline-start");
        assert_eq!(ClearValue::InlineEnd.as_str(), "inline-end");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            ClearValue::try_from(&CssValue::Keyword("none".to_string())),
            Ok(ClearValue::None)
        );
        assert_eq!(
            ClearValue::try_from(&CssValue::Keyword("LEFT".to_string())),
            Ok(ClearValue::Left)
        );
        assert_eq!(ClearValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation (initial value)
        assert_eq!(ClearValue::default(), ClearValue::None);
    }

    #[test]
    fn test_text_align_last_value() {
        // Test parsing keyword strings to TextAlignLastValue
        assert_eq!(
            TextAlignLastValue::parse("auto"),
            Some(TextAlignLastValue::Auto)
        );
        assert_eq!(
            TextAlignLastValue::parse("start"),
            Some(TextAlignLastValue::Start)
        );
        assert_eq!(
            TextAlignLastValue::parse("end"),
            Some(TextAlignLastValue::End)
        );
        assert_eq!(
            TextAlignLastValue::parse("left"),
            Some(TextAlignLastValue::Left)
        );
        assert_eq!(
            TextAlignLastValue::parse("right"),
            Some(TextAlignLastValue::Right)
        );
        assert_eq!(
            TextAlignLastValue::parse("center"),
            Some(TextAlignLastValue::Center)
        );
        assert_eq!(
            TextAlignLastValue::parse("justify"),
            Some(TextAlignLastValue::Justify)
        );

        // Parsing is case-insensitive
        assert_eq!(
            TextAlignLastValue::parse("AUTO"),
            Some(TextAlignLastValue::Auto)
        );
        assert_eq!(
            TextAlignLastValue::parse("Start"),
            Some(TextAlignLastValue::Start)
        );
        assert_eq!(
            TextAlignLastValue::parse("END"),
            Some(TextAlignLastValue::End)
        );
        assert_eq!(
            TextAlignLastValue::parse("Left"),
            Some(TextAlignLastValue::Left)
        );
        assert_eq!(
            TextAlignLastValue::parse("RIGHT"),
            Some(TextAlignLastValue::Right)
        );
        assert_eq!(
            TextAlignLastValue::parse("Center"),
            Some(TextAlignLastValue::Center)
        );
        assert_eq!(
            TextAlignLastValue::parse("JUSTIFY"),
            Some(TextAlignLastValue::Justify)
        );

        // Unknown keyword is rejected
        assert_eq!(TextAlignLastValue::parse("invalid"), None);
        assert_eq!(TextAlignLastValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<TextAlignLastValue>(),
            Ok(TextAlignLastValue::Auto)
        );
        assert_eq!(
            "justify".parse::<TextAlignLastValue>(),
            Ok(TextAlignLastValue::Justify)
        );
        assert_eq!("BOGUS".parse::<TextAlignLastValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TextAlignLastValue::Auto.as_str(), "auto");
        assert_eq!(TextAlignLastValue::Start.as_str(), "start");
        assert_eq!(TextAlignLastValue::End.as_str(), "end");
        assert_eq!(TextAlignLastValue::Left.as_str(), "left");
        assert_eq!(TextAlignLastValue::Right.as_str(), "right");
        assert_eq!(TextAlignLastValue::Center.as_str(), "center");
        assert_eq!(TextAlignLastValue::Justify.as_str(), "justify");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TextAlignLastValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(TextAlignLastValue::Auto)
        );
        assert_eq!(
            TextAlignLastValue::try_from(&CssValue::Keyword("JUSTIFY".to_string())),
            Ok(TextAlignLastValue::Justify)
        );
        assert_eq!(
            TextAlignLastValue::try_from(&CssValue::TextAlignLast(TextAlignLastValue::Start)),
            Ok(TextAlignLastValue::Start)
        );
        assert_eq!(
            TextAlignLastValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );
    }

    #[test]
    fn test_unicode_bidi_value() {
        // Test parsing keyword strings to UnicodeBidiValue
        assert_eq!(
            UnicodeBidiValue::parse("normal"),
            Some(UnicodeBidiValue::Normal)
        );
        assert_eq!(
            UnicodeBidiValue::parse("embed"),
            Some(UnicodeBidiValue::Embed)
        );
        assert_eq!(
            UnicodeBidiValue::parse("isolate"),
            Some(UnicodeBidiValue::Isolate)
        );
        assert_eq!(
            UnicodeBidiValue::parse("bidi-override"),
            Some(UnicodeBidiValue::BidiOverride)
        );
        assert_eq!(
            UnicodeBidiValue::parse("isolate-override"),
            Some(UnicodeBidiValue::IsolateOverride)
        );
        assert_eq!(
            UnicodeBidiValue::parse("plaintext"),
            Some(UnicodeBidiValue::Plaintext)
        );

        // Parsing is case-insensitive
        assert_eq!(
            UnicodeBidiValue::parse("NORMAL"),
            Some(UnicodeBidiValue::Normal)
        );
        assert_eq!(
            UnicodeBidiValue::parse("Embed"),
            Some(UnicodeBidiValue::Embed)
        );
        assert_eq!(
            UnicodeBidiValue::parse("ISOLATE"),
            Some(UnicodeBidiValue::Isolate)
        );
        assert_eq!(
            UnicodeBidiValue::parse("Bidi-Override"),
            Some(UnicodeBidiValue::BidiOverride)
        );
        assert_eq!(
            UnicodeBidiValue::parse("Isolate-Override"),
            Some(UnicodeBidiValue::IsolateOverride)
        );
        assert_eq!(
            UnicodeBidiValue::parse("PLAINTEXT"),
            Some(UnicodeBidiValue::Plaintext)
        );

        // Unknown keyword is rejected
        assert_eq!(UnicodeBidiValue::parse("invalid"), None);
        assert_eq!(UnicodeBidiValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "normal".parse::<UnicodeBidiValue>(),
            Ok(UnicodeBidiValue::Normal)
        );
        assert_eq!(
            "bidi-override".parse::<UnicodeBidiValue>(),
            Ok(UnicodeBidiValue::BidiOverride)
        );
        assert_eq!("BOGUS".parse::<UnicodeBidiValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(UnicodeBidiValue::Normal.as_str(), "normal");
        assert_eq!(UnicodeBidiValue::Embed.as_str(), "embed");
        assert_eq!(UnicodeBidiValue::Isolate.as_str(), "isolate");
        assert_eq!(UnicodeBidiValue::BidiOverride.as_str(), "bidi-override");
        assert_eq!(
            UnicodeBidiValue::IsolateOverride.as_str(),
            "isolate-override"
        );
        assert_eq!(UnicodeBidiValue::Plaintext.as_str(), "plaintext");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            UnicodeBidiValue::try_from(&CssValue::Keyword("normal".to_string())),
            Ok(UnicodeBidiValue::Normal)
        );
        assert_eq!(
            UnicodeBidiValue::try_from(&CssValue::Keyword("BIDI-OVERRIDE".to_string())),
            Ok(UnicodeBidiValue::BidiOverride)
        );
        assert_eq!(
            UnicodeBidiValue::try_from(&CssValue::UnicodeBidi(UnicodeBidiValue::Isolate)),
            Ok(UnicodeBidiValue::Isolate)
        );
        assert_eq!(UnicodeBidiValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_pointer_events_value() {
        // Test parsing keyword strings to PointerEventsValue
        assert_eq!(
            PointerEventsValue::parse("auto"),
            Some(PointerEventsValue::Auto)
        );
        assert_eq!(
            PointerEventsValue::parse("none"),
            Some(PointerEventsValue::None)
        );
        assert_eq!(
            PointerEventsValue::parse("visiblePainted"),
            Some(PointerEventsValue::VisiblePainted)
        );
        assert_eq!(
            PointerEventsValue::parse("visiblepainted"),
            Some(PointerEventsValue::VisiblePainted)
        );
        assert_eq!(
            PointerEventsValue::parse("visiblefill"),
            Some(PointerEventsValue::VisibleFill)
        );
        assert_eq!(
            PointerEventsValue::parse("visiblestroke"),
            Some(PointerEventsValue::VisibleStroke)
        );
        assert_eq!(
            PointerEventsValue::parse("visible"),
            Some(PointerEventsValue::Visible)
        );
        assert_eq!(
            PointerEventsValue::parse("painted"),
            Some(PointerEventsValue::Painted)
        );
        assert_eq!(
            PointerEventsValue::parse("fill"),
            Some(PointerEventsValue::Fill)
        );
        assert_eq!(
            PointerEventsValue::parse("stroke"),
            Some(PointerEventsValue::Stroke)
        );
        assert_eq!(
            PointerEventsValue::parse("all"),
            Some(PointerEventsValue::All)
        );
        assert_eq!(PointerEventsValue::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<PointerEventsValue>(),
            Ok(PointerEventsValue::Auto)
        );
        assert_eq!(
            "none".parse::<PointerEventsValue>(),
            Ok(PointerEventsValue::None)
        );
        assert_eq!(
            "visiblepainted".parse::<PointerEventsValue>(),
            Ok(PointerEventsValue::VisiblePainted)
        );
        assert_eq!("BOGUS".parse::<PointerEventsValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(PointerEventsValue::Auto.as_str(), "auto");
        assert_eq!(PointerEventsValue::None.as_str(), "none");
        assert_eq!(
            PointerEventsValue::VisiblePainted.as_str(),
            "visiblePainted"
        );
        assert_eq!(PointerEventsValue::VisibleFill.as_str(), "visibleFill");
        assert_eq!(PointerEventsValue::VisibleStroke.as_str(), "visibleStroke");
        assert_eq!(PointerEventsValue::Visible.as_str(), "visible");
        assert_eq!(PointerEventsValue::Painted.as_str(), "painted");
        assert_eq!(PointerEventsValue::Fill.as_str(), "fill");
        assert_eq!(PointerEventsValue::Stroke.as_str(), "stroke");
        assert_eq!(PointerEventsValue::All.as_str(), "all");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            PointerEventsValue::try_from(&CssValue::Keyword("visiblePainted".to_string())),
            Ok(PointerEventsValue::VisiblePainted)
        );
        assert_eq!(
            PointerEventsValue::try_from(&CssValue::Keyword("NONE".to_string())),
            Ok(PointerEventsValue::None)
        );
        assert_eq!(
            PointerEventsValue::try_from(&CssValue::Number(1.0)),
            Err(())
        );
    }

    #[test]
    fn test_image_rendering_value() {
        // Test parsing keyword strings to ImageRendering
        assert_eq!(ImageRendering::parse("auto"), Some(ImageRendering::Auto));
        assert_eq!(
            ImageRendering::parse("smooth"),
            Some(ImageRendering::Smooth)
        );
        assert_eq!(
            ImageRendering::parse("high-quality"),
            Some(ImageRendering::HighQuality)
        );
        assert_eq!(
            ImageRendering::parse("crisp-edges"),
            Some(ImageRendering::CrispEdges)
        );
        assert_eq!(
            ImageRendering::parse("pixelated"),
            Some(ImageRendering::Pixelated)
        );
        assert_eq!(ImageRendering::parse("AUTO"), Some(ImageRendering::Auto));
        assert_eq!(
            ImageRendering::parse("Smooth"),
            Some(ImageRendering::Smooth)
        );
        assert_eq!(
            ImageRendering::parse("High-Quality"),
            Some(ImageRendering::HighQuality)
        );
        assert_eq!(
            ImageRendering::parse("Crisp-Edges"),
            Some(ImageRendering::CrispEdges)
        );
        assert_eq!(
            ImageRendering::parse("Pixelated"),
            Some(ImageRendering::Pixelated)
        );
        assert_eq!(ImageRendering::parse("invalid"), None);

        // Test FromStr implementation
        assert_eq!("auto".parse::<ImageRendering>(), Ok(ImageRendering::Auto));
        assert_eq!(
            "smooth".parse::<ImageRendering>(),
            Ok(ImageRendering::Smooth)
        );
        assert_eq!(
            "high-quality".parse::<ImageRendering>(),
            Ok(ImageRendering::HighQuality)
        );
        assert_eq!(
            "crisp-edges".parse::<ImageRendering>(),
            Ok(ImageRendering::CrispEdges)
        );
        assert_eq!(
            "pixelated".parse::<ImageRendering>(),
            Ok(ImageRendering::Pixelated)
        );
        assert_eq!("BOGUS".parse::<ImageRendering>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(ImageRendering::Auto.as_str(), "auto");
        assert_eq!(ImageRendering::Smooth.as_str(), "smooth");
        assert_eq!(ImageRendering::HighQuality.as_str(), "high-quality");
        assert_eq!(ImageRendering::CrispEdges.as_str(), "crisp-edges");
        assert_eq!(ImageRendering::Pixelated.as_str(), "pixelated");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            ImageRendering::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(ImageRendering::Auto)
        );
        assert_eq!(
            ImageRendering::try_from(&CssValue::Keyword("PIXELATED".to_string())),
            Ok(ImageRendering::Pixelated)
        );
        assert_eq!(ImageRendering::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_font_kerning_value() {
        // Test parsing keyword strings to FontKerningValue
        assert_eq!(
            FontKerningValue::parse("auto"),
            Some(FontKerningValue::Auto)
        );
        assert_eq!(
            FontKerningValue::parse("normal"),
            Some(FontKerningValue::Normal)
        );
        assert_eq!(
            FontKerningValue::parse("none"),
            Some(FontKerningValue::None)
        );
        assert_eq!(
            FontKerningValue::parse("AUTO"),
            Some(FontKerningValue::Auto)
        );
        assert_eq!(
            FontKerningValue::parse("Normal"),
            Some(FontKerningValue::Normal)
        );
        assert_eq!(
            FontKerningValue::parse("None"),
            Some(FontKerningValue::None)
        );
        assert_eq!(FontKerningValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<FontKerningValue>(),
            Ok(FontKerningValue::Auto)
        );
        assert_eq!(
            "normal".parse::<FontKerningValue>(),
            Ok(FontKerningValue::Normal)
        );
        assert_eq!(
            "none".parse::<FontKerningValue>(),
            Ok(FontKerningValue::None)
        );
        assert_eq!("BOGUS".parse::<FontKerningValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(FontKerningValue::Auto.as_str(), "auto");
        assert_eq!(FontKerningValue::Normal.as_str(), "normal");
        assert_eq!(FontKerningValue::None.as_str(), "none");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            FontKerningValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(FontKerningValue::Auto)
        );
        assert_eq!(
            FontKerningValue::try_from(&CssValue::Keyword("NORMAL".to_string())),
            Ok(FontKerningValue::Normal)
        );
        assert_eq!(
            FontKerningValue::try_from(&CssValue::Keyword("none".to_string())),
            Ok(FontKerningValue::None)
        );
        assert_eq!(FontKerningValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_text_justify_value() {
        // Test parsing keyword strings to TextJustifyValue
        assert_eq!(
            TextJustifyValue::parse("auto"),
            Some(TextJustifyValue::Auto)
        );
        assert_eq!(
            TextJustifyValue::parse("inter-word"),
            Some(TextJustifyValue::InterWord)
        );
        assert_eq!(
            TextJustifyValue::parse("inter-character"),
            Some(TextJustifyValue::InterCharacter)
        );
        assert_eq!(
            TextJustifyValue::parse("none"),
            Some(TextJustifyValue::None)
        );
        assert_eq!(
            TextJustifyValue::parse("AUTO"),
            Some(TextJustifyValue::Auto)
        );
        assert_eq!(
            TextJustifyValue::parse("Inter-Word"),
            Some(TextJustifyValue::InterWord)
        );
        assert_eq!(
            TextJustifyValue::parse("Inter-Character"),
            Some(TextJustifyValue::InterCharacter)
        );
        assert_eq!(
            TextJustifyValue::parse("NONE"),
            Some(TextJustifyValue::None)
        );
        assert_eq!(TextJustifyValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "auto".parse::<TextJustifyValue>(),
            Ok(TextJustifyValue::Auto)
        );
        assert_eq!(
            "inter-word".parse::<TextJustifyValue>(),
            Ok(TextJustifyValue::InterWord)
        );
        assert_eq!(
            "inter-character".parse::<TextJustifyValue>(),
            Ok(TextJustifyValue::InterCharacter)
        );
        assert_eq!(
            "none".parse::<TextJustifyValue>(),
            Ok(TextJustifyValue::None)
        );
        assert_eq!("BOGUS".parse::<TextJustifyValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TextJustifyValue::Auto.as_str(), "auto");
        assert_eq!(TextJustifyValue::InterWord.as_str(), "inter-word");
        assert_eq!(TextJustifyValue::InterCharacter.as_str(), "inter-character");
        assert_eq!(TextJustifyValue::None.as_str(), "none");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TextJustifyValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(TextJustifyValue::Auto)
        );
        assert_eq!(
            TextJustifyValue::try_from(&CssValue::Keyword("INTER-WORD".to_string())),
            Ok(TextJustifyValue::InterWord)
        );
        assert_eq!(
            TextJustifyValue::try_from(&CssValue::Keyword("inter-character".to_string())),
            Ok(TextJustifyValue::InterCharacter)
        );
        assert_eq!(
            TextJustifyValue::try_from(&CssValue::Keyword("none".to_string())),
            Ok(TextJustifyValue::None)
        );
        assert_eq!(TextJustifyValue::try_from(&CssValue::Number(1.0)), Err(()));
    }

    #[test]
    fn test_word_break_value() {
        // Test parsing keyword strings to WordBreakValue
        assert_eq!(
            WordBreakValue::parse("normal"),
            Some(WordBreakValue::Normal)
        );
        assert_eq!(
            WordBreakValue::parse("break-all"),
            Some(WordBreakValue::BreakAll)
        );
        assert_eq!(
            WordBreakValue::parse("keep-all"),
            Some(WordBreakValue::KeepAll)
        );
        assert_eq!(
            WordBreakValue::parse("break-word"),
            Some(WordBreakValue::BreakWord)
        );
        assert_eq!(
            WordBreakValue::parse("NORMAL"),
            Some(WordBreakValue::Normal)
        );
        assert_eq!(
            WordBreakValue::parse("Break-All"),
            Some(WordBreakValue::BreakAll)
        );
        assert_eq!(
            WordBreakValue::parse("Keep-All"),
            Some(WordBreakValue::KeepAll)
        );
        assert_eq!(
            WordBreakValue::parse("BREAK-WORD"),
            Some(WordBreakValue::BreakWord)
        );
        assert_eq!(WordBreakValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "normal".parse::<WordBreakValue>(),
            Ok(WordBreakValue::Normal)
        );
        assert_eq!(
            "break-all".parse::<WordBreakValue>(),
            Ok(WordBreakValue::BreakAll)
        );
        assert_eq!(
            "keep-all".parse::<WordBreakValue>(),
            Ok(WordBreakValue::KeepAll)
        );
        assert_eq!(
            "break-word".parse::<WordBreakValue>(),
            Ok(WordBreakValue::BreakWord)
        );
        assert_eq!("BOGUS".parse::<WordBreakValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(WordBreakValue::Normal.as_str(), "normal");
        assert_eq!(WordBreakValue::BreakAll.as_str(), "break-all");
        assert_eq!(WordBreakValue::KeepAll.as_str(), "keep-all");
        assert_eq!(WordBreakValue::BreakWord.as_str(), "break-word");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            WordBreakValue::try_from(&CssValue::Keyword("normal".to_string())),
            Ok(WordBreakValue::Normal)
        );
        assert_eq!(
            WordBreakValue::try_from(&CssValue::Keyword("BREAK-ALL".to_string())),
            Ok(WordBreakValue::BreakAll)
        );
        assert_eq!(
            WordBreakValue::try_from(&CssValue::Keyword("keep-all".to_string())),
            Ok(WordBreakValue::KeepAll)
        );
        assert_eq!(
            WordBreakValue::try_from(&CssValue::Keyword("break-word".to_string())),
            Ok(WordBreakValue::BreakWord)
        );
        assert_eq!(WordBreakValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(WordBreakValue::default(), WordBreakValue::Normal);
    }

    #[test]
    fn test_line_break_value() {
        // Test parsing keyword strings to LineBreakValue
        assert_eq!(LineBreakValue::parse("auto"), Some(LineBreakValue::Auto));
        assert_eq!(LineBreakValue::parse("loose"), Some(LineBreakValue::Loose));
        assert_eq!(
            LineBreakValue::parse("normal"),
            Some(LineBreakValue::Normal)
        );
        assert_eq!(
            LineBreakValue::parse("strict"),
            Some(LineBreakValue::Strict)
        );
        assert_eq!(
            LineBreakValue::parse("anywhere"),
            Some(LineBreakValue::Anywhere)
        );
        assert_eq!(LineBreakValue::parse("AUTO"), Some(LineBreakValue::Auto));
        assert_eq!(
            LineBreakValue::parse("Strict"),
            Some(LineBreakValue::Strict)
        );
        assert_eq!(LineBreakValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!("auto".parse::<LineBreakValue>(), Ok(LineBreakValue::Auto));
        assert_eq!("loose".parse::<LineBreakValue>(), Ok(LineBreakValue::Loose));
        assert_eq!(
            "normal".parse::<LineBreakValue>(),
            Ok(LineBreakValue::Normal)
        );
        assert_eq!(
            "strict".parse::<LineBreakValue>(),
            Ok(LineBreakValue::Strict)
        );
        assert_eq!(
            "anywhere".parse::<LineBreakValue>(),
            Ok(LineBreakValue::Anywhere)
        );
        assert_eq!("BOGUS".parse::<LineBreakValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(LineBreakValue::Auto.as_str(), "auto");
        assert_eq!(LineBreakValue::Loose.as_str(), "loose");
        assert_eq!(LineBreakValue::Normal.as_str(), "normal");
        assert_eq!(LineBreakValue::Strict.as_str(), "strict");
        assert_eq!(LineBreakValue::Anywhere.as_str(), "anywhere");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            LineBreakValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(LineBreakValue::Auto)
        );
        assert_eq!(
            LineBreakValue::try_from(&CssValue::Keyword("STRICT".to_string())),
            Ok(LineBreakValue::Strict)
        );
        assert_eq!(
            LineBreakValue::try_from(&CssValue::LineBreak(LineBreakValue::Anywhere)),
            Ok(LineBreakValue::Anywhere)
        );
        assert_eq!(LineBreakValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(LineBreakValue::default(), LineBreakValue::Auto);
    }

    #[test]
    fn test_overflow_wrap_value() {
        // Test parsing keyword strings to OverflowWrapValue
        assert_eq!(
            OverflowWrapValue::parse("normal"),
            Some(OverflowWrapValue::Normal)
        );
        assert_eq!(
            OverflowWrapValue::parse("break-word"),
            Some(OverflowWrapValue::BreakWord)
        );
        assert_eq!(
            OverflowWrapValue::parse("anywhere"),
            Some(OverflowWrapValue::Anywhere)
        );
        assert_eq!(
            OverflowWrapValue::parse("NORMAL"),
            Some(OverflowWrapValue::Normal)
        );
        assert_eq!(
            OverflowWrapValue::parse("Break-Word"),
            Some(OverflowWrapValue::BreakWord)
        );
        assert_eq!(
            OverflowWrapValue::parse("ANYWHERE"),
            Some(OverflowWrapValue::Anywhere)
        );
        assert_eq!(OverflowWrapValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "normal".parse::<OverflowWrapValue>(),
            Ok(OverflowWrapValue::Normal)
        );
        assert_eq!(
            "break-word".parse::<OverflowWrapValue>(),
            Ok(OverflowWrapValue::BreakWord)
        );
        assert_eq!(
            "anywhere".parse::<OverflowWrapValue>(),
            Ok(OverflowWrapValue::Anywhere)
        );
        assert_eq!("BOGUS".parse::<OverflowWrapValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(OverflowWrapValue::Normal.as_str(), "normal");
        assert_eq!(OverflowWrapValue::BreakWord.as_str(), "break-word");
        assert_eq!(OverflowWrapValue::Anywhere.as_str(), "anywhere");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            OverflowWrapValue::try_from(&CssValue::Keyword("normal".to_string())),
            Ok(OverflowWrapValue::Normal)
        );
        assert_eq!(
            OverflowWrapValue::try_from(&CssValue::Keyword("BREAK-WORD".to_string())),
            Ok(OverflowWrapValue::BreakWord)
        );
        assert_eq!(
            OverflowWrapValue::try_from(&CssValue::Keyword("anywhere".to_string())),
            Ok(OverflowWrapValue::Anywhere)
        );
        assert_eq!(OverflowWrapValue::try_from(&CssValue::Number(1.0)), Err(()));

        // Test Default implementation
        assert_eq!(OverflowWrapValue::default(), OverflowWrapValue::Normal);
    }

    #[test]
    fn test_text_decoration_style_value() {
        // Test default value
        assert_eq!(
            TextDecorationStyleValue::default(),
            TextDecorationStyleValue::Solid
        );

        // Test parsing keyword strings to TextDecorationStyleValue
        assert_eq!(
            TextDecorationStyleValue::parse("solid"),
            Some(TextDecorationStyleValue::Solid)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("double"),
            Some(TextDecorationStyleValue::Double)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("dotted"),
            Some(TextDecorationStyleValue::Dotted)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("dashed"),
            Some(TextDecorationStyleValue::Dashed)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("wavy"),
            Some(TextDecorationStyleValue::Wavy)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("SOLID"),
            Some(TextDecorationStyleValue::Solid)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("Double"),
            Some(TextDecorationStyleValue::Double)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("Dotted"),
            Some(TextDecorationStyleValue::Dotted)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("DASHED"),
            Some(TextDecorationStyleValue::Dashed)
        );
        assert_eq!(
            TextDecorationStyleValue::parse("Wavy"),
            Some(TextDecorationStyleValue::Wavy)
        );
        assert_eq!(TextDecorationStyleValue::parse("bogus"), None);

        // Test FromStr implementation
        assert_eq!(
            "solid".parse::<TextDecorationStyleValue>(),
            Ok(TextDecorationStyleValue::Solid)
        );
        assert_eq!(
            "double".parse::<TextDecorationStyleValue>(),
            Ok(TextDecorationStyleValue::Double)
        );
        assert_eq!(
            "dotted".parse::<TextDecorationStyleValue>(),
            Ok(TextDecorationStyleValue::Dotted)
        );
        assert_eq!(
            "dashed".parse::<TextDecorationStyleValue>(),
            Ok(TextDecorationStyleValue::Dashed)
        );
        assert_eq!(
            "wavy".parse::<TextDecorationStyleValue>(),
            Ok(TextDecorationStyleValue::Wavy)
        );
        assert_eq!("BOGUS".parse::<TextDecorationStyleValue>(), Err(()));

        // Test serialization to canonical CSS keywords
        assert_eq!(TextDecorationStyleValue::Solid.as_str(), "solid");
        assert_eq!(TextDecorationStyleValue::Double.as_str(), "double");
        assert_eq!(TextDecorationStyleValue::Dotted.as_str(), "dotted");
        assert_eq!(TextDecorationStyleValue::Dashed.as_str(), "dashed");
        assert_eq!(TextDecorationStyleValue::Wavy.as_str(), "wavy");

        // Test TryFrom<&CssValue> implementation
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Keyword("solid".to_string())),
            Ok(TextDecorationStyleValue::Solid)
        );
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Keyword("DOUBLE".to_string())),
            Ok(TextDecorationStyleValue::Double)
        );
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Keyword("dotted".to_string())),
            Ok(TextDecorationStyleValue::Dotted)
        );
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Keyword("dashed".to_string())),
            Ok(TextDecorationStyleValue::Dashed)
        );
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Keyword("wavy".to_string())),
            Ok(TextDecorationStyleValue::Wavy)
        );
        assert_eq!(
            TextDecorationStyleValue::try_from(&CssValue::Number(1.0)),
            Err(())
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

        // 7. Skew tests
        let val_skew_1 = parse("skew(45deg)").unwrap();
        if let CssValue::Transform(ref fns) = val_skew_1 {
            if let TransformFn::Matrix(m) = fns[0] {
                assert!((m[0] - 1.0).abs() < 1e-5);
                assert!((m[1] - 0.0).abs() < 1e-5);
                assert!((m[2] - 1.0).abs() < 1e-5);
                assert!((m[3] - 1.0).abs() < 1e-5);
                assert!((m[4] - 0.0).abs() < 1e-5);
                assert!((m[5] - 0.0).abs() < 1e-5);
            } else {
                panic!("Expected Matrix");
            }
        } else {
            panic!("Expected Transform");
        }

        let val_skew_2 = parse("skew(45deg, 45deg)").unwrap();
        if let CssValue::Transform(ref fns) = val_skew_2 {
            if let TransformFn::Matrix(m) = fns[0] {
                assert!((m[0] - 1.0).abs() < 1e-5);
                assert!((m[1] - 1.0).abs() < 1e-5);
                assert!((m[2] - 1.0).abs() < 1e-5);
                assert!((m[3] - 1.0).abs() < 1e-5);
                assert!((m[4] - 0.0).abs() < 1e-5);
                assert!((m[5] - 0.0).abs() < 1e-5);
            } else {
                panic!("Expected Matrix");
            }
        } else {
            panic!("Expected Transform");
        }

        let val_skewx = parse("skewX(45deg)").unwrap();
        if let CssValue::Transform(ref fns) = val_skewx {
            if let TransformFn::Matrix(m) = fns[0] {
                assert!((m[0] - 1.0).abs() < 1e-5);
                assert!((m[1] - 0.0).abs() < 1e-5);
                assert!((m[2] - 1.0).abs() < 1e-5);
                assert!((m[3] - 1.0).abs() < 1e-5);
                assert!((m[4] - 0.0).abs() < 1e-5);
                assert!((m[5] - 0.0).abs() < 1e-5);
            } else {
                panic!("Expected Matrix");
            }
        } else {
            panic!("Expected Transform");
        }

        let val_skewy = parse("skewY(45deg)").unwrap();
        if let CssValue::Transform(ref fns) = val_skewy {
            if let TransformFn::Matrix(m) = fns[0] {
                assert!((m[0] - 1.0).abs() < 1e-5);
                assert!((m[1] - 1.0).abs() < 1e-5);
                assert!((m[2] - 0.0).abs() < 1e-5);
                assert!((m[3] - 1.0).abs() < 1e-5);
                assert!((m[4] - 0.0).abs() < 1e-5);
                assert!((m[5] - 0.0).abs() < 1e-5);
            } else {
                panic!("Expected Matrix");
            }
        } else {
            panic!("Expected Transform");
        }

        // 8. Invalid inputs return None
        assert!(parse("skew(10deg, 20deg, 30deg)").is_none());
        assert!(parse("skewX(10deg, 20deg)").is_none());
        assert!(parse("skewY(10deg, 20deg)").is_none());
        assert!(parse("translate(1px, 2px, 3px)").is_none());
        assert!(parse("scale(10px)").is_none());
        assert!(parse("rotate(45)").is_none()); // unitless non-zero angle is invalid
        assert!(parse("translate(10)").is_none()); // unitless non-zero length is invalid
    }

    #[test]
    fn test_parse_matrix_t0508() {
        let parse = |input: &str| {
            let components = crate::css::parser::parse_component_values(input);
            parse_transform(&components)
        };

        // matrix(1, 0, 0, 1, 10, 20)
        let val = parse("matrix(1, 0, 0, 1, 10, 20)").unwrap();
        assert_eq!(
            val,
            CssValue::Transform(vec![TransformFn::Matrix([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])])
        );

        // wrong arg count matrix(1, 2, 3)
        assert!(parse("matrix(1, 2, 3)").is_none());
        assert!(parse("matrix(1, 2, 3, 4, 5, 6, 7)").is_none());
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
            Some(CssValue::ScrollBehavior(ScrollBehaviorValue::Smooth))
        );
        assert_eq!(
            parse_property_value(
                "scroll-behavior",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::ScrollBehavior(ScrollBehaviorValue::Auto))
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
        assert!(is_valid_property_value(
            "scroll-behavior",
            &CssValue::ScrollBehavior(ScrollBehaviorValue::Smooth)
        ));
        assert!(is_valid_property_value(
            "scroll-behavior",
            &CssValue::ScrollBehavior(ScrollBehaviorValue::Auto)
        ));
        assert!(!is_valid_property_value(
            "scroll-behavior",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value for print-color-adjust (t0645)
        assert_eq!(
            parse_property_value(
                "print-color-adjust",
                &[token(CssToken::Ident("exact".to_string()))]
            ),
            Some(CssValue::PrintColorAdjust(PrintColorAdjustValue::Exact))
        );
        assert_eq!(
            parse_property_value(
                "print-color-adjust",
                &[token(CssToken::Ident("economy".to_string()))]
            ),
            Some(CssValue::PrintColorAdjust(PrintColorAdjustValue::Economy))
        );
        assert_eq!(
            parse_property_value(
                "print-color-adjust",
                &[token(CssToken::Ident("ECONOMY".to_string()))]
            ),
            Some(CssValue::PrintColorAdjust(PrintColorAdjustValue::Economy))
        );
        assert_eq!(
            parse_property_value(
                "print-color-adjust",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for print-color-adjust (t0645)
        assert!(is_valid_property_value(
            "print-color-adjust",
            &CssValue::Keyword("exact".to_string())
        ));
        assert!(is_valid_property_value(
            "print-color-adjust",
            &CssValue::Keyword("economy".to_string())
        ));
        assert!(is_valid_property_value(
            "print-color-adjust",
            &CssValue::Keyword("ECONOMY".to_string())
        ));
        assert!(is_valid_property_value(
            "print-color-adjust",
            &CssValue::PrintColorAdjust(PrintColorAdjustValue::Exact)
        ));
        assert!(is_valid_property_value(
            "print-color-adjust",
            &CssValue::PrintColorAdjust(PrintColorAdjustValue::Economy)
        ));
        assert!(!is_valid_property_value(
            "print-color-adjust",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value for forced-color-adjust (t0648)
        assert_eq!(
            parse_property_value(
                "forced-color-adjust",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::ForcedColorAdjust(ForcedColorAdjustValue::Auto))
        );
        assert_eq!(
            parse_property_value(
                "forced-color-adjust",
                &[token(CssToken::Ident("none".to_string()))]
            ),
            Some(CssValue::ForcedColorAdjust(ForcedColorAdjustValue::None))
        );
        assert_eq!(
            parse_property_value(
                "forced-color-adjust",
                &[token(CssToken::Ident("NONE".to_string()))]
            ),
            Some(CssValue::ForcedColorAdjust(ForcedColorAdjustValue::None))
        );
        assert_eq!(
            parse_property_value(
                "forced-color-adjust",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for forced-color-adjust (t0648)
        assert!(is_valid_property_value(
            "forced-color-adjust",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(is_valid_property_value(
            "forced-color-adjust",
            &CssValue::Keyword("none".to_string())
        ));
        assert!(is_valid_property_value(
            "forced-color-adjust",
            &CssValue::Keyword("NONE".to_string())
        ));
        assert!(is_valid_property_value(
            "forced-color-adjust",
            &CssValue::ForcedColorAdjust(ForcedColorAdjustValue::Auto)
        ));
        assert!(is_valid_property_value(
            "forced-color-adjust",
            &CssValue::ForcedColorAdjust(ForcedColorAdjustValue::None)
        ));
        assert!(!is_valid_property_value(
            "forced-color-adjust",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value for color-scheme (t0652)
        assert_eq!(
            parse_property_value(
                "color-scheme",
                &[token(CssToken::Ident("normal".to_string()))]
            ),
            Some(CssValue::ColorScheme(ColorSchemeValue::Normal))
        );
        assert_eq!(
            parse_property_value(
                "color-scheme",
                &[token(CssToken::Ident("light".to_string()))]
            ),
            Some(CssValue::ColorScheme(ColorSchemeValue::Light))
        );
        assert_eq!(
            parse_property_value(
                "color-scheme",
                &[token(CssToken::Ident("dark".to_string()))]
            ),
            Some(CssValue::ColorScheme(ColorSchemeValue::Dark))
        );
        assert_eq!(
            parse_property_value(
                "color-scheme",
                &[token(CssToken::Ident("LIGHT".to_string()))]
            ),
            Some(CssValue::ColorScheme(ColorSchemeValue::Light))
        );
        assert_eq!(
            parse_property_value(
                "color-scheme",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        // Test is_valid_property_value for color-scheme (t0652)
        assert!(is_valid_property_value(
            "color-scheme",
            &CssValue::Keyword("normal".to_string())
        ));
        assert!(is_valid_property_value(
            "color-scheme",
            &CssValue::Keyword("light".to_string())
        ));
        assert!(is_valid_property_value(
            "color-scheme",
            &CssValue::Keyword("DARK".to_string())
        ));
        assert!(is_valid_property_value(
            "color-scheme",
            &CssValue::ColorScheme(ColorSchemeValue::Normal)
        ));
        assert!(is_valid_property_value(
            "color-scheme",
            &CssValue::ColorScheme(ColorSchemeValue::Light)
        ));
        assert!(!is_valid_property_value(
            "color-scheme",
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

        // Test parse_property_value and is_valid_property_value for image-rendering (t0541)
        assert!(is_known_layout_property("image-rendering"));
        assert!(is_known_layout_property("Image-Rendering"));

        for (val, expected_variant) in &[
            ("auto", ImageRenderingValue::Auto),
            ("smooth", ImageRenderingValue::Smooth),
            ("high-quality", ImageRenderingValue::HighQuality),
            ("crisp-edges", ImageRenderingValue::CrispEdges),
            ("pixelated", ImageRenderingValue::Pixelated),
            ("AUTO", ImageRenderingValue::Auto),
            ("Smooth", ImageRenderingValue::Smooth),
        ] {
            assert_eq!(
                parse_property_value(
                    "image-rendering",
                    &[token(CssToken::Ident(val.to_string()))]
                ),
                Some(CssValue::ImageRendering(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "image-rendering",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "auto",
            "smooth",
            "high-quality",
            "crisp-edges",
            "pixelated",
            "AUTO",
            "Smooth",
        ] {
            assert!(is_valid_property_value(
                "image-rendering",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "image-rendering",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for font-variant-caps (t0601)
        assert!(is_known_layout_property("font-variant-caps"));
        assert!(is_known_layout_property("Font-Variant-Caps"));

        for (val, expected_variant) in &[
            ("normal", FontVariantCapsValue::Normal),
            ("small-caps", FontVariantCapsValue::SmallCaps),
            ("all-small-caps", FontVariantCapsValue::AllSmallCaps),
            ("petite-caps", FontVariantCapsValue::PetiteCaps),
            ("all-petite-caps", FontVariantCapsValue::AllPetiteCaps),
            ("unicase", FontVariantCapsValue::Unicase),
            ("titling-caps", FontVariantCapsValue::TitlingCaps),
            ("NORMAL", FontVariantCapsValue::Normal),
            ("Small-Caps", FontVariantCapsValue::SmallCaps),
        ] {
            assert_eq!(
                parse_property_value(
                    "font-variant-caps",
                    &[token(CssToken::Ident(val.to_string()))]
                ),
                Some(CssValue::FontVariantCaps(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "font-variant-caps",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "normal",
            "small-caps",
            "all-small-caps",
            "petite-caps",
            "all-petite-caps",
            "unicase",
            "titling-caps",
            "NORMAL",
            "Small-Caps",
        ] {
            assert!(is_valid_property_value(
                "font-variant-caps",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "font-variant-caps",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for font-kerning (t0543)
        assert!(is_known_layout_property("font-kerning"));
        assert!(is_known_layout_property("Font-Kerning"));

        for val in &["auto", "normal", "none", "AUTO", "Normal"] {
            assert_eq!(
                parse_property_value("font-kerning", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "font-kerning",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["auto", "normal", "none", "AUTO", "Normal"] {
            assert!(is_valid_property_value(
                "font-kerning",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "font-kerning",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for text-justify (t0547)
        assert!(is_known_layout_property("text-justify"));
        assert!(is_known_layout_property("Text-Justify"));

        for val in &[
            "auto",
            "inter-word",
            "inter-character",
            "none",
            "AUTO",
            "Inter-Word",
        ] {
            assert_eq!(
                parse_property_value("text-justify", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "text-justify",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "auto",
            "inter-word",
            "inter-character",
            "none",
            "AUTO",
            "Inter-Word",
        ] {
            assert!(is_valid_property_value(
                "text-justify",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "text-justify",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for caption-side (t0549)
        assert!(is_known_layout_property("caption-side"));
        assert!(is_known_layout_property("Caption-Side"));

        for val in &["top", "bottom", "TOP", "Bottom"] {
            assert_eq!(
                parse_property_value("caption-side", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "caption-side",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["top", "bottom", "TOP", "Bottom"] {
            assert!(is_valid_property_value(
                "caption-side",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "caption-side",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for break-inside (t0658)
        assert!(is_known_layout_property("break-inside"));
        assert!(is_known_layout_property("Break-Inside"));

        for val in &[
            "auto",
            "avoid",
            "avoid-page",
            "avoid-column",
            "avoid-region",
            "AVOID-COLUMN",
        ] {
            assert_eq!(
                parse_property_value("break-inside", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "break-inside",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "auto",
            "avoid",
            "avoid-page",
            "avoid-column",
            "avoid-region",
            "AVOID-COLUMN",
        ] {
            assert!(is_valid_property_value(
                "break-inside",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "break-inside",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for pointer-events (t0553)
        assert!(is_known_layout_property("pointer-events"));
        assert!(is_known_layout_property("Pointer-Events"));

        for val in &[
            "auto",
            "none",
            "visiblePainted",
            "visibleFill",
            "visibleStroke",
            "visible",
            "painted",
            "fill",
            "stroke",
            "all",
            "visiblepainted",
            "AUTO",
            "None",
        ] {
            assert_eq!(
                parse_property_value("pointer-events", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "pointer-events",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "auto",
            "none",
            "visiblePainted",
            "visibleFill",
            "visibleStroke",
            "visible",
            "painted",
            "fill",
            "stroke",
            "all",
            "visiblepainted",
            "AUTO",
            "None",
        ] {
            assert!(is_valid_property_value(
                "pointer-events",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "pointer-events",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for object-fit (t0545)
        assert!(is_known_layout_property("object-fit"));
        assert!(is_known_layout_property("Object-Fit"));

        for val in &[
            "fill",
            "contain",
            "cover",
            "none",
            "scale-down",
            "FILL",
            "Scale-Down",
        ] {
            assert_eq!(
                parse_property_value("object-fit", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "object-fit",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "fill",
            "contain",
            "cover",
            "none",
            "scale-down",
            "FILL",
            "Scale-Down",
        ] {
            assert!(is_valid_property_value(
                "object-fit",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "object-fit",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for word-break
        assert!(is_known_layout_property("word-break"));
        assert!(is_known_layout_property("Word-Break"));

        for val in &[
            "normal",
            "break-all",
            "keep-all",
            "break-word",
            "NORMAL",
            "Break-All",
        ] {
            assert_eq!(
                parse_property_value("word-break", &[token(CssToken::Ident(val.to_string()))]),
                Some(CssValue::Keyword(val.to_string()))
            );
        }
        assert_eq!(
            parse_property_value(
                "word-break",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "normal",
            "break-all",
            "keep-all",
            "break-word",
            "NORMAL",
            "Break-All",
        ] {
            assert!(is_valid_property_value(
                "word-break",
                &CssValue::Keyword(val.to_string())
            ));
        }
        assert!(!is_valid_property_value(
            "word-break",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for overflow-wrap and word-wrap
        assert!(is_known_layout_property("overflow-wrap"));
        assert!(is_known_layout_property("word-wrap"));
        assert!(is_known_layout_property("Overflow-Wrap"));
        assert!(is_known_layout_property("Word-Wrap"));

        for prop in &["overflow-wrap", "word-wrap"] {
            for val in &["normal", "break-word", "anywhere", "NORMAL", "Break-Word"] {
                assert_eq!(
                    parse_property_value(prop, &[token(CssToken::Ident(val.to_string()))]),
                    Some(CssValue::Keyword(val.to_string()))
                );
            }
            assert_eq!(
                parse_property_value(prop, &[token(CssToken::Ident("invalid-value".to_string()))]),
                None
            );

            for val in &["normal", "break-word", "anywhere", "NORMAL", "Break-Word"] {
                assert!(is_valid_property_value(
                    prop,
                    &CssValue::Keyword(val.to_string())
                ));
            }
            assert!(!is_valid_property_value(
                prop,
                &CssValue::Keyword("invalid-value".to_string())
            ));
        }

        // Test parse_property_value and is_valid_property_value for line-break
        assert!(is_known_layout_property("line-break"));
        assert!(is_known_layout_property("Line-Break"));

        for (val, expected_variant) in &[
            ("auto", LineBreakValue::Auto),
            ("loose", LineBreakValue::Loose),
            ("normal", LineBreakValue::Normal),
            ("strict", LineBreakValue::Strict),
            ("anywhere", LineBreakValue::Anywhere),
            ("AUTO", LineBreakValue::Auto),
            ("Strict", LineBreakValue::Strict),
        ] {
            assert_eq!(
                parse_property_value("line-break", &[token(CssToken::Ident((*val).to_string()))]),
                Some(CssValue::LineBreak(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "line-break",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &[
            "auto", "loose", "normal", "strict", "anywhere", "AUTO", "Strict",
        ] {
            assert!(is_valid_property_value(
                "line-break",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "line-break",
                &CssValue::LineBreak(LineBreakValue::Auto)
            ));
        }
        assert!(!is_valid_property_value(
            "line-break",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for text-orientation
        assert!(is_known_layout_property("text-orientation"));
        assert!(is_known_layout_property("Text-Orientation"));

        for (val, expected_variant) in &[
            ("mixed", TextOrientationValue::Mixed),
            ("upright", TextOrientationValue::Upright),
            ("sideways", TextOrientationValue::Sideways),
            ("MIXED", TextOrientationValue::Mixed),
            ("Upright", TextOrientationValue::Upright),
        ] {
            assert_eq!(
                parse_property_value(
                    "text-orientation",
                    &[token(CssToken::Ident((*val).to_string()))]
                ),
                Some(CssValue::TextOrientation(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "text-orientation",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["mixed", "upright", "sideways", "MIXED", "Upright"] {
            assert!(is_valid_property_value(
                "text-orientation",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "text-orientation",
                &CssValue::TextOrientation(TextOrientationValue::Mixed)
            ));
        }
        assert!(!is_valid_property_value(
            "text-orientation",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for box-decoration-break
        assert!(is_known_layout_property("box-decoration-break"));
        assert!(is_known_layout_property("Box-Decoration-Break"));

        for (val, expected_variant) in &[
            ("slice", BoxDecorationBreakValue::Slice),
            ("clone", BoxDecorationBreakValue::Clone),
            ("SLICE", BoxDecorationBreakValue::Slice),
            ("Clone", BoxDecorationBreakValue::Clone),
        ] {
            assert_eq!(
                parse_property_value(
                    "box-decoration-break",
                    &[token(CssToken::Ident((*val).to_string()))]
                ),
                Some(CssValue::BoxDecorationBreak(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "box-decoration-break",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["slice", "clone", "SLICE", "Clone"] {
            assert!(is_valid_property_value(
                "box-decoration-break",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "box-decoration-break",
                &CssValue::BoxDecorationBreak(BoxDecorationBreakValue::Slice)
            ));
        }
        assert!(!is_valid_property_value(
            "box-decoration-break",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for mask-type
        assert!(is_known_layout_property("mask-type"));
        assert!(is_known_layout_property("Mask-Type"));

        for (val, expected_variant) in &[
            ("luminance", MaskTypeValue::Luminance),
            ("alpha", MaskTypeValue::Alpha),
            ("LUMINANCE", MaskTypeValue::Luminance),
            ("Alpha", MaskTypeValue::Alpha),
        ] {
            assert_eq!(
                parse_property_value("mask-type", &[token(CssToken::Ident((*val).to_string()))]),
                Some(CssValue::MaskType(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "mask-type",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["luminance", "alpha", "LUMINANCE", "Alpha"] {
            assert!(is_valid_property_value(
                "mask-type",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "mask-type",
                &CssValue::MaskType(MaskTypeValue::Luminance)
            ));
        }
        assert!(!is_valid_property_value(
            "mask-type",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for font-variant-position
        assert!(is_known_layout_property("font-variant-position"));
        assert!(is_known_layout_property("Font-Variant-Position"));

        for (val, expected_variant) in &[
            ("normal", FontVariantPositionValue::Normal),
            ("sub", FontVariantPositionValue::Sub),
            ("super", FontVariantPositionValue::Super),
            ("NORMAL", FontVariantPositionValue::Normal),
            ("Sub", FontVariantPositionValue::Sub),
        ] {
            assert_eq!(
                parse_property_value(
                    "font-variant-position",
                    &[token(CssToken::Ident((*val).to_string()))]
                ),
                Some(CssValue::FontVariantPosition(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "font-variant-position",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["normal", "sub", "super", "NORMAL", "Sub"] {
            assert!(is_valid_property_value(
                "font-variant-position",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "font-variant-position",
                &CssValue::FontVariantPosition(FontVariantPositionValue::Normal)
            ));
        }
        assert!(!is_valid_property_value(
            "font-variant-position",
            &CssValue::Keyword("invalid-value".to_string())
        ));

        // Test parse_property_value and is_valid_property_value for font-optical-sizing
        assert!(is_known_layout_property("font-optical-sizing"));
        assert!(is_known_layout_property("Font-Optical-Sizing"));

        for (val, expected_variant) in &[
            ("auto", FontOpticalSizingValue::Auto),
            ("none", FontOpticalSizingValue::None),
            ("AUTO", FontOpticalSizingValue::Auto),
            ("None", FontOpticalSizingValue::None),
        ] {
            assert_eq!(
                parse_property_value(
                    "font-optical-sizing",
                    &[token(CssToken::Ident((*val).to_string()))]
                ),
                Some(CssValue::FontOpticalSizing(*expected_variant))
            );
        }
        assert_eq!(
            parse_property_value(
                "font-optical-sizing",
                &[token(CssToken::Ident("invalid-value".to_string()))]
            ),
            None
        );

        for val in &["auto", "none", "AUTO", "None"] {
            assert!(is_valid_property_value(
                "font-optical-sizing",
                &CssValue::Keyword((*val).to_string())
            ));
            assert!(is_valid_property_value(
                "font-optical-sizing",
                &CssValue::FontOpticalSizing(FontOpticalSizingValue::Auto)
            ));
        }
        assert!(!is_valid_property_value(
            "font-optical-sizing",
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

    #[test]
    fn test_grid_parsing_and_recognition() {
        // 1. display: grid
        assert_eq!(
            parse_property_value("display", &[token(CssToken::Ident("grid".to_string()))]),
            Some(CssValue::Display(DisplayValue::Grid))
        );
        // display: inline-grid
        assert_eq!(
            parse_property_value(
                "display",
                &[token(CssToken::Ident("inline-grid".to_string()))]
            ),
            Some(CssValue::Display(DisplayValue::InlineGrid))
        );

        // 2. grid-template-columns: 100px 1fr auto 25%
        let components_cols = [
            token(CssToken::Dimension {
                value: 100.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 1.0,
                unit: "fr".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Ident("auto".to_string())),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(25.0)),
        ];
        assert_eq!(
            parse_property_value("grid-template-columns", &components_cols),
            Some(CssValue::GridTemplate(vec![
                GridTrackSize::Px(100.0),
                GridTrackSize::Fr(1.0),
                GridTrackSize::Auto,
                GridTrackSize::Percent(25.0),
            ]))
        );

        // 3. grid-template-rows: 1fr 1fr
        let components_rows = [
            token(CssToken::Dimension {
                value: 1.0,
                unit: "fr".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Dimension {
                value: 1.0,
                unit: "fr".to_string(),
            }),
        ];
        assert_eq!(
            parse_property_value("grid-template-rows", &components_rows),
            Some(CssValue::GridTemplate(vec![
                GridTrackSize::Fr(1.0),
                GridTrackSize::Fr(1.0),
            ]))
        );

        // 4. A single auto track parses to [Auto]
        assert_eq!(
            parse_property_value(
                "grid-template-columns",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::GridTemplate(vec![GridTrackSize::Auto]))
        );
    }

    #[test]
    fn test_scroll_snap_parsing_and_recognition() {
        // Test scroll-snap-type: x mandatory
        assert_eq!(
            parse_property_value(
                "scroll-snap-type",
                &[
                    token(CssToken::Ident("x".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("mandatory".to_string())),
                ]
            ),
            Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::Axis(
                ScrollSnapAxis::X,
                ScrollSnapStrictness::Mandatory,
            )))
        );

        // Test scroll-snap-type: y
        assert_eq!(
            parse_property_value(
                "scroll-snap-type",
                &[token(CssToken::Ident("y".to_string()))]
            ),
            Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::Axis(
                ScrollSnapAxis::Y,
                ScrollSnapStrictness::Proximity,
            )))
        );

        // Test scroll-snap-type: none
        assert_eq!(
            parse_property_value(
                "scroll-snap-type",
                &[token(CssToken::Ident("none".to_string()))]
            ),
            Some(CssValue::ScrollSnapType(ScrollSnapTypeValue::None))
        );

        // Test scroll-snap-align: start
        assert_eq!(
            parse_property_value(
                "scroll-snap-align",
                &[token(CssToken::Ident("start".to_string()))]
            ),
            Some(CssValue::ScrollSnapAlign(ScrollSnapAlignValue {
                block: ScrollSnapAlignKeyword::Start,
                inline: ScrollSnapAlignKeyword::Start,
            }))
        );

        // Test scroll-snap-align: start end
        assert_eq!(
            parse_property_value(
                "scroll-snap-align",
                &[
                    token(CssToken::Ident("start".to_string())),
                    token(CssToken::Whitespace),
                    token(CssToken::Ident("end".to_string())),
                ]
            ),
            Some(CssValue::ScrollSnapAlign(ScrollSnapAlignValue {
                block: ScrollSnapAlignKeyword::Start,
                inline: ScrollSnapAlignKeyword::End,
            }))
        );

        // Test invalid value: scroll-snap-type: banana
        assert_eq!(
            parse_property_value(
                "scroll-snap-type",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("scroll-snap-type"));
        assert!(is_known_layout_property("scroll-snap-align"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "scroll-snap-type",
            &CssValue::ScrollSnapType(ScrollSnapTypeValue::Axis(
                ScrollSnapAxis::X,
                ScrollSnapStrictness::Mandatory
            ))
        ));
        assert!(is_valid_property_value(
            "scroll-snap-align",
            &CssValue::ScrollSnapAlign(ScrollSnapAlignValue {
                block: ScrollSnapAlignKeyword::Start,
                inline: ScrollSnapAlignKeyword::End,
            })
        ));
    }

    #[test]
    fn test_mix_blend_mode_parsing_and_recognition() {
        // Test mix-blend-mode: multiply
        assert_eq!(
            parse_property_value(
                "mix-blend-mode",
                &[token(CssToken::Ident("multiply".to_string()))]
            ),
            Some(CssValue::MixBlendMode(MixBlendModeValue::Multiply))
        );

        // Test normal default
        assert_eq!(
            parse_property_value(
                "mix-blend-mode",
                &[token(CssToken::Ident("normal".to_string()))]
            ),
            Some(CssValue::MixBlendMode(MixBlendModeValue::Normal))
        );

        // Test color-dodge (hyphenated)
        assert_eq!(
            parse_property_value(
                "mix-blend-mode",
                &[token(CssToken::Ident("color-dodge".to_string()))]
            ),
            Some(CssValue::MixBlendMode(MixBlendModeValue::ColorDodge))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "mix-blend-mode",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("mix-blend-mode"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "mix-blend-mode",
            &CssValue::MixBlendMode(MixBlendModeValue::Multiply)
        ));
        assert!(is_valid_property_value(
            "mix-blend-mode",
            &CssValue::Keyword("multiply".to_string())
        ));
        assert!(!is_valid_property_value(
            "mix-blend-mode",
            &CssValue::Keyword("banana".to_string())
        ));
    }

    #[test]
    fn test_background_blend_mode_parsing_and_recognition() {
        // Test background-blend-mode: multiply
        assert_eq!(
            parse_property_value(
                "background-blend-mode",
                &[token(CssToken::Ident("multiply".to_string()))]
            ),
            Some(CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::Multiply
            ))
        );

        // Test normal default
        assert_eq!(
            parse_property_value(
                "background-blend-mode",
                &[token(CssToken::Ident("normal".to_string()))]
            ),
            Some(CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::Normal
            ))
        );

        // Test color-dodge (hyphenated)
        assert_eq!(
            parse_property_value(
                "background-blend-mode",
                &[token(CssToken::Ident("color-dodge".to_string()))]
            ),
            Some(CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::ColorDodge
            ))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "background-blend-mode",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("background-blend-mode"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "background-blend-mode",
            &CssValue::BackgroundBlendMode(BackgroundBlendModeValue::Multiply)
        ));
        assert!(is_valid_property_value(
            "background-blend-mode",
            &CssValue::Keyword("multiply".to_string())
        ));
        assert!(!is_valid_property_value(
            "background-blend-mode",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test BackgroundBlendModeValue::parse directly
        assert_eq!(
            BackgroundBlendModeValue::parse("multiply"),
            Some(BackgroundBlendModeValue::Multiply)
        );
        assert_eq!(
            BackgroundBlendModeValue::parse("normal"),
            Some(BackgroundBlendModeValue::Normal)
        );
        assert_eq!(BackgroundBlendModeValue::parse("banana"), None);
        assert_eq!(
            BackgroundBlendModeValue::parse("MULTIPLY"),
            Some(BackgroundBlendModeValue::Multiply)
        );
    }

    #[test]
    fn test_isolation_parsing_and_recognition() {
        // Test isolation: isolate
        assert_eq!(
            parse_property_value(
                "isolation",
                &[token(CssToken::Ident("isolate".to_string()))]
            ),
            Some(CssValue::Isolation(IsolationValue::Isolate))
        );

        // Test auto default
        assert_eq!(
            parse_property_value("isolation", &[token(CssToken::Ident("auto".to_string()))]),
            Some(CssValue::Isolation(IsolationValue::Auto))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value("isolation", &[token(CssToken::Ident("banana".to_string()))]),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("isolation"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "isolation",
            &CssValue::Isolation(IsolationValue::Isolate)
        ));
        assert!(is_valid_property_value(
            "isolation",
            &CssValue::Keyword("isolate".to_string())
        ));
        assert!(!is_valid_property_value(
            "isolation",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test IsolationValue::parse directly
        assert_eq!(
            IsolationValue::parse("isolate"),
            Some(IsolationValue::Isolate)
        );
        assert_eq!(IsolationValue::parse("auto"), Some(IsolationValue::Auto));
        assert_eq!(IsolationValue::parse("banana"), None);
        assert_eq!(
            IsolationValue::parse("ISOLATE"),
            Some(IsolationValue::Isolate)
        );
    }

    #[test]
    fn test_resize_parsing_and_recognition() {
        // Test resize: both
        assert_eq!(
            parse_property_value("resize", &[token(CssToken::Ident("both".to_string()))]),
            Some(CssValue::Resize(ResizeValue::Both))
        );

        // Test none default
        assert_eq!(
            parse_property_value("resize", &[token(CssToken::Ident("none".to_string()))]),
            Some(CssValue::Resize(ResizeValue::None))
        );

        // Test horizontal
        assert_eq!(
            parse_property_value(
                "resize",
                &[token(CssToken::Ident("horizontal".to_string()))]
            ),
            Some(CssValue::Resize(ResizeValue::Horizontal))
        );

        // Test vertical
        assert_eq!(
            parse_property_value("resize", &[token(CssToken::Ident("vertical".to_string()))]),
            Some(CssValue::Resize(ResizeValue::Vertical))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value("resize", &[token(CssToken::Ident("banana".to_string()))]),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("resize"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "resize",
            &CssValue::Resize(ResizeValue::Both)
        ));
        assert!(is_valid_property_value(
            "resize",
            &CssValue::Keyword("both".to_string())
        ));
        assert!(!is_valid_property_value(
            "resize",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test ResizeValue::parse directly
        assert_eq!(ResizeValue::parse("both"), Some(ResizeValue::Both));
        assert_eq!(ResizeValue::parse("none"), Some(ResizeValue::None));
        assert_eq!(
            ResizeValue::parse("horizontal"),
            Some(ResizeValue::Horizontal)
        );
        assert_eq!(ResizeValue::parse("vertical"), Some(ResizeValue::Vertical));
        assert_eq!(ResizeValue::parse("banana"), None);
        assert_eq!(ResizeValue::parse("BOTH"), Some(ResizeValue::Both));
    }

    #[test]
    fn test_backface_visibility_parsing_and_recognition() {
        // Test backface-visibility: visible
        assert_eq!(
            parse_property_value(
                "backface-visibility",
                &[token(CssToken::Ident("visible".to_string()))]
            ),
            Some(CssValue::BackfaceVisibility(
                BackfaceVisibilityValue::Visible
            ))
        );

        // Test backface-visibility: hidden
        assert_eq!(
            parse_property_value(
                "backface-visibility",
                &[token(CssToken::Ident("hidden".to_string()))]
            ),
            Some(CssValue::BackfaceVisibility(
                BackfaceVisibilityValue::Hidden
            ))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "backface-visibility",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("backface-visibility"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "backface-visibility",
            &CssValue::BackfaceVisibility(BackfaceVisibilityValue::Visible)
        ));
        assert!(is_valid_property_value(
            "backface-visibility",
            &CssValue::Keyword("visible".to_string())
        ));
        assert!(!is_valid_property_value(
            "backface-visibility",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test BackfaceVisibilityValue::parse directly
        assert_eq!(
            BackfaceVisibilityValue::parse("visible"),
            Some(BackfaceVisibilityValue::Visible)
        );
        assert_eq!(
            BackfaceVisibilityValue::parse("hidden"),
            Some(BackfaceVisibilityValue::Hidden)
        );
        assert_eq!(BackfaceVisibilityValue::parse("banana"), None);
        assert_eq!(
            BackfaceVisibilityValue::parse("VISIBLE"),
            Some(BackfaceVisibilityValue::Visible)
        );
    }

    #[test]
    fn test_empty_cells_parsing_and_recognition() {
        // Test empty-cells: show
        assert_eq!(
            parse_property_value("empty-cells", &[token(CssToken::Ident("show".to_string()))]),
            Some(CssValue::EmptyCells(EmptyCellsValue::Show))
        );

        // Test empty-cells: hide
        assert_eq!(
            parse_property_value("empty-cells", &[token(CssToken::Ident("hide".to_string()))]),
            Some(CssValue::EmptyCells(EmptyCellsValue::Hide))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "empty-cells",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("empty-cells"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "empty-cells",
            &CssValue::EmptyCells(EmptyCellsValue::Show)
        ));
        assert!(is_valid_property_value(
            "empty-cells",
            &CssValue::Keyword("show".to_string())
        ));
        assert!(!is_valid_property_value(
            "empty-cells",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test EmptyCellsValue::parse directly
        assert_eq!(EmptyCellsValue::parse("show"), Some(EmptyCellsValue::Show));
        assert_eq!(EmptyCellsValue::parse("hide"), Some(EmptyCellsValue::Hide));
        assert_eq!(EmptyCellsValue::parse("banana"), None);
        assert_eq!(EmptyCellsValue::parse("SHOW"), Some(EmptyCellsValue::Show));
    }

    #[test]
    fn test_border_collapse_parsing_and_recognition() {
        // Test border-collapse: separate
        assert_eq!(
            parse_property_value(
                "border-collapse",
                &[token(CssToken::Ident("separate".to_string()))]
            ),
            Some(CssValue::Keyword("separate".to_string()))
        );

        // Test border-collapse: collapse
        assert_eq!(
            parse_property_value(
                "border-collapse",
                &[token(CssToken::Ident("collapse".to_string()))]
            ),
            Some(CssValue::Keyword("collapse".to_string()))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "border-collapse",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("border-collapse"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "border-collapse",
            &CssValue::Keyword("separate".to_string())
        ));
        assert!(is_valid_property_value(
            "border-collapse",
            &CssValue::Keyword("collapse".to_string())
        ));
        assert!(!is_valid_property_value(
            "border-collapse",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test BorderCollapseValue::parse directly
        assert_eq!(
            BorderCollapseValue::parse("separate"),
            Some(BorderCollapseValue::Separate)
        );
        assert_eq!(
            BorderCollapseValue::parse("collapse"),
            Some(BorderCollapseValue::Collapse)
        );
        assert_eq!(BorderCollapseValue::parse("banana"), None);
        assert_eq!(
            BorderCollapseValue::parse("SEPARATE"),
            Some(BorderCollapseValue::Separate)
        );
    }

    #[test]
    fn test_text_align_last_parsing_and_recognition() {
        // Test text-align-last: auto
        assert_eq!(
            parse_property_value(
                "text-align-last",
                &[token(CssToken::Ident("auto".to_string()))]
            ),
            Some(CssValue::TextAlignLast(TextAlignLastValue::Auto))
        );

        // Test text-align-last: justify
        assert_eq!(
            parse_property_value(
                "text-align-last",
                &[token(CssToken::Ident("justify".to_string()))]
            ),
            Some(CssValue::TextAlignLast(TextAlignLastValue::Justify))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "text-align-last",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("text-align-last"));
        assert!(is_known_layout_property("Text-Align-Last"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "text-align-last",
            &CssValue::TextAlignLast(TextAlignLastValue::Auto)
        ));
        assert!(is_valid_property_value(
            "text-align-last",
            &CssValue::Keyword("auto".to_string())
        ));
        assert!(!is_valid_property_value(
            "text-align-last",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test TextAlignLastValue::parse directly
        assert_eq!(
            TextAlignLastValue::parse("auto"),
            Some(TextAlignLastValue::Auto)
        );
        assert_eq!(
            TextAlignLastValue::parse("justify"),
            Some(TextAlignLastValue::Justify)
        );
        assert_eq!(TextAlignLastValue::parse("banana"), None);
        assert_eq!(
            TextAlignLastValue::parse("JUSTIFY"),
            Some(TextAlignLastValue::Justify)
        );
    }

    #[test]
    fn test_unicode_bidi_parsing_and_recognition() {
        // Test unicode-bidi: normal
        assert_eq!(
            parse_property_value(
                "unicode-bidi",
                &[token(CssToken::Ident("normal".to_string()))]
            ),
            Some(CssValue::UnicodeBidi(UnicodeBidiValue::Normal))
        );

        // Test unicode-bidi: isolate-override
        assert_eq!(
            parse_property_value(
                "unicode-bidi",
                &[token(CssToken::Ident("isolate-override".to_string()))]
            ),
            Some(CssValue::UnicodeBidi(UnicodeBidiValue::IsolateOverride))
        );

        // Test invalid keyword "banana" -> None
        assert_eq!(
            parse_property_value(
                "unicode-bidi",
                &[token(CssToken::Ident("banana".to_string()))]
            ),
            None
        );

        // Test is_known_layout_property
        assert!(is_known_layout_property("unicode-bidi"));
        assert!(is_known_layout_property("Unicode-Bidi"));

        // Test is_valid_property_value
        assert!(is_valid_property_value(
            "unicode-bidi",
            &CssValue::UnicodeBidi(UnicodeBidiValue::Normal)
        ));
        assert!(is_valid_property_value(
            "unicode-bidi",
            &CssValue::Keyword("normal".to_string())
        ));
        assert!(!is_valid_property_value(
            "unicode-bidi",
            &CssValue::Keyword("banana".to_string())
        ));

        // Test UnicodeBidiValue::parse directly
        assert_eq!(
            UnicodeBidiValue::parse("normal"),
            Some(UnicodeBidiValue::Normal)
        );
        assert_eq!(
            UnicodeBidiValue::parse("isolate-override"),
            Some(UnicodeBidiValue::IsolateOverride)
        );
        assert_eq!(UnicodeBidiValue::parse("banana"), None);
        assert_eq!(
            UnicodeBidiValue::parse("ISOLATE-OVERRIDE"),
            Some(UnicodeBidiValue::IsolateOverride)
        );
    }

    #[test]
    fn test_print_color_adjust_direct() {
        use std::str::FromStr;

        // Test PrintColorAdjustValue::parse
        assert_eq!(
            PrintColorAdjustValue::parse("economy"),
            Some(PrintColorAdjustValue::Economy)
        );
        assert_eq!(
            PrintColorAdjustValue::parse("exact"),
            Some(PrintColorAdjustValue::Exact)
        );
        assert_eq!(
            PrintColorAdjustValue::parse("ECONOMY"),
            Some(PrintColorAdjustValue::Economy)
        );
        assert_eq!(PrintColorAdjustValue::parse("banana"), None);

        // Test as_str
        assert_eq!(PrintColorAdjustValue::Economy.as_str(), "economy");
        assert_eq!(PrintColorAdjustValue::Exact.as_str(), "exact");

        // Test FromStr
        assert_eq!(
            PrintColorAdjustValue::from_str("economy"),
            Ok(PrintColorAdjustValue::Economy)
        );
        assert_eq!(
            PrintColorAdjustValue::from_str("exact"),
            Ok(PrintColorAdjustValue::Exact)
        );
        assert_eq!(PrintColorAdjustValue::from_str("banana"), Err(()));

        // Test TryFrom<&CssValue>
        assert_eq!(
            PrintColorAdjustValue::try_from(&CssValue::PrintColorAdjust(
                PrintColorAdjustValue::Exact
            )),
            Ok(PrintColorAdjustValue::Exact)
        );
        assert_eq!(
            PrintColorAdjustValue::try_from(&CssValue::Keyword("economy".to_string())),
            Ok(PrintColorAdjustValue::Economy)
        );
        assert_eq!(
            PrintColorAdjustValue::try_from(&CssValue::Keyword("banana".to_string())),
            Err(())
        );
    }

    #[test]
    fn test_forced_color_adjust_direct() {
        use std::str::FromStr;

        // Default value
        assert_eq!(
            ForcedColorAdjustValue::default(),
            ForcedColorAdjustValue::Auto
        );

        // Test ForcedColorAdjustValue::parse
        assert_eq!(
            ForcedColorAdjustValue::parse("auto"),
            Some(ForcedColorAdjustValue::Auto)
        );
        assert_eq!(
            ForcedColorAdjustValue::parse("none"),
            Some(ForcedColorAdjustValue::None)
        );
        assert_eq!(
            ForcedColorAdjustValue::parse("AUTO"),
            Some(ForcedColorAdjustValue::Auto)
        );
        assert_eq!(ForcedColorAdjustValue::parse("banana"), None);

        // Test as_str
        assert_eq!(ForcedColorAdjustValue::Auto.as_str(), "auto");
        assert_eq!(ForcedColorAdjustValue::None.as_str(), "none");

        // Test FromStr
        assert_eq!(
            ForcedColorAdjustValue::from_str("auto"),
            Ok(ForcedColorAdjustValue::Auto)
        );
        assert_eq!(
            ForcedColorAdjustValue::from_str("none"),
            Ok(ForcedColorAdjustValue::None)
        );
        assert_eq!(ForcedColorAdjustValue::from_str("banana"), Err(()));

        // Test TryFrom<&CssValue>
        assert_eq!(
            ForcedColorAdjustValue::try_from(&CssValue::ForcedColorAdjust(
                ForcedColorAdjustValue::None
            )),
            Ok(ForcedColorAdjustValue::None)
        );
        assert_eq!(
            ForcedColorAdjustValue::try_from(&CssValue::Keyword("auto".to_string())),
            Ok(ForcedColorAdjustValue::Auto)
        );
        assert_eq!(
            ForcedColorAdjustValue::try_from(&CssValue::Keyword("banana".to_string())),
            Err(())
        );
    }

    #[test]
    fn test_color_scheme_direct() {
        use std::str::FromStr;

        // Test ColorSchemeValue::parse
        assert_eq!(
            ColorSchemeValue::parse("normal"),
            Some(ColorSchemeValue::Normal)
        );
        assert_eq!(
            ColorSchemeValue::parse("light"),
            Some(ColorSchemeValue::Light)
        );
        assert_eq!(
            ColorSchemeValue::parse("dark"),
            Some(ColorSchemeValue::Dark)
        );
        assert_eq!(
            ColorSchemeValue::parse("NORMAL"),
            Some(ColorSchemeValue::Normal)
        );
        assert_eq!(ColorSchemeValue::parse("banana"), None);

        // Test as_str
        assert_eq!(ColorSchemeValue::Normal.as_str(), "normal");
        assert_eq!(ColorSchemeValue::Light.as_str(), "light");
        assert_eq!(ColorSchemeValue::Dark.as_str(), "dark");

        // Test FromStr
        assert_eq!(
            ColorSchemeValue::from_str("normal"),
            Ok(ColorSchemeValue::Normal)
        );
        assert_eq!(
            ColorSchemeValue::from_str("light"),
            Ok(ColorSchemeValue::Light)
        );
        assert_eq!(
            ColorSchemeValue::from_str("dark"),
            Ok(ColorSchemeValue::Dark)
        );
        assert_eq!(ColorSchemeValue::from_str("banana"), Err(()));

        // Test TryFrom<&CssValue>
        assert_eq!(
            ColorSchemeValue::try_from(&CssValue::ColorScheme(ColorSchemeValue::Light)),
            Ok(ColorSchemeValue::Light)
        );
        assert_eq!(
            ColorSchemeValue::try_from(&CssValue::Keyword("dark".to_string())),
            Ok(ColorSchemeValue::Dark)
        );
        assert_eq!(
            ColorSchemeValue::try_from(&CssValue::Keyword("banana".to_string())),
            Err(())
        );
    }

    #[test]
    fn test_grid_template_with_functions() {
        // minmax(100px, 1fr)
        let minmax_comp = ComponentValue::Function {
            name: "minmax".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 100.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "fr".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_property_value("grid-template-columns", std::slice::from_ref(&minmax_comp)),
            Some(CssValue::Keyword("minmax(100px, 1fr)".to_string()))
        );

        // fit-content(40%)
        let fit_comp = ComponentValue::Function {
            name: "fit-content".to_string(),
            value: vec![token(CssToken::Percentage(40.0))],
        };
        assert_eq!(
            parse_property_value("grid-template-columns", std::slice::from_ref(&fit_comp)),
            Some(CssValue::Keyword("fit-content(40%)".to_string()))
        );

        // repeat(3, 20px)
        let repeat_comp = ComponentValue::Function {
            name: "repeat".to_string(),
            value: vec![
                token(CssToken::Number(3.0)),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_property_value("grid-template-columns", std::slice::from_ref(&repeat_comp)),
            Some(CssValue::Keyword("repeat(3, 20px)".to_string()))
        );

        // Combination: 100px minmax(100px, 1fr)
        let combo = [
            token(CssToken::Dimension {
                value: 100.0,
                unit: "px".to_string(),
            }),
            token(CssToken::Whitespace),
            minmax_comp,
        ];
        assert_eq!(
            parse_property_value("grid-template-columns", &combo),
            Some(CssValue::Keyword("100px minmax(100px, 1fr)".to_string()))
        );
    }

    #[test]
    fn test_attr_parsing() {
        // attr(data-size)
        let attr_simple = ComponentValue::Function {
            name: "attr".to_string(),
            value: vec![token(CssToken::Ident("data-size".to_string()))],
        };
        let inner_val1 = match &attr_simple {
            ComponentValue::Function { value, .. } => value,
            _ => unreachable!(),
        };
        let parsed = parse_attr_function(inner_val1);
        assert!(parsed.is_some());
        let val = parsed.unwrap();
        assert_eq!(val.name, "data-size");
        assert_eq!(val.type_or_unit, None);
        assert!(val.fallback.is_none());

        assert_eq!(
            parse_value(&[attr_simple]),
            Some(CssValue::Keyword("attr(data-size)".to_string()))
        );

        // attr(data-margin px, 10px)
        let attr_complex = ComponentValue::Function {
            name: "attr".to_string(),
            value: vec![
                token(CssToken::Ident("data-margin".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("px".to_string())),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        let inner_val2 = match &attr_complex {
            ComponentValue::Function { value, .. } => value,
            _ => unreachable!(),
        };
        let parsed2 = parse_attr_function(inner_val2);
        assert!(parsed2.is_some());
        let val2 = parsed2.unwrap();
        assert_eq!(val2.name, "data-margin");
        assert_eq!(val2.type_or_unit, Some("px".to_string()));
        assert!(val2.fallback.is_some());

        assert_eq!(
            parse_value(&[attr_complex]),
            Some(CssValue::Keyword("attr(data-margin px,10px)".to_string()))
        );
    }

    #[test]
    fn test_toggle_parsing() {
        // toggle(italic, normal)
        let toggle_comp = ComponentValue::Function {
            name: "toggle".to_string(),
            value: vec![
                token(CssToken::Ident("italic".to_string())),
                token(CssToken::Comma),
                token(CssToken::Ident("normal".to_string())),
            ],
        };
        let inner_val = match &toggle_comp {
            ComponentValue::Function { value, .. } => value,
            _ => unreachable!(),
        };
        let parsed = parse_toggle_function(inner_val);
        assert!(parsed.is_some());
        let val = parsed.unwrap();
        assert_eq!(val.values.len(), 2);
        assert_eq!(val.values[0], CssValue::Keyword("italic".to_string()));
        assert_eq!(val.values[1], CssValue::Keyword("normal".to_string()));

        assert_eq!(
            parse_value(&[toggle_comp]),
            Some(CssValue::Keyword("toggle(italic,normal)".to_string()))
        );
    }

    #[test]
    fn test_scroll_view_parsing() {
        // scroll(root block)
        let scroll_comp = ComponentValue::Function {
            name: "scroll".to_string(),
            value: vec![
                token(CssToken::Ident("root".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("block".to_string())),
            ],
        };
        let inner_val1 = match &scroll_comp {
            ComponentValue::Function { value, .. } => value,
            _ => unreachable!(),
        };
        let parsed = parse_scroll_function(inner_val1);
        assert!(parsed.is_some());
        let val = parsed.unwrap();
        assert_eq!(val.scroller, Some("root".to_string()));
        assert_eq!(val.axis, Some("block".to_string()));

        assert_eq!(
            parse_value(&[scroll_comp]),
            Some(CssValue::Keyword("scroll(root block)".to_string()))
        );

        // view(block)
        let view_comp = ComponentValue::Function {
            name: "view".to_string(),
            value: vec![token(CssToken::Ident("block".to_string()))],
        };
        let inner_val2 = match &view_comp {
            ComponentValue::Function { value, .. } => value,
            _ => unreachable!(),
        };
        let parsed2 = parse_view_function(inner_val2);
        assert!(parsed2.is_some());
        let val2 = parsed2.unwrap();
        assert_eq!(val2.axis, Some("block".to_string()));
        assert_eq!(val2.inset, None);

        assert_eq!(
            parse_value(&[view_comp]),
            Some(CssValue::Keyword("view(block)".to_string()))
        );
    }

    #[test]
    fn test_modern_css_value_functions() {
        // Test calc(10px + 20px)
        let calc_comp = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[calc_comp]),
            Some(CssValue::Length(30.0, LengthUnit::Px))
        );

        // Test clamp(10px, 50%, 100px)
        let clamp_comp = ComponentValue::Function {
            name: "clamp".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Percentage(50.0)),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 100.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[clamp_comp]),
            Some(CssValue::Keyword("clamp(10px,50%,100px)".to_string()))
        );

        // Test sin(45deg)
        let sin_comp = ComponentValue::Function {
            name: "sin".to_string(),
            value: vec![token(CssToken::Dimension {
                value: 45.0,
                unit: "deg".to_string(),
            })],
        };
        assert_eq!(
            parse_value(&[sin_comp]),
            Some(CssValue::Keyword("sin(45deg)".to_string()))
        );

        // Test blur(5px)
        let blur_comp = ComponentValue::Function {
            name: "blur".to_string(),
            value: vec![token(CssToken::Dimension {
                value: 5.0,
                unit: "px".to_string(),
            })],
        };
        assert_eq!(
            parse_value(&[blur_comp]),
            Some(CssValue::Keyword("blur(5px)".to_string()))
        );

        // Test polygon(50% 0%, 0% 100%, 100% 100%)
        let polygon_comp = ComponentValue::Function {
            name: "polygon".to_string(),
            value: vec![
                token(CssToken::Percentage(50.0)),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(0.0)),
                token(CssToken::Comma),
                token(CssToken::Percentage(0.0)),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(100.0)),
                token(CssToken::Comma),
                token(CssToken::Percentage(100.0)),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(100.0)),
            ],
        };
        assert_eq!(
            parse_value(&[polygon_comp]),
            Some(CssValue::Keyword(
                "polygon(50% 0%,0% 100%,100% 100%)".to_string()
            ))
        );

        // Test env(safe-area-inset-top)
        let env_comp = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![token(CssToken::Ident("safe-area-inset-top".to_string()))],
        };
        assert_eq!(
            parse_value(&[env_comp]),
            Some(CssValue::Keyword("env(safe-area-inset-top)".to_string()))
        );

        // Test calc-size(auto, 100% - 20px)
        let calc_size_comp = ComponentValue::Function {
            name: "calc-size".to_string(),
            value: vec![
                token(CssToken::Ident("auto".to_string())),
                token(CssToken::Comma),
                token(CssToken::Percentage(100.0)),
                token(CssToken::Whitespace),
                token(CssToken::Delim('-')),
                token(CssToken::Whitespace),
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[calc_size_comp]),
            Some(CssValue::Keyword("calc-size(auto,100% - 20px)".to_string()))
        );

        // Test container-progress(...)
        let cp_comp = ComponentValue::Function {
            name: "container-progress".to_string(),
            value: vec![
                token(CssToken::Ident("width".to_string())),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 100.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 500.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[cp_comp]),
            Some(CssValue::Keyword(
                "container-progress(width,100px,500px)".to_string()
            ))
        );

        // Test scroll-progress(...)
        let sp_comp = ComponentValue::Function {
            name: "scroll-progress".to_string(),
            value: vec![
                token(CssToken::Ident("block".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Ident("nearest".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[sp_comp]),
            Some(CssValue::Keyword(
                "scroll-progress(block nearest)".to_string()
            ))
        );

        // Test view-progress(...)
        let vp_comp = ComponentValue::Function {
            name: "view-progress".to_string(),
            value: vec![token(CssToken::Ident("inline".to_string()))],
        };
        assert_eq!(
            parse_value(&[vp_comp]),
            Some(CssValue::Keyword("view-progress(inline)".to_string()))
        );

        // Test image(...)
        let img_comp = ComponentValue::Function {
            name: "image".to_string(),
            value: vec![
                token(CssToken::String("fallback.png".to_string())),
                token(CssToken::Comma),
                token(CssToken::Ident("blue".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[img_comp]),
            Some(CssValue::Keyword(
                "image(\"fallback.png\",blue)".to_string()
            ))
        );

        // Test element(...)
        let elem_comp = ComponentValue::Function {
            name: "element".to_string(),
            value: vec![token(CssToken::Hash("my-canvas".to_string()))],
        };
        assert_eq!(
            parse_value(&[elem_comp]),
            Some(CssValue::Keyword("element(#my-canvas)".to_string()))
        );

        // Test paint(...)
        let paint_comp = ComponentValue::Function {
            name: "paint".to_string(),
            value: vec![token(CssToken::Ident("custom-painter".to_string()))],
        };
        assert_eq!(
            parse_value(&[paint_comp]),
            Some(CssValue::Keyword("paint(custom-painter)".to_string()))
        );

        // Test src(...)
        let src_comp = ComponentValue::Function {
            name: "src".to_string(),
            value: vec![token(CssToken::String("font.woff2".to_string()))],
        };
        assert_eq!(
            parse_value(&[src_comp]),
            Some(CssValue::Keyword("src(\"font.woff2\")".to_string()))
        );

        // Test shape(...)
        let shape_comp = ComponentValue::Function {
            name: "shape".to_string(),
            value: vec![
                token(CssToken::Ident("from".to_string())),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(0.0)),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(0.0)),
            ],
        };
        assert_eq!(
            parse_value(&[shape_comp]),
            Some(CssValue::Keyword("shape(from 0% 0%)".to_string()))
        );

        // Test ray(...)
        let ray_comp = ComponentValue::Function {
            name: "ray".to_string(),
            value: vec![token(CssToken::Dimension {
                value: 45.0,
                unit: "deg".to_string(),
            })],
        };
        assert_eq!(
            parse_value(&[ray_comp]),
            Some(CssValue::Keyword("ray(45deg)".to_string()))
        );
    }

    #[test]
    fn test_t0842_missing_css_value_features() {
        // Test parsing of newly added length units in parse_property_value
        let absolute_units = [
            ("in", 5.0 * 96.0),
            ("cm", 5.0 * 96.0 / 2.54),
            ("mm", 5.0 * 9.6 / 2.54),
            ("pc", 5.0 * 16.0),
            ("q", 5.0 * 96.0 / 101.6),
        ];

        for &(unit_str, expected_px) in &absolute_units {
            let comp = token(CssToken::Dimension {
                value: 5.0,
                unit: unit_str.to_string(),
            });
            // 1. Check general parse_property_value for "margin-left" (absolute units convert to Px on-the-fly)
            assert_eq!(
                parse_property_value("margin-left", std::slice::from_ref(&comp)),
                Some(CssValue::Length(expected_px, LengthUnit::Px))
            );

            // 2. Check parse_length_or_percent
            let parsed_lop = parse_length_or_percent(&comp);
            assert!(parsed_lop.is_some());
            let lop = parsed_lop.unwrap();
            assert_eq!(lop.unit, LengthUnit::Px);
            assert_eq!(lop.value, expected_px);
        }

        let relative_units = ["ex", "ch", "vmin", "vmax"];
        for unit_str in &relative_units {
            let comp = token(CssToken::Dimension {
                value: 5.0,
                unit: unit_str.to_string(),
            });
            // Relative units compile to Keyword
            assert_eq!(
                parse_property_value("margin-left", std::slice::from_ref(&comp)),
                Some(CssValue::Keyword(format!("5{}", unit_str)))
            );
        }

        // Test global keywords handled centrally in parse_property_value
        let global_keywords = ["inherit", "initial", "unset", "revert", "revert-layer"];
        for kw in &global_keywords {
            let comp = token(CssToken::Ident(kw.to_string()));

            // Check properties that use standard parser
            assert_eq!(
                parse_property_value("margin-left", std::slice::from_ref(&comp)),
                Some(CssValue::Keyword(kw.to_string()))
            );

            // Check properties that usually use specialized parsers (like mix-blend-mode or resize)
            assert_eq!(
                parse_property_value("mix-blend-mode", std::slice::from_ref(&comp)),
                Some(CssValue::Keyword(kw.to_string()))
            );
            assert_eq!(
                parse_property_value("resize", std::slice::from_ref(&comp)),
                Some(CssValue::Keyword(kw.to_string()))
            );
        }
    }

    #[test]
    fn test_t0890_css_values_completeness() {
        // 1. Test rgb space-separated / modern slash syntax
        let rgb_modern = vec![
            token(CssToken::Number(255.0)),
            token(CssToken::Whitespace),
            token(CssToken::Number(0.0)),
            token(CssToken::Whitespace),
            token(CssToken::Number(0.0)),
            token(CssToken::Whitespace),
            token(CssToken::Delim('/')),
            token(CssToken::Whitespace),
            token(CssToken::Number(0.5)),
        ];
        assert_eq!(
            parse_rgb_function(&rgb_modern),
            Some(Color::Rgba(255, 0, 0, 127))
        );

        // 2. Test rgb legacy / percentage alpha
        let rgba_percentage = vec![
            token(CssToken::Number(0.0)),
            token(CssToken::Comma),
            token(CssToken::Number(255.0)),
            token(CssToken::Comma),
            token(CssToken::Number(0.0)),
            token(CssToken::Comma),
            token(CssToken::Percentage(50.0)),
        ];
        assert_eq!(
            parse_rgb_function(&rgba_percentage),
            Some(Color::Rgba(0, 255, 0, 127))
        );

        // 3. Test hsl hue angle / slash syntax
        let hsl_angle = vec![
            token(CssToken::Dimension {
                value: 120.0,
                unit: "deg".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(100.0)),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(50.0)),
            token(CssToken::Whitespace),
            token(CssToken::Delim('/')),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(50.0)),
        ];
        assert_eq!(
            parse_hsl_function(&hsl_angle),
            Some(Color::Rgba(0, 255, 0, 127))
        );

        // 4. Test hwb hue angle / slash syntax
        let hwb_angle = vec![
            token(CssToken::Dimension {
                value: 120.0,
                unit: "deg".to_string(),
            }),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(0.0)),
            token(CssToken::Whitespace),
            token(CssToken::Percentage(0.0)),
            token(CssToken::Whitespace),
            token(CssToken::Delim('/')),
            token(CssToken::Whitespace),
            token(CssToken::Number(0.5)),
        ];
        assert_eq!(
            parse_hwb_function(&hwb_angle),
            Some(Color::Rgba(0, 255, 0, 127))
        );

        // 5. Test color-mix with srgb-linear colorspace interpolation
        let color_mix_linear = vec![
            token(CssToken::Ident("in".to_string())),
            token(CssToken::Whitespace),
            token(CssToken::Ident("srgb-linear".to_string())),
            token(CssToken::Comma),
            token(CssToken::Ident("red".to_string())),
            token(CssToken::Comma),
            token(CssToken::Ident("blue".to_string())),
        ];
        assert_eq!(
            parse_color_mix_function(&color_mix_linear),
            Some(Color::Rgba(188, 0, 188, 255))
        );
    }

    #[test]
    fn test_t0906_css_values_additive_parsing() {
        // 1. Test parsing of new viewport & relative units to Keyword in parse_single_value
        let test_units = [
            "svw", "svh", "lvw", "lvh", "dvw", "dvh", "svmin", "svmax", "lvmin", "lvmax", "dvmin",
            "dvmax", "vi", "svi", "lvi", "dvi", "vb", "svb", "lvb", "dvb", "rex", "rch", "ric",
            "rcap", "ic", "cap", "lh", "rlh",
        ];
        for unit in &test_units {
            let comp = token(CssToken::Dimension {
                value: 42.0,
                unit: unit.to_string(),
            });
            assert_eq!(
                parse_value(&[comp]),
                Some(CssValue::Keyword(format!("42{}", unit)))
            );
        }

        // 2. Test relative RGB with min() function evaluation
        let rgb_components = vec![
            token(CssToken::Ident("from".to_string())),
            token(CssToken::Ident("red".to_string())),
            ComponentValue::Function {
                name: "min".to_string(),
                value: vec![
                    token(CssToken::Ident("r".to_string())),
                    token(CssToken::Comma),
                    token(CssToken::Number(100.0)),
                ],
            },
            token(CssToken::Number(0.0)),
            token(CssToken::Number(0.0)),
        ];
        // red is rgba(255, 0, 0). min(r, 100) -> min(255, 100) -> 100.
        // Result should be rgba(100, 0, 0, 255).
        assert_eq!(
            parse_rgb_function(&rgb_components),
            Some(Color::Rgba(100, 0, 0, 255))
        );

        // 3. Test relative HSL with clamp() function evaluation
        let hsl_components = vec![
            token(CssToken::Ident("from".to_string())),
            token(CssToken::Ident("green".to_string())), // green is rgba(0, 128, 0), H=120, S=100%, L=25.1%
            token(CssToken::Ident("h".to_string())),
            token(CssToken::Percentage(100.0)),
            ComponentValue::Function {
                name: "clamp".to_string(),
                value: vec![
                    token(CssToken::Number(30.0)),
                    token(CssToken::Comma),
                    token(CssToken::Ident("l".to_string())),
                    token(CssToken::Comma),
                    token(CssToken::Number(40.0)),
                ],
            },
        ];
        // green's L is 25.1%. clamp(30, l, 40) -> clamp(30, 25.1, 40) -> 30.
        // Result should be HSL(120, 100%, 30%) -> rgba(0, 153, 0, 255)
        assert_eq!(
            parse_hsl_function(&hsl_components),
            Some(Color::Rgba(0, 153, 0, 255))
        );
    }

    #[test]
    fn test_t0943_css_values_additions() {
        // 1. Test new parsed length, angle, resolution, and container query units
        let test_units = [
            "deg", "rad", "grad", "turn", "dpi", "dpcm", "dppx", "x", "cqw", "cqh", "cqi", "cqb",
            "cqmin", "cqmax",
        ];
        for unit in &test_units {
            let comp = token(CssToken::Dimension {
                value: 12.5,
                unit: unit.to_string(),
            });
            assert_eq!(
                parse_value(&[comp]),
                Some(CssValue::Keyword(format!("12.5{}", unit)))
            );
        }

        // 2. Test env() fallback validation
        // Valid env with no fallback
        let env_valid_no_fallback = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![token(CssToken::Ident("safe-area-inset-top".to_string()))],
        };
        assert_eq!(
            parse_value(&[env_valid_no_fallback]),
            Some(CssValue::Keyword("env(safe-area-inset-top)".to_string()))
        );

        // Valid env with fallback
        let env_valid_with_fallback = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![
                token(CssToken::Ident("safe-area-inset-bottom".to_string())),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[env_valid_with_fallback]),
            Some(CssValue::Keyword(
                "env(safe-area-inset-bottom,20px)".to_string()
            ))
        );

        // Invalid env: empty
        let env_empty = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![],
        };
        assert_eq!(parse_value(&[env_empty]), None);

        // Invalid env: first arg not ident
        let env_first_not_ident = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![token(CssToken::Number(10.0))],
        };
        assert_eq!(parse_value(&[env_first_not_ident]), None);

        // Invalid env: missing fallback after comma
        let env_missing_fallback = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![
                token(CssToken::Ident("safe-area-inset-left".to_string())),
                token(CssToken::Comma),
            ],
        };
        assert_eq!(parse_value(&[env_missing_fallback]), None);

        // Invalid env: missing comma before fallback
        let env_missing_comma = ComponentValue::Function {
            name: "env".to_string(),
            value: vec![
                token(CssToken::Ident("safe-area-inset-left".to_string())),
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(parse_value(&[env_missing_comma]), None);

        // 3. Test nested calc() validation
        // Valid nested calc
        let nested_calc_valid = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 5.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('*')),
                        token(CssToken::Whitespace),
                        token(CssToken::Number(2.0)),
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[nested_calc_valid]),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );

        // Invalid nested calc (contains invalid token/function)
        let nested_calc_invalid = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "invalid-fn".to_string(),
                    value: vec![token(CssToken::Number(5.0))],
                },
            ],
        };
        assert_eq!(parse_value(&[nested_calc_invalid]), None);
    }

    #[test]
    fn test_t0963_css_values_additions() {
        // 1. Valid var with no fallback
        let var_no_fallback = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![token(CssToken::Ident("--my-color".to_string()))],
        };
        assert_eq!(
            parse_value(&[var_no_fallback]),
            Some(CssValue::Keyword("var(--my-color)".to_string()))
        );

        // 2. Valid var with fallback
        let var_with_fallback = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![
                token(CssToken::Ident("--my-color".to_string())),
                token(CssToken::Comma),
                token(CssToken::Ident("red".to_string())),
            ],
        };
        assert_eq!(
            parse_value(&[var_with_fallback]),
            Some(CssValue::Keyword("var(--my-color,red)".to_string()))
        );

        // 3. Valid var with empty fallback
        let var_empty_fallback = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![
                token(CssToken::Ident("--my-color".to_string())),
                token(CssToken::Comma),
                token(CssToken::Whitespace),
            ],
        };
        assert_eq!(
            parse_value(&[var_empty_fallback]),
            Some(CssValue::Keyword("var(--my-color, )".to_string()))
        );

        // 4. Invalid var: empty
        let var_empty = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![],
        };
        assert_eq!(parse_value(&[var_empty]), None);

        // 5. Invalid var: not starting with --
        let var_invalid_name = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![token(CssToken::Ident("my-color".to_string()))],
        };
        assert_eq!(parse_value(&[var_invalid_name]), None);

        // 6. Invalid var: missing comma before fallback
        let var_missing_comma = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![
                token(CssToken::Ident("--my-color".to_string())),
                token(CssToken::Ident("red".to_string())),
            ],
        };
        assert_eq!(parse_value(&[var_missing_comma]), None);
    }

    #[test]
    fn test_t0984_calc_nesting_clamp_min_max_and_units() {
        // 1. Nested calc() validation: calc(1px + calc(2px + calc(3px + 4px)))
        let nested_calc_deep = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 2.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('+')),
                        token(CssToken::Whitespace),
                        ComponentValue::Function {
                            name: "calc".to_string(),
                            value: vec![
                                token(CssToken::Dimension {
                                    value: 3.0,
                                    unit: "px".to_string(),
                                }),
                                token(CssToken::Whitespace),
                                token(CssToken::Delim('+')),
                                token(CssToken::Whitespace),
                                token(CssToken::Dimension {
                                    value: 4.0,
                                    unit: "px".to_string(),
                                }),
                            ],
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[nested_calc_deep]),
            Some(CssValue::Length(10.0, LengthUnit::Px))
        );

        // 2. clamp(), min(), max() parsing with nested expressions
        // min(10px, calc(5px + 2px))
        let min_with_nested_calc = ComponentValue::Function {
            name: "min".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 5.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('+')),
                        token(CssToken::Whitespace),
                        token(CssToken::Dimension {
                            value: 2.0,
                            unit: "px".to_string(),
                        }),
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[min_with_nested_calc]),
            Some(CssValue::Length(7.0, LengthUnit::Px))
        );

        // clamp(calc(1px + 1px), 50%, calc(100px - 10px))
        let clamp_with_nested_calcs = ComponentValue::Function {
            name: "clamp".to_string(),
            value: vec![
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 1.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('+')),
                        token(CssToken::Whitespace),
                        token(CssToken::Dimension {
                            value: 1.0,
                            unit: "px".to_string(),
                        }),
                    ],
                },
                token(CssToken::Comma),
                token(CssToken::Percentage(50.0)),
                token(CssToken::Comma),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 100.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('-')),
                        token(CssToken::Whitespace),
                        token(CssToken::Dimension {
                            value: 10.0,
                            unit: "px".to_string(),
                        }),
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[clamp_with_nested_calcs]),
            Some(CssValue::Keyword("clamp(2px,50%,90px)".to_string()))
        );

        // 3. Additional unit conversions: physical absolute units to Px
        // 1in = 96px
        let comp_in = token(CssToken::Dimension {
            value: 1.0,
            unit: "in".to_string(),
        });
        assert_eq!(
            parse_value(&[comp_in]),
            Some(CssValue::Length(96.0, LengthUnit::Px))
        );

        // 2.54cm = 96px
        let comp_cm = token(CssToken::Dimension {
            value: 2.54,
            unit: "cm".to_string(),
        });
        assert_eq!(
            parse_value(&[comp_cm]),
            Some(CssValue::Length(96.0, LengthUnit::Px))
        );

        // 25.4mm = 96px
        let comp_mm = token(CssToken::Dimension {
            value: 25.4,
            unit: "mm".to_string(),
        });
        let val_mm = match parse_value(&[comp_mm]).unwrap() {
            CssValue::Length(v, LengthUnit::Px) => v,
            _ => panic!("Expected Px length"),
        };
        assert!((val_mm - 96.0).abs() < 1e-4);

        // 6pc = 96px
        let comp_pc = token(CssToken::Dimension {
            value: 6.0,
            unit: "pc".to_string(),
        });
        assert_eq!(
            parse_value(&[comp_pc]),
            Some(CssValue::Length(96.0, LengthUnit::Px))
        );

        // 101.6q = 96px
        let comp_q = token(CssToken::Dimension {
            value: 101.6,
            unit: "q".to_string(),
        });
        let val_q = match parse_value(&[comp_q]).unwrap() {
            CssValue::Length(v, LengthUnit::Px) => v,
            _ => panic!("Expected Px length"),
        };
        assert!((val_q - 96.0).abs() < 1e-4);
    }

    #[test]
    fn test_t1003_css_values_extension_coverage() {
        // 1. Math functions min/max/clamp evaluation and simplification
        // min(120px, 80px) -> 80px
        let min_comp = ComponentValue::Function {
            name: "min".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 120.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 80.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[min_comp]),
            Some(CssValue::Length(80.0, LengthUnit::Px))
        );

        // max(1in, 100px) -> 1in is 96px, so max(96px, 100px) -> 100px
        let max_comp = ComponentValue::Function {
            name: "max".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 1.0,
                    unit: "in".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 100.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[max_comp]),
            Some(CssValue::Length(100.0, LengthUnit::Px))
        );

        // clamp(50px, 75px, 100px) -> 75px
        let clamp_comp = ComponentValue::Function {
            name: "clamp".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 50.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 75.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 100.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[clamp_comp]),
            Some(CssValue::Length(75.0, LengthUnit::Px))
        );

        // 2. Unit Normalization: pt unit conversion (12pt = 16px)
        let pt_comp = token(CssToken::Dimension {
            value: 12.0,
            unit: "pt".to_string(),
        });
        // Non-calc pt parses to Length(12.0, Pt)
        assert_eq!(
            parse_value(std::slice::from_ref(&pt_comp)),
            Some(CssValue::Length(12.0, LengthUnit::Pt))
        );
        // Inside calc(), pt gets normalized to Px
        let calc_pt = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![pt_comp],
        };
        assert_eq!(
            parse_value(&[calc_pt]),
            Some(CssValue::Length(16.0, LengthUnit::Px))
        );

        // 3. Global keywords and is_valid_property_value case-insensitivity
        let props = ["scroll-snap-type", "mix-blend-mode", "resize"];
        let keywords = ["InHeRiT", "INITIAL", "unset", "revert-layer"];
        for prop in &props {
            for kw in &keywords {
                let val = CssValue::Keyword(kw.to_string());
                assert!(is_valid_property_value(prop, &val));
            }
        }

        // 4. Edge-case serialization: sum/diff preserves spaces around operators
        let mixed_calc = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                token(CssToken::Percentage(50.0)),
            ],
        };
        assert_eq!(
            parse_value(&[mixed_calc]),
            Some(CssValue::Keyword("calc(10px + 50%)".to_string()))
        );
    }

    #[test]
    fn test_t1019_css_values_correctness() {
        // 1. calc() nesting and unit mixing with insertion-order preservation
        let mixed_nesting_comp = ComponentValue::Function {
            name: "calc".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Whitespace),
                token(CssToken::Delim('+')),
                token(CssToken::Whitespace),
                ComponentValue::Function {
                    name: "calc".to_string(),
                    value: vec![
                        token(CssToken::Dimension {
                            value: 20.0,
                            unit: "px".to_string(),
                        }),
                        token(CssToken::Whitespace),
                        token(CssToken::Delim('+')),
                        token(CssToken::Whitespace),
                        token(CssToken::Percentage(10.0)),
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[mixed_nesting_comp]),
            Some(CssValue::Keyword("calc(30px + 10%)".to_string()))
        );

        // 2. clamp() resolution with min > max (max(min, min(val, max)) behavior)
        let clamp_min_gt_max = ComponentValue::Function {
            name: "clamp".to_string(),
            value: vec![
                token(CssToken::Dimension {
                    value: 20.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 15.0,
                    unit: "px".to_string(),
                }),
                token(CssToken::Comma),
                token(CssToken::Dimension {
                    value: 10.0,
                    unit: "px".to_string(),
                }),
            ],
        };
        assert_eq!(
            parse_value(&[clamp_min_gt_max]),
            Some(CssValue::Length(20.0, LengthUnit::Px))
        );

        // 3. LengthOrPercent resolution contexts
        let lp_pct = LengthOrPercent {
            value: 50.0,
            unit: LengthUnit::Percent,
        };
        assert_eq!(lp_pct.resolve(500.0, 16.0, 16.0, 1024.0, 768.0), 250.0);

        let lp_em = LengthOrPercent {
            value: 2.0,
            unit: LengthUnit::Em,
        };
        assert_eq!(lp_em.resolve(500.0, 18.0, 16.0, 1024.0, 768.0), 36.0);

        let val_len = CssValue::Length(1.5, LengthUnit::Rem);
        assert_eq!(
            val_len.resolve_to_px(500.0, 16.0, 16.0, 1024.0, 768.0),
            Some(24.0)
        );

        // 4. var() fallback chains with multiple levels of nesting
        let var_chain = ComponentValue::Function {
            name: "var".to_string(),
            value: vec![
                token(CssToken::Ident("--x".to_string())),
                token(CssToken::Comma),
                ComponentValue::Function {
                    name: "var".to_string(),
                    value: vec![
                        token(CssToken::Ident("--y".to_string())),
                        token(CssToken::Comma),
                        ComponentValue::Function {
                            name: "var".to_string(),
                            value: vec![
                                token(CssToken::Ident("--z".to_string())),
                                token(CssToken::Comma),
                                token(CssToken::Ident("red".to_string())),
                                token(CssToken::Comma),
                                token(CssToken::Ident("green".to_string())),
                            ],
                        },
                    ],
                },
            ],
        };
        assert_eq!(
            parse_value(&[var_chain]),
            Some(CssValue::Keyword(
                "var(--x,var(--y,var(--z,red,green)))".to_string()
            ))
        );

        // 5. Non-negative transition-duration and animation-duration validation
        let valid_duration = CssValue::Keyword("200ms".to_string());
        assert!(is_valid_property_value(
            "transition-duration",
            &valid_duration
        ));
        assert!(is_valid_property_value(
            "animation-duration",
            &valid_duration
        ));

        let invalid_duration = CssValue::Keyword("-50ms".to_string());
        assert!(!is_valid_property_value(
            "transition-duration",
            &invalid_duration
        ));
        assert!(!is_valid_property_value(
            "animation-duration",
            &invalid_duration
        ));

        let valid_delay = CssValue::Keyword("-50ms".to_string());
        assert!(is_valid_property_value("transition-delay", &valid_delay));
        assert!(is_valid_property_value("animation-delay", &valid_delay));
    }
}
