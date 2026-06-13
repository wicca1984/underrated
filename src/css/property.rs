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
    // NON-INHERITED PROPERTIES
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
        name: "column-count",
        inherited: false,
        initial: "auto",
    },
    PropertyMetadata {
        name: "column-width",
        inherited: false,
        initial: "auto",
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
        name: "margin",
        longhands: &["margin-top", "margin-right", "margin-bottom", "margin-left"],
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
        name: "border-width",
        longhands: &[
            "border-top-width",
            "border-right-width",
            "border-bottom-width",
            "border-left-width",
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
        name: "border-color",
        longhands: &[
            "border-top-color",
            "border-right-color",
            "border-bottom-color",
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
        name: "overflow",
        longhands: &["overflow-x", "overflow-y"],
    },
    ShorthandExpansion {
        name: "gap",
        longhands: &["row-gap", "column-gap"],
    },
    ShorthandExpansion {
        name: "inset",
        longhands: &["top", "right", "bottom", "left"],
    },
    ShorthandExpansion {
        name: "place-items",
        longhands: &["align-items", "justify-items"],
    },
    ShorthandExpansion {
        name: "place-content",
        longhands: &["align-content", "justify-content"],
    },
    ShorthandExpansion {
        name: "place-self",
        longhands: &["align-self", "justify-self"],
    },
    ShorthandExpansion {
        name: "columns",
        longhands: &["column-width", "column-count"],
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
        assert!(!is_inherited("max-width"));
    }

    #[test]
    fn test_initial_value() {
        assert_eq!(initial_value("display"), Some("inline"));
        assert_eq!(initial_value("width"), Some("auto"));
        assert_eq!(initial_value("border-top-color"), Some("currentcolor"));
        assert_eq!(initial_value("not-a-real-prop"), None);
        assert_eq!(initial_value("flex-shrink"), Some("1"));
        assert_eq!(initial_value("border-collapse"), Some("separate"));
        assert_eq!(initial_value("background-repeat"), Some("repeat"));
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

        let overflow = shorthand_longhands("OVERFLOW");
        assert!(overflow.is_some());
        assert_eq!(overflow.unwrap().len(), 2);

        assert_eq!(shorthand_longhands("color"), None);

        let radius = shorthand_longhands("border-radius");
        assert!(radius.is_some());
        let radius_slice = radius.unwrap();
        assert_eq!(radius_slice.len(), 4);
        assert_eq!(radius_slice[0], "border-top-left-radius");
    }

    #[test]
    fn test_shorthand_expansions_no_duplicates() {
        let mut names = HashSet::new();
        for sh in SHORTHAND_EXPANSIONS {
            assert_eq!(
                sh.name,
                sh.name.to_lowercase(),
                "Shorthand name '{}' must be lowercase",
                sh.name
            );
            assert!(
                names.insert(sh.name),
                "Duplicate shorthand name found: {}",
                sh.name
            );
        }
        assert_eq!(names.len(), SHORTHAND_EXPANSIONS.len());
    }
}
