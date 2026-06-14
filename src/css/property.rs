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
    // NON-INHERITED PROPERTIES
    PropertyMetadata {
        name: "scrollbar-gutter",
        inherited: false,
        initial: "auto",
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
        name: "text-decoration",
        longhands: &[
            "text-decoration-line",
            "text-decoration-style",
            "text-decoration-color",
            "text-decoration-thickness",
        ],
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
}
