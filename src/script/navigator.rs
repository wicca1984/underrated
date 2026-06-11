//! Navigator JS object implementation.
//!
//! This module defines the `Navigator` object which provides information about the
//! application/browser and the environment.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsObject, JsString};

/// Creates the standard `navigator` object with the required properties.
///
/// Under the W3C and HTML specifications, `navigator` exposes client information.
///
/// # Required properties:
/// - `userAgent` (returns "underrated/1.0")
/// - `platform` (returns "Rust")
/// - `language` (returns "en-US")
pub fn create_navigator(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .property(
            JsString::from("userAgent"),
            JsString::from("underrated/1.0"),
            Attribute::all(),
        )
        .property(
            JsString::from("platform"),
            JsString::from("Rust"),
            Attribute::all(),
        )
        .property(
            JsString::from("language"),
            JsString::from("en-US"),
            Attribute::all(),
        )
        .build()
}
