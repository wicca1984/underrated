//! Generic property-metadata foundation for MS-CSS-Generic.
//!
//! This module provides a static, queryable metadata table describing common CSS properties.
//! Consumers of this metadata (such as cascade, style resolution, or inheritance logic)
//! will be wired up in later tasks.
//!
//! // TODO(spec): This table is an initial representative subset of CSS properties to be expanded
//! // in subsequent tasks. Shorthand-expansion metadata is intentionally out of scope for now.

/// Metadata representing a CSS property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyMetadata {
    /// The canonical ASCII name of the property (lowercase).
    pub name: &'static str,
    /// Whether the property is inherited by default.
    pub inherited: bool,
    /// The canonical CSS initial value as a string representation.
    pub initial: &'static str,
    /// Whether the property is animatable.
    pub animatable: bool,
}

/// Static table of well-known CSS longhand properties.
///
/// // TODO(spec): Expand this list as needed for comprehensive CSS property support.
static PROPERTY_METADATA: &[PropertyMetadata] = &[
    // INHERITED PROPERTIES
    PropertyMetadata {
        name: "color",
        inherited: true,
        initial: "black",
        animatable: true,
    },
    PropertyMetadata {
        name: "font-family",
        inherited: true,
        initial: "serif",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-size",
        inherited: true,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "font-size-adjust",
        inherited: true,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "font-style",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-weight",
        inherited: true,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "line-height",
        inherited: true,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "text-align",
        inherited: true,
        initial: "start",
        animatable: false,
    },
    PropertyMetadata {
        name: "letter-spacing",
        inherited: true,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "word-spacing",
        inherited: true,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "white-space",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "visibility",
        inherited: true,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "list-style-type",
        inherited: true,
        initial: "disc",
        animatable: false,
    },
    PropertyMetadata {
        name: "direction",
        inherited: true,
        initial: "ltr",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-transform",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "cursor",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-stretch",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-indent",
        inherited: true,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "word-break",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "overflow-wrap",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "word-wrap",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-align-last",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "caption-side",
        inherited: true,
        initial: "top",
        animatable: false,
    },
    PropertyMetadata {
        name: "color-interpolation",
        inherited: true,
        initial: "sRGB",
        animatable: true,
    },
    PropertyMetadata {
        name: "empty-cells",
        inherited: true,
        initial: "show",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-collapse",
        inherited: true,
        initial: "separate",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-spacing",
        inherited: true,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "list-style-position",
        inherited: true,
        initial: "outside",
        animatable: false,
    },
    PropertyMetadata {
        name: "list-style-image",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "quotes",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "tab-size",
        inherited: true,
        initial: "8",
        animatable: true,
    },
    PropertyMetadata {
        name: "hyphens",
        inherited: true,
        initial: "manual",
        animatable: false,
    },
    PropertyMetadata {
        name: "accent-color",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "caret-color",
        inherited: true,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "clip-rule",
        inherited: true,
        initial: "nonzero",
        animatable: false,
    },
    PropertyMetadata {
        name: "scrollbar-width",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scrollbar-color",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-wrap",
        inherited: true,
        initial: "wrap",
        animatable: false,
    },
    PropertyMetadata {
        name: "forced-color-adjust",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "caret-shape",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-autospace",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-spacing-trim",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "hyphenate-character",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "hyphenate-limit-chars",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "ruby-position",
        inherited: true,
        initial: "alternate",
        animatable: false,
    },
    PropertyMetadata {
        name: "ruby-align",
        inherited: true,
        initial: "space-around",
        animatable: false,
    },
    PropertyMetadata {
        name: "ruby-overhang",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "ruby-merge",
        inherited: true,
        initial: "separate",
        animatable: false,
    },
    PropertyMetadata {
        name: "math-style",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "math-depth",
        inherited: true,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "line-break",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "white-space-collapse",
        inherited: true,
        initial: "collapse",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-wrap-style",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-wrap-mode",
        inherited: true,
        initial: "wrap",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-underline-position",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-emphasis-color",
        inherited: true,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-emphasis-style",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-emphasis-position",
        inherited: true,
        initial: "over right",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-justify",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-combine-upright",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-decoration-skip-ink",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "hanging-punctuation",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-rendering",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "block-ellipsis",
        inherited: true,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "reading-order",
        inherited: true,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "writing-mode",
        inherited: true,
        initial: "horizontal-tb",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-orientation",
        inherited: true,
        initial: "mixed",
        animatable: false,
    },
    PropertyMetadata {
        name: "math-shift",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-shadow",
        inherited: true,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "interpolate-size",
        inherited: true,
        initial: "numeric-only",
        animatable: false,
    },
    PropertyMetadata {
        name: "speak",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-kerning",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-optical-sizing",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-palette",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-caps",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-ligatures",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-numeric",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-position",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-east-asian",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-alternates",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variant-emoji",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-synthesis-weight",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-synthesis-style",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-synthesis-small-caps",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-feature-settings",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "font-variation-settings",
        inherited: true,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "font-language-override",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-size-adjust",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "fill",
        inherited: true,
        initial: "black",
        animatable: true,
    },
    PropertyMetadata {
        name: "stroke",
        inherited: true,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "stroke-width",
        inherited: true,
        initial: "1",
        animatable: true,
    },
    PropertyMetadata {
        name: "paint-order",
        inherited: true,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "image-orientation",
        inherited: true,
        initial: "from-image",
        animatable: false,
    },
    // NON-INHERITED PROPERTIES
    PropertyMetadata {
        name: "scrollbar-gutter",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "color-scheme",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "display",
        inherited: false,
        initial: "inline",
        animatable: false,
    },
    PropertyMetadata {
        name: "width",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "height",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "margin-top",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "margin-right",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "margin-bottom",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "margin-left",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "padding-top",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "padding-right",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "padding-bottom",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "padding-left",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-top-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-top-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-top-color",
        inherited: false,
        initial: "currentcolor",
        animatable: true,
    },
    PropertyMetadata {
        name: "background-color",
        inherited: false,
        initial: "transparent",
        animatable: true,
    },
    PropertyMetadata {
        name: "position",
        inherited: false,
        initial: "static",
        animatable: false,
    },
    PropertyMetadata {
        name: "top",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "right",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "bottom",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "left",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "float",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "clear",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "overflow",
        inherited: false,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "overflow-x",
        inherited: false,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "overflow-y",
        inherited: false,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "line-clamp",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "z-index",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "box-sizing",
        inherited: false,
        initial: "content-box",
        animatable: false,
    },
    PropertyMetadata {
        name: "backdrop-filter",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "filter",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "mix-blend-mode",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "isolation",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "initial-letter",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "resize",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "backface-visibility",
        inherited: false,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "clip",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "clip-path",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "opacity",
        inherited: false,
        initial: "1",
        animatable: true,
    },
    PropertyMetadata {
        name: "margin-block-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "margin-block-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "padding-block-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "padding-block-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-right-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-bottom-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-left-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-right-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-bottom-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-left-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-right-color",
        inherited: false,
        initial: "currentcolor",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-bottom-color",
        inherited: false,
        initial: "currentcolor",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-left-color",
        inherited: false,
        initial: "currentcolor",
        animatable: true,
    },
    PropertyMetadata {
        name: "background-image",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-repeat",
        inherited: false,
        initial: "repeat",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-repeat-x",
        inherited: false,
        initial: "repeat",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-repeat-y",
        inherited: false,
        initial: "repeat",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-position",
        inherited: false,
        initial: "0% 0%",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-position-x",
        inherited: false,
        initial: "0%",
        animatable: true,
    },
    PropertyMetadata {
        name: "background-position-y",
        inherited: false,
        initial: "0%",
        animatable: true,
    },
    PropertyMetadata {
        name: "background-size",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-attachment",
        inherited: false,
        initial: "scroll",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-origin",
        inherited: false,
        initial: "padding-box",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-clip",
        inherited: false,
        initial: "border-box",
        animatable: false,
    },
    PropertyMetadata {
        name: "background-blend-mode",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-top-left-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-top-right-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-bottom-right-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-bottom-left-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-image-source",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-image-slice",
        inherited: false,
        initial: "100%",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-image-width",
        inherited: false,
        initial: "1",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-image-outset",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-image-repeat",
        inherited: false,
        initial: "stretch",
        animatable: false,
    },
    PropertyMetadata {
        name: "outline-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "outline-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "outline-color",
        inherited: false,
        initial: "invert",
        animatable: true,
    },
    PropertyMetadata {
        name: "min-width",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "min-height",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "max-width",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "max-height",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "flex-grow",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "flex-shrink",
        inherited: false,
        initial: "1",
        animatable: true,
    },
    PropertyMetadata {
        name: "flex-basis",
        inherited: false,
        initial: "auto",
        animatable: true,
    },
    PropertyMetadata {
        name: "flex-direction",
        inherited: false,
        initial: "row",
        animatable: false,
    },
    PropertyMetadata {
        name: "flex-wrap",
        inherited: false,
        initial: "nowrap",
        animatable: false,
    },
    PropertyMetadata {
        name: "justify-content",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "align-items",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "row-gap",
        inherited: false,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "column-gap",
        inherited: false,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "justify-items",
        inherited: false,
        initial: "legacy",
        animatable: false,
    },
    PropertyMetadata {
        name: "align-content",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "align-self",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "justify-self",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "order",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "table-layout",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "vertical-align",
        inherited: false,
        initial: "baseline",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-decoration-line",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-decoration-color",
        inherited: false,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-decoration-style",
        inherited: false,
        initial: "solid",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-overflow",
        inherited: false,
        initial: "clip",
        animatable: false,
    },
    PropertyMetadata {
        name: "object-fit",
        inherited: false,
        initial: "fill",
        animatable: false,
    },
    PropertyMetadata {
        name: "object-position",
        inherited: false,
        initial: "50% 50%",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-behavior",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "print-color-adjust",
        inherited: true,
        initial: "economy",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-snap-type",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-snap-align",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-snap-stop",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-top",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-right",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-bottom",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-left",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-block",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-block-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-block-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-inline",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-inline-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-margin-inline-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-top",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-right",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-bottom",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-left",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-block",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-block-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-block-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-inline",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-inline-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-padding-inline-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overflow-clip-margin",
        inherited: false,
        initial: "0px",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-block",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-block-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-block-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-inline",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-inline-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inset-inline-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overscroll-behavior",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overscroll-behavior-x",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overscroll-behavior-y",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overscroll-behavior-block",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "overscroll-behavior-inline",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "user-select",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "pointer-events",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "appearance",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "transition-duration",
        inherited: false,
        initial: "0s",
        animatable: false,
    },
    PropertyMetadata {
        name: "transition-property",
        inherited: false,
        initial: "all",
        animatable: false,
    },
    PropertyMetadata {
        name: "transition-timing-function",
        inherited: false,
        initial: "ease",
        animatable: false,
    },
    PropertyMetadata {
        name: "transition-delay",
        inherited: false,
        initial: "0s",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-count",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-width",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-span",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-fill",
        inherited: false,
        initial: "balance",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-rule-width",
        inherited: false,
        initial: "medium",
        animatable: true,
    },
    PropertyMetadata {
        name: "column-rule-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "column-rule-color",
        inherited: false,
        initial: "currentcolor",
        animatable: true,
    },
    PropertyMetadata {
        name: "image-rendering",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "contain",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-decoration-thickness",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-underline-offset",
        inherited: true,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "counter-reset",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "counter-increment",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "counter-set",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "orphans",
        inherited: true,
        initial: "2",
        animatable: false,
    },
    PropertyMetadata {
        name: "widows",
        inherited: true,
        initial: "2",
        animatable: false,
    },
    PropertyMetadata {
        name: "break-before",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "break-after",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "break-inside",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "box-decoration-break",
        inherited: false,
        initial: "slice",
        animatable: false,
    },
    PropertyMetadata {
        name: "mask-type",
        inherited: false,
        initial: "luminance",
        animatable: false,
    },
    PropertyMetadata {
        name: "field-sizing",
        inherited: false,
        initial: "fixed",
        animatable: false,
    },
    PropertyMetadata {
        name: "shape-outside",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "shape-margin",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "shape-image-threshold",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "anchor-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "view-transition-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "contain-intrinsic-width",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "contain-intrinsic-height",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "content-visibility",
        inherited: false,
        initial: "visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-timeline",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-timeline-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-timeline-axis",
        inherited: false,
        initial: "block",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-box-trim",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-box-edge",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "-webkit-line-clamp",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "alignment-baseline",
        inherited: false,
        initial: "baseline",
        animatable: false,
    },
    PropertyMetadata {
        name: "baseline-shift",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "baseline-source",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "dominant-baseline",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "scroll-marker-group",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "reading-flow",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "position-area",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "position-try-fallbacks",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "position-try-order",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "position-visibility",
        inherited: false,
        initial: "anchors-visible",
        animatable: false,
    },
    PropertyMetadata {
        name: "timeline-scope",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "view-transition-class",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "overlay",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "anchor-scope",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "view-timeline-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "view-timeline-axis",
        inherited: false,
        initial: "block",
        animatable: false,
    },
    PropertyMetadata {
        name: "view-timeline-inset",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "container-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "container-type",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "aspect-ratio",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "unicode-bidi",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-template-columns",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-template-rows",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-template-areas",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-auto-columns",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-auto-rows",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-auto-flow",
        inherited: false,
        initial: "row",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-row-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-row-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-column-start",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "grid-column-end",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "box-shadow",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "position-anchor",
        inherited: false,
        initial: "implicit",
        animatable: false,
    },
    PropertyMetadata {
        name: "contain-intrinsic-block-size",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "contain-intrinsic-inline-size",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "block-size",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "inline-size",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "min-block-size",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "min-inline-size",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "max-block-size",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "max-inline-size",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "margin-inline-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "margin-inline-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "padding-inline-start",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "padding-inline-end",
        inherited: false,
        initial: "0",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-start-width",
        inherited: false,
        initial: "medium",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-start-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-start-color",
        inherited: false,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-end-width",
        inherited: false,
        initial: "medium",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-end-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-block-end-color",
        inherited: false,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-start-width",
        inherited: false,
        initial: "medium",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-start-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-start-color",
        inherited: false,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-end-width",
        inherited: false,
        initial: "medium",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-end-style",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-inline-end-color",
        inherited: false,
        initial: "currentcolor",
        animatable: false,
    },
    PropertyMetadata {
        name: "border-start-start-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-start-end-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-end-start-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "border-end-end-radius",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "speak-as",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "text-spacing",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "line-fit-edge",
        inherited: false,
        initial: "leading",
        animatable: false,
    },
    PropertyMetadata {
        name: "will-change",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "touch-action",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "transform",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "transform-origin",
        inherited: false,
        initial: "50% 50%",
        animatable: true,
    },
    PropertyMetadata {
        name: "translate",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "scale",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "rotate",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "perspective",
        inherited: false,
        initial: "none",
        animatable: true,
    },
    PropertyMetadata {
        name: "animation-name",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-duration",
        inherited: false,
        initial: "0s",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-timing-function",
        inherited: false,
        initial: "ease",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-delay",
        inherited: false,
        initial: "0s",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-iteration-count",
        inherited: false,
        initial: "1",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-direction",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-fill-mode",
        inherited: false,
        initial: "none",
        animatable: false,
    },
    PropertyMetadata {
        name: "animation-play-state",
        inherited: false,
        initial: "running",
        animatable: false,
    },
    PropertyMetadata {
        name: "content",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "transition-behavior",
        inherited: false,
        initial: "normal",
        animatable: false,
    },
    PropertyMetadata {
        name: "outline-offset",
        inherited: false,
        initial: "0",
        animatable: true,
    },
    PropertyMetadata {
        name: "grid-row-gap",
        inherited: false,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "grid-column-gap",
        inherited: false,
        initial: "normal",
        animatable: true,
    },
    PropertyMetadata {
        name: "page-break-before",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "page-break-after",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
    PropertyMetadata {
        name: "page-break-inside",
        inherited: false,
        initial: "auto",
        animatable: false,
    },
];

/// Maps a CSS shorthand property to the ordered list of longhand properties it expands into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShorthandExpansion {
    /// The canonical lowercase name of the shorthand property.
    pub name: &'static str,
    /// The ordered longhand property names this shorthand sets.
    pub longhands: &'static [&'static str],
}

/// Static table of shorthand properties and their corresponding ordered longhands.
static SHORTHAND_EXPANSIONS: &[ShorthandExpansion] = &[
    ShorthandExpansion {
        name: "animation",
        longhands: &[
            "animation-name",
            "animation-duration",
            "animation-timing-function",
            "animation-delay",
            "animation-iteration-count",
            "animation-direction",
            "animation-fill-mode",
            "animation-play-state",
        ],
    },
    ShorthandExpansion {
        name: "background",
        longhands: &[
            "background-color",
            "background-image",
            "background-position",
            "background-size",
            "background-repeat",
            "background-origin",
            "background-clip",
            "background-attachment",
        ],
    },
    ShorthandExpansion {
        name: "background-position",
        longhands: &["background-position-x", "background-position-y"],
    },
    ShorthandExpansion {
        name: "background-repeat",
        longhands: &["background-repeat-x", "background-repeat-y"],
    },
    ShorthandExpansion {
        name: "border",
        longhands: &["border-width", "border-style", "border-color"],
    },
    ShorthandExpansion {
        name: "border-block",
        longhands: &[
            "border-block-start-width",
            "border-block-start-style",
            "border-block-start-color",
            "border-block-end-width",
            "border-block-end-style",
            "border-block-end-color",
        ],
    },
    ShorthandExpansion {
        name: "border-block-color",
        longhands: &["border-block-start-color", "border-block-end-color"],
    },
    ShorthandExpansion {
        name: "border-block-end",
        longhands: &[
            "border-block-end-width",
            "border-block-end-style",
            "border-block-end-color",
        ],
    },
    ShorthandExpansion {
        name: "border-block-start",
        longhands: &[
            "border-block-start-width",
            "border-block-start-style",
            "border-block-start-color",
        ],
    },
    ShorthandExpansion {
        name: "border-block-style",
        longhands: &["border-block-start-style", "border-block-end-style"],
    },
    ShorthandExpansion {
        name: "border-block-width",
        longhands: &["border-block-start-width", "border-block-end-width"],
    },
    ShorthandExpansion {
        name: "border-bottom",
        longhands: &[
            "border-bottom-width",
            "border-bottom-style",
            "border-bottom-color",
        ],
    },
    ShorthandExpansion {
        name: "border-color",
        longhands: &[
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
            "border-left-color",
        ],
    },
    ShorthandExpansion {
        name: "border-image",
        longhands: &[
            "border-image-source",
            "border-image-slice",
            "border-image-width",
            "border-image-outset",
            "border-image-repeat",
        ],
    },
    ShorthandExpansion {
        name: "border-inline",
        longhands: &[
            "border-inline-start-width",
            "border-inline-start-style",
            "border-inline-start-color",
            "border-inline-end-width",
            "border-inline-end-style",
            "border-inline-end-color",
        ],
    },
    ShorthandExpansion {
        name: "border-inline-color",
        longhands: &["border-inline-start-color", "border-inline-end-color"],
    },
    ShorthandExpansion {
        name: "border-inline-end",
        longhands: &[
            "border-inline-end-width",
            "border-inline-end-style",
            "border-inline-end-color",
        ],
    },
    ShorthandExpansion {
        name: "border-inline-start",
        longhands: &[
            "border-inline-start-width",
            "border-inline-start-style",
            "border-inline-start-color",
        ],
    },
    ShorthandExpansion {
        name: "border-inline-style",
        longhands: &["border-inline-start-style", "border-inline-end-style"],
    },
    ShorthandExpansion {
        name: "border-inline-width",
        longhands: &["border-inline-start-width", "border-inline-end-width"],
    },
    ShorthandExpansion {
        name: "border-left",
        longhands: &[
            "border-left-width",
            "border-left-style",
            "border-left-color",
        ],
    },
    ShorthandExpansion {
        name: "border-radius",
        longhands: &[
            "border-top-left-radius",
            "border-top-right-radius",
            "border-bottom-right-radius",
            "border-bottom-left-radius",
        ],
    },
    ShorthandExpansion {
        name: "border-right",
        longhands: &[
            "border-right-width",
            "border-right-style",
            "border-right-color",
        ],
    },
    ShorthandExpansion {
        name: "border-style",
        longhands: &[
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style",
        ],
    },
    ShorthandExpansion {
        name: "border-top",
        longhands: &["border-top-width", "border-top-style", "border-top-color"],
    },
    ShorthandExpansion {
        name: "border-width",
        longhands: &[
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
        ],
    },
    ShorthandExpansion {
        name: "caret",
        longhands: &["caret-color", "caret-shape"],
    },
    ShorthandExpansion {
        name: "column-rule",
        longhands: &[
            "column-rule-width",
            "column-rule-style",
            "column-rule-color",
        ],
    },
    ShorthandExpansion {
        name: "columns",
        longhands: &["column-width", "column-count"],
    },
    ShorthandExpansion {
        name: "contain-intrinsic-size",
        longhands: &["contain-intrinsic-width", "contain-intrinsic-height"],
    },
    ShorthandExpansion {
        name: "container",
        longhands: &["container-name", "container-type"],
    },
    ShorthandExpansion {
        name: "flex",
        longhands: &["flex-grow", "flex-shrink", "flex-basis"],
    },
    ShorthandExpansion {
        name: "flex-flow",
        longhands: &["flex-direction", "flex-wrap"],
    },
    ShorthandExpansion {
        name: "font",
        longhands: &[
            "font-style",
            "font-variant",
            "font-weight",
            "font-size",
            "line-height",
            "font-family",
        ],
    },
    ShorthandExpansion {
        name: "font-synthesis",
        longhands: &[
            "font-synthesis-weight",
            "font-synthesis-style",
            "font-synthesis-small-caps",
        ],
    },
    ShorthandExpansion {
        name: "font-variant",
        longhands: &[
            "font-variant-ligatures",
            "font-variant-caps",
            "font-variant-numeric",
            "font-variant-east-asian",
            "font-variant-alternates",
            "font-variant-position",
            "font-variant-emoji",
        ],
    },
    ShorthandExpansion {
        name: "gap",
        longhands: &["row-gap", "column-gap"],
    },
    ShorthandExpansion {
        name: "grid",
        longhands: &[
            "grid-template-rows",
            "grid-template-columns",
            "grid-template-areas",
            "grid-auto-rows",
            "grid-auto-columns",
            "grid-auto-flow",
        ],
    },
    ShorthandExpansion {
        name: "grid-area",
        longhands: &[
            "grid-row-start",
            "grid-column-start",
            "grid-row-end",
            "grid-column-end",
        ],
    },
    ShorthandExpansion {
        name: "grid-column",
        longhands: &["grid-column-start", "grid-column-end"],
    },
    ShorthandExpansion {
        name: "grid-gap",
        longhands: &["grid-row-gap", "grid-column-gap"],
    },
    ShorthandExpansion {
        name: "grid-row",
        longhands: &["grid-row-start", "grid-row-end"],
    },
    ShorthandExpansion {
        name: "grid-template",
        longhands: &[
            "grid-template-columns",
            "grid-template-rows",
            "grid-template-areas",
        ],
    },
    ShorthandExpansion {
        name: "inset",
        longhands: &["top", "right", "bottom", "left"],
    },
    ShorthandExpansion {
        name: "inset-block",
        longhands: &["inset-block-start", "inset-block-end"],
    },
    ShorthandExpansion {
        name: "inset-inline",
        longhands: &["inset-inline-start", "inset-inline-end"],
    },
    ShorthandExpansion {
        name: "list-style",
        longhands: &["list-style-type", "list-style-position", "list-style-image"],
    },
    ShorthandExpansion {
        name: "margin",
        longhands: &["margin-top", "margin-right", "margin-bottom", "margin-left"],
    },
    ShorthandExpansion {
        name: "margin-block",
        longhands: &["margin-block-start", "margin-block-end"],
    },
    ShorthandExpansion {
        name: "margin-inline",
        longhands: &["margin-inline-start", "margin-inline-end"],
    },
    ShorthandExpansion {
        name: "outline",
        longhands: &["outline-width", "outline-style", "outline-color"],
    },
    ShorthandExpansion {
        name: "overflow",
        longhands: &["overflow-x", "overflow-y"],
    },
    ShorthandExpansion {
        name: "overscroll-behavior",
        longhands: &["overscroll-behavior-x", "overscroll-behavior-y"],
    },
    ShorthandExpansion {
        name: "padding",
        longhands: &[
            "padding-top",
            "padding-right",
            "padding-bottom",
            "padding-left",
        ],
    },
    ShorthandExpansion {
        name: "padding-block",
        longhands: &["padding-block-start", "padding-block-end"],
    },
    ShorthandExpansion {
        name: "padding-inline",
        longhands: &["padding-inline-start", "padding-inline-end"],
    },
    ShorthandExpansion {
        name: "place-content",
        longhands: &["align-content", "justify-content"],
    },
    ShorthandExpansion {
        name: "place-items",
        longhands: &["align-items", "justify-items"],
    },
    ShorthandExpansion {
        name: "place-self",
        longhands: &["align-self", "justify-self"],
    },
    ShorthandExpansion {
        name: "position-try",
        longhands: &["position-try-order", "position-try-fallbacks"],
    },
    ShorthandExpansion {
        name: "scroll-margin",
        longhands: &[
            "scroll-margin-top",
            "scroll-margin-right",
            "scroll-margin-bottom",
            "scroll-margin-left",
        ],
    },
    ShorthandExpansion {
        name: "scroll-margin-block",
        longhands: &["scroll-margin-block-start", "scroll-margin-block-end"],
    },
    ShorthandExpansion {
        name: "scroll-margin-inline",
        longhands: &["scroll-margin-inline-start", "scroll-margin-inline-end"],
    },
    ShorthandExpansion {
        name: "scroll-padding",
        longhands: &[
            "scroll-padding-top",
            "scroll-padding-right",
            "scroll-padding-bottom",
            "scroll-padding-left",
        ],
    },
    ShorthandExpansion {
        name: "scroll-padding-block",
        longhands: &["scroll-padding-block-start", "scroll-padding-block-end"],
    },
    ShorthandExpansion {
        name: "scroll-padding-inline",
        longhands: &["scroll-padding-inline-start", "scroll-padding-inline-end"],
    },
    ShorthandExpansion {
        name: "scroll-timeline",
        longhands: &["scroll-timeline-name", "scroll-timeline-axis"],
    },
    ShorthandExpansion {
        name: "text-decoration",
        longhands: &[
            "text-decoration-line",
            "text-decoration-style",
            "text-decoration-color",
            "text-decoration-thickness",
        ],
    },
    ShorthandExpansion {
        name: "text-emphasis",
        longhands: &["text-emphasis-style", "text-emphasis-color"],
    },
    ShorthandExpansion {
        name: "text-spacing",
        longhands: &["text-autospace", "text-spacing-trim"],
    },
    ShorthandExpansion {
        name: "transition",
        longhands: &[
            "transition-property",
            "transition-duration",
            "transition-timing-function",
            "transition-delay",
        ],
    },
    ShorthandExpansion {
        name: "view-timeline",
        longhands: &["view-timeline-name", "view-timeline-axis"],
    },
];

/// Returns the ordered longhand property names for a shorthand, if `name` is a known shorthand.
/// The lookup is ASCII-case-insensitive, matching `lookup`.
pub fn shorthand_longhands(name: &str) -> Option<&'static [&'static str]> {
    if let Some(sh) = SHORTHAND_EXPANSIONS
        .iter()
        .find(|sh| sh.name.eq_ignore_ascii_case(name))
    {
        return Some(sh.longhands);
    }

    // Edge case / fallback: Strip vendor prefixes and check if unprefixed counterpart exists.
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    for prefix in &["-webkit-", "-moz-", "-ms-", "-o-"] {
        if lower.starts_with(prefix) {
            let unprefixed = &trimmed[prefix.len()..];
            if let Some(sh) = SHORTHAND_EXPANSIONS
                .iter()
                .find(|sh| sh.name.eq_ignore_ascii_case(unprefixed))
            {
                return Some(sh.longhands);
            }
        }
    }

    None
}

/// Looks up the metadata for a CSS property by name.
///
/// This lookup is case-insensitive.
pub fn lookup(name: &str) -> Option<&'static PropertyMetadata> {
    if let Some(prop) = PROPERTY_METADATA
        .iter()
        .find(|prop| prop.name.eq_ignore_ascii_case(name))
    {
        return Some(prop);
    }

    // Edge case / fallback: Strip vendor prefixes and check if unprefixed counterpart exists.
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    for prefix in &["-webkit-", "-moz-", "-ms-", "-o-"] {
        if lower.starts_with(prefix) {
            let unprefixed = &trimmed[prefix.len()..];
            if let Some(prop) = PROPERTY_METADATA
                .iter()
                .find(|prop| prop.name.eq_ignore_ascii_case(unprefixed))
            {
                return Some(prop);
            }
        }
    }

    None
}

/// Checks if a property name is syntactically valid in CSS.
///
/// A property name is valid if it is a registered longhand, a registered shorthand,
/// or a CSS custom property (starts with `--`).
pub fn is_valid_property_name(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.starts_with("--") {
        trimmed.len() > 2
    } else {
        lookup(trimmed).is_some() || shorthand_longhands(trimmed).is_some()
    }
}

/// Checks if a value string is a standard CSS-wide keyword.
///
/// These keywords are: `initial`, `inherit`, `unset`, `revert`, `revert-layer`.
/// This check is ASCII case-insensitive.
pub fn is_css_wide_keyword(value: &str) -> bool {
    let val_trimmed = value.trim();
    val_trimmed.eq_ignore_ascii_case("initial")
        || val_trimmed.eq_ignore_ascii_case("inherit")
        || val_trimmed.eq_ignore_ascii_case("unset")
        || val_trimmed.eq_ignore_ascii_case("revert")
        || val_trimmed.eq_ignore_ascii_case("revert-layer")
}

/// Convenience helper to check if a CSS property is inherited.
///
/// Returns `false` if the property is unknown.
pub fn is_inherited(name: &str) -> bool {
    lookup(name).is_some_and(|prop| prop.inherited)
}

/// Convenience helper to get the canonical initial value for a CSS property.
///
/// Returns `None` if the property is unknown.
pub fn initial_value(name: &str) -> Option<&'static str> {
    lookup(name).map(|prop| prop.initial)
}

/// Convenience helper to check if a CSS property is animatable.
///
/// Returns `false` if the property is unknown.
pub fn is_animatable(name: &str) -> bool {
    lookup(name).is_some_and(|prop| prop.animatable)
}

/// Represents an expanded property longhand with its resolved string value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpandedProperty {
    pub name: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShorthandError {
    InvalidShorthand,
    InvalidValue,
    TooManyValues,
    ZeroValues,
}

/// Expands a shorthand property and its raw whitespace-split values into longhands.
///
/// Handles:
/// - CSS-wide keywords (e.g. `inherit`, `initial`) on any shorthand.
/// - `margin` & `padding` (1 to 4 values).
/// - `border-width`, `border-style`, `border-color` (1 to 4 values).
/// - `border` & `border-top`, `border-right`, `border-bottom`, `border-left` (order-independent properties).
/// - `border-radius` (optional `/` separation for horizontal/vertical corners, 1-4 values each).
pub fn expand_shorthand_values(
    shorthand_name: &str,
    values: &[&str],
) -> Result<Vec<ExpandedProperty>, ShorthandError> {
    let lower_shorthand = shorthand_name.trim().to_ascii_lowercase();

    // Check zero values
    if values.is_empty() {
        return Err(ShorthandError::ZeroValues);
    }

    // Check for CSS-wide keyword as single value
    if values.len() == 1 && is_css_wide_keyword(values[0]) {
        if let Some(longhands) = shorthand_longhands(&lower_shorthand) {
            let kw = values[0].trim().to_ascii_lowercase();
            return Ok(longhands
                .iter()
                .map(|lh| ExpandedProperty {
                    name: lh,
                    value: kw.clone(),
                })
                .collect());
        } else {
            return Err(ShorthandError::InvalidShorthand);
        }
    }

    // If more than 1 value, none can be a CSS-wide keyword
    if values.len() > 1 && values.iter().any(|&v| is_css_wide_keyword(v)) {
        return Err(ShorthandError::InvalidValue);
    }

    match lower_shorthand.as_str() {
        "margin" | "padding" | "border-width" | "border-style" | "border-color" | "inset"
        | "scroll-margin" | "scroll-padding" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 4 {
                return Err(ShorthandError::TooManyValues);
            }
            let (v0, v1, v2, v3) = match values.len() {
                1 => (
                    (*values[0]).to_string(),
                    (*values[0]).to_string(),
                    (*values[0]).to_string(),
                    (*values[0]).to_string(),
                ),
                2 => (
                    (*values[0]).to_string(),
                    (*values[1]).to_string(),
                    (*values[0]).to_string(),
                    (*values[1]).to_string(),
                ),
                3 => (
                    (*values[0]).to_string(),
                    (*values[1]).to_string(),
                    (*values[2]).to_string(),
                    (*values[1]).to_string(),
                ),
                _ => (
                    (*values[0]).to_string(),
                    (*values[1]).to_string(),
                    (*values[2]).to_string(),
                    (*values[3]).to_string(),
                ),
            };
            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: v0,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: v1,
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: v2,
                },
                ExpandedProperty {
                    name: longhands[3],
                    value: v3,
                },
            ])
        }
        "border-top"
        | "border-right"
        | "border-bottom"
        | "border-left"
        | "border-block-start"
        | "border-block-end"
        | "border-inline-start"
        | "border-inline-end" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut width = None;
            let mut style = None;
            let mut color = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_border_style_keyword(&lower) {
                    if style.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    style = Some(val.to_string());
                } else if is_border_width_keyword(&lower) || is_length_value(&lower) {
                    if width.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    width = Some(val.to_string());
                } else {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                }
            }

            let w = width.unwrap_or_else(|| "medium".to_string());
            let s = style.unwrap_or_else(|| "none".to_string());
            let c = color.unwrap_or_else(|| "currentcolor".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: w,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: s,
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: c,
                },
            ])
        }
        "border" => {
            // border sets all 4 edges
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut width = None;
            let mut style = None;
            let mut color = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_border_style_keyword(&lower) {
                    if style.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    style = Some(val.to_string());
                } else if is_border_width_keyword(&lower) || is_length_value(&lower) {
                    if width.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    width = Some(val.to_string());
                } else {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                }
            }

            let w = width.unwrap_or_else(|| "medium".to_string());
            let s = style.unwrap_or_else(|| "none".to_string());
            let c = color.unwrap_or_else(|| "currentcolor".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: "border-top-width",
                    value: w.clone(),
                },
                ExpandedProperty {
                    name: "border-top-style",
                    value: s.clone(),
                },
                ExpandedProperty {
                    name: "border-top-color",
                    value: c.clone(),
                },
                ExpandedProperty {
                    name: "border-right-width",
                    value: w.clone(),
                },
                ExpandedProperty {
                    name: "border-right-style",
                    value: s.clone(),
                },
                ExpandedProperty {
                    name: "border-right-color",
                    value: c.clone(),
                },
                ExpandedProperty {
                    name: "border-bottom-width",
                    value: w.clone(),
                },
                ExpandedProperty {
                    name: "border-bottom-style",
                    value: s.clone(),
                },
                ExpandedProperty {
                    name: "border-bottom-color",
                    value: c.clone(),
                },
                ExpandedProperty {
                    name: "border-left-width",
                    value: w.clone(),
                },
                ExpandedProperty {
                    name: "border-left-style",
                    value: s.clone(),
                },
                ExpandedProperty {
                    name: "border-left-color",
                    value: c.clone(),
                },
            ])
        }
        "border-radius" => {
            let slash_idx = values.iter().position(|&v| v == "/");
            let (h_raw, v_raw) = match slash_idx {
                Some(idx) => (&values[..idx], Some(&values[idx + 1..])),
                None => (values, None),
            };

            if h_raw.is_empty() || h_raw.len() > 4 || h_raw.contains(&"/") {
                return Err(ShorthandError::InvalidValue);
            }
            if v_raw.is_some_and(|v| v.is_empty() || v.len() > 4 || v.contains(&"/")) {
                return Err(ShorthandError::InvalidValue);
            }

            let h_expanded = expand_radius_1_to_4(h_raw);
            let v_expanded = v_raw.map(expand_radius_1_to_4);

            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;

            let build_val = |idx: usize| -> String {
                let h = &h_expanded[idx];
                if let Some(ref v) = v_expanded {
                    format!("{} {}", h, v[idx])
                } else {
                    h.clone()
                }
            };

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: build_val(0),
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: build_val(1),
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: build_val(2),
                },
                ExpandedProperty {
                    name: longhands[3],
                    value: build_val(3),
                },
            ])
        }
        "margin-block"
        | "margin-inline"
        | "padding-block"
        | "padding-inline"
        | "inset-block"
        | "inset-inline"
        | "scroll-margin-block"
        | "scroll-margin-inline"
        | "scroll-padding-block"
        | "scroll-padding-inline"
        | "contain-intrinsic-size"
        | "background-position"
        | "background-repeat"
        | "overscroll-behavior"
        | "gap"
        | "grid-gap"
        | "place-content"
        | "place-items"
        | "place-self"
        | "border-block-color"
        | "border-block-style"
        | "border-block-width"
        | "border-inline-color"
        | "border-inline-style"
        | "border-inline-width"
        | "text-spacing"
        | "overflow" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 2 {
                return Err(ShorthandError::TooManyValues);
            }
            let v0 = (*values[0]).to_string();
            let v1 = if values.len() > 1 {
                (*values[1]).to_string()
            } else {
                (*values[0]).to_string()
            };
            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: v0,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: v1,
                },
            ])
        }
        "scroll-timeline" | "view-timeline" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 2 {
                return Err(ShorthandError::TooManyValues);
            }
            let v0 = (*values[0]).to_string();
            let v1 = if values.len() > 1 {
                (*values[1]).to_string()
            } else {
                "block".to_string()
            };
            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: v0,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: v1,
                },
            ])
        }
        "border-block" | "border-inline" => {
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut width = None;
            let mut style = None;
            let mut color = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_border_style_keyword(&lower) {
                    if style.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    style = Some(val.to_string());
                } else if is_border_width_keyword(&lower) || is_length_value(&lower) {
                    if width.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    width = Some(val.to_string());
                } else {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                }
            }

            let w = width.unwrap_or_else(|| "medium".to_string());
            let s = style.unwrap_or_else(|| "none".to_string());
            let c = color.unwrap_or_else(|| "currentcolor".to_string());

            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: w.clone(),
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: s.clone(),
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: c.clone(),
                },
                ExpandedProperty {
                    name: longhands[3],
                    value: w,
                },
                ExpandedProperty {
                    name: longhands[4],
                    value: s,
                },
                ExpandedProperty {
                    name: longhands[5],
                    value: c,
                },
            ])
        }
        "outline" => {
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut width = None;
            let mut style = None;
            let mut color = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_border_style_keyword(&lower) || lower == "auto" {
                    if style.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    style = Some(val.to_string());
                } else if is_border_width_keyword(&lower) || is_length_value(&lower) {
                    if width.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    width = Some(val.to_string());
                } else {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                }
            }

            let w = width.unwrap_or_else(|| "medium".to_string());
            let s = style.unwrap_or_else(|| "none".to_string());
            let c = color.unwrap_or_else(|| "currentcolor".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: "outline-width",
                    value: w,
                },
                ExpandedProperty {
                    name: "outline-style",
                    value: s,
                },
                ExpandedProperty {
                    name: "outline-color",
                    value: c,
                },
            ])
        }
        "text-emphasis" => {
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut style_parts = Vec::new();
            let mut color = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_color_token(&lower) {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                } else {
                    style_parts.push(val.to_string());
                }
            }

            let s = if style_parts.is_empty() {
                "none".to_string()
            } else {
                style_parts.join(" ")
            };
            let c = color.unwrap_or_else(|| "currentcolor".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: "text-emphasis-style",
                    value: s,
                },
                ExpandedProperty {
                    name: "text-emphasis-color",
                    value: c,
                },
            ])
        }
        "caret" => {
            if values.len() > 2 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut color = None;
            let mut shape = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if lower == "auto" {
                    // auto can be both. handled as None -> auto below.
                } else if matches!(lower.as_str(), "bar" | "block" | "underscore") {
                    if shape.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    shape = Some(val.to_string());
                } else if is_color_token(&lower) {
                    if color.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    color = Some(val.to_string());
                } else {
                    return Err(ShorthandError::InvalidValue);
                }
            }

            let c = color.unwrap_or_else(|| "auto".to_string());
            let s = shape.unwrap_or_else(|| "auto".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: "caret-color",
                    value: c,
                },
                ExpandedProperty {
                    name: "caret-shape",
                    value: s,
                },
            ])
        }
        "columns" => {
            if values.len() > 2 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut width = None;
            let mut count = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if lower == "auto" {
                    // handled as None -> auto below.
                } else {
                    let is_int = {
                        let mut chars = lower.chars();
                        if let Some(first) = chars.next() {
                            let rest = if first == '+' || first == '-' {
                                chars.as_str()
                            } else {
                                &lower
                            };
                            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
                        } else {
                            false
                        }
                    };

                    if is_int {
                        if count.is_some() {
                            return Err(ShorthandError::InvalidValue);
                        }
                        count = Some(val.to_string());
                    } else if is_length_value(&lower) {
                        if width.is_some() {
                            return Err(ShorthandError::InvalidValue);
                        }
                        width = Some(val.to_string());
                    } else {
                        return Err(ShorthandError::InvalidValue);
                    }
                }
            }

            let w = width.unwrap_or_else(|| "auto".to_string());
            let c = count.unwrap_or_else(|| "auto".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: "column-width",
                    value: w,
                },
                ExpandedProperty {
                    name: "column-count",
                    value: c,
                },
            ])
        }
        "font-synthesis" => {
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            if values
                .iter()
                .any(|&v| v.trim().eq_ignore_ascii_case("none"))
            {
                if values.len() == 1 {
                    return Ok(vec![
                        ExpandedProperty {
                            name: "font-synthesis-weight",
                            value: "none".to_string(),
                        },
                        ExpandedProperty {
                            name: "font-synthesis-style",
                            value: "none".to_string(),
                        },
                        ExpandedProperty {
                            name: "font-synthesis-small-caps",
                            value: "none".to_string(),
                        },
                    ]);
                } else {
                    return Err(ShorthandError::InvalidValue);
                }
            }

            let mut weight = "none".to_string();
            let mut style = "none".to_string();
            let mut small_caps = "none".to_string();

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                match lower.as_str() {
                    "weight" => weight = "auto".to_string(),
                    "style" => style = "auto".to_string(),
                    "small-caps" => small_caps = "auto".to_string(),
                    _ => return Err(ShorthandError::InvalidValue),
                }
            }

            Ok(vec![
                ExpandedProperty {
                    name: "font-synthesis-weight",
                    value: weight,
                },
                ExpandedProperty {
                    name: "font-synthesis-style",
                    value: style,
                },
                ExpandedProperty {
                    name: "font-synthesis-small-caps",
                    value: small_caps,
                },
            ])
        }
        "flex-flow" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 2 {
                return Err(ShorthandError::TooManyValues);
            }
            let mut direction = None;
            let mut wrap = None;
            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if matches!(
                    lower.as_str(),
                    "row" | "row-reverse" | "column" | "column-reverse"
                ) {
                    if direction.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    direction = Some(val.to_string());
                } else if matches!(lower.as_str(), "nowrap" | "wrap" | "wrap-reverse") {
                    if wrap.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    wrap = Some(val.to_string());
                } else {
                    return Err(ShorthandError::InvalidValue);
                }
            }
            let dir = direction.unwrap_or_else(|| "row".to_string());
            let wrp = wrap.unwrap_or_else(|| "nowrap".to_string());
            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: dir,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: wrp,
                },
            ])
        }
        "flex" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }
            if values.len() == 1 && values[0].trim().eq_ignore_ascii_case("none") {
                return Ok(vec![
                    ExpandedProperty {
                        name: "flex-grow",
                        value: "0".to_string(),
                    },
                    ExpandedProperty {
                        name: "flex-shrink",
                        value: "0".to_string(),
                    },
                    ExpandedProperty {
                        name: "flex-basis",
                        value: "auto".to_string(),
                    },
                ]);
            }

            let mut grow = None;
            let mut shrink = None;
            let mut basis = None;

            if values.len() == 1 {
                let val = values[0];
                let lower = val.trim().to_ascii_lowercase();
                if lower.parse::<f64>().is_ok() {
                    grow = Some(val.to_string());
                    shrink = Some("1".to_string());
                    basis = Some("0%".to_string());
                } else {
                    grow = Some("1".to_string());
                    shrink = Some("1".to_string());
                    basis = Some(val.to_string());
                }
            } else if values.len() == 2 {
                let val0 = values[0];
                let val1 = values[1];
                let lower1 = val1.trim().to_ascii_lowercase();
                if lower1.parse::<f64>().is_ok() {
                    grow = Some(val0.to_string());
                    shrink = Some(val1.to_string());
                    basis = Some("0%".to_string());
                } else {
                    grow = Some(val0.to_string());
                    shrink = Some("1".to_string());
                    basis = Some(val1.to_string());
                }
            } else if values.len() == 3 {
                grow = Some(values[0].to_string());
                shrink = Some(values[1].to_string());
                basis = Some(values[2].to_string());
            }

            let g = grow.ok_or(ShorthandError::InvalidValue)?;
            let s = shrink.ok_or(ShorthandError::InvalidValue)?;
            let b = basis.ok_or(ShorthandError::InvalidValue)?;

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: g,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: s,
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: b,
                },
            ])
        }
        "grid-column" | "grid-row" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            let slash_idx = values.iter().position(|&v| v == "/");
            let (start_vals, end_vals) = match slash_idx {
                Some(idx) => (&values[..idx], Some(&values[idx + 1..])),
                None => (values, None),
            };

            if start_vals.len() != 1 {
                return Err(ShorthandError::InvalidValue);
            }
            if end_vals.is_some_and(|ev| ev.len() != 1) {
                return Err(ShorthandError::InvalidValue);
            }

            let start_val = start_vals[0].to_string();
            let end_val = match end_vals {
                Some(ev) => ev[0].to_string(),
                None => "auto".to_string(),
            };

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: start_val,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: end_val,
                },
            ])
        }
        "list-style" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 3 {
                return Err(ShorthandError::TooManyValues);
            }

            let mut list_type = None;
            let mut position = None;
            let mut image = None;

            if values.len() == 1 && values[0].trim().eq_ignore_ascii_case("none") {
                list_type = Some("none".to_string());
                image = Some("none".to_string());
            } else {
                for &val in values {
                    let lower = val.trim().to_ascii_lowercase();
                    if matches!(lower.as_str(), "inside" | "outside") {
                        if position.is_some() {
                            return Err(ShorthandError::InvalidValue);
                        }
                        position = Some(val.to_string());
                    } else if lower.starts_with("url(")
                        || lower.starts_with("image(")
                        || lower.starts_with("linear-gradient(")
                    {
                        if image.is_some() {
                            return Err(ShorthandError::InvalidValue);
                        }
                        image = Some(val.to_string());
                    } else {
                        if list_type.is_some() {
                            return Err(ShorthandError::InvalidValue);
                        }
                        list_type = Some(val.to_string());
                    }
                }
            }

            let t = list_type.unwrap_or_else(|| "disc".to_string());
            let p = position.unwrap_or_else(|| "outside".to_string());
            let img = image.unwrap_or_else(|| "none".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: t,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: p,
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: img,
                },
            ])
        }
        "transition" => {
            let longhands =
                shorthand_longhands(&lower_shorthand).ok_or(ShorthandError::InvalidShorthand)?;
            if values.len() > 4 {
                return Err(ShorthandError::TooManyValues);
            }

            let mut property = None;
            let mut duration = None;
            let mut timing_fn = None;
            let mut delay = None;

            for &val in values {
                let lower = val.trim().to_ascii_lowercase();
                if is_time_value(&lower) {
                    if duration.is_none() {
                        duration = Some(val.to_string());
                    } else if delay.is_none() {
                        delay = Some(val.to_string());
                    } else {
                        return Err(ShorthandError::InvalidValue);
                    }
                } else if is_timing_function_value(&lower) {
                    if timing_fn.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    timing_fn = Some(val.to_string());
                } else {
                    if property.is_some() {
                        return Err(ShorthandError::InvalidValue);
                    }
                    property = Some(val.to_string());
                }
            }

            let prop = property.unwrap_or_else(|| "all".to_string());
            let dur = duration.unwrap_or_else(|| "0s".to_string());
            let tf = timing_fn.unwrap_or_else(|| "ease".to_string());
            let dl = delay.unwrap_or_else(|| "0s".to_string());

            Ok(vec![
                ExpandedProperty {
                    name: longhands[0],
                    value: prop,
                },
                ExpandedProperty {
                    name: longhands[1],
                    value: dur,
                },
                ExpandedProperty {
                    name: longhands[2],
                    value: tf,
                },
                ExpandedProperty {
                    name: longhands[3],
                    value: dl,
                },
            ])
        }
        _ => Err(ShorthandError::InvalidShorthand),
    }
}

fn is_border_style_keyword(s: &str) -> bool {
    matches!(
        s,
        "none"
            | "hidden"
            | "dotted"
            | "dashed"
            | "solid"
            | "double"
            | "groove"
            | "ridge"
            | "inset"
            | "outset"
    )
}

fn is_border_width_keyword(s: &str) -> bool {
    matches!(s, "thin" | "medium" | "thick")
}

fn is_length_value(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed == "0" {
        return true;
    }
    if trimmed.is_empty() {
        return false;
    }
    let bytes = trimmed.as_bytes();
    let first = bytes[0];
    if first.is_ascii_digit() || first == b'+' || first == b'-' || first == b'.' {
        trimmed.chars().any(|c| c.is_ascii_digit())
    } else {
        false
    }
}

fn expand_radius_1_to_4(values: &[&str]) -> [String; 4] {
    match values.len() {
        1 => [
            (*values[0]).to_string(),
            (*values[0]).to_string(),
            (*values[0]).to_string(),
            (*values[0]).to_string(),
        ],
        2 => [
            (*values[0]).to_string(),
            (*values[1]).to_string(),
            (*values[0]).to_string(),
            (*values[1]).to_string(),
        ],
        3 => [
            (*values[0]).to_string(),
            (*values[1]).to_string(),
            (*values[2]).to_string(),
            (*values[1]).to_string(),
        ],
        _ => [
            (*values[0]).to_string(),
            (*values[1]).to_string(),
            (*values[2]).to_string(),
            (*values[3]).to_string(),
        ],
    }
}

fn is_time_value(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    for unit in &["ms", "s"] {
        if lower.ends_with(unit) {
            let num_part = &lower[..lower.len() - unit.len()];
            if num_part.parse::<f64>().is_ok() {
                return true;
            }
        }
    }
    false
}

fn is_timing_function_value(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out" | "step-start" | "step-end"
    ) || lower.starts_with("cubic-bezier(")
        || lower.starts_with("steps(")
        || lower.starts_with("linear(")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalDirection {
    BlockStart,
    BlockEnd,
    InlineStart,
    InlineEnd,
}

pub fn resolve_logical_direction(
    logical: LogicalDirection,
    writing_mode: &str,
    direction: &str,
) -> &'static str {
    let wm = writing_mode.trim().to_ascii_lowercase();
    let dir = direction.trim().to_ascii_lowercase();
    let is_rtl = dir == "rtl";

    match wm.as_str() {
        "vertical-rl" | "sideways-rl" => match logical {
            LogicalDirection::BlockStart => "right",
            LogicalDirection::BlockEnd => "left",
            LogicalDirection::InlineStart => {
                if is_rtl {
                    "bottom"
                } else {
                    "top"
                }
            }
            LogicalDirection::InlineEnd => {
                if is_rtl {
                    "top"
                } else {
                    "bottom"
                }
            }
        },
        "vertical-lr" | "sideways-lr" => match logical {
            LogicalDirection::BlockStart => "left",
            LogicalDirection::BlockEnd => "right",
            LogicalDirection::InlineStart => {
                if is_rtl {
                    "bottom"
                } else {
                    "top"
                }
            }
            LogicalDirection::InlineEnd => {
                if is_rtl {
                    "top"
                } else {
                    "bottom"
                }
            }
        },
        _ => {
            // horizontal-tb or fallback
            match logical {
                LogicalDirection::BlockStart => "top",
                LogicalDirection::BlockEnd => "bottom",
                LogicalDirection::InlineStart => {
                    if is_rtl {
                        "right"
                    } else {
                        "left"
                    }
                }
                LogicalDirection::InlineEnd => {
                    if is_rtl {
                        "left"
                    } else {
                        "right"
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalCorner {
    StartStart,
    StartEnd,
    EndStart,
    EndEnd,
}

pub fn resolve_logical_corner(
    corner: LogicalCorner,
    writing_mode: &str,
    direction: &str,
) -> &'static str {
    let wm = writing_mode.trim().to_ascii_lowercase();
    let dir = direction.trim().to_ascii_lowercase();
    let is_rtl = dir == "rtl";

    match wm.as_str() {
        "vertical-rl" | "sideways-rl" => match corner {
            LogicalCorner::StartStart => {
                if is_rtl {
                    "border-bottom-right-radius"
                } else {
                    "border-top-right-radius"
                }
            }
            LogicalCorner::StartEnd => {
                if is_rtl {
                    "border-top-right-radius"
                } else {
                    "border-bottom-right-radius"
                }
            }
            LogicalCorner::EndStart => {
                if is_rtl {
                    "border-bottom-left-radius"
                } else {
                    "border-top-left-radius"
                }
            }
            LogicalCorner::EndEnd => {
                if is_rtl {
                    "border-top-left-radius"
                } else {
                    "border-bottom-left-radius"
                }
            }
        },
        "vertical-lr" | "sideways-lr" => match corner {
            LogicalCorner::StartStart => {
                if is_rtl {
                    "border-bottom-left-radius"
                } else {
                    "border-top-left-radius"
                }
            }
            LogicalCorner::StartEnd => {
                if is_rtl {
                    "border-top-left-radius"
                } else {
                    "border-bottom-left-radius"
                }
            }
            LogicalCorner::EndStart => {
                if is_rtl {
                    "border-bottom-right-radius"
                } else {
                    "border-top-right-radius"
                }
            }
            LogicalCorner::EndEnd => {
                if is_rtl {
                    "border-top-right-radius"
                } else {
                    "border-bottom-right-radius"
                }
            }
        },
        _ => {
            // horizontal-tb or fallback
            match corner {
                LogicalCorner::StartStart => {
                    if is_rtl {
                        "border-top-right-radius"
                    } else {
                        "border-top-left-radius"
                    }
                }
                LogicalCorner::StartEnd => {
                    if is_rtl {
                        "border-top-left-radius"
                    } else {
                        "border-top-right-radius"
                    }
                }
                LogicalCorner::EndStart => {
                    if is_rtl {
                        "border-bottom-right-radius"
                    } else {
                        "border-bottom-left-radius"
                    }
                }
                LogicalCorner::EndEnd => {
                    if is_rtl {
                        "border-bottom-left-radius"
                    } else {
                        "border-bottom-right-radius"
                    }
                }
            }
        }
    }
}

pub fn map_logical_to_physical(
    property: &str,
    writing_mode: &str,
    direction: &str,
) -> Option<&'static str> {
    let prop = property.trim().to_ascii_lowercase();
    let is_vertical = matches!(
        writing_mode.trim().to_ascii_lowercase().as_str(),
        "vertical-rl" | "vertical-lr" | "sideways-rl" | "sideways-lr"
    );

    match prop.as_str() {
        // Sizing
        "block-size" => Some(if is_vertical { "width" } else { "height" }),
        "inline-size" => Some(if is_vertical { "height" } else { "width" }),
        "min-block-size" => Some(if is_vertical {
            "min-width"
        } else {
            "min-height"
        }),
        "min-inline-size" => Some(if is_vertical {
            "min-height"
        } else {
            "min-width"
        }),
        "max-block-size" => Some(if is_vertical {
            "max-width"
        } else {
            "max-height"
        }),
        "max-inline-size" => Some(if is_vertical {
            "max-height"
        } else {
            "max-width"
        }),

        // Margins
        "margin-block-start" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockStart, writing_mode, direction);
            Some(match dir {
                "top" => "margin-top",
                "right" => "margin-right",
                "bottom" => "margin-bottom",
                "left" => "margin-left",
                _ => "margin-top",
            })
        }
        "margin-block-end" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockEnd, writing_mode, direction);
            Some(match dir {
                "top" => "margin-top",
                "right" => "margin-right",
                "bottom" => "margin-bottom",
                "left" => "margin-left",
                _ => "margin-bottom",
            })
        }
        "margin-inline-start" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineStart, writing_mode, direction);
            Some(match dir {
                "top" => "margin-top",
                "right" => "margin-right",
                "bottom" => "margin-bottom",
                "left" => "margin-left",
                _ => "margin-left",
            })
        }
        "margin-inline-end" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineEnd, writing_mode, direction);
            Some(match dir {
                "top" => "margin-top",
                "right" => "margin-right",
                "bottom" => "margin-bottom",
                "left" => "margin-left",
                _ => "margin-right",
            })
        }

        // Paddings
        "padding-block-start" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockStart, writing_mode, direction);
            Some(match dir {
                "top" => "padding-top",
                "right" => "padding-right",
                "bottom" => "padding-bottom",
                "left" => "padding-left",
                _ => "padding-top",
            })
        }
        "padding-block-end" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockEnd, writing_mode, direction);
            Some(match dir {
                "top" => "padding-top",
                "right" => "padding-right",
                "bottom" => "padding-bottom",
                "left" => "padding-left",
                _ => "padding-bottom",
            })
        }
        "padding-inline-start" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineStart, writing_mode, direction);
            Some(match dir {
                "top" => "padding-top",
                "right" => "padding-right",
                "bottom" => "padding-bottom",
                "left" => "padding-left",
                _ => "padding-left",
            })
        }
        "padding-inline-end" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineEnd, writing_mode, direction);
            Some(match dir {
                "top" => "padding-top",
                "right" => "padding-right",
                "bottom" => "padding-bottom",
                "left" => "padding-left",
                _ => "padding-right",
            })
        }

        // Insets
        "inset-block-start" => Some(resolve_logical_direction(
            LogicalDirection::BlockStart,
            writing_mode,
            direction,
        )),
        "inset-block-end" => Some(resolve_logical_direction(
            LogicalDirection::BlockEnd,
            writing_mode,
            direction,
        )),
        "inset-inline-start" => Some(resolve_logical_direction(
            LogicalDirection::InlineStart,
            writing_mode,
            direction,
        )),
        "inset-inline-end" => Some(resolve_logical_direction(
            LogicalDirection::InlineEnd,
            writing_mode,
            direction,
        )),

        // Border Widths
        "border-block-start-width" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-width",
                "right" => "border-right-width",
                "bottom" => "border-bottom-width",
                "left" => "border-left-width",
                _ => "border-top-width",
            })
        }
        "border-block-end-width" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-width",
                "right" => "border-right-width",
                "bottom" => "border-bottom-width",
                "left" => "border-left-width",
                _ => "border-bottom-width",
            })
        }
        "border-inline-start-width" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-width",
                "right" => "border-right-width",
                "bottom" => "border-bottom-width",
                "left" => "border-left-width",
                _ => "border-left-width",
            })
        }
        "border-inline-end-width" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-width",
                "right" => "border-right-width",
                "bottom" => "border-bottom-width",
                "left" => "border-left-width",
                _ => "border-right-width",
            })
        }

        // Border Styles
        "border-block-start-style" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-style",
                "right" => "border-right-style",
                "bottom" => "border-bottom-style",
                "left" => "border-left-style",
                _ => "border-top-style",
            })
        }
        "border-block-end-style" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-style",
                "right" => "border-right-style",
                "bottom" => "border-bottom-style",
                "left" => "border-left-style",
                _ => "border-bottom-style",
            })
        }
        "border-inline-start-style" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-style",
                "right" => "border-right-style",
                "bottom" => "border-bottom-style",
                "left" => "border-left-style",
                _ => "border-left-style",
            })
        }
        "border-inline-end-style" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-style",
                "right" => "border-right-style",
                "bottom" => "border-bottom-style",
                "left" => "border-left-style",
                _ => "border-right-style",
            })
        }

        // Border Colors
        "border-block-start-color" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-color",
                "right" => "border-right-color",
                "bottom" => "border-bottom-color",
                "left" => "border-left-color",
                _ => "border-top-color",
            })
        }
        "border-block-end-color" => {
            let dir =
                resolve_logical_direction(LogicalDirection::BlockEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-color",
                "right" => "border-right-color",
                "bottom" => "border-bottom-color",
                "left" => "border-left-color",
                _ => "border-bottom-color",
            })
        }
        "border-inline-start-color" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineStart, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-color",
                "right" => "border-right-color",
                "bottom" => "border-bottom-color",
                "left" => "border-left-color",
                _ => "border-left-color",
            })
        }
        "border-inline-end-color" => {
            let dir =
                resolve_logical_direction(LogicalDirection::InlineEnd, writing_mode, direction);
            Some(match dir {
                "top" => "border-top-color",
                "right" => "border-right-color",
                "bottom" => "border-bottom-color",
                "left" => "border-left-color",
                _ => "border-right-color",
            })
        }

        // Corner Radii
        "border-start-start-radius" => Some(resolve_logical_corner(
            LogicalCorner::StartStart,
            writing_mode,
            direction,
        )),
        "border-start-end-radius" => Some(resolve_logical_corner(
            LogicalCorner::StartEnd,
            writing_mode,
            direction,
        )),
        "border-end-start-radius" => Some(resolve_logical_corner(
            LogicalCorner::EndStart,
            writing_mode,
            direction,
        )),
        "border-end-end-radius" => Some(resolve_logical_corner(
            LogicalCorner::EndEnd,
            writing_mode,
            direction,
        )),

        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPropertyRegistration {
    pub name: String,
    pub syntax: String,
    pub inherits: bool,
    pub initial_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomPropertyValidationError {
    InvalidName,
    InvalidSyntax,
    MissingInitialValue,
    InvalidInitialValue,
}

pub fn validate_custom_property_registration(
    reg: &CustomPropertyRegistration,
) -> Result<(), CustomPropertyValidationError> {
    let trimmed_name = reg.name.trim();
    if !trimmed_name.starts_with("--") || trimmed_name.len() <= 2 {
        return Err(CustomPropertyValidationError::InvalidName);
    }

    let trimmed_syntax = reg.syntax.trim();
    if trimmed_syntax.is_empty() {
        return Err(CustomPropertyValidationError::InvalidSyntax);
    }

    if trimmed_syntax == "*" {
        return Ok(());
    }

    let components = parse_syntax_components(trimmed_syntax)?;

    let val_str = match &reg.initial_value {
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                return Err(CustomPropertyValidationError::MissingInitialValue);
            }
            trimmed
        }
        None => return Err(CustomPropertyValidationError::MissingInitialValue),
    };

    let mut matches_any = false;
    for comp in &components {
        if validate_value_by_syntax_component(val_str, comp) {
            matches_any = true;
            break;
        }
    }

    if !matches_any {
        return Err(CustomPropertyValidationError::InvalidInitialValue);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxType {
    Length,
    Number,
    Percentage,
    LengthPercentage,
    Color,
    Image,
    Url,
    Integer,
    Angle,
    Time,
    Resolution,
    TransformFunction,
    TransformList,
    CustomIdent,
    Keyword(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxMultiplier {
    None,
    SpaceSeparated,
    CommaSeparated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxComponent {
    pub ty: SyntaxType,
    pub multiplier: SyntaxMultiplier,
}

fn parse_syntax_components(
    syntax: &str,
) -> Result<Vec<SyntaxComponent>, CustomPropertyValidationError> {
    let mut comps = Vec::new();
    for part in syntax.split('|') {
        let trimmed_part = part.trim();
        if trimmed_part.is_empty() {
            return Err(CustomPropertyValidationError::InvalidSyntax);
        }

        let (base_part, multiplier) = if let Some(stripped) = trimmed_part.strip_suffix('+') {
            (stripped.trim(), SyntaxMultiplier::SpaceSeparated)
        } else if let Some(stripped) = trimmed_part.strip_suffix('#') {
            (stripped.trim(), SyntaxMultiplier::CommaSeparated)
        } else {
            (trimmed_part, SyntaxMultiplier::None)
        };

        if base_part.is_empty() {
            return Err(CustomPropertyValidationError::InvalidSyntax);
        }

        let ty = if base_part.starts_with('<') && base_part.ends_with('>') {
            let inner = base_part[1..base_part.len() - 1].trim();
            match inner {
                "length" => SyntaxType::Length,
                "number" => SyntaxType::Number,
                "percentage" => SyntaxType::Percentage,
                "length-percentage" => SyntaxType::LengthPercentage,
                "color" => SyntaxType::Color,
                "image" => SyntaxType::Image,
                "url" => SyntaxType::Url,
                "integer" => SyntaxType::Integer,
                "angle" => SyntaxType::Angle,
                "time" => SyntaxType::Time,
                "resolution" => SyntaxType::Resolution,
                "transform-function" => SyntaxType::TransformFunction,
                "transform-list" => SyntaxType::TransformList,
                "custom-ident" => SyntaxType::CustomIdent,
                _ => return Err(CustomPropertyValidationError::InvalidSyntax),
            }
        } else {
            if is_valid_css_identifier(base_part) {
                SyntaxType::Keyword(base_part.to_string())
            } else {
                return Err(CustomPropertyValidationError::InvalidSyntax);
            }
        };

        comps.push(SyntaxComponent { ty, multiplier });
    }

    if comps.is_empty() {
        return Err(CustomPropertyValidationError::InvalidSyntax);
    }

    Ok(comps)
}

fn is_valid_css_identifier(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }
    if is_css_wide_keyword(trimmed) {
        return false;
    }
    let bytes = trimmed.as_bytes();
    if bytes[0].is_ascii_digit() {
        return false;
    }
    if bytes[0] == b'-' && bytes.len() > 1 && bytes[1].is_ascii_digit() {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn validate_value_by_syntax_component(val: &str, comp: &SyntaxComponent) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return false;
    }

    match comp.multiplier {
        SyntaxMultiplier::None => validate_single_value_by_type(trimmed, &comp.ty),
        SyntaxMultiplier::SpaceSeparated => {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                return false;
            }
            parts
                .into_iter()
                .all(|part| validate_single_value_by_type(part, &comp.ty))
        }
        SyntaxMultiplier::CommaSeparated => {
            let parts: Vec<&str> = trimmed.split(',').collect();
            if parts.is_empty() {
                return false;
            }
            parts
                .into_iter()
                .all(|part| validate_single_value_by_type(part.trim(), &comp.ty))
        }
    }
}

fn validate_single_value_by_type(val: &str, ty: &SyntaxType) -> bool {
    let lower = val.trim().to_ascii_lowercase();
    match ty {
        SyntaxType::Length => is_length_token(&lower),
        SyntaxType::Number => lower.parse::<f64>().is_ok(),
        SyntaxType::Percentage => {
            if lower.ends_with('%') {
                lower[..lower.len() - 1].parse::<f64>().is_ok()
            } else {
                false
            }
        }
        SyntaxType::LengthPercentage => {
            is_length_token(&lower) || {
                if lower.ends_with('%') {
                    lower[..lower.len() - 1].parse::<f64>().is_ok()
                } else {
                    false
                }
            }
        }
        SyntaxType::Color => is_color_token(&lower),
        SyntaxType::Image => is_image_token(&lower),
        SyntaxType::Url => is_url_token(&lower),
        SyntaxType::Integer => {
            if let Some(first) = lower.chars().next() {
                let rest = if first == '+' || first == '-' {
                    &lower[1..]
                } else {
                    &lower
                };
                !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
            } else {
                false
            }
        }
        SyntaxType::Angle => {
            for unit in &["deg", "grad", "rad", "turn"] {
                if lower.ends_with(unit) {
                    return lower[..lower.len() - unit.len()].parse::<f64>().is_ok();
                }
            }
            false
        }
        SyntaxType::Time => {
            for unit in &["s", "ms"] {
                if lower.ends_with(unit) {
                    return lower[..lower.len() - unit.len()].parse::<f64>().is_ok();
                }
            }
            false
        }
        SyntaxType::Resolution => {
            for unit in &["dpi", "dpcm", "dppx"] {
                if lower.ends_with(unit) {
                    return lower[..lower.len() - unit.len()].parse::<f64>().is_ok();
                }
            }
            false
        }
        SyntaxType::TransformFunction => is_transform_function_token(&lower),
        SyntaxType::TransformList => {
            let parts: Vec<&str> = lower.split_whitespace().collect();
            if parts.is_empty() {
                return false;
            }
            parts.into_iter().all(is_transform_function_token)
        }
        SyntaxType::CustomIdent => is_custom_ident_token(&lower),
        SyntaxType::Keyword(kw) => is_keyword_token(&lower, kw),
    }
}

fn is_length_token(s: &str) -> bool {
    if s == "0" {
        return true;
    }
    let lower = s.to_ascii_lowercase();
    for unit in &["px", "em", "rem", "vh", "vw", "cm", "mm", "in", "pt", "pc"] {
        if lower.ends_with(unit) {
            let num_part = &lower[..lower.len() - unit.len()];
            if num_part.parse::<f64>().is_ok() {
                return true;
            }
        }
    }
    false
}

fn is_color_token(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let s_trimmed = lower.trim();
    if let Some(hex) = s_trimmed.strip_prefix('#') {
        if hex.len() == 3 || hex.len() == 4 || hex.len() == 6 || hex.len() == 8 {
            return hex.chars().all(|c| c.is_ascii_hexdigit());
        }
        return false;
    }
    if s_trimmed.starts_with("rgb(")
        || s_trimmed.starts_with("rgba(")
        || s_trimmed.starts_with("hsl(")
        || s_trimmed.starts_with("hsla(")
    {
        return s_trimmed.ends_with(')');
    }
    matches!(
        s_trimmed,
        "transparent"
            | "currentcolor"
            | "black"
            | "silver"
            | "gray"
            | "white"
            | "maroon"
            | "red"
            | "purple"
            | "fuchsia"
            | "green"
            | "lime"
            | "olive"
            | "yellow"
            | "navy"
            | "blue"
            | "teal"
            | "aqua"
            | "orange"
            | "aliceblue"
            | "antiquewhite"
            | "aquamarine"
            | "azure"
            | "beige"
            | "bisque"
            | "blanchedalmond"
            | "blueviolet"
            | "brown"
            | "burlywood"
            | "cadetblue"
            | "chartreuse"
            | "chocolate"
            | "coral"
            | "cornflowerblue"
            | "cornsilk"
            | "crimson"
            | "cyan"
            | "darkblue"
            | "darkcyan"
            | "darkgoldenrod"
            | "darkgray"
            | "darkgreen"
            | "darkgrey"
            | "darkkhaki"
            | "darkmagenta"
            | "darkolivegreen"
            | "darkorange"
            | "darkorchid"
            | "darkred"
            | "darksalmon"
            | "darkseagreen"
            | "darkslateblue"
            | "darkslategray"
            | "darkslategrey"
            | "darkturquoise"
            | "darkviolet"
            | "deeppink"
            | "deepskyblue"
            | "dimgray"
            | "dimgrey"
            | "dodgerblue"
            | "firebrick"
            | "floralwhite"
            | "forestgreen"
            | "gainsboro"
            | "ghostwhite"
            | "gold"
            | "goldenrod"
            | "greenyellow"
            | "grey"
            | "honeydew"
            | "hotpink"
            | "indianred"
            | "indigo"
            | "ivory"
            | "khaki"
            | "lavender"
            | "lavenderblush"
            | "lawngreen"
            | "lemonchiffon"
            | "lightblue"
            | "lightcoral"
            | "lightcyan"
            | "lightgoldenrodyellow"
            | "lightgray"
            | "lightgreen"
            | "lightgrey"
            | "lightpink"
            | "lightsalmon"
            | "lightseagreen"
            | "lightskyblue"
            | "lightslategray"
            | "lightslategrey"
            | "lightsteelblue"
            | "lightyellow"
            | "limegreen"
            | "linen"
            | "magenta"
            | "mediumaquamarine"
            | "mediumblue"
            | "mediumorchid"
            | "mediumpurple"
            | "mediumseagreen"
            | "mediumslateblue"
            | "mediumspringgreen"
            | "mediumturquoise"
            | "mediumvioletred"
            | "midnightblue"
            | "mintcream"
            | "mistyrose"
            | "moccasin"
            | "navajowhite"
            | "oldlace"
            | "olivedrab"
            | "orangered"
            | "orchid"
            | "palegoldenrod"
            | "palegreen"
            | "paleturquoise"
            | "palevioletred"
            | "papayawhip"
            | "peachpuff"
            | "peru"
            | "pink"
            | "plum"
            | "powderblue"
            | "rosybrown"
            | "royalblue"
            | "saddlebrown"
            | "salmon"
            | "sandybrown"
            | "seagreen"
            | "seashell"
            | "sienna"
            | "skyblue"
            | "slateblue"
            | "slategray"
            | "slategrey"
            | "snow"
            | "springgreen"
            | "steelblue"
            | "tan"
            | "thistle"
            | "tomato"
            | "turquoise"
            | "violet"
            | "wheat"
            | "whitesmoke"
            | "yellowgreen"
            | "rebeccapurple"
    )
}

fn is_url_token(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.starts_with("url(") && lower.ends_with(')')
}

fn is_image_token(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    is_url_token(s)
        || lower.starts_with("linear-gradient(")
        || lower.starts_with("radial-gradient(")
        || lower.starts_with("conic-gradient(")
        || lower.starts_with("repeating-linear-gradient(")
        || lower.starts_with("repeating-radial-gradient(")
        || lower.starts_with("repeating-conic-gradient(")
}

fn is_transform_function_token(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    let s_trimmed = lower.trim();
    (s_trimmed.starts_with("translate(")
        || s_trimmed.starts_with("translatex(")
        || s_trimmed.starts_with("translatey(")
        || s_trimmed.starts_with("translatez(")
        || s_trimmed.starts_with("translate3d(")
        || s_trimmed.starts_with("scale(")
        || s_trimmed.starts_with("scalex(")
        || s_trimmed.starts_with("scaley(")
        || s_trimmed.starts_with("scalez(")
        || s_trimmed.starts_with("scale3d(")
        || s_trimmed.starts_with("rotate(")
        || s_trimmed.starts_with("rotatex(")
        || s_trimmed.starts_with("rotatey(")
        || s_trimmed.starts_with("rotatez(")
        || s_trimmed.starts_with("rotate3d(")
        || s_trimmed.starts_with("skew(")
        || s_trimmed.starts_with("skewx(")
        || s_trimmed.starts_with("skewy(")
        || s_trimmed.starts_with("matrix(")
        || s_trimmed.starts_with("matrix3d(")
        || s_trimmed.starts_with("perspective("))
        && s_trimmed.ends_with(')')
}

fn is_custom_ident_token(s: &str) -> bool {
    is_valid_css_identifier(s)
        && !matches!(
            s.to_ascii_lowercase().as_str(),
            "length"
                | "number"
                | "percentage"
                | "length-percentage"
                | "color"
                | "image"
                | "url"
                | "integer"
                | "angle"
                | "time"
                | "resolution"
                | "transform-function"
                | "transform-list"
                | "custom-ident"
        )
}

fn is_keyword_token(s: &str, expected: &str) -> bool {
    s.eq_ignore_ascii_case(expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_find_missing_longhands() {
        let mut missing = Vec::new();
        for sh in SHORTHAND_EXPANSIONS {
            for lh in sh.longhands {
                // If it is a shorthand itself, ignore it
                let is_shorthand = SHORTHAND_EXPANSIONS.iter().any(|s| s.name == *lh);
                if !is_shorthand && lookup(lh).is_none() && !missing.contains(lh) {
                    missing.push(*lh);
                }
            }
        }
        if !missing.is_empty() {
            panic!(
                "Missing longhand properties in PROPERTY_METADATA: {:?}",
                missing
            );
        }
    }

    #[test]
    fn test_is_inherited() {
        assert!(is_inherited("color"));
        assert!(is_inherited("Color"));
        assert!(is_inherited("FONT-SIZE"));
        assert!(!is_inherited("margin-top"));
        assert!(!is_inherited("not-a-real-prop"));
        assert!(is_inherited("text-indent"));
        assert!(is_inherited("color-interpolation"));
        assert!(!is_inherited("max-width"));
        assert!(is_inherited("text-wrap"));
        assert!(is_inherited("TEXT-WRAP"));
        assert!(is_inherited("font-kerning"));
        assert!(!is_inherited("will-change"));
    }

    #[test]
    fn test_initial_value() {
        assert_eq!(initial_value("display"), Some("inline"));
        assert_eq!(initial_value("width"), Some("auto"));
        assert_eq!(initial_value("border-top-color"), Some("currentcolor"));
        assert_eq!(initial_value("not-a-real-prop"), None);
        assert_eq!(initial_value("flex-shrink"), Some("1"));
        assert_eq!(initial_value("border-collapse"), Some("separate"));
        assert_eq!(initial_value("color-interpolation"), Some("sRGB"));
        assert_eq!(initial_value("background-repeat"), Some("repeat"));
        assert_eq!(initial_value("font-optical-sizing"), Some("auto"));
        assert_eq!(initial_value("will-change"), Some("auto"));
        assert_eq!(initial_value("text-wrap"), Some("wrap"));
    }

    #[test]
    fn test_is_animatable() {
        assert!(is_animatable("color"));
        assert!(is_animatable("width"));
        assert!(is_animatable("opacity"));
        assert!(is_animatable("margin-top"));
        assert!(is_animatable("background-color"));
        assert!(is_animatable("transform"));
        assert!(is_animatable("translate"));
        assert!(is_animatable("scale"));
        assert!(is_animatable("rotate"));
        assert!(is_animatable("perspective"));
        assert!(is_animatable("transform-origin"));

        assert!(!is_animatable("display"));
        assert!(!is_animatable("font-family"));
        assert!(!is_animatable("direction"));
        assert!(!is_animatable("animation-name"));
        assert!(!is_animatable("not-a-real-prop"));
    }

    #[test]
    fn test_lookup() {
        let meta = lookup("FONT-SIZE");
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.name, "font-size");
        assert!(meta.inherited);
        assert_eq!(meta.initial, "medium");

        let bbc = lookup("BORDER-BOTTOM-COLOR");
        assert!(bbc.is_some());
        let bbc = bbc.unwrap();
        assert_eq!(bbc.name, "border-bottom-color");
        assert!(!bbc.inherited);
    }

    #[test]
    fn test_additional_properties_t0688() {
        let row_gap = lookup("row-gap");
        assert!(row_gap.is_some());
        let row_gap = row_gap.unwrap();
        assert_eq!(row_gap.name, "row-gap");
        assert!(!row_gap.inherited);
        assert_eq!(row_gap.initial, "normal");

        let column_gap = lookup("column-gap");
        assert!(column_gap.is_some());
        let column_gap = column_gap.unwrap();
        assert_eq!(column_gap.name, "column-gap");
        assert!(!column_gap.inherited);
        assert_eq!(column_gap.initial, "normal");

        let justify_items = lookup("justify-items");
        assert!(justify_items.is_some());
        let justify_items = justify_items.unwrap();
        assert_eq!(justify_items.name, "justify-items");
        assert!(!justify_items.inherited);
        assert_eq!(justify_items.initial, "legacy");

        let align_content = lookup("align-content");
        assert!(align_content.is_some());
        let align_content = align_content.unwrap();
        assert_eq!(align_content.name, "align-content");
        assert!(!align_content.inherited);
        assert_eq!(align_content.initial, "normal");
    }

    #[test]
    fn test_additive_properties_t0789() {
        // Test anchor positioning
        let pos_anchor = lookup("position-anchor").expect("position-anchor must be registered");
        assert_eq!(pos_anchor.name, "position-anchor");
        assert!(!pos_anchor.inherited);
        assert_eq!(pos_anchor.initial, "implicit");

        // Test containment properties
        let contain_block = lookup("contain-intrinsic-block-size")
            .expect("contain-intrinsic-block-size must be registered");
        assert_eq!(contain_block.name, "contain-intrinsic-block-size");
        assert!(!contain_block.inherited);
        assert_eq!(contain_block.initial, "none");

        // Test logical sizing
        let block_size = lookup("block-size").expect("block-size must be registered");
        assert_eq!(block_size.name, "block-size");
        assert!(!block_size.inherited);
        assert_eq!(block_size.initial, "auto");

        let min_block_size = lookup("min-block-size").expect("min-block-size must be registered");
        assert_eq!(min_block_size.name, "min-block-size");
        assert!(!min_block_size.inherited);
        assert_eq!(min_block_size.initial, "0");

        let max_block_size = lookup("max-block-size").expect("max-block-size must be registered");
        assert_eq!(max_block_size.name, "max-block-size");
        assert!(!max_block_size.inherited);
        assert_eq!(max_block_size.initial, "none");

        // Test logical margin & padding
        let margin_start =
            lookup("margin-inline-start").expect("margin-inline-start must be registered");
        assert_eq!(margin_start.name, "margin-inline-start");
        assert!(!margin_start.inherited);
        assert_eq!(margin_start.initial, "0");

        let padding_end =
            lookup("padding-inline-end").expect("padding-inline-end must be registered");
        assert_eq!(padding_end.name, "padding-inline-end");
        assert!(!padding_end.inherited);
        assert_eq!(padding_end.initial, "0");

        // Test logical border width, style, color
        let border_block_start_w = lookup("border-block-start-width")
            .expect("border-block-start-width must be registered");
        assert_eq!(border_block_start_w.name, "border-block-start-width");
        assert!(!border_block_start_w.inherited);
        assert_eq!(border_block_start_w.initial, "medium");

        let border_inline_end_s =
            lookup("border-inline-end-style").expect("border-inline-end-style must be registered");
        assert_eq!(border_inline_end_s.name, "border-inline-end-style");
        assert!(!border_inline_end_s.inherited);
        assert_eq!(border_inline_end_s.initial, "none");

        let border_block_end_c =
            lookup("border-block-end-color").expect("border-block-end-color must be registered");
        assert_eq!(border_block_end_c.name, "border-block-end-color");
        assert!(!border_block_end_c.inherited);
        assert_eq!(border_block_end_c.initial, "currentcolor");

        // Test logical border radius
        let border_start_start_r = lookup("border-start-start-radius")
            .expect("border-start-start-radius must be registered");
        assert_eq!(border_start_start_r.name, "border-start-start-radius");
        assert!(!border_start_start_r.inherited);
        assert_eq!(border_start_start_r.initial, "0");
    }

    #[test]
    fn test_additive_properties_t0469() {
        let filter = lookup("filter");
        assert!(filter.is_some());
        let filter = filter.unwrap();
        assert_eq!(filter.name, "filter");
        assert!(!filter.inherited);
        assert_eq!(filter.initial, "none");

        let backdrop_filter = lookup("backdrop-filter");
        assert!(backdrop_filter.is_some());
        let backdrop_filter = backdrop_filter.unwrap();
        assert_eq!(backdrop_filter.name, "backdrop-filter");
        assert!(!backdrop_filter.inherited);
        assert_eq!(backdrop_filter.initial, "none");

        let mix_blend_mode = lookup("mix-blend-mode");
        assert!(mix_blend_mode.is_some());
        let mix_blend_mode = mix_blend_mode.unwrap();
        assert_eq!(mix_blend_mode.name, "mix-blend-mode");
        assert!(!mix_blend_mode.inherited);
        assert_eq!(mix_blend_mode.initial, "normal");

        let isolation = lookup("isolation");
        assert!(isolation.is_some());
        let isolation = isolation.unwrap();
        assert_eq!(isolation.name, "isolation");
        assert!(!isolation.inherited);
        assert_eq!(isolation.initial, "auto");

        let resize = lookup("resize");
        assert!(resize.is_some());
        let resize = resize.unwrap();
        assert_eq!(resize.name, "resize");
        assert!(!resize.inherited);
        assert_eq!(resize.initial, "none");

        let backface_visibility = lookup("backface-visibility");
        assert!(backface_visibility.is_some());
        let backface_visibility = backface_visibility.unwrap();
        assert_eq!(backface_visibility.name, "backface-visibility");
        assert!(!backface_visibility.inherited);
        assert_eq!(backface_visibility.initial, "visible");
    }

    #[test]
    fn test_additive_properties_t0471() {
        let object_position = lookup("object-position");
        assert!(object_position.is_some());
        let object_position = object_position.unwrap();
        assert_eq!(object_position.name, "object-position");
        assert!(!object_position.inherited);
        assert_eq!(object_position.initial, "50% 50%");
    }

    #[test]
    fn test_additive_properties_t0473() {
        let scroll_behavior = lookup("scroll-behavior");
        assert!(scroll_behavior.is_some());
        let scroll_behavior = scroll_behavior.unwrap();
        assert_eq!(scroll_behavior.name, "scroll-behavior");
        assert!(!scroll_behavior.inherited);
        assert_eq!(scroll_behavior.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0693() {
        let scrollbar_width = lookup("scrollbar-width");
        assert!(scrollbar_width.is_some());
        let scrollbar_width = scrollbar_width.unwrap();
        assert_eq!(scrollbar_width.name, "scrollbar-width");
        assert!(scrollbar_width.inherited);
        assert_eq!(scrollbar_width.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0698() {
        let scrollbar_color = lookup("scrollbar-color");
        assert!(scrollbar_color.is_some());
        let scrollbar_color = scrollbar_color.unwrap();
        assert_eq!(scrollbar_color.name, "scrollbar-color");
        assert!(scrollbar_color.inherited);
        assert_eq!(scrollbar_color.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0702() {
        let scrollbar_gutter = lookup("scrollbar-gutter");
        assert!(scrollbar_gutter.is_some());
        let scrollbar_gutter = scrollbar_gutter.unwrap();
        assert_eq!(scrollbar_gutter.name, "scrollbar-gutter");
        assert!(!scrollbar_gutter.inherited);
        assert_eq!(scrollbar_gutter.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0706() {
        let text_wrap = lookup("text-wrap");
        assert!(text_wrap.is_some());
        let text_wrap = text_wrap.unwrap();
        assert_eq!(text_wrap.name, "text-wrap");
        assert!(text_wrap.inherited);
        assert_eq!(text_wrap.initial, "wrap");
    }

    #[test]
    fn test_additive_properties_t0710() {
        let forced_color_adjust = lookup("forced-color-adjust");
        assert!(forced_color_adjust.is_some());
        let forced_color_adjust = forced_color_adjust.unwrap();
        assert_eq!(forced_color_adjust.name, "forced-color-adjust");
        assert!(forced_color_adjust.inherited);
        assert_eq!(forced_color_adjust.initial, "auto");

        let caret_shape = lookup("caret-shape");
        assert!(caret_shape.is_some());
        let caret_shape = caret_shape.unwrap();
        assert_eq!(caret_shape.name, "caret-shape");
        assert!(caret_shape.inherited);
        assert_eq!(caret_shape.initial, "auto");

        let field_sizing = lookup("field-sizing");
        assert!(field_sizing.is_some());
        let field_sizing = field_sizing.unwrap();
        assert_eq!(field_sizing.name, "field-sizing");
        assert!(!field_sizing.inherited);
        assert_eq!(field_sizing.initial, "fixed");
    }

    #[test]
    fn test_additive_properties_t0714() {
        let text_autospace = lookup("text-autospace");
        assert!(text_autospace.is_some());
        let text_autospace = text_autospace.unwrap();
        assert_eq!(text_autospace.name, "text-autospace");
        assert!(text_autospace.inherited);
        assert_eq!(text_autospace.initial, "normal");

        let text_spacing_trim = lookup("text-spacing-trim");
        assert!(text_spacing_trim.is_some());
        let text_spacing_trim = text_spacing_trim.unwrap();
        assert_eq!(text_spacing_trim.name, "text-spacing-trim");
        assert!(text_spacing_trim.inherited);
        assert_eq!(text_spacing_trim.initial, "normal");

        let hyphenate_character = lookup("hyphenate-character");
        assert!(hyphenate_character.is_some());
        let hyphenate_character = hyphenate_character.unwrap();
        assert_eq!(hyphenate_character.name, "hyphenate-character");
        assert!(hyphenate_character.inherited);
        assert_eq!(hyphenate_character.initial, "auto");

        let ruby_position = lookup("ruby-position");
        assert!(ruby_position.is_some());
        let ruby_position = ruby_position.unwrap();
        assert_eq!(ruby_position.name, "ruby-position");
        assert!(ruby_position.inherited);
        assert_eq!(ruby_position.initial, "alternate");
    }

    #[test]
    fn test_additive_properties_t0717() {
        let line_break = lookup("line-break");
        assert!(line_break.is_some());
        let line_break = line_break.unwrap();
        assert_eq!(line_break.name, "line-break");
        assert!(line_break.inherited);
        assert_eq!(line_break.initial, "auto");

        let white_space_collapse = lookup("white-space-collapse");
        assert!(white_space_collapse.is_some());
        let white_space_collapse = white_space_collapse.unwrap();
        assert_eq!(white_space_collapse.name, "white-space-collapse");
        assert!(white_space_collapse.inherited);
        assert_eq!(white_space_collapse.initial, "collapse");

        let text_wrap_style = lookup("text-wrap-style");
        assert!(text_wrap_style.is_some());
        let text_wrap_style = text_wrap_style.unwrap();
        assert_eq!(text_wrap_style.name, "text-wrap-style");
        assert!(text_wrap_style.inherited);
        assert_eq!(text_wrap_style.initial, "auto");

        let text_wrap_mode = lookup("text-wrap-mode");
        assert!(text_wrap_mode.is_some());
        let text_wrap_mode = text_wrap_mode.unwrap();
        assert_eq!(text_wrap_mode.name, "text-wrap-mode");
        assert!(text_wrap_mode.inherited);
        assert_eq!(text_wrap_mode.initial, "wrap");

        let text_underline_position = lookup("text-underline-position");
        assert!(text_underline_position.is_some());
        let text_underline_position = text_underline_position.unwrap();
        assert_eq!(text_underline_position.name, "text-underline-position");
        assert!(text_underline_position.inherited);
        assert_eq!(text_underline_position.initial, "auto");

        let text_emphasis_color = lookup("text-emphasis-color");
        assert!(text_emphasis_color.is_some());
        let text_emphasis_color = text_emphasis_color.unwrap();
        assert_eq!(text_emphasis_color.name, "text-emphasis-color");
        assert!(text_emphasis_color.inherited);
        assert_eq!(text_emphasis_color.initial, "currentcolor");
    }

    #[test]
    fn test_additive_properties_t0721() {
        // scroll-margin
        let scroll_margin = lookup("scroll-margin");
        assert!(scroll_margin.is_some());
        let scroll_margin = scroll_margin.unwrap();
        assert_eq!(scroll_margin.name, "scroll-margin");
        assert!(!scroll_margin.inherited);
        assert_eq!(scroll_margin.initial, "0");

        // scroll-margin-top
        let scroll_margin_top = lookup("scroll-margin-top");
        assert!(scroll_margin_top.is_some());
        let scroll_margin_top = scroll_margin_top.unwrap();
        assert_eq!(scroll_margin_top.name, "scroll-margin-top");
        assert!(!scroll_margin_top.inherited);
        assert_eq!(scroll_margin_top.initial, "0");

        // scroll-padding
        let scroll_padding = lookup("scroll-padding");
        assert!(scroll_padding.is_some());
        let scroll_padding = scroll_padding.unwrap();
        assert_eq!(scroll_padding.name, "scroll-padding");
        assert!(!scroll_padding.inherited);
        assert_eq!(scroll_padding.initial, "auto");

        // scroll-padding-inline-end
        let scroll_padding_inline_end = lookup("scroll-padding-inline-end");
        assert!(scroll_padding_inline_end.is_some());
        let scroll_padding_inline_end = scroll_padding_inline_end.unwrap();
        assert_eq!(scroll_padding_inline_end.name, "scroll-padding-inline-end");
        assert!(!scroll_padding_inline_end.inherited);
        assert_eq!(scroll_padding_inline_end.initial, "auto");

        // overflow-clip-margin
        let overflow_clip_margin = lookup("overflow-clip-margin");
        assert!(overflow_clip_margin.is_some());
        let overflow_clip_margin = overflow_clip_margin.unwrap();
        assert_eq!(overflow_clip_margin.name, "overflow-clip-margin");
        assert!(!overflow_clip_margin.inherited);
        assert_eq!(overflow_clip_margin.initial, "0px");

        // inset-block
        let inset_block = lookup("inset-block");
        assert!(inset_block.is_some());
        let inset_block = inset_block.unwrap();
        assert_eq!(inset_block.name, "inset-block");
        assert!(!inset_block.inherited);
        assert_eq!(inset_block.initial, "auto");

        // inset-inline-start
        let inset_inline_start = lookup("inset-inline-start");
        assert!(inset_inline_start.is_some());
        let inset_inline_start = inset_inline_start.unwrap();
        assert_eq!(inset_inline_start.name, "inset-inline-start");
        assert!(!inset_inline_start.inherited);
        assert_eq!(inset_inline_start.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0725() {
        // text-justify
        let text_justify = lookup("text-justify");
        assert!(text_justify.is_some());
        let text_justify = text_justify.unwrap();
        assert_eq!(text_justify.name, "text-justify");
        assert!(text_justify.inherited);
        assert_eq!(text_justify.initial, "auto");

        // text-combine-upright
        let text_combine_upright = lookup("text-combine-upright");
        assert!(text_combine_upright.is_some());
        let text_combine_upright = text_combine_upright.unwrap();
        assert_eq!(text_combine_upright.name, "text-combine-upright");
        assert!(text_combine_upright.inherited);
        assert_eq!(text_combine_upright.initial, "none");

        // text-decoration-skip-ink
        let text_decoration_skip_ink = lookup("text-decoration-skip-ink");
        assert!(text_decoration_skip_ink.is_some());
        let text_decoration_skip_ink = text_decoration_skip_ink.unwrap();
        assert_eq!(text_decoration_skip_ink.name, "text-decoration-skip-ink");
        assert!(text_decoration_skip_ink.inherited);
        assert_eq!(text_decoration_skip_ink.initial, "auto");

        // hanging-punctuation
        let hanging_punctuation = lookup("hanging-punctuation");
        assert!(hanging_punctuation.is_some());
        let hanging_punctuation = hanging_punctuation.unwrap();
        assert_eq!(hanging_punctuation.name, "hanging-punctuation");
        assert!(hanging_punctuation.inherited);
        assert_eq!(hanging_punctuation.initial, "none");

        // color-scheme
        let color_scheme = lookup("color-scheme");
        assert!(color_scheme.is_some());
        let color_scheme = color_scheme.unwrap();
        assert_eq!(color_scheme.name, "color-scheme");
        assert!(!color_scheme.inherited);
        assert_eq!(color_scheme.initial, "normal");

        // text-rendering
        let text_rendering = lookup("text-rendering");
        assert!(text_rendering.is_some());
        let text_rendering = text_rendering.unwrap();
        assert_eq!(text_rendering.name, "text-rendering");
        assert!(text_rendering.inherited);
        assert_eq!(text_rendering.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0645() {
        let print_color_adjust = lookup("print-color-adjust");
        assert!(print_color_adjust.is_some());
        let print_color_adjust = print_color_adjust.unwrap();
        assert_eq!(print_color_adjust.name, "print-color-adjust");
        assert!(print_color_adjust.inherited);
        assert_eq!(print_color_adjust.initial, "economy");
    }

    #[test]
    fn test_additive_properties_t0475() {
        let user_select = lookup("user-select");
        assert!(user_select.is_some());
        let user_select = user_select.unwrap();
        assert_eq!(user_select.name, "user-select");
        assert!(!user_select.inherited);
        assert_eq!(user_select.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0477() {
        let accent_color = lookup("accent-color");
        assert!(accent_color.is_some());
        let accent_color = accent_color.unwrap();
        assert_eq!(accent_color.name, "accent-color");
        assert!(accent_color.inherited);
        assert_eq!(accent_color.initial, "auto");

        let caret_color = lookup("caret-color");
        assert!(caret_color.is_some());
        let caret_color = caret_color.unwrap();
        assert_eq!(caret_color.name, "caret-color");
        assert!(caret_color.inherited);
        assert_eq!(caret_color.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0479() {
        let timing = lookup("transition-timing-function");
        assert!(timing.is_some());
        let timing = timing.unwrap();
        assert_eq!(timing.name, "transition-timing-function");
        assert!(!timing.inherited);
        assert_eq!(timing.initial, "ease");

        let delay = lookup("transition-delay");
        assert!(delay.is_some());
        let delay = delay.unwrap();
        assert_eq!(delay.name, "transition-delay");
        assert!(!delay.inherited);
        assert_eq!(delay.initial, "0s");
    }

    #[test]
    fn test_additive_properties_t0485() {
        let overscroll_behavior = lookup("overscroll-behavior");
        assert!(overscroll_behavior.is_some());
        let overscroll_behavior = overscroll_behavior.unwrap();
        assert_eq!(overscroll_behavior.name, "overscroll-behavior");
        assert!(!overscroll_behavior.inherited);
        assert_eq!(overscroll_behavior.initial, "auto");

        let overscroll_behavior_x = lookup("overscroll-behavior-x");
        assert!(overscroll_behavior_x.is_some());
        let overscroll_behavior_x = overscroll_behavior_x.unwrap();
        assert_eq!(overscroll_behavior_x.name, "overscroll-behavior-x");
        assert!(!overscroll_behavior_x.inherited);
        assert_eq!(overscroll_behavior_x.initial, "auto");

        let overscroll_behavior_y = lookup("overscroll-behavior-y");
        assert!(overscroll_behavior_y.is_some());
        let overscroll_behavior_y = overscroll_behavior_y.unwrap();
        assert_eq!(overscroll_behavior_y.name, "overscroll-behavior-y");
        assert!(!overscroll_behavior_y.inherited);
        assert_eq!(overscroll_behavior_y.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0495() {
        let scroll_snap_type = lookup("scroll-snap-type");
        assert!(scroll_snap_type.is_some());
        let scroll_snap_type = scroll_snap_type.unwrap();
        assert_eq!(scroll_snap_type.name, "scroll-snap-type");
        assert!(!scroll_snap_type.inherited);
        assert_eq!(scroll_snap_type.initial, "none");

        let scroll_snap_align = lookup("scroll-snap-align");
        assert!(scroll_snap_align.is_some());
        let scroll_snap_align = scroll_snap_align.unwrap();
        assert_eq!(scroll_snap_align.name, "scroll-snap-align");
        assert!(!scroll_snap_align.inherited);
        assert_eq!(scroll_snap_align.initial, "none");

        let scroll_snap_stop = lookup("scroll-snap-stop");
        assert!(scroll_snap_stop.is_some());
        let scroll_snap_stop = scroll_snap_stop.unwrap();
        assert_eq!(scroll_snap_stop.name, "scroll-snap-stop");
        assert!(!scroll_snap_stop.inherited);
        assert_eq!(scroll_snap_stop.initial, "normal");

        let scroll_padding = lookup("scroll-padding");
        assert!(scroll_padding.is_some());
        let scroll_padding = scroll_padding.unwrap();
        assert_eq!(scroll_padding.name, "scroll-padding");
        assert!(!scroll_padding.inherited);
        assert_eq!(scroll_padding.initial, "auto");

        let scroll_margin = lookup("scroll-margin");
        assert!(scroll_margin.is_some());
        let scroll_margin = scroll_margin.unwrap();
        assert_eq!(scroll_margin.name, "scroll-margin");
        assert!(!scroll_margin.inherited);
        assert_eq!(scroll_margin.initial, "0");
    }

    #[test]
    fn test_additive_properties_t0498() {
        let clip_path = lookup("clip-path");
        assert!(clip_path.is_some());
        let clip_path = clip_path.unwrap();
        assert_eq!(clip_path.name, "clip-path");
        assert!(!clip_path.inherited);
        assert_eq!(clip_path.initial, "none");

        let clip = lookup("clip");
        assert!(clip.is_some());
        let clip = clip.unwrap();
        assert_eq!(clip.name, "clip");
        assert!(!clip.inherited);
        assert_eq!(clip.initial, "auto");

        let clip_rule = lookup("clip-rule");
        assert!(clip_rule.is_some());
        let clip_rule = clip_rule.unwrap();
        assert_eq!(clip_rule.name, "clip-rule");
        assert!(clip_rule.inherited);
        assert_eq!(clip_rule.initial, "nonzero");
    }

    #[test]
    fn test_additive_properties_t0524() {
        let image_rendering = lookup("image-rendering");
        assert!(image_rendering.is_some());
        let image_rendering = image_rendering.unwrap();
        assert_eq!(image_rendering.name, "image-rendering");
        assert!(image_rendering.inherited);
        assert_eq!(image_rendering.initial, "auto");

        let contain = lookup("contain");
        assert!(contain.is_some());
        let contain = contain.unwrap();
        assert_eq!(contain.name, "contain");
        assert!(!contain.inherited);
        assert_eq!(contain.initial, "none");

        let text_decor_thick = lookup("text-decoration-thickness");
        assert!(text_decor_thick.is_some());
        let text_decor_thick = text_decor_thick.unwrap();
        assert_eq!(text_decor_thick.name, "text-decoration-thickness");
        assert!(!text_decor_thick.inherited);
        assert_eq!(text_decor_thick.initial, "auto");

        let text_under_offset = lookup("text-underline-offset");
        assert!(text_under_offset.is_some());
        let text_under_offset = text_under_offset.unwrap();
        assert_eq!(text_under_offset.name, "text-underline-offset");
        assert!(text_under_offset.inherited);
        assert_eq!(text_under_offset.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0530() {
        let counter_reset = lookup("counter-reset");
        assert!(counter_reset.is_some());
        let counter_reset = counter_reset.unwrap();
        assert_eq!(counter_reset.name, "counter-reset");
        assert!(!counter_reset.inherited);
        assert_eq!(counter_reset.initial, "none");

        let counter_increment = lookup("counter-increment");
        assert!(counter_increment.is_some());
        let counter_increment = counter_increment.unwrap();
        assert_eq!(counter_increment.name, "counter-increment");
        assert!(!counter_increment.inherited);
        assert_eq!(counter_increment.initial, "none");
    }

    #[test]
    fn test_additive_properties_t0536() {
        let orphans = lookup("orphans");
        assert!(orphans.is_some());
        let orphans = orphans.unwrap();
        assert_eq!(orphans.name, "orphans");
        assert!(orphans.inherited);
        assert_eq!(orphans.initial, "2");

        let widows = lookup("widows");
        assert!(widows.is_some());
        let widows = widows.unwrap();
        assert_eq!(widows.name, "widows");
        assert!(widows.inherited);
        assert_eq!(widows.initial, "2");

        let break_before = lookup("break-before");
        assert!(break_before.is_some());
        let break_before = break_before.unwrap();
        assert_eq!(break_before.name, "break-before");
        assert!(!break_before.inherited);
        assert_eq!(break_before.initial, "auto");

        let break_after = lookup("break-after");
        assert!(break_after.is_some());
        let break_after = break_after.unwrap();
        assert_eq!(break_after.name, "break-after");
        assert!(!break_after.inherited);
        assert_eq!(break_after.initial, "auto");

        let break_inside = lookup("BREAK-INSIDE");
        assert!(break_inside.is_some());
        let break_inside = break_inside.unwrap();
        assert_eq!(break_inside.name, "break-inside");
        assert!(!break_inside.inherited);
        assert_eq!(break_inside.initial, "auto");

        let box_decoration_break = lookup("BOX-DECORATION-BREAK");
        assert!(box_decoration_break.is_some());
        let box_decoration_break = box_decoration_break.unwrap();
        assert_eq!(box_decoration_break.name, "box-decoration-break");
        assert!(!box_decoration_break.inherited);
        assert_eq!(box_decoration_break.initial, "slice");

        let mask_type = lookup("MASK-TYPE");
        assert!(mask_type.is_some());
        let mask_type = mask_type.unwrap();
        assert_eq!(mask_type.name, "mask-type");
        assert!(!mask_type.inherited);
        assert_eq!(mask_type.initial, "luminance");
    }

    #[test]
    fn test_additive_properties_t0745() {
        let shape_outside = lookup("shape-outside");
        assert!(shape_outside.is_some());
        let shape_outside = shape_outside.unwrap();
        assert_eq!(shape_outside.name, "shape-outside");
        assert!(!shape_outside.inherited);
        assert_eq!(shape_outside.initial, "none");

        let shape_margin = lookup("shape-margin");
        assert!(shape_margin.is_some());
        let shape_margin = shape_margin.unwrap();
        assert_eq!(shape_margin.name, "shape-margin");
        assert!(!shape_margin.inherited);
        assert_eq!(shape_margin.initial, "0");

        let shape_image_threshold = lookup("shape-image-threshold");
        assert!(shape_image_threshold.is_some());
        let shape_image_threshold = shape_image_threshold.unwrap();
        assert_eq!(shape_image_threshold.name, "shape-image-threshold");
        assert!(!shape_image_threshold.inherited);
        assert_eq!(shape_image_threshold.initial, "0");
    }

    #[test]
    fn test_additive_properties_t0753() {
        let anchor_name = lookup("anchor-name");
        assert!(anchor_name.is_some());
        let anchor_name = anchor_name.unwrap();
        assert_eq!(anchor_name.name, "anchor-name");
        assert!(!anchor_name.inherited);
        assert_eq!(anchor_name.initial, "none");

        let view_transition_name = lookup("view-transition-name");
        assert!(view_transition_name.is_some());
        let view_transition_name = view_transition_name.unwrap();
        assert_eq!(view_transition_name.name, "view-transition-name");
        assert!(!view_transition_name.inherited);
        assert_eq!(view_transition_name.initial, "none");

        let contain_intrinsic_width = lookup("contain-intrinsic-width");
        assert!(contain_intrinsic_width.is_some());
        let contain_intrinsic_width = contain_intrinsic_width.unwrap();
        assert_eq!(contain_intrinsic_width.name, "contain-intrinsic-width");
        assert!(!contain_intrinsic_width.inherited);
        assert_eq!(contain_intrinsic_width.initial, "none");

        let contain_intrinsic_height = lookup("contain-intrinsic-height");
        assert!(contain_intrinsic_height.is_some());
        let contain_intrinsic_height = contain_intrinsic_height.unwrap();
        assert_eq!(contain_intrinsic_height.name, "contain-intrinsic-height");
        assert!(!contain_intrinsic_height.inherited);
        assert_eq!(contain_intrinsic_height.initial, "none");

        let content_visibility = lookup("content-visibility");
        assert!(content_visibility.is_some());
        let content_visibility = content_visibility.unwrap();
        assert_eq!(content_visibility.name, "content-visibility");
        assert!(!content_visibility.inherited);
        assert_eq!(content_visibility.initial, "visible");

        let animation_timeline = lookup("animation-timeline");
        assert!(animation_timeline.is_some());
        let animation_timeline = animation_timeline.unwrap();
        assert_eq!(animation_timeline.name, "animation-timeline");
        assert!(!animation_timeline.inherited);
        assert_eq!(animation_timeline.initial, "auto");

        let scroll_timeline_name = lookup("scroll-timeline-name");
        assert!(scroll_timeline_name.is_some());
        let scroll_timeline_name = scroll_timeline_name.unwrap();
        assert_eq!(scroll_timeline_name.name, "scroll-timeline-name");
        assert!(!scroll_timeline_name.inherited);
        assert_eq!(scroll_timeline_name.initial, "none");

        let scroll_timeline_axis = lookup("scroll-timeline-axis");
        assert!(scroll_timeline_axis.is_some());
        let scroll_timeline_axis = scroll_timeline_axis.unwrap();
        assert_eq!(scroll_timeline_axis.name, "scroll-timeline-axis");
        assert!(!scroll_timeline_axis.inherited);
        assert_eq!(scroll_timeline_axis.initial, "block");
    }

    #[test]
    fn test_property_text_emphasis_style_t0757() {
        let prop = lookup("text-emphasis-style");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "text-emphasis-style");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "none");
    }

    #[test]
    fn test_property_text_emphasis_position_t0757() {
        let prop = lookup("text-emphasis-position");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "text-emphasis-position");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "over right");
    }

    #[test]
    fn test_property_math_style_t0757() {
        let prop = lookup("math-style");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "math-style");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "normal");
    }

    #[test]
    fn test_property_math_depth_t0757() {
        let prop = lookup("math-depth");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "math-depth");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "0");
    }

    #[test]
    fn test_property_ruby_align_t0757() {
        let prop = lookup("ruby-align");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "ruby-align");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "space-around");
    }

    #[test]
    fn test_property_hyphenate_limit_chars_t0757() {
        let prop = lookup("hyphenate-limit-chars");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "hyphenate-limit-chars");
        assert!(prop.inherited);
        assert_eq!(prop.initial, "auto");
    }

    #[test]
    fn test_property_initial_letter_t0757() {
        let prop = lookup("initial-letter");
        assert!(prop.is_some());
        let prop = prop.unwrap();
        assert_eq!(prop.name, "initial-letter");
        assert!(!prop.inherited);
        assert_eq!(prop.initial, "normal");
    }

    #[test]
    fn test_properties_t0760() {
        let text_box_trim = lookup("text-box-trim");
        assert!(text_box_trim.is_some());
        let text_box_trim = text_box_trim.unwrap();
        assert_eq!(text_box_trim.name, "text-box-trim");
        assert!(!text_box_trim.inherited);
        assert_eq!(text_box_trim.initial, "none");

        let text_box_edge = lookup("text-box-edge");
        assert!(text_box_edge.is_some());
        let text_box_edge = text_box_edge.unwrap();
        assert_eq!(text_box_edge.name, "text-box-edge");
        assert!(!text_box_edge.inherited);
        assert_eq!(text_box_edge.initial, "auto");

        let webkit_line_clamp = lookup("-webkit-line-clamp");
        assert!(webkit_line_clamp.is_some());
        let webkit_line_clamp = webkit_line_clamp.unwrap();
        assert_eq!(webkit_line_clamp.name, "-webkit-line-clamp");
        assert!(!webkit_line_clamp.inherited);
        assert_eq!(webkit_line_clamp.initial, "none");

        let block_ellipsis = lookup("block-ellipsis");
        assert!(block_ellipsis.is_some());
        let block_ellipsis = block_ellipsis.unwrap();
        assert_eq!(block_ellipsis.name, "block-ellipsis");
        assert!(block_ellipsis.inherited);
        assert_eq!(block_ellipsis.initial, "none");

        let alignment_baseline = lookup("alignment-baseline");
        assert!(alignment_baseline.is_some());
        let alignment_baseline = alignment_baseline.unwrap();
        assert_eq!(alignment_baseline.name, "alignment-baseline");
        assert!(!alignment_baseline.inherited);
        assert_eq!(alignment_baseline.initial, "baseline");

        let baseline_shift = lookup("baseline-shift");
        assert!(baseline_shift.is_some());
        let baseline_shift = baseline_shift.unwrap();
        assert_eq!(baseline_shift.name, "baseline-shift");
        assert!(!baseline_shift.inherited);
        assert_eq!(baseline_shift.initial, "0");

        let baseline_source = lookup("baseline-source");
        assert!(baseline_source.is_some());
        let baseline_source = baseline_source.unwrap();
        assert_eq!(baseline_source.name, "baseline-source");
        assert!(!baseline_source.inherited);
        assert_eq!(baseline_source.initial, "auto");

        let dominant_baseline = lookup("dominant-baseline");
        assert!(dominant_baseline.is_some());
        let dominant_baseline = dominant_baseline.unwrap();
        assert_eq!(dominant_baseline.name, "dominant-baseline");
        assert!(!dominant_baseline.inherited);
        assert_eq!(dominant_baseline.initial, "auto");
    }

    #[test]
    fn test_properties_t0765() {
        let font_size_adjust = lookup("font-size-adjust");
        assert!(font_size_adjust.is_some());
        let font_size_adjust = font_size_adjust.unwrap();
        assert_eq!(font_size_adjust.name, "font-size-adjust");
        assert!(font_size_adjust.inherited);
        assert_eq!(font_size_adjust.initial, "none");

        let ruby_overhang = lookup("ruby-overhang");
        assert!(ruby_overhang.is_some());
        let ruby_overhang = ruby_overhang.unwrap();
        assert_eq!(ruby_overhang.name, "ruby-overhang");
        assert!(ruby_overhang.inherited);
        assert_eq!(ruby_overhang.initial, "auto");

        let ruby_merge = lookup("ruby-merge");
        assert!(ruby_merge.is_some());
        let ruby_merge = ruby_merge.unwrap();
        assert_eq!(ruby_merge.name, "ruby-merge");
        assert!(ruby_merge.inherited);
        assert_eq!(ruby_merge.initial, "separate");

        let line_clamp = lookup("line-clamp");
        assert!(line_clamp.is_some());
        let line_clamp = line_clamp.unwrap();
        assert_eq!(line_clamp.name, "line-clamp");
        assert!(!line_clamp.inherited);
        assert_eq!(line_clamp.initial, "none");
    }

    #[test]
    fn test_properties_t0773() {
        // scroll-marker-group
        let scroll_marker_group = lookup("scroll-marker-group");
        assert!(scroll_marker_group.is_some());
        let scroll_marker_group = scroll_marker_group.unwrap();
        assert_eq!(scroll_marker_group.name, "scroll-marker-group");
        assert!(!scroll_marker_group.inherited);
        assert_eq!(scroll_marker_group.initial, "none");

        // reading-flow
        let reading_flow = lookup("reading-flow");
        assert!(reading_flow.is_some());
        let reading_flow = reading_flow.unwrap();
        assert_eq!(reading_flow.name, "reading-flow");
        assert!(!reading_flow.inherited);
        assert_eq!(reading_flow.initial, "normal");

        // reading-order
        let reading_order = lookup("reading-order");
        assert!(reading_order.is_some());
        let reading_order = reading_order.unwrap();
        assert_eq!(reading_order.name, "reading-order");
        assert!(reading_order.inherited);
        assert_eq!(reading_order.initial, "0");

        // position-area
        let position_area = lookup("position-area");
        assert!(position_area.is_some());
        let position_area = position_area.unwrap();
        assert_eq!(position_area.name, "position-area");
        assert!(!position_area.inherited);
        assert_eq!(position_area.initial, "none");

        // position-try-fallbacks
        let position_try_fallbacks = lookup("position-try-fallbacks");
        assert!(position_try_fallbacks.is_some());
        let position_try_fallbacks = position_try_fallbacks.unwrap();
        assert_eq!(position_try_fallbacks.name, "position-try-fallbacks");
        assert!(!position_try_fallbacks.inherited);
        assert_eq!(position_try_fallbacks.initial, "none");

        // position-try-order
        let position_try_order = lookup("position-try-order");
        assert!(position_try_order.is_some());
        let position_try_order = position_try_order.unwrap();
        assert_eq!(position_try_order.name, "position-try-order");
        assert!(!position_try_order.inherited);
        assert_eq!(position_try_order.initial, "normal");

        // position-visibility
        let position_visibility = lookup("position-visibility");
        assert!(position_visibility.is_some());
        let position_visibility = position_visibility.unwrap();
        assert_eq!(position_visibility.name, "position-visibility");
        assert!(!position_visibility.inherited);
        assert_eq!(position_visibility.initial, "anchors-visible");

        // timeline-scope
        let timeline_scope = lookup("timeline-scope");
        assert!(timeline_scope.is_some());
        let timeline_scope = timeline_scope.unwrap();
        assert_eq!(timeline_scope.name, "timeline-scope");
        assert!(!timeline_scope.inherited);
        assert_eq!(timeline_scope.initial, "none");

        // view-transition-class
        let view_transition_class = lookup("view-transition-class");
        assert!(view_transition_class.is_some());
        let view_transition_class = view_transition_class.unwrap();
        assert_eq!(view_transition_class.name, "view-transition-class");
        assert!(!view_transition_class.inherited);
        assert_eq!(view_transition_class.initial, "none");

        // overlay
        let overlay = lookup("overlay");
        assert!(overlay.is_some());
        let overlay = overlay.unwrap();
        assert_eq!(overlay.name, "overlay");
        assert!(!overlay.inherited);
        assert_eq!(overlay.initial, "none");

        // position-try (shorthand)
        let position_try_shorthand = shorthand_longhands("position-try");
        assert!(position_try_shorthand.is_some());
        assert_eq!(
            position_try_shorthand.unwrap(),
            &["position-try-order", "position-try-fallbacks"][..]
        );
    }

    #[test]
    fn test_properties_t0779() {
        // Inherited properties:
        // writing-mode
        let writing_mode = lookup("writing-mode").expect("writing-mode must be registered");
        assert!(writing_mode.inherited);
        assert_eq!(writing_mode.initial, "horizontal-tb");

        // text-orientation
        let text_orientation =
            lookup("text-orientation").expect("text-orientation must be registered");
        assert!(text_orientation.inherited);
        assert_eq!(text_orientation.initial, "mixed");

        // math-shift
        let math_shift = lookup("math-shift").expect("math-shift must be registered");
        assert!(math_shift.inherited);
        assert_eq!(math_shift.initial, "normal");

        // text-shadow
        let text_shadow = lookup("text-shadow").expect("text-shadow must be registered");
        assert!(text_shadow.inherited);
        assert_eq!(text_shadow.initial, "none");

        // Non-inherited properties:
        // anchor-scope
        let anchor_scope = lookup("anchor-scope").expect("anchor-scope must be registered");
        assert!(!anchor_scope.inherited);
        assert_eq!(anchor_scope.initial, "none");

        // view-timeline-name
        let view_timeline_name =
            lookup("view-timeline-name").expect("view-timeline-name must be registered");
        assert!(!view_timeline_name.inherited);
        assert_eq!(view_timeline_name.initial, "none");

        // view-timeline-axis
        let view_timeline_axis =
            lookup("view-timeline-axis").expect("view-timeline-axis must be registered");
        assert!(!view_timeline_axis.inherited);
        assert_eq!(view_timeline_axis.initial, "block");

        // view-timeline-inset
        let view_timeline_inset =
            lookup("view-timeline-inset").expect("view-timeline-inset must be registered");
        assert!(!view_timeline_inset.inherited);
        assert_eq!(view_timeline_inset.initial, "auto");

        // container-name
        let container_name = lookup("container-name").expect("container-name must be registered");
        assert!(!container_name.inherited);
        assert_eq!(container_name.initial, "none");

        // container-type
        let container_type = lookup("container-type").expect("container-type must be registered");
        assert!(!container_type.inherited);
        assert_eq!(container_type.initial, "normal");

        // aspect-ratio
        let aspect_ratio = lookup("aspect-ratio").expect("aspect-ratio must be registered");
        assert!(!aspect_ratio.inherited);
        assert_eq!(aspect_ratio.initial, "auto");

        // unicode-bidi
        let unicode_bidi = lookup("unicode-bidi").expect("unicode-bidi must be registered");
        assert!(!unicode_bidi.inherited);
        assert_eq!(unicode_bidi.initial, "normal");

        // grid-template-columns
        let gtc =
            lookup("grid-template-columns").expect("grid-template-columns must be registered");
        assert!(!gtc.inherited);
        assert_eq!(gtc.initial, "none");

        // grid-template-rows
        let gtr = lookup("grid-template-rows").expect("grid-template-rows must be registered");
        assert!(!gtr.inherited);
        assert_eq!(gtr.initial, "none");

        // grid-template-areas
        let gta = lookup("grid-template-areas").expect("grid-template-areas must be registered");
        assert!(!gta.inherited);
        assert_eq!(gta.initial, "none");

        // grid-auto-columns
        let gac = lookup("grid-auto-columns").expect("grid-auto-columns must be registered");
        assert!(!gac.inherited);
        assert_eq!(gac.initial, "auto");

        // grid-auto-rows
        let gar = lookup("grid-auto-rows").expect("grid-auto-rows must be registered");
        assert!(!gar.inherited);
        assert_eq!(gar.initial, "auto");

        // grid-auto-flow
        let gaf = lookup("grid-auto-flow").expect("grid-auto-flow must be registered");
        assert!(!gaf.inherited);
        assert_eq!(gaf.initial, "row");

        // box-shadow
        let box_shadow = lookup("box-shadow").expect("box-shadow must be registered");
        assert!(!box_shadow.inherited);
        assert_eq!(box_shadow.initial, "none");

        // Shorthands:
        // container
        let container_lh =
            shorthand_longhands("container").expect("container shorthand must be registered");
        assert_eq!(container_lh, &["container-name", "container-type"][..]);

        // grid-template
        let grid_template_lh = shorthand_longhands("grid-template")
            .expect("grid-template shorthand must be registered");
        assert_eq!(
            grid_template_lh,
            &[
                "grid-template-columns",
                "grid-template-rows",
                "grid-template-areas"
            ][..]
        );

        // view-timeline
        let view_timeline_lh = shorthand_longhands("view-timeline")
            .expect("view-timeline shorthand must be registered");
        assert_eq!(
            view_timeline_lh,
            &["view-timeline-name", "view-timeline-axis"][..]
        );
    }

    #[test]
    fn test_property_text_emphasis_shorthand_t0757() {
        let lh = shorthand_longhands("text-emphasis");
        assert!(lh.is_some());
        let lh = lh.unwrap();
        assert_eq!(lh, &["text-emphasis-style", "text-emphasis-color"][..]);
    }

    #[test]
    fn test_no_duplicate_names() {
        let mut names = HashSet::new();
        for prop in PROPERTY_METADATA {
            // Ensure names are stored in lowercase for consistency
            assert_eq!(
                prop.name,
                prop.name.to_lowercase(),
                "Property name '{}' must be lowercase",
                prop.name
            );
            assert!(
                names.insert(prop.name),
                "Duplicate property name found: {}",
                prop.name
            );
        }
        assert_eq!(names.len(), PROPERTY_METADATA.len());
    }

    #[test]
    fn test_shorthand_longhands() {
        assert_eq!(
            shorthand_longhands("margin"),
            Some(&["margin-top", "margin-right", "margin-bottom", "margin-left"][..])
        );
        assert_eq!(
            shorthand_longhands("margin-block"),
            Some(&["margin-block-start", "margin-block-end"][..])
        );
        assert_eq!(
            shorthand_longhands("margin-inline"),
            Some(&["margin-inline-start", "margin-inline-end"][..])
        );
        assert_eq!(
            shorthand_longhands("padding-block"),
            Some(&["padding-block-start", "padding-block-end"][..])
        );
        assert_eq!(
            shorthand_longhands("padding-inline"),
            Some(&["padding-inline-start", "padding-inline-end"][..])
        );

        let overflow = shorthand_longhands("OVERFLOW");
        assert!(overflow.is_some());
        assert_eq!(overflow.unwrap().len(), 2);

        assert_eq!(shorthand_longhands("color"), None);

        let radius = shorthand_longhands("border-radius");
        assert!(radius.is_some());
        let radius_slice = radius.unwrap();
        assert_eq!(radius_slice.len(), 4);
        assert_eq!(radius_slice[0], "border-top-left-radius");

        assert_eq!(
            shorthand_longhands("border-top"),
            Some(&["border-top-width", "border-top-style", "border-top-color"][..])
        );
        assert_eq!(
            shorthand_longhands("border"),
            Some(&["border-width", "border-style", "border-color"][..])
        );
        assert_eq!(
            shorthand_longhands("border-style"),
            Some(
                &[
                    "border-top-style",
                    "border-right-style",
                    "border-bottom-style",
                    "border-left-style"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("font"),
            Some(
                &[
                    "font-style",
                    "font-variant",
                    "font-weight",
                    "font-size",
                    "line-height",
                    "font-family"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("Flex"),
            Some(&["flex-grow", "flex-shrink", "flex-basis"][..])
        );
        assert_eq!(
            shorthand_longhands("text-decoration"),
            Some(
                &[
                    "text-decoration-line",
                    "text-decoration-style",
                    "text-decoration-color",
                    "text-decoration-thickness"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("list-style"),
            Some(&["list-style-type", "list-style-position", "list-style-image"][..])
        );
        assert_eq!(
            shorthand_longhands("outline"),
            Some(&["outline-width", "outline-style", "outline-color"][..])
        );
        assert_eq!(
            shorthand_longhands("flex-flow"),
            Some(&["flex-direction", "flex-wrap"][..])
        );
        assert_eq!(
            shorthand_longhands("animation"),
            Some(
                &[
                    "animation-name",
                    "animation-duration",
                    "animation-timing-function",
                    "animation-delay",
                    "animation-iteration-count",
                    "animation-direction",
                    "animation-fill-mode",
                    "animation-play-state"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("background"),
            Some(
                &[
                    "background-color",
                    "background-image",
                    "background-position",
                    "background-size",
                    "background-repeat",
                    "background-origin",
                    "background-clip",
                    "background-attachment"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("transition"),
            Some(
                &[
                    "transition-property",
                    "transition-duration",
                    "transition-timing-function",
                    "transition-delay"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("grid-area"),
            Some(
                &[
                    "grid-row-start",
                    "grid-column-start",
                    "grid-row-end",
                    "grid-column-end"
                ][..]
            )
        );
        assert_eq!(
            shorthand_longhands("grid-column"),
            Some(&["grid-column-start", "grid-column-end"][..])
        );
        assert_eq!(
            shorthand_longhands("grid-row"),
            Some(&["grid-row-start", "grid-row-end"][..])
        );
        assert_eq!(
            shorthand_longhands("contain-intrinsic-size"),
            Some(&["contain-intrinsic-width", "contain-intrinsic-height"][..])
        );
        assert_eq!(
            shorthand_longhands("scroll-timeline"),
            Some(&["scroll-timeline-name", "scroll-timeline-axis"][..])
        );
        assert_eq!(shorthand_longhands("completely-unknown"), None);
    }

    #[test]
    fn test_shorthand_expansions_no_duplicates() {
        let mut names = HashSet::new();
        let mut last_name = "";
        for sh in SHORTHAND_EXPANSIONS {
            assert_eq!(
                sh.name,
                sh.name.to_lowercase(),
                "Shorthand name '{}' must be lowercase",
                sh.name
            );
            assert!(
                sh.name > last_name,
                "SHORTHAND_EXPANSIONS is not sorted alphabetically: '{}' comes after '{}'",
                sh.name,
                last_name
            );
            assert!(
                names.insert(sh.name),
                "Duplicate shorthand name found: {}",
                sh.name
            );
            last_name = sh.name;
        }
        assert_eq!(names.len(), SHORTHAND_EXPANSIONS.len());
    }

    #[test]
    fn test_additive_properties_t0806() {
        // interpolate-size (CSS Values 5; inherited; initial "numeric-only")
        let interp = lookup("interpolate-size").expect("interpolate-size must be registered");
        assert_eq!(interp.name, "interpolate-size");
        assert!(interp.inherited);
        assert_eq!(interp.initial, "numeric-only");

        // speak (CSS Speech; inherited; initial "auto")
        let sp = lookup("speak").expect("speak must be registered");
        assert_eq!(sp.name, "speak");
        assert!(sp.inherited);
        assert_eq!(sp.initial, "auto");

        // speak-as (CSS Speech; not inherited; initial "normal")
        let sp_as = lookup("speak-as").expect("speak-as must be registered");
        assert_eq!(sp_as.name, "speak-as");
        assert!(!sp_as.inherited);
        assert_eq!(sp_as.initial, "normal");

        // text-spacing (CSS Text 4 shorthand; not inherited; initial "normal")
        let txt_sp = lookup("text-spacing").expect("text-spacing must be registered");
        assert_eq!(txt_sp.name, "text-spacing");
        assert!(!txt_sp.inherited);
        assert_eq!(txt_sp.initial, "normal");

        let txt_sp_longhands = shorthand_longhands("text-spacing")
            .expect("text-spacing shorthand expansion must be registered");
        assert_eq!(txt_sp_longhands, &["text-autospace", "text-spacing-trim"]);

        // line-fit-edge (CSS Inline 3; not inherited; initial "leading")
        let line_fit = lookup("line-fit-edge").expect("line-fit-edge must be registered");
        assert_eq!(line_fit.name, "line-fit-edge");
        assert!(!line_fit.inherited);
        assert_eq!(line_fit.initial, "leading");
    }

    #[test]
    fn test_additive_properties_t0819() {
        // font-kerning (inherited; initial "auto")
        let fk = lookup("font-kerning").expect("font-kerning must be registered");
        assert_eq!(fk.name, "font-kerning");
        assert!(fk.inherited);
        assert_eq!(fk.initial, "auto");

        // font-optical-sizing (inherited; initial "auto")
        let fos = lookup("font-optical-sizing").expect("font-optical-sizing must be registered");
        assert_eq!(fos.name, "font-optical-sizing");
        assert!(fos.inherited);
        assert_eq!(fos.initial, "auto");

        // font-palette (inherited; initial "normal")
        let fp = lookup("font-palette").expect("font-palette must be registered");
        assert_eq!(fp.name, "font-palette");
        assert!(fp.inherited);
        assert_eq!(fp.initial, "normal");

        // font-variant-caps (inherited; initial "normal")
        let fvc = lookup("font-variant-caps").expect("font-variant-caps must be registered");
        assert_eq!(fvc.name, "font-variant-caps");
        assert!(fvc.inherited);
        assert_eq!(fvc.initial, "normal");

        // font-variant-ligatures (inherited; initial "normal")
        let fvl =
            lookup("font-variant-ligatures").expect("font-variant-ligatures must be registered");
        assert_eq!(fvl.name, "font-variant-ligatures");
        assert!(fvl.inherited);
        assert_eq!(fvl.initial, "normal");

        // font-variant-numeric (inherited; initial "normal")
        let fvn = lookup("font-variant-numeric").expect("font-variant-numeric must be registered");
        assert_eq!(fvn.name, "font-variant-numeric");
        assert!(fvn.inherited);
        assert_eq!(fvn.initial, "normal");

        // font-variant-position (inherited; initial "normal")
        let fvp =
            lookup("font-variant-position").expect("font-variant-position must be registered");
        assert_eq!(fvp.name, "font-variant-position");
        assert!(fvp.inherited);
        assert_eq!(fvp.initial, "normal");

        // font-variant-east-asian (inherited; initial "normal")
        let fvea =
            lookup("font-variant-east-asian").expect("font-variant-east-asian must be registered");
        assert_eq!(fvea.name, "font-variant-east-asian");
        assert!(fvea.inherited);
        assert_eq!(fvea.initial, "normal");

        // font-variant-alternates (inherited; initial "normal")
        let fva =
            lookup("font-variant-alternates").expect("font-variant-alternates must be registered");
        assert_eq!(fva.name, "font-variant-alternates");
        assert!(fva.inherited);
        assert_eq!(fva.initial, "normal");

        // font-synthesis-weight (inherited; initial "auto")
        let fsw =
            lookup("font-synthesis-weight").expect("font-synthesis-weight must be registered");
        assert_eq!(fsw.name, "font-synthesis-weight");
        assert!(fsw.inherited);
        assert_eq!(fsw.initial, "auto");

        // font-synthesis-style (inherited; initial "auto")
        let fss = lookup("font-synthesis-style").expect("font-synthesis-style must be registered");
        assert_eq!(fss.name, "font-synthesis-style");
        assert!(fss.inherited);
        assert_eq!(fss.initial, "auto");

        // font-synthesis-small-caps (inherited; initial "auto")
        let fssc = lookup("font-synthesis-small-caps")
            .expect("font-synthesis-small-caps must be registered");
        assert_eq!(fssc.name, "font-synthesis-small-caps");
        assert!(fssc.inherited);
        assert_eq!(fssc.initial, "auto");

        // will-change (not inherited; initial "auto")
        let wc = lookup("will-change").expect("will-change must be registered");
        assert_eq!(wc.name, "will-change");
        assert!(!wc.inherited);
        assert_eq!(wc.initial, "auto");

        // touch-action (not inherited; initial "auto")
        let ta = lookup("touch-action").expect("touch-action must be registered");
        assert_eq!(ta.name, "touch-action");
        assert!(!ta.inherited);
        assert_eq!(ta.initial, "auto");
    }

    #[test]
    fn test_additive_properties_t0834() {
        // overscroll-behavior-block (not inherited; initial "auto")
        let ob_b = lookup("overscroll-behavior-block")
            .expect("overscroll-behavior-block must be registered");
        assert_eq!(ob_b.name, "overscroll-behavior-block");
        assert!(!ob_b.inherited);
        assert_eq!(ob_b.initial, "auto");

        // overscroll-behavior-inline (not inherited; initial "auto")
        let ob_i = lookup("overscroll-behavior-inline")
            .expect("overscroll-behavior-inline must be registered");
        assert_eq!(ob_i.name, "overscroll-behavior-inline");
        assert!(!ob_i.inherited);
        assert_eq!(ob_i.initial, "auto");

        // overscroll-behavior shorthand expansion
        let ob_sh = shorthand_longhands("overscroll-behavior")
            .expect("overscroll-behavior shorthand must be registered");
        assert_eq!(
            ob_sh,
            &["overscroll-behavior-x", "overscroll-behavior-y"][..]
        );

        // scroll-margin shorthand expansion
        let sm_sh = shorthand_longhands("scroll-margin")
            .expect("scroll-margin shorthand must be registered");
        assert_eq!(
            sm_sh,
            &[
                "scroll-margin-top",
                "scroll-margin-right",
                "scroll-margin-bottom",
                "scroll-margin-left"
            ][..]
        );

        // scroll-margin-block shorthand expansion
        let smb_sh = shorthand_longhands("scroll-margin-block")
            .expect("scroll-margin-block shorthand must be registered");
        assert_eq!(
            smb_sh,
            &["scroll-margin-block-start", "scroll-margin-block-end"][..]
        );

        // scroll-margin-inline shorthand expansion
        let smi_sh = shorthand_longhands("scroll-margin-inline")
            .expect("scroll-margin-inline shorthand must be registered");
        assert_eq!(
            smi_sh,
            &["scroll-margin-inline-start", "scroll-margin-inline-end"][..]
        );

        // scroll-padding shorthand expansion
        let sp_sh = shorthand_longhands("scroll-padding")
            .expect("scroll-padding shorthand must be registered");
        assert_eq!(
            sp_sh,
            &[
                "scroll-padding-top",
                "scroll-padding-right",
                "scroll-padding-bottom",
                "scroll-padding-left"
            ][..]
        );

        // scroll-padding-block shorthand expansion
        let spb_sh = shorthand_longhands("scroll-padding-block")
            .expect("scroll-padding-block shorthand must be registered");
        assert_eq!(
            spb_sh,
            &["scroll-padding-block-start", "scroll-padding-block-end"][..]
        );

        // scroll-padding-inline shorthand expansion
        let spi_sh = shorthand_longhands("scroll-padding-inline")
            .expect("scroll-padding-inline shorthand must be registered");
        assert_eq!(
            spi_sh,
            &["scroll-padding-inline-start", "scroll-padding-inline-end"][..]
        );
    }

    #[test]
    fn test_additive_properties_t0849() {
        let props = [
            ("transform", false, "none", true),
            ("transform-origin", false, "50% 50%", true),
            ("translate", false, "none", true),
            ("scale", false, "none", true),
            ("rotate", false, "none", true),
            ("perspective", false, "none", true),
            ("animation-name", false, "none", false),
            ("animation-duration", false, "0s", false),
            ("animation-timing-function", false, "ease", false),
            ("animation-delay", false, "0s", false),
            ("animation-iteration-count", false, "1", false),
            ("animation-direction", false, "normal", false),
            ("animation-fill-mode", false, "none", false),
            ("animation-play-state", false, "running", false),
        ];

        for (name, inherited, initial, animatable) in props {
            let meta =
                lookup(name).unwrap_or_else(|| panic!("property {} must be registered", name));
            assert_eq!(meta.name, name);
            assert_eq!(meta.inherited, inherited, "inherited mismatch for {}", name);
            assert_eq!(meta.initial, initial, "initial mismatch for {}", name);
            assert_eq!(
                meta.animatable, animatable,
                "animatable mismatch for {}",
                name
            );
        }
    }

    #[test]
    fn test_additive_properties_t0862() {
        let props = [
            ("word-wrap", true, "normal", false),
            ("column-span", false, "none", false),
            ("column-fill", false, "balance", false),
            ("background-blend-mode", false, "normal", false),
        ];

        for (name, inherited, initial, animatable) in props {
            let meta =
                lookup(name).unwrap_or_else(|| panic!("property {} must be registered", name));
            assert_eq!(meta.name, name);
            assert_eq!(meta.inherited, inherited, "inherited mismatch for {}", name);
            assert_eq!(meta.initial, initial, "initial mismatch for {}", name);
            assert_eq!(
                meta.animatable, animatable,
                "animatable mismatch for {}",
                name
            );
        }
    }

    #[test]
    fn test_additive_properties_t0884() {
        let props = [
            ("background-origin", false, "padding-box", false),
            ("background-clip", false, "border-box", false),
            ("grid-row-start", false, "auto", false),
            ("grid-row-end", false, "auto", false),
            ("grid-column-start", false, "auto", false),
            ("grid-column-end", false, "auto", false),
            ("overflow-x", false, "visible", false),
            ("overflow-y", false, "visible", false),
            ("justify-self", false, "auto", false),
        ];

        for (name, inherited, initial, animatable) in props {
            let meta =
                lookup(name).unwrap_or_else(|| panic!("property {} must be registered", name));
            assert_eq!(meta.name, name);
            assert_eq!(meta.inherited, inherited, "inherited mismatch for {}", name);
            assert_eq!(meta.initial, initial, "initial mismatch for {}", name);
            assert_eq!(
                meta.animatable, animatable,
                "animatable mismatch for {}",
                name
            );
        }
    }

    #[test]
    fn test_additive_properties_t0905() {
        let props = [
            ("font-feature-settings", true, "normal", false),
            ("font-variation-settings", true, "normal", true),
            ("font-language-override", true, "normal", false),
            ("appearance", false, "none", false),
            ("counter-set", false, "none", false),
            ("column-rule-width", false, "medium", true),
            ("column-rule-style", false, "none", false),
            ("column-rule-color", false, "currentcolor", true),
            ("background-repeat-x", false, "repeat", false),
            ("background-repeat-y", false, "repeat", false),
            ("background-position-x", false, "0%", true),
            ("background-position-y", false, "0%", true),
        ];

        for (name, inherited, initial, animatable) in props {
            let meta =
                lookup(name).unwrap_or_else(|| panic!("property {} must be registered", name));
            assert_eq!(meta.name, name);
            assert_eq!(meta.inherited, inherited, "inherited mismatch for {}", name);
            assert_eq!(meta.initial, initial, "initial mismatch for {}", name);
            assert_eq!(
                meta.animatable, animatable,
                "animatable mismatch for {}",
                name
            );
        }

        // Test shorthands
        let cr =
            shorthand_longhands("column-rule").expect("column-rule shorthand must be registered");
        assert_eq!(
            cr,
            &[
                "column-rule-width",
                "column-rule-style",
                "column-rule-color"
            ]
        );

        let grid = shorthand_longhands("grid").expect("grid shorthand must be registered");
        assert_eq!(
            grid,
            &[
                "grid-template-rows",
                "grid-template-columns",
                "grid-template-areas",
                "grid-auto-rows",
                "grid-auto-columns",
                "grid-auto-flow",
            ]
        );
    }

    #[test]
    fn test_additive_properties_t0924() {
        let props = [
            ("content", false, "normal", false),
            ("text-size-adjust", true, "auto", false),
            ("transition-behavior", false, "normal", false),
            ("fill", true, "black", true),
            ("stroke", true, "none", true),
            ("stroke-width", true, "1", true),
            ("paint-order", true, "normal", false),
            ("image-orientation", true, "from-image", false),
        ];

        for (name, inherited, initial, animatable) in props {
            let meta =
                lookup(name).unwrap_or_else(|| panic!("property {} must be registered", name));
            assert_eq!(meta.name, name);
            assert_eq!(meta.inherited, inherited, "inherited mismatch for {}", name);
            assert_eq!(meta.initial, initial, "initial mismatch for {}", name);
            assert_eq!(
                meta.animatable, animatable,
                "animatable mismatch for {}",
                name
            );
        }

        // Test caret shorthand
        let caret_lh = shorthand_longhands("caret").expect("caret shorthand must be registered");
        assert_eq!(caret_lh, &["caret-color", "caret-shape"][..]);
    }

    #[test]
    fn test_additive_properties_t0950() {
        // Verify pointer-events is inherited as per spec-correct classification
        let pe = lookup("pointer-events").expect("pointer-events must be registered");
        assert_eq!(pe.name, "pointer-events");
        assert!(pe.inherited, "pointer-events should be inherited");

        // Verify the 11 added shorthand expansions
        let bg_pos = shorthand_longhands("background-position")
            .expect("background-position shorthand must be registered");
        assert_eq!(
            bg_pos,
            &["background-position-x", "background-position-y"][..]
        );

        let bg_rep = shorthand_longhands("background-repeat")
            .expect("background-repeat shorthand must be registered");
        assert_eq!(bg_rep, &["background-repeat-x", "background-repeat-y"][..]);

        let border_block =
            shorthand_longhands("border-block").expect("border-block shorthand must be registered");
        assert_eq!(
            border_block,
            &[
                "border-block-start-width",
                "border-block-start-style",
                "border-block-start-color",
                "border-block-end-width",
                "border-block-end-style",
                "border-block-end-color",
            ][..]
        );

        let border_block_end = shorthand_longhands("border-block-end")
            .expect("border-block-end shorthand must be registered");
        assert_eq!(
            border_block_end,
            &[
                "border-block-end-width",
                "border-block-end-style",
                "border-block-end-color",
            ][..]
        );

        let border_block_start = shorthand_longhands("border-block-start")
            .expect("border-block-start shorthand must be registered");
        assert_eq!(
            border_block_start,
            &[
                "border-block-start-width",
                "border-block-start-style",
                "border-block-start-color",
            ][..]
        );

        let border_inline = shorthand_longhands("border-inline")
            .expect("border-inline shorthand must be registered");
        assert_eq!(
            border_inline,
            &[
                "border-inline-start-width",
                "border-inline-start-style",
                "border-inline-start-color",
                "border-inline-end-width",
                "border-inline-end-style",
                "border-inline-end-color",
            ][..]
        );

        let border_inline_end = shorthand_longhands("border-inline-end")
            .expect("border-inline-end shorthand must be registered");
        assert_eq!(
            border_inline_end,
            &[
                "border-inline-end-width",
                "border-inline-end-style",
                "border-inline-end-color",
            ][..]
        );

        let border_inline_start = shorthand_longhands("border-inline-start")
            .expect("border-inline-start shorthand must be registered");
        assert_eq!(
            border_inline_start,
            &[
                "border-inline-start-width",
                "border-inline-start-style",
                "border-inline-start-color",
            ][..]
        );

        let font_variant =
            shorthand_longhands("font-variant").expect("font-variant shorthand must be registered");
        assert_eq!(
            font_variant,
            &[
                "font-variant-ligatures",
                "font-variant-caps",
                "font-variant-numeric",
                "font-variant-east-asian",
                "font-variant-alternates",
                "font-variant-position",
                "font-variant-emoji",
            ][..]
        );

        let inset_block =
            shorthand_longhands("inset-block").expect("inset-block shorthand must be registered");
        assert_eq!(inset_block, &["inset-block-start", "inset-block-end"][..]);

        let inset_inline =
            shorthand_longhands("inset-inline").expect("inset-inline shorthand must be registered");
        assert_eq!(
            inset_inline,
            &["inset-inline-start", "inset-inline-end"][..]
        );
    }

    #[test]
    fn test_additive_properties_t0970() {
        // 1. Verify font-variant-emoji is registered correctly
        let fve = lookup("font-variant-emoji").expect("font-variant-emoji must be registered");
        assert_eq!(fve.name, "font-variant-emoji");
        assert!(fve.inherited, "font-variant-emoji should be inherited");
        assert_eq!(fve.initial, "normal");
        assert!(!fve.animatable);

        // 2. Verify font-synthesis shorthand and its longhands
        let fs_sh = shorthand_longhands("font-synthesis")
            .expect("font-synthesis shorthand must be registered");
        assert_eq!(
            fs_sh,
            &[
                "font-synthesis-weight",
                "font-synthesis-style",
                "font-synthesis-small-caps",
            ][..]
        );

        // 3. Verify border-image shorthand and its longhands
        let bi_sh =
            shorthand_longhands("border-image").expect("border-image shorthand must be registered");
        assert_eq!(
            bi_sh,
            &[
                "border-image-source",
                "border-image-slice",
                "border-image-width",
                "border-image-outset",
                "border-image-repeat",
            ][..]
        );

        // 4. Verify border-image-* longhand properties in PROPERTY_METADATA
        let bis = lookup("border-image-source").expect("border-image-source must be registered");
        assert_eq!(bis.name, "border-image-source");
        assert!(!bis.inherited);
        assert_eq!(bis.initial, "none");
        assert!(!bis.animatable);

        let bisl = lookup("border-image-slice").expect("border-image-slice must be registered");
        assert_eq!(bisl.name, "border-image-slice");
        assert!(!bisl.inherited);
        assert_eq!(bisl.initial, "100%");
        assert!(bisl.animatable);

        let biw = lookup("border-image-width").expect("border-image-width must be registered");
        assert_eq!(biw.name, "border-image-width");
        assert!(!biw.inherited);
        assert_eq!(biw.initial, "1");
        assert!(biw.animatable);

        let bio = lookup("border-image-outset").expect("border-image-outset must be registered");
        assert_eq!(bio.name, "border-image-outset");
        assert!(!bio.inherited);
        assert_eq!(bio.initial, "0");
        assert!(bio.animatable);

        let bir = lookup("border-image-repeat").expect("border-image-repeat must be registered");
        assert_eq!(bir.name, "border-image-repeat");
        assert!(!bir.inherited);
        assert_eq!(bir.initial, "stretch");
        assert!(!bir.animatable);
    }

    #[test]
    fn test_additive_properties_t0993() {
        // 1. Verify longhands
        let oo = lookup("outline-offset").expect("outline-offset must be registered");
        assert_eq!(oo.name, "outline-offset");
        assert!(!oo.inherited);
        assert_eq!(oo.initial, "0");
        assert!(oo.animatable);

        let grg = lookup("grid-row-gap").expect("grid-row-gap must be registered");
        assert_eq!(grg.name, "grid-row-gap");
        assert!(!grg.inherited);
        assert_eq!(grg.initial, "normal");
        assert!(grg.animatable);

        let gcg = lookup("grid-column-gap").expect("grid-column-gap must be registered");
        assert_eq!(gcg.name, "grid-column-gap");
        assert!(!gcg.inherited);
        assert_eq!(gcg.initial, "normal");
        assert!(gcg.animatable);

        let pbb = lookup("page-break-before").expect("page-break-before must be registered");
        assert_eq!(pbb.name, "page-break-before");
        assert!(!pbb.inherited);
        assert_eq!(pbb.initial, "auto");
        assert!(!pbb.animatable);

        let pba = lookup("page-break-after").expect("page-break-after must be registered");
        assert_eq!(pba.name, "page-break-after");
        assert!(!pba.inherited);
        assert_eq!(pba.initial, "auto");
        assert!(!pba.animatable);

        let pbi = lookup("page-break-inside").expect("page-break-inside must be registered");
        assert_eq!(pbi.name, "page-break-inside");
        assert!(!pbi.inherited);
        assert_eq!(pbi.initial, "auto");
        assert!(!pbi.animatable);

        // 2. Verify logical shorthands in SHORTHAND_EXPANSIONS
        let bbc = shorthand_longhands("border-block-color")
            .expect("border-block-color shorthand must be registered");
        assert_eq!(
            bbc,
            &["border-block-start-color", "border-block-end-color"][..]
        );

        let bbs = shorthand_longhands("border-block-style")
            .expect("border-block-style shorthand must be registered");
        assert_eq!(
            bbs,
            &["border-block-start-style", "border-block-end-style"][..]
        );

        let bbw = shorthand_longhands("border-block-width")
            .expect("border-block-width shorthand must be registered");
        assert_eq!(
            bbw,
            &["border-block-start-width", "border-block-end-width"][..]
        );

        let bic = shorthand_longhands("border-inline-color")
            .expect("border-inline-color shorthand must be registered");
        assert_eq!(
            bic,
            &["border-inline-start-color", "border-inline-end-color"][..]
        );

        let bis = shorthand_longhands("border-inline-style")
            .expect("border-inline-style shorthand must be registered");
        assert_eq!(
            bis,
            &["border-inline-start-style", "border-inline-end-style"][..]
        );

        let biw = shorthand_longhands("border-inline-width")
            .expect("border-inline-width shorthand must be registered");
        assert_eq!(
            biw,
            &["border-inline-start-width", "border-inline-end-width"][..]
        );

        let gg = shorthand_longhands("grid-gap").expect("grid-gap shorthand must be registered");
        assert_eq!(gg, &["grid-row-gap", "grid-column-gap"][..]);

        // 3. Verify vendor-prefixed lookups (edge cases)
        let prefixed_width = lookup("-webkit-width").expect("should find width");
        assert_eq!(prefixed_width.name, "width");

        let prefixed_color = lookup("-moz-color").expect("should find color");
        assert_eq!(prefixed_color.name, "color");

        let prefixed_margin_top = lookup("-ms-margin-top").expect("should find margin-top");
        assert_eq!(prefixed_margin_top.name, "margin-top");

        let prefixed_opacity = lookup("-o-opacity").expect("should find opacity");
        assert_eq!(prefixed_opacity.name, "opacity");

        let prefixed_border_radius = shorthand_longhands("-webkit-border-radius")
            .expect("should find border-radius shorthands");
        assert_eq!(prefixed_border_radius[0], "border-top-left-radius");

        let prefixed_transition =
            shorthand_longhands("-moz-transition").expect("should find transition shorthands");
        assert_eq!(prefixed_transition[0], "transition-property");

        let prefixed_border_block =
            shorthand_longhands("-ms-border-block").expect("should find border-block shorthands");
        assert_eq!(prefixed_border_block[0], "border-block-start-width");

        // 4. Verify is_valid_property_name helper
        assert!(is_valid_property_name("width"));
        assert!(is_valid_property_name("margin"));
        assert!(is_valid_property_name("-webkit-width"));
        assert!(is_valid_property_name("-moz-border-radius"));
        assert!(is_valid_property_name("--custom-variable-name"));
        assert!(is_valid_property_name("  --with-spaces  "));
        assert!(is_valid_property_name("  -webkit-transform  "));
        assert!(!is_valid_property_name("--"));
        assert!(!is_valid_property_name("-"));
        assert!(!is_valid_property_name("completely-unknown-property"));

        // 5. Verify is_css_wide_keyword helper
        assert!(is_css_wide_keyword("initial"));
        assert!(is_css_wide_keyword("  INHERIT  "));
        assert!(is_css_wide_keyword("unset"));
        assert!(is_css_wide_keyword("revert"));
        assert!(is_css_wide_keyword("revert-layer"));
        assert!(!is_css_wide_keyword("none"));
        assert!(!is_css_wide_keyword("auto"));
        assert!(!is_css_wide_keyword("solid"));
        assert!(!is_css_wide_keyword("red"));
    }

    #[test]
    fn test_shorthand_expansion_edge_cases_t1014() {
        // 1. Margin expansions
        let m1 = expand_shorthand_values("margin", &["10px"]).unwrap();
        assert_eq!(m1.len(), 4);
        assert_eq!(m1[0].name, "margin-top");
        assert_eq!(m1[0].value, "10px");
        assert_eq!(m1[1].name, "margin-right");
        assert_eq!(m1[1].value, "10px");
        assert_eq!(m1[2].name, "margin-bottom");
        assert_eq!(m1[2].value, "10px");
        assert_eq!(m1[3].name, "margin-left");
        assert_eq!(m1[3].value, "10px");

        let m2 = expand_shorthand_values("margin", &["10px", "20px"]).unwrap();
        assert_eq!(m2[0].value, "10px");
        assert_eq!(m2[1].value, "20px");
        assert_eq!(m2[2].value, "10px");
        assert_eq!(m2[3].value, "20px");

        let m3 = expand_shorthand_values("margin", &["10px", "20px", "30px"]).unwrap();
        assert_eq!(m3[0].value, "10px");
        assert_eq!(m3[1].value, "20px");
        assert_eq!(m3[2].value, "30px");
        assert_eq!(m3[3].value, "20px");

        let m4 = expand_shorthand_values("margin", &["10px", "20px", "30px", "40px"]).unwrap();
        assert_eq!(m4[0].value, "10px");
        assert_eq!(m4[1].value, "20px");
        assert_eq!(m4[2].value, "30px");
        assert_eq!(m4[3].value, "40px");

        // 2. CSS-wide keyword inheritance edge cases
        let m_inherit = expand_shorthand_values("margin", &["inherit"]).unwrap();
        assert_eq!(m_inherit.len(), 4);
        assert_eq!(m_inherit[0].value, "inherit");
        assert_eq!(m_inherit[3].value, "inherit");

        assert_eq!(
            expand_shorthand_values("margin", &["inherit", "10px"]),
            Err(ShorthandError::InvalidValue)
        );

        // 3. Border edges (order independent)
        let bt1 = expand_shorthand_values("border-top", &["solid"]).unwrap();
        assert_eq!(bt1[0].value, "medium"); // width
        assert_eq!(bt1[1].value, "solid"); // style
        assert_eq!(bt1[2].value, "currentcolor"); // color

        let bt2 = expand_shorthand_values("border-top", &["1px", "red", "dashed"]).unwrap();
        assert_eq!(bt2[0].value, "1px");
        assert_eq!(bt2[1].value, "dashed");
        assert_eq!(bt2[2].value, "red");

        // 4. Border shorthand (sets all 4 edges)
        let b1 = expand_shorthand_values("border", &["5px", "double"]).unwrap();
        assert_eq!(b1.len(), 12);
        assert_eq!(b1[0].name, "border-top-width");
        assert_eq!(b1[0].value, "5px");
        assert_eq!(b1[1].name, "border-top-style");
        assert_eq!(b1[1].value, "double");
        assert_eq!(b1[2].name, "border-top-color");
        assert_eq!(b1[2].value, "currentcolor");
        assert_eq!(b1[9].name, "border-left-width");
        assert_eq!(b1[9].value, "5px");
        assert_eq!(b1[10].name, "border-left-style");
        assert_eq!(b1[10].value, "double");
        assert_eq!(b1[11].name, "border-left-color");
        assert_eq!(b1[11].value, "currentcolor");

        // 5. Border-radius with slash horizontal/vertical corners expansion
        let br1 = expand_shorthand_values("border-radius", &["10px", "20px", "/", "30px"]).unwrap();
        assert_eq!(br1.len(), 4);
        assert_eq!(br1[0].name, "border-top-left-radius");
        assert_eq!(br1[0].value, "10px 30px");
        assert_eq!(br1[1].name, "border-top-right-radius");
        assert_eq!(br1[1].value, "20px 30px");
        assert_eq!(br1[2].name, "border-bottom-right-radius");
        assert_eq!(br1[2].value, "10px 30px");
        assert_eq!(br1[3].name, "border-bottom-left-radius");
        assert_eq!(br1[3].value, "20px 30px");

        let br2 = expand_shorthand_values("border-radius", &["5px"]).unwrap();
        assert_eq!(br2[0].value, "5px");
        assert_eq!(br2[3].value, "5px");

        assert_eq!(
            expand_shorthand_values("border-radius", &["5px", "/", "/", "10px"]),
            Err(ShorthandError::InvalidValue)
        );
    }

    #[test]
    fn test_shorthand_expansion_extended_t1033() {
        // 1. inset, scroll-margin, scroll-padding (1-4 values positional)
        let ins1 = expand_shorthand_values("inset", &["10px"]).unwrap();
        assert_eq!(ins1.len(), 4);
        assert_eq!(ins1[0].name, "top");
        assert_eq!(ins1[0].value, "10px");
        assert_eq!(ins1[1].name, "right");
        assert_eq!(ins1[1].value, "10px");
        assert_eq!(ins1[2].name, "bottom");
        assert_eq!(ins1[2].value, "10px");
        assert_eq!(ins1[3].name, "left");
        assert_eq!(ins1[3].value, "10px");

        let ins2 = expand_shorthand_values("inset", &["10px", "20px"]).unwrap();
        assert_eq!(ins2[0].value, "10px"); // top
        assert_eq!(ins2[1].value, "20px"); // right
        assert_eq!(ins2[2].value, "10px"); // bottom
        assert_eq!(ins2[3].value, "20px"); // left

        let ins3 = expand_shorthand_values("inset", &["10px", "20px", "30px"]).unwrap();
        assert_eq!(ins3[0].value, "10px"); // top
        assert_eq!(ins3[1].value, "20px"); // right
        assert_eq!(ins3[2].value, "30px"); // bottom
        assert_eq!(ins3[3].value, "20px"); // left

        let ins4 = expand_shorthand_values("inset", &["10px", "20px", "30px", "40px"]).unwrap();
        assert_eq!(ins4[0].value, "10px"); // top
        assert_eq!(ins4[1].value, "20px"); // right
        assert_eq!(ins4[2].value, "30px"); // bottom
        assert_eq!(ins4[3].value, "40px"); // left

        assert_eq!(
            expand_shorthand_values("inset", &["10px", "20px", "30px", "40px", "50px"]),
            Err(ShorthandError::TooManyValues)
        );

        // scroll-margin and scroll-padding
        let sm = expand_shorthand_values("scroll-margin", &["5px", "15px"]).unwrap();
        assert_eq!(sm.len(), 4);
        assert_eq!(sm[0].name, "scroll-margin-top");
        assert_eq!(sm[0].value, "5px");
        assert_eq!(sm[1].name, "scroll-margin-right");
        assert_eq!(sm[1].value, "15px");

        let sp = expand_shorthand_values("scroll-padding", &["2px"]).unwrap();
        assert_eq!(sp.len(), 4);
        assert_eq!(sp[0].name, "scroll-padding-top");
        assert_eq!(sp[0].value, "2px");

        // 2. 1-to-2 positional (simple directional / logical)
        let mb1 = expand_shorthand_values("margin-block", &["10px"]).unwrap();
        assert_eq!(mb1.len(), 2);
        assert_eq!(mb1[0].name, "margin-block-start");
        assert_eq!(mb1[0].value, "10px");
        assert_eq!(mb1[1].name, "margin-block-end");
        assert_eq!(mb1[1].value, "10px");

        let mb2 = expand_shorthand_values("margin-block", &["10px", "20px"]).unwrap();
        assert_eq!(mb2[0].value, "10px");
        assert_eq!(mb2[1].value, "20px");

        assert_eq!(
            expand_shorthand_values("margin-block", &["10px", "20px", "30px"]),
            Err(ShorthandError::TooManyValues)
        );

        let gp = expand_shorthand_values("gap", &["20px", "30px"]).unwrap();
        assert_eq!(gp.len(), 2);
        assert_eq!(gp[0].name, "row-gap");
        assert_eq!(gp[0].value, "20px");
        assert_eq!(gp[1].name, "column-gap");
        assert_eq!(gp[1].value, "30px");

        let ob = expand_shorthand_values("overscroll-behavior", &["contain"]).unwrap();
        assert_eq!(ob.len(), 2);
        assert_eq!(ob[0].name, "overscroll-behavior-x");
        assert_eq!(ob[0].value, "contain");
        assert_eq!(ob[1].name, "overscroll-behavior-y");
        assert_eq!(ob[1].value, "contain");

        // 3. scroll-timeline and view-timeline (1-to-2 with "block" default)
        let st1 = expand_shorthand_values("scroll-timeline", &["my-timeline"]).unwrap();
        assert_eq!(st1.len(), 2);
        assert_eq!(st1[0].name, "scroll-timeline-name");
        assert_eq!(st1[0].value, "my-timeline");
        assert_eq!(st1[1].name, "scroll-timeline-axis");
        assert_eq!(st1[1].value, "block");

        let st2 = expand_shorthand_values("scroll-timeline", &["my-timeline", "inline"]).unwrap();
        assert_eq!(st2[0].value, "my-timeline");
        assert_eq!(st2[1].value, "inline");

        // 4. border-block-start etc.
        let bbs1 = expand_shorthand_values("border-block-start", &["solid", "red"]).unwrap();
        assert_eq!(bbs1.len(), 3);
        assert_eq!(bbs1[0].name, "border-block-start-width");
        assert_eq!(bbs1[0].value, "medium");
        assert_eq!(bbs1[1].name, "border-block-start-style");
        assert_eq!(bbs1[1].value, "solid");
        assert_eq!(bbs1[2].name, "border-block-start-color");
        assert_eq!(bbs1[2].value, "red");

        // 5. border-block / border-inline (sets 6 longhands)
        let bb1 = expand_shorthand_values("border-block", &["5px", "dashed"]).unwrap();
        assert_eq!(bb1.len(), 6);
        assert_eq!(bb1[0].name, "border-block-start-width");
        assert_eq!(bb1[0].value, "5px");
        assert_eq!(bb1[1].name, "border-block-start-style");
        assert_eq!(bb1[1].value, "dashed");
        assert_eq!(bb1[2].name, "border-block-start-color");
        assert_eq!(bb1[2].value, "currentcolor");
        assert_eq!(bb1[3].name, "border-block-end-width");
        assert_eq!(bb1[3].value, "5px");
        assert_eq!(bb1[4].name, "border-block-end-style");
        assert_eq!(bb1[4].value, "dashed");
        assert_eq!(bb1[5].name, "border-block-end-color");
        assert_eq!(bb1[5].value, "currentcolor");

        // 6. outline (sets width, style, color; style supports auto)
        let out1 = expand_shorthand_values("outline", &["2px", "auto", "blue"]).unwrap();
        assert_eq!(out1.len(), 3);
        assert_eq!(out1[0].name, "outline-width");
        assert_eq!(out1[0].value, "2px");
        assert_eq!(out1[1].name, "outline-style");
        assert_eq!(out1[1].value, "auto");
        assert_eq!(out1[2].name, "outline-color");
        assert_eq!(out1[2].value, "blue");

        // 7. text-emphasis (up to 3 values positional/order-independent, combining non-color)
        let te1 = expand_shorthand_values("text-emphasis", &["filled", "circle", "red"]).unwrap();
        assert_eq!(te1.len(), 2);
        assert_eq!(te1[0].name, "text-emphasis-style");
        assert_eq!(te1[0].value, "filled circle");
        assert_eq!(te1[1].name, "text-emphasis-color");
        assert_eq!(te1[1].value, "red");

        let te2 = expand_shorthand_values("text-emphasis", &["red", "open"]).unwrap();
        assert_eq!(te2[0].value, "open");
        assert_eq!(te2[1].value, "red");

        // 8. caret
        let car1 = expand_shorthand_values("caret", &["red", "block"]).unwrap();
        assert_eq!(car1.len(), 2);
        assert_eq!(car1[0].name, "caret-color");
        assert_eq!(car1[0].value, "red");
        assert_eq!(car1[1].name, "caret-shape");
        assert_eq!(car1[1].value, "block");

        let car2 = expand_shorthand_values("caret", &["auto"]).unwrap();
        assert_eq!(car2[0].value, "auto");
        assert_eq!(car2[1].value, "auto");

        // 9. columns
        let col1 = expand_shorthand_values("columns", &["12em", "3"]).unwrap();
        assert_eq!(col1.len(), 2);
        assert_eq!(col1[0].name, "column-width");
        assert_eq!(col1[0].value, "12em");
        assert_eq!(col1[1].name, "column-count");
        assert_eq!(col1[1].value, "3");

        let col2 = expand_shorthand_values("columns", &["auto"]).unwrap();
        assert_eq!(col2[0].value, "auto");
        assert_eq!(col2[1].value, "auto");

        // 10. font-synthesis
        let fs1 = expand_shorthand_values("font-synthesis", &["none"]).unwrap();
        assert_eq!(fs1.len(), 3);
        assert_eq!(fs1[0].name, "font-synthesis-weight");
        assert_eq!(fs1[0].value, "none");
        assert_eq!(fs1[1].name, "font-synthesis-style");
        assert_eq!(fs1[1].value, "none");
        assert_eq!(fs1[2].name, "font-synthesis-small-caps");
        assert_eq!(fs1[2].value, "none");

        let fs2 = expand_shorthand_values("font-synthesis", &["weight", "small-caps"]).unwrap();
        assert_eq!(fs2[0].value, "auto");
        assert_eq!(fs2[1].value, "none");
        assert_eq!(fs2[2].value, "auto");

        // 11. overflow
        let ov1 = expand_shorthand_values("overflow", &["auto"]).unwrap();
        assert_eq!(ov1.len(), 2);
        assert_eq!(ov1[0].name, "overflow-x");
        assert_eq!(ov1[0].value, "auto");
        assert_eq!(ov1[1].name, "overflow-y");
        assert_eq!(ov1[1].value, "auto");

        let ov2 = expand_shorthand_values("overflow", &["scroll", "hidden"]).unwrap();
        assert_eq!(ov2[0].value, "scroll");
        assert_eq!(ov2[1].value, "hidden");

        // 12. flex-flow
        let ff1 = expand_shorthand_values("flex-flow", &["column-reverse"]).unwrap();
        assert_eq!(ff1.len(), 2);
        assert_eq!(ff1[0].name, "flex-direction");
        assert_eq!(ff1[0].value, "column-reverse");
        assert_eq!(ff1[1].name, "flex-wrap");
        assert_eq!(ff1[1].value, "nowrap");

        let ff2 = expand_shorthand_values("flex-flow", &["wrap", "row-reverse"]).unwrap();
        assert_eq!(ff2[0].value, "row-reverse");
        assert_eq!(ff2[1].value, "wrap");

        // 13. flex
        let fl1 = expand_shorthand_values("flex", &["none"]).unwrap();
        assert_eq!(fl1.len(), 3);
        assert_eq!(fl1[0].name, "flex-grow");
        assert_eq!(fl1[0].value, "0");
        assert_eq!(fl1[1].name, "flex-shrink");
        assert_eq!(fl1[1].value, "0");
        assert_eq!(fl1[2].name, "flex-basis");
        assert_eq!(fl1[2].value, "auto");

        let fl2 = expand_shorthand_values("flex", &["2"]).unwrap();
        assert_eq!(fl2[0].value, "2");
        assert_eq!(fl2[1].value, "1");
        assert_eq!(fl2[2].value, "0%");

        let fl3 = expand_shorthand_values("flex", &["10%"]).unwrap();
        assert_eq!(fl3[0].value, "1");
        assert_eq!(fl3[1].value, "1");
        assert_eq!(fl3[2].value, "10%");

        let fl4 = expand_shorthand_values("flex", &["2", "3"]).unwrap();
        assert_eq!(fl4[0].value, "2");
        assert_eq!(fl4[1].value, "3");
        assert_eq!(fl4[2].value, "0%");

        let fl5 = expand_shorthand_values("flex", &["2", "auto"]).unwrap();
        assert_eq!(fl5[0].value, "2");
        assert_eq!(fl5[1].value, "1");
        assert_eq!(fl5[2].value, "auto");

        let fl6 = expand_shorthand_values("flex", &["2", "3", "10px"]).unwrap();
        assert_eq!(fl6[0].value, "2");
        assert_eq!(fl6[1].value, "3");
        assert_eq!(fl6[2].value, "10px");

        // 14. grid-column
        let gc1 = expand_shorthand_values("grid-column", &["span 2"]).unwrap();
        assert_eq!(gc1.len(), 2);
        assert_eq!(gc1[0].name, "grid-column-start");
        assert_eq!(gc1[0].value, "span 2");
        assert_eq!(gc1[1].name, "grid-column-end");
        assert_eq!(gc1[1].value, "auto");

        let gc2 = expand_shorthand_values("grid-column", &["3", "/", "4"]).unwrap();
        assert_eq!(gc2[0].value, "3");
        assert_eq!(gc2[1].value, "4");

        // 15. list-style
        let ls1 = expand_shorthand_values("list-style", &["none"]).unwrap();
        assert_eq!(ls1.len(), 3);
        assert_eq!(ls1[0].name, "list-style-type");
        assert_eq!(ls1[0].value, "none");
        assert_eq!(ls1[1].name, "list-style-position");
        assert_eq!(ls1[1].value, "outside");
        assert_eq!(ls1[2].name, "list-style-image");
        assert_eq!(ls1[2].value, "none");

        let ls2 = expand_shorthand_values("list-style", &["inside", "square"]).unwrap();
        assert_eq!(ls2[0].value, "square");
        assert_eq!(ls2[1].value, "inside");
        assert_eq!(ls2[2].value, "none");

        let ls3 = expand_shorthand_values("list-style", &["url(bullet.png)", "inside"]).unwrap();
        assert_eq!(ls3[0].value, "disc");
        assert_eq!(ls3[1].value, "inside");
        assert_eq!(ls3[2].value, "url(bullet.png)");

        // 16. transition
        let tr1 = expand_shorthand_values("transition", &["opacity", "2s"]).unwrap();
        assert_eq!(tr1.len(), 4);
        assert_eq!(tr1[0].name, "transition-property");
        assert_eq!(tr1[0].value, "opacity");
        assert_eq!(tr1[1].name, "transition-duration");
        assert_eq!(tr1[1].value, "2s");
        assert_eq!(tr1[2].name, "transition-timing-function");
        assert_eq!(tr1[2].value, "ease");
        assert_eq!(tr1[3].name, "transition-delay");
        assert_eq!(tr1[3].value, "0s");

        let tr2 =
            expand_shorthand_values("transition", &["width", "0.5s", "linear", "1s"]).unwrap();
        assert_eq!(tr2[0].value, "width");
        assert_eq!(tr2[1].value, "0.5s");
        assert_eq!(tr2[2].value, "linear");
        assert_eq!(tr2[3].value, "1s");
    }

    #[test]
    fn test_logical_properties_mapping_t1014() {
        // Block and inline sizing
        assert_eq!(
            map_logical_to_physical("block-size", "horizontal-tb", "ltr"),
            Some("height")
        );
        assert_eq!(
            map_logical_to_physical("block-size", "vertical-rl", "ltr"),
            Some("width")
        );
        assert_eq!(
            map_logical_to_physical("inline-size", "horizontal-tb", "ltr"),
            Some("width")
        );
        assert_eq!(
            map_logical_to_physical("inline-size", "vertical-rl", "ltr"),
            Some("height")
        );

        // Margins and paddings
        assert_eq!(
            map_logical_to_physical("margin-block-start", "horizontal-tb", "ltr"),
            Some("margin-top")
        );
        assert_eq!(
            map_logical_to_physical("margin-block-end", "horizontal-tb", "ltr"),
            Some("margin-bottom")
        );
        assert_eq!(
            map_logical_to_physical("margin-inline-start", "horizontal-tb", "ltr"),
            Some("margin-left")
        );
        assert_eq!(
            map_logical_to_physical("margin-inline-start", "horizontal-tb", "rtl"),
            Some("margin-right")
        );
        assert_eq!(
            map_logical_to_physical("margin-inline-start", "vertical-rl", "ltr"),
            Some("margin-top")
        );
        assert_eq!(
            map_logical_to_physical("margin-inline-start", "vertical-rl", "rtl"),
            Some("margin-bottom")
        );

        assert_eq!(
            map_logical_to_physical("padding-inline-end", "horizontal-tb", "ltr"),
            Some("padding-right")
        );
        assert_eq!(
            map_logical_to_physical("padding-inline-end", "horizontal-tb", "rtl"),
            Some("padding-left")
        );

        // Insets
        assert_eq!(
            map_logical_to_physical("inset-block-start", "horizontal-tb", "ltr"),
            Some("top")
        );
        assert_eq!(
            map_logical_to_physical("inset-inline-start", "horizontal-tb", "ltr"),
            Some("left")
        );

        // Border width, style, color
        assert_eq!(
            map_logical_to_physical("border-block-start-width", "horizontal-tb", "ltr"),
            Some("border-top-width")
        );
        assert_eq!(
            map_logical_to_physical("border-inline-end-style", "horizontal-tb", "ltr"),
            Some("border-right-style")
        );
        assert_eq!(
            map_logical_to_physical("border-block-end-color", "horizontal-tb", "ltr"),
            Some("border-bottom-color")
        );

        // Border logical corners radius
        assert_eq!(
            map_logical_to_physical("border-start-start-radius", "horizontal-tb", "ltr"),
            Some("border-top-left-radius")
        );
        assert_eq!(
            map_logical_to_physical("border-start-start-radius", "horizontal-tb", "rtl"),
            Some("border-top-right-radius")
        );
        assert_eq!(
            map_logical_to_physical("border-start-start-radius", "vertical-rl", "ltr"),
            Some("border-top-right-radius")
        );
        assert_eq!(
            map_logical_to_physical("border-start-start-radius", "vertical-rl", "rtl"),
            Some("border-bottom-right-radius")
        );

        // Unknown
        assert_eq!(
            map_logical_to_physical("not-logical", "horizontal-tb", "ltr"),
            None
        );
    }

    #[test]
    fn test_custom_property_registration_validation_t1014() {
        // 1. Valid syntax * (any)
        let r1 = CustomPropertyRegistration {
            name: "--my-var".to_string(),
            syntax: "*".to_string(),
            inherits: true,
            initial_value: None,
        };
        assert_eq!(validate_custom_property_registration(&r1), Ok(()));

        // 2. Valid syntax <color>
        let r2 = CustomPropertyRegistration {
            name: "--my-color".to_string(),
            syntax: "<color>".to_string(),
            inherits: true,
            initial_value: Some("red".to_string()),
        };
        assert_eq!(validate_custom_property_registration(&r2), Ok(()));

        // 3. Valid keyword
        let r3 = CustomPropertyRegistration {
            name: "--my-keyword".to_string(),
            syntax: "auto".to_string(),
            inherits: false,
            initial_value: Some("auto".to_string()),
        };
        assert_eq!(validate_custom_property_registration(&r3), Ok(()));

        // 4. Valid alternation
        let r4 = CustomPropertyRegistration {
            name: "--my-alt".to_string(),
            syntax: "<length> | none".to_string(),
            inherits: true,
            initial_value: Some("none".to_string()),
        };
        assert_eq!(validate_custom_property_registration(&r4), Ok(()));

        // 5. Valid list multiplier (comma separated)
        let r5 = CustomPropertyRegistration {
            name: "--my-colors".to_string(),
            syntax: "<color>#".to_string(),
            inherits: true,
            initial_value: Some("red, blue, #fff".to_string()),
        };
        assert_eq!(validate_custom_property_registration(&r5), Ok(()));

        // 6. Invalid name (no double-hyphen or too short)
        let r_err1 = CustomPropertyRegistration {
            name: "invalid-name".to_string(),
            syntax: "*".to_string(),
            inherits: true,
            initial_value: None,
        };
        assert_eq!(
            validate_custom_property_registration(&r_err1),
            Err(CustomPropertyValidationError::InvalidName)
        );

        let r_err2 = CustomPropertyRegistration {
            name: "--".to_string(),
            syntax: "*".to_string(),
            inherits: true,
            initial_value: None,
        };
        assert_eq!(
            validate_custom_property_registration(&r_err2),
            Err(CustomPropertyValidationError::InvalidName)
        );

        // 7. Invalid syntax descriptor (unclosed brackets or unknown type)
        let r_err3 = CustomPropertyRegistration {
            name: "--my-var".to_string(),
            syntax: "<unknown>".to_string(),
            inherits: true,
            initial_value: Some("foo".to_string()),
        };
        assert_eq!(
            validate_custom_property_registration(&r_err3),
            Err(CustomPropertyValidationError::InvalidSyntax)
        );

        // 8. Missing initial value
        let r_err4 = CustomPropertyRegistration {
            name: "--my-var".to_string(),
            syntax: "<color>".to_string(),
            inherits: true,
            initial_value: None,
        };
        assert_eq!(
            validate_custom_property_registration(&r_err4),
            Err(CustomPropertyValidationError::MissingInitialValue)
        );

        // 9. Invalid initial value against color
        let r_err5 = CustomPropertyRegistration {
            name: "--my-var".to_string(),
            syntax: "<color>".to_string(),
            inherits: true,
            initial_value: Some("not-a-color".to_string()),
        };
        assert_eq!(
            validate_custom_property_registration(&r_err5),
            Err(CustomPropertyValidationError::InvalidInitialValue)
        );

        // 10. Invalid initial value against integer (float is invalid)
        let r_err6 = CustomPropertyRegistration {
            name: "--my-var".to_string(),
            syntax: "<integer>".to_string(),
            inherits: true,
            initial_value: Some("12.3".to_string()),
        };
        assert_eq!(
            validate_custom_property_registration(&r_err6),
            Err(CustomPropertyValidationError::InvalidInitialValue)
        );
    }
}
