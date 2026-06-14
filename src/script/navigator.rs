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

/// Helper to construct a resolved Promise.
fn resolve_promise(val: JsValue, context: &mut Context) -> Result<JsValue, JsError> {
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

    let promise = resolve_method.call(&JsValue::from(promise_constructor), &[val], context)?;
    Ok(promise)
}

/// Native implementation of `navigator.storage.persisted()`.
fn storage_persisted(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    resolve_promise(JsValue::from(false), context)
}

/// Native implementation of `navigator.storage.persist()`.
fn storage_persist(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    resolve_promise(JsValue::from(false), context)
}

/// Native implementation of `navigator.storage.estimate()`.
fn storage_estimate(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let estimate_val = ObjectInitializer::new(context)
        .property(JsString::from("usage"), 0, Attribute::all())
        .property(
            JsString::from("quota"),
            10_737_418_240_u64,
            Attribute::all(),
        ) // 10 GB
        .build();
    resolve_promise(estimate_val.into(), context)
}

/// Native implementation of `navigator.permissions.query()`.
fn permissions_query(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let desc_val = args.first().ok_or_else(|| {
        JsError::from(
            JsNativeError::typ().with_message("TypeError: query requires at least 1 argument"),
        )
    })?;

    let name = if let Some(desc_obj) = desc_val.as_object() {
        desc_obj
            .get(JsString::from("name"), context)?
            .to_string(context)?
    } else {
        return Err(JsError::from(JsNativeError::typ().with_message(
            "TypeError: permission descriptor must be an object",
        )));
    };

    // Construct a PermissionStatus-like object
    let status = ObjectInitializer::new(context)
        .property(
            JsString::from("state"),
            JsString::from("prompt"),
            Attribute::all(),
        )
        .property(JsString::from("name"), name, Attribute::all())
        .property(
            JsString::from("onchange"),
            JsValue::null(),
            Attribute::all(),
        )
        .build();

    resolve_promise(status.into(), context)
}

/// Native implementation of `navigator.mediaCapabilities.decodingInfo()`.
fn media_capabilities_decoding_info(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let info = ObjectInitializer::new(context)
        .property(JsString::from("supported"), true, Attribute::all())
        .property(JsString::from("smooth"), true, Attribute::all())
        .property(JsString::from("powerEfficient"), true, Attribute::all())
        .build();
    resolve_promise(info.into(), context)
}

/// Native implementation of `navigator.mediaCapabilities.encodingInfo()`.
fn media_capabilities_encoding_info(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let info = ObjectInitializer::new(context)
        .property(JsString::from("supported"), true, Attribute::all())
        .property(JsString::from("smooth"), true, Attribute::all())
        .property(JsString::from("powerEfficient"), true, Attribute::all())
        .build();
    resolve_promise(info.into(), context)
}

/// Native implementation of `navigator.share()`.
fn navigator_share(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    resolve_promise(JsValue::undefined(), context)
}

/// Native implementation of `navigator.canShare()`.
fn navigator_can_share(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::from(true))
}

/// Native implementation of `navigator.mediaDevices.enumerateDevices()`.
fn media_devices_enumerate_devices(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    use boa_engine::object::builtins::JsArray;
    let empty_array = JsArray::from_iter(std::iter::empty(), context);
    resolve_promise(empty_array.into(), context)
}

/// Native implementation of `navigator.mediaDevices.getSupportedConstraints()`.
fn media_devices_get_supported_constraints(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let constraints = ObjectInitializer::new(context)
        .property(JsString::from("deviceId"), true, Attribute::all())
        .property(JsString::from("groupId"), true, Attribute::all())
        .property(JsString::from("facingMode"), true, Attribute::all())
        .property(JsString::from("frameRate"), true, Attribute::all())
        .property(JsString::from("width"), true, Attribute::all())
        .property(JsString::from("height"), true, Attribute::all())
        .property(JsString::from("aspectRatio"), true, Attribute::all())
        .build();
    Ok(constraints.into())
}

/// Native implementation of `navigator.mediaDevices.getUserMedia()`.
fn media_devices_get_user_media(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let media_stream = ObjectInitializer::new(context)
        .property(
            JsString::from("id"),
            JsString::from("dummy-stream-id"),
            Attribute::all(),
        )
        .property(JsString::from("active"), true, Attribute::all())
        .build();
    resolve_promise(media_stream.into(), context)
}

/// Native implementation of `navigator.mediaDevices.getDisplayMedia()`.
fn media_devices_get_display_media(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let media_stream = ObjectInitializer::new(context)
        .property(
            JsString::from("id"),
            JsString::from("dummy-display-stream-id"),
            Attribute::all(),
        )
        .property(JsString::from("active"), true, Attribute::all())
        .build();
    resolve_promise(media_stream.into(), context)
}

/// Native implementation of `navigator.locks.query()`.
fn locks_query(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    use boa_engine::object::builtins::JsArray;
    let pending = JsArray::from_iter(std::iter::empty(), context);
    let held = JsArray::from_iter(std::iter::empty(), context);
    let lock_manager_snapshot = ObjectInitializer::new(context)
        .property(JsString::from("pending"), pending, Attribute::all())
        .property(JsString::from("held"), held, Attribute::all())
        .build();
    resolve_promise(lock_manager_snapshot.into(), context)
}

/// Native implementation of `navigator.locks.request()`.
fn locks_request(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let callback = match args.get(1) {
        Some(val) => val.as_callable(),
        None => match args.get(2) {
            Some(val) => val.as_callable(),
            None => None,
        },
    };

    let name = args
        .first()
        .and_then(|v| v.to_string(context).ok())
        .unwrap_or_else(|| JsString::from("default"));

    if let Some(callback_fn) = callback {
        let lock_obj = ObjectInitializer::new(context)
            .property(JsString::from("name"), name, Attribute::all())
            .property(
                JsString::from("mode"),
                JsString::from("exclusive"),
                Attribute::all(),
            )
            .build();
        let callback_res = callback_fn.call(&JsValue::undefined(), &[lock_obj.into()], context)?;
        resolve_promise(callback_res, context)
    } else {
        resolve_promise(JsValue::undefined(), context)
    }
}

/// Native implementation of `navigator.wakeLock.request()`.
fn wake_lock_request(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    let lock_type = args
        .first()
        .and_then(|v| v.to_string(context).ok())
        .unwrap_or_else(|| JsString::from("screen"));

    let sentinel = ObjectInitializer::new(context)
        .property(JsString::from("released"), false, Attribute::all())
        .property(JsString::from("type"), lock_type, Attribute::all())
        .property(
            JsString::from("onrelease"),
            JsValue::null(),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(wake_lock_sentinel_release),
            JsString::from("release"),
            0,
        )
        .build();

    resolve_promise(sentinel.into(), context)
}

/// Native implementation of `wakeLockSentinel.release()`.
fn wake_lock_sentinel_release(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    resolve_promise(JsValue::undefined(), context)
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

    let storage = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(storage_persisted),
            JsString::from("persisted"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_persist),
            JsString::from("persist"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(storage_estimate),
            JsString::from("estimate"),
            0,
        )
        .build();

    let permissions = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(permissions_query),
            JsString::from("query"),
            1,
        )
        .build();

    let user_activation = ObjectInitializer::new(context)
        .property(JsString::from("isActive"), false, Attribute::all())
        .property(JsString::from("hasBeenActive"), false, Attribute::all())
        .build();

    let connection = ObjectInitializer::new(context)
        .property(JsString::from("downlink"), 10.0, Attribute::all())
        .property(
            JsString::from("effectiveType"),
            JsString::from("4g"),
            Attribute::all(),
        )
        .property(JsString::from("rtt"), 50, Attribute::all())
        .property(JsString::from("saveData"), false, Attribute::all())
        .property(
            JsString::from("onchange"),
            JsValue::null(),
            Attribute::all(),
        )
        .build();

    let media_capabilities = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(media_capabilities_decoding_info),
            JsString::from("decodingInfo"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(media_capabilities_encoding_info),
            JsString::from("encodingInfo"),
            1,
        )
        .build();

    let media_devices = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(media_devices_enumerate_devices),
            JsString::from("enumerateDevices"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(media_devices_get_supported_constraints),
            JsString::from("getSupportedConstraints"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(media_devices_get_user_media),
            JsString::from("getUserMedia"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(media_devices_get_display_media),
            JsString::from("getDisplayMedia"),
            1,
        )
        .build();

    let locks = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(locks_query),
            JsString::from("query"),
            0,
        )
        .function(
            NativeFunction::from_fn_ptr(locks_request),
            JsString::from("request"),
            2,
        )
        .build();

    let wake_lock = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(wake_lock_request),
            JsString::from("request"),
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
        .property(JsString::from("storage"), storage, Attribute::all())
        .property(JsString::from("permissions"), permissions, Attribute::all())
        .property(
            JsString::from("userActivation"),
            user_activation,
            Attribute::all(),
        )
        .property(JsString::from("connection"), connection, Attribute::all())
        .property(
            JsString::from("mediaCapabilities"),
            media_capabilities,
            Attribute::all(),
        )
        .property(
            JsString::from("mediaDevices"),
            media_devices,
            Attribute::all(),
        )
        .property(JsString::from("locks"), locks, Attribute::all())
        .property(JsString::from("wakeLock"), wake_lock, Attribute::all())
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
        .function(
            NativeFunction::from_fn_ptr(navigator_share),
            JsString::from("share"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(navigator_can_share),
            JsString::from("canShare"),
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

    #[test]
    fn test_navigator_completeness_and_async_apis() {
        let mut context = Context::default();
        let navigator = create_navigator(&mut context);
        let _ = context.register_global_property(
            JsString::from("navigator"),
            navigator,
            Attribute::all(),
        );

        let sync_checks = Source::from_bytes(
            r#"
            typeof navigator.storage === 'object' &&
            typeof navigator.permissions === 'object' &&
            typeof navigator.userActivation === 'object' &&
            typeof navigator.connection === 'object' &&
            typeof navigator.mediaCapabilities === 'object' &&
            navigator.connection.downlink === 10 &&
            navigator.connection.effectiveType === '4g' &&
            navigator.connection.rtt === 50 &&
            navigator.connection.saveData === false &&
            navigator.connection.onchange === null &&
            navigator.userActivation.isActive === false &&
            navigator.userActivation.hasBeenActive === false
            "#,
        );
        let res_sync = context.eval(sync_checks).unwrap();
        assert_eq!(res_sync.as_boolean(), Some(true));

        let async_checks = Source::from_bytes(
            r#"
            let storage_estimate = null;
            let storage_persisted = null;
            let storage_persist = null;
            let permission_status = null;
            let media_decoding = null;
            let media_encoding = null;

            navigator.storage.estimate()
                .then(res => { storage_estimate = res; });
            navigator.storage.persisted()
                .then(res => { storage_persisted = res; });
            navigator.storage.persist()
                .then(res => { storage_persist = res; });
            navigator.permissions.query({ name: 'geolocation' })
                .then(res => { permission_status = res; });
            navigator.mediaCapabilities.decodingInfo({})
                .then(res => { media_decoding = res; });
            navigator.mediaCapabilities.encodingInfo({})
                .then(res => { media_encoding = res; });
            "#,
        );
        context.eval(async_checks).unwrap();
        let _ = context.run_jobs();

        let check_async_results = Source::from_bytes(
            r#"
            storage_estimate !== null &&
            storage_estimate.usage === 0 &&
            storage_estimate.quota === 10 * 1024 * 1024 * 1024 &&
            storage_persisted === false &&
            storage_persist === false &&
            permission_status !== null &&
            permission_status.state === 'prompt' &&
            permission_status.name === 'geolocation' &&
            permission_status.onchange === null &&
            media_decoding !== null &&
            media_decoding.supported === true &&
            media_decoding.smooth === true &&
            media_decoding.powerEfficient === true &&
            media_encoding !== null &&
            media_encoding.supported === true &&
            media_encoding.smooth === true &&
            media_encoding.powerEfficient === true
            "#,
        );
        let res_async = context.eval(check_async_results).unwrap();
        assert_eq!(res_async.as_boolean(), Some(true));
    }

    #[test]
    fn test_navigator_added_apis() {
        let mut context = Context::default();
        let navigator = create_navigator(&mut context);
        let _ = context.register_global_property(
            JsString::from("navigator"),
            navigator,
            Attribute::all(),
        );

        let sync_checks = Source::from_bytes(
            r#"
            typeof navigator.share === 'function' &&
            typeof navigator.canShare === 'function' &&
            navigator.canShare({}) === true &&
            typeof navigator.mediaDevices === 'object' &&
            typeof navigator.mediaDevices.enumerateDevices === 'function' &&
            typeof navigator.mediaDevices.getSupportedConstraints === 'function' &&
            typeof navigator.mediaDevices.getUserMedia === 'function' &&
            typeof navigator.mediaDevices.getDisplayMedia === 'function' &&
            typeof navigator.locks === 'object' &&
            typeof navigator.locks.query === 'function' &&
            typeof navigator.locks.request === 'function' &&
            typeof navigator.wakeLock === 'object' &&
            typeof navigator.wakeLock.request === 'function'
            "#,
        );
        let res_sync = context.eval(sync_checks).unwrap();
        assert_eq!(res_sync.as_boolean(), Some(true));

        let async_checks = Source::from_bytes(
            r#"
            let share_resolved = false;
            let devices = null;
            let constraints = null;
            let stream = null;
            let display_stream = null;
            let lock_query_res = null;
            let lock_request_called = false;
            let wake_lock_sentinel = null;

            navigator.share({ title: 'Test' })
                .then(() => { share_resolved = true; });

            navigator.mediaDevices.enumerateDevices()
                .then(res => { devices = res; });

            constraints = navigator.mediaDevices.getSupportedConstraints();

            navigator.mediaDevices.getUserMedia({ video: true })
                .then(res => { stream = res; });

            navigator.mediaDevices.getDisplayMedia({ video: true })
                .then(res => { display_stream = res; });

            navigator.locks.query()
                .then(res => { lock_query_res = res; });

            navigator.locks.request('my-lock', (lock) => {
                if (lock && lock.name === 'my-lock' && lock.mode === 'exclusive') {
                    lock_request_called = true;
                }
            });

            navigator.wakeLock.request('screen')
                .then(res => { wake_lock_sentinel = res; });
            "#,
        );
        context.eval(async_checks).unwrap();
        let _ = context.run_jobs();

        let check_async_results = Source::from_bytes(
            r#"
            share_resolved === true &&
            Array.isArray(devices) &&
            devices.length === 0 &&
            constraints !== null &&
            constraints.deviceId === true &&
            stream !== null &&
            stream.id === 'dummy-stream-id' &&
            stream.active === true &&
            display_stream !== null &&
            display_stream.id === 'dummy-display-stream-id' &&
            display_stream.active === true &&
            lock_query_res !== null &&
            Array.isArray(lock_query_res.pending) &&
            Array.isArray(lock_query_res.held) &&
            lock_request_called === true &&
            wake_lock_sentinel !== null &&
            wake_lock_sentinel.released === false &&
            wake_lock_sentinel.type === 'screen' &&
            typeof wake_lock_sentinel.release === 'function'
            "#,
        );
        let res_async = context.eval(check_async_results).unwrap();
        assert_eq!(res_async.as_boolean(), Some(true));
    }
}
