//! Navigator JS object implementation.
//!
//! This module defines the `Navigator` object which provides information about the
//! application/browser and the environment.

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsNativeError, JsObject, JsString, JsValue, NativeFunction};

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

/// Native implementation of `navigator.sendBeacon()`.
fn navigator_send_beacon(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let url_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: sendBeacon requires at least 1 argument"),
        )
    })?;
    let url_str = url_val
        .to_string(context)?
        .to_std_string()
        .unwrap_or_default();

    // Get the global document location to resolve relative URLs
    let global = context.global_object().clone();
    let base_url = if let Ok(doc_loc) = global.get(JsString::from("__document_location__"), context)
    {
        if let Some(doc_loc_obj) = doc_loc.as_object() {
            if let Ok(href_val) = doc_loc_obj.get(JsString::from("href"), context) {
                let href_str = href_val
                    .to_string(context)?
                    .to_std_string()
                    .unwrap_or_default();
                if !href_str.is_empty() {
                    crate::url::Url::parse(&href_str).ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let base_url = base_url.unwrap_or_else(|| {
        crate::url::Url::parse("http://localhost/").unwrap_or_else(|_| crate::url::Url {
            scheme: "http".to_string(),
            host: Some("localhost".to_string()),
            port: None,
            path: "/".to_string(),
            query: None,
            fragment: None,
        })
    });

    let is_valid = crate::url::Url::parse_with_base(&url_str, &base_url).is_ok()
        || crate::url::Url::parse(&url_str).is_ok();

    if !is_valid {
        return Err(JsError::from(
            JsNativeError::typ().with_message("TypeError: Invalid URL"),
        ));
    }

    Ok(JsValue::from(true))
}

/// Native implementation of `navigator.userAgentData.getHighEntropyValues()`.
fn navigator_user_agent_data_get_high_entropy_values(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    use boa_engine::object::builtins::JsArray;

    let brands_array = JsArray::from_iter(
        vec![
            ObjectInitializer::new(context)
                .property(
                    JsString::from("brand"),
                    JsString::from("underrated"),
                    Attribute::all(),
                )
                .property(
                    JsString::from("version"),
                    JsString::from("1.0"),
                    Attribute::all(),
                )
                .build()
                .into(),
        ],
        context,
    );

    let resolved_val = ObjectInitializer::new(context)
        .property(JsString::from("brands"), brands_array, Attribute::all())
        .property(JsString::from("mobile"), false, Attribute::all())
        .property(
            JsString::from("platform"),
            JsString::from(platform()),
            Attribute::all(),
        )
        .property(
            JsString::from("architecture"),
            JsString::from("x86_64"),
            Attribute::all(),
        )
        .property(
            JsString::from("model"),
            JsString::from(""),
            Attribute::all(),
        )
        .property(
            JsString::from("platformVersion"),
            JsString::from("1.0"),
            Attribute::all(),
        )
        .property(
            JsString::from("uaFullVersion"),
            JsString::from(user_agent()),
            Attribute::all(),
        )
        .build();

    let promise_constructor = context
        .global_object()
        .get(JsString::from("Promise"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise constructor not found"))
        })?
        .clone();

    let resolve_method = promise_constructor
        .get(JsString::from("resolve"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise.resolve not found"))
        })?
        .clone();

    let promise = resolve_method.call(
        &JsValue::from(promise_constructor),
        &[resolved_val.into()],
        context,
    )?;
    Ok(promise)
}

/// Native implementation of `navigator.geolocation.getCurrentPosition()`.
fn geolocation_get_current_position(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(success_callback) = args.first()
        && let Some(callback_fn) = success_callback.as_callable()
    {
        let coords = ObjectInitializer::new(context)
            .property(JsString::from("latitude"), 37.7749, Attribute::all())
            .property(JsString::from("longitude"), -122.4194, Attribute::all())
            .property(
                JsString::from("altitude"),
                JsValue::null(),
                Attribute::all(),
            )
            .property(JsString::from("accuracy"), 10.0, Attribute::all())
            .property(
                JsString::from("altitudeAccuracy"),
                JsValue::null(),
                Attribute::all(),
            )
            .property(JsString::from("heading"), JsValue::null(), Attribute::all())
            .property(JsString::from("speed"), JsValue::null(), Attribute::all())
            .build();

        let position = ObjectInitializer::new(context)
            .property(JsString::from("coords"), coords, Attribute::all())
            .property(
                JsString::from("timestamp"),
                JsValue::from(1000),
                Attribute::all(),
            )
            .build();

        callback_fn.call(&JsValue::undefined(), &[position.into()], context)?;
    }
    Ok(JsValue::undefined())
}

/// Native implementation of `navigator.geolocation.watchPosition()`.
fn geolocation_watch_position(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    if let Some(success_callback) = args.first()
        && let Some(callback_fn) = success_callback.as_callable()
    {
        let coords = ObjectInitializer::new(context)
            .property(JsString::from("latitude"), 37.7749, Attribute::all())
            .property(JsString::from("longitude"), -122.4194, Attribute::all())
            .property(
                JsString::from("altitude"),
                JsValue::null(),
                Attribute::all(),
            )
            .property(JsString::from("accuracy"), 10.0, Attribute::all())
            .property(
                JsString::from("altitudeAccuracy"),
                JsValue::null(),
                Attribute::all(),
            )
            .property(JsString::from("heading"), JsValue::null(), Attribute::all())
            .property(JsString::from("speed"), JsValue::null(), Attribute::all())
            .build();

        let position = ObjectInitializer::new(context)
            .property(JsString::from("coords"), coords, Attribute::all())
            .property(
                JsString::from("timestamp"),
                JsValue::from(1000),
                Attribute::all(),
            )
            .build();

        callback_fn.call(&JsValue::undefined(), &[position.into()], context)?;
    }
    Ok(JsValue::from(1))
}

/// Native implementation of `navigator.geolocation.clearWatch()`.
fn geolocation_clear_watch(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::undefined())
}

/// Native implementation of `navigator.clipboard.readText()`.
fn clipboard_read_text(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let promise_constructor = context
        .global_object()
        .get(JsString::from("Promise"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise constructor not found"))
        })?
        .clone();

    let resolve_method = promise_constructor
        .get(JsString::from("resolve"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise.resolve not found"))
        })?
        .clone();

    let promise = resolve_method.call(
        &JsValue::from(promise_constructor),
        &[JsValue::from(JsString::from(""))],
        context,
    )?;
    Ok(promise)
}

/// Native implementation of `navigator.clipboard.writeText()`.
fn clipboard_write_text(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let promise_constructor = context
        .global_object()
        .get(JsString::from("Promise"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise constructor not found"))
        })?
        .clone();

    let resolve_method = promise_constructor
        .get(JsString::from("resolve"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise.resolve not found"))
        })?
        .clone();

    let promise = resolve_method.call(
        &JsValue::from(promise_constructor),
        &[JsValue::undefined()],
        context,
    )?;
    Ok(promise)
}

/// Native implementation of `navigator.serviceWorker.register()`.
fn service_worker_register(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let promise_constructor = context
        .global_object()
        .get(JsString::from("Promise"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise constructor not found"))
        })?
        .clone();

    let resolve_method = promise_constructor
        .get(JsString::from("resolve"), context)?
        .as_object()
        .ok_or_else(|| {
            JsError::from(JsNativeError::typ().with_message("Promise.resolve not found"))
        })?
        .clone();

    let registration = ObjectInitializer::new(context).build();

    let promise = resolve_method.call(
        &JsValue::from(promise_constructor),
        &[registration.into()],
        context,
    )?;
    Ok(promise)
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

    let plugins_array = JsArray::from_iter(std::iter::empty(), context);
    let mime_types_array = JsArray::from_iter(std::iter::empty(), context);

    let brands_array = JsArray::from_iter(
        vec![
            ObjectInitializer::new(context)
                .property(
                    JsString::from("brand"),
                    JsString::from("underrated"),
                    Attribute::all(),
                )
                .property(
                    JsString::from("version"),
                    JsString::from("1.0"),
                    Attribute::all(),
                )
                .build()
                .into(),
        ],
        context,
    );

    let user_agent_data = ObjectInitializer::new(context)
        .property(JsString::from("brands"), brands_array, Attribute::all())
        .property(JsString::from("mobile"), false, Attribute::all())
        .property(
            JsString::from("platform"),
            JsString::from(platform()),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(navigator_user_agent_data_get_high_entropy_values),
            JsString::from("getHighEntropyValues"),
            1,
        )
        .build();

    let geolocation = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(geolocation_get_current_position),
            JsString::from("getCurrentPosition"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(geolocation_watch_position),
            JsString::from("watchPosition"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(geolocation_clear_watch),
            JsString::from("clearWatch"),
            1,
        )
        .build();

    let clipboard = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(clipboard_read_text),
            JsString::from("readText"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(clipboard_write_text),
            JsString::from("writeText"),
            1,
        )
        .build();

    let service_worker = ObjectInitializer::new(context)
        .property(
            JsString::from("controller"),
            JsValue::null(),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(service_worker_register),
            JsString::from("register"),
            1,
        )
        .build();

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
        .property(
            JsString::from("userAgentData"),
            user_agent_data,
            Attribute::all(),
        )
        .property(JsString::from("geolocation"), geolocation, Attribute::all())
        .property(JsString::from("clipboard"), clipboard, Attribute::all())
        .property(
            JsString::from("serviceWorker"),
            service_worker,
            Attribute::all(),
        )
        .property(JsString::from("plugins"), plugins_array, Attribute::all())
        .property(
            JsString::from("mimeTypes"),
            mime_types_array,
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
        .function(
            NativeFunction::from_fn_ptr(navigator_send_beacon),
            JsString::from("sendBeacon"),
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
            navigator.vibrate(100) === false &&
            typeof navigator.sendBeacon === 'function' &&
            navigator.sendBeacon('/gen_204') === true &&
            typeof navigator.userAgentData === 'object' &&
            Array.isArray(navigator.userAgentData.brands) &&
            navigator.userAgentData.mobile === false &&
            navigator.userAgentData.platform === 'Rust' &&
            typeof navigator.userAgentData.getHighEntropyValues === 'function' &&
            typeof navigator.geolocation === 'object' &&
            typeof navigator.geolocation.getCurrentPosition === 'function' &&
            typeof navigator.clipboard === 'object' &&
            typeof navigator.clipboard.readText === 'function' &&
            typeof navigator.serviceWorker === 'object' &&
            typeof navigator.serviceWorker.register === 'function' &&
            Array.isArray(navigator.plugins) &&
            navigator.plugins.length === 0 &&
            Array.isArray(navigator.mimeTypes) &&
            navigator.mimeTypes.length === 0
            "#,
        );
        let res = context.eval(source).unwrap();
        assert_eq!(res.as_boolean(), Some(true));
    }

    #[test]
    fn test_navigator_async_and_beacon_details() {
        let mut context = Context::default();
        let navigator = create_navigator(&mut context);
        let _ = context.register_global_property(
            JsString::from("navigator"),
            navigator,
            Attribute::all(),
        );

        let no_args_beacon_source = Source::from_bytes(
            r#"
            let threw = false;
            try {
                navigator.sendBeacon();
            } catch (e) {
                threw = e.message.includes("requires at least 1 argument");
            }
            threw;
            "#,
        );
        let res_no_args = context.eval(no_args_beacon_source).unwrap();
        assert_eq!(res_no_args.as_boolean(), Some(true));

        let async_source = Source::from_bytes(
            r#"
            let uadata_resolved = null;
            let clipboard_resolved = null;
            let sw_resolved = null;

            navigator.userAgentData.getHighEntropyValues(["architecture"])
                .then(res => { uadata_resolved = res; });

            navigator.clipboard.readText()
                .then(res => { clipboard_resolved = res; });

            navigator.serviceWorker.register("/sw.js")
                .then(res => { sw_resolved = res; });
            "#,
        );
        context.eval(async_source).unwrap();
        let _ = context.run_jobs();

        let check_source = Source::from_bytes(
            r#"
            uadata_resolved !== null &&
            uadata_resolved.architecture === 'x86_64' &&
            uadata_resolved.platform === 'Rust' &&
            clipboard_resolved === '' &&
            sw_resolved !== null
            "#,
        );
        let res_check = context.eval(check_source).unwrap();
        assert_eq!(res_check.as_boolean(), Some(true));
    }
}
