//! Screen JS object implementation.
//!
//! This module defines the `Screen` object which provides information about the
//! screen of the device on which the application is running.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsNativeError, JsObject, JsString, JsValue, NativeFunction};

/// Builds a generic `NotSupportedError`-like object used as a fallback when the
/// `DOMException` constructor is unavailable. Per web API conventions the value a
/// promise rejects with should be an object exposing `name`/`message`, not a bare string.
fn not_supported_error_object(context: &mut Context) -> JsValue {
    let error_object = ObjectInitializer::new(context)
        .property(
            JsString::from("name"),
            JsString::from("NotSupportedError"),
            Attribute::all(),
        )
        .property(
            JsString::from("message"),
            JsString::from("lock() is not supported on this device."),
            Attribute::all(),
        )
        .build();
    JsValue::from(error_object)
}

/// Native implementation of `screen.orientation.lock()`.
fn screen_orientation_lock(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    // TODO(spec): real orientation locking is not implemented in this engine.
    let dom_exception_constructor = context
        .global_object()
        .get(JsString::from("DOMException"), context);

    let error_val = if let Some(constructor_obj) = dom_exception_constructor
        .ok()
        .as_ref()
        .and_then(|val| val.as_object())
    {
        let args = [
            JsValue::from(JsString::from("lock() is not supported on this device.")),
            JsValue::from(JsString::from("NotSupportedError")),
        ];
        if let Ok(exception_obj) = constructor_obj.construct(&args, None, context) {
            JsValue::from(exception_obj)
        } else {
            not_supported_error_object(context)
        }
    } else {
        not_supported_error_object(context)
    };

    let promise_constructor = context
        .global_object()
        .get(JsString::from("Promise"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise constructor not found"))
        })?;

    let reject_method = promise_constructor
        .get(JsString::from("reject"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise.reject not found"))
        })?;

    let promise = reject_method.call(&JsValue::from(promise_constructor), &[error_val], context)?;
    Ok(promise)
}

/// Native implementation of `screen.orientation.unlock()`.
fn screen_orientation_unlock(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    // TODO(spec): real orientation locking/unlocking is not implemented in this engine.
    Ok(JsValue::undefined())
}

/// Creates the standard `screen` object with the required read-only properties per the CSSOM View spec.
///
/// # Required properties:
/// - `width` (returns 1280)
/// - `height` (returns 720)
/// - `availWidth` (returns 1280)
/// - `availHeight` (returns 720)
/// - `colorDepth` (returns 24)
/// - `pixelDepth` (returns 24)
/// - `availLeft` (returns 0)
/// - `availTop` (returns 0)
/// - `left` (returns 0)
/// - `top` (returns 0)
/// - `orientation` (returns a ScreenOrientation-like object)
pub fn create_screen(context: &mut Context) -> JsObject {
    let ro = Attribute::ENUMERABLE | Attribute::CONFIGURABLE;

    let orientation = ObjectInitializer::new(context)
        .property(
            JsString::from("type"),
            JsString::from("landscape-primary"),
            ro,
        )
        .property(JsString::from("angle"), 0, ro)
        .property(
            JsString::from("onchange"),
            JsValue::null(),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(screen_orientation_lock),
            JsString::from("lock"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(screen_orientation_unlock),
            JsString::from("unlock"),
            0,
        )
        .build();

    ObjectInitializer::new(context)
        .property(JsString::from("width"), 1280, Attribute::all())
        .property(JsString::from("height"), 720, Attribute::all())
        .property(JsString::from("availWidth"), 1280, Attribute::all())
        .property(JsString::from("availHeight"), 720, Attribute::all())
        .property(JsString::from("colorDepth"), 24, Attribute::all())
        .property(JsString::from("pixelDepth"), 24, Attribute::all())
        .property(JsString::from("availLeft"), 0, Attribute::all())
        .property(JsString::from("availTop"), 0, Attribute::all())
        .property(JsString::from("left"), 0, Attribute::all())
        .property(JsString::from("top"), 0, Attribute::all())
        .property(JsString::from("orientation"), orientation, ro)
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
            screen.pixelDepth === 24 &&
            screen.availLeft === 0 &&
            screen.availTop === 0 &&
            screen.left === 0 &&
            screen.top === 0
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_screen_orientation_properties() {
        let mut context = Context::default();
        let screen = create_screen(&mut context);
        let _ =
            context.register_global_property(JsString::from("screen"), screen, Attribute::all());

        let source = Source::from_bytes(
            r#"
            screen.orientation.type === "landscape-primary" &&
            screen.orientation.angle === 0 &&
            screen.orientation.onchange === null &&
            (screen.orientation.lock("landscape") instanceof Promise) &&
            screen.orientation.unlock() === undefined
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));

        // Test assignability of onchange
        let source_assign = Source::from_bytes(
            r#"
            screen.orientation.onchange = () => {};
            typeof screen.orientation.onchange === "function"
            "#,
        );
        let res_assign = context.eval(source_assign).unwrap();
        assert_eq!(res_assign.as_boolean(), Some(true));
    }
}
