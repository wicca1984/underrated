//! Navigator JS object implementation.
//!
//! This module defines the `Navigator` object which provides information about the
//! application/browser and the environment.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsObject, JsString, JsValue, NativeFunction};

/// Returns the user agent string.
pub fn user_agent() -> &'static str {
    "underrated/1.0"
}

/// Returns the platform name.
pub fn platform() -> &'static str {
    "Rust"
}

/// Returns the primary language.
pub fn language() -> &'static str {
    "en-US"
}

/// Returns the list of preferred languages.
pub fn languages() -> Vec<&'static str> {
    vec!["en-US"]
}

/// Returns the application name.
pub fn app_name() -> &'static str {
    "Netscape"
}

/// Returns the browser version.
pub fn app_version() -> &'static str {
    "5.0"
}

/// Returns the browser code name.
pub fn app_code_name() -> &'static str {
    "Mozilla"
}

/// Returns the product name.
pub fn product() -> &'static str {
    "Gecko"
}

/// Returns the product sub-version (build date).
pub fn product_sub() -> &'static str {
    "20030107"
}

/// Returns the vendor name.
pub fn vendor() -> &'static str {
    ""
}

/// Returns the vendor sub-version.
pub fn vendor_sub() -> &'static str {
    ""
}

/// Returns whether the browser is online.
pub fn on_line() -> bool {
    true
}

/// Returns whether cookies are enabled.
pub fn cookie_enabled() -> bool {
    true
}

/// Returns the number of logical processors.
pub fn hardware_concurrency() -> u32 {
    4
}

/// Returns the maximum number of simultaneous touch points.
pub fn max_touch_points() -> u32 {
    0
}

/// Returns the do-not-track preference.
pub fn do_not_track() -> Option<&'static str> {
    None
}

/// Returns whether PDF files can be viewed inline.
pub fn pdf_viewer_enabled() -> bool {
    true
}

/// Returns whether the browser is controlled by automation.
pub fn webdriver() -> bool {
    false
}

/// Returns the approximate amount of device memory (RAM) in gigabytes.
pub fn device_memory() -> f64 {
    8.0
}

/// Returns the operating system CPU architecture or version.
pub fn oscpu() -> &'static str {
    "Linux x86_64"
}

/// Native implementation of `navigator.javaEnabled()`.
fn navigator_java_enabled(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::from(false))
}

/// Native implementation of `navigator.vibrate()`.
fn navigator_vibrate(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::from(false))
}

/// Creates the standard `navigator` object with the required properties.
///
/// Under the W3C and HTML specifications, `navigator` exposes client information.
pub fn create_navigator(context: &mut Context) -> JsObject {
    use boa_engine::JsValue;
    use boa_engine::object::builtins::JsArray;

    let languages_array = JsArray::from_iter(
        languages()
            .into_iter()
            .map(|lang| JsValue::from(JsString::from(lang))),
        context,
    );

    let do_not_track_val = match do_not_track() {
        Some(val) => JsValue::from(JsString::from(val)),
        None => JsValue::null(),
    };

    ObjectInitializer::new(context)
        .property(
            JsString::from("userAgent"),
            JsString::from(user_agent()),
            Attribute::all(),
        )
        .property(
            JsString::from("platform"),
            JsString::from(platform()),
            Attribute::all(),
        )
        .property(
            JsString::from("language"),
            JsString::from(language()),
            Attribute::all(),
        )
        .property(
            JsString::from("languages"),
            languages_array,
            Attribute::all(),
        )
        .property(
            JsString::from("appName"),
            JsString::from(app_name()),
            Attribute::all(),
        )
        .property(
            JsString::from("appVersion"),
            JsString::from(app_version()),
            Attribute::all(),
        )
        .property(
            JsString::from("appCodeName"),
            JsString::from(app_code_name()),
            Attribute::all(),
        )
        .property(
            JsString::from("product"),
            JsString::from(product()),
            Attribute::all(),
        )
        .property(
            JsString::from("productSub"),
            JsString::from(product_sub()),
            Attribute::all(),
        )
        .property(
            JsString::from("vendor"),
            JsString::from(vendor()),
            Attribute::all(),
        )
        .property(
            JsString::from("vendorSub"),
            JsString::from(vendor_sub()),
            Attribute::all(),
        )
        .property(JsString::from("onLine"), on_line(), Attribute::all())
        .property(
            JsString::from("cookieEnabled"),
            cookie_enabled(),
            Attribute::all(),
        )
        .property(
            JsString::from("hardwareConcurrency"),
            hardware_concurrency(),
            Attribute::all(),
        )
        .property(
            JsString::from("maxTouchPoints"),
            max_touch_points(),
            Attribute::all(),
        )
        .property(
            JsString::from("doNotTrack"),
            do_not_track_val,
            Attribute::all(),
        )
        .property(
            JsString::from("pdfViewerEnabled"),
            pdf_viewer_enabled(),
            Attribute::all(),
        )
        .property(JsString::from("webdriver"), webdriver(), Attribute::all())
        .property(
            JsString::from("deviceMemory"),
            device_memory(),
            Attribute::all(),
        )
        .property(
            JsString::from("oscpu"),
            JsString::from(oscpu()),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(navigator_java_enabled),
            JsString::from("javaEnabled"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(navigator_vibrate),
            JsString::from("vibrate"),
            1,
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    #[test]
    fn test_navigator_rust_accessors() {
        assert_eq!(user_agent(), "underrated/1.0");
        assert_eq!(platform(), "Rust");
        assert_eq!(language(), "en-US");
        assert_eq!(languages(), vec!["en-US"]);
        assert_eq!(app_name(), "Netscape");
        assert_eq!(app_version(), "5.0");
        assert_eq!(app_code_name(), "Mozilla");
        assert_eq!(product(), "Gecko");
        assert_eq!(product_sub(), "20030107");
        assert_eq!(vendor(), "");
        assert_eq!(vendor_sub(), "");
        assert!(on_line());
        assert!(cookie_enabled());
        assert_eq!(hardware_concurrency(), 4);
        assert_eq!(max_touch_points(), 0);
        assert_eq!(do_not_track(), None);
        assert!(pdf_viewer_enabled());
        assert!(!webdriver());
        assert_eq!(device_memory(), 8.0);
        assert_eq!(oscpu(), "Linux x86_64");
    }

    #[test]
    fn test_navigator_js_properties() {
        let mut context = Context::default();
        let navigator = create_navigator(&mut context);
        let _ = context.register_global_property(
            JsString::from("navigator"),
            navigator,
            Attribute::all(),
        );

        let source = Source::from_bytes(
            r#"
            navigator.userAgent === 'underrated/1.0' &&
            navigator.platform === 'Rust' &&
            navigator.language === 'en-US' &&
            Array.isArray(navigator.languages) &&
            navigator.languages.length === 1 &&
            navigator.languages[0] === 'en-US' &&
            navigator.appName === 'Netscape' &&
            navigator.appVersion === '5.0' &&
            navigator.appCodeName === 'Mozilla' &&
            navigator.product === 'Gecko' &&
            navigator.productSub === '20030107' &&
            navigator.vendor === '' &&
            navigator.vendorSub === '' &&
            navigator.onLine === true &&
            navigator.cookieEnabled === true &&
            navigator.hardwareConcurrency === 4 &&
            navigator.maxTouchPoints === 0 &&
            navigator.doNotTrack === null &&
            navigator.pdfViewerEnabled === true &&
            navigator.webdriver === false &&
            navigator.deviceMemory === 8 &&
            navigator.oscpu === 'Linux x86_64' &&
            typeof navigator.javaEnabled === 'function' &&
            navigator.javaEnabled() === false &&
            typeof navigator.vibrate === 'function' &&
            navigator.vibrate(100) === false
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }
}
