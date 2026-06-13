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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextOrientationValue {
    Mixed,
    Upright,
    Sideways,
}

impl TextOrientationValue {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "mixed" => Some(Self::Mixed),
            "upright" => Some(Self::Upright),
            "sideways" | "sideways-right" => Some(Self::Sideways),
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
            | "scroll-snap-type"
            | "scroll-snap-align"
            | "mix-blend-mode"
            | "background-blend-mode"
            | "isolation"
            | "resize"
            | "backface-visibility"
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
            | "grid-template-columns"
            | "grid-template-rows"
            | "image-rendering"
            | "font-kerning"
            | "text-justify"
            | "word-break"
            | "overflow-wrap"
            | "word-wrap"
            | "object-fit"
            | "caption-side"
            | "pointer-events"
    )
}

/// Validates that a CSS value is valid for a layout-related property.
pub fn is_valid_property_value(name: &str, value: &CssValue) -> bool {
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
            _ => false,
        },
        "image-rendering" => match value {
            CssValue::Keyword(kw) => {
                matches!(
                    kw.to_ascii_lowercase().as_str(),
                    "auto" | "smooth" | "high-quality" | "crisp-edges" | "pixelated"
                )
            }
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

fn parse_grid_template(components: &[ComponentValue]) -> Option<CssValue> {
    let mut tracks = Vec::new();

    for component in components {
        match component {
            ComponentValue::Token(CssToken::Whitespace) => {
                // Skip whitespace
            }
            ComponentValue::Token(CssToken::Dimension { value, unit }) => {
                let lower_unit = unit.to_ascii_lowercase();
                match lower_unit.as_str() {
                    "px" => tracks.push(GridTrackSize::Px(*value as f32)),
                    "em" | "rem" | "pt" | "vw" | "vh" => {
                        tracks.push(GridTrackSize::Px(*value as f32));
                    }
                    "fr" => tracks.push(GridTrackSize::Fr(*value as f32)),
                    _ => {
                        // TODO(spec): minmax(), repeat(), fit-content, named lines not yet supported
                    }
                }
            }
            ComponentValue::Token(CssToken::Percentage(v)) => {
                tracks.push(GridTrackSize::Percent(*v as f32));
            }
            ComponentValue::Token(CssToken::Number(v)) if *v == 0.0 => {
                tracks.push(GridTrackSize::Px(0.0));
            }
            ComponentValue::Token(CssToken::Ident(s)) if s.eq_ignore_ascii_case("auto") => {
                tracks.push(GridTrackSize::Auto);
            }
            _ => {
                // TODO(spec): minmax(), repeat(), fit-content, named lines not yet supported
            }
        }
    }

    Some(CssValue::GridTemplate(tracks))
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

/// Parses a list of component values for a specific property, returning a typed CSS value if it matches a known layout property.
pub fn parse_property_value(
    property_name: &str,
    components: &[ComponentValue],
) -> Option<CssValue> {
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
        "image-rendering" => {
            if let CssValue::Keyword(kw) = &val {
                match kw.to_ascii_lowercase().as_str() {
                    "auto" | "smooth" | "high-quality" | "crisp-edges" | "pixelated" => Some(val),
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
            if lower_unit == "s" || lower_unit == "ms" || lower_unit == "fr" {
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
                || name.eq_ignore_ascii_case("conic-gradient")
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
            TextOrientationValue::parse("sideways-right"),
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
        assert_eq!(
            TextOrientationValue::parse("Sideways-Right"),
            Some(TextOrientationValue::Sideways)
        );
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
        assert_eq!(
            "sideways-right".parse::<TextOrientationValue>(),
            Ok(TextOrientationValue::Sideways)
        );
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
            TextOrientationValue::try_from(&CssValue::Keyword("sideways-right".to_string())),
            Ok(TextOrientationValue::Sideways)
        );
        assert_eq!(
            TextOrientationValue::try_from(&CssValue::Number(1.0)),
            Err(())
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

        // 7. Invalid inputs return None
        assert!(parse("skew(10deg)").is_none());
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

        // Test parse_property_value and is_valid_property_value for image-rendering (t0541)
        assert!(is_known_layout_property("image-rendering"));
        assert!(is_known_layout_property("Image-Rendering"));

        for val in &[
            "auto",
            "smooth",
            "high-quality",
            "crisp-edges",
            "pixelated",
            "AUTO",
            "Smooth",
        ] {
            assert_eq!(
                parse_property_value(
                    "image-rendering",
                    &[token(CssToken::Ident(val.to_string()))]
                ),
                Some(CssValue::Keyword(val.to_string()))
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
}
