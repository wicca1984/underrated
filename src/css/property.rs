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
    },
    PropertyMetadata {
        name: "font-family",
        inherited: true,
        initial: "serif",
    },
    PropertyMetadata {
        name: "font-size",
        inherited: true,
        initial: "medium",
    },
    PropertyMetadata {
        name: "font-size-adjust",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "font-style",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "font-weight",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "line-height",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-align",
        inherited: true,
        initial: "start",
    },
    PropertyMetadata {
        name: "letter-spacing",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "word-spacing",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "white-space",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "visibility",
        inherited: true,
        initial: "visible",
    },
    PropertyMetadata {
        name: "list-style-type",
        inherited: true,
        initial: "disc",
    },
    PropertyMetadata {
        name: "direction",
        inherited: true,
        initial: "ltr",
    },
    PropertyMetadata {
        name: "text-transform",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "cursor",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "font-variant",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "font-stretch",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-indent",
        inherited: true,
        initial: "0",
    },
    PropertyMetadata {
        name: "word-break",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "overflow-wrap",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-align-last",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "caption-side",
        inherited: true,
        initial: "top",
    },
    PropertyMetadata {
        name: "color-interpolation",
        inherited: true,
        initial: "sRGB",
    },
    PropertyMetadata {
        name: "empty-cells",
        inherited: true,
        initial: "show",
    },
    PropertyMetadata {
        name: "border-collapse",
        inherited: true,
        initial: "separate",
    },
    PropertyMetadata {
        name: "border-spacing",
        inherited: true,
        initial: "0",
    },
    PropertyMetadata {
        name: "list-style-position",
        inherited: true,
        initial: "outside",
    },
    PropertyMetadata {
        name: "list-style-image",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "quotes",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "tab-size",
        inherited: true,
        initial: "8",
    },
    PropertyMetadata {
        name: "hyphens",
        inherited: true,
        initial: "manual",
    },
    PropertyMetadata {
        name: "accent-color",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "caret-color",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "clip-rule",
        inherited: true,
        initial: "nonzero",
    },
    PropertyMetadata {
        name: "scrollbar-width",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scrollbar-color",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-wrap",
        inherited: true,
        initial: "wrap",
    },
    PropertyMetadata {
        name: "forced-color-adjust",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "caret-shape",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-autospace",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-spacing-trim",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "hyphenate-character",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "hyphenate-limit-chars",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "ruby-position",
        inherited: true,
        initial: "alternate",
    },
    PropertyMetadata {
        name: "ruby-align",
        inherited: true,
        initial: "space-around",
    },
    PropertyMetadata {
        name: "ruby-overhang",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "ruby-merge",
        inherited: true,
        initial: "separate",
    },
    PropertyMetadata {
        name: "math-style",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "math-depth",
        inherited: true,
        initial: "0",
    },
    PropertyMetadata {
        name: "line-break",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "white-space-collapse",
        inherited: true,
        initial: "collapse",
    },
    PropertyMetadata {
        name: "text-wrap-style",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-wrap-mode",
        inherited: true,
        initial: "wrap",
    },
    PropertyMetadata {
        name: "text-underline-position",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-emphasis-color",
        inherited: true,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "text-emphasis-style",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-emphasis-position",
        inherited: true,
        initial: "over right",
    },
    PropertyMetadata {
        name: "text-justify",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-combine-upright",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-decoration-skip-ink",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "hanging-punctuation",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-rendering",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "block-ellipsis",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "reading-order",
        inherited: true,
        initial: "0",
    },
    PropertyMetadata {
        name: "writing-mode",
        inherited: true,
        initial: "horizontal-tb",
    },
    PropertyMetadata {
        name: "text-orientation",
        inherited: true,
        initial: "mixed",
    },
    PropertyMetadata {
        name: "math-shift",
        inherited: true,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-shadow",
        inherited: true,
        initial: "none",
    },
    PropertyMetadata {
        name: "interpolate-size",
        inherited: true,
        initial: "numeric-only",
    },
    PropertyMetadata {
        name: "speak",
        inherited: true,
        initial: "auto",
    },
    // NON-INHERITED PROPERTIES
    PropertyMetadata {
        name: "scrollbar-gutter",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "color-scheme",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "display",
        inherited: false,
        initial: "inline",
    },
    PropertyMetadata {
        name: "width",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "height",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "margin-top",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "margin-right",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "margin-bottom",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "margin-left",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-top",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-right",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-bottom",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-left",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-top-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-top-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-top-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "background-color",
        inherited: false,
        initial: "transparent",
    },
    PropertyMetadata {
        name: "position",
        inherited: false,
        initial: "static",
    },
    PropertyMetadata {
        name: "top",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "right",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "bottom",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "left",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "float",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "clear",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "overflow",
        inherited: false,
        initial: "visible",
    },
    PropertyMetadata {
        name: "line-clamp",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "z-index",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "box-sizing",
        inherited: false,
        initial: "content-box",
    },
    PropertyMetadata {
        name: "backdrop-filter",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "filter",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "mix-blend-mode",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "isolation",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "initial-letter",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "resize",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "backface-visibility",
        inherited: false,
        initial: "visible",
    },
    PropertyMetadata {
        name: "clip",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "clip-path",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "opacity",
        inherited: false,
        initial: "1",
    },
    PropertyMetadata {
        name: "margin-block-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "margin-block-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-block-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-block-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-right-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-bottom-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-left-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-right-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-bottom-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-left-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-right-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-bottom-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-left-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "background-image",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "background-repeat",
        inherited: false,
        initial: "repeat",
    },
    PropertyMetadata {
        name: "background-position",
        inherited: false,
        initial: "0% 0%",
    },
    PropertyMetadata {
        name: "background-size",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "background-attachment",
        inherited: false,
        initial: "scroll",
    },
    PropertyMetadata {
        name: "border-top-left-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-top-right-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-bottom-right-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-bottom-left-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "outline-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "outline-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "outline-color",
        inherited: false,
        initial: "invert",
    },
    PropertyMetadata {
        name: "min-width",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "min-height",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "max-width",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "max-height",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "flex-grow",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "flex-shrink",
        inherited: false,
        initial: "1",
    },
    PropertyMetadata {
        name: "flex-basis",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "flex-direction",
        inherited: false,
        initial: "row",
    },
    PropertyMetadata {
        name: "flex-wrap",
        inherited: false,
        initial: "nowrap",
    },
    PropertyMetadata {
        name: "justify-content",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "align-items",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "row-gap",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "column-gap",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "justify-items",
        inherited: false,
        initial: "legacy",
    },
    PropertyMetadata {
        name: "align-content",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "align-self",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "order",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "table-layout",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "vertical-align",
        inherited: false,
        initial: "baseline",
    },
    PropertyMetadata {
        name: "text-decoration-line",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-decoration-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "text-decoration-style",
        inherited: false,
        initial: "solid",
    },
    PropertyMetadata {
        name: "text-overflow",
        inherited: false,
        initial: "clip",
    },
    PropertyMetadata {
        name: "object-fit",
        inherited: false,
        initial: "fill",
    },
    PropertyMetadata {
        name: "object-position",
        inherited: false,
        initial: "50% 50%",
    },
    PropertyMetadata {
        name: "scroll-behavior",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "print-color-adjust",
        inherited: true,
        initial: "economy",
    },
    PropertyMetadata {
        name: "scroll-snap-type",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "scroll-snap-align",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "scroll-snap-stop",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "scroll-padding",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-margin",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-top",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-right",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-bottom",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-left",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-block",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-block-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-block-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-inline",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-inline-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-margin-inline-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "scroll-padding-top",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-right",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-bottom",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-left",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-block",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-block-start",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-block-end",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-inline",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-inline-start",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-padding-inline-end",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "overflow-clip-margin",
        inherited: false,
        initial: "0px",
    },
    PropertyMetadata {
        name: "inset-block",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inset-block-start",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inset-block-end",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inset-inline",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inset-inline-start",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inset-inline-end",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "overscroll-behavior",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "overscroll-behavior-x",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "overscroll-behavior-y",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "user-select",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "pointer-events",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "transition-duration",
        inherited: false,
        initial: "0s",
    },
    PropertyMetadata {
        name: "transition-property",
        inherited: false,
        initial: "all",
    },
    PropertyMetadata {
        name: "transition-timing-function",
        inherited: false,
        initial: "ease",
    },
    PropertyMetadata {
        name: "transition-delay",
        inherited: false,
        initial: "0s",
    },
    PropertyMetadata {
        name: "column-count",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "column-width",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "image-rendering",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "contain",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-decoration-thickness",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "text-underline-offset",
        inherited: true,
        initial: "auto",
    },
    PropertyMetadata {
        name: "counter-reset",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "counter-increment",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "orphans",
        inherited: true,
        initial: "2",
    },
    PropertyMetadata {
        name: "widows",
        inherited: true,
        initial: "2",
    },
    PropertyMetadata {
        name: "break-before",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "break-after",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "break-inside",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "box-decoration-break",
        inherited: false,
        initial: "slice",
    },
    PropertyMetadata {
        name: "mask-type",
        inherited: false,
        initial: "luminance",
    },
    PropertyMetadata {
        name: "field-sizing",
        inherited: false,
        initial: "fixed",
    },
    PropertyMetadata {
        name: "shape-outside",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "shape-margin",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "shape-image-threshold",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "anchor-name",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "view-transition-name",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "contain-intrinsic-width",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "contain-intrinsic-height",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "content-visibility",
        inherited: false,
        initial: "visible",
    },
    PropertyMetadata {
        name: "animation-timeline",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-timeline-name",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "scroll-timeline-axis",
        inherited: false,
        initial: "block",
    },
    PropertyMetadata {
        name: "text-box-trim",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "text-box-edge",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "-webkit-line-clamp",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "alignment-baseline",
        inherited: false,
        initial: "baseline",
    },
    PropertyMetadata {
        name: "baseline-shift",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "baseline-source",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "dominant-baseline",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "scroll-marker-group",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "reading-flow",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "position-area",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "position-try-fallbacks",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "position-try-order",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "position-visibility",
        inherited: false,
        initial: "anchors-visible",
    },
    PropertyMetadata {
        name: "timeline-scope",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "view-transition-class",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "overlay",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "anchor-scope",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "view-timeline-name",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "view-timeline-axis",
        inherited: false,
        initial: "block",
    },
    PropertyMetadata {
        name: "view-timeline-inset",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "container-name",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "container-type",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "aspect-ratio",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "unicode-bidi",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "grid-template-columns",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "grid-template-rows",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "grid-template-areas",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "grid-auto-columns",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "grid-auto-rows",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "grid-auto-flow",
        inherited: false,
        initial: "row",
    },
    PropertyMetadata {
        name: "box-shadow",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "position-anchor",
        inherited: false,
        initial: "implicit",
    },
    PropertyMetadata {
        name: "contain-intrinsic-block-size",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "contain-intrinsic-inline-size",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "block-size",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "inline-size",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "min-block-size",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "min-inline-size",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "max-block-size",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "max-inline-size",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "margin-inline-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "margin-inline-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-inline-start",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "padding-inline-end",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-block-start-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-block-start-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-block-start-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-block-end-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-block-end-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-block-end-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-inline-start-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-inline-start-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-inline-start-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-inline-end-width",
        inherited: false,
        initial: "medium",
    },
    PropertyMetadata {
        name: "border-inline-end-style",
        inherited: false,
        initial: "none",
    },
    PropertyMetadata {
        name: "border-inline-end-color",
        inherited: false,
        initial: "currentcolor",
    },
    PropertyMetadata {
        name: "border-start-start-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-start-end-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-end-start-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "border-end-end-radius",
        inherited: false,
        initial: "0",
    },
    PropertyMetadata {
        name: "speak-as",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "text-spacing",
        inherited: false,
        initial: "normal",
    },
    PropertyMetadata {
        name: "line-fit-edge",
        inherited: false,
        initial: "leading",
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
        name: "border",
        longhands: &["border-width", "border-style", "border-color"],
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
        name: "gap",
        longhands: &["row-gap", "column-gap"],
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
    SHORTHAND_EXPANSIONS
        .iter()
        .find(|sh| sh.name.eq_ignore_ascii_case(name))
        .map(|sh| sh.longhands)
}

/// Looks up the metadata for a CSS property by name.
///
/// This lookup is case-insensitive.
pub fn lookup(name: &str) -> Option<&'static PropertyMetadata> {
    PROPERTY_METADATA
        .iter()
        .find(|prop| prop.name.eq_ignore_ascii_case(name))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
        assert_eq!(initial_value("text-wrap"), Some("wrap"));
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
}
