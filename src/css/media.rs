#![forbid(unsafe_code)]
#![allow(dead_code)]

// spec: https://www.w3.org/TR/css-conditional-3/
// spec: https://www.w3.org/TR/mediaqueries-4/

use crate::css::parser::{ComponentValue, QualifiedRule, Rule, Stylesheet};
use crate::css::{CssToken, CssTokenizer};
use std::cell::Cell;

/// Represents the preferred color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    Light,
    Dark,
}

/// Represents the preferred reduced motion setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefersReducedMotion {
    NoPreference,
    Reduce,
}

/// Represents the preferred contrast setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefersContrast {
    NoPreference,
    More,
    Less,
    Custom,
}

/// Represents the preferred reduced data setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefersReducedData {
    NoPreference,
    Reduce,
}

/// Represents the preferred reduced transparency setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefersReducedTransparency {
    NoPreference,
    Reduce,
}

/// Represents whether forced colors are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForcedColors {
    None,
    Active,
}

/// Represents whether inverted colors are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvertedColors {
    None,
    Inverted,
}

/// Represents the update frequency of the output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    None,
    Slow,
    Fast,
}

/// Represents the scripting support of the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scripting {
    None,
    InitialOnly,
    Enabled,
}

/// Represents whether the primary input mechanism can hover over elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hover {
    None,
    Hover,
}

/// Represents the accuracy of the primary pointing device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pointer {
    None,
    Coarse,
    Fine,
}

/// Represents the color gamut of the output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorGamut {
    Srgb,
    P3,
    Rec2020,
}

/// Represents the display mode of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
    WindowControlsOverlay,
    PictureInPicture,
    Tabbed,
    Borderless,
}

/// Represents the overflow behavior of the output device in the block axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowBlock {
    None,
    Scroll,
    Paged,
    OptionalPaged,
}

/// Represents the overflow behavior of the output device in the inline axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverflowInline {
    None,
    Scroll,
}

/// Represents the dynamic range of the output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicRange {
    Standard,
    High,
}

/// Represents the environment blending mode of the output device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentBlending {
    Opaque,
    Additive,
    Subtractive,
}

/// Represents the ambient light level of the environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightLevel {
    Dim,
    Normal,
    Washed,
}

/// Represents the posture of the device (for foldables).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePosture {
    Continuous,
    Folded,
}

/// Represents the navigation controls available on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavControls {
    None,
    Back,
}

/// Represents the shape of the display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayShape {
    Rect,
    Round,
}

thread_local! {
    static PREFERRED_COLOR_SCHEME: Cell<ColorScheme> = const { Cell::new(ColorScheme::Light) };
    static PREFERS_REDUCED_MOTION: Cell<PrefersReducedMotion> = const { Cell::new(PrefersReducedMotion::NoPreference) };
    static PREFERS_CONTRAST: Cell<PrefersContrast> = const { Cell::new(PrefersContrast::NoPreference) };
    static PREFERS_REDUCED_DATA: Cell<PrefersReducedData> = const { Cell::new(PrefersReducedData::NoPreference) };
    static PREFERS_REDUCED_TRANSPARENCY: Cell<PrefersReducedTransparency> = const { Cell::new(PrefersReducedTransparency::NoPreference) };
    static FORCED_COLORS: Cell<ForcedColors> = const { Cell::new(ForcedColors::None) };
    static INVERTED_COLORS: Cell<InvertedColors> = const { Cell::new(InvertedColors::None) };
    static UPDATE_MODE: Cell<UpdateMode> = const { Cell::new(UpdateMode::Fast) };
    static SCRIPTING: Cell<Scripting> = const { Cell::new(Scripting::Enabled) };
    static HOVER: Cell<Hover> = const { Cell::new(Hover::Hover) };
    static ANY_HOVER: Cell<Hover> = const { Cell::new(Hover::Hover) };
    static POINTER: Cell<Pointer> = const { Cell::new(Pointer::Fine) };
    static ANY_POINTER: Cell<Pointer> = const { Cell::new(Pointer::Fine) };
    static VIEWPORT_H: Cell<f32> = const { Cell::new(1024.0) };
    static DYNAMIC_RANGE: Cell<DynamicRange> = const { Cell::new(DynamicRange::Standard) };
    static VIDEO_DYNAMIC_RANGE: Cell<DynamicRange> = const { Cell::new(DynamicRange::Standard) };
    static ENVIRONMENT_BLENDING: Cell<EnvironmentBlending> = const { Cell::new(EnvironmentBlending::Opaque) };
    static LIGHT_LEVEL: Cell<LightLevel> = const { Cell::new(LightLevel::Normal) };
    static DEVICE_POSTURE: Cell<DevicePosture> = const { Cell::new(DevicePosture::Continuous) };
    static NAV_CONTROLS: Cell<NavControls> = const { Cell::new(NavControls::None) };
    static VIDEO_COLOR_GAMUT: Cell<ColorGamut> = const { Cell::new(ColorGamut::Srgb) };
    static DISPLAY_SHAPE: Cell<DisplayShape> = const { Cell::new(DisplayShape::Rect) };
    static HORIZONTAL_VIEWPORT_SEGMENTS: Cell<i32> = const { Cell::new(1) };
    static VERTICAL_VIEWPORT_SEGMENTS: Cell<i32> = const { Cell::new(1) };
    static DEVICE_PIXEL_RATIO: Cell<f32> = const { Cell::new(1.0) };
    static COLOR_GAMUT: Cell<ColorGamut> = const { Cell::new(ColorGamut::Srgb) };
    static DEVICE_WIDTH: Cell<f32> = const { Cell::new(1920.0) };
    static DEVICE_HEIGHT: Cell<f32> = const { Cell::new(1080.0) };
    static DISPLAY_MODE: Cell<DisplayMode> = const { Cell::new(DisplayMode::Browser) };
    static OVERFLOW_BLOCK: Cell<OverflowBlock> = const { Cell::new(OverflowBlock::Scroll) };
    static OVERFLOW_INLINE: Cell<OverflowInline> = const { Cell::new(OverflowInline::Scroll) };
}

/// Sets the device pixel ratio (DPR) for the current thread (default 1.0).
pub fn set_device_pixel_ratio(dpr: f32) {
    DEVICE_PIXEL_RATIO.with(|c| c.set(dpr));
}

/// Gets the device pixel ratio (DPR) for the current thread.
pub fn device_pixel_ratio() -> f32 {
    DEVICE_PIXEL_RATIO.with(|c| c.get())
}

/// Sets the horizontal viewport segments for the current thread.
pub fn set_horizontal_viewport_segments(val: i32) {
    HORIZONTAL_VIEWPORT_SEGMENTS.with(|c| c.set(val));
}

/// Gets the horizontal viewport segments for the current thread.
pub fn horizontal_viewport_segments() -> i32 {
    HORIZONTAL_VIEWPORT_SEGMENTS.with(|c| c.get())
}

/// Sets the vertical viewport segments for the current thread.
pub fn set_vertical_viewport_segments(val: i32) {
    VERTICAL_VIEWPORT_SEGMENTS.with(|c| c.set(val));
}

/// Gets the vertical viewport segments for the current thread.
pub fn vertical_viewport_segments() -> i32 {
    VERTICAL_VIEWPORT_SEGMENTS.with(|c| c.get())
}

/// Sets the viewport height for the current thread (default 1024.0 matching standard height).
pub fn set_viewport_h(h: f32) {
    VIEWPORT_H.with(|c| c.set(h));
}

/// Gets the viewport height for the current thread.
pub fn viewport_h() -> f32 {
    VIEWPORT_H.with(|c| c.get())
}

/// Sets the device width for the current thread (default 1920.0).
pub fn set_device_width(w: f32) {
    DEVICE_WIDTH.with(|c| c.set(w));
}

/// Gets the device width for the current thread.
pub fn device_width() -> f32 {
    DEVICE_WIDTH.with(|c| c.get())
}

/// Sets the device height for the current thread (default 1080.0).
pub fn set_device_height(h: f32) {
    DEVICE_HEIGHT.with(|c| c.set(h));
}

/// Gets the device height for the current thread.
pub fn device_height() -> f32 {
    DEVICE_HEIGHT.with(|c| c.get())
}

/// Sets the preferred color scheme for the current thread.
pub fn set_preferred_color_scheme(scheme: ColorScheme) {
    PREFERRED_COLOR_SCHEME.with(|c| c.set(scheme));
}

/// Gets the preferred color scheme for the current thread.
pub fn preferred_color_scheme() -> ColorScheme {
    PREFERRED_COLOR_SCHEME.with(|c| c.get())
}

/// Sets the preferred reduced motion setting for the current thread.
pub fn set_prefers_reduced_motion(val: PrefersReducedMotion) {
    PREFERS_REDUCED_MOTION.with(|c| c.set(val));
}

/// Gets the preferred reduced motion setting for the current thread.
pub fn prefers_reduced_motion() -> PrefersReducedMotion {
    PREFERS_REDUCED_MOTION.with(|c| c.get())
}

/// Sets the preferred contrast setting for the current thread.
pub fn set_prefers_contrast(val: PrefersContrast) {
    PREFERS_CONTRAST.with(|c| c.set(val));
}

/// Gets the preferred contrast setting for the current thread.
pub fn prefers_contrast() -> PrefersContrast {
    PREFERS_CONTRAST.with(|c| c.get())
}

/// Sets the preferred reduced data setting for the current thread.
pub fn set_prefers_reduced_data(val: PrefersReducedData) {
    PREFERS_REDUCED_DATA.with(|c| c.set(val));
}

/// Gets the preferred reduced data setting for the current thread.
pub fn prefers_reduced_data() -> PrefersReducedData {
    PREFERS_REDUCED_DATA.with(|c| c.get())
}

/// Sets the preferred reduced transparency setting for the current thread.
pub fn set_prefers_reduced_transparency(val: PrefersReducedTransparency) {
    PREFERS_REDUCED_TRANSPARENCY.with(|c| c.set(val));
}

/// Gets the preferred reduced transparency setting for the current thread.
pub fn prefers_reduced_transparency() -> PrefersReducedTransparency {
    PREFERS_REDUCED_TRANSPARENCY.with(|c| c.get())
}

/// Sets the forced colors setting for the current thread.
pub fn set_forced_colors(val: ForcedColors) {
    FORCED_COLORS.with(|c| c.set(val));
}

/// Gets the forced colors setting for the current thread.
pub fn forced_colors() -> ForcedColors {
    FORCED_COLORS.with(|c| c.get())
}

/// Sets the inverted colors setting for the current thread.
pub fn set_inverted_colors(val: InvertedColors) {
    INVERTED_COLORS.with(|c| c.set(val));
}

/// Gets the inverted colors setting for the current thread.
pub fn inverted_colors() -> InvertedColors {
    INVERTED_COLORS.with(|c| c.get())
}

/// Sets the update mode setting for the current thread.
pub fn set_update_mode(val: UpdateMode) {
    UPDATE_MODE.with(|c| c.set(val));
}

/// Gets the update mode setting for the current thread.
pub fn update_mode() -> UpdateMode {
    UPDATE_MODE.with(|c| c.get())
}

/// Sets the scripting setting for the current thread.
pub fn set_scripting(val: Scripting) {
    SCRIPTING.with(|c| c.set(val));
}

/// Gets the scripting setting for the current thread.
pub fn scripting() -> Scripting {
    SCRIPTING.with(|c| c.get())
}

/// Sets the hover setting for the current thread.
pub fn set_hover(val: Hover) {
    HOVER.with(|c| c.set(val));
}

/// Gets the hover setting for the current thread.
pub fn hover() -> Hover {
    HOVER.with(|c| c.get())
}

/// Sets the any-hover setting for the current thread.
pub fn set_any_hover(val: Hover) {
    ANY_HOVER.with(|c| c.set(val));
}

/// Gets the any-hover setting for the current thread.
pub fn any_hover() -> Hover {
    ANY_HOVER.with(|c| c.get())
}

/// Sets the pointer setting for the current thread.
pub fn set_pointer(val: Pointer) {
    POINTER.with(|c| c.set(val));
}

/// Gets the pointer setting for the current thread.
pub fn pointer() -> Pointer {
    POINTER.with(|c| c.get())
}

/// Sets the any-pointer setting for the current thread.
pub fn set_any_pointer(val: Pointer) {
    ANY_POINTER.with(|c| c.set(val));
}

/// Gets the any-pointer setting for the current thread.
pub fn any_pointer() -> Pointer {
    ANY_POINTER.with(|c| c.get())
}

/// Sets the dynamic range setting for the current thread.
pub fn set_dynamic_range(val: DynamicRange) {
    DYNAMIC_RANGE.with(|c| c.set(val));
}

/// Gets the dynamic range setting for the current thread.
pub fn dynamic_range() -> DynamicRange {
    DYNAMIC_RANGE.with(|c| c.get())
}

/// Sets the video dynamic range setting for the current thread.
pub fn set_video_dynamic_range(val: DynamicRange) {
    VIDEO_DYNAMIC_RANGE.with(|c| c.set(val));
}

/// Gets the video dynamic range setting for the current thread.
pub fn video_dynamic_range() -> DynamicRange {
    VIDEO_DYNAMIC_RANGE.with(|c| c.get())
}

/// Sets the environment blending mode for the current thread.
pub fn set_environment_blending(val: EnvironmentBlending) {
    ENVIRONMENT_BLENDING.with(|c| c.set(val));
}

/// Gets the environment blending mode for the current thread.
pub fn environment_blending() -> EnvironmentBlending {
    ENVIRONMENT_BLENDING.with(|c| c.get())
}

/// Sets the ambient light level for the current thread.
pub fn set_light_level(val: LightLevel) {
    LIGHT_LEVEL.with(|c| c.set(val));
}

/// Gets the ambient light level for the current thread.
pub fn light_level() -> LightLevel {
    LIGHT_LEVEL.with(|c| c.get())
}

/// Sets the device posture for the current thread.
pub fn set_device_posture(val: DevicePosture) {
    DEVICE_POSTURE.with(|c| c.set(val));
}

/// Gets the device posture for the current thread.
pub fn device_posture() -> DevicePosture {
    DEVICE_POSTURE.with(|c| c.get())
}

/// Sets the navigation controls setting for the current thread.
pub fn set_nav_controls(val: NavControls) {
    NAV_CONTROLS.with(|c| c.set(val));
}

/// Gets the navigation controls setting for the current thread.
pub fn nav_controls() -> NavControls {
    NAV_CONTROLS.with(|c| c.get())
}

/// Sets the video color gamut setting for the current thread.
pub fn set_video_color_gamut(val: ColorGamut) {
    VIDEO_COLOR_GAMUT.with(|c| c.set(val));
}

/// Gets the video color gamut setting for the current thread.
pub fn video_color_gamut() -> ColorGamut {
    VIDEO_COLOR_GAMUT.with(|c| c.get())
}

/// Sets the display shape setting for the current thread.
pub fn set_display_shape(val: DisplayShape) {
    DISPLAY_SHAPE.with(|c| c.set(val));
}

/// Gets the display shape setting for the current thread.
pub fn display_shape() -> DisplayShape {
    DISPLAY_SHAPE.with(|c| c.get())
}

/// Sets the color gamut of the output device.
pub fn set_color_gamut(val: ColorGamut) {
    COLOR_GAMUT.with(|c| c.set(val));
}

/// Gets the color gamut of the output device.
pub fn color_gamut() -> ColorGamut {
    COLOR_GAMUT.with(|c| c.get())
}

/// Sets the display mode of the application.
pub fn set_display_mode(val: DisplayMode) {
    DISPLAY_MODE.with(|c| c.set(val));
}

/// Gets the display mode of the application.
pub fn display_mode() -> DisplayMode {
    DISPLAY_MODE.with(|c| c.get())
}

/// Sets the overflow behavior of the output device in the block axis.
pub fn set_overflow_block(val: OverflowBlock) {
    OVERFLOW_BLOCK.with(|c| c.set(val));
}

/// Gets the overflow behavior of the output device in the block axis.
pub fn overflow_block() -> OverflowBlock {
    OVERFLOW_BLOCK.with(|c| c.get())
}

/// Sets the overflow behavior of the output device in the inline axis.
pub fn set_overflow_inline(val: OverflowInline) {
    OVERFLOW_INLINE.with(|c| c.set(val));
}

/// Gets the overflow behavior of the output device in the inline axis.
pub fn overflow_inline() -> OverflowInline {
    OVERFLOW_INLINE.with(|c| c.get())
}

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

fn find_top_level_operators(tokens: &[CssToken]) -> (bool, bool, Vec<usize>) {
    let mut has_and = false;
    let mut has_or = false;
    let mut op_indices = Vec::new();
    let mut depth = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            CssToken::LeftParen | CssToken::LeftBrace | CssToken::LeftBracket => {
                depth += 1;
            }
            CssToken::RightParen | CssToken::RightBrace | CssToken::RightBracket => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => {
                if depth == 0 {
                    if is_ident(token, "and") {
                        has_and = true;
                        op_indices.push(i);
                    } else if is_ident(token, "or") {
                        has_or = true;
                        op_indices.push(i);
                    }
                }
            }
        }
    }
    (has_and, has_or, op_indices)
}

fn evaluate_media_condition(tokens: &[CssToken], viewport_w: f32) -> Option<bool> {
    if tokens.is_empty() {
        return None;
    }

    let (has_and, has_or, op_indices) = find_top_level_operators(tokens);

    if has_and && has_or {
        // Mixing 'and' and 'or' without parentheses is invalid
        return Some(false);
    }

    if has_and {
        let mut last_idx = 0;
        let mut parts = Vec::new();
        for &op_idx in &op_indices {
            parts.push(&tokens[last_idx..op_idx]);
            last_idx = op_idx + 1;
        }
        parts.push(&tokens[last_idx..]);

        let mut all_match = true;
        for part in parts {
            if part.is_empty() {
                return Some(false);
            }
            if let Some(res) = evaluate_media_condition(part, viewport_w) {
                if !res {
                    all_match = false;
                }
            } else {
                return Some(false);
            }
        }
        return Some(all_match);
    }

    if has_or {
        let mut last_idx = 0;
        let mut parts = Vec::new();
        for &op_idx in &op_indices {
            parts.push(&tokens[last_idx..op_idx]);
            last_idx = op_idx + 1;
        }
        parts.push(&tokens[last_idx..]);

        let mut any_match = false;
        for part in parts {
            if part.is_empty() {
                return Some(false);
            }
            if let Some(res) = evaluate_media_condition(part, viewport_w) {
                if res {
                    any_match = true;
                }
            } else {
                return Some(false);
            }
        }
        return Some(any_match);
    }

    if tokens.len() >= 2 && is_ident(&tokens[0], "not") {
        if let Some(res) = evaluate_media_condition(&tokens[1..], viewport_w) {
            return Some(!res);
        } else {
            return Some(false);
        }
    }

    if tokens.len() >= 2
        && matches!(tokens[0], CssToken::LeftParen)
        && matches!(tokens[tokens.len() - 1], CssToken::RightParen)
    {
        let inner = &tokens[1..tokens.len() - 1];
        let (has_and_inner, has_or_inner, _) = find_top_level_operators(inner);
        let is_cond = has_and_inner
            || has_or_inner
            || (inner.len() >= 2 && is_ident(&inner[0], "not"))
            || (inner.len() >= 2
                && matches!(inner[0], CssToken::LeftParen)
                && matches!(inner[inner.len() - 1], CssToken::RightParen));

        if is_cond && let Some(res) = evaluate_media_condition(inner, viewport_w) {
            return Some(res);
        }

        return Some(evaluate_feature(inner, viewport_w));
    }

    Some(evaluate_feature(tokens, viewport_w))
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

    if idx < tokens.len() {
        if let CssToken::LeftParen = &tokens[idx] {
            let cond_res = evaluate_media_condition(&tokens[idx..], viewport_w).unwrap_or(false);
            matches = cond_res;
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

            if idx < tokens.len() {
                if is_ident(&tokens[idx], "and") {
                    idx += 1;
                    let cond_res =
                        evaluate_media_condition(&tokens[idx..], viewport_w).unwrap_or(false);
                    if !cond_res {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }
        } else {
            matches = false;
        }
    }

    if is_negated { !matches } else { matches }
}

/// Parses a CSS `<ratio>` from a slice of tokens.
fn parse_ratio(tokens: &[CssToken]) -> Option<f32> {
    if tokens.is_empty() {
        return None;
    }
    match tokens.len() {
        1 => match &tokens[0] {
            CssToken::Number(val) if *val > 0.0 => Some(*val as f32),
            _ => None,
        },
        3 => match (&tokens[0], &tokens[1], &tokens[2]) {
            (CssToken::Number(num), CssToken::Delim('/'), CssToken::Number(den))
                if *num > 0.0 && *den > 0.0 =>
            {
                Some((*num / *den) as f32)
            }
            _ => None,
        },
        _ => None,
    }
}

/// Parses a CSS `<resolution>` from a slice of tokens.
fn parse_resolution(tokens: &[CssToken]) -> Option<f32> {
    if tokens.len() != 1 {
        return None;
    }
    match &tokens[0] {
        CssToken::Dimension { value, unit } => {
            let unit_lower = unit.to_ascii_lowercase();
            match unit_lower.as_str() {
                "dppx" | "x" => Some(*value as f32),
                "dpi" => Some(*value as f32 / 96.0),
                "dpcm" => Some(*value as f32 / (96.0 / 2.54)),
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Lt,  // <
    Lte, // <=
    Gt,  // >
    Gte, // >=
    Eq,  // =
}

fn parse_op(tokens: &[CssToken], start: usize) -> Option<(Op, usize)> {
    if start >= tokens.len() {
        return None;
    }
    match &tokens[start] {
        CssToken::Delim('<') => {
            if start + 1 < tokens.len() && matches!(&tokens[start + 1], CssToken::Delim('=')) {
                return Some((Op::Lte, 2));
            }
            Some((Op::Lt, 1))
        }
        CssToken::Delim('>') => {
            if start + 1 < tokens.len() && matches!(&tokens[start + 1], CssToken::Delim('=')) {
                return Some((Op::Gte, 2));
            }
            Some((Op::Gt, 1))
        }
        CssToken::Delim('=') => Some((Op::Eq, 1)),
        _ => None,
    }
}

fn find_operators(tokens: &[CssToken]) -> Vec<(Op, usize, usize)> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Some((op, len)) = parse_op(tokens, i) {
            ops.push((op, i, len));
            i += len;
        } else {
            i += 1;
        }
    }
    ops
}

#[derive(Debug, Clone, PartialEq)]
enum FeatureValue {
    Length(f32),
    Ratio(f32),
    Number(f32),
    Resolution(f32),
}

fn resolve_length_unit(value: f32, unit: &str, viewport_w: f32) -> Option<f32> {
    let unit_lower = unit.to_ascii_lowercase();
    match unit_lower.as_str() {
        "px" => Some(value),
        "em" | "rem" => Some(value * 16.0),
        "in" => Some(value * 96.0),
        "cm" => Some(value * (96.0 / 2.54)),
        "mm" => Some(value * (9.6 / 2.54)),
        "pt" => Some(value * (96.0 / 72.0)),
        "pc" => Some(value * 16.0),
        "vw" => Some(value * viewport_w / 100.0),
        "vh" => Some(value * viewport_h() / 100.0),
        "vmin" => Some(value * f32::min(viewport_w, viewport_h()) / 100.0),
        "vmax" => Some(value * f32::max(viewport_w, viewport_h()) / 100.0),
        _ => None,
    }
}

fn parse_length_val(tokens: &[CssToken], viewport_w: f32) -> Option<f32> {
    if tokens.len() != 1 {
        return None;
    }
    match &tokens[0] {
        CssToken::Dimension { value, unit } => resolve_length_unit(*value as f32, unit, viewport_w),
        CssToken::Number(value) => Some(*value as f32),
        _ => None,
    }
}

fn parse_number_val(tokens: &[CssToken]) -> Option<f32> {
    if tokens.len() != 1 {
        return None;
    }
    if let CssToken::Number(value) = &tokens[0] {
        Some(*value as f32)
    } else {
        None
    }
}

fn parse_resolution_val(tokens: &[CssToken]) -> Option<f32> {
    if tokens.is_empty() {
        return None;
    }
    if let Some(res) = parse_resolution(tokens) {
        return Some(res);
    }
    if let Some(ratio) = parse_ratio(tokens) {
        return Some(ratio);
    }
    if let [CssToken::Number(val)] = tokens {
        return Some(*val as f32);
    }
    None
}

fn parse_feature_value(
    kind: &FeatureValue,
    tokens: &[CssToken],
    viewport_w: f32,
) -> Option<FeatureValue> {
    match kind {
        FeatureValue::Length(_) => parse_length_val(tokens, viewport_w).map(FeatureValue::Length),
        FeatureValue::Ratio(_) => parse_ratio(tokens).map(FeatureValue::Ratio),
        FeatureValue::Number(_) => parse_number_val(tokens).map(FeatureValue::Number),
        FeatureValue::Resolution(_) => parse_resolution_val(tokens).map(FeatureValue::Resolution),
    }
}

fn get_range_feature_value(name: &str, viewport_w: f32) -> Option<FeatureValue> {
    let clean_name = if let Some(stripped) = name.strip_prefix("min-") {
        stripped
    } else if let Some(stripped) = name.strip_prefix("max-") {
        stripped
    } else if name.starts_with("-webkit-min-") || name.starts_with("-webkit-max-") {
        "-webkit-device-pixel-ratio"
    } else {
        name
    };

    match clean_name {
        "width" => Some(FeatureValue::Length(viewport_w)),
        "height" => Some(FeatureValue::Length(viewport_h())),
        "device-width" => Some(FeatureValue::Length(device_width())),
        "device-height" => Some(FeatureValue::Length(device_height())),
        "aspect-ratio" => {
            let ratio = if viewport_h() > 0.0 {
                viewport_w / viewport_h()
            } else {
                0.0
            };
            Some(FeatureValue::Ratio(ratio))
        }
        "device-aspect-ratio" => {
            let ratio = if device_height() > 0.0 {
                device_width() / device_height()
            } else {
                0.0
            };
            Some(FeatureValue::Ratio(ratio))
        }
        "color" => Some(FeatureValue::Number(8.0)),
        "color-index" => Some(FeatureValue::Number(0.0)),
        "monochrome" => Some(FeatureValue::Number(0.0)),
        "grid" => Some(FeatureValue::Number(0.0)),
        "resolution" | "-webkit-device-pixel-ratio" | "device-pixel-ratio" => {
            Some(FeatureValue::Resolution(device_pixel_ratio()))
        }
        "horizontal-viewport-segments" => {
            Some(FeatureValue::Number(horizontal_viewport_segments() as f32))
        }
        "vertical-viewport-segments" => {
            Some(FeatureValue::Number(vertical_viewport_segments() as f32))
        }
        _ => None,
    }
}

fn compare_values(curr: FeatureValue, op: Op, target: FeatureValue) -> bool {
    match (curr, target) {
        (FeatureValue::Length(c), FeatureValue::Length(t)) => match op {
            Op::Lt => c < t,
            Op::Lte => c <= t,
            Op::Gt => c > t,
            Op::Gte => c >= t,
            Op::Eq => (c - t).abs() < 1e-5,
        },
        (FeatureValue::Ratio(c), FeatureValue::Ratio(t)) => match op {
            Op::Lt => c < t - 1e-5,
            Op::Lte => c <= t + 1e-5,
            Op::Gt => c > t + 1e-5,
            Op::Gte => c >= t - 1e-5,
            Op::Eq => (c - t).abs() < 1e-5,
        },
        (FeatureValue::Number(c), FeatureValue::Number(t)) => match op {
            Op::Lt => c < t,
            Op::Lte => c <= t,
            Op::Gt => c > t,
            Op::Gte => c >= t,
            Op::Eq => (c - t).abs() < 1e-5,
        },
        (FeatureValue::Resolution(c), FeatureValue::Resolution(t)) => match op {
            Op::Lt => c < t,
            Op::Lte => c <= t,
            Op::Gt => c > t,
            Op::Gte => c >= t,
            Op::Eq => (c - t).abs() < 1e-5,
        },
        _ => false,
    }
}

fn evaluate_range_query(ops: &[(Op, usize, usize)], tokens: &[CssToken], viewport_w: f32) -> bool {
    if ops.len() == 1 {
        let (op, op_idx, op_len) = ops[0];
        // Form 1: <mf-name> <op> <value>
        if let (1, CssToken::Ident(name)) = (op_idx, &tokens[0]) {
            let feature_name = name.to_ascii_lowercase();
            let value_tokens = &tokens[op_idx + op_len..];
            if let Some(kind) = get_range_feature_value(&feature_name, viewport_w)
                && let Some(target_val) = parse_feature_value(&kind, value_tokens, viewport_w)
            {
                return compare_values(kind, op, target_val);
            }
        }
        // Form 2: <value> <op> <mf-name>
        if let (true, CssToken::Ident(name)) = (
            op_idx + op_len + 1 == tokens.len(),
            &tokens[op_idx + op_len],
        ) {
            let feature_name = name.to_ascii_lowercase();
            let value_tokens = &tokens[0..op_idx];
            if let Some(kind) = get_range_feature_value(&feature_name, viewport_w)
                && let Some(target_val) = parse_feature_value(&kind, value_tokens, viewport_w)
            {
                return compare_values(target_val, op, kind);
            }
        }
    } else if ops.len() == 2 {
        let (op1, op1_idx, op1_len) = ops[0];
        let (op2, op2_idx, op2_len) = ops[1];
        // Form 3: <value> <op1> <mf-name> <op2> <value>
        if let (true, CssToken::Ident(name)) =
            (op2_idx == op1_idx + op1_len + 1, &tokens[op1_idx + op1_len])
        {
            let feature_name = name.to_ascii_lowercase();
            let value1_tokens = &tokens[0..op1_idx];
            let value2_tokens = &tokens[op2_idx + op2_len..];
            // Check direction
            let is_lt = (op1 == Op::Lt || op1 == Op::Lte) && (op2 == Op::Lt || op2 == Op::Lte);
            let is_gt = (op1 == Op::Gt || op1 == Op::Gte) && (op2 == Op::Gt || op2 == Op::Gte);
            let kind_opt = if is_lt || is_gt {
                get_range_feature_value(&feature_name, viewport_w)
            } else {
                None
            };
            if let Some(kind) = kind_opt {
                let val1 = parse_feature_value(&kind, value1_tokens, viewport_w);
                let val2 = parse_feature_value(&kind, value2_tokens, viewport_w);
                if let (Some(v1), Some(v2)) = (val1, val2) {
                    return compare_values(v1, op1, kind.clone()) && compare_values(kind, op2, v2);
                }
            }
        }
    }
    false
}

/// Evaluates a single media feature, e.g., max-width: 600px.
fn evaluate_feature(tokens: &[CssToken], viewport_w: f32) -> bool {
    if tokens.is_empty() {
        return false;
    }

    // 1. Check for range query operators first, since range queries don't necessarily start with an Ident.
    let ops = find_operators(tokens);
    if !ops.is_empty() {
        return evaluate_range_query(&ops, tokens, viewport_w);
    }

    // 2. Otherwise, fall back to the existing colon-based or boolean queries.
    let feature_name = if let CssToken::Ident(name) = &tokens[0] {
        name.to_ascii_lowercase()
    } else {
        return false;
    };

    if tokens.len() == 1 {
        match feature_name.as_str() {
            "prefers-color-scheme" => return true,
            "prefers-reduced-motion" => return true,
            "prefers-contrast" => return true,
            "prefers-reduced-data" => return true,
            "prefers-reduced-transparency" => return true,
            "forced-colors" => return forced_colors() != ForcedColors::None,
            "inverted-colors" => return inverted_colors() != InvertedColors::None,
            "update" => return update_mode() != UpdateMode::None,
            "scripting" => return scripting() != Scripting::None,
            "hover" => return hover() != Hover::None,
            "any-hover" => return any_hover() != Hover::None,
            "pointer" => return pointer() != Pointer::None,
            "any-pointer" => return any_pointer() != Pointer::None,
            "color-gamut" => return true,
            "display-mode" => return true,
            "overflow-block" => return true,
            "overflow-inline" => return true,
            "dynamic-range" => return true,
            "video-dynamic-range" => return true,
            "environment-blending" => return true,
            "light-level" => return true,
            "device-posture" => return true,
            "nav-controls" => return nav_controls() != NavControls::None,
            "video-color-gamut" => return true,
            "shape" => return true,
            "display-shape" => return true,
            "horizontal-viewport-segments" => return horizontal_viewport_segments() != 0,
            "vertical-viewport-segments" => return vertical_viewport_segments() != 0,
            "orientation" => return true,
            "monochrome" => return false,
            "grid" => return false,
            "scan" => return true,
            "width" => return true,
            "min-width" => return true,
            "max-width" => return true,
            "height" => return true,
            "min-height" => return true,
            "max-height" => return true,
            "aspect-ratio" => return true,
            "min-aspect-ratio" => return true,
            "max-aspect-ratio" => return true,
            "color" => return true,
            "min-color" => return true,
            "max-color" => return true,
            "color-index" => return false,
            "min-color-index" => return true,
            "max-color-index" => return true,
            "resolution" => return true,
            "min-resolution" => return true,
            "max-resolution" => return true,
            "device-pixel-ratio" => return true,
            "min-device-pixel-ratio" => return true,
            "max-device-pixel-ratio" => return true,
            "-webkit-device-pixel-ratio" => return true,
            "-webkit-min-device-pixel-ratio" => return true,
            "-webkit-max-device-pixel-ratio" => return true,
            "device-width" => return true,
            "min-device-width" => return true,
            "max-device-width" => return true,
            "device-height" => return true,
            "min-device-height" => return true,
            "max-device-height" => return true,
            "device-aspect-ratio" => return true,
            "min-device-aspect-ratio" => return true,
            "max-device-aspect-ratio" => return true,
            _ => return false,
        }
    }

    if tokens.len() < 3 {
        return false;
    }

    if !matches!(tokens[1], CssToken::Colon) {
        return false;
    }

    if feature_name == "prefers-color-scheme" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = preferred_color_scheme();
            match (current, val_lower.as_str()) {
                (ColorScheme::Light, "light") => return true,
                (ColorScheme::Dark, "dark") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "prefers-reduced-motion" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = prefers_reduced_motion();
            match (current, val_lower.as_str()) {
                (PrefersReducedMotion::NoPreference, "no-preference") => return true,
                (PrefersReducedMotion::Reduce, "reduce") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "prefers-contrast" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = prefers_contrast();
            match (current, val_lower.as_str()) {
                (PrefersContrast::NoPreference, "no-preference") => return true,
                (PrefersContrast::More, "more") => return true,
                (PrefersContrast::Less, "less") => return true,
                (PrefersContrast::Custom, "custom") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "prefers-reduced-data" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = prefers_reduced_data();
            match (current, val_lower.as_str()) {
                (PrefersReducedData::NoPreference, "no-preference") => return true,
                (PrefersReducedData::Reduce, "reduce") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "prefers-reduced-transparency" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = prefers_reduced_transparency();
            match (current, val_lower.as_str()) {
                (PrefersReducedTransparency::NoPreference, "no-preference") => return true,
                (PrefersReducedTransparency::Reduce, "reduce") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "forced-colors" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = forced_colors();
            match (current, val_lower.as_str()) {
                (ForcedColors::None, "none") => return true,
                (ForcedColors::Active, "active") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "inverted-colors" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = inverted_colors();
            match (current, val_lower.as_str()) {
                (InvertedColors::None, "none") => return true,
                (InvertedColors::Inverted, "inverted") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "update" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = update_mode();
            match (current, val_lower.as_str()) {
                (UpdateMode::None, "none") => return true,
                (UpdateMode::Slow, "slow") => return true,
                (UpdateMode::Fast, "fast") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "scripting" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = scripting();
            match (current, val_lower.as_str()) {
                (Scripting::None, "none") => return true,
                (Scripting::InitialOnly, "initial-only") => return true,
                (Scripting::Enabled, "enabled") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "hover" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = hover();
            match (current, val_lower.as_str()) {
                (Hover::None, "none") => return true,
                (Hover::Hover, "hover") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "any-hover" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = any_hover();
            match (current, val_lower.as_str()) {
                (Hover::None, "none") => return true,
                (Hover::Hover, "hover") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "pointer" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = pointer();
            match (current, val_lower.as_str()) {
                (Pointer::None, "none") => return true,
                (Pointer::Coarse, "coarse") => return true,
                (Pointer::Fine, "fine") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "any-pointer" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = any_pointer();
            match (current, val_lower.as_str()) {
                (Pointer::None, "none") => return true,
                (Pointer::Coarse, "coarse") => return true,
                (Pointer::Fine, "fine") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "color-gamut" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = color_gamut();
            match val_lower.as_str() {
                "srgb" => return true,
                "p3" => return matches!(current, ColorGamut::P3 | ColorGamut::Rec2020),
                "rec2020" => return matches!(current, ColorGamut::Rec2020),
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "display-mode" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = display_mode();
            match (current, val_lower.as_str()) {
                (DisplayMode::Fullscreen, "fullscreen") => return true,
                (DisplayMode::Standalone, "standalone") => return true,
                (DisplayMode::MinimalUi, "minimal-ui") => return true,
                (DisplayMode::Browser, "browser") => return true,
                (DisplayMode::WindowControlsOverlay, "window-controls-overlay") => return true,
                (DisplayMode::PictureInPicture, "picture-in-picture") => return true,
                (DisplayMode::Tabbed, "tabbed") => return true,
                (DisplayMode::Borderless, "borderless") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "overflow-block" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = overflow_block();
            match (current, val_lower.as_str()) {
                (OverflowBlock::None, "none") => return true,
                (OverflowBlock::Scroll, "scroll") => return true,
                (OverflowBlock::Paged, "paged") => return true,
                (OverflowBlock::OptionalPaged, "optional-paged") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "overflow-inline" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = overflow_inline();
            match (current, val_lower.as_str()) {
                (OverflowInline::None, "none") => return true,
                (OverflowInline::Scroll, "scroll") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "dynamic-range" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = dynamic_range();
            match val_lower.as_str() {
                "standard" => return true,
                "high" => return matches!(current, DynamicRange::High),
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "video-dynamic-range" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = video_dynamic_range();
            match val_lower.as_str() {
                "standard" => return true,
                "high" => return matches!(current, DynamicRange::High),
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "environment-blending" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = environment_blending();
            match (current, val_lower.as_str()) {
                (EnvironmentBlending::Opaque, "opaque") => return true,
                (EnvironmentBlending::Additive, "additive") => return true,
                (EnvironmentBlending::Subtractive, "subtractive") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "light-level" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = light_level();
            match (current, val_lower.as_str()) {
                (LightLevel::Dim, "dim") => return true,
                (LightLevel::Normal, "normal") => return true,
                (LightLevel::Washed, "washed") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "device-posture" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = device_posture();
            match (current, val_lower.as_str()) {
                (DevicePosture::Continuous, "continuous") => return true,
                (DevicePosture::Folded, "folded") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "nav-controls" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = nav_controls();
            match (current, val_lower.as_str()) {
                (NavControls::None, "none") => return true,
                (NavControls::Back, "back") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "video-color-gamut" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = video_color_gamut();
            match val_lower.as_str() {
                "srgb" => return true,
                "p3" => return matches!(current, ColorGamut::P3 | ColorGamut::Rec2020),
                "rec2020" => return matches!(current, ColorGamut::Rec2020),
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "shape" || feature_name == "display-shape" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let current = display_shape();
            match (current, val_lower.as_str()) {
                (DisplayShape::Rect, "rect") => return true,
                (DisplayShape::Round, "round") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "horizontal-viewport-segments"
        || feature_name == "min-horizontal-viewport-segments"
        || feature_name == "max-horizontal-viewport-segments"
    {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = horizontal_viewport_segments();
            match feature_name.as_str() {
                "horizontal-viewport-segments" => return current == limit,
                "min-horizontal-viewport-segments" => return current >= limit,
                "max-horizontal-viewport-segments" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "vertical-viewport-segments"
        || feature_name == "min-vertical-viewport-segments"
        || feature_name == "max-vertical-viewport-segments"
    {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = vertical_viewport_segments();
            match feature_name.as_str() {
                "vertical-viewport-segments" => return current == limit,
                "min-vertical-viewport-segments" => return current >= limit,
                "max-vertical-viewport-segments" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "orientation" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            let is_portrait = viewport_h() >= viewport_w;
            match (is_portrait, val_lower.as_str()) {
                (true, "portrait") => return true,
                (false, "landscape") => return true,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "monochrome"
        || feature_name == "min-monochrome"
        || feature_name == "max-monochrome"
    {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = 0; // On color display, monochrome is 0
            match feature_name.as_str() {
                "monochrome" => return current == limit,
                "min-monochrome" => return current >= limit,
                "max-monochrome" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "grid" || feature_name == "min-grid" || feature_name == "max-grid" {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = 0; // bitmap display is 0
            match feature_name.as_str() {
                "grid" => return current == limit,
                "min-grid" => return current >= limit,
                "max-grid" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "scan" {
        if let CssToken::Ident(val) = &tokens[2] {
            let val_lower = val.to_ascii_lowercase();
            match val_lower.as_str() {
                "progressive" => return true,
                "interlace" => return false,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "aspect-ratio"
        || feature_name == "min-aspect-ratio"
        || feature_name == "max-aspect-ratio"
    {
        if let Some(target_ratio) = parse_ratio(&tokens[2..]) {
            let current_ratio = if viewport_h() > 0.0 {
                viewport_w / viewport_h()
            } else {
                0.0
            };
            match feature_name.as_str() {
                "aspect-ratio" => return (current_ratio - target_ratio).abs() < 1e-5,
                "min-aspect-ratio" => return current_ratio >= target_ratio - 1e-5,
                "max-aspect-ratio" => return current_ratio <= target_ratio + 1e-5,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "device-aspect-ratio"
        || feature_name == "min-device-aspect-ratio"
        || feature_name == "max-device-aspect-ratio"
    {
        if let Some(target_ratio) = parse_ratio(&tokens[2..]) {
            let current_ratio = if device_height() > 0.0 {
                device_width() / device_height()
            } else {
                0.0
            };
            match feature_name.as_str() {
                "device-aspect-ratio" => return (current_ratio - target_ratio).abs() < 1e-5,
                "min-device-aspect-ratio" => return current_ratio >= target_ratio - 1e-5,
                "max-device-aspect-ratio" => return current_ratio <= target_ratio + 1e-5,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "color" || feature_name == "min-color" || feature_name == "max-color" {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = 8; // standard 8-bit color depth per color component
            match feature_name.as_str() {
                "color" => return current == limit,
                "min-color" => return current >= limit,
                "max-color" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "color-index"
        || feature_name == "min-color-index"
        || feature_name == "max-color-index"
    {
        if let CssToken::Number(val) = &tokens[2] {
            let limit = *val as i32;
            let current = 0; // standard displays do not use color lookup tables
            match feature_name.as_str() {
                "color-index" => return current == limit,
                "min-color-index" => return current >= limit,
                "max-color-index" => return current <= limit,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "resolution"
        || feature_name == "min-resolution"
        || feature_name == "max-resolution"
    {
        if let Some(target_res) = parse_resolution(&tokens[2..]) {
            let current_res = device_pixel_ratio();
            match feature_name.as_str() {
                "resolution" => return (current_res - target_res).abs() < 1e-5,
                "min-resolution" => return current_res >= target_res - 1e-5,
                "max-resolution" => return current_res <= target_res + 1e-5,
                _ => return false,
            }
        }
        return false;
    }

    if feature_name == "device-pixel-ratio"
        || feature_name == "min-device-pixel-ratio"
        || feature_name == "max-device-pixel-ratio"
        || feature_name == "-webkit-device-pixel-ratio"
        || feature_name == "-webkit-min-device-pixel-ratio"
        || feature_name == "-webkit-max-device-pixel-ratio"
    {
        if let Some(target_dpr) = parse_ratio(&tokens[2..]) {
            let current_dpr = device_pixel_ratio();
            match feature_name.as_str() {
                "device-pixel-ratio" | "-webkit-device-pixel-ratio" => {
                    return (current_dpr - target_dpr).abs() < 1e-5;
                }
                "min-device-pixel-ratio" | "-webkit-min-device-pixel-ratio" => {
                    return current_dpr >= target_dpr - 1e-5;
                }
                "max-device-pixel-ratio" | "-webkit-max-device-pixel-ratio" => {
                    return current_dpr <= target_dpr + 1e-5;
                }
                _ => return false,
            }
        }
        return false;
    }

    let value_px = match &tokens[2] {
        CssToken::Dimension { value, unit } => resolve_length_unit(*value as f32, unit, viewport_w),
        CssToken::Number(value) => Some(*value as f32),
        _ => None,
    };

    match feature_name.as_str() {
        "min-width" => value_px.is_some_and(|limit| viewport_w >= limit),
        "max-width" => value_px.is_some_and(|limit| viewport_w <= limit),
        "width" => value_px.is_some_and(|limit| (viewport_w - limit).abs() < 1e-5),
        "min-height" => value_px.is_some_and(|limit| viewport_h() >= limit),
        "max-height" => value_px.is_some_and(|limit| viewport_h() <= limit),
        "height" => value_px.is_some_and(|limit| (viewport_h() - limit).abs() < 1e-5),
        "min-device-width" => value_px.is_some_and(|limit| device_width() >= limit),
        "max-device-width" => value_px.is_some_and(|limit| device_width() <= limit),
        "device-width" => value_px.is_some_and(|limit| (device_width() - limit).abs() < 1e-5),
        "min-device-height" => value_px.is_some_and(|limit| device_height() >= limit),
        "max-device-height" => value_px.is_some_and(|limit| device_height() <= limit),
        "device-height" => value_px.is_some_and(|limit| (device_height() - limit).abs() < 1e-5),
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

/// Helper to filter out top-level whitespace from a list of component values.
fn clean_values(values: &[ComponentValue]) -> Vec<&ComponentValue> {
    values
        .iter()
        .filter(|v| !matches!(v, ComponentValue::Token(CssToken::Whitespace)))
        .collect()
}

/// Helper to check if a component value is a case-insensitive identifier matching `keyword`.
fn is_keyword(cv: &ComponentValue, keyword: &str) -> bool {
    if let ComponentValue::Token(t) = cv {
        is_ident(t, keyword)
    } else {
        false
    }
}

/// Splits a slice of component values on a top-level keyword (like "and" or "or").
fn split_by_keyword<'a>(
    values: &[&'a ComponentValue],
    keyword: &str,
) -> Vec<Vec<&'a ComponentValue>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    for &cv in values {
        if is_keyword(cv, keyword) {
            parts.push(std::mem::take(&mut current));
        } else {
            current.push(cv);
        }
    }
    parts.push(current);
    parts
}

/// Evaluates a CSS `@supports` condition (represented as a slice of `ComponentValue` pointers).
fn evaluate_supports_condition(values: &[&ComponentValue]) -> bool {
    if values.is_empty() {
        return false;
    }

    // 1. Check for 'and' and 'or' combinators at this level
    let mut has_and = false;
    let mut has_or = false;
    for &cv in values {
        if is_keyword(cv, "and") {
            has_and = true;
        }
        if is_keyword(cv, "or") {
            has_or = true;
        }
    }

    if has_and && has_or {
        // Mixing 'and' and 'or' without parentheses is invalid
        return false;
    }

    if has_and {
        let parts = split_by_keyword(values, "and");
        for part in &parts {
            if part.is_empty() {
                return false;
            }
        }
        return parts.iter().all(|part| evaluate_supports_condition(part));
    }

    if has_or {
        let parts = split_by_keyword(values, "or");
        for part in &parts {
            if part.is_empty() {
                return false;
            }
        }
        return parts.iter().any(|part| evaluate_supports_condition(part));
    }

    // 2. Check for negation
    if is_keyword(values[0], "not") {
        if values.len() < 2 {
            return false;
        }
        // Negate the evaluation of the rest of the condition
        return !evaluate_supports_condition(&values[1..]);
    }

    // 3. Single `<supports-in-parens>` operand
    if values.len() == 1 {
        if let ComponentValue::SimpleBlock {
            associated: '(',
            value: inner_values,
        } = values[0]
        {
            // Find the first top-level colon inside this parenthesis block.
            if let Some(colon_idx) = inner_values
                .iter()
                .position(|cv| matches!(cv, ComponentValue::Token(CssToken::Colon)))
            {
                // Before colon: property name (must be exactly one Ident, ignoring Whitespace)
                let before_colon = &inner_values[..colon_idx];
                let name_tokens: Vec<&ComponentValue> = before_colon
                    .iter()
                    .filter(|cv| !matches!(cv, ComponentValue::Token(CssToken::Whitespace)))
                    .collect();

                if name_tokens.len() == 1
                    && let ComponentValue::Token(CssToken::Ident(prop_name)) = name_tokens[0]
                {
                    let name = prop_name.trim();
                    if name.is_empty() {
                        return false;
                    }
                    let is_recognized = crate::css::property::lookup(name).is_some()
                        || crate::css::property::shorthand_longhands(name).is_some();
                    if !is_recognized {
                        return false;
                    }

                    // After colon: value (keep inner whitespace, trim leading/trailing whitespace)
                    let after_colon = &inner_values[colon_idx + 1..];
                    let mut start = 0;
                    while start < after_colon.len()
                        && matches!(
                            after_colon[start],
                            ComponentValue::Token(CssToken::Whitespace)
                        )
                    {
                        start += 1;
                    }
                    let mut end = after_colon.len();
                    while end > start
                        && matches!(
                            after_colon[end - 1],
                            ComponentValue::Token(CssToken::Whitespace)
                        )
                    {
                        end -= 1;
                    }

                    let val_components: Vec<ComponentValue> = after_colon[start..end].to_vec();
                    return crate::css::values::parse_property_value(name, &val_components)
                        .is_some();
                }
            }

            // Otherwise, it must be a nested `<supports-condition>`
            let cleaned_inner = clean_values(inner_values);
            return evaluate_supports_condition(&cleaned_inner);
        } else {
            // TODO(spec): selector(...) / font-tech(...) / font-format(...) / other general-enclosed
            return false;
        }
    }

    false
}

/// Evaluates a supports condition string.
pub fn supports_condition_matches(condition: &str) -> bool {
    let components = crate::css::parser::parse_component_values(condition);
    let cleaned = clean_values(&components);
    evaluate_supports_condition(&cleaned)
}

/// Hostile stylesheets can nest @media rules arbitrarily deep.
/// To prevent resource exhaustion and stack overflow, we restrict depth.
const MAX_MEDIA_NEST_DEPTH: usize = 32;

enum RulesSource<'a> {
    Borrowed(&'a [Rule]),
    Owned(Vec<Rule>),
}

impl<'a> RulesSource<'a> {
    fn as_slice(&self) -> &[Rule] {
        match self {
            RulesSource::Borrowed(s) => s,
            RulesSource::Owned(v) => v,
        }
    }
}

struct Frame<'a> {
    rules: RulesSource<'a>,
    index: usize,
    depth: usize,
}

/// Iteratively extracts all active qualified rules from a stylesheet under the given viewport width.
pub fn extract_matched_rules(stylesheet: &Stylesheet, viewport_w: f32) -> Vec<QualifiedRule> {
    let mut matched = Vec::new();
    let mut stack = vec![Frame {
        rules: RulesSource::Borrowed(&stylesheet.rules),
        index: 0,
        depth: 0,
    }];

    while let Some(frame) = stack.last_mut() {
        let rules_slice = frame.rules.as_slice();
        if frame.index >= rules_slice.len() {
            stack.pop();
            continue;
        }

        let rule = &rules_slice[frame.index];
        frame.index += 1;

        match rule {
            Rule::Qualified(qualified) => {
                matched.push(qualified.clone());
            }
            Rule::At(at_rule) if at_rule.name == "media" => {
                let query_str = serialize_component_values(&at_rule.prelude);
                if media_matches(&query_str, viewport_w)
                    && let Some(block) = &at_rule.block
                {
                    let next_depth = frame.depth + 1;
                    if next_depth > MAX_MEDIA_NEST_DEPTH {
                        eprintln!(
                            "css: @media nesting exceeded {MAX_MEDIA_NEST_DEPTH}, skipping deeper rules"
                        );
                        continue;
                    }

                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    stack.push(Frame {
                        rules: RulesSource::Owned(inner_stylesheet.rules),
                        index: 0,
                        depth: next_depth,
                    });
                }
            }
            Rule::At(at_rule) if at_rule.name == "supports" => {
                let cleaned_prelude = clean_values(&at_rule.prelude);
                if evaluate_supports_condition(&cleaned_prelude)
                    && let Some(block) = &at_rule.block
                {
                    let next_depth = frame.depth + 1;
                    if next_depth > MAX_MEDIA_NEST_DEPTH {
                        eprintln!(
                            "css: @supports nesting exceeded {MAX_MEDIA_NEST_DEPTH}, skipping deeper rules"
                        );
                        continue;
                    }

                    let inner_css = serialize_component_values(block);
                    let inner_stylesheet = crate::css::parser::parse_stylesheet(&inner_css);
                    stack.push(Frame {
                        rules: RulesSource::Owned(inner_stylesheet.rules),
                        index: 0,
                        depth: next_depth,
                    });
                }
            }
            _ => {
                // Ignore other at-rules
            }
        }
    }

    matched
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

    #[test]
    fn test_extract_nested_media_preserves_order() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            .top-start { color: red; }
            @media (min-width: 1px) {
                .inner-1 { color: green; }
                .inner-2 { color: blue; }
            }
            .top-end { color: yellow; }
        ",
        );

        let matched = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched.len(), 4);
        assert_eq!(
            serialize_component_values(&matched[0].prelude),
            ".top-start "
        );
        assert_eq!(serialize_component_values(&matched[1].prelude), ".inner-1 ");
        assert_eq!(serialize_component_values(&matched[2].prelude), ".inner-2 ");
        assert_eq!(serialize_component_values(&matched[3].prelude), ".top-end ");
    }

    #[test]
    fn test_extract_unmatched_media_skipped() {
        let stylesheet = crate::css::parser::parse_stylesheet(
            "
            @media (min-width: 1px) {
                .outer { color: red; }
                @media (min-width: 1000px) {
                    .inner { color: green; }
                }
            }
        ",
        );

        let matched = extract_matched_rules(&stylesheet, 500.0);
        assert_eq!(matched.len(), 1);
        assert_eq!(serialize_component_values(&matched[0].prelude), ".outer ");
    }

    #[test]
    fn test_extract_deeply_nested_media_no_overflow() {
        let mut css = String::new();
        for _ in 0..2000 {
            css.push_str("@media (min-width: 1px) { ");
        }
        css.push_str(".deepest { color: red; }");
        for _ in 0..2000 {
            css.push('}');
        }

        let stylesheet = crate::css::parser::parse_stylesheet(&css);
        let matched = extract_matched_rules(&stylesheet, 500.0);
        // Assert it successfully executed without stack overflow.
        // It might be empty or some rules depending on depth guard, which is correct.
        let _ = matched;
    }

    #[test]
    fn test_prefers_color_scheme_default() {
        // Default is light
        assert!(media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(!media_matches("(prefers-color-scheme: dark)", 1000.0));
        // Boolean context
        assert!(media_matches("(prefers-color-scheme)", 1000.0));
    }

    #[test]
    fn test_prefers_color_scheme_configured() {
        // Set to dark
        set_preferred_color_scheme(ColorScheme::Dark);
        assert!(!media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(media_matches("(prefers-color-scheme: dark)", 1000.0));
        assert!(media_matches("(prefers-color-scheme)", 1000.0));

        // Set back to light
        set_preferred_color_scheme(ColorScheme::Light);
        assert!(media_matches("(prefers-color-scheme: light)", 1000.0));
        assert!(!media_matches("(prefers-color-scheme: dark)", 1000.0));
        assert!(media_matches("(prefers-color-scheme)", 1000.0));
    }

    #[test]
    fn test_prefers_color_scheme_case_insensitive() {
        set_preferred_color_scheme(ColorScheme::Dark);
        assert!(media_matches("(PREFERS-COLOR-SCHEME: DaRk)", 1000.0));
        assert!(!media_matches("(PREFERS-COLOR-SCHEME: LiGhT)", 1000.0));
        // Reset to default
        set_preferred_color_scheme(ColorScheme::Light);
    }

    #[test]
    fn test_prefers_reduced_motion() {
        // Default: no-preference
        assert!(media_matches(
            "(prefers-reduced-motion: no-preference)",
            1000.0
        ));
        assert!(!media_matches("(prefers-reduced-motion: reduce)", 1000.0));
        assert!(media_matches("(prefers-reduced-motion)", 1000.0));

        // Configure: reduce
        set_prefers_reduced_motion(PrefersReducedMotion::Reduce);
        assert!(!media_matches(
            "(prefers-reduced-motion: no-preference)",
            1000.0
        ));
        assert!(media_matches("(prefers-reduced-motion: reduce)", 1000.0));
        assert!(media_matches("(prefers-reduced-motion)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(prefers-reduced-motion: unknown)", 1000.0));
        assert!(!media_matches("(prefers-reduced-motion-wrong)", 1000.0));

        // Case insensitivity
        assert!(media_matches("(PREFERS-REDUCED-MOTION: ReDuCe)", 1000.0));

        // Reset
        set_prefers_reduced_motion(PrefersReducedMotion::NoPreference);
    }

    #[test]
    fn test_prefers_contrast() {
        // Default: no-preference
        assert!(media_matches("(prefers-contrast: no-preference)", 1000.0));
        assert!(!media_matches("(prefers-contrast: more)", 1000.0));
        assert!(media_matches("(prefers-contrast)", 1000.0));

        // Configure: more
        set_prefers_contrast(PrefersContrast::More);
        assert!(!media_matches("(prefers-contrast: no-preference)", 1000.0));
        assert!(media_matches("(prefers-contrast: more)", 1000.0));
        assert!(media_matches("(prefers-contrast)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(prefers-contrast: unknown)", 1000.0));

        // Case insensitivity
        assert!(media_matches("(PREFERS-CONTRAST: MoRe)", 1000.0));

        // Reset
        set_prefers_contrast(PrefersContrast::NoPreference);
    }

    #[test]
    fn test_prefers_reduced_data() {
        // Default: no-preference
        assert!(media_matches(
            "(prefers-reduced-data: no-preference)",
            1000.0
        ));
        assert!(!media_matches("(prefers-reduced-data: reduce)", 1000.0));
        assert!(media_matches("(prefers-reduced-data)", 1000.0));

        // Configure: reduce
        set_prefers_reduced_data(PrefersReducedData::Reduce);
        assert!(!media_matches(
            "(prefers-reduced-data: no-preference)",
            1000.0
        ));
        assert!(media_matches("(prefers-reduced-data: reduce)", 1000.0));
        assert!(media_matches("(prefers-reduced-data)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(prefers-reduced-data: unknown)", 1000.0));

        // Case insensitivity
        assert!(media_matches("(PREFERS-REDUCED-DATA: ReDuCe)", 1000.0));

        // Reset
        set_prefers_reduced_data(PrefersReducedData::NoPreference);
    }

    #[test]
    fn test_prefers_reduced_transparency() {
        // Default: no-preference
        assert!(media_matches(
            "(prefers-reduced-transparency: no-preference)",
            1000.0
        ));
        assert!(!media_matches(
            "(prefers-reduced-transparency: reduce)",
            1000.0
        ));
        assert!(media_matches("(prefers-reduced-transparency)", 1000.0));

        // Configure: reduce
        set_prefers_reduced_transparency(PrefersReducedTransparency::Reduce);
        assert!(!media_matches(
            "(prefers-reduced-transparency: no-preference)",
            1000.0
        ));
        assert!(media_matches(
            "(prefers-reduced-transparency: reduce)",
            1000.0
        ));
        assert!(media_matches("(prefers-reduced-transparency)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches(
            "(prefers-reduced-transparency: unknown)",
            1000.0
        ));

        // Case insensitivity
        assert!(media_matches(
            "(PREFERS-REDUCED-TRANSPARENCY: ReDuCe)",
            1000.0
        ));

        // Reset
        set_prefers_reduced_transparency(PrefersReducedTransparency::NoPreference);
    }

    #[test]
    fn test_forced_colors() {
        // Default: none
        assert!(media_matches("(forced-colors: none)", 1000.0));
        assert!(!media_matches("(forced-colors: active)", 1000.0));
        // Boolean context (since default is none, it evaluates to false)
        assert!(!media_matches("(forced-colors)", 1000.0));

        // Configure: active
        set_forced_colors(ForcedColors::Active);
        assert!(!media_matches("(forced-colors: none)", 1000.0));
        assert!(media_matches("(forced-colors: active)", 1000.0));
        assert!(media_matches("(forced-colors)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(forced-colors: unknown)", 1000.0));

        // Case insensitivity
        assert!(media_matches("(FORCED-COLORS: AcTiVe)", 1000.0));

        // Reset
        set_forced_colors(ForcedColors::None);
    }

    #[test]
    fn test_inverted_colors() {
        // Default: none
        assert!(media_matches("(inverted-colors: none)", 1000.0));
        assert!(!media_matches("(inverted-colors: inverted)", 1000.0));
        // Boolean context (default none -> false)
        assert!(!media_matches("(inverted-colors)", 1000.0));

        // Configure: inverted
        set_inverted_colors(InvertedColors::Inverted);
        assert!(!media_matches("(inverted-colors: none)", 1000.0));
        assert!(media_matches("(inverted-colors: inverted)", 1000.0));
        assert!(media_matches("(inverted-colors)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(inverted-colors: unknown)", 1000.0));

        // Case insensitivity
        assert!(media_matches("(INVERTED-COLORS: InVeRtEd)", 1000.0));

        // Reset
        set_inverted_colors(InvertedColors::None);
    }

    #[test]
    fn test_update_mode() {
        // Default: fast
        assert!(media_matches("(update: fast)", 1000.0));
        assert!(!media_matches("(update: slow)", 1000.0));
        assert!(!media_matches("(update: none)", 1000.0));
        // Boolean context
        assert!(media_matches("(update)", 1000.0));

        // Configure: slow
        set_update_mode(UpdateMode::Slow);
        assert!(media_matches("(update: slow)", 1000.0));
        assert!(!media_matches("(update: fast)", 1000.0));
        assert!(media_matches("(update)", 1000.0));

        // Configure: none
        set_update_mode(UpdateMode::None);
        assert!(media_matches("(update: none)", 1000.0));
        assert!(!media_matches("(update: fast)", 1000.0));
        assert!(!media_matches("(update)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(update: unknown)", 1000.0));

        // Case insensitivity
        set_update_mode(UpdateMode::Slow);
        assert!(media_matches("(UPDATE: SlOw)", 1000.0));

        // Reset
        set_update_mode(UpdateMode::Fast);
    }

    #[test]
    fn test_scripting() {
        // Default: enabled
        assert!(media_matches("(scripting: enabled)", 1000.0));
        assert!(!media_matches("(scripting: initial-only)", 1000.0));
        assert!(!media_matches("(scripting: none)", 1000.0));
        // Boolean context
        assert!(media_matches("(scripting)", 1000.0));

        // Configure: initial-only
        set_scripting(Scripting::InitialOnly);
        assert!(media_matches("(scripting: initial-only)", 1000.0));
        assert!(!media_matches("(scripting: enabled)", 1000.0));
        assert!(media_matches("(scripting)", 1000.0));

        // Configure: none
        set_scripting(Scripting::None);
        assert!(media_matches("(scripting: none)", 1000.0));
        assert!(!media_matches("(scripting: enabled)", 1000.0));
        assert!(!media_matches("(scripting)", 1000.0));

        // Unknown value/wrong feature
        assert!(!media_matches("(scripting: unknown)", 1000.0));

        // Case insensitivity
        set_scripting(Scripting::InitialOnly);
        assert!(media_matches("(SCRIPTING: InItIaL-oNlY)", 1000.0));

        // Reset
        set_scripting(Scripting::Enabled);
    }

    #[test]
    fn test_supports_condition_matches_basic() {
        // Supported basic feature
        assert!(supports_condition_matches("(color: red)"));
        assert!(supports_condition_matches("(display: block)"));

        // Unsupported basic feature (unknown property name)
        assert!(!supports_condition_matches("(totally-not-a-prop: 5px)"));

        // Negated unsupported feature -> true
        assert!(supports_condition_matches("not (totally-not-a-prop: 5px)"));

        // Negated supported feature -> false
        assert!(!supports_condition_matches("not (color: red)"));

        // Conjunction (and):
        // true and true -> true
        assert!(supports_condition_matches(
            "(color: red) and (display: block)"
        ));
        // true and false -> false
        assert!(!supports_condition_matches(
            "(color: red) and (totally-not-a-prop: 5px)"
        ));
        // false and false -> false
        assert!(!supports_condition_matches(
            "(totally-not-a-prop: 5px) and (totally-not-another-prop: 10px)"
        ));

        // Disjunction (or):
        // true or false -> true
        assert!(supports_condition_matches(
            "(color: red) or (totally-not-a-prop: 5px)"
        ));
        // false or false -> false
        assert!(!supports_condition_matches(
            "(totally-not-a-prop: 5px) or (totally-not-another-prop: 10px)"
        ));

        // Nesting and complex combinations:
        assert!(supports_condition_matches(
            "((color: red) and (display: block))"
        ));
        assert!(supports_condition_matches(
            "not ((color: red) and (totally-not-a-prop: 5px))"
        ));
    }

    #[test]
    fn test_extract_matched_rules_supports() {
        // 1. @supports (color: red) { div { color: red; } } -> div rule is returned
        let stylesheet1 =
            crate::css::parser::parse_stylesheet("@supports (color: red) { div { color: red; } }");
        let matched1 = extract_matched_rules(&stylesheet1, 1000.0);
        assert_eq!(matched1.len(), 1);
        assert_eq!(serialize_component_values(&matched1[0].prelude), "div ");

        // 2. @supports (totally-not-a-prop: 5px) { div { color: red } } -> div rule is NOT returned
        let stylesheet2 = crate::css::parser::parse_stylesheet(
            "@supports (totally-not-a-prop: 5px) { div { color: red; } }",
        );
        let matched2 = extract_matched_rules(&stylesheet2, 1000.0);
        assert!(matched2.is_empty());

        // 3. @supports not (totally-not-a-prop: 5px) { div { color: red } } -> div rule IS returned
        let stylesheet3 = crate::css::parser::parse_stylesheet(
            "@supports not (totally-not-a-prop: 5px) { div { color: red; } }",
        );
        let matched3 = extract_matched_rules(&stylesheet3, 1000.0);
        assert_eq!(matched3.len(), 1);
        assert_eq!(serialize_component_values(&matched3[0].prelude), "div ");

        // 4. Conjunction and disjunction nested rules
        let stylesheet4 = crate::css::parser::parse_stylesheet(
            "
            @supports (color: red) and (totally-not-a-prop: 5px) {
                span { color: green; }
            }
            @supports (color: red) or (totally-not-a-prop: 5px) {
                p { color: blue; }
            }
            ",
        );
        let matched4 = extract_matched_rules(&stylesheet4, 1000.0);
        assert_eq!(matched4.len(), 1);
        assert_eq!(serialize_component_values(&matched4[0].prelude), "p ");
    }

    #[test]
    fn test_hover_feature() {
        // Default: Hover
        assert!(media_matches("(hover: hover)", 1000.0));
        assert!(!media_matches("(hover: none)", 1000.0));
        // Boolean context: should be true for hover != None
        assert!(media_matches("(hover)", 1000.0));

        // Configure: None
        set_hover(Hover::None);
        assert!(!media_matches("(hover: hover)", 1000.0));
        assert!(media_matches("(hover: none)", 1000.0));
        assert!(!media_matches("(hover)", 1000.0));

        // Case insensitivity
        set_hover(Hover::Hover);
        assert!(media_matches("(HOVER: HoVeR)", 1000.0));

        // Unknown value
        assert!(!media_matches("(hover: unknown)", 1000.0));

        // Reset
        set_hover(Hover::Hover);
    }

    #[test]
    fn test_any_hover_feature() {
        // Default: Hover
        assert!(media_matches("(any-hover: hover)", 1000.0));
        assert!(!media_matches("(any-hover: none)", 1000.0));
        // Boolean context: true for any_hover != None
        assert!(media_matches("(any-hover)", 1000.0));

        // Configure: None
        set_any_hover(Hover::None);
        assert!(!media_matches("(any-hover: hover)", 1000.0));
        assert!(media_matches("(any-hover: none)", 1000.0));
        assert!(!media_matches("(any-hover)", 1000.0));

        // Case insensitivity
        set_any_hover(Hover::Hover);
        assert!(media_matches("(ANY-HOVER: HoVeR)", 1000.0));

        // Reset
        set_any_hover(Hover::Hover);
    }

    #[test]
    fn test_pointer_feature() {
        // Default: Fine
        assert!(media_matches("(pointer: fine)", 1000.0));
        assert!(!media_matches("(pointer: coarse)", 1000.0));
        assert!(!media_matches("(pointer: none)", 1000.0));
        // Boolean context: true for pointer != None
        assert!(media_matches("(pointer)", 1000.0));

        // Configure: Coarse
        set_pointer(Pointer::Coarse);
        assert!(!media_matches("(pointer: fine)", 1000.0));
        assert!(media_matches("(pointer: coarse)", 1000.0));
        assert!(!media_matches("(pointer: none)", 1000.0));
        assert!(media_matches("(pointer)", 1000.0));

        // Configure: None
        set_pointer(Pointer::None);
        assert!(!media_matches("(pointer: fine)", 1000.0));
        assert!(!media_matches("(pointer: coarse)", 1000.0));
        assert!(media_matches("(pointer: none)", 1000.0));
        assert!(!media_matches("(pointer)", 1000.0));

        // Case insensitivity
        set_pointer(Pointer::Fine);
        assert!(media_matches("(POINTER: FiNe)", 1000.0));

        // Unknown value
        assert!(!media_matches("(pointer: unknown)", 1000.0));

        // Reset
        set_pointer(Pointer::Fine);
    }

    #[test]
    fn test_any_pointer_feature() {
        // Default: Fine
        assert!(media_matches("(any-pointer: fine)", 1000.0));
        assert!(!media_matches("(any-pointer: coarse)", 1000.0));
        assert!(!media_matches("(any-pointer: none)", 1000.0));
        // Boolean context: true for any_pointer != None
        assert!(media_matches("(any-pointer)", 1000.0));

        // Configure: Coarse
        set_any_pointer(Pointer::Coarse);
        assert!(!media_matches("(any-pointer: fine)", 1000.0));
        assert!(media_matches("(any-pointer: coarse)", 1000.0));
        assert!(!media_matches("(any-pointer: none)", 1000.0));
        assert!(media_matches("(any-pointer)", 1000.0));

        // Configure: None
        set_any_pointer(Pointer::None);
        assert!(!media_matches("(any-pointer: fine)", 1000.0));
        assert!(!media_matches("(any-pointer: coarse)", 1000.0));
        assert!(media_matches("(any-pointer: none)", 1000.0));
        assert!(!media_matches("(any-pointer)", 1000.0));

        // Reset
        set_any_pointer(Pointer::Fine);
    }

    #[test]
    fn test_color_gamut_feature() {
        // Default color-gamut: srgb
        assert!(media_matches("(color-gamut: srgb)", 1000.0));
        assert!(!media_matches("(color-gamut: p3)", 1000.0));
        assert!(!media_matches("(color-gamut: rec2020)", 1000.0));
        // Case insensitivity
        assert!(media_matches("(COLOR-GAMUT: SrGb)", 1000.0));
        // Boolean context: should be true
        assert!(media_matches("(color-gamut)", 1000.0));
    }

    #[test]
    fn test_display_mode_feature() {
        // Default display-mode: browser
        assert!(media_matches("(display-mode: browser)", 1000.0));
        assert!(!media_matches("(display-mode: fullscreen)", 1000.0));
        assert!(!media_matches("(display-mode: standalone)", 1000.0));
        // Case insensitivity
        assert!(media_matches("(DISPLAY-MODE: BrOwSeR)", 1000.0));
        // Boolean context: should be true
        assert!(media_matches("(display-mode)", 1000.0));
    }

    #[test]
    fn test_overflow_block_feature() {
        // Default overflow-block: scroll
        assert!(media_matches("(overflow-block: scroll)", 1000.0));
        assert!(!media_matches("(overflow-block: none)", 1000.0));
        assert!(!media_matches("(overflow-block: paged)", 1000.0));
        // Case insensitivity
        assert!(media_matches("(OVERFLOW-BLOCK: ScRoLl)", 1000.0));
        // Boolean context: should be true
        assert!(media_matches("(overflow-block)", 1000.0));
    }

    #[test]
    fn test_overflow_inline_feature() {
        // Default overflow-inline: scroll
        assert!(media_matches("(overflow-inline: scroll)", 1000.0));
        assert!(!media_matches("(overflow-inline: none)", 1000.0));
        // Case insensitivity
        assert!(media_matches("(OVERFLOW-INLINE: ScRoLl)", 1000.0));
        // Boolean context: should be true
        assert!(media_matches("(overflow-inline)", 1000.0));
    }

    #[test]
    fn test_orientation_feature() {
        // viewport_h is thread-local and defaults to 1024
        // With viewport_w = 1000.0, viewport_h = 1024.0 -> portrait
        set_viewport_h(1024.0);
        assert!(media_matches("(orientation: portrait)", 1000.0));
        assert!(!media_matches("(orientation: landscape)", 1000.0));
        assert!(media_matches("(orientation)", 1000.0));

        // With viewport_w = 1200.0, viewport_h = 1024.0 -> landscape
        assert!(!media_matches("(orientation: portrait)", 1200.0));
        assert!(media_matches("(orientation: landscape)", 1200.0));
        assert!(media_matches("(orientation)", 1200.0));

        // Let's change viewport_h to 500.0
        set_viewport_h(500.0);
        // With viewport_w = 600.0, viewport_h = 500.0 -> landscape
        assert!(!media_matches("(orientation: portrait)", 600.0));
        assert!(media_matches("(orientation: landscape)", 600.0));

        // With viewport_w = 400.0, viewport_h = 500.0 -> portrait
        assert!(media_matches("(orientation: portrait)", 400.0));
        assert!(!media_matches("(orientation: landscape)", 400.0));

        // Reset to default
        set_viewport_h(1024.0);
    }

    #[test]
    fn test_monochrome_feature() {
        // Monochrome evaluates to 0
        assert!(media_matches("(monochrome: 0)", 1000.0));
        assert!(!media_matches("(monochrome: 1)", 1000.0));
        assert!(!media_matches("(monochrome)", 1000.0));

        assert!(media_matches("(min-monochrome: 0)", 1000.0));
        assert!(!media_matches("(min-monochrome: 1)", 1000.0));

        assert!(media_matches("(max-monochrome: 0)", 1000.0));
        assert!(media_matches("(max-monochrome: 1)", 1000.0));
    }

    #[test]
    fn test_grid_feature() {
        // Grid evaluates to 0
        assert!(media_matches("(grid: 0)", 1000.0));
        assert!(!media_matches("(grid: 1)", 1000.0));
        assert!(!media_matches("(grid)", 1000.0));

        assert!(media_matches("(min-grid: 0)", 1000.0));
        assert!(!media_matches("(min-grid: 1)", 1000.0));

        assert!(media_matches("(max-grid: 0)", 1000.0));
        assert!(media_matches("(max-grid: 1)", 1000.0));
    }

    #[test]
    fn test_scan_feature() {
        // Scan evaluates to progressive
        assert!(media_matches("(scan: progressive)", 1000.0));
        assert!(!media_matches("(scan: interlace)", 1000.0));
        assert!(media_matches("(scan)", 1000.0));
    }

    #[test]
    fn test_height_features() {
        set_viewport_h(800.0);
        assert!(media_matches("(height: 800px)", 1000.0));
        assert!(media_matches("(min-height: 700px)", 1000.0));
        assert!(media_matches("(max-height: 900px)", 1000.0));
        assert!(!media_matches("(height: 600px)", 1000.0));
        assert!(media_matches("(height)", 1000.0));
        set_viewport_h(1024.0); // Reset to default
    }

    #[test]
    fn test_aspect_ratio_features() {
        set_viewport_h(1000.0);
        // viewport_w = 1000.0, viewport_h = 1000.0 -> ratio 1.0
        assert!(media_matches("(aspect-ratio: 1)", 1000.0));
        assert!(media_matches("(aspect-ratio: 1/1)", 1000.0));
        assert!(media_matches("(min-aspect-ratio: 0.5)", 1000.0));
        assert!(media_matches("(max-aspect-ratio: 1.5)", 1000.0));
        assert!(media_matches("(aspect-ratio)", 1000.0));

        set_viewport_h(500.0);
        // viewport_w = 1000.0, viewport_h = 500.0 -> ratio 2.0 (i.e. 2/1)
        assert!(media_matches("(aspect-ratio: 2/1)", 1000.0));
        assert!(media_matches("(aspect-ratio: 2)", 1000.0));
        assert!(media_matches("(min-aspect-ratio: 16/9)", 1000.0)); // 2.0 >= 1.77777
        assert!(!media_matches("(max-aspect-ratio: 4/3)", 1000.0)); // 2.0 <= 1.33333

        set_viewport_h(1024.0); // Reset to default
    }

    #[test]
    fn test_color_and_color_index_features() {
        // color depth = 8 (non-zero)
        assert!(media_matches("(color)", 1000.0));
        assert!(media_matches("(color: 8)", 1000.0));
        assert!(media_matches("(min-color: 4)", 1000.0));
        assert!(media_matches("(max-color: 16)", 1000.0));
        assert!(!media_matches("(color: 10)", 1000.0));

        // color-index = 0 (zero)
        assert!(!media_matches("(color-index)", 1000.0));
        assert!(media_matches("(color-index: 0)", 1000.0));
        assert!(media_matches("(min-color-index: 0)", 1000.0));
        assert!(!media_matches("(min-color-index: 1)", 1000.0));
        assert!(media_matches("(max-color-index: 0)", 1000.0));
        assert!(media_matches("(max-color-index: 4)", 1000.0));
    }

    #[test]
    fn test_dynamic_range_features() {
        // Default dynamic-range: standard
        assert!(media_matches("(dynamic-range: standard)", 1000.0));
        assert!(!media_matches("(dynamic-range: high)", 1000.0));
        assert!(media_matches("(dynamic-range)", 1000.0));

        // Configure: High (also matches standard)
        set_dynamic_range(DynamicRange::High);
        assert!(media_matches("(dynamic-range: standard)", 1000.0));
        assert!(media_matches("(dynamic-range: high)", 1000.0));

        // Reset to standard
        set_dynamic_range(DynamicRange::Standard);

        // Default video-dynamic-range: standard
        assert!(media_matches("(video-dynamic-range: standard)", 1000.0));
        assert!(!media_matches("(video-dynamic-range: high)", 1000.0));
        assert!(media_matches("(video-dynamic-range)", 1000.0));

        // Configure: High (also matches standard)
        set_video_dynamic_range(DynamicRange::High);
        assert!(media_matches("(video-dynamic-range: standard)", 1000.0));
        assert!(media_matches("(video-dynamic-range: high)", 1000.0));

        // Reset to standard
        set_video_dynamic_range(DynamicRange::Standard);
    }

    #[test]
    fn test_environment_blending_feature() {
        // Default environment-blending: opaque
        assert!(media_matches("(environment-blending: opaque)", 1000.0));
        assert!(!media_matches("(environment-blending: additive)", 1000.0));
        assert!(!media_matches(
            "(environment-blending: subtractive)",
            1000.0
        ));
        assert!(media_matches("(environment-blending)", 1000.0));

        // Configure: Additive
        set_environment_blending(EnvironmentBlending::Additive);
        assert!(media_matches("(environment-blending: additive)", 1000.0));
        assert!(!media_matches("(environment-blending: opaque)", 1000.0));

        // Reset
        set_environment_blending(EnvironmentBlending::Opaque);
    }

    #[test]
    fn test_light_level_feature() {
        // Default light-level: normal
        assert!(media_matches("(light-level: normal)", 1000.0));
        assert!(!media_matches("(light-level: dim)", 1000.0));
        assert!(!media_matches("(light-level: washed)", 1000.0));
        assert!(media_matches("(light-level)", 1000.0));

        // Configure: Dim
        set_light_level(LightLevel::Dim);
        assert!(media_matches("(light-level: dim)", 1000.0));
        assert!(!media_matches("(light-level: normal)", 1000.0));

        // Reset
        set_light_level(LightLevel::Normal);
    }

    #[test]
    fn test_device_posture_feature() {
        // Default device-posture: continuous
        assert!(media_matches("(device-posture: continuous)", 1000.0));
        assert!(!media_matches("(device-posture: folded)", 1000.0));
        assert!(media_matches("(device-posture)", 1000.0));

        // Configure: Folded
        set_device_posture(DevicePosture::Folded);
        assert!(media_matches("(device-posture: folded)", 1000.0));
        assert!(!media_matches("(device-posture: continuous)", 1000.0));

        // Reset
        set_device_posture(DevicePosture::Continuous);
    }

    #[test]
    fn test_nav_controls_feature() {
        // Default nav-controls: none
        assert!(media_matches("(nav-controls: none)", 1000.0));
        assert!(!media_matches("(nav-controls: back)", 1000.0));
        assert!(!media_matches("(nav-controls)", 1000.0)); // none is falsy in boolean context

        // Configure: Back
        set_nav_controls(NavControls::Back);
        assert!(media_matches("(nav-controls: back)", 1000.0));
        assert!(!media_matches("(nav-controls: none)", 1000.0));
        assert!(media_matches("(nav-controls)", 1000.0));

        // Reset
        set_nav_controls(NavControls::None);
    }

    #[test]
    fn test_video_color_gamut_feature() {
        // Default video-color-gamut: srgb
        assert!(media_matches("(video-color-gamut: srgb)", 1000.0));
        assert!(!media_matches("(video-color-gamut: p3)", 1000.0));
        assert!(media_matches("(video-color-gamut)", 1000.0));

        // Configure: P3 (also matches srgb)
        set_video_color_gamut(ColorGamut::P3);
        assert!(media_matches("(video-color-gamut: p3)", 1000.0));
        assert!(media_matches("(video-color-gamut: srgb)", 1000.0));

        // Reset
        set_video_color_gamut(ColorGamut::Srgb);
    }

    #[test]
    fn test_display_shape_feature() {
        // Default shape: rect
        assert!(media_matches("(shape: rect)", 1000.0));
        assert!(!media_matches("(shape: round)", 1000.0));
        assert!(media_matches("(shape)", 1000.0));

        assert!(media_matches("(display-shape: rect)", 1000.0));
        assert!(!media_matches("(display-shape: round)", 1000.0));
        assert!(media_matches("(display-shape)", 1000.0));

        // Configure: Round
        set_display_shape(DisplayShape::Round);
        assert!(media_matches("(shape: round)", 1000.0));
        assert!(!media_matches("(shape: rect)", 1000.0));
        assert!(media_matches("(display-shape: round)", 1000.0));
        assert!(!media_matches("(display-shape: rect)", 1000.0));

        // Reset
        set_display_shape(DisplayShape::Rect);
    }

    #[test]
    fn test_viewport_segments_features() {
        // Default horizontal-viewport-segments: 1
        assert!(media_matches("(horizontal-viewport-segments: 1)", 1000.0));
        assert!(!media_matches("(horizontal-viewport-segments: 2)", 1000.0));
        assert!(media_matches(
            "(min-horizontal-viewport-segments: 1)",
            1000.0
        ));
        assert!(media_matches(
            "(max-horizontal-viewport-segments: 1)",
            1000.0
        ));
        assert!(media_matches("(horizontal-viewport-segments)", 1000.0));

        // Default vertical-viewport-segments: 1
        assert!(media_matches("(vertical-viewport-segments: 1)", 1000.0));
        assert!(!media_matches("(vertical-viewport-segments: 2)", 1000.0));
        assert!(media_matches("(min-vertical-viewport-segments: 1)", 1000.0));
        assert!(media_matches("(max-vertical-viewport-segments: 1)", 1000.0));
        assert!(media_matches("(vertical-viewport-segments)", 1000.0));

        // Configure horizontal to 2
        set_horizontal_viewport_segments(2);
        assert!(media_matches("(horizontal-viewport-segments: 2)", 1000.0));
        assert!(!media_matches("(horizontal-viewport-segments: 1)", 1000.0));
        assert!(media_matches(
            "(min-horizontal-viewport-segments: 2)",
            1000.0
        ));
        assert!(media_matches(
            "(max-horizontal-viewport-segments: 2)",
            1000.0
        ));
        assert!(media_matches("(horizontal-viewport-segments)", 1000.0));

        // Configure vertical to 3
        set_vertical_viewport_segments(3);
        assert!(media_matches("(vertical-viewport-segments: 3)", 1000.0));
        assert!(!media_matches("(vertical-viewport-segments: 1)", 1000.0));
        assert!(media_matches("(min-vertical-viewport-segments: 2)", 1000.0));
        assert!(media_matches("(max-vertical-viewport-segments: 4)", 1000.0));
        assert!(media_matches("(vertical-viewport-segments)", 1000.0));

        // Reset to default
        set_horizontal_viewport_segments(1);
        set_vertical_viewport_segments(1);
    }

    #[test]
    fn test_resolution_and_device_pixel_ratio_features() {
        // Default dpr is 1.0
        assert_eq!(device_pixel_ratio(), 1.0);
        assert!(media_matches("(resolution: 1dppx)", 1000.0));
        assert!(media_matches("(resolution: 1x)", 1000.0));
        assert!(media_matches("(resolution: 96dpi)", 1000.0));
        assert!(media_matches("(resolution: 37.795275dpcm)", 1000.0)); // 96 / 2.54 = 37.795275
        assert!(media_matches("(resolution)", 1000.0));

        assert!(media_matches("(device-pixel-ratio: 1)", 1000.0));
        assert!(media_matches("(device-pixel-ratio: 1/1)", 1000.0));
        assert!(media_matches("(-webkit-device-pixel-ratio: 1)", 1000.0));
        assert!(media_matches("(device-pixel-ratio)", 1000.0));

        // Let's set device pixel ratio to 2.0
        set_device_pixel_ratio(2.0);
        assert_eq!(device_pixel_ratio(), 2.0);

        // Test resolution with dpr=2.0
        assert!(media_matches("(resolution: 2dppx)", 1000.0));
        assert!(media_matches("(resolution: 2x)", 1000.0));
        assert!(media_matches("(resolution: 192dpi)", 1000.0));
        assert!(media_matches("(min-resolution: 1.5dppx)", 1000.0));
        assert!(media_matches("(min-resolution: 144dpi)", 1000.0));
        assert!(media_matches("(max-resolution: 3dppx)", 1000.0));
        assert!(media_matches("(max-resolution: 288dpi)", 1000.0));

        // Test device-pixel-ratio with dpr=2.0
        assert!(media_matches("(device-pixel-ratio: 2)", 1000.0));
        assert!(media_matches("(device-pixel-ratio: 2/1)", 1000.0));
        assert!(media_matches("(min-device-pixel-ratio: 1.5)", 1000.0));
        assert!(media_matches("(max-device-pixel-ratio: 2.5)", 1000.0));
        assert!(media_matches("(-webkit-device-pixel-ratio: 2)", 1000.0));
        assert!(media_matches(
            "(-webkit-min-device-pixel-ratio: 1.5)",
            1000.0
        ));
        assert!(media_matches(
            "(-webkit-max-device-pixel-ratio: 2.5)",
            1000.0
        ));

        // Clean up: Reset to 1.0
        set_device_pixel_ratio(1.0);
    }

    #[test]
    fn test_range_media_queries() {
        // --- Form 1: <mf-name> <op> <value> ---
        // width comparisons (viewport_w = 600.0)
        assert!(media_matches("(width >= 400px)", 600.0));
        assert!(media_matches("(width >= 600px)", 600.0));
        assert!(!media_matches("(width >= 800px)", 600.0));
        assert!(media_matches("(width > 400px)", 600.0));
        assert!(!media_matches("(width > 600px)", 600.0));

        assert!(media_matches("(width <= 800px)", 600.0));
        assert!(media_matches("(width <= 600px)", 600.0));
        assert!(!media_matches("(width <= 400px)", 600.0));
        assert!(media_matches("(width < 800px)", 600.0));
        assert!(!media_matches("(width < 600px)", 600.0));

        assert!(media_matches("(width = 600px)", 600.0));
        assert!(!media_matches("(width = 500px)", 600.0));

        // height comparisons (viewport_h = 1024.0)
        set_viewport_h(1024.0);
        assert!(media_matches("(height >= 500px)", 600.0));
        assert!(media_matches("(height < 2000px)", 600.0));
        assert!(media_matches("(height = 1024px)", 600.0));

        // resolution comparisons
        set_device_pixel_ratio(2.0);
        assert!(media_matches("(resolution >= 1.5dppx)", 600.0));
        assert!(media_matches("(resolution >= 2x)", 600.0));
        assert!(!media_matches("(resolution > 2x)", 600.0));
        assert!(media_matches("(resolution <= 3x)", 600.0));
        assert!(media_matches("(resolution = 2dppx)", 600.0));
        set_device_pixel_ratio(1.0);

        // --- Form 2: <value> <op> <mf-name> ---
        assert!(media_matches("(400px <= width)", 600.0));
        assert!(media_matches("(600px <= width)", 600.0));
        assert!(!media_matches("(800px <= width)", 600.0));
        assert!(media_matches("(400px < width)", 600.0));
        assert!(!media_matches("(600px < width)", 600.0));

        assert!(media_matches("(800px >= width)", 600.0));
        assert!(media_matches("(600px >= width)", 600.0));
        assert!(!media_matches("(400px >= width)", 600.0));
        assert!(media_matches("(800px > width)", 600.0));
        assert!(!media_matches("(600px > width)", 600.0));

        assert!(media_matches("(600px = width)", 600.0));
        assert!(!media_matches("(500px = width)", 600.0));

        // --- Form 3: <value> <op1> <mf-name> <op2> <value> ---
        assert!(media_matches("(400px <= width <= 800px)", 600.0));
        assert!(media_matches("(400px < width < 800px)", 600.0));
        assert!(media_matches("(400px <= width < 600px)", 500.0));
        assert!(!media_matches("(400px <= width <= 500px)", 600.0));
        assert!(!media_matches("(700px <= width <= 900px)", 600.0));

        // Mixed/Unsupported or invalid directions
        assert!(!media_matches("(400px <= width >= 800px)", 600.0)); // conflicting ops

        // resolution double range
        set_device_pixel_ratio(2.0);
        assert!(media_matches("(1x <= resolution <= 3x)", 600.0));
        assert!(!media_matches("(2.5x <= resolution <= 3x)", 600.0));
        set_device_pixel_ratio(1.0);
    }

    #[test]
    fn test_media_query_or_logical_combinator() {
        // Simple 'or'
        assert!(media_matches("(width >= 1000px) or (hover: hover)", 600.0));
        assert!(media_matches("(width >= 500px) or (hover: none)", 600.0));
        assert!(!media_matches("(width >= 1000px) or (hover: none)", 600.0));

        // Multiple 'or'
        assert!(media_matches(
            "(width >= 1000px) or (hover: none) or (pointer: fine)",
            600.0
        ));

        // Nested condition inside parens with 'or' and 'and'
        assert!(media_matches(
            "((hover: hover) or (pointer: coarse)) and (width >= 500px)",
            600.0
        ));
        assert!(!media_matches(
            "((hover: hover) or (pointer: coarse)) and (width >= 1000px)",
            600.0
        ));
    }

    #[test]
    fn test_media_query_nested_not_logical_combinator() {
        // 'not' prefixing parenthesized feature
        assert!(media_matches("(not (hover: none))", 600.0));
        assert!(!media_matches("(not (hover: hover))", 600.0));

        // Nested 'not' combined with 'and' / 'or'
        assert!(media_matches(
            "(not (hover: none)) and (width >= 500px)",
            600.0
        ));
        assert!(!media_matches(
            "(not (hover: hover)) or (width >= 1000px)",
            600.0
        ));
    }

    #[test]
    fn test_device_dimensions_and_aspect_ratio() {
        set_device_width(1920.0);
        set_device_height(1080.0);

        // 1. Existential/boolean queries
        assert!(media_matches("(device-width)", 1000.0));
        assert!(media_matches("(device-height)", 1000.0));
        assert!(media_matches("(device-aspect-ratio)", 1000.0));

        // 2. Colon-based queries (exact, min, max)
        assert!(media_matches("(device-width: 1920px)", 1000.0));
        assert!(media_matches("(min-device-width: 1000px)", 1000.0));
        assert!(media_matches("(max-device-width: 2000px)", 1000.0));
        assert!(!media_matches("(device-width: 1000px)", 1000.0));

        assert!(media_matches("(device-height: 1080px)", 1000.0));
        assert!(media_matches("(min-device-height: 1000px)", 1000.0));
        assert!(media_matches("(max-device-height: 1200px)", 1000.0));
        assert!(!media_matches("(device-height: 1000px)", 1000.0));

        // Aspect ratio: 1920 / 1080 = 1.777777... (i.e. 16/9)
        assert!(media_matches("(device-aspect-ratio: 16/9)", 1000.0));
        assert!(media_matches("(min-device-aspect-ratio: 4/3)", 1000.0));
        assert!(media_matches("(max-device-aspect-ratio: 2/1)", 1000.0));
        assert!(!media_matches("(device-aspect-ratio: 4/3)", 1000.0));

        // 3. Range-based queries (Form 1, 2, 3)
        assert!(media_matches("(device-width >= 1000px)", 1000.0));
        assert!(media_matches("(1000px <= device-width <= 2000px)", 1000.0));
        assert!(media_matches("(device-height <= 1200px)", 1000.0));
        assert!(media_matches("(device-aspect-ratio > 1.5)", 1000.0));
        assert!(media_matches("(1.5 < device-aspect-ratio < 2.0)", 1000.0));

        // Reset
        set_device_width(1920.0);
        set_device_height(1080.0);
    }

    #[test]
    fn test_color_gamut_subset() {
        // Rec2020 should match srgb, p3, rec2020
        set_color_gamut(ColorGamut::Rec2020);
        assert!(media_matches("(color-gamut: srgb)", 1000.0));
        assert!(media_matches("(color-gamut: p3)", 1000.0));
        assert!(media_matches("(color-gamut: rec2020)", 1000.0));

        // P3 should match srgb, p3, but not rec2020
        set_color_gamut(ColorGamut::P3);
        assert!(media_matches("(color-gamut: srgb)", 1000.0));
        assert!(media_matches("(color-gamut: p3)", 1000.0));
        assert!(!media_matches("(color-gamut: rec2020)", 1000.0));

        // Srgb should match srgb, but not p3 or rec2020
        set_color_gamut(ColorGamut::Srgb);
        assert!(media_matches("(color-gamut: srgb)", 1000.0));
        assert!(!media_matches("(color-gamut: p3)", 1000.0));
        assert!(!media_matches("(color-gamut: rec2020)", 1000.0));
    }

    #[test]
    fn test_new_standard_media_features() {
        // 1. video-color-gamut subsetting rules
        set_video_color_gamut(ColorGamut::Rec2020);
        assert!(media_matches("(video-color-gamut: srgb)", 1000.0));
        assert!(media_matches("(video-color-gamut: p3)", 1000.0));
        assert!(media_matches("(video-color-gamut: rec2020)", 1000.0));

        set_video_color_gamut(ColorGamut::P3);
        assert!(media_matches("(video-color-gamut: srgb)", 1000.0));
        assert!(media_matches("(video-color-gamut: p3)", 1000.0));
        assert!(!media_matches("(video-color-gamut: rec2020)", 1000.0));

        set_video_color_gamut(ColorGamut::Srgb);
        assert!(media_matches("(video-color-gamut: srgb)", 1000.0));
        assert!(!media_matches("(video-color-gamut: p3)", 1000.0));
        assert!(!media_matches("(video-color-gamut: rec2020)", 1000.0));

        // 2. dynamic-range and video-dynamic-range subsetting rules
        set_dynamic_range(DynamicRange::High);
        assert!(media_matches("(dynamic-range: standard)", 1000.0));
        assert!(media_matches("(dynamic-range: high)", 1000.0));

        set_dynamic_range(DynamicRange::Standard);
        assert!(media_matches("(dynamic-range: standard)", 1000.0));
        assert!(!media_matches("(dynamic-range: high)", 1000.0));

        set_video_dynamic_range(DynamicRange::High);
        assert!(media_matches("(video-dynamic-range: standard)", 1000.0));
        assert!(media_matches("(video-dynamic-range: high)", 1000.0));

        set_video_dynamic_range(DynamicRange::Standard);
        assert!(media_matches("(video-dynamic-range: standard)", 1000.0));
        assert!(!media_matches("(video-dynamic-range: high)", 1000.0));

        // 3. display-mode with new Tabbed and Borderless values and thread-local configuration
        set_display_mode(DisplayMode::Tabbed);
        assert!(media_matches("(display-mode: tabbed)", 1000.0));
        assert!(!media_matches("(display-mode: borderless)", 1000.0));
        assert!(!media_matches("(display-mode: browser)", 1000.0));

        set_display_mode(DisplayMode::Borderless);
        assert!(!media_matches("(display-mode: tabbed)", 1000.0));
        assert!(media_matches("(display-mode: borderless)", 1000.0));

        set_display_mode(DisplayMode::Browser);
        assert!(media_matches("(display-mode: browser)", 1000.0));

        // 4. overflow-block and overflow-inline configurations
        set_overflow_block(OverflowBlock::None);
        assert!(media_matches("(overflow-block: none)", 1000.0));
        assert!(!media_matches("(overflow-block: scroll)", 1000.0));

        set_overflow_block(OverflowBlock::Scroll);
        assert!(media_matches("(overflow-block: scroll)", 1000.0));

        set_overflow_inline(OverflowInline::None);
        assert!(media_matches("(overflow-inline: none)", 1000.0));
        assert!(!media_matches("(overflow-inline: scroll)", 1000.0));

        set_overflow_inline(OverflowInline::Scroll);
        assert!(media_matches("(overflow-inline: scroll)", 1000.0));
    }

    #[test]
    fn test_absolute_physical_and_viewport_units() {
        // --- 1. Absolute Physical Units ---
        // 1in = 96px. If viewport_w is 96px, it should match (width: 1in).
        assert!(media_matches("(width: 1in)", 96.0));
        assert!(media_matches("(min-width: 0.5in)", 96.0));
        assert!(!media_matches("(width: 2in)", 96.0));

        // 1cm = 96 / 2.54 px = 37.795px. Use inequalities for safety.
        assert!(media_matches("(min-width: 1cm)", 38.0));
        assert!(!media_matches("(min-width: 1cm)", 37.0));
        assert!(media_matches("(max-width: 1cm)", 37.0));
        assert!(!media_matches("(max-width: 1cm)", 38.0));
        assert!(media_matches("(min-width: 5mm)", 20.0)); // 5mm = 18.89px

        // 1pt = 96 / 72 px = 1.33333px. Use inequalities for safety.
        assert!(media_matches("(min-width: 100pt)", 134.0));
        assert!(!media_matches("(min-width: 100pt)", 133.0));
        assert!(media_matches("(max-width: 100pt)", 133.0));
        assert!(!media_matches("(max-width: 100pt)", 134.0));

        // 1pc = 16px. If viewport_w is 16px, it should match (width: 1pc).
        assert!(media_matches("(width: 1pc)", 16.0));
        assert!(media_matches("(width: 10pc)", 160.0));

        // --- 2. Viewport-Relative Units ---
        // viewport_w = 800px, viewport_h() = 600px
        set_viewport_h(600.0);

        // 100vw = 800px
        assert!(media_matches("(width: 100vw)", 800.0));
        assert!(media_matches("(min-width: 10vw)", 800.0));
        assert!(!media_matches("(width: 10vw)", 800.0));
        assert!(!media_matches("(max-width: 20vw)", 800.0));
        assert!(media_matches("(max-width: 120vw)", 800.0));

        // 50vh = 300px
        assert!(media_matches("(width: 50vh)", 300.0)); // 50vh of 600px = 300px, so width: 300px matches when viewport_w is 300.0!
        assert!(media_matches("(min-width: 20vh)", 800.0)); // 20vh of 600px = 120px, 800 >= 120 is true!

        // vmin = min(vw, vh).
        // If viewport_w = 800, viewport_h = 600, then min(800, 600) = 600.
        // So 50vmin = 50% of 600 = 300px.
        assert!(media_matches("(min-width: 50vmin)", 800.0)); // 800 >= 300 is true!

        // vmax = max(vw, vh).
        // If viewport_w = 800, viewport_h = 600, then max(800, 600) = 800.
        // So 50vmax = 50% of 800 = 400px.
        assert!(media_matches("(min-width: 50vmax)", 800.0)); // 800 >= 400 is true!

        // --- 3. Range queries with new units ---
        assert!(media_matches("(2vw <= width <= 200vw)", 800.0));

        set_viewport_h(1024.0); // Reset
    }

    #[test]
    fn test_extended_range_syntax_and_ratios() {
        set_device_pixel_ratio(2.0);
        assert!(media_matches("(device-pixel-ratio >= 2/1)", 1000.0));
        assert!(media_matches("(1.5 <= device-pixel-ratio <= 2.5)", 1000.0));
        assert!(media_matches("(device-pixel-ratio = 2/1)", 1000.0));
        assert!(media_matches("(device-pixel-ratio < 3/1)", 1000.0));
        set_device_pixel_ratio(1.0);

        set_viewport_h(1000.0);
        // viewport_w = 1600.0, viewport_h = 1000.0 -> ratio = 1.6
        assert!(media_matches("(aspect-ratio > 1.5)", 1600.0));
        assert!(media_matches("(1 < aspect-ratio < 2)", 1600.0));
        assert!(media_matches("(aspect-ratio = 16/10)", 1600.0));
        set_viewport_h(1024.0);
    }

    #[test]
    fn test_nested_logical_parentheses() {
        assert!(media_matches("((((width >= 400px))))", 500.0));

        set_viewport_h(500.0);
        assert!(media_matches(
            "(((width >= 400px) and (height >= 400px)))",
            500.0
        ));
        assert!(media_matches("(not (((width < 400px))))", 500.0));
        assert!(media_matches(
            "((width >= 400px) or (height >= 400px))",
            300.0
        ));
        assert!(media_matches(
            "not ((width < 400px) and (height < 400px))",
            500.0
        ));

        // Logical combinations
        assert!(media_matches(
            "(not (hover: none)) and (not (pointer: coarse))",
            1000.0
        ));
        assert!(media_matches(
            "((not (hover: hover)) or (not (pointer: coarse)))",
            1000.0
        ));
        set_viewport_h(1024.0);
    }

    #[test]
    fn test_feature_coverage_extreme_cases() {
        // Floating point widths
        assert!(media_matches("(width >= 400.5px)", 401.0));
        assert!(!media_matches("(width >= 400.5px)", 400.0));

        // Orientation
        assert!(media_matches("(orientation)", 1000.0));

        // Discrete features
        assert!(media_matches("(monochrome: 0)", 1000.0));
        assert!(media_matches("(grid: 0)", 1000.0));
        assert!(media_matches("(color-index: 0)", 1000.0));
        assert!(media_matches("(scan: progressive)", 1000.0));
        assert!(media_matches("(display-mode: Browser)", 1000.0));
        assert!(media_matches("(light-level: normal)", 1000.0));
        assert!(media_matches("(any-pointer: fine)", 1000.0));
        assert!(media_matches("(any-hover: hover)", 1000.0));
    }
}
