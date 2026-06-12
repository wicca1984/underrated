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
];

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
    }

    #[test]
    fn test_initial_value() {
        assert_eq!(initial_value("display"), Some("inline"));
        assert_eq!(initial_value("width"), Some("auto"));
        assert_eq!(initial_value("border-top-color"), Some("currentcolor"));
        assert_eq!(initial_value("not-a-real-prop"), None);
    }

    #[test]
    fn test_lookup() {
        let meta = lookup("FONT-SIZE");
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.name, "font-size");
        assert!(meta.inherited);
        assert_eq!(meta.initial, "medium");
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
}
