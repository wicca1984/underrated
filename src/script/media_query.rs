//! Implementation of the `window.matchMedia()` Web API.
//!
//! Spec: <https://drafts.csswg.org/cssom-view/#dom-window-matchmedia>

use boa_engine::object::ObjectInitializer;
use boa_engine::property::Attribute;
use boa_engine::{Context, JsError, JsString, JsValue, NativeFunction};

/// Native implementation of the `noop_listener` for `MediaQueryList`.
///
/// Accepts a callback or options and does nothing.
fn noop_listener(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> Result<JsValue, JsError> {
    Ok(JsValue::undefined())
}

/// Native implementation of `window.matchMedia(query)`.
///
/// Spec: <https://drafts.csswg.org/cssom-view/#dom-window-matchmedia>
pub fn match_media(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> Result<JsValue, JsError> {
    // 1. Get query string, default to empty string if not provided.
    let query_str = if let Some(arg) = args.first() {
        arg.to_string(context)?.to_std_string().unwrap_or_default()
    } else {
        String::new()
    };

    // 2. Evaluate query against current viewport width (default 1280 matching screen.width).
    // TODO(spec): Event firing on viewport change is out of scope. Use 1280 matching screen width.
    let matches = crate::css::media::media_matches(&query_str, 1280.0);

    // 3. Create MediaQueryList-like object.
    let mql = ObjectInitializer::new(context)
        .property(JsString::from("matches"), matches, Attribute::all())
        .property(
            JsString::from("media"),
            JsString::from(query_str),
            Attribute::all(),
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("addListener"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("removeListener"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("addEventListener"),
            2,
        )
        .function(
            NativeFunction::from_fn_ptr(noop_listener),
            JsString::from("removeEventListener"),
            2,
        )
        .build();

    Ok(JsValue::from(mql))
}
