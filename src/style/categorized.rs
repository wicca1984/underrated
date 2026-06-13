//! Production target style type per ADR 0001, introduced additively.
//! The legacy `ComputedStyle` is migrated off in later tasks.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// Sentinel for `line-height: normal` / unset. The typed `line_height: u32` px field
/// cannot otherwise distinguish an unspecified line-height (which must fall back to the
/// font's intrinsic line height at layout time) from an authored pixel value — a
/// distinction the legacy HashMap `ComputedStyle` preserved via `Option`.
pub const LINE_HEIGHT_NORMAL: u32 = u32::MAX;

/// Group of inherited text and font properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedText {
    pub color: String,
    pub font_family: String,
    pub font_size: u32,
    pub font_style: String,
    pub font_weight: String,
    pub line_height: u32,
    /// Set when `line-height` was specified as a unitless number (`line-height: 2`).
    /// Spec: a unitless line-height inherits the *number*, and each element resolves
    /// it against its own font-size. `line_height` (px) holds this element's resolved
    /// value; this preserves the number so descendants recompute correctly and
    /// `get("line-height")` reports `Number(n)` rather than the resolved px.
    pub line_height_number: Option<f32>,
    pub text_align: String,
    pub letter_spacing: i32,
    pub word_spacing: i32,
    pub white_space: String,
    pub direction: String,
    pub text_transform: String,
    pub font_variant: String,
    pub font_stretch: String,
    pub text_indent: i32,
    pub word_break: String,
    pub line_break: String,
    pub text_orientation: String,
    pub overflow_wrap: String,
    pub text_align_last: String,
    pub tab_size: u32,
    pub hyphens: String,
    pub text_rendering: String,
    pub image_rendering: String,
    pub font_variant_caps: String,
    pub text_shadow: Option<crate::css::values::CssValue>,
}

impl Default for InheritedText {
    fn default() -> Self {
        Self {
            // Empty = unspecified. Kept distinct from an explicit black so that the
            // paint-time link-color default (resolve_text_color) can tell an authored
            // color from the initial value; parse_css_color("") is None -> black fallback.
            color: String::new(),
            font_family: "sans-serif".to_string(),
            font_size: 16,
            font_style: "normal".to_string(),
            font_weight: "normal".to_string(),
            line_height: LINE_HEIGHT_NORMAL,
            line_height_number: None,
            text_align: "start".to_string(),
            letter_spacing: -1,
            word_spacing: -1,
            white_space: "normal".to_string(),
            direction: "ltr".to_string(),
            text_transform: "none".to_string(),
            font_variant: "normal".to_string(),
            font_stretch: "normal".to_string(),
            text_indent: -1,
            word_break: "normal".to_string(),
            line_break: "auto".to_string(),
            text_orientation: "mixed".to_string(),
            overflow_wrap: "normal".to_string(),
            text_align_last: "auto".to_string(),
            tab_size: 8,
            hyphens: "manual".to_string(),
            text_rendering: "auto".to_string(),
            image_rendering: "auto".to_string(),
            font_variant_caps: "normal".to_string(),
            text_shadow: None,
        }
    }
}

/// Group of inherited list properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedList {
    pub list_style_type: String,
    pub list_style_position: String,
    pub list_style_image: String,
}

impl Default for InheritedList {
    fn default() -> Self {
        Self {
            list_style_type: "disc".to_string(),
            list_style_position: "outside".to_string(),
            list_style_image: "none".to_string(),
        }
    }
}

/// Group of inherited table properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedTable {
    pub caption_side: String,
    pub border_collapse: String,
    pub border_spacing: u32,
}

impl Default for InheritedTable {
    fn default() -> Self {
        Self {
            caption_side: "top".to_string(),
            border_collapse: "separate".to_string(),
            border_spacing: 0,
        }
    }
}

/// Group of inherited UI properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedUI {
    pub cursor: String,
    pub quotes: String,
    pub accent_color: String,
    pub caret_color: String,
}

impl Default for InheritedUI {
    fn default() -> Self {
        Self {
            cursor: "auto".to_string(),
            quotes: "auto".to_string(),
            accent_color: "auto".to_string(),
            caret_color: "auto".to_string(),
        }
    }
}

/// Group of inherited effect properties.
#[derive(Debug, Clone, PartialEq)]
pub struct InheritedEffects {
    pub visibility: String,
    pub empty_cells: String,
}

impl Default for InheritedEffects {
    fn default() -> Self {
        Self {
            visibility: "visible".to_string(),
            empty_cells: "show".to_string(),
        }
    }
}

/// Group of reset box layout properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetBox {
    pub display: String,
    pub width: i32,
    pub height: i32,
    pub position: String,
    pub float: String,
    pub clear: String,
    pub overflow: String,
    pub overflow_x: String,
    pub overflow_y: String,
    pub z_index: i32,
    pub box_sizing: String,
    pub min_width: i32,
    pub min_height: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub vertical_align: i32,
    pub object_fit: String,
    pub object_position: String,
    pub scroll_behavior: String,
    pub user_select: String,
    pub pointer_events: String,
    pub aspect_ratio: String,
}

impl Default for ResetBox {
    fn default() -> Self {
        Self {
            display: "inline".to_string(),
            width: -1,
            height: -1,
            position: "static".to_string(),
            float: "none".to_string(),
            clear: "none".to_string(),
            overflow: "visible".to_string(),
            overflow_x: "visible".to_string(),
            overflow_y: "visible".to_string(),
            z_index: 0,
            box_sizing: "content-box".to_string(),
            min_width: -1,
            min_height: -1,
            max_width: -1,
            max_height: -1,
            vertical_align: -1,
            object_fit: "fill".to_string(),
            object_position: "50% 50%".to_string(),
            scroll_behavior: "auto".to_string(),
            user_select: "auto".to_string(),
            pointer_events: "auto".to_string(),
            aspect_ratio: "auto".to_string(),
        }
    }
}

/// Group of reset spacing and border properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetSurround {
    pub margin_top: i32,
    pub margin_right: i32,
    pub margin_bottom: i32,
    pub margin_left: i32,
    pub margin_block_start: i32,
    pub margin_block_end: i32,
    pub padding_top: i32,
    pub padding_right: i32,
    pub padding_bottom: i32,
    pub padding_left: i32,
    pub padding_block_start: i32,
    pub padding_block_end: i32,
    pub border_top_width: i32,
    pub border_right_width: i32,
    pub border_bottom_width: i32,
    pub border_left_width: i32,
    pub border_top_style: String,
    pub border_right_style: String,
    pub border_bottom_style: String,
    pub border_left_style: String,
    pub border_top_color: String,
    pub border_right_color: String,
    pub border_bottom_color: String,
    pub border_left_color: String,
    /// Base `border-color` (the non-per-edge shorthand value). Unlike the per-edge
    /// colors, this is NOT stripped when an `outset`/`inset` style triggers the UA
    /// bevel synthesis, so paint can still recover the resolved border color (e.g. the
    /// UA button's silver) that the legacy HashMap kept on the `border`/`border-color`
    /// shorthand entries.
    pub border_color: String,
    pub border_top_left_radius: i32,
    pub border_top_right_radius: i32,
    pub border_bottom_right_radius: i32,
    pub border_bottom_left_radius: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl Default for ResetSurround {
    fn default() -> Self {
        Self {
            margin_top: 0,
            margin_right: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_block_start: 0,
            margin_block_end: 0,
            padding_top: 0,
            padding_right: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_block_start: 0,
            padding_block_end: 0,
            border_top_width: -1,
            border_right_width: -1,
            border_bottom_width: -1,
            border_left_width: -1,
            border_top_style: "none".to_string(),
            border_right_style: "none".to_string(),
            border_bottom_style: "none".to_string(),
            border_left_style: "none".to_string(),
            border_top_color: "currentcolor".to_string(),
            border_right_color: "currentcolor".to_string(),
            border_bottom_color: "currentcolor".to_string(),
            border_left_color: "currentcolor".to_string(),
            border_color: "currentcolor".to_string(),
            border_top_left_radius: -1,
            border_top_right_radius: -1,
            border_bottom_right_radius: -1,
            border_bottom_left_radius: -1,
            top: -1,
            right: -1,
            bottom: -1,
            left: -1,
        }
    }
}

/// Group of reset background properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetBackground {
    pub background_color: String,
    pub background_image: String,
    pub background_repeat: String,
    pub background_position: String,
    pub background_size: String,
    pub background_attachment: String,
}

impl Default for ResetBackground {
    fn default() -> Self {
        Self {
            background_color: "transparent".to_string(),
            background_image: "none".to_string(),
            background_repeat: "repeat".to_string(),
            background_position: "0% 0%".to_string(),
            background_size: "auto".to_string(),
            background_attachment: "scroll".to_string(),
        }
    }
}

/// Group of reset flexbox properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetFlex {
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: i32,
    pub flex_direction: String,
    pub flex_wrap: String,
    pub justify_content: String,
    pub align_items: String,
    pub align_self: String,
    pub order: i32,
    pub align_content: String,
    pub row_gap: i32,
    pub column_gap: i32,
    pub column_count: i32,
    pub column_width: i32,
}

impl Default for ResetFlex {
    fn default() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: -1,
            flex_direction: "row".to_string(),
            flex_wrap: "nowrap".to_string(),
            justify_content: "normal".to_string(),
            align_items: "normal".to_string(),
            align_self: "auto".to_string(),
            order: 0,
            align_content: "normal".to_string(),
            row_gap: -1,
            column_gap: -1,
            column_count: -1,
            column_width: -1,
        }
    }
}

/// Group of reset table properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetTable {
    pub table_layout: String,
}

impl Default for ResetTable {
    fn default() -> Self {
        Self {
            table_layout: "auto".to_string(),
        }
    }
}

/// Group of reset visual effects and transitions properties.
#[derive(Debug, Clone, PartialEq)]
pub struct ResetEffects {
    pub opacity: f32,
    pub outline_width: i32,
    pub outline_style: String,
    pub outline_color: String,
    pub outline_offset: i32,
    pub transition_duration: u32,
    pub transition_property: String,
    pub transition_timing_function: String,
    pub transition_delay: String,
    pub text_decoration_line: String,
    pub text_decoration_color: String,
    pub text_decoration_style: String,
    pub text_overflow: String,
    pub box_shadow: Option<crate::css::values::CssValue>,
    pub transform: Vec<crate::css::values::TransformFn>,
    pub mask_type: String,
}

impl Default for ResetEffects {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            outline_width: -1,
            outline_style: "none".to_string(),
            outline_color: "invert".to_string(),
            outline_offset: 0,
            transition_duration: 0,
            transition_property: "all".to_string(),
            transition_timing_function: "ease".to_string(),
            transition_delay: "0s".to_string(),
            // Empty string = "unspecified" (CSS-initial). Kept distinct from an
            // explicitly-authored `none` so that decoration-propagation consumers
            // (paint::resolve_text_decorations) can tell a default node from one that
            // explicitly cancels ancestor decorations. `get()` maps it back to "none".
            text_decoration_line: String::new(),
            text_decoration_color: "currentcolor".to_string(),
            // Empty = unspecified (CSS-initial `solid`). Kept distinct from an explicit
            // `solid` so a default leaf node does not shadow an ancestor's authored style
            // during decoration propagation. `get()` maps it back to "solid".
            text_decoration_style: String::new(),
            text_overflow: "clip".to_string(),
            box_shadow: None,
            transform: Vec::new(),
            mask_type: "luminance".to_string(),
        }
    }
}

// Process-wide shared initial allocations for Style-Sharing.
static INITIAL_INHERITED_TEXT: OnceLock<Arc<InheritedText>> = OnceLock::new();
static INITIAL_INHERITED_LIST: OnceLock<Arc<InheritedList>> = OnceLock::new();
static INITIAL_INHERITED_TABLE: OnceLock<Arc<InheritedTable>> = OnceLock::new();
static INITIAL_INHERITED_UI: OnceLock<Arc<InheritedUI>> = OnceLock::new();
static INITIAL_INHERITED_EFFECTS: OnceLock<Arc<InheritedEffects>> = OnceLock::new();

static INITIAL_RESET_BOX: OnceLock<Arc<ResetBox>> = OnceLock::new();
static INITIAL_RESET_SURROUND: OnceLock<Arc<ResetSurround>> = OnceLock::new();
static INITIAL_RESET_BACKGROUND: OnceLock<Arc<ResetBackground>> = OnceLock::new();
static INITIAL_RESET_FLEX: OnceLock<Arc<ResetFlex>> = OnceLock::new();
static INITIAL_RESET_TABLE: OnceLock<Arc<ResetTable>> = OnceLock::new();
static INITIAL_RESET_EFFECTS: OnceLock<Arc<ResetEffects>> = OnceLock::new();

/// Production target style type per ADR 0001, carrying all 11 CSS categories.
#[derive(Debug, Clone, PartialEq)]
pub struct CategorizedComputedStyle {
    pub inherited_text: Arc<InheritedText>,
    pub inherited_list: Arc<InheritedList>,
    pub inherited_table: Arc<InheritedTable>,
    pub inherited_ui: Arc<InheritedUI>,
    pub inherited_effects: Arc<InheritedEffects>,
    pub reset_box: Arc<ResetBox>,
    pub reset_surround: Arc<ResetSurround>,
    pub reset_background: Arc<ResetBackground>,
    pub reset_flex: Arc<ResetFlex>,
    pub reset_table: Arc<ResetTable>,
    pub reset_effects: Arc<ResetEffects>,
    pub extra_values: Option<Arc<HashMap<String, crate::css::values::CssValue>>>,
}

impl Default for CategorizedComputedStyle {
    fn default() -> Self {
        Self::initial()
    }
}

fn parse_css_color_simple(s: &str) -> Option<crate::css::values::Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#')
        && hex.len() == 6
    {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        return Some(crate::css::values::Color::Rgba(r, g, b, 255));
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inside = &s[4..s.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|x| x.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(crate::css::values::Color::Rgba(r, g, b, 255));
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inside = &s[5..s.len() - 1];
        let parts: Vec<&str> = inside.split(',').map(|x| x.trim()).collect();
        if parts.len() == 4 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a_f = parts[3].parse::<f32>().ok()?;
            return Some(crate::css::values::Color::Rgba(
                r,
                g,
                b,
                (a_f * 255.0) as u8,
            ));
        }
    }
    match s.to_ascii_lowercase().as_str() {
        "red" => Some(crate::css::values::Color::Rgba(255, 0, 0, 255)),
        "green" => Some(crate::css::values::Color::Rgba(0, 255, 0, 255)),
        "blue" => Some(crate::css::values::Color::Rgba(0, 0, 255, 255)),
        "black" => Some(crate::css::values::Color::Rgba(0, 0, 0, 255)),
        "white" => Some(crate::css::values::Color::Rgba(255, 255, 255, 255)),
        "yellow" => Some(crate::css::values::Color::Rgba(255, 255, 0, 255)),
        "magenta" => Some(crate::css::values::Color::Rgba(255, 0, 255, 255)),
        "cyan" => Some(crate::css::values::Color::Rgba(0, 255, 255, 255)),
        "transparent" => Some(crate::css::values::Color::Rgba(0, 0, 0, 0)),
        _ => None,
    }
}

fn is_inherited_property_name(name: &str) -> bool {
    matches!(
        name,
        "color"
            | "font-family"
            | "font-size"
            | "font-style"
            | "font-weight"
            | "line-height"
            | "text-align"
            | "letter-spacing"
            | "word-spacing"
            | "white-space"
            | "direction"
            | "text-transform"
            | "text-indent"
            | "word-break"
            | "line-break"
            | "text-orientation"
            | "overflow-wrap"
            | "word-wrap"
            | "list-style-type"
            | "list-style-position"
            | "list-style-image"
            | "border-collapse"
            | "border-spacing"
            | "caption-side"
            | "cursor"
            | "accent-color"
            | "caret-color"
            | "visibility"
            // Not actually inherited; listed only so `get()` synthesizes the
            // initial value ("ease"/"0s") from typed storage when unset/invalid.
            | "transition-timing-function"
            | "transition-delay"
    )
}

impl CategorizedComputedStyle {
    /// Returns a new `CategorizedComputedStyle` sharing process-wide initial allocations.
    pub fn initial() -> Self {
        let inherited_text = INITIAL_INHERITED_TEXT
            .get_or_init(|| Arc::new(InheritedText::default()))
            .clone();
        let inherited_list = INITIAL_INHERITED_LIST
            .get_or_init(|| Arc::new(InheritedList::default()))
            .clone();
        let inherited_table = INITIAL_INHERITED_TABLE
            .get_or_init(|| Arc::new(InheritedTable::default()))
            .clone();
        let inherited_ui = INITIAL_INHERITED_UI
            .get_or_init(|| Arc::new(InheritedUI::default()))
            .clone();
        let inherited_effects = INITIAL_INHERITED_EFFECTS
            .get_or_init(|| Arc::new(InheritedEffects::default()))
            .clone();

        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        let reset_surround = INITIAL_RESET_SURROUND
            .get_or_init(|| Arc::new(ResetSurround::default()))
            .clone();
        let reset_background = INITIAL_RESET_BACKGROUND
            .get_or_init(|| Arc::new(ResetBackground::default()))
            .clone();
        let reset_flex = INITIAL_RESET_FLEX
            .get_or_init(|| Arc::new(ResetFlex::default()))
            .clone();
        let reset_table = INITIAL_RESET_TABLE
            .get_or_init(|| Arc::new(ResetTable::default()))
            .clone();
        let reset_effects = INITIAL_RESET_EFFECTS
            .get_or_init(|| Arc::new(ResetEffects::default()))
            .clone();

        Self {
            inherited_text,
            inherited_list,
            inherited_table,
            inherited_ui,
            inherited_effects,
            reset_box,
            reset_surround,
            reset_background,
            reset_flex,
            reset_table,
            reset_effects,
            extra_values: None,
        }
    }

    /// Inherits properties from a parent style node.
    /// Inherited categories are cloned (pointer copy, zero-alloc).
    /// Reset categories get the fresh process-wide initial allocation.
    pub fn inherit_from(parent: &Self) -> Self {
        let reset_box = INITIAL_RESET_BOX
            .get_or_init(|| Arc::new(ResetBox::default()))
            .clone();
        let reset_surround = INITIAL_RESET_SURROUND
            .get_or_init(|| Arc::new(ResetSurround::default()))
            .clone();
        let reset_background = INITIAL_RESET_BACKGROUND
            .get_or_init(|| Arc::new(ResetBackground::default()))
            .clone();
        let reset_flex = INITIAL_RESET_FLEX
            .get_or_init(|| Arc::new(ResetFlex::default()))
            .clone();
        let reset_table = INITIAL_RESET_TABLE
            .get_or_init(|| Arc::new(ResetTable::default()))
            .clone();
        let reset_effects = INITIAL_RESET_EFFECTS
            .get_or_init(|| Arc::new(ResetEffects::default()))
            .clone();

        Self {
            inherited_text: parent.inherited_text.clone(),
            inherited_list: parent.inherited_list.clone(),
            inherited_table: parent.inherited_table.clone(),
            inherited_ui: parent.inherited_ui.clone(),
            inherited_effects: parent.inherited_effects.clone(),
            reset_box,
            reset_surround,
            reset_background,
            reset_flex,
            reset_table,
            reset_effects,
            extra_values: None,
        }
    }

    /// Set text color.
    pub fn set_color(&mut self, color: String) {
        Arc::make_mut(&mut self.inherited_text).color = color;
    }

    /// Set font size.
    pub fn set_font_size(&mut self, font_size: u32) {
        Arc::make_mut(&mut self.inherited_text).font_size = font_size;
    }

    /// Set display.
    pub fn set_display(&mut self, display: String) {
        Arc::make_mut(&mut self.reset_box).display = display;
    }

    /// Set width.
    pub fn set_width(&mut self, width: i32) {
        Arc::make_mut(&mut self.reset_box).width = width;
    }

    /// Set height.
    pub fn set_height(&mut self, height: i32) {
        Arc::make_mut(&mut self.reset_box).height = height;
    }

    /// Set z-index.
    pub fn set_z_index(&mut self, z_index: i32) {
        Arc::make_mut(&mut self.reset_box).z_index = z_index;
    }

    /// Set box shadow.
    pub fn set_box_shadow(&mut self, box_shadow: crate::css::values::CssValue) {
        Arc::make_mut(&mut self.reset_effects).box_shadow = Some(box_shadow);
    }

    /// Set text shadow.
    pub fn set_text_shadow(&mut self, text_shadow: crate::css::values::CssValue) {
        Arc::make_mut(&mut self.inherited_text).text_shadow = Some(text_shadow);
    }

    /// Set property.
    pub fn set_property(&mut self, name: &str, value: &crate::css::values::CssValue) {
        use crate::css::values::{CssValue, ZIndex};
        if name == "scroll-behavior" {
            let is_valid = match value {
                CssValue::Keyword(kw) => {
                    matches!(kw.to_ascii_lowercase().as_str(), "auto" | "smooth")
                }
                _ => false,
            };
            if !is_valid {
                return;
            }
        }

        if name == "user-select" {
            let is_valid = match value {
                CssValue::Keyword(kw) => {
                    matches!(
                        kw.to_ascii_lowercase().as_str(),
                        "auto" | "text" | "none" | "contain" | "all"
                    )
                }
                _ => false,
            };
            if !is_valid {
                return;
            }
        }

        if name == "accent-color" || name == "caret-color" {
            let is_valid = match value {
                CssValue::Color(_) => true,
                CssValue::Keyword(kw) => {
                    matches!(kw.to_ascii_lowercase().as_str(), "auto" | "currentcolor")
                }
                _ => false,
            };
            if !is_valid {
                return;
            }
        }

        if (name == "transition-timing-function" || name == "transition-delay")
            && !crate::css::values::is_valid_property_value(name, value)
        {
            return;
        }

        if self.extra_values.is_none() {
            self.extra_values = Some(Arc::new(HashMap::new()));
        }
        if let Some(ref mut map) = self.extra_values {
            Arc::make_mut(map).insert(name.to_string(), value.clone());
        }

        let fs = self.inherited_text.font_size;

        match name {
            // InheritedText
            "color" => self.set_color(css_value_to_string(value)),
            "font-family" => {
                Arc::make_mut(&mut self.inherited_text).font_family = css_value_to_string(value)
            }
            "font-size" => {
                let px = match value {
                    CssValue::Length(v, _) => v.round().max(1.0) as u32,
                    _ => 16,
                };
                self.set_font_size(px);
            }
            "font-style" => {
                Arc::make_mut(&mut self.inherited_text).font_style = css_value_to_string(value)
            }
            "font-weight" => {
                Arc::make_mut(&mut self.inherited_text).font_weight = css_value_to_string(value)
            }
            "line-height" => {
                let fs = self.inherited_text.font_size;
                let (px, number) = match value {
                    CssValue::Length(v, _) => (v.round().max(0.0) as u32, None),
                    // A unitless number resolves to px against this element's font-size,
                    // but the number itself is what inherits (descendants recompute).
                    CssValue::Number(v) => ((v * fs as f32).round().max(0.0) as u32, Some(*v)),
                    // `normal` (and any non-length keyword) stays unspecified so layout
                    // falls back to the font's intrinsic line height.
                    _ => (LINE_HEIGHT_NORMAL, None),
                };
                let it = Arc::make_mut(&mut self.inherited_text);
                it.line_height = px;
                it.line_height_number = number;
            }
            "text-align" => {
                Arc::make_mut(&mut self.inherited_text).text_align = css_value_to_string(value)
            }
            "letter-spacing" => {
                Arc::make_mut(&mut self.inherited_text).letter_spacing = value_to_px(value, fs)
            }
            "word-spacing" => {
                Arc::make_mut(&mut self.inherited_text).word_spacing = value_to_px(value, fs)
            }
            "white-space" => {
                Arc::make_mut(&mut self.inherited_text).white_space = css_value_to_string(value)
            }
            "direction" => {
                Arc::make_mut(&mut self.inherited_text).direction = css_value_to_string(value)
            }
            "text-transform" => {
                Arc::make_mut(&mut self.inherited_text).text_transform = css_value_to_string(value)
            }
            "font-variant" => {
                Arc::make_mut(&mut self.inherited_text).font_variant = css_value_to_string(value)
            }
            "font-stretch" => {
                Arc::make_mut(&mut self.inherited_text).font_stretch = css_value_to_string(value)
            }
            "text-indent" => {
                Arc::make_mut(&mut self.inherited_text).text_indent = value_to_px(value, fs)
            }
            "word-break" => {
                Arc::make_mut(&mut self.inherited_text).word_break = css_value_to_string(value)
            }
            "line-break" => {
                Arc::make_mut(&mut self.inherited_text).line_break = css_value_to_string(value)
            }
            "text-orientation" => {
                Arc::make_mut(&mut self.inherited_text).text_orientation =
                    css_value_to_string(value)
            }
            // `word-wrap` is the legacy alias of `overflow-wrap`; normalize both
            // into the same typed field (the raw alias was lost in the migration).
            "overflow-wrap" | "word-wrap" => {
                Arc::make_mut(&mut self.inherited_text).overflow_wrap = css_value_to_string(value)
            }
            "text-align-last" => {
                Arc::make_mut(&mut self.inherited_text).text_align_last = css_value_to_string(value)
            }
            "tab-size" => Arc::make_mut(&mut self.inherited_text).tab_size = value_to_u32(value),
            "hyphens" => {
                Arc::make_mut(&mut self.inherited_text).hyphens = css_value_to_string(value)
            }
            "text-rendering" => {
                Arc::make_mut(&mut self.inherited_text).text_rendering = css_value_to_string(value)
            }
            "image-rendering" => {
                Arc::make_mut(&mut self.inherited_text).image_rendering = css_value_to_string(value)
            }
            "font-variant-caps" => {
                Arc::make_mut(&mut self.inherited_text).font_variant_caps =
                    css_value_to_string(value)
            }
            "text-shadow" => {
                Arc::make_mut(&mut self.inherited_text).text_shadow = Some(value.clone())
            }

            // InheritedList
            "list-style-type" => {
                Arc::make_mut(&mut self.inherited_list).list_style_type = css_value_to_string(value)
            }
            "list-style-position" => {
                Arc::make_mut(&mut self.inherited_list).list_style_position =
                    css_value_to_string(value)
            }
            "list-style-image" => {
                Arc::make_mut(&mut self.inherited_list).list_style_image =
                    css_value_to_string(value)
            }

            // InheritedTable
            "caption-side" => {
                Arc::make_mut(&mut self.inherited_table).caption_side = css_value_to_string(value)
            }
            "border-collapse" => {
                Arc::make_mut(&mut self.inherited_table).border_collapse =
                    css_value_to_string(value)
            }
            "border-spacing" => {
                Arc::make_mut(&mut self.inherited_table).border_spacing = value_to_u32(value)
            }

            // InheritedUI
            "cursor" => Arc::make_mut(&mut self.inherited_ui).cursor = css_value_to_string(value),
            "quotes" => Arc::make_mut(&mut self.inherited_ui).quotes = css_value_to_string(value),
            "accent-color" => {
                Arc::make_mut(&mut self.inherited_ui).accent_color = css_value_to_string(value)
            }
            "caret-color" => {
                Arc::make_mut(&mut self.inherited_ui).caret_color = css_value_to_string(value)
            }

            // InheritedEffects
            "visibility" => {
                Arc::make_mut(&mut self.inherited_effects).visibility = css_value_to_string(value)
            }
            "empty-cells" => {
                Arc::make_mut(&mut self.inherited_effects).empty_cells = css_value_to_string(value)
            }

            // ResetBox
            "display" => self.set_display(css_value_to_string(value)),
            "width" => self.set_width(value_to_px_or_auto(value, fs)),
            "height" => self.set_height(height_value_to_px_or_auto(value, fs)),
            "position" => Arc::make_mut(&mut self.reset_box).position = css_value_to_string(value),
            "float" => Arc::make_mut(&mut self.reset_box).float = css_value_to_string(value),
            "clear" => Arc::make_mut(&mut self.reset_box).clear = css_value_to_string(value),
            "overflow-x" => {
                Arc::make_mut(&mut self.reset_box).overflow_x = css_value_to_string(value);
            }
            "overflow-y" => {
                Arc::make_mut(&mut self.reset_box).overflow_y = css_value_to_string(value);
            }
            "overflow" => {
                let s = css_value_to_string(value);
                let box_mut = Arc::make_mut(&mut self.reset_box);
                box_mut.overflow = s.clone();
                box_mut.overflow_x = s.clone();
                box_mut.overflow_y = s;
            }
            "z-index" => {
                let z = match value {
                    CssValue::ZIndex(ZIndex::Auto) => i32::MIN,
                    CssValue::ZIndex(ZIndex::Index(v)) => *v,
                    CssValue::Keyword(s) if s.eq_ignore_ascii_case("auto") => i32::MIN,
                    CssValue::Number(v) => v.round() as i32,
                    _ => i32::MIN,
                };
                self.set_z_index(z);
            }
            "box-sizing" => {
                Arc::make_mut(&mut self.reset_box).box_sizing = css_value_to_string(value)
            }
            "min-width" => {
                Arc::make_mut(&mut self.reset_box).min_width = width_px_or_percent_band(value, fs)
            }
            "min-height" => Arc::make_mut(&mut self.reset_box).min_height = value_to_px(value, fs),
            "max-width" => {
                Arc::make_mut(&mut self.reset_box).max_width = width_px_or_percent_band(value, fs)
            }
            "max-height" => Arc::make_mut(&mut self.reset_box).max_height = value_to_px(value, fs),
            "vertical-align" => {
                let v = match value {
                    CssValue::Keyword(kw) => match kw.as_str() {
                        "baseline" => -1,
                        "sub" => -2,
                        "super" => -3,
                        "text-top" | "top" => -4,
                        "text-bottom" | "bottom" => -5,
                        "middle" => -6,
                        _ => -1,
                    },
                    // Percentage is relative to the line-height, which is unknown
                    // at style time; store the raw percent in a distinct band and
                    // resolve it during layout.
                    CssValue::Length(val, crate::css::values::LengthUnit::Percent) => {
                        (val.round() as i32) + 200000
                    }
                    _ => value_to_px(value, fs) + 100000,
                };
                Arc::make_mut(&mut self.reset_box).vertical_align = v;
            }
            "object-fit" => {
                Arc::make_mut(&mut self.reset_box).object_fit = css_value_to_string(value)
            }
            "object-position" => {
                Arc::make_mut(&mut self.reset_box).object_position = css_value_to_string(value)
            }
            "scroll-behavior" => {
                Arc::make_mut(&mut self.reset_box).scroll_behavior = css_value_to_string(value)
            }
            "user-select" => {
                Arc::make_mut(&mut self.reset_box).user_select = css_value_to_string(value)
            }
            "pointer-events" => {
                Arc::make_mut(&mut self.reset_box).pointer_events = css_value_to_string(value)
            }
            "aspect-ratio" => {
                Arc::make_mut(&mut self.reset_box).aspect_ratio = css_value_to_string(value)
            }

            // ResetSurround
            "margin-top" => {
                Arc::make_mut(&mut self.reset_surround).margin_top = value_to_px(value, fs)
            }
            "margin-right" => {
                Arc::make_mut(&mut self.reset_surround).margin_right = value_to_px(value, fs)
            }
            "margin-bottom" => {
                Arc::make_mut(&mut self.reset_surround).margin_bottom = value_to_px(value, fs)
            }
            "margin-left" => {
                Arc::make_mut(&mut self.reset_surround).margin_left = value_to_px(value, fs)
            }
            "margin-block-start" => {
                Arc::make_mut(&mut self.reset_surround).margin_block_start = value_to_px(value, fs)
            }
            "margin-block-end" => {
                Arc::make_mut(&mut self.reset_surround).margin_block_end = value_to_px(value, fs)
            }
            "padding-top" => {
                Arc::make_mut(&mut self.reset_surround).padding_top = value_to_px(value, fs)
            }
            "padding-right" => {
                Arc::make_mut(&mut self.reset_surround).padding_right = value_to_px(value, fs)
            }
            "padding-bottom" => {
                Arc::make_mut(&mut self.reset_surround).padding_bottom = value_to_px(value, fs)
            }
            "padding-left" => {
                Arc::make_mut(&mut self.reset_surround).padding_left = value_to_px(value, fs)
            }
            "padding-block-start" => {
                Arc::make_mut(&mut self.reset_surround).padding_block_start = value_to_px(value, fs)
            }
            "padding-block-end" => {
                Arc::make_mut(&mut self.reset_surround).padding_block_end = value_to_px(value, fs)
            }
            "border-top-width" => {
                Arc::make_mut(&mut self.reset_surround).border_top_width = value_to_px(value, fs)
            }
            "border-right-width" => {
                Arc::make_mut(&mut self.reset_surround).border_right_width = value_to_px(value, fs)
            }
            "border-bottom-width" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_width = value_to_px(value, fs)
            }
            "border-left-width" => {
                Arc::make_mut(&mut self.reset_surround).border_left_width = value_to_px(value, fs)
            }
            "border-top-style" => {
                Arc::make_mut(&mut self.reset_surround).border_top_style =
                    css_value_to_string(value)
            }
            "border-right-style" => {
                Arc::make_mut(&mut self.reset_surround).border_right_style =
                    css_value_to_string(value)
            }
            "border-bottom-style" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_style =
                    css_value_to_string(value)
            }
            "border-left-style" => {
                Arc::make_mut(&mut self.reset_surround).border_left_style =
                    css_value_to_string(value)
            }
            "border-top-color" => {
                Arc::make_mut(&mut self.reset_surround).border_top_color =
                    css_value_to_string(value)
            }
            "border-right-color" => {
                Arc::make_mut(&mut self.reset_surround).border_right_color =
                    css_value_to_string(value)
            }
            "border-bottom-color" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_color =
                    css_value_to_string(value)
            }
            "border-left-color" => {
                Arc::make_mut(&mut self.reset_surround).border_left_color =
                    css_value_to_string(value)
            }
            // Base (non-per-edge) border-color. The cascade also expands this to the
            // per-edge longhands (which may be stripped for an outset/inset bevel), but
            // the base value is retained so paint can recover the resolved color.
            "border-color" => {
                Arc::make_mut(&mut self.reset_surround).border_color = css_value_to_string(value)
            }
            "border-top-left-radius" => {
                Arc::make_mut(&mut self.reset_surround).border_top_left_radius =
                    value_to_px(value, fs)
            }
            "border-top-right-radius" => {
                Arc::make_mut(&mut self.reset_surround).border_top_right_radius =
                    value_to_px(value, fs)
            }
            "border-bottom-right-radius" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_right_radius =
                    value_to_px(value, fs)
            }
            "border-bottom-left-radius" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_left_radius =
                    value_to_px(value, fs)
            }
            "top" => Arc::make_mut(&mut self.reset_surround).top = value_to_px(value, fs),
            "right" => Arc::make_mut(&mut self.reset_surround).right = value_to_px(value, fs),
            "bottom" => Arc::make_mut(&mut self.reset_surround).bottom = value_to_px(value, fs),
            "left" => Arc::make_mut(&mut self.reset_surround).left = value_to_px(value, fs),

            // ResetBackground
            "background-color" => {
                Arc::make_mut(&mut self.reset_background).background_color =
                    css_value_to_string(value)
            }
            "background-image" => {
                Arc::make_mut(&mut self.reset_background).background_image =
                    css_value_to_string(value)
            }
            "background-repeat" => {
                Arc::make_mut(&mut self.reset_background).background_repeat =
                    css_value_to_string(value)
            }
            "background-position" => {
                Arc::make_mut(&mut self.reset_background).background_position =
                    css_value_to_string(value)
            }
            "background-size" => {
                Arc::make_mut(&mut self.reset_background).background_size =
                    css_value_to_string(value)
            }
            "background-attachment" => {
                Arc::make_mut(&mut self.reset_background).background_attachment =
                    css_value_to_string(value)
            }

            // ResetFlex
            "flex-grow" => Arc::make_mut(&mut self.reset_flex).flex_grow = value_to_f32(value),
            "flex-shrink" => Arc::make_mut(&mut self.reset_flex).flex_shrink = value_to_f32(value),
            "flex-basis" => Arc::make_mut(&mut self.reset_flex).flex_basis = value_to_px(value, fs),
            "flex-direction" => {
                Arc::make_mut(&mut self.reset_flex).flex_direction = css_value_to_string(value)
            }
            "flex-wrap" => {
                Arc::make_mut(&mut self.reset_flex).flex_wrap = css_value_to_string(value)
            }
            "justify-content" => {
                Arc::make_mut(&mut self.reset_flex).justify_content = css_value_to_string(value)
            }
            "align-items" => {
                Arc::make_mut(&mut self.reset_flex).align_items = css_value_to_string(value)
            }
            "align-self" => {
                Arc::make_mut(&mut self.reset_flex).align_self = css_value_to_string(value)
            }
            "order" => Arc::make_mut(&mut self.reset_flex).order = value_to_px(value, fs),
            "align-content" => {
                Arc::make_mut(&mut self.reset_flex).align_content = css_value_to_string(value)
            }
            "row-gap" => Arc::make_mut(&mut self.reset_flex).row_gap = value_to_px(value, fs),
            "column-gap" => Arc::make_mut(&mut self.reset_flex).column_gap = value_to_px(value, fs),
            "gap" => {
                let mut leaves = Vec::new();
                flatten_value(value, &mut leaves);
                let (row_px, col_px) = if leaves.len() >= 2 {
                    (value_to_px(&leaves[0], fs), value_to_px(&leaves[1], fs))
                } else {
                    let p = value_to_px(value, fs);
                    (p, p)
                };
                let flex = Arc::make_mut(&mut self.reset_flex);
                if flex.row_gap == -1 {
                    flex.row_gap = row_px;
                }
                if flex.column_gap == -1 {
                    flex.column_gap = col_px;
                }
            }
            "column-count" => {
                let count = match value {
                    CssValue::Number(v) => {
                        let n = v.round() as i32;
                        if n >= 1 { n } else { -1 }
                    }
                    _ => -1,
                };
                Arc::make_mut(&mut self.reset_flex).column_count = count;
            }
            "column-width" => {
                let w = value_to_px_or_auto(value, fs);
                let width = if w >= 0 { w } else { -1 };
                Arc::make_mut(&mut self.reset_flex).column_width = width;
            }
            "columns" => {
                let mut leaves = Vec::new();
                flatten_value(value, &mut leaves);
                let mut parsed_count = -1;
                let mut parsed_width = -1;
                for leaf in &leaves {
                    match leaf {
                        CssValue::Length(..) => {
                            let w = value_to_px_or_auto(leaf, fs);
                            if w >= 0 {
                                parsed_width = w;
                            }
                        }
                        CssValue::Number(v) => {
                            if *v == 0.0 {
                                parsed_width = 0;
                            } else {
                                let n = v.round() as i32;
                                if n >= 1 {
                                    parsed_count = n;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                let flex = Arc::make_mut(&mut self.reset_flex);
                flex.column_count = parsed_count;
                flex.column_width = parsed_width;
            }

            // ResetTable
            "table-layout" => {
                Arc::make_mut(&mut self.reset_table).table_layout = css_value_to_string(value)
            }

            // ResetEffects
            "opacity" => Arc::make_mut(&mut self.reset_effects).opacity = value_to_f32(value),
            "outline-width" => {
                Arc::make_mut(&mut self.reset_effects).outline_width = value_to_px(value, fs)
            }
            "outline-style" => {
                Arc::make_mut(&mut self.reset_effects).outline_style = css_value_to_string(value)
            }
            "outline-color" => {
                Arc::make_mut(&mut self.reset_effects).outline_color = css_value_to_string(value)
            }
            "outline-offset" => {
                Arc::make_mut(&mut self.reset_effects).outline_offset = value_to_px(value, fs)
            }
            "transition-duration" => {
                Arc::make_mut(&mut self.reset_effects).transition_duration = value_to_u32(value)
            }
            "transition-property" => {
                Arc::make_mut(&mut self.reset_effects).transition_property =
                    css_value_to_string(value)
            }
            "transition-timing-function" => {
                Arc::make_mut(&mut self.reset_effects).transition_timing_function =
                    css_value_to_string(value)
            }
            "transition-delay" => {
                let s = css_value_to_string(value);
                Arc::make_mut(&mut self.reset_effects).transition_delay =
                    if s == "0" { "0s".to_string() } else { s };
            }
            "text-decoration-line" => {
                Arc::make_mut(&mut self.reset_effects).text_decoration_line =
                    css_value_to_string(value)
            }
            "text-decoration-color" => {
                Arc::make_mut(&mut self.reset_effects).text_decoration_color =
                    css_value_to_string(value)
            }
            "text-decoration-style" => {
                Arc::make_mut(&mut self.reset_effects).text_decoration_style =
                    css_value_to_string(value)
            }
            "mask-type" => {
                Arc::make_mut(&mut self.reset_effects).mask_type = css_value_to_string(value);
            }
            "text-decoration" => {
                let val_str = css_value_to_string(value);
                Arc::make_mut(&mut self.reset_effects).text_decoration_line = val_str;
            }
            "border" => {
                let mut leaves = Vec::new();
                flatten_value(value, &mut leaves);
                for leaf in leaves {
                    match leaf {
                        CssValue::Length(_, _) | CssValue::Number(_) => {
                            let px = value_to_px(&leaf, fs);
                            let surround = Arc::make_mut(&mut self.reset_surround);
                            surround.border_top_width = px;
                            surround.border_right_width = px;
                            surround.border_bottom_width = px;
                            surround.border_left_width = px;
                        }
                        CssValue::Color(_) => {
                            let col = css_value_to_string(&leaf);
                            let surround = Arc::make_mut(&mut self.reset_surround);
                            surround.border_top_color = col.clone();
                            surround.border_right_color = col.clone();
                            surround.border_bottom_color = col.clone();
                            surround.border_left_color = col.clone();
                            surround.border_color = col;
                        }
                        CssValue::Keyword(ref kw) => {
                            if kw == "thin" || kw == "medium" || kw == "thick" {
                                let px = value_to_px(&leaf, fs);
                                let surround = Arc::make_mut(&mut self.reset_surround);
                                surround.border_top_width = px;
                                surround.border_right_width = px;
                                surround.border_bottom_width = px;
                                surround.border_left_width = px;
                            } else if kw == "none"
                                || kw == "solid"
                                || kw == "double"
                                || kw == "dotted"
                                || kw == "dashed"
                                || kw == "groove"
                                || kw == "ridge"
                                || kw == "inset"
                                || kw == "outset"
                            {
                                let surround = Arc::make_mut(&mut self.reset_surround);
                                surround.border_top_style = kw.clone();
                                surround.border_right_style = kw.clone();
                                surround.border_bottom_style = kw.clone();
                                surround.border_left_style = kw.clone();
                            } else {
                                let col = css_value_to_string(&leaf);
                                let surround = Arc::make_mut(&mut self.reset_surround);
                                surround.border_top_color = col.clone();
                                surround.border_right_color = col.clone();
                                surround.border_bottom_color = col.clone();
                                surround.border_left_color = col.clone();
                                surround.border_color = col;
                            }
                        }
                        _ => {}
                    }
                }
            }
            "outline" => {
                let mut leaves = Vec::new();
                flatten_value(value, &mut leaves);
                for leaf in leaves {
                    match leaf {
                        CssValue::Length(_, _) | CssValue::Number(_) => {
                            Arc::make_mut(&mut self.reset_effects).outline_width =
                                value_to_px(&leaf, fs);
                        }
                        CssValue::Color(ref _c) => {
                            Arc::make_mut(&mut self.reset_effects).outline_color =
                                css_value_to_string(&leaf);
                        }
                        CssValue::Keyword(ref kw) => {
                            if kw == "thin" || kw == "medium" || kw == "thick" {
                                Arc::make_mut(&mut self.reset_effects).outline_width =
                                    value_to_px(&leaf, fs);
                            } else if kw == "none"
                                || kw == "solid"
                                || kw == "double"
                                || kw == "dotted"
                                || kw == "dashed"
                            {
                                Arc::make_mut(&mut self.reset_effects).outline_style = kw.clone();
                            } else {
                                Arc::make_mut(&mut self.reset_effects).outline_color = kw.clone();
                            }
                        }
                        _ => {}
                    }
                }
            }
            "text-overflow" => {
                Arc::make_mut(&mut self.reset_effects).text_overflow = css_value_to_string(value)
            }
            "box-shadow" => Arc::make_mut(&mut self.reset_effects).box_shadow = Some(value.clone()),
            "transform" => {
                if let crate::css::values::CssValue::Transform(fns) = value {
                    Arc::make_mut(&mut self.reset_effects).transform = fns.clone();
                } else if css_value_to_string(value).eq_ignore_ascii_case("none") {
                    Arc::make_mut(&mut self.reset_effects).transform = Vec::new();
                }
            }

            _ => {}
        }
    }

    /// Remove property.
    pub fn remove_property(&mut self, name: &str) {
        match name {
            "border-top-color" => {
                Arc::make_mut(&mut self.reset_surround).border_top_color =
                    "currentcolor".to_string()
            }
            "border-right-color" => {
                Arc::make_mut(&mut self.reset_surround).border_right_color =
                    "currentcolor".to_string()
            }
            "border-bottom-color" => {
                Arc::make_mut(&mut self.reset_surround).border_bottom_color =
                    "currentcolor".to_string()
            }
            "border-left-color" => {
                Arc::make_mut(&mut self.reset_surround).border_left_color =
                    "currentcolor".to_string()
            }
            _ => {}
        }
    }

    /// Get property as CssValue for compatibility and style tests.
    pub fn get(&self, name: &str) -> Option<&crate::css::values::CssValue> {
        if let Some(ref map) = self.extra_values
            && let Some(val) = map.get(name)
        {
            return Some(val);
        }
        if is_inherited_property_name(name) {
            let s = self.get_property_as_string(name)?;
            let s = s.trim();
            if !s.is_empty() {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut in_parens = 0;
                for c in s.chars() {
                    if c == '(' {
                        in_parens += 1;
                        current.push(c);
                    } else if c == ')' {
                        if in_parens > 0 {
                            in_parens -= 1;
                        }
                        current.push(c);
                    } else if c.is_whitespace() && in_parens == 0 {
                        if !current.is_empty() {
                            parts.push(current.clone());
                            current.clear();
                        }
                    } else {
                        current.push(c);
                    }
                }
                if !current.is_empty() {
                    parts.push(current);
                }

                fn parse_single(p: &str) -> crate::css::values::CssValue {
                    if let Some(color) = parse_css_color_simple(p) {
                        return crate::css::values::CssValue::Color(color);
                    }
                    if let Some(num_str) = p.strip_suffix("px")
                        && let Ok(v) = num_str.parse::<f32>()
                    {
                        return crate::css::values::CssValue::Length(
                            v,
                            crate::css::values::LengthUnit::Px,
                        );
                    }
                    if let Some(num_str) = p.strip_suffix("em")
                        && let Ok(v) = num_str.parse::<f32>()
                    {
                        return crate::css::values::CssValue::Length(
                            v,
                            crate::css::values::LengthUnit::Em,
                        );
                    }
                    if let Some(num_str) = p.strip_suffix('%')
                        && let Ok(v) = num_str.parse::<f32>()
                    {
                        return crate::css::values::CssValue::Length(
                            v,
                            crate::css::values::LengthUnit::Percent,
                        );
                    }
                    if let Ok(v) = p.parse::<f32>() {
                        return crate::css::values::CssValue::Number(v);
                    }
                    crate::css::values::CssValue::Keyword(p.to_string())
                }

                let val = if parts.is_empty() {
                    crate::css::values::CssValue::Keyword(s.to_string())
                } else if parts.len() == 1 {
                    parse_single(&parts[0])
                } else {
                    let list = parts.into_iter().map(|x| parse_single(&x)).collect();
                    crate::css::values::CssValue::Multiple(list)
                };

                return Some(Box::leak(Box::new(val)));
            }
        }
        None
    }

    /// Insert property for compatibility and layout tests.
    pub fn insert(&mut self, prop: String, value: crate::css::values::CssValue) {
        self.set_property(&prop, &value);
    }

    /// Get property as string.
    pub fn get_property_as_string(&self, name: &str) -> Option<String> {
        match name {
            // InheritedText
            "color" => {
                if self.inherited_text.color.is_empty() {
                    None
                } else {
                    Some(self.inherited_text.color.clone())
                }
            }
            "font-family" => Some(self.inherited_text.font_family.clone()),
            "font-size" => Some(format!("{}px", self.inherited_text.font_size)),
            "font-style" => Some(self.inherited_text.font_style.clone()),
            "font-weight" => Some(self.inherited_text.font_weight.clone()),
            "line-height" => Some(if self.inherited_text.line_height == LINE_HEIGHT_NORMAL {
                "normal".to_string()
            } else {
                format!("{}px", self.inherited_text.line_height)
            }),
            "text-align" => Some(self.inherited_text.text_align.clone()),
            "letter-spacing" => Some(if self.inherited_text.letter_spacing == -1 {
                "normal".to_string()
            } else {
                format!("{}px", self.inherited_text.letter_spacing)
            }),
            "word-spacing" => Some(if self.inherited_text.word_spacing == -1 {
                "normal".to_string()
            } else {
                format!("{}px", self.inherited_text.word_spacing)
            }),
            "white-space" => Some(self.inherited_text.white_space.clone()),
            "direction" => Some(self.inherited_text.direction.clone()),
            "text-transform" => Some(self.inherited_text.text_transform.clone()),
            "font-variant" => Some(self.inherited_text.font_variant.clone()),
            "font-stretch" => Some(self.inherited_text.font_stretch.clone()),
            "text-indent" => Some(if self.inherited_text.text_indent == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.inherited_text.text_indent)
            }),
            "word-break" => Some(self.inherited_text.word_break.clone()),
            "line-break" => Some(self.inherited_text.line_break.clone()),
            "text-orientation" => Some(self.inherited_text.text_orientation.clone()),
            "overflow-wrap" => Some(self.inherited_text.overflow_wrap.clone()),
            "text-align-last" => Some(self.inherited_text.text_align_last.clone()),
            "tab-size" => Some(self.inherited_text.tab_size.to_string()),
            "hyphens" => Some(self.inherited_text.hyphens.clone()),
            "text-rendering" => Some(self.inherited_text.text_rendering.clone()),
            "image-rendering" => Some(self.inherited_text.image_rendering.clone()),
            "font-variant-caps" => Some(self.inherited_text.font_variant_caps.clone()),
            "text-shadow" => self
                .inherited_text
                .text_shadow
                .as_ref()
                .map(css_value_to_string),

            // InheritedList
            "list-style-type" => Some(self.inherited_list.list_style_type.clone()),
            "list-style-position" => Some(self.inherited_list.list_style_position.clone()),
            "list-style-image" => Some(self.inherited_list.list_style_image.clone()),

            // InheritedTable
            "caption-side" => Some(self.inherited_table.caption_side.clone()),
            "border-collapse" => Some(self.inherited_table.border_collapse.clone()),
            "border-spacing" => Some(self.inherited_table.border_spacing.to_string()),

            // InheritedUI
            "cursor" => Some(self.inherited_ui.cursor.clone()),
            "quotes" => Some(self.inherited_ui.quotes.clone()),
            "accent-color" => Some(self.inherited_ui.accent_color.clone()),
            "caret-color" => Some(self.inherited_ui.caret_color.clone()),

            // InheritedEffects
            "visibility" => Some(self.inherited_effects.visibility.clone()),
            "empty-cells" => Some(self.inherited_effects.empty_cells.clone()),

            // ResetBox
            "display" => Some(self.reset_box.display.clone()),
            "width" => Some(if self.reset_box.width == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_box.width)
            }),
            "height" => Some(if self.reset_box.height == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_box.height)
            }),
            "position" => Some(self.reset_box.position.clone()),
            "float" => Some(self.reset_box.float.clone()),
            "clear" => Some(self.reset_box.clear.clone()),
            "overflow" => Some(self.reset_box.overflow.clone()),
            "overflow-x" => Some(self.reset_box.overflow_x.clone()),
            "overflow-y" => Some(self.reset_box.overflow_y.clone()),
            "z-index" => Some(if self.reset_box.z_index == i32::MIN {
                "auto".to_string()
            } else {
                self.reset_box.z_index.to_string()
            }),
            "box-sizing" => Some(self.reset_box.box_sizing.clone()),
            "min-width" => Some(if self.reset_box.min_width == -1 {
                "0px".to_string()
            } else if self.reset_box.min_width >= WIDTH_PERCENT_BAND {
                format!("{}%", self.reset_box.min_width - WIDTH_PERCENT_BAND)
            } else {
                format!("{}px", self.reset_box.min_width)
            }),
            "min-height" => Some(if self.reset_box.min_height == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_box.min_height)
            }),
            "max-width" => Some(if self.reset_box.max_width == -1 {
                "none".to_string()
            } else if self.reset_box.max_width >= WIDTH_PERCENT_BAND {
                format!("{}%", self.reset_box.max_width - WIDTH_PERCENT_BAND)
            } else {
                format!("{}px", self.reset_box.max_width)
            }),
            "max-height" => Some(if self.reset_box.max_height == -1 {
                "none".to_string()
            } else {
                format!("{}px", self.reset_box.max_height)
            }),
            "vertical-align" => Some(match self.reset_box.vertical_align {
                -1 => "baseline".to_string(),
                -2 => "sub".to_string(),
                -3 => "super".to_string(),
                -4 => "top".to_string(),
                -5 => "bottom".to_string(),
                -6 => "middle".to_string(),
                v if v >= 50000 => format!("{}px", v - 100000),
                v => format!("{}px", v),
            }),
            "object-fit" => Some(self.reset_box.object_fit.clone()),
            "object-position" => Some(self.reset_box.object_position.clone()),
            "scroll-behavior" => Some(self.reset_box.scroll_behavior.clone()),
            "user-select" => Some(self.reset_box.user_select.clone()),
            "pointer-events" => Some(self.reset_box.pointer_events.clone()),
            "aspect-ratio" => Some(self.reset_box.aspect_ratio.clone()),

            // ResetSurround
            "margin-top" => Some(if self.reset_surround.margin_top == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_top)
            }),
            "margin-right" => Some(if self.reset_surround.margin_right == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_right)
            }),
            "margin-bottom" => Some(if self.reset_surround.margin_bottom == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_bottom)
            }),
            "margin-left" => Some(if self.reset_surround.margin_left == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_left)
            }),
            "margin-block-start" => Some(if self.reset_surround.margin_block_start == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_block_start)
            }),
            "margin-block-end" => Some(if self.reset_surround.margin_block_end == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.margin_block_end)
            }),
            "padding-top" => Some(if self.reset_surround.padding_top == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_top)
            }),
            "padding-right" => Some(if self.reset_surround.padding_right == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_right)
            }),
            "padding-bottom" => Some(if self.reset_surround.padding_bottom == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_bottom)
            }),
            "padding-left" => Some(if self.reset_surround.padding_left == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_left)
            }),
            "padding-block-start" => Some(if self.reset_surround.padding_block_start == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_block_start)
            }),
            "padding-block-end" => Some(if self.reset_surround.padding_block_end == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.padding_block_end)
            }),
            "border-top-width" => Some(if self.reset_surround.border_top_width == -1 {
                "medium".to_string()
            } else {
                format!("{}px", self.reset_surround.border_top_width)
            }),
            "border-right-width" => Some(if self.reset_surround.border_right_width == -1 {
                "medium".to_string()
            } else {
                format!("{}px", self.reset_surround.border_right_width)
            }),
            "border-bottom-width" => Some(if self.reset_surround.border_bottom_width == -1 {
                "medium".to_string()
            } else {
                format!("{}px", self.reset_surround.border_bottom_width)
            }),
            "border-left-width" => Some(if self.reset_surround.border_left_width == -1 {
                "medium".to_string()
            } else {
                format!("{}px", self.reset_surround.border_left_width)
            }),
            "border-top-style" => Some(self.reset_surround.border_top_style.clone()),
            "border-right-style" => Some(self.reset_surround.border_right_style.clone()),
            "border-bottom-style" => Some(self.reset_surround.border_bottom_style.clone()),
            "border-left-style" => Some(self.reset_surround.border_left_style.clone()),
            "border-top-color" => Some(self.reset_surround.border_top_color.clone()),
            "border-right-color" => Some(self.reset_surround.border_right_color.clone()),
            "border-bottom-color" => Some(self.reset_surround.border_bottom_color.clone()),
            "border-left-color" => Some(self.reset_surround.border_left_color.clone()),
            "border-top-left-radius" => Some(if self.reset_surround.border_top_left_radius == -1 {
                "0px".to_string()
            } else {
                format!("{}px", self.reset_surround.border_top_left_radius)
            }),
            "border-top-right-radius" => {
                Some(if self.reset_surround.border_top_right_radius == -1 {
                    "0px".to_string()
                } else {
                    format!("{}px", self.reset_surround.border_top_right_radius)
                })
            }
            "border-bottom-right-radius" => {
                Some(if self.reset_surround.border_bottom_right_radius == -1 {
                    "0px".to_string()
                } else {
                    format!("{}px", self.reset_surround.border_bottom_right_radius)
                })
            }
            "border-bottom-left-radius" => {
                Some(if self.reset_surround.border_bottom_left_radius == -1 {
                    "0px".to_string()
                } else {
                    format!("{}px", self.reset_surround.border_bottom_left_radius)
                })
            }
            "top" => Some(if self.reset_surround.top == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_surround.top)
            }),
            "right" => Some(if self.reset_surround.right == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_surround.right)
            }),
            "bottom" => Some(if self.reset_surround.bottom == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_surround.bottom)
            }),
            "left" => Some(if self.reset_surround.left == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_surround.left)
            }),

            // ResetBackground
            "background-color" => Some(self.reset_background.background_color.clone()),
            "background-image" => Some(self.reset_background.background_image.clone()),
            "background-repeat" => Some(self.reset_background.background_repeat.clone()),
            "background-position" => Some(self.reset_background.background_position.clone()),
            "background-size" => Some(self.reset_background.background_size.clone()),
            "background-attachment" => Some(self.reset_background.background_attachment.clone()),

            // ResetFlex
            "flex-grow" => Some(self.reset_flex.flex_grow.to_string()),
            "flex-shrink" => Some(self.reset_flex.flex_shrink.to_string()),
            "flex-basis" => Some(if self.reset_flex.flex_basis == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_flex.flex_basis)
            }),
            "flex-direction" => Some(self.reset_flex.flex_direction.clone()),
            "flex-wrap" => Some(self.reset_flex.flex_wrap.clone()),
            "justify-content" => Some(self.reset_flex.justify_content.clone()),
            "align-items" => Some(self.reset_flex.align_items.clone()),
            "align-self" => Some(self.reset_flex.align_self.clone()),
            "order" => Some(self.reset_flex.order.to_string()),
            "align-content" => Some(self.reset_flex.align_content.clone()),
            "row-gap" => Some(if self.reset_flex.row_gap == -1 {
                "normal".to_string()
            } else {
                format!("{}px", self.reset_flex.row_gap)
            }),
            "column-gap" => Some(if self.reset_flex.column_gap == -1 {
                "normal".to_string()
            } else {
                format!("{}px", self.reset_flex.column_gap)
            }),
            "column-count" => Some(if self.reset_flex.column_count == -1 {
                "auto".to_string()
            } else {
                self.reset_flex.column_count.to_string()
            }),
            "column-width" => Some(if self.reset_flex.column_width == -1 {
                "auto".to_string()
            } else {
                format!("{}px", self.reset_flex.column_width)
            }),

            // ResetTable
            "table-layout" => Some(self.reset_table.table_layout.clone()),

            // ResetEffects
            "opacity" => Some(self.reset_effects.opacity.to_string()),
            "outline-width" => Some(if self.reset_effects.outline_width == -1 {
                "medium".to_string()
            } else {
                format!("{}px", self.reset_effects.outline_width)
            }),
            "outline-style" => Some(self.reset_effects.outline_style.clone()),
            "outline-color" => Some(self.reset_effects.outline_color.clone()),
            "outline-offset" => Some(format!("{}px", self.reset_effects.outline_offset)),
            "transition-duration" => Some(format!("{}s", self.reset_effects.transition_duration)),
            "transition-property" => Some(self.reset_effects.transition_property.clone()),
            "transition-timing-function" => {
                Some(self.reset_effects.transition_timing_function.clone())
            }
            "transition-delay" => Some(self.reset_effects.transition_delay.clone()),
            "text-decoration-line" => Some(if self.reset_effects.text_decoration_line.is_empty() {
                "none".to_string()
            } else {
                self.reset_effects.text_decoration_line.clone()
            }),
            "text-decoration-color" => Some(self.reset_effects.text_decoration_color.clone()),
            "text-decoration-style" => {
                Some(if self.reset_effects.text_decoration_style.is_empty() {
                    "solid".to_string()
                } else {
                    self.reset_effects.text_decoration_style.clone()
                })
            }
            "text-overflow" => Some(self.reset_effects.text_overflow.clone()),
            "mask-type" => Some(self.reset_effects.mask_type.clone()),
            "box-shadow" => self
                .reset_effects
                .box_shadow
                .as_ref()
                .map(css_value_to_string),
            "transform" => Some(if self.reset_effects.transform.is_empty() {
                "none".to_string()
            } else {
                css_value_to_string(&crate::css::values::CssValue::Transform(
                    self.reset_effects.transform.clone(),
                ))
            }),

            _ => None,
        }
    }
}

// Private value conversion helpers for CategorizedComputedStyle
fn css_value_to_string(val: &crate::css::values::CssValue) -> String {
    use crate::css::values::{
        AlignItemsValue, BackfaceVisibilityValue, BackgroundBlendModeValue, BoxSizingValue, Color,
        CssValue, DisplayValue, EmptyCellsValue, FlexDirectionValue, IsolationValue,
        JustifyContentValue, LengthUnit, MixBlendModeValue, OverflowValue, PositionValue,
        ResizeValue, ZIndex,
    };
    match val {
        CssValue::Keyword(s) => s.clone(),
        CssValue::Length(v, unit) => {
            let unit_str = match unit {
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
        CssValue::Number(v) => format!("{}", v),
        CssValue::Color(Color::Rgba(r, g, b, a)) => {
            if *a == 255 {
                format!("rgb({}, {}, {})", r, g, b)
            } else {
                format!("rgba({}, {}, {}, {})", r, g, b, *a as f32 / 255.0)
            }
        }
        CssValue::Multiple(vec) => vec
            .iter()
            .map(css_value_to_string)
            .collect::<Vec<_>>()
            .join(" "),
        CssValue::Position(pv) => match pv {
            PositionValue::Static => "static".to_string(),
            PositionValue::Relative => "relative".to_string(),
            PositionValue::Absolute => "absolute".to_string(),
            PositionValue::Fixed => "fixed".to_string(),
            PositionValue::Sticky => "sticky".to_string(),
        },
        CssValue::Overflow(ov) => match ov {
            OverflowValue::Visible => "visible".to_string(),
            OverflowValue::Hidden => "hidden".to_string(),
            OverflowValue::Scroll => "scroll".to_string(),
            OverflowValue::Auto => "auto".to_string(),
        },
        CssValue::BoxSizing(bs) => match bs {
            BoxSizingValue::ContentBox => "content-box".to_string(),
            BoxSizingValue::BorderBox => "border-box".to_string(),
        },
        CssValue::Display(dv) => match dv {
            DisplayValue::Block => "block".to_string(),
            DisplayValue::Inline => "inline".to_string(),
            DisplayValue::InlineBlock => "inline-block".to_string(),
            DisplayValue::None => "none".to_string(),
            DisplayValue::Flex => "flex".to_string(),
            DisplayValue::Grid => "grid".to_string(),
            DisplayValue::InlineGrid => "inline-grid".to_string(),
            DisplayValue::Table => "table".to_string(),
            DisplayValue::TableRow => "table-row".to_string(),
            DisplayValue::TableCell => "table-cell".to_string(),
        },
        CssValue::FlexDirection(fd) => match fd {
            FlexDirectionValue::Row => "row".to_string(),
            FlexDirectionValue::RowReverse => "row-reverse".to_string(),
            FlexDirectionValue::Column => "column".to_string(),
            FlexDirectionValue::ColumnReverse => "column-reverse".to_string(),
        },
        CssValue::JustifyContent(jc) => match jc {
            JustifyContentValue::FlexStart => "flex-start".to_string(),
            JustifyContentValue::FlexEnd => "flex-end".to_string(),
            JustifyContentValue::Center => "center".to_string(),
            JustifyContentValue::SpaceBetween => "space-between".to_string(),
            JustifyContentValue::SpaceAround => "space-around".to_string(),
            JustifyContentValue::SpaceEvenly => "space-evenly".to_string(),
        },
        CssValue::AlignItems(ai) => match ai {
            AlignItemsValue::Stretch => "stretch".to_string(),
            AlignItemsValue::FlexStart => "flex-start".to_string(),
            AlignItemsValue::FlexEnd => "flex-end".to_string(),
            AlignItemsValue::Center => "center".to_string(),
            AlignItemsValue::Baseline => "baseline".to_string(),
        },
        CssValue::Transform(vec) => vec
            .iter()
            .map(|tf| match tf {
                crate::css::values::TransformFn::Translate { x, y } => {
                    let fmt_lp = |lp: &crate::css::values::LengthOrPercent| {
                        let u_str = match lp.unit {
                            LengthUnit::Px => "px",
                            LengthUnit::Em => "em",
                            LengthUnit::Rem => "rem",
                            LengthUnit::Pt => "pt",
                            LengthUnit::Percent => "%",
                            LengthUnit::Vw => "vw",
                            LengthUnit::Vh => "vh",
                        };
                        format!("{}{}", lp.value, u_str)
                    };
                    format!("translate({}, {})", fmt_lp(x), fmt_lp(y))
                }
                crate::css::values::TransformFn::TranslateX(x) => {
                    let fmt_lp = |lp: &crate::css::values::LengthOrPercent| {
                        let u_str = match lp.unit {
                            LengthUnit::Px => "px",
                            LengthUnit::Em => "em",
                            LengthUnit::Rem => "rem",
                            LengthUnit::Pt => "pt",
                            LengthUnit::Percent => "%",
                            LengthUnit::Vw => "vw",
                            LengthUnit::Vh => "vh",
                        };
                        format!("{}{}", lp.value, u_str)
                    };
                    format!("translatex({})", fmt_lp(x))
                }
                crate::css::values::TransformFn::TranslateY(y) => {
                    let fmt_lp = |lp: &crate::css::values::LengthOrPercent| {
                        let u_str = match lp.unit {
                            LengthUnit::Px => "px",
                            LengthUnit::Em => "em",
                            LengthUnit::Rem => "rem",
                            LengthUnit::Pt => "pt",
                            LengthUnit::Percent => "%",
                            LengthUnit::Vw => "vw",
                            LengthUnit::Vh => "vh",
                        };
                        format!("{}{}", lp.value, u_str)
                    };
                    format!("translatey({})", fmt_lp(y))
                }
                crate::css::values::TransformFn::Scale { x, y } => {
                    if x == y {
                        format!("scale({})", x)
                    } else {
                        format!("scale({}, {})", x, y)
                    }
                }
                crate::css::values::TransformFn::ScaleX(x) => format!("scalex({})", x),
                crate::css::values::TransformFn::ScaleY(y) => format!("scaley({})", y),
                crate::css::values::TransformFn::Rotate(crate::css::values::AngleDeg(deg)) => {
                    format!("rotate({}deg)", deg)
                }
                crate::css::values::TransformFn::Matrix(m) => {
                    format!(
                        "matrix({}, {}, {}, {}, {}, {})",
                        m[0], m[1], m[2], m[3], m[4], m[5]
                    )
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        CssValue::ZIndex(zi) => match zi {
            ZIndex::Auto => "auto".to_string(),
            ZIndex::Index(n) => n.to_string(),
        },
        CssValue::Opacity(val) => val.to_string(),
        CssValue::GridTemplate(tracks) => tracks
            .iter()
            .map(|track| match track {
                crate::css::values::GridTrackSize::Px(v) => format!("{}px", v),
                crate::css::values::GridTrackSize::Percent(v) => format!("{}%", v),
                crate::css::values::GridTrackSize::Fr(v) => format!("{}fr", v),
                crate::css::values::GridTrackSize::Auto => "auto".to_string(),
            })
            .collect::<Vec<_>>()
            .join(" "),
        CssValue::ScrollSnapType(sst) => match sst {
            crate::css::values::ScrollSnapTypeValue::None => "none".to_string(),
            crate::css::values::ScrollSnapTypeValue::Axis(axis, strictness) => {
                let axis_str = match axis {
                    crate::css::values::ScrollSnapAxis::X => "x",
                    crate::css::values::ScrollSnapAxis::Y => "y",
                    crate::css::values::ScrollSnapAxis::Block => "block",
                    crate::css::values::ScrollSnapAxis::Inline => "inline",
                    crate::css::values::ScrollSnapAxis::Both => "both",
                };
                let strict_str = match strictness {
                    crate::css::values::ScrollSnapStrictness::Mandatory => "mandatory",
                    crate::css::values::ScrollSnapStrictness::Proximity => "proximity",
                };
                format!("{} {}", axis_str, strict_str)
            }
        },
        CssValue::ScrollSnapAlign(ssa) => {
            let fmt_kw = |kw: crate::css::values::ScrollSnapAlignKeyword| match kw {
                crate::css::values::ScrollSnapAlignKeyword::None => "none",
                crate::css::values::ScrollSnapAlignKeyword::Start => "start",
                crate::css::values::ScrollSnapAlignKeyword::End => "end",
                crate::css::values::ScrollSnapAlignKeyword::Center => "center",
            };
            if ssa.block == ssa.inline {
                fmt_kw(ssa.block).to_string()
            } else {
                format!("{} {}", fmt_kw(ssa.block), fmt_kw(ssa.inline))
            }
        }
        CssValue::MixBlendMode(mbm) => match mbm {
            MixBlendModeValue::Normal => "normal".to_string(),
            MixBlendModeValue::Multiply => "multiply".to_string(),
            MixBlendModeValue::Screen => "screen".to_string(),
            MixBlendModeValue::Overlay => "overlay".to_string(),
            MixBlendModeValue::Darken => "darken".to_string(),
            MixBlendModeValue::Lighten => "lighten".to_string(),
            MixBlendModeValue::ColorDodge => "color-dodge".to_string(),
            MixBlendModeValue::ColorBurn => "color-burn".to_string(),
            MixBlendModeValue::HardLight => "hard-light".to_string(),
            MixBlendModeValue::SoftLight => "soft-light".to_string(),
            MixBlendModeValue::Difference => "difference".to_string(),
            MixBlendModeValue::Exclusion => "exclusion".to_string(),
            MixBlendModeValue::Hue => "hue".to_string(),
            MixBlendModeValue::Saturation => "saturation".to_string(),
            MixBlendModeValue::Color => "color".to_string(),
            MixBlendModeValue::Luminosity => "luminosity".to_string(),
        },
        CssValue::BackgroundBlendMode(bbm) => match bbm {
            BackgroundBlendModeValue::Normal => "normal".to_string(),
            BackgroundBlendModeValue::Multiply => "multiply".to_string(),
            BackgroundBlendModeValue::Screen => "screen".to_string(),
            BackgroundBlendModeValue::Overlay => "overlay".to_string(),
            BackgroundBlendModeValue::Darken => "darken".to_string(),
            BackgroundBlendModeValue::Lighten => "lighten".to_string(),
            BackgroundBlendModeValue::ColorDodge => "color-dodge".to_string(),
            BackgroundBlendModeValue::ColorBurn => "color-burn".to_string(),
            BackgroundBlendModeValue::HardLight => "hard-light".to_string(),
            BackgroundBlendModeValue::SoftLight => "soft-light".to_string(),
            BackgroundBlendModeValue::Difference => "difference".to_string(),
            BackgroundBlendModeValue::Exclusion => "exclusion".to_string(),
            BackgroundBlendModeValue::Hue => "hue".to_string(),
            BackgroundBlendModeValue::Saturation => "saturation".to_string(),
            BackgroundBlendModeValue::Color => "color".to_string(),
            BackgroundBlendModeValue::Luminosity => "luminosity".to_string(),
        },
        CssValue::Isolation(iso) => match iso {
            IsolationValue::Auto => "auto".to_string(),
            IsolationValue::Isolate => "isolate".to_string(),
        },
        CssValue::Resize(res) => match res {
            ResizeValue::None => "none".to_string(),
            ResizeValue::Both => "both".to_string(),
            ResizeValue::Horizontal => "horizontal".to_string(),
            ResizeValue::Vertical => "vertical".to_string(),
        },
        CssValue::BackfaceVisibility(bv) => match bv {
            BackfaceVisibilityValue::Visible => "visible".to_string(),
            BackfaceVisibilityValue::Hidden => "hidden".to_string(),
        },
        CssValue::EmptyCells(ec) => match ec {
            EmptyCellsValue::Show => "show".to_string(),
            EmptyCellsValue::Hide => "hide".to_string(),
        },
        CssValue::Hyphens(h) => h.as_str().to_string(),
        CssValue::LineBreak(lb) => lb.as_str().to_string(),
        CssValue::TextOrientation(to) => to.as_str().to_string(),
        CssValue::TextRendering(tr) => tr.as_str().to_string(),
        CssValue::ImageRendering(ir) => ir.as_str().to_string(),
        CssValue::FontVariantCaps(fvc) => fvc.as_str().to_string(),
        CssValue::FontVariantPosition(fvp) => fvp.as_str().to_string(),
        CssValue::FontStretch(fs) => fs.as_str().to_string(),
        CssValue::FontOpticalSizing(fos) => fos.as_str().to_string(),
        // TODO(spec): text-align-last fully plumbing to style and layout is a future task
        CssValue::TextAlignLast(tal) => tal.as_str().to_string(),
        // TODO(spec): unicode-bidi fully plumbing to style and layout is a future task
        CssValue::UnicodeBidi(ub) => ub.as_str().to_string(),
        CssValue::BoxDecorationBreak(bdb) => bdb.as_str().to_string(),
        CssValue::MaskType(mt) => mt.as_str().to_string(),
        CssValue::ScrollBehavior(sb) => sb.as_str().to_string(),
        CssValue::PrintColorAdjust(pca) => pca.as_str().to_string(),
        CssValue::ForcedColorAdjust(fca) => fca.as_str().to_string(),
    }
}

/// Like `value_to_px`, but for `<length> | auto` box dimensions (width/height): a
/// unitless non-zero number is an invalid length per CSS and resolves to `auto` (-1),
/// while a unitless `0` is valid and resolves to `0`. The legacy HashMap preserved this
/// by storing the raw `CssValue`; the typed i32 field needs the distinction made here.
fn value_to_px_or_auto(val: &crate::css::values::CssValue, font_size: u32) -> i32 {
    if let crate::css::values::CssValue::Number(v) = val {
        return if *v == 0.0 { 0 } else { -1 };
    }
    value_to_px(val, font_size)
}

/// Special helper for `height`: resolves a percentage `height` to `auto` (-1)
/// because containing-block heights are indefinite (content-driven) in this engine.
/// All other values resolve normally as in `value_to_px_or_auto`.
fn height_value_to_px_or_auto(val: &crate::css::values::CssValue, font_size: u32) -> i32 {
    if let crate::css::values::CssValue::Length(_, crate::css::values::LengthUnit::Percent) = val {
        return -1;
    }
    value_to_px_or_auto(val, font_size)
}

/// Percentage offset for min/max-width: values `>= WIDTH_PERCENT_BAND` encode a
/// percentage (`stored - WIDTH_PERCENT_BAND`) to be resolved against the
/// containing block at layout time; lower non-`-1` values are plain px. The i32
/// typed field cannot otherwise represent a percentage, which the pre-migration
/// HashMap style preserved.
pub const WIDTH_PERCENT_BAND: i32 = 1_000_000;

fn width_px_or_percent_band(val: &crate::css::values::CssValue, font_size: u32) -> i32 {
    if let crate::css::values::CssValue::Length(p, crate::css::values::LengthUnit::Percent) = val {
        return p.round() as i32 + WIDTH_PERCENT_BAND;
    }
    value_to_px(val, font_size)
}

fn value_to_px(val: &crate::css::values::CssValue, font_size: u32) -> i32 {
    match val {
        crate::css::values::CssValue::Length(v, unit) => match unit {
            crate::css::values::LengthUnit::Px => v.round() as i32,
            crate::css::values::LengthUnit::Em => (v * font_size as f32).round() as i32,
            crate::css::values::LengthUnit::Rem => (v * 16.0).round() as i32,
            crate::css::values::LengthUnit::Pt => (v * 96.0 / 72.0).round() as i32,
            _ => v.round() as i32,
        },
        crate::css::values::CssValue::Number(v) => v.round() as i32,
        _ => -1,
    }
}

fn flatten_value(
    val: &crate::css::values::CssValue,
    leaves: &mut Vec<crate::css::values::CssValue>,
) {
    match val {
        crate::css::values::CssValue::Multiple(list) => {
            for item in list {
                flatten_value(item, leaves);
            }
        }
        _ => {
            leaves.push(val.clone());
        }
    }
}

fn value_to_u32(val: &crate::css::values::CssValue) -> u32 {
    match val {
        crate::css::values::CssValue::Length(v, _) => v.round().max(0.0) as u32,
        crate::css::values::CssValue::Number(v) => v.round().max(0.0) as u32,
        _ => 0,
    }
}

fn value_to_f32(val: &crate::css::values::CssValue) -> f32 {
    match val {
        crate::css::values::CssValue::Number(v) => *v,
        crate::css::values::CssValue::Opacity(v) => *v,
        crate::css::values::CssValue::Length(v, _) => *v,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorized_style_sharing_and_inheritance() {
        let parent = CategorizedComputedStyle::initial();
        let child = CategorizedComputedStyle::inherit_from(&parent);

        // Inherited categories must have pointer equality between parent and child
        assert!(Arc::ptr_eq(&parent.inherited_text, &child.inherited_text));
        assert!(Arc::ptr_eq(&parent.inherited_list, &child.inherited_list));
        assert!(Arc::ptr_eq(&parent.inherited_table, &child.inherited_table));
        assert!(Arc::ptr_eq(&parent.inherited_ui, &child.inherited_ui));
        assert!(Arc::ptr_eq(
            &parent.inherited_effects,
            &child.inherited_effects
        ));

        // Reset categories must be shared with initial style (same shared pointers)
        let fresh_initial = CategorizedComputedStyle::initial();
        assert!(Arc::ptr_eq(&child.reset_box, &fresh_initial.reset_box));
        assert!(Arc::ptr_eq(
            &child.reset_surround,
            &fresh_initial.reset_surround
        ));
        assert!(Arc::ptr_eq(
            &child.reset_background,
            &fresh_initial.reset_background
        ));
        assert!(Arc::ptr_eq(&child.reset_flex, &fresh_initial.reset_flex));
        assert!(Arc::ptr_eq(&child.reset_table, &fresh_initial.reset_table));
        assert!(Arc::ptr_eq(
            &child.reset_effects,
            &fresh_initial.reset_effects
        ));
    }

    #[test]
    fn test_categorized_style_cow() {
        let original = CategorizedComputedStyle::initial();
        let mut cloned = original.clone();

        // Before any mutation, all category pointers must be equal
        assert!(Arc::ptr_eq(
            &original.inherited_text,
            &cloned.inherited_text
        ));
        assert!(Arc::ptr_eq(&original.reset_box, &cloned.reset_box));

        // Mutating cloned text color (inherited)
        cloned.set_color("red".to_string());

        // The mutated category should diverge. The initial color is the empty
        // "unspecified" sentinel (resolves to black at paint time).
        assert_eq!(original.inherited_text.color, "");
        assert_eq!(cloned.inherited_text.color, "red");
        assert!(!Arc::ptr_eq(
            &original.inherited_text,
            &cloned.inherited_text
        ));

        // Untouched categories must remain pointer-equal
        assert!(Arc::ptr_eq(&original.reset_box, &cloned.reset_box));
        assert!(Arc::ptr_eq(&original.reset_flex, &cloned.reset_flex));

        // Mutating cloned reset display
        cloned.set_display("block".to_string());
        assert_eq!(original.reset_box.display, "inline");
        assert_eq!(cloned.reset_box.display, "block");
        assert!(!Arc::ptr_eq(&original.reset_box, &cloned.reset_box));
    }

    #[test]
    fn test_initial_defaults() {
        let initial = CategorizedComputedStyle::initial();

        // Empty = unspecified sentinel (resolves to black at paint time).
        assert_eq!(initial.inherited_text.color, "");
        assert_eq!(initial.inherited_text.font_size, 16);
        assert_eq!(initial.reset_box.display, "inline");
        assert_eq!(initial.reset_box.width, -1);
        assert_eq!(initial.reset_effects.opacity, 1.0);
    }

    #[test]
    fn test_multicolumn_parsing() {
        use crate::css::values::{CssValue, LengthUnit};

        let mut style = CategorizedComputedStyle::initial();

        // 1. column-count integer parsed
        style.set_property("column-count", &CssValue::Number(3.0));
        assert_eq!(style.reset_flex.column_count, 3);
        assert_eq!(
            style.get_property_as_string("column-count"),
            Some("3".to_string())
        );

        // 2. column-count auto -> -1
        style.set_property("column-count", &CssValue::Keyword("auto".to_string()));
        assert_eq!(style.reset_flex.column_count, -1);
        assert_eq!(
            style.get_property_as_string("column-count"),
            Some("auto".to_string())
        );

        // Invalid column-count (e.g. 0 or negative) -> -1
        style.set_property("column-count", &CssValue::Number(0.0));
        assert_eq!(style.reset_flex.column_count, -1);
        style.set_property("column-count", &CssValue::Number(-5.0));
        assert_eq!(style.reset_flex.column_count, -1);

        // 3. column-width px parsed
        style.set_property("column-width", &CssValue::Length(200.0, LengthUnit::Px));
        assert_eq!(style.reset_flex.column_width, 200);
        assert_eq!(
            style.get_property_as_string("column-width"),
            Some("200px".to_string())
        );

        // column-width auto -> -1
        style.set_property("column-width", &CssValue::Keyword("auto".to_string()));
        assert_eq!(style.reset_flex.column_width, -1);
        assert_eq!(
            style.get_property_as_string("column-width"),
            Some("auto".to_string())
        );

        // 4. columns: 200px 3 sets both
        let cols_both = CssValue::Multiple(vec![
            CssValue::Length(200.0, LengthUnit::Px),
            CssValue::Number(3.0),
        ]);
        style.set_property("columns", &cols_both);
        assert_eq!(style.reset_flex.column_width, 200);
        assert_eq!(style.reset_flex.column_count, 3);
        assert_eq!(
            style.get_property_as_string("column-width"),
            Some("200px".to_string())
        );
        assert_eq!(
            style.get_property_as_string("column-count"),
            Some("3".to_string())
        );

        // 5. columns: auto 2 sets count only (width should be auto / -1)
        let cols_count_only = CssValue::Multiple(vec![
            CssValue::Keyword("auto".to_string()),
            CssValue::Number(2.0),
        ]);
        style.set_property("columns", &cols_count_only);
        assert_eq!(style.reset_flex.column_width, -1);
        assert_eq!(style.reset_flex.column_count, 2);
        assert_eq!(
            style.get_property_as_string("column-width"),
            Some("auto".to_string())
        );
        assert_eq!(
            style.get_property_as_string("column-count"),
            Some("2".to_string())
        );

        // 6. columns: 150px set width only (count should reset to -1)
        let cols_width_only = CssValue::Length(150.0, LengthUnit::Px);
        style.set_property("columns", &cols_width_only);
        assert_eq!(style.reset_flex.column_width, 150);
        assert_eq!(style.reset_flex.column_count, -1);
        assert_eq!(
            style.get_property_as_string("column-width"),
            Some("150px".to_string())
        );
        assert_eq!(
            style.get_property_as_string("column-count"),
            Some("auto".to_string())
        );
    }

    #[test]
    fn test_overflow_computed_t0499() {
        use crate::css::values::{CssValue, OverflowValue};

        let mut style = CategorizedComputedStyle::initial();

        // Assert initial defaults
        assert_eq!(style.reset_box.overflow, "visible");
        assert_eq!(style.reset_box.overflow_x, "visible");
        assert_eq!(style.reset_box.overflow_y, "visible");
        assert_eq!(
            style.get_property_as_string("overflow"),
            Some("visible".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-x"),
            Some("visible".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-y"),
            Some("visible".to_string())
        );

        // Set overflow: hidden
        style.set_property("overflow", &CssValue::Overflow(OverflowValue::Hidden));
        assert_eq!(style.reset_box.overflow, "hidden");
        assert_eq!(style.reset_box.overflow_x, "hidden");
        assert_eq!(style.reset_box.overflow_y, "hidden");
        assert_eq!(
            style.get_property_as_string("overflow"),
            Some("hidden".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-x"),
            Some("hidden".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-y"),
            Some("hidden".to_string())
        );

        // Set overflow-x: scroll
        style.set_property("overflow-x", &CssValue::Overflow(OverflowValue::Scroll));
        assert_eq!(style.reset_box.overflow_x, "scroll");
        assert_eq!(style.reset_box.overflow_y, "hidden"); // unchanged
        assert_eq!(
            style.get_property_as_string("overflow-x"),
            Some("scroll".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-y"),
            Some("hidden".to_string())
        );

        // Set overflow-y: auto
        style.set_property("overflow-y", &CssValue::Overflow(OverflowValue::Auto));
        assert_eq!(style.reset_box.overflow_x, "scroll"); // unchanged
        assert_eq!(style.reset_box.overflow_y, "auto");
        assert_eq!(
            style.get_property_as_string("overflow-x"),
            Some("scroll".to_string())
        );
        assert_eq!(
            style.get_property_as_string("overflow-y"),
            Some("auto".to_string())
        );
    }

    #[test]
    fn test_transform_categorization() {
        use crate::css::values::{CssValue, LengthOrPercent, LengthUnit, TransformFn};

        let mut style = CategorizedComputedStyle::initial();

        // 1. Initial default is empty
        assert!(style.reset_effects.transform.is_empty());
        assert_eq!(
            style.get_property_as_string("transform"),
            Some("none".to_string())
        );

        // 2. Set transform: translate(10px, 20px)
        let t_val = CssValue::Transform(vec![TransformFn::Translate {
            x: LengthOrPercent {
                value: 10.0,
                unit: LengthUnit::Px,
            },
            y: LengthOrPercent {
                value: 20.0,
                unit: LengthUnit::Px,
            },
        }]);
        style.set_property("transform", &t_val);
        assert_eq!(style.reset_effects.transform.len(), 1);
        if let TransformFn::Translate { x, y } = &style.reset_effects.transform[0] {
            assert_eq!(x.value, 10.0);
            assert_eq!(x.unit, LengthUnit::Px);
            assert_eq!(y.value, 20.0);
            assert_eq!(y.unit, LengthUnit::Px);
        } else {
            panic!("Expected Translate fn");
        }
        assert_eq!(
            style.get_property_as_string("transform"),
            Some("translate(10px, 20px)".to_string())
        );

        // 3. Set transform: none
        style.set_property("transform", &CssValue::Keyword("none".to_string()));
        assert!(style.reset_effects.transform.is_empty());
        assert_eq!(
            style.get_property_as_string("transform"),
            Some("none".to_string())
        );
    }

    #[test]
    fn test_mix_blend_mode_serialization() {
        use crate::css::values::{CssValue, MixBlendModeValue};

        assert_eq!(
            css_value_to_string(&CssValue::MixBlendMode(MixBlendModeValue::Multiply)),
            "multiply".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::MixBlendMode(MixBlendModeValue::ColorDodge)),
            "color-dodge".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::MixBlendMode(MixBlendModeValue::Normal)),
            "normal".to_string()
        );
    }

    #[test]
    fn test_background_blend_mode_serialization() {
        use crate::css::values::{BackgroundBlendModeValue, CssValue};

        assert_eq!(
            css_value_to_string(&CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::Multiply
            )),
            "multiply".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::ColorDodge
            )),
            "color-dodge".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::BackgroundBlendMode(
                BackgroundBlendModeValue::Normal
            )),
            "normal".to_string()
        );
    }

    #[test]
    fn test_isolation_serialization() {
        use crate::css::values::{CssValue, IsolationValue};

        assert_eq!(
            css_value_to_string(&CssValue::Isolation(IsolationValue::Auto)),
            "auto".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::Isolation(IsolationValue::Isolate)),
            "isolate".to_string()
        );
    }

    #[test]
    fn test_resize_serialization() {
        use crate::css::values::{CssValue, ResizeValue};

        assert_eq!(
            css_value_to_string(&CssValue::Resize(ResizeValue::None)),
            "none".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::Resize(ResizeValue::Both)),
            "both".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::Resize(ResizeValue::Horizontal)),
            "horizontal".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::Resize(ResizeValue::Vertical)),
            "vertical".to_string()
        );
    }

    #[test]
    fn test_backface_visibility_serialization() {
        use crate::css::values::{BackfaceVisibilityValue, CssValue};

        assert_eq!(
            css_value_to_string(&CssValue::BackfaceVisibility(
                BackfaceVisibilityValue::Visible
            )),
            "visible".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::BackfaceVisibility(
                BackfaceVisibilityValue::Hidden
            )),
            "hidden".to_string()
        );
    }

    #[test]
    fn test_empty_cells_serialization() {
        use crate::css::values::{CssValue, EmptyCellsValue};

        assert_eq!(
            css_value_to_string(&CssValue::EmptyCells(EmptyCellsValue::Show)),
            "show".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::EmptyCells(EmptyCellsValue::Hide)),
            "hide".to_string()
        );
    }

    #[test]
    fn test_image_rendering_style_categorization() {
        use crate::css::values::{CssValue, ImageRenderingValue};

        let mut style = CategorizedComputedStyle::initial();
        // Check initial default is auto
        assert_eq!(
            style.get_property_as_string("image-rendering"),
            Some("auto".to_string())
        );

        // Set to pixelated and read back
        style.set_property(
            "image-rendering",
            &CssValue::ImageRendering(ImageRenderingValue::Pixelated),
        );
        assert_eq!(
            style.get_property_as_string("image-rendering"),
            Some("pixelated".to_string())
        );

        // Set to crisp-edges and read back
        style.set_property(
            "image-rendering",
            &CssValue::ImageRendering(ImageRenderingValue::CrispEdges),
        );
        assert_eq!(
            style.get_property_as_string("image-rendering"),
            Some("crisp-edges".to_string())
        );

        // Test css_value_to_string serialization directly
        assert_eq!(
            css_value_to_string(&CssValue::ImageRendering(ImageRenderingValue::Auto)),
            "auto".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::ImageRendering(ImageRenderingValue::CrispEdges)),
            "crisp-edges".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::ImageRendering(ImageRenderingValue::Pixelated)),
            "pixelated".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::ImageRendering(ImageRenderingValue::Smooth)),
            "smooth".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::ImageRendering(ImageRenderingValue::HighQuality)),
            "high-quality".to_string()
        );
    }

    #[test]
    fn test_line_break_style_categorization() {
        use crate::css::values::{CssValue, LineBreakValue};

        let mut style = CategorizedComputedStyle::initial();
        // Check initial default is auto
        assert_eq!(
            style.get_property_as_string("line-break"),
            Some("auto".to_string())
        );

        // Set to strict and read back
        style.set_property("line-break", &CssValue::LineBreak(LineBreakValue::Strict));
        assert_eq!(
            style.get_property_as_string("line-break"),
            Some("strict".to_string())
        );

        // Set to anywhere and read back
        style.set_property("line-break", &CssValue::LineBreak(LineBreakValue::Anywhere));
        assert_eq!(
            style.get_property_as_string("line-break"),
            Some("anywhere".to_string())
        );

        // Test css_value_to_string serialization directly
        assert_eq!(
            css_value_to_string(&CssValue::LineBreak(LineBreakValue::Auto)),
            "auto".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::LineBreak(LineBreakValue::Loose)),
            "loose".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::LineBreak(LineBreakValue::Normal)),
            "normal".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::LineBreak(LineBreakValue::Strict)),
            "strict".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::LineBreak(LineBreakValue::Anywhere)),
            "anywhere".to_string()
        );
    }

    #[test]
    fn test_text_orientation_style_categorization() {
        use crate::css::values::{CssValue, TextOrientationValue};

        let mut style = CategorizedComputedStyle::initial();
        // Check initial default is mixed
        assert_eq!(
            style.get_property_as_string("text-orientation"),
            Some("mixed".to_string())
        );

        // Set to upright and read back
        style.set_property(
            "text-orientation",
            &CssValue::TextOrientation(TextOrientationValue::Upright),
        );
        assert_eq!(
            style.get_property_as_string("text-orientation"),
            Some("upright".to_string())
        );

        // Set to sideways and read back
        style.set_property(
            "text-orientation",
            &CssValue::TextOrientation(TextOrientationValue::Sideways),
        );
        assert_eq!(
            style.get_property_as_string("text-orientation"),
            Some("sideways".to_string())
        );

        // Test css_value_to_string serialization directly
        assert_eq!(
            css_value_to_string(&CssValue::TextOrientation(TextOrientationValue::Mixed)),
            "mixed".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::TextOrientation(TextOrientationValue::Upright)),
            "upright".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::TextOrientation(TextOrientationValue::Sideways)),
            "sideways".to_string()
        );
    }

    #[test]
    fn test_font_variant_caps_style_categorization() {
        use crate::css::values::{CssValue, FontVariantCapsValue};

        let mut style = CategorizedComputedStyle::initial();
        // Check initial default is normal
        assert_eq!(
            style.get_property_as_string("font-variant-caps"),
            Some("normal".to_string())
        );

        // Set to small-caps and read back
        style.set_property(
            "font-variant-caps",
            &CssValue::FontVariantCaps(FontVariantCapsValue::SmallCaps),
        );
        assert_eq!(
            style.get_property_as_string("font-variant-caps"),
            Some("small-caps".to_string())
        );

        // Set to unicase and read back
        style.set_property(
            "font-variant-caps",
            &CssValue::FontVariantCaps(FontVariantCapsValue::Unicase),
        );
        assert_eq!(
            style.get_property_as_string("font-variant-caps"),
            Some("unicase".to_string())
        );

        // Test css_value_to_string serialization directly
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(FontVariantCapsValue::Normal)),
            "normal".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(FontVariantCapsValue::SmallCaps)),
            "small-caps".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(
                FontVariantCapsValue::AllSmallCaps
            )),
            "all-small-caps".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(FontVariantCapsValue::PetiteCaps)),
            "petite-caps".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(
                FontVariantCapsValue::AllPetiteCaps
            )),
            "all-petite-caps".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(FontVariantCapsValue::Unicase)),
            "unicase".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontVariantCaps(
                FontVariantCapsValue::TitlingCaps
            )),
            "titling-caps".to_string()
        );
    }

    #[test]
    fn test_font_stretch_style_categorization() {
        use crate::css::values::{CssValue, FontStretchValue};

        let mut style = CategorizedComputedStyle::initial();
        // Check initial default is normal
        assert_eq!(
            style.get_property_as_string("font-stretch"),
            Some("normal".to_string())
        );

        // Set to condensed and read back
        style.set_property(
            "font-stretch",
            &CssValue::FontStretch(FontStretchValue::Condensed),
        );
        assert_eq!(
            style.get_property_as_string("font-stretch"),
            Some("condensed".to_string())
        );

        // Set to expanded and read back
        style.set_property(
            "font-stretch",
            &CssValue::FontStretch(FontStretchValue::Expanded),
        );
        assert_eq!(
            style.get_property_as_string("font-stretch"),
            Some("expanded".to_string())
        );

        // Test css_value_to_string serialization directly for some values
        assert_eq!(
            css_value_to_string(&CssValue::FontStretch(FontStretchValue::Normal)),
            "normal".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontStretch(FontStretchValue::Condensed)),
            "condensed".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontStretch(FontStretchValue::Expanded)),
            "expanded".to_string()
        );
    }

    #[test]
    fn test_font_optical_sizing_style_categorization() {
        use crate::css::values::{CssValue, FontOpticalSizingValue};

        assert_eq!(
            css_value_to_string(&CssValue::FontOpticalSizing(FontOpticalSizingValue::Auto)),
            "auto".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::FontOpticalSizing(FontOpticalSizingValue::None)),
            "none".to_string()
        );
    }

    #[test]
    fn test_mask_type_style_categorization() {
        use crate::css::values::{CssValue, MaskTypeValue};

        let mut style = CategorizedComputedStyle::initial();

        // Check default
        assert_eq!(style.reset_effects.mask_type, "luminance");
        assert_eq!(
            style.get_property_as_string("mask-type"),
            Some("luminance".to_string())
        );

        // Set to alpha via CssValue::MaskType
        style.set_property("mask-type", &CssValue::MaskType(MaskTypeValue::Alpha));
        assert_eq!(style.reset_effects.mask_type, "alpha");
        assert_eq!(
            style.get_property_as_string("mask-type"),
            Some("alpha".to_string())
        );

        // Set to luminance via CssValue::Keyword
        style.set_property("mask-type", &CssValue::Keyword("luminance".to_string()));
        assert_eq!(style.reset_effects.mask_type, "luminance");
        assert_eq!(
            style.get_property_as_string("mask-type"),
            Some("luminance".to_string())
        );

        // css_value_to_string serialization tests
        assert_eq!(
            css_value_to_string(&CssValue::MaskType(MaskTypeValue::Luminance)),
            "luminance".to_string()
        );
        assert_eq!(
            css_value_to_string(&CssValue::MaskType(MaskTypeValue::Alpha)),
            "alpha".to_string()
        );
    }
}
