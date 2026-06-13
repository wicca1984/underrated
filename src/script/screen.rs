//! Screen JS object implementation.
//!
//! This module defines the `Screen` object which provides information about the
//! screen of the device on which the application is running.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsObject, JsString};

/// Creates the standard `screen` object with the required read-only properties per the CSSOM View spec.
///
/// # Required properties:
/// - `width` (returns 1280)
/// - `height` (returns 720)
/// - `availWidth` (returns 1280)
/// - `availHeight` (returns 720)
/// - `colorDepth` (returns 24)
/// - `pixelDepth` (returns 24)
pub fn create_screen(context: &mut Context) -> JsObject {
    ObjectInitializer::new(context)
        .property(JsString::from("width"), 1280, Attribute::all())
        .property(JsString::from("height"), 720, Attribute::all())
        .property(JsString::from("availWidth"), 1280, Attribute::all())
        .property(JsString::from("availHeight"), 720, Attribute::all())
        .property(JsString::from("colorDepth"), 24, Attribute::all())
        .property(JsString::from("pixelDepth"), 24, Attribute::all())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_screen_properties() {
        let mut context = Context::default();
        let screen = create_screen(&mut context);
        let _ =
            context.register_global_property(JsString::from("screen"), screen, Attribute::all());

        let source = Source::from_bytes(
            r#"
            screen.width === 1280 &&
            screen.height === 720 &&
            screen.availWidth === 1280 &&
            screen.availHeight === 720 &&
            screen.colorDepth === 24 &&
            screen.pixelDepth === 24
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
